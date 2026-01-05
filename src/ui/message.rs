use crate::app::InitResult;

#[derive(Debug, Clone)]
pub enum Message {
    // Initialization
    Initialized(Result<InitResult, String>),
    
    // Navigation
    NavigateUp,
    NavigateDown(usize),
    
    // No-op
    None,
}
