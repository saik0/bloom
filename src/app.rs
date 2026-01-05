use crate::core::NodeId;
use crate::red::Zipper;
use crate::ui::{render_viewport, Message};
use iced::{Element, Task, Theme};
use surrealdb::{engine::local::Db, Surreal};
use std::sync::Arc;
use surrealdb::engine::local::Mem;

pub struct BloomApp {
    db: Option<Arc<Surreal<Db>>>,
    zipper: Zipper,
    status: String,
}

#[derive(Debug, Clone)]
pub struct InitResult {
    pub db: Arc<Surreal<Db>>,
    pub root_id: NodeId,
}

impl BloomApp {
    pub fn new() -> (Self, Task<Message>) {
        let app = Self {
            db: None,
            zipper: Zipper::new(NodeId::root("placeholder")),
            status: "Initializing...".to_string(),
        };
        
        (app, Task::perform(init_app(), Message::Initialized))
    }

    pub fn title(&self) -> String {
        "Bloom IDE - Phase 1 MVP".to_string()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Initialized(result) => {
                match result {
                    Ok(init) => {
                        self.db = Some(init.db);
                        self.zipper = Zipper::new(init.root_id);
                        self.status = "Ready ✅".to_string();
                    }
                    Err(e) => {
                        self.status = format!("Error: {}", e);
                    }
                }
                Task::none()
            }
            Message::NavigateUp => {
                if self.zipper.up() {
                    self.status = "Navigated up".to_string();
                }
                Task::none()
            }
            Message::NavigateDown(_index) => {
                // TODO: Implement child navigation
                Task::none()
            }
            Message::None => Task::none(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        if let Some(db) = &self.db {
            render_viewport(db, &self.zipper)
        } else {
            iced::widget::container(
                iced::widget::text(&self.status)
                    .size(20)
            )
            .center(iced::Fill)
            .into()
        }
    }

    pub fn theme(&self) -> Theme {
        Theme::Dark
    }
}

async fn init_app() -> Result<InitResult, String> {
    // Initialize SurrealDB
    let db = Surreal::new::<Mem>(())
        .await
        .map_err(|e| e.to_string())?;
    
    db.use_ns("bloom").use_db("bloom")
        .await
        .map_err(|e| e.to_string())?;
    
    // Load schema
    let schema = include_str!("green/schema.surql");
    db.query(schema)
        .await
        .map_err(|e| e.to_string())?;
    
    // Parse a simple test file
    let source = r#"fn main() {
    println!("Hello, Bloom!");
    let x = 42;
}
"#;
    
    let root_id = crate::parse::parse_rust(&db, source, "test.rs")
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(InitResult {
        db: Arc::new(db),
        root_id,
    })
}

pub fn run() -> iced::Result {
    iced::application(BloomApp::new, BloomApp::update, BloomApp::view)
        .title(BloomApp::title)
        .theme(BloomApp::theme)
        .run()
}
