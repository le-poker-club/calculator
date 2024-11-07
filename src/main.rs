use std::collections::HashMap;
use std::env::VarError;
use std::time::Duration;
use std::{env, panic};

use crate::models::model::THREAD_LOCAL_DATA;
use crate::utils::log::{log_error_debug, log_info_debug, log_info_display};
use actix_http;
use actix_http::body;
use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::middleware::{from_fn, Next};
use actix_web::web::Query;
use actix_web::{dev, web, App, Error, HttpServer};
use anyhow::anyhow;
use flexi_logger::{Age, Cleanup, Criterion, Duplicate, FileSpec, Naming, WriteMode};
use uuid::Uuid;

mod handlers;
mod models;
mod services;
mod utils;

fn panic_hook() {
    panic::set_hook(Box::new(|e| {
        log_error_debug("", &anyhow!("panic found:{:?}", e));
    }));
}

fn bytes_to_payload(buf: web::Bytes) -> dev::Payload {
    let (_, mut pl) = actix_http::h1::Payload::create(true);
    pl.unread_data(buf);
    dev::Payload::from(pl)
}

async fn mutate_body_type_with_extractors(
    string_body: String,
    query: Query<HashMap<String, String>>,
    mut req: ServiceRequest,
    next: Next<impl MessageBody + 'static>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let my_uuid = Uuid::new_v4();
    THREAD_LOCAL_DATA.set(my_uuid);
    log_info_display("req url", req.uri());
    log_info_display("req body is", &string_body);
    log_info_debug("req query string", &query);
    req.set_payload(bytes_to_payload(web::Bytes::from(string_body)));
    let res = next.call(req).await?;
    let (req, res) = res.into_parts();
    let (empty_rsp, rsp_body) = res.into_parts();
    let rsp_body_bytes = body::to_bytes(rsp_body).await.ok().unwrap();
    log_info_display(
        "rsp body is",
        &String::from_utf8(rsp_body_bytes.to_vec()).unwrap(),
    );
    let new_rsp = empty_rsp.set_body(rsp_body_bytes);
    let service_rsp = ServiceResponse::new(req, new_rsp);
    Ok(service_rsp)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    panic_hook();
    let mut log_dir = "";
    match env::var("PROFILE") {
        Ok(_) => log_dir = "/data/logs",
        Err(_) => log_dir = "./logs",
    }
    let mut ip:String;
    match env::var("POD_IP") {
        Ok(v) => ip = v,
        Err(_) => ip = "".to_string(),
    }
    let _e = flexi_logger::Logger::try_with_str("info")
        .unwrap()
        .log_to_file(FileSpec::default().basename(format!("{}-{}","calculate",ip)).directory(log_dir))
        .duplicate_to_stdout(Duplicate::Debug)
        .append()
        .write_mode(WriteMode::Async)
        .rotate(
            Criterion::AgeOrSize(Age::Day, 500 * 1024 * 1024), // 每天轮转一次日志或500M轮转一次
            Naming::Timestamps,
            Cleanup::KeepLogAndCompressedFiles(1, 30), // 保留30天的日志，1天前日志压缩
        )
        .format(flexi_logger::opt_format)
        .start()
        .expect("error");
    // test();
    HttpServer::new(|| {
        App::new()
            .wrap(from_fn(mutate_body_type_with_extractors))
            .wrap(from_fn(timeout_2secs))
            .service(handlers::controller::submit)
            .service(handlers::controller::hello)
            .service(handlers::controller::calculate_outs)
    })
    .client_request_timeout(Duration::from_secs(1))
    .bind(("0.0.0.0", 8090))?
    .run()
    .await
}

async fn timeout_2secs(
    req: ServiceRequest,
    next: Next<impl MessageBody + 'static>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    match tokio::time::timeout(Duration::from_secs(2), next.call(req)).await {
        Ok(res) => res,
        Err(_err) => Err(actix_web::error::ErrorRequestTimeout("")),
    }
}
