//! Bloom IDE Application
//!
//! # Architecture
//!
//! ```text
//! rowan SyntaxNode (RED)     ←── Source of truth, editor state
//!        ↓
//!      Zipper               ←── Navigation cursor
//!        ↓
//!    UI renders             ←── Direct projection of rowan tree
//!        
//!        ↓ (async, background)
//!        
//!    SurrealDB (GREEN)      ←── Side-channel for analytics/RAG
//! ```
//!
//! The rowan tree IS the editor. SurrealDB is for queries rowan can't do.

use crate::parse;
use crate::red::Zipper;
use crate::green;
use crate::ui::{render_viewport, Message};
use iced::{Element, Task, Theme};
use ra_ap_syntax::SyntaxNode;

pub struct BloomApp {
    /// The rowan syntax tree - THIS IS THE SOURCE OF TRUTH
    _root: SyntaxNode,
    
    /// Navigation cursor into the rowan tree
    zipper: Zipper,
    
    /// Status message
    status: String,
}

impl BloomApp {
    pub fn new() -> (Self, Task<Message>) {
        // Parse initial source - rowan is the source of truth
        let source = r#"fn main() {
    println!("Hello, Bloom!");
    let x = 42;
}
"#;
        let root = parse::parse_rust(source);
        let zipper = Zipper::new(root.clone());
        
        // Extract projection data synchronously (SyntaxNode is !Send)
        let (nodes, root_hash) = green::extract_projection(&root);

        let app = Self {
            _root: root,
            zipper,
            status: "Rowan ready ✅".to_string(),
        };

        // Insert into DB in background (the extracted data IS Send)
        (app, Task::perform(
            init_side_channel(nodes, root_hash),
            |result| match result {
                Ok(_) => Message::ProjectionComplete(Ok("DB synced".to_string())),
                Err(e) => Message::ProjectionComplete(Err(e)),
            }
        ))
    }

    pub fn title(&self) -> String {
        "Bloom IDE - Rowan is Truth".to_string()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::NavigateUp => {
                if self.zipper.up() {
                    self.status = format!("Depth {}", self.zipper.depth());
                } else {
                    self.status = "Already at root".to_string();
                }
                Task::none()
            }

            Message::NavigateDown(child_index) => {
                if self.zipper.down(child_index) {
                    self.status = format!("Depth {}", self.zipper.depth());
                } else {
                    self.status = "No such child".to_string();
                }
                Task::none()
            }

            Message::NextSibling => {
                if self.zipper.next_sibling() {
                    self.status = "→ Next".to_string();
                } else {
                    self.status = "No next sibling".to_string();
                }
                Task::none()
            }

            Message::PrevSibling => {
                if self.zipper.prev_sibling() {
                    self.status = "← Prev".to_string();
                } else {
                    self.status = "No prev sibling".to_string();
                }
                Task::none()
            }

            Message::ProjectionComplete(result) => {
                match result {
                    Ok(msg) => self.status = format!("✅ {}", msg),
                    Err(e) => self.status = format!("⚠️ DB: {}", e),
                }
                Task::none()
            }

            Message::None => Task::none(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        render_viewport(&self.zipper, &self.status)
    }

    pub fn theme(&self) -> Theme {
        Theme::Dark
    }
}

/// Initialize the side-channel database and insert pre-extracted projection
async fn init_side_channel(
    nodes: Vec<green::projection::ProjectedNode>,
    root_hash: String,
) -> Result<(), String> {
    use surrealdb::{engine::local::Mem, Surreal};
    
    let db = Surreal::new::<Mem>(())
        .await
        .map_err(|e| e.to_string())?;

    db.use_ns("bloom").use_db("bloom")
        .await
        .map_err(|e| e.to_string())?;

    db.query(green::SCHEMA)
        .await
        .map_err(|e| e.to_string())?;

    // Insert pre-extracted projection
    green::insert_projection(&db, nodes, "main.rs".to_string(), root_hash).await?;

    Ok(())
}

pub fn run() -> iced::Result {
    iced::application(BloomApp::new, BloomApp::update, BloomApp::view)
        .title(BloomApp::title)
        .theme(BloomApp::theme)
        .run()
}
