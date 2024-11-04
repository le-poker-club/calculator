use holdem_hand_evaluator::Hand;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use uuid::Uuid;

// 定义一个线程本地变量，每个线程会有自己独立的 RefCell
thread_local! {
    pub static THREAD_LOCAL_DATA: RefCell<Uuid> = RefCell::new(Uuid::new_v4());
}

impl CalculateOutsReq {
    pub(crate) fn into_rating_req(&self) -> CalculateRatingReq {
        return CalculateRatingReq {
            clients: self.clients.clone(),
            deal_cards: self.deal_cards.clone(),
        };
    }
}

#[derive(Deserialize, Serialize)]
pub struct CalculateOutsReq {
    pub clients: Vec<UserCards>,
    pub deal_cards: Vec<String>, // 公共牌
}
#[derive(Deserialize, Serialize)]
pub struct CalculateOutsRsp {
    pub code: u32,
    pub outs: Vec<Outs>,
    pub msg: String,
}
#[derive(Deserialize, Serialize)]
pub struct Outs {
    pub cards: Vec<String>,
    pub uid: String,
}

#[derive(Deserialize, Serialize)]
pub struct CalculateRatingReq {
    pub clients: Vec<UserCards>,
    #[serde(default)]
    pub deal_cards: Vec<String>, // 公共牌
}
#[derive(Deserialize, Serialize)]
pub struct CalculateRatingRsp {
    pub code: u32,
    pub clients_rate: Vec<ClientRate>,
    pub msg: String,
}
#[derive(Deserialize, Serialize)]
pub struct ClientRate {
    pub uid: String,
    pub rate: u64, // 1000为分母
}

#[derive(Deserialize, Serialize, Clone)]
pub struct UserCards {
    pub hands: [String; 2], // 手牌
    pub uid: String,        // 用户uid
}

#[derive(Deserialize, Serialize)]
pub struct Info {
    pub(crate) username: String,
}

pub struct CardsInfo<'doc> {
    pub hands: Hand,
    pub uid: &'doc String,
}
