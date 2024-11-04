use actix_web::{get, post, web, Responder};

use crate::models::model::{CalculateOutsReq, CalculateOutsRsp, CalculateRatingReq, CalculateRatingRsp};
use crate::services::evaluator::{calculate_rating_valid, CalculateRating, Evaluator};

// 计算胜率
#[post("/calculate_rating")]
pub async fn submit(req: web::Json<CalculateRatingReq>) -> web::Json<CalculateRatingRsp> {
    let evaluator = Evaluator {};
    let rsp: CalculateRatingRsp = evaluator.calculate_rating(req.into_inner()).await;
    return web::Json(rsp);
}

#[post("/calculate_outs")]
pub async fn calculate_outs(req: web::Json<CalculateOutsReq>) -> web::Json<CalculateOutsRsp> {
    let evaluator = Evaluator {};
    let rsp: CalculateOutsRsp = evaluator.calculate_outs(req.into_inner()).await;
    return web::Json(rsp);
}

#[get("/hello")]
pub async fn hello() -> impl Responder {
    return "ok";
}
