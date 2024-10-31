use holdem_hand_evaluator::Hand;
use crate::models::model::THREAD_LOCAL_DATA;

pub  fn log_info_display<T: std::fmt::Display>(input: &str,obj:&T) {
    THREAD_LOCAL_DATA.with_borrow(|v|log::info!("{}:{}(trace_id:{})", input,obj,v));
}
pub  fn log_info_debug<T: std::fmt::Debug>(input: &str,obj:&T) {
    THREAD_LOCAL_DATA.with_borrow(|v|log::info!("{}:{:?}(trace_id:{})", input,obj,v));
}
pub  fn log_debug_display<T: std::fmt::Display>(input: &str,obj:&T) {
    THREAD_LOCAL_DATA.with_borrow(|v|log::debug!("{}:{}(trace_id:{})", input,obj,v));
}
pub  fn log_debug_debug<T: std::fmt::Debug>(input: &str,obj:&T) {
    THREAD_LOCAL_DATA.with_borrow(|v|log::debug!("{}:{:?}(trace_id:{})", input,obj,v));
}