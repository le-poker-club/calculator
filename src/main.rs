use std::collections::HashMap;
use std::fmt::Debug;
use std::io::Read;
use std::panic;
use std::time::Duration;

use actix_web::{web, App, HttpServer, Error, HttpMessage, dev};
use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::middleware::{from_fn, Logger, Next};
use actix_web::web::{BytesMut, Payload, Query};
use anyhow::anyhow;
use flexi_logger::{Age, Cleanup, Criterion, Duplicate, FileSpec, Naming, WriteMode};
use holdem_hand_evaluator::{heads_up_win_frequency, Hand};
use log::info;
use serde_json::Value;
use actix_http;
use actix_http::body;

mod handlers;
mod models;
mod services;

fn panic_hook() {
    panic::set_hook(Box::new(|e| {
        log::error!("{:?}", anyhow!("panic found:{:?}", e));
    }));
}
fn test() {
    panic!("hello2");
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
    println!("body is: {string_body}");
    println!("query string: {query:?}");
    req.set_payload(bytes_to_payload(web::Bytes::from(string_body)));
    let res = next.call(req).await?;
    let (req, res) = res.into_parts();
    let (empty_rsp, rsp_body) = res.into_parts();
    let rsp_body_bytes = body::to_bytes(rsp_body).await.ok().unwrap();
    println!("{:?}", rsp_body_bytes);
    let new_rsp = empty_rsp.set_body(rsp_body_bytes);
    let service_rsp = ServiceResponse::new(req, new_rsp);
    Ok(service_rsp)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    panic_hook();
    let _e = flexi_logger::Logger::try_with_str("info")
        .unwrap()
        .log_to_file(
            FileSpec::default()
                .basename("calculate")
                .directory("./logs"),
        )
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
            .service(handlers::controller::submit)
            .service(handlers::controller::hello)
    })
    .client_request_timeout(Duration::from_secs(1))
    .bind(("127.0.0.1", 8090))?
    .run()
    .await
}
