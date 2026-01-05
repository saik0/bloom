use crate::red::Zipper;
use crate::ui::Message;
use iced::widget::{column, container, text, Row};
use iced::{Border, Color, Element, Fill, Theme};
use surrealdb::{engine::local::Db, Surreal};

pub fn render_viewport(
    _db: &Surreal<Db>,
    zipper: &Zipper,
) -> Element<'static, Message> {
    // Render breadcrumb
    let breadcrumb = render_breadcrumb(zipper);

    // Render focused node (placeholder for now)
    let focus_text = text(format!("Focus: {:?}", zipper.focus))
        .size(16);

    let path_info = text(format!("Path depth: {}", zipper.path.len()))
        .size(14)
        .color(Color::from_rgb(0.6, 0.6, 0.6));

    let content = column![
        breadcrumb,
        focus_text,
        path_info,
    ]
        .spacing(10)
        .padding(20);

    container(content)
        .width(Fill)
        .height(Fill)
        .style(main_container_style)
        .into()
}

fn render_breadcrumb(zipper: &Zipper) -> Element<'static, Message> {
    let mut breadcrumb = Row::new().spacing(8);

    // Add path items
    for (idx, crumb) in zipper.path.iter().enumerate() {
        if idx > 0 {
            breadcrumb = breadcrumb.push(text("›").color(Color::from_rgb(0.4, 0.4, 0.4)));
        }
        breadcrumb = breadcrumb.push(
            text(format!("{}", crumb.node_id.as_uuid()))
                .size(13)
                .color(Color::from_rgb(0.34, 0.61, 0.84))
        );
    }

    // Add focus
    if !zipper.path.is_empty() {
        breadcrumb = breadcrumb.push(text("›").color(Color::from_rgb(0.4, 0.4, 0.4)));
    }
    breadcrumb = breadcrumb.push(
        text(format!("{}", zipper.focus.as_uuid()))
            .size(13)
            .color(Color::from_rgb(0.81, 0.57, 0.47))
    );

    container(breadcrumb)
        .padding(12)
        .style(breadcrumb_container_style)
        .into()
}

// Static style functions - no allocations!
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