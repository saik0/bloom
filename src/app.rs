use crate::parse::rust::{TreeNode, get_node, get_children, update_node_text};
use crate::red::Zipper;
use crate::ui::{render_viewport, Message};
use iced::{Element, Task, Theme};
use surrealdb::{engine::local::Mem, Surreal};
use std::sync::Arc;
use surrealdb::engine::local::Db;

pub struct BloomApp {
    db: Option<Arc<Surreal<Db>>>,
    zipper: Zipper,
    status: String,

    // View cache
    focused_node: Option<TreeNode>,
    child_nodes: Vec<(String, TreeNode)>,

    // Editing state
    editing_node_id: Option<String>,
    edit_buffer: String,
}

#[derive(Debug, Clone)]
pub struct InitResult {
    pub db: Arc<Surreal<Db>>,
    pub root_id: String,
}

impl BloomApp {
    pub fn new() -> (Self, Task<Message>) {
        let app = Self {
            db: None,
            zipper: Zipper::new("placeholder".to_string()),
            status: "Initializing...".to_string(),
            focused_node: None,
            child_nodes: Vec::new(),
            editing_node_id: None,
            edit_buffer: String::new(),
        };

        (app, Task::perform(init_app(), Message::Initialized))
    }

    pub fn title(&self) -> String {
        "Bloom IDE - Phase 2A: Inline Editing".to_string()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Initialized(result) => {
                match result {
                    Ok(init) => {
                        self.db = Some(init.db.clone());
                        self.zipper = Zipper::new(init.root_id.clone());
                        self.status = "Ready ✅".to_string();

                        // Load initial view at root
                        let db = init.db.clone();
                        let root_id = init.root_id;
                        return Task::perform(
                            async move { load_focus(db, root_id).await },
                            Message::FocusLoaded
                        );
                    }
                    Err(e) => {
                        self.status = format!("Error: {}", e);
                    }
                }
                Task::none()
            }

            Message::FocusLoaded(result) => {
                match result {
                    Ok((id, node, children)) => {
                        self.focused_node = Some(node);
                        self.child_nodes = children;
                        self.status = format!(
                            "Depth {} | {} children",
                            self.zipper.depth(),
                            self.child_nodes.len()
                        );
                    }
                    Err(e) => {
                        self.status = format!("Load error: {}", e);
                    }
                }
                Task::none()
            }

            Message::NavigateUp => {
                if self.zipper.up() {
                    // Successfully moved up
                    if let Some(db) = &self.db {
                        let db = db.clone();
                        let focus_id = self.zipper.focus.clone();
                        self.status = "Navigating up...".to_string();
                        return Task::perform(
                            async move { load_focus(db, focus_id).await },
                            Message::FocusLoaded
                        );
                    }
                } else {
                    self.status = "Already at root".to_string();
                }
                Task::none()
            }

            Message::NavigateDown(child_id, child_index) => {
                // Move zipper down
                self.zipper.down(child_id.clone(), child_index);

                if let Some(db) = &self.db {
                    let db = db.clone();
                    self.status = "Navigating down...".to_string();
                    return Task::perform(
                        async move { load_focus(db, child_id).await },
                        Message::FocusLoaded
                    );
                }
                Task::none()
            }

            Message::StartEdit(node_id) => {
                // Find the node in child_nodes
                if let Some((_, node)) = self.child_nodes.iter().find(|(id, _)| *id == node_id) {
                    self.editing_node_id = Some(node_id);
                    self.edit_buffer = node.text.clone();
                    self.status = "Editing...".to_string();
                }
                Task::none()
            }

            Message::UpdateEditText(text) => {
                self.edit_buffer = text;
                Task::none()
            }

            Message::CommitEdit => {
                if let (Some(node_id), Some(db)) = (&self.editing_node_id, &self.db) {
                    let node_id_clone = node_id.clone();
                    let new_text = self.edit_buffer.clone();
                    let db = db.clone();
                    let focus_id = self.zipper.focus.clone();

                    // Clear editing state
                    self.editing_node_id = None;
                    self.edit_buffer.clear();
                    self.status = "Saving edit...".to_string();

                    // Update node in background, then refresh view
                    return Task::perform(
                        async move {
                            update_node_text(&db, &node_id_clone, new_text).await?;
                            load_focus(db, focus_id).await
                        },
                        Message::FocusLoaded
                    );
                }
                Task::none()
            }

            Message::CancelEdit => {
                self.editing_node_id = None;
                self.edit_buffer.clear();
                self.status = "Edit cancelled".to_string();
                Task::none()
            }

            Message::None => Task::none(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        if self.db.is_some() {
            render_viewport(
                &self.zipper,
                &self.focused_node,
                &self.child_nodes,
                &self.editing_node_id,
                &self.edit_buffer,
                &self.status,
            )
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
    let db = Surreal::new::<Mem>(())
        .await
        .map_err(|e| e.to_string())?;

    db.use_ns("bloom").use_db("bloom")
        .await
        .map_err(|e| e.to_string())?;

    let schema = include_str!("green/schema.surql");
    db.query(schema)
        .await
        .map_err(|e| e.to_string())?;

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

async fn load_focus(
    db: Arc<Surreal<Db>>,
    focus_id: String,
) -> Result<(String, TreeNode, Vec<(String, TreeNode)>), String> {
    let (id, node) = get_node(&db, &focus_id).await?;
    let children = get_children(&db, &focus_id).await?;
    Ok((id, node, children))
}

pub fn run() -> iced::Result {
    iced::application(BloomApp::new, BloomApp::update, BloomApp::view)
        .title(BloomApp::title)
        .theme(BloomApp::theme)
        .run()
}