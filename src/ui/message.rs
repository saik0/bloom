#[derive(Debug, Clone)]
pub enum Message {
    // Navigation - these operate on the rowan tree directly
    NavigateUp,
    NavigateDown(usize),  // child index
    NextSibling,
    PrevSibling,

    // Projection status (background task)
    ProjectionComplete(Result<String, String>),
    
    // No-op
    None,
}
