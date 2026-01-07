use crate::red::Zipper;
use crate::ui::Message;
use iced::widget::{button, column, container, row, scrollable, text, Column, Row};
use iced::{Border, Color, Element, Fill, Length, Theme};
use ra_ap_syntax::SyntaxNode;

pub fn render_viewport(
    zipper: &Zipper,
    status: &str,
) -> Element<'static, Message> {
    let breadcrumb = render_breadcrumb(zipper);
    
    let focus = zipper.focus();
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
        button(text("← Prev").size(14))
            .on_press(Message::PrevSibling)
            .padding(8),
        button(text("Next →").size(14))
            .on_press(Message::NextSibling)
            .padding(8),
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

    // Focus info section - directly from rowan SyntaxNode
    let kind = format!("{:?}", focus.kind());
    let text_content = focus.text().to_string();
    let text_preview = if text_content.len() > 80 {
        format!("{}...", &text_content[..80])
    } else {
        text_content.clone()
    };
    let range = focus.text_range();
    let child_count = zipper.child_count();

    let focus_section = column![
        text(format!("🎯 Focus: {}", kind))
            .size(20)
            .color(Color::from_rgb(0.86, 0.61, 0.45)),
        row![
            text("Span: ").size(14).color(Color::from_rgb(0.61, 0.86, 0.98)),
            text(format!("{}..{}", u32::from(range.start()), u32::from(range.end())))
                .size(14)
                .color(Color::from_rgb(0.71, 0.81, 0.66)),
        ].spacing(8),
        row![
            text("Text: ").size(14).color(Color::from_rgb(0.61, 0.86, 0.98)),
            text(text_preview).size(14).color(Color::from_rgb(0.86, 0.86, 0.79)),
        ].spacing(8),
        row![
            text("Children: ").size(14).color(Color::from_rgb(0.61, 0.86, 0.98)),
            text(format!("{}", child_count)).size(14).color(Color::from_rgb(0.71, 0.81, 0.66)),
        ].spacing(8),
    ]
    .spacing(8);

    content_col = content_col.push(focus_section);

    // Children list - directly from rowan
    if child_count > 0 {
        content_col = content_col.push(
            text("📂 Children".to_string())
                .size(18)
                .color(Color::from_rgb(0.34, 0.61, 0.84))
        );

        let mut children_col = Column::new().spacing(8);
        
        for (idx, child) in zipper.children().enumerate() {
            let child_card = render_child_node(&child, idx);
            children_col = children_col.push(child_card);
        }

        content_col = content_col.push(
            scrollable(children_col).height(Length::Fill)
        );
    }

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

fn render_child_node(node: &SyntaxNode, index: usize) -> Element<'static, Message> {
    let kind = format!("{:?}", node.kind());
    let text_content = node.text().to_string();
    let has_children = node.children().count() > 0;
    let num_children = node.children().count();

    let display_text = if text_content.len() > 50 {
        format!("{}...", &text_content[..50])
    } else {
        text_content
    };

    let indicator_color = if has_children {
        Color::from_rgb(0.34, 0.61, 0.84)
    } else {
        Color::from_rgb(0.5, 0.5, 0.5)
    };

    let nav_button: Element<'static, Message> = if has_children {
        button(text(format!("⬇ ({})", num_children)).size(12))
            .on_press(Message::NavigateDown(index))
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
            container(text(""))
                .width(Length::Fixed(4.0))
                .height(Length::Fixed(50.0))
                .style(move |_theme| container::Style {
                    background: Some(indicator_color.into()),
                    ..Default::default()
                }),
            column![
                row![
                    text(kind)
                        .size(13)
                        .color(Color::from_rgb(0.31, 0.76, 0.69)),
                    type_label,
                ]
                .spacing(10),
                row![
                    text(format!("\"{}\"", display_text))
                        .size(13)
                        .color(Color::from_rgb(0.86, 0.86, 0.79)),
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

fn render_breadcrumb(zipper: &Zipper) -> Element<'static, Message> {
    let mut breadcrumb = Row::new().spacing(8);

    for (idx, (kind, _child_idx)) in zipper.breadcrumbs().iter().enumerate() {
        if idx > 0 {
            breadcrumb = breadcrumb.push(text("›").color(Color::from_rgb(0.4, 0.4, 0.4)));
        }
        let short_kind = if kind.len() > 12 {
            &kind[..12]
        } else {
            kind
        };
        breadcrumb = breadcrumb.push(
            text(short_kind.to_string())
                .size(13)
                .color(Color::from_rgb(0.34, 0.61, 0.84))
        );
    }

    // Current focus
    if !zipper.breadcrumbs().is_empty() {
        breadcrumb = breadcrumb.push(text("›").color(Color::from_rgb(0.4, 0.4, 0.4)));
    }
    let focus_kind = format!("{:?}", zipper.focus().kind());
    let short_focus = if focus_kind.len() > 12 {
        &focus_kind[..12]
    } else {
        &focus_kind
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
