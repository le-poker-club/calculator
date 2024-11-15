#[cfg(test)]
mod tests {
    use crate::models::model::{
        CalculateOutsReq, CalculateOutsRsp, CalculateRatingReq, CalculateRatingRsp, UserCards,
    };
    use crate::services::evaluator::{CalculateRating, Evaluator};

    #[tokio::test]
    async fn test_calculate_rating() {
        let mut req = CalculateRatingReq {
            clients: vec![],
            deal_cards: vec![],
            dead_cards: vec![],
        };
        req.clients.push(UserCards {
            hands: ["As".to_string(), "Ks".to_string()],
            uid: "1".to_string(),
        });
        req.clients.push(UserCards {
            hands: ["2s".to_string(), "Ts".to_string()],
            uid: "2".to_string(),
        });
        // req.deal_cards.push("Ac".to_string());
        // req.deal_cards.push("Js".to_string());
        // req.deal_cards.push("Kh".to_string());
        // req.deal_cards.push("Kc".to_string());
        // req.deal_cards.push("Kd".to_string());
        let evaluator = Evaluator {};
        let rsp: CalculateRatingRsp = evaluator.calculate_rating(req).await;
    }

    /**
        1	16.72%
        2	96.01%
        3	0.66%
    */
    #[tokio::test]
    async fn test_calculate_rating2() {
        let mut req = CalculateRatingReq {
            clients: vec![],
            deal_cards: vec![],
        };
        req.clients.push(UserCards {
            hands: ["3c".to_string(), "8c".to_string()],
            uid: "1".to_string(),
        });
        req.clients.push(UserCards {
            hands: ["Td".to_string(), "8d".to_string()],
            uid: "2".to_string(),
        });
        req.clients.push(UserCards {
            hands: ["Qc".to_string(), "5h".to_string()],
            uid: "3".to_string(),
        });
        req.deal_cards.push("6h".to_string());
        req.deal_cards.push("9s".to_string());
        req.deal_cards.push("7c".to_string());
        // req.deal_cards.push("Kc".to_string());
        // req.deal_cards.push("Kd".to_string());
        let evaluator = Evaluator {};
        let rsp: CalculateRatingRsp = evaluator.calculate_rating(req).await;
    }
}
