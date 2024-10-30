#[cfg(test)]
mod tests {
    use crate::models::model::{CalculateRatingReq, CalculateRatingRsp, UserCards};
    use crate::services::evaluator::{CalculateRating, Evaluator};

    #[tokio::test]
    async fn test_calculate_rating() {
        let mut req = CalculateRatingReq {
            clients: vec![],
            deal_cards: vec![],
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
        // req.deal_cards.push("Ks".to_string());
        // req.deal_cards.push("Kc".to_string());
        let evaluator = Evaluator {};
        let rsp: CalculateRatingRsp = evaluator.calculate_rating(req).await;
    }
}
