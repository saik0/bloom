use crate::parse::rust::TreeNode;
use crate::red::Zipper;
use crate::ui::Message;
use iced::widget::{button, column, container, row, scrollable, text, text_input, Column, Row};
use iced::{Border, Color, Element, Fill, Length, Theme};

pub fn render_viewport(
    zipper: &Zipper,
    focused_node: &Option<TreeNode>,
    child_nodes: &[(String, TreeNode)],
    editing_node_id: &Option<String>,
    edit_buffer: &str,
    status: &str,
) -> Element<'static, Message> {
    let breadcrumb = render_breadcrumb(zipper);

    // Up button - only enabled if we can go up
    let can_go_up = zipper.depth() > 0;

    let up_button = if can_go_up {
        button(text("⬆ Up").size(14))
            .on_press(Message::NavigateUp)
            .padding(8)
            .style(|theme, status| button::Style {
                background: Some(Color::from_rgb(0.2, 0.4, 0.6).into()),
                text_color: Color::WHITE,
                ..button::primary(theme, status)
            })
    } else {
        button(text("⬆ Up").size(14))
            .padding(8)
            .style(|_theme, _status| button::Style {
                background: Some(Color::from_rgb(0.2, 0.2, 0.2).into()),
                text_color: Color::from_rgb(0.4, 0.4, 0.4),
                ..Default::default()
            })
    };

    let nav_buttons = row![
        up_button,
        text(format!("Depth: {}", zipper.depth()))
            .size(14)
            .color(Color::from_rgb(0.6, 0.6, 0.6)),
        text(status.to_string())
            .size(12)
            .color(Color::from_rgb(0.5, 0.7, 0.5)),
    ]
        .spacing(15)
        .padding(10);

    let mut content_col = Column::new().spacing(15).padding(20);

    // Focus info section
    let focus_section = if let Some(node) = focused_node {
        let kind = node.kind.clone();
        let text_preview = if node.text.len() > 80 {
            format!("{}...", &node.text[..80])
        } else {
            node.text.clone()
        };
        let num_children = node.children.len();

        column![
            text(format!("🎯 Focus: {}", kind))
                .size(20)
                .color(Color::from_rgb(0.86, 0.61, 0.45)),
            row![
                text("Text: ").size(14).color(Color::from_rgb(0.61, 0.86, 0.98)),
                text(text_preview).size(14).color(Color::from_rgb(0.86, 0.86, 0.79)),
            ].spacing(8),
            row![
                text("Children: ").size(14).color(Color::from_rgb(0.61, 0.86, 0.98)),
                text(format!("{}", num_children)).size(14).color(Color::from_rgb(0.71, 0.81, 0.66)),
            ].spacing(8),
        ]
            .spacing(8)
    } else {
        column![
            text("Loading...").size(16).color(Color::from_rgb(0.6, 0.6, 0.6)),
        ]
    };

    content_col = content_col.push(focus_section);

    // Children header
    if !child_nodes.is_empty() {
        content_col = content_col.push(
            text("📂 Children".to_string())
                .size(18)
                .color(Color::from_rgb(0.34, 0.61, 0.84))
        );
    }

    // Children list
    let mut children_col = Column::new().spacing(8);

    for (idx, (child_id, child_node)) in child_nodes.iter().enumerate() {
        let is_editing = editing_node_id.as_ref() == Some(child_id);

        let child_card = if is_editing {
            render_editing_node(child_node, edit_buffer)
        } else {
            render_child_node(child_id, child_node, idx)
        };

        children_col = children_col.push(child_card);
    }

    content_col = content_col.push(
        scrollable(children_col).height(Length::Fill)
    );

    container(
        column![
            breadcrumb,
            nav_buttons,
            content_col,
        ]
    )
        .width(Fill)
        .height(Fill)
        .style(main_container_style)
        .into()
}

fn render_child_node(node_id: &str, node: &TreeNode, index: usize) -> Element<'static, Message> {
    let node_id_for_edit = node_id.to_string();
    let node_id_for_nav = node_id.to_string();
    let node_kind = node.kind.clone();
    let node_text = node.text.clone();
    let has_children = !node.children.is_empty();
    let num_children = node.children.len();
    let has_content = !node_text.trim().is_empty();

    let display_text = if node_text.len() > 50 {
        format!("{}...", &node_text[..50])
    } else {
        node_text.clone()
    };

    // Blue for branches, gray for leaves
    let indicator_color = if has_children {
        Color::from_rgb(0.34, 0.61, 0.84)
    } else {
        Color::from_rgb(0.5, 0.5, 0.5)
    };

    // Navigate down button (only for nodes with children)
    let nav_button: Element<'static, Message> = if has_children {
        button(text(format!("⬇ ({})", num_children)).size(12))
            .on_press(Message::NavigateDown(node_id_for_nav, index))
            .padding(4)
            .style(|theme, status| button::Style {
                background: Some(Color::from_rgb(0.2, 0.4, 0.3).into()),
                text_color: Color::from_rgb(0.8, 1.0, 0.8),
                ..button::primary(theme, status)
            })
            .into()
    } else {
        text("leaf").size(11).color(Color::from_rgb(0.4, 0.4, 0.4)).into()
    };

    let type_label = if has_children {
        text("branch").size(11).color(Color::from_rgb(0.43, 0.43, 0.43))
    } else {
        text("token").size(11).color(Color::from_rgb(0.43, 0.43, 0.43))
    };

    container(
        row![
            // Left indicator bar
            container(text(""))
                .width(Length::Fixed(4.0))
                .height(Length::Fixed(50.0))
                .style(move |_theme| container::Style {
                    background: Some(indicator_color.into()),
                    ..Default::default()
                }),

            // Content
            column![
                row![
                    text(node_kind)
                        .size(13)
                        .color(Color::from_rgb(0.31, 0.76, 0.69)),
                    type_label,
                ]
                .spacing(10),

                row![
                    if has_content {
                        button(
                            text(format!("\"{}\"", display_text))
                                .size(13)
                                .color(Color::from_rgb(0.86, 0.86, 0.79))
                        )
                        .on_press(Message::StartEdit(node_id_for_edit))
                        .style(|_theme, _status| button::Style {
                            background: None,
                            text_color: Color::from_rgb(0.86, 0.86, 0.79),
                            border: Border::default(),
                            ..Default::default()
                        })
                    } else {
                        button(text("(empty)").size(11).color(Color::from_rgb(0.5, 0.5, 0.5)))
                            .style(|_theme, _status| button::Style {
                                background: None,
                                text_color: Color::from_rgb(0.5, 0.5, 0.5),
                                border: Border::default(),
                                ..Default::default()
                            })
                    },
                    nav_button,
                ]
                .spacing(10),
            ]
            .spacing(5)
            .padding(10),
        ]
            .spacing(8)
    )
        .width(Length::Fixed(650.0))
        .style(node_card_style)
        .into()
}

fn render_editing_node(node: &TreeNode, edit_buffer: &str) -> Element<'static, Message> {
    let edit_buffer = edit_buffer.to_string();
    let node_kind = node.kind.clone();

    container(
        row![
            // Green indicator for editing
            container(text(""))
                .width(Length::Fixed(4.0))
                .height(Length::Fixed(120.0))
                .style(|_theme| container::Style {
                    background: Some(Color::from_rgb(0.11, 0.64, 0.38).into()),
                    ..Default::default()
                }),

            column![
                row![
                    text(node_kind)
                        .size(13)
                        .color(Color::from_rgb(0.31, 0.76, 0.69)),
                    text("✏️ EDITING")
                        .size(11)
                        .color(Color::from_rgb(0.11, 0.64, 0.38)),
                ]
                .spacing(10),

                text_input("Type here to edit...", &edit_buffer)
                    .on_input(Message::UpdateEditText)
                    .on_submit(Message::CommitEdit)
                    .padding(10)
                    .size(14)
                    .style(|theme, status| {
                        let mut style = text_input::default(theme, status);
                        style.background = iced::Background::Color(Color::from_rgb(0.15, 0.15, 0.15));
                        style.border.color = Color::from_rgb(0.11, 0.64, 0.38);
                        style.border.width = 2.0;
                        style
                    }),

                row![
                    button(
                        row![text("✓").size(14), text(" Save").size(12)].spacing(4)
                    )
                        .on_press(Message::CommitEdit)
                        .padding(8)
                        .style(|theme, status| button::Style {
                            background: Some(Color::from_rgb(0.11, 0.64, 0.38).into()),
                            text_color: Color::WHITE,
                            border: Border::default(),
                            ..button::primary(theme, status)
                        }),

                    button(
                        row![text("✕").size(14), text(" Cancel").size(12)].spacing(4)
                    )
                        .on_press(Message::CancelEdit)
                        .padding(8)
                        .style(|theme, status| button::Style {
                            background: Some(Color::from_rgb(0.64, 0.11, 0.11).into()),
                            text_color: Color::WHITE,
                            border: Border::default(),
                            ..button::primary(theme, status)
                        }),
                ]
                .spacing(10),
            ]
            .spacing(10)
            .padding(15),
        ]
            .spacing(8)
    )
        .width(Length::Fixed(650.0))
        .style(editing_card_style)
        .into()
}

fn render_breadcrumb(zipper: &Zipper) -> Element<'static, Message> {
    let mut breadcrumb = Row::new().spacing(8);

    // Path nodes
    for (idx, crumb) in zipper.path.iter().enumerate() {
        if idx > 0 {
            breadcrumb = breadcrumb.push(text("›").color(Color::from_rgb(0.4, 0.4, 0.4)));
        }
        let short_id = if crumb.node_id.len() > 8 {
            &crumb.node_id[..8]
        } else {
            &crumb.node_id
        };
        breadcrumb = breadcrumb.push(
            text(short_id.to_string())
                .size(13)
                .color(Color::from_rgb(0.34, 0.61, 0.84))
        );
    }

    // Current focus (highlighted differently)
    if !zipper.path.is_empty() {
        breadcrumb = breadcrumb.push(text("›").color(Color::from_rgb(0.4, 0.4, 0.4)));
    }
    let short_focus = if zipper.focus.len() > 8 {
        &zipper.focus[..8]
    } else {
        &zipper.focus
    };
    breadcrumb = breadcrumb.push(
        text(short_focus.to_string())
            .size(13)
            .color(Color::from_rgb(0.81, 0.57, 0.47))
    );

    container(breadcrumb)
        .padding(12)
        .style(breadcrumb_container_style)
        .into()
}

fn main_container_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Color::from_rgb(0.12, 0.12, 0.12).into()),
        ..Default::default()
    }
}

fn breadcrumb_container_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Color::from_rgb(0.15, 0.15, 0.18).into()),
        border: Border {
            color: Color::from_rgb(0.24, 0.24, 0.26),
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

fn node_card_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Color::from_rgb(0.15, 0.15, 0.16).into()),
        border: Border {
            color: Color::from_rgb(0.24, 0.24, 0.26),
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}

fn editing_card_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Color::from_rgb(0.11, 0.20, 0.15).into()),
        border: Border {
            color: Color::from_rgb(0.11, 0.64, 0.38),
            width: 2.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}