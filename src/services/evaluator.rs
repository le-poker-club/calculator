use std::collections::{HashMap, HashSet};

use async_trait::async_trait;
use chrono::Local;
use holdem_hand_evaluator::Hand;
use itertools::Itertools;

use crate::models::error_model;
use crate::models::model::{CalculateRatingReq, CalculateRatingRsp, CardsInfo, ClientRate};

// 可参见heads_up_win_frequency方法的compute_alive_cards方法。该方法可以直接找出可用的card
pub async fn evaluate(input: &String) -> u16 {
    let hand = input.parse::<Hand>().unwrap();
    let rank = hand.evaluate();
    return rank;
}
#[async_trait]
pub trait CalculateRating {
    async fn calculate_rating(&self, req: CalculateRatingReq) -> CalculateRatingRsp;
}

pub struct Evaluator {}

fn calculate_rating_valid(req: &CalculateRatingReq)->bool{
    let length = req.deal_cards.len() + req.clients.len() * 2;
    let demo = String::from("");
    let mut vec:Vec<&String> = vec![&demo;length];
    let mut i = 0;
    req.clients.iter().for_each(|x1| {
        vec[i] =  &x1.hands[0];
        vec[i+1] = &x1.hands[1];
        i += 2;
    });
    req.deal_cards.iter().for_each(|x2| {
        vec [i] = &x2;
        i += 1;
    });
    if vec.iter().duplicates().count() > 0{
       return false;
    }
    let empty = "".to_string();
    if vec.into_iter().contains(&empty) {
        return false;
    }
    return true;
}

#[async_trait]
// 对deal_cards为空的情况进行测试
impl CalculateRating for Evaluator {
    async fn calculate_rating(&self,req: CalculateRatingReq) -> CalculateRatingRsp {
        if !calculate_rating_valid(&req){
            return CalculateRatingRsp {
                code: error_model::ERROR_INVALID,
                clients_rate: vec![],
                msg: "req has duplicates or has empty string input".to_string(),
            };
        }
        let user_cards = convert(&req);
        if user_cards.len() < 2 {
            return CalculateRatingRsp {
                code: error_model::ERROR_INVALID,
                clients_rate: vec![],
                msg: "clients length invalid".to_string(),
            };
        }

        let board = if let Some(board) = req
            .deal_cards
            .iter()
            .map(|x| x.parse::<Hand>().unwrap())
            .reduce(|acc, e| acc + e)
        {
            board
        } else {
            Hand::new()
        };
        // 获取全部的hands和board的mark
        let mut mask = if let Some(mask) =
            user_cards
                .iter()
                .map(|x| x.hands.get_mask())
                .reduce(|acc, hand| {
                    return acc | hand;
                }) {
            mask
        } else {
            0
        };
        mask = mask | board.get_mask();
        // // 计算剩余的cards
        let alive_cards = compute_alive_cards(mask);
        let remain_card = 5 - board.len();
        let mut alive_card_index: Vec<i32> = Vec::new();
        (0..remain_card).for_each(|i| {
            alive_card_index.push(i as i32 - 1);
        });
        // 根据cards进行胜率计算
        let mut index = 0;
        let mut win_count_by_uid = HashMap::new();
        let mut used_cards: HashSet<i32> = HashSet::new();
        // 插入select宏
        tokio::select! {
            // 1s足够了
            _ = async{
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }=>{
            },
            _ = async{
                // 限制循环次数
                let mut loop_time:u32 = 0;
                while index < remain_card && loop_time < 11000{
                    loop_time+=1;
                    match get_index(
                        &mut alive_card_index,
                        index,
                        alive_cards.len() as i32,
                        &mut used_cards,
                    ) {
                        (true, _, _) => break,
                        (false, true, _) => {
                            index -= 1; // 跳到上一层
                            tokio::task::yield_now().await;
                            continue;
                        }
                        (false, false, _) => {
                            if index == remain_card - 1 {
                                let mut new_board = Hand::new();
                                // 获取待发的牌
                                (0..remain_card).for_each(|i| {
                                    new_board =
                                        new_board.add_card(alive_cards[alive_card_index[i] as usize]);
                                });
                                log::debug!("alive_card_index:{:?}", alive_card_index);
                                //
                                new_board = new_board + board;
                                let mut max_evaluate: u16 = 0;
                                let mut max_value_uids = Vec::new();
                                // 组合全部的牌，进行计算
                                user_cards.iter().for_each(|user_card| {
                                    let evaluate_hand = user_card.hands + new_board;
                                    let value = evaluate_hand.evaluate();
                                    if value > max_evaluate {
                                        max_value_uids.clear();
                                        max_value_uids.push(user_card.uid);
                                        max_evaluate = value;
                                    } else if value == max_evaluate {
                                        max_value_uids.push(user_card.uid);
                                    }
                                });
                                // 根据结果将对应的map值进行更新
                                for uid in max_value_uids {
                                    if let Some(x) = win_count_by_uid.get(uid) {
                                        win_count_by_uid.insert(uid, x + 1);
                                    } else {
                                        win_count_by_uid.insert(uid, 1u64);
                                    }
                                }
                                // 命中最后一层后，最后一层需要index+1继续
                                continue;
                            }
                        }
                    }
                    index += 1;
                }
            }=>{}
        }
        // 根据win_count_by_uid进行rating的计算
        let total_num: u64 = win_count_by_uid.iter().map(|(_, v)| v).sum();
        let mut calculate_rating_rsp = CalculateRatingRsp {
            code: 0,
            clients_rate: vec![],
            msg: "".to_string(),
        };
        for client in &req.clients {
            let uid = &client.uid;
            let uid_copy = uid.clone();
            if let Some(v) = win_count_by_uid.get(&uid) {
                calculate_rating_rsp.clients_rate.push(ClientRate {
                    uid: uid_copy,
                    rate: v * 1000 / total_num,
                })
            } else {
                calculate_rating_rsp.clients_rate.push(ClientRate {
                    uid: uid_copy,
                    rate: 0,
                })
            }
        }
        return calculate_rating_rsp;
    }
}

fn get_index(
    alive_card_index: &mut Vec<i32>,
    current_index: usize,
    alive_cards_len: i32,
    used_cards: &mut HashSet<i32>,
) -> (bool, bool, i32) {
    used_cards.remove(&alive_card_index[current_index]);
    alive_card_index[current_index] += 1;
    while let Some(_) = used_cards.get(&alive_card_index[current_index]) {
        alive_card_index[current_index] += 1;
    }
    used_cards.insert(alive_card_index[current_index]);
    let mut finish = false;
    let mut return_to_previous = false;
    if alive_card_index[current_index] >= alive_cards_len {
        // 选择的第0号元素已经超了
        if current_index == 0 {
            finish = true;
        } else {
            // 恢复从0开始
            alive_card_index[current_index] = -1;
            // 此轮结束，从上层继续
            return_to_previous = true;
        }
    }
    (finish, return_to_previous, alive_card_index[current_index])
}

fn convert(req: &CalculateRatingReq) -> Vec<CardsInfo> {
    let mut cards = Vec::new();
    req.clients.iter().for_each(|x| {
        let hand = x.hands[1].parse::<Hand>().unwrap() + x.hands[0].parse::<Hand>().unwrap();
        let card_info = CardsInfo {
            hands: hand,
            uid: &x.uid,
        };
        cards.push(card_info);
    });
    return cards;
}
/// number of ranks
pub const NUMBER_OF_RANKS: usize = 13;

/// number of ranks
pub const NUMBER_OF_CARDS: usize = 4 * NUMBER_OF_RANKS;

fn compute_alive_cards(mask: u64) -> Vec<usize> {
    let mut result = Vec::new();
    for i in 0..NUMBER_OF_CARDS {
        if (CARDS[i].1 & mask) == 0 {
            result.push(i);
        }
    }
    result
}

/// (card key, bit mask) of cards
#[rustfmt::skip]
pub const CARDS: [(u64, u64); NUMBER_OF_CARDS] = [
    /* 2c */ (RANK_BASES[0] + (SUIT_BASES[0] << SUIT_SHIFT), 0x1),
    /* 2d */ (RANK_BASES[0] + (SUIT_BASES[1] << SUIT_SHIFT), 0x10000),
    /* 2h */ (RANK_BASES[0] + (SUIT_BASES[2] << SUIT_SHIFT), 0x100000000),
    /* 2s */ (RANK_BASES[0] + (SUIT_BASES[3] << SUIT_SHIFT), 0x1000000000000),
    /* 3c */ (RANK_BASES[1] + (SUIT_BASES[0] << SUIT_SHIFT), 0x2),
    /* 3d */ (RANK_BASES[1] + (SUIT_BASES[1] << SUIT_SHIFT), 0x20000),
    /* 3h */ (RANK_BASES[1] + (SUIT_BASES[2] << SUIT_SHIFT), 0x200000000),
    /* 3s */ (RANK_BASES[1] + (SUIT_BASES[3] << SUIT_SHIFT), 0x2000000000000),
    /* 4c */ (RANK_BASES[2] + (SUIT_BASES[0] << SUIT_SHIFT), 0x4),
    /* 4d */ (RANK_BASES[2] + (SUIT_BASES[1] << SUIT_SHIFT), 0x40000),
    /* 4h */ (RANK_BASES[2] + (SUIT_BASES[2] << SUIT_SHIFT), 0x400000000),
    /* 4s */ (RANK_BASES[2] + (SUIT_BASES[3] << SUIT_SHIFT), 0x4000000000000),
    /* 5c */ (RANK_BASES[3] + (SUIT_BASES[0] << SUIT_SHIFT), 0x8),
    /* 5d */ (RANK_BASES[3] + (SUIT_BASES[1] << SUIT_SHIFT), 0x80000),
    /* 5h */ (RANK_BASES[3] + (SUIT_BASES[2] << SUIT_SHIFT), 0x800000000),
    /* 5s */ (RANK_BASES[3] + (SUIT_BASES[3] << SUIT_SHIFT), 0x8000000000000),
    /* 6c */ (RANK_BASES[4] + (SUIT_BASES[0] << SUIT_SHIFT), 0x10),
    /* 6d */ (RANK_BASES[4] + (SUIT_BASES[1] << SUIT_SHIFT), 0x100000),
    /* 6h */ (RANK_BASES[4] + (SUIT_BASES[2] << SUIT_SHIFT), 0x1000000000),
    /* 6s */ (RANK_BASES[4] + (SUIT_BASES[3] << SUIT_SHIFT), 0x10000000000000),
    /* 7c */ (RANK_BASES[5] + (SUIT_BASES[0] << SUIT_SHIFT), 0x20),
    /* 7d */ (RANK_BASES[5] + (SUIT_BASES[1] << SUIT_SHIFT), 0x200000),
    /* 7h */ (RANK_BASES[5] + (SUIT_BASES[2] << SUIT_SHIFT), 0x2000000000),
    /* 7s */ (RANK_BASES[5] + (SUIT_BASES[3] << SUIT_SHIFT), 0x20000000000000),
    /* 8c */ (RANK_BASES[6] + (SUIT_BASES[0] << SUIT_SHIFT), 0x40),
    /* 8d */ (RANK_BASES[6] + (SUIT_BASES[1] << SUIT_SHIFT), 0x400000),
    /* 8h */ (RANK_BASES[6] + (SUIT_BASES[2] << SUIT_SHIFT), 0x4000000000),
    /* 8s */ (RANK_BASES[6] + (SUIT_BASES[3] << SUIT_SHIFT), 0x40000000000000),
    /* 9c */ (RANK_BASES[7] + (SUIT_BASES[0] << SUIT_SHIFT), 0x80),
    /* 9d */ (RANK_BASES[7] + (SUIT_BASES[1] << SUIT_SHIFT), 0x800000),
    /* 9h */ (RANK_BASES[7] + (SUIT_BASES[2] << SUIT_SHIFT), 0x8000000000),
    /* 9s */ (RANK_BASES[7] + (SUIT_BASES[3] << SUIT_SHIFT), 0x80000000000000),
    /* Tc */ (RANK_BASES[8] + (SUIT_BASES[0] << SUIT_SHIFT), 0x100),
    /* Td */ (RANK_BASES[8] + (SUIT_BASES[1] << SUIT_SHIFT), 0x1000000),
    /* Th */ (RANK_BASES[8] + (SUIT_BASES[2] << SUIT_SHIFT), 0x10000000000),
    /* Ts */ (RANK_BASES[8] + (SUIT_BASES[3] << SUIT_SHIFT), 0x100000000000000),
    /* Jc */ (RANK_BASES[9] + (SUIT_BASES[0] << SUIT_SHIFT), 0x200),
    /* Jd */ (RANK_BASES[9] + (SUIT_BASES[1] << SUIT_SHIFT), 0x2000000),
    /* Jh */ (RANK_BASES[9] + (SUIT_BASES[2] << SUIT_SHIFT), 0x20000000000),
    /* Js */ (RANK_BASES[9] + (SUIT_BASES[3] << SUIT_SHIFT), 0x200000000000000),
    /* Qc */ (RANK_BASES[10] + (SUIT_BASES[0] << SUIT_SHIFT), 0x400),
    /* Qd */ (RANK_BASES[10] + (SUIT_BASES[1] << SUIT_SHIFT), 0x4000000),
    /* Qh */ (RANK_BASES[10] + (SUIT_BASES[2] << SUIT_SHIFT), 0x40000000000),
    /* Qs */ (RANK_BASES[10] + (SUIT_BASES[3] << SUIT_SHIFT), 0x400000000000000),
    /* Kc */ (RANK_BASES[11] + (SUIT_BASES[0] << SUIT_SHIFT), 0x800),
    /* Kd */ (RANK_BASES[11] + (SUIT_BASES[1] << SUIT_SHIFT), 0x8000000),
    /* Kh */ (RANK_BASES[11] + (SUIT_BASES[2] << SUIT_SHIFT), 0x80000000000),
    /* Ks */ (RANK_BASES[11] + (SUIT_BASES[3] << SUIT_SHIFT), 0x800000000000000),
    /* Ac */ (RANK_BASES[12] + (SUIT_BASES[0] << SUIT_SHIFT), 0x1000),
    /* Ad */ (RANK_BASES[12] + (SUIT_BASES[1] << SUIT_SHIFT), 0x10000000),
    /* Ah */ (RANK_BASES[12] + (SUIT_BASES[2] << SUIT_SHIFT), 0x100000000000),
    /* As */ (RANK_BASES[12] + (SUIT_BASES[3] << SUIT_SHIFT), 0x1000000000000000),
];

/// rank keys that guarantee a unique sum for every rank combination of 5-7 cards.
pub const RANK_BASES: [u64; NUMBER_OF_RANKS] = [
    0x000800, 0x002000, 0x024800, 0x025005, 0x03102e, 0x05f0e4, 0x13dc93, 0x344211, 0x35a068,
    0x377813, 0x378001, 0x378800, 0x380000,
];

/// suit keys (club, diamond, heart, spade)
pub const SUIT_BASES: [u64; 4] = [0x1000, 0x0100, 0x0010, 0x0001];

/// shift value for suit
pub const SUIT_SHIFT: usize = 48;
