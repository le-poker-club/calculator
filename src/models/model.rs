use holdem_hand_evaluator::Hand;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct CalculateRatingReq {
    pub clients: Vec<UserCards>,
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

#[derive(Deserialize, Serialize)]
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
