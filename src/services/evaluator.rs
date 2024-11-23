use std::collections::{HashMap, HashSet};
use std::string::ToString;

use async_trait::async_trait;
use holdem_hand_evaluator::Hand;
use itertools::Itertools;
use rand::{thread_rng, Rng};

use crate::models::error_model;
use crate::models::model::{
    CalculateOutsReq, CalculateOutsRsp, CalculateRatingReq, CalculateRatingRsp, CardsInfo,
    ClientRate, Outs,
};
use crate::utils::log::log_info_debug;

#[async_trait]
pub trait CalculateRating {
    async fn calculate_rating(&self, req: CalculateRatingReq) -> CalculateRatingRsp;
    async fn calculate_outs(&self, req: CalculateOutsReq) -> CalculateOutsRsp;
}

pub struct Evaluator {}

pub fn calculate_rating_valid(req: &CalculateRatingReq) -> (bool, Vec<CardsInfo>) {
    let length = req.deal_cards.len() + req.clients.len() * 2;
    let demo = String::from("");
    let mut vec: Vec<&String> = vec![&demo; length];
    let mut i = 0;
    req.clients.iter().for_each(|x1| {
        vec[i] = &x1.hands[0];
        vec[i + 1] = &x1.hands[1];
        i += 2;
    });
    req.deal_cards.iter().for_each(|x2| {
        vec[i] = &x2;
        i += 1;
    });
    if vec.iter().duplicates().count() > 0 {
        return (false, vec![]);
    }
    let empty = "".to_string();
    if vec.into_iter().contains(&empty) {
        return (false, vec![]);
    }
    let user_cards = convert(&req);
    if user_cards.len() < 2 {
        return (false, vec![]);
    }
    return (true, user_cards);
}

impl Evaluator {
    fn get_board_and_alive_cards(
        &self,
        deal_cards: &Vec<String>,
        dead_cards: &Vec<String>,
        user_cards: &Vec<CardsInfo>,
    ) -> (Hand, Vec<usize>) {
        let board = if let Some(board) = deal_cards
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
        let dead_cards_hands = if let Some(dead_card) = dead_cards
            .iter()
            .map(|x| x.parse::<Hand>().unwrap())
            .reduce(|acc, e| acc + e)
        {
            dead_card
        } else {
            Hand::new()
        };
        mask = mask | board.get_mask() | dead_cards_hands.get_mask();
        // // 计算剩余的cards
        let alive_cards = compute_alive_cards(mask);
        (board, alive_cards)
    }
}

#[async_trait]
// win的牌放在前面，draw的牌放在后面
impl CalculateRating for Evaluator {
    async fn calculate_outs(&self, req: CalculateOutsReq) -> CalculateOutsRsp {
        let temp = req.into_rating_req();
        let (valid, user_cards) = calculate_rating_valid(&temp);
        if !valid || req.deal_cards.len() < 3 {
            return CalculateOutsRsp {
                code: error_model::ERROR_INVALID,
                outs: vec![],
                msg: "eq has duplicates or has empty string input,or client.len is lt 2 or req deal cards should gt 2"
                    .to_string(),
            };
        }
        let mut outs_by_uid = HashMap::new();
        let mut draw_outs_by_uid = HashMap::new();
        for card_info in &user_cards {
            outs_by_uid.insert(card_info.uid, vec![]);
            draw_outs_by_uid.insert(card_info.uid, vec![]);
        }
        if req.deal_cards.len() < 5 {
            let (board, alive_cards) =
                self.get_board_and_alive_cards(&req.deal_cards, &req.dead_cards, &user_cards);
            let mut i = 0;
            while i < alive_cards.len() {
                let mut new_board = Hand::new();
                new_board = new_board.add_card(alive_cards[i]);
                new_board = new_board + board;
                let mut max_evaluate: u16 = 0;
                let mut max_value_uids = vec![];
                let mut draw_value_uids = vec![];
                user_cards.iter().for_each(|user_card| {
                    let evaluate_hand = user_card.hands + new_board;
                    let value = evaluate_hand.evaluate();
                    if value > max_evaluate {
                        max_value_uids.clear();
                        max_value_uids.push(user_card.uid);
                        max_evaluate = value;
                        draw_value_uids.clear();
                    } else if value == max_evaluate {
                        draw_value_uids.extend(max_value_uids.iter());
                        max_value_uids.clear();
                        draw_value_uids.push(user_card.uid);
                    }
                });
                for uid in max_value_uids {
                    outs_by_uid.get_mut(uid).unwrap().push(alive_cards[i]);
                }
                for uid in draw_value_uids {
                    draw_outs_by_uid.get_mut(uid).unwrap().push(alive_cards[i]);
                }
                i += 1;
            }
            draw_outs_by_uid.into_iter().for_each(|(uid, draw_outs)| {
                outs_by_uid.get_mut(uid).unwrap().extend(draw_outs);
            });
        }

        let mut return_outs = vec![];
        for (uid, outs) in outs_by_uid.into_iter() {
            let mut outs_string = vec![];
            for card in outs {
                outs_string.push(CARDSSTRING[card].to_string());
            }
            let out = Outs {
                cards: outs_string,
                uid: uid.to_string(),
            };
            return_outs.push(out);
        }
        return CalculateOutsRsp {
            code: 0,
            outs: return_outs,
            msg: "".to_string(),
        };
    }
    async fn calculate_rating(&self, req: CalculateRatingReq) -> CalculateRatingRsp {
        let (valid, user_cards) = calculate_rating_valid(&req);
        if !valid {
            return CalculateRatingRsp {
                code: error_model::ERROR_INVALID,
                clients_rate: vec![],
                msg: "req has duplicates or has empty string input,or client.len is lt 2"
                    .to_string(),
            };
        }
        let (board, alive_cards) =
            self.get_board_and_alive_cards(&req.deal_cards, &req.dead_cards, &user_cards);
        let remain_card = 5 - board.len();
        let mut alive_card_index: Vec<i32> = Vec::new();
        (0..remain_card).for_each(|i| {
            alive_card_index.push(i as i32 - 1);
        });
        // 根据cards进行胜率计算
        let mut index = 0;
        let mut win_count_by_uid = HashMap::new();
        let mut draw_count_by_uid = HashMap::new();
        let mut draw_count: u64 = 0;
        // 已经出过的公共牌
        let mut used_cards: HashSet<i32> = HashSet::new();
        let max_loop: u32 = 11000;
        // 如果remain_card >= 3 采用随机法直接计算
        if remain_card >= 3 {
            let mut rng = thread_rng();
            let mut loop_time: u32 = 0;
            while loop_time < max_loop {
                let mut new_board = Hand::new();
                new_board += board;
                let mut i = 0;
                while i < remain_card {
                    let random_number = rng.gen_range(0..alive_cards.len());
                    if new_board.contains(alive_cards[random_number]) {
                        continue;
                    }
                    new_board = new_board.add_card(alive_cards[random_number]);
                    i += 1;
                }
                add_to_win_count(
                    &user_cards,
                    &mut draw_count_by_uid,
                    &mut draw_count,
                    new_board,
                    &mut win_count_by_uid,
                );
                loop_time += 1;
            }
        } else {
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
                    let mut can_remove_first = false;
                    while (index < remain_card ||remain_card == 0)  && loop_time < max_loop{
                        loop_time+=1;
                        let mut index_get=(false,false);
                        if remain_card != 0{
                            index_get =  get_index(
                            &mut alive_card_index,
                            index,
                            alive_cards.len() as i32,
                            &mut used_cards,
                            &can_remove_first,
                        );
                        }

                        match index_get{
                            (true, _) => break,
                            (false, true) => {
                                index -= 1; // 跳到上一层
                                tokio::task::yield_now().await;
                                continue;
                            }
                            (false, false) => {
                                if  remain_card == 0 || index == remain_card - 1 {
                                    can_remove_first = true;
                                    let mut new_board = Hand::new();
                                    // let mut debg = vec![];
                                    // 获取待发的牌
                                    (0..remain_card).for_each(|i| {
                                        new_board =
                                            new_board.add_card(alive_cards[alive_card_index[i] as usize]);
                                        // debg.push(alive_card_index[i] as usize);
                                    });
                                    // log_debug_debug("alive_card_index", &debg);
                                    new_board = new_board + board;
                                    add_to_win_count(&user_cards,&mut draw_count_by_uid,&mut draw_count,new_board,&mut win_count_by_uid);
                                    if remain_card == 0{
                                        loop_time = max_loop+1;
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
        }
        // 根据win_count_by_uid进行rating的计算
        let mut total_num: u64 = win_count_by_uid.iter().map(|(_, v)| v).sum();
        total_num += draw_count;
        let mut calculate_rating_rsp = CalculateRatingRsp {
            code: 0,
            clients_rate: vec![],
            msg: "".to_string(),
        };
        log_info_debug("draw", &draw_count_by_uid);
        log_info_debug("win", &win_count_by_uid);
        for client in &req.clients {
            let uid = &client.uid;
            let uid_copy = uid.clone();
            let zero_u64: u64 = 0;
            let draw_count_value = draw_count_by_uid.get(uid).unwrap_or_else(|| &zero_u64);
            let win_count_value = win_count_by_uid.get(uid).unwrap_or_else(|| &zero_u64);
            calculate_rating_rsp.clients_rate.push(ClientRate {
                uid: uid_copy,
                rate: (win_count_value * 10000 + draw_count_value * 5000) / total_num,
            })
        }
        return calculate_rating_rsp;
    }
}

fn add_to_win_count(
    user_cards: &Vec<CardsInfo>,
    draw_count_by_uid: &mut HashMap<String, u64>,
    draw_count: &mut u64,
    new_board: Hand,
    win_count_by_uid: &mut HashMap<String, u64>,
) {
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
    let mut temp = win_count_by_uid;
    if max_value_uids.len() > 1 {
        temp = draw_count_by_uid;
        *draw_count += 1;
    }
    for uid in max_value_uids {
        if let Some(x) = temp.get(uid) {
            temp.insert(uid.clone(), x + 1);
        } else {
            temp.insert(uid.clone(), 1u64);
        }
    }
}

fn get_index(
    alive_card_index: &mut Vec<i32>,
    current_index: usize,
    alive_cards_len: i32,
    used_cards: &mut HashSet<i32>,
    can_remove: &bool,
) -> (bool, bool) {
    // 第一次的时候不能remove
    if *can_remove && alive_card_index[current_index] >= 0 {
        used_cards.remove(&alive_card_index[current_index]);
    }

    alive_card_index[current_index] += 1;
    // 在alive_card_index内已经存在对应的牌了
    while alive_card_index[current_index] < alive_cards_len {
        if let Some(_) = used_cards.get(&alive_card_index[current_index]) {
            alive_card_index[current_index] += 1;
            continue;
        }
        break;
    }
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
    } else {
        used_cards.insert(alive_card_index[current_index]);
    }
    (finish, return_to_previous)
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

pub const CARDSSTRING: [&str; NUMBER_OF_CARDS] = [
    "2s", "2h", "2c", "2d", "3s", "3h", "3c", "3d", "4s", "4h", "4c", "4d", "5s", "5h", "5c", "5d",
    "6s", "6h", "6c", "6d", "7s", "7h", "7c", "7d", "8s", "8h", "8c", "8d", "9s", "9h", "9c", "9d",
    "Ts", "Th", "Tc", "Td", "Js", "Jh", "Jc", "Jd", "Qs", "Qh", "Qc", "Qd", "Ks", "Kh", "Kc", "Kd",
    "As", "Ah", "Ac", "Ad",
];

/// (card key, bit mask) of cards
#[rustfmt::skip]
pub const CARDS: [(u64, u64); NUMBER_OF_CARDS] = [
    /* 2s */ (RANK_BASES[0] + (SUIT_BASES[3] << SUIT_SHIFT), 0x1000000000000),
    /* 2h */ (RANK_BASES[0] + (SUIT_BASES[2] << SUIT_SHIFT), 0x100000000),
    /* 2c */ (RANK_BASES[0] + (SUIT_BASES[0] << SUIT_SHIFT), 0x1),
    /* 2d */ (RANK_BASES[0] + (SUIT_BASES[1] << SUIT_SHIFT), 0x10000),
    /* 3s */ (RANK_BASES[1] + (SUIT_BASES[3] << SUIT_SHIFT), 0x2000000000000),
    /* 3h */ (RANK_BASES[1] + (SUIT_BASES[2] << SUIT_SHIFT), 0x200000000),
    /* 3c */ (RANK_BASES[1] + (SUIT_BASES[0] << SUIT_SHIFT), 0x2),
    /* 3d */ (RANK_BASES[1] + (SUIT_BASES[1] << SUIT_SHIFT), 0x20000),
    /* 4s */ (RANK_BASES[2] + (SUIT_BASES[3] << SUIT_SHIFT), 0x4000000000000),
    /* 4h */ (RANK_BASES[2] + (SUIT_BASES[2] << SUIT_SHIFT), 0x400000000),
    /* 4c */ (RANK_BASES[2] + (SUIT_BASES[0] << SUIT_SHIFT), 0x4),
    /* 4d */ (RANK_BASES[2] + (SUIT_BASES[1] << SUIT_SHIFT), 0x40000),
    /* 5s */ (RANK_BASES[3] + (SUIT_BASES[3] << SUIT_SHIFT), 0x8000000000000),
    /* 5h */ (RANK_BASES[3] + (SUIT_BASES[2] << SUIT_SHIFT), 0x800000000),
    /* 5c */ (RANK_BASES[3] + (SUIT_BASES[0] << SUIT_SHIFT), 0x8),
    /* 5d */ (RANK_BASES[3] + (SUIT_BASES[1] << SUIT_SHIFT), 0x80000),
    /* 6s */ (RANK_BASES[4] + (SUIT_BASES[3] << SUIT_SHIFT), 0x10000000000000),
    /* 6h */ (RANK_BASES[4] + (SUIT_BASES[2] << SUIT_SHIFT), 0x1000000000),
    /* 6c */ (RANK_BASES[4] + (SUIT_BASES[0] << SUIT_SHIFT), 0x10),
    /* 6d */ (RANK_BASES[4] + (SUIT_BASES[1] << SUIT_SHIFT), 0x100000),
    /* 7s */ (RANK_BASES[5] + (SUIT_BASES[3] << SUIT_SHIFT), 0x20000000000000),
    /* 7h */ (RANK_BASES[5] + (SUIT_BASES[2] << SUIT_SHIFT), 0x2000000000),
    /* 7c */ (RANK_BASES[5] + (SUIT_BASES[0] << SUIT_SHIFT), 0x20),
    /* 7d */ (RANK_BASES[5] + (SUIT_BASES[1] << SUIT_SHIFT), 0x200000),
    /* 8s */ (RANK_BASES[6] + (SUIT_BASES[3] << SUIT_SHIFT), 0x40000000000000),
    /* 8h */ (RANK_BASES[6] + (SUIT_BASES[2] << SUIT_SHIFT), 0x4000000000),
    /* 8c */ (RANK_BASES[6] + (SUIT_BASES[0] << SUIT_SHIFT), 0x40),
    /* 8d */ (RANK_BASES[6] + (SUIT_BASES[1] << SUIT_SHIFT), 0x400000),
    /* 9s */ (RANK_BASES[7] + (SUIT_BASES[3] << SUIT_SHIFT), 0x80000000000000),
    /* 9h */ (RANK_BASES[7] + (SUIT_BASES[2] << SUIT_SHIFT), 0x8000000000),
    /* 9c */ (RANK_BASES[7] + (SUIT_BASES[0] << SUIT_SHIFT), 0x80),
    /* 9d */ (RANK_BASES[7] + (SUIT_BASES[1] << SUIT_SHIFT), 0x800000),
    /* Ts */ (RANK_BASES[8] + (SUIT_BASES[3] << SUIT_SHIFT), 0x100000000000000),
    /* Th */ (RANK_BASES[8] + (SUIT_BASES[2] << SUIT_SHIFT), 0x10000000000),
    /* Tc */ (RANK_BASES[8] + (SUIT_BASES[0] << SUIT_SHIFT), 0x100),
    /* Td */ (RANK_BASES[8] + (SUIT_BASES[1] << SUIT_SHIFT), 0x1000000),
    /* Js */ (RANK_BASES[9] + (SUIT_BASES[3] << SUIT_SHIFT), 0x200000000000000),
    /* Jh */ (RANK_BASES[9] + (SUIT_BASES[2] << SUIT_SHIFT), 0x20000000000),
    /* Jc */ (RANK_BASES[9] + (SUIT_BASES[0] << SUIT_SHIFT), 0x200),
    /* Jd */ (RANK_BASES[9] + (SUIT_BASES[1] << SUIT_SHIFT), 0x2000000),
    /* Qs */ (RANK_BASES[10] + (SUIT_BASES[3] << SUIT_SHIFT), 0x400000000000000),
    /* Qh */ (RANK_BASES[10] + (SUIT_BASES[2] << SUIT_SHIFT), 0x40000000000),
    /* Qc */ (RANK_BASES[10] + (SUIT_BASES[0] << SUIT_SHIFT), 0x400),
    /* Qd */ (RANK_BASES[10] + (SUIT_BASES[1] << SUIT_SHIFT), 0x4000000),
    /* Ks */ (RANK_BASES[11] + (SUIT_BASES[3] << SUIT_SHIFT), 0x800000000000000),
    /* Kh */ (RANK_BASES[11] + (SUIT_BASES[2] << SUIT_SHIFT), 0x80000000000),
    /* Kc */ (RANK_BASES[11] + (SUIT_BASES[0] << SUIT_SHIFT), 0x800),
    /* Kd */ (RANK_BASES[11] + (SUIT_BASES[1] << SUIT_SHIFT), 0x8000000),
    /* As */ (RANK_BASES[12] + (SUIT_BASES[3] << SUIT_SHIFT), 0x1000000000000000),
    /* Ah */ (RANK_BASES[12] + (SUIT_BASES[2] << SUIT_SHIFT), 0x100000000000),
    /* Ac */ (RANK_BASES[12] + (SUIT_BASES[0] << SUIT_SHIFT), 0x1000),
    /* Ad */ (RANK_BASES[12] + (SUIT_BASES[1] << SUIT_SHIFT), 0x10000000),
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
