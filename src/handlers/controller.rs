use actix_web::{get, post, web, Responder};

use crate::models::model::{CalculateRatingReq, CalculateRatingRsp};
use crate::services::evaluator::{CalculateRating, Evaluator};

// 计算胜率
#[post("/calculate_rating")]
pub async fn submit(req: web::Json<CalculateRatingReq>) -> web::Json<CalculateRatingRsp> {
    let evaluator = Evaluator {};
    let rsp: CalculateRatingRsp = evaluator.calculate_rating(req.into_inner()).await;
    return web::Json(rsp);
}

#[get("/hello")]
pub async fn hello() -> impl Responder {
    return "ok";
}
