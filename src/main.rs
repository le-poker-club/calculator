use std::panic;
use std::time::Duration;

use actix_web::{web, App, HttpServer};
use anyhow::anyhow;
use flexi_logger::{Age, Cleanup, Criterion, Duplicate, FileSpec, Naming, WriteMode};
use holdem_hand_evaluator::{heads_up_win_frequency, Hand};
use log::info;

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
            .service(handlers::controller::submit)
            .service(handlers::controller::hello)
    })
    .client_request_timeout(Duration::from_secs(1))
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
