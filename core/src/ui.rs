use alloc::{format, string::String, vec::Vec};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Paragraph, Wrap},
};

use crate::App;

pub(crate) fn draw(app: &mut App, frame: &mut Frame) {
    let area = frame.area();
    if area.width == 0 || area.height == 0 {
        return;
    }

    let visible = app.visible_text();
    let animating = app.is_animating();
    let choice_height = if animating || app.choices().is_empty() || area.height < 5 {
        0
    } else {
        let width = area.width.saturating_sub(2).max(1);
        let wanted = app
            .choices()
            .iter()
            .enumerate()
            .map(|(i, c)| {
                Paragraph::new(format!("  {}. {}", i + 1, c))
                    .wrap(Wrap { trim: true })
                    .line_count(width)
                    .max(1)
            })
            .sum::<usize>()
            + 2;
        wanted
            .min((area.height as usize / 3).max(3))
            .min(area.height as usize - 2) as u16
    };
    let story_area = Rect {
        height: area.height - choice_height,
        ..area
    };
    let story = Paragraph::new(visible)
        .wrap(Wrap { trim: true })
        .block(Block::bordered().title("THE INTERCEPT"));
    let width = story_area.width.saturating_sub(2).max(1);
    let total = story.line_count(width).max(1);
    let viewport = story_area.height.saturating_sub(2) as usize;
    app.set_viewport(total, viewport);
    frame.render_widget(
        story.scroll((app.scroll().min(u16::MAX as usize) as u16, 0)),
        story_area,
    );

    if choice_height == 0 {
        return;
    }
    let choice_area = Rect {
        y: area.y + story_area.height,
        height: choice_height,
        ..area
    };
    frame.render_widget(Block::bordered(), choice_area);
    let inner = Rect {
        x: choice_area.x.saturating_add(1),
        y: choice_area.y.saturating_add(1),
        width: choice_area.width.saturating_sub(2),
        height: choice_area.height.saturating_sub(2),
    };
    if inner.width == 0 || inner.height == 0 {
        return;
    }
    let rendered: Vec<(String, usize)> = app
        .choices()
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let text = format!("{}. {}", i + 1, c);
            let rows = Paragraph::new(format!("  {text}"))
                .wrap(Wrap { trim: true })
                .line_count(inner.width)
                .max(1);
            (text, rows)
        })
        .collect();
    let selected = app
        .selected_choice()
        .unwrap_or(0)
        .min(rendered.len().saturating_sub(1));
    let mut first = 0;
    let mut used = rendered
        .iter()
        .take(selected + 1)
        .map(|(_, rows)| *rows)
        .sum::<usize>();
    while used > inner.height as usize && first < selected {
        used -= rendered[first].1;
        first += 1;
    }
    let mut y = inner.y;
    for (index, (text, rows)) in rendered.iter().enumerate().skip(first) {
        if y >= inner.bottom() {
            break;
        }
        let height = (*rows).min((inner.bottom() - y) as usize) as u16;
        let style = if index == selected {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        let prefix = if index == selected { "> " } else { "  " };
        let paragraph = Paragraph::new(format!("{prefix}{text}"))
            .style(style)
            .wrap(Wrap { trim: true });
        frame.render_widget(
            paragraph,
            Rect {
                x: inner.x,
                y,
                width: inner.width,
                height,
            },
        );
        y += height;
    }
}
