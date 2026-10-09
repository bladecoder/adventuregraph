use alloc::{format, string::String, vec::Vec};
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Paragraph, Widget, Wrap},
};

use crate::{App, StoryLine};
use tui_big_text::{BigText, PixelSize};

pub(crate) fn draw(app: &mut App, frame: &mut Frame) {
    let area = frame.area();
    if area.width == 0 || area.height == 0 {
        return;
    }

    frame.render_widget(
        Block::default().style(Style::default().bg(Color::Black)),
        area,
    );

    let visible = app.visible_lines();
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
    let block = Block::bordered().title("ADVENTUREGRAPH");
    let inner = block.inner(story_area);
    frame.render_widget(block, story_area);
    let total = history_height(&visible, inner.width.max(1));
    app.set_viewport(total, inner.height as usize);
    render_history(&visible, inner, app.scroll(), frame.buffer_mut());

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

fn banner_height(mode: PixelSize) -> u16 {
    match mode {
        PixelSize::Full | PixelSize::HalfWidth => 8,
        PixelSize::HalfHeight | PixelSize::Quadrant => 4,
        PixelSize::ThirdHeight | PixelSize::Sextant => 3,
        PixelSize::QuarterHeight | PixelSize::Octant => 2,
    }
}
fn paragraph(line: &StoryLine) -> Paragraph<'_> {
    Paragraph::new(
        line.text
            .split('\n')
            .map(|part| Line::styled(part, line.style).alignment(line.alignment))
            .collect::<Vec<_>>(),
    )
    .wrap(Wrap { trim: true })
}
fn line_height(line: &StoryLine, width: u16) -> usize {
    match line.banner {
        Some(mode) => line.text.split('\n').count() * usize::from(banner_height(mode)),
        None => paragraph(line).line_count(width).max(1),
    }
}
fn history_height(lines: &[StoryLine], width: u16) -> usize {
    lines
        .iter()
        .map(|line| line_height(line, width))
        .sum::<usize>()
        + lines.len().saturating_sub(1)
}
fn render_history(lines: &[StoryLine], area: Rect, scroll: usize, buffer: &mut Buffer) {
    if area.is_empty() {
        return;
    }
    let mut row = 0usize;
    let mut scratch = Buffer::empty(Rect::new(0, 0, area.width, 8));
    for line in lines {
        let height = line_height(line, area.width);
        let start = row.max(scroll);
        let end = (row + height).min(scroll + usize::from(area.height));
        if start < end {
            if let Some(mode) = line.banner {
                let h = usize::from(banner_height(mode));
                for (index, part) in line.text.split('\n').enumerate() {
                    let top = row + index * h;
                    let first = top.max(start);
                    let last = (top + h).min(end);
                    if first >= last {
                        continue;
                    }
                    scratch.reset();
                    BigText::builder()
                        .lines(alloc::vec![Line::from(part)])
                        .style(line.style)
                        .alignment(line.alignment)
                        .pixel_size(mode)
                        .build()
                        .render(Rect::new(0, 0, area.width, h as u16), &mut scratch);
                    for y in first..last {
                        for x in 0..area.width {
                            buffer[(area.x + x, area.y + (y - scroll) as u16)] =
                                scratch[(x, (y - top) as u16)].clone();
                        }
                    }
                }
            } else {
                paragraph(line)
                    .scroll(((start - row).min(u16::MAX as usize) as u16, 0))
                    .render(
                        Rect::new(
                            area.x,
                            area.y + (start - scroll) as u16,
                            area.width,
                            (end - start) as u16,
                        ),
                        buffer,
                    );
            }
        }
        row += height + 1;
        if row >= scroll + usize::from(area.height) {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{layout::Alignment, style::Style};
    #[test]
    fn mixed_history_scroll_preserves_wrapping_separators_and_banner_rows() {
        let lines = alloc::vec![
            StoryLine {
                text: "normal text wraps across several rows".into(),
                style: Style::default(),
                alignment: Alignment::Center,
                banner: None
            },
            StoryLine {
                text: "Ab\nZ".into(),
                style: Style::default().fg(Color::Red),
                alignment: Alignment::Right,
                banner: Some(PixelSize::Full)
            },
            StoryLine {
                text: "last normal line".into(),
                style: Style::default(),
                alignment: Alignment::Left,
                banner: None
            },
        ];
        let height = history_height(&lines, 13) as u16;
        let mut full = Buffer::empty(Rect::new(0, 0, 13, height));
        render_history(&lines, full.area, 0, &mut full);
        for scroll in 0..height {
            let mut clipped = Buffer::empty(Rect::new(0, 0, 13, 3));
            render_history(&lines, clipped.area, usize::from(scroll), &mut clipped);
            for y in 0..3.min(height - scroll) {
                for x in 0..13 {
                    assert_eq!(clipped[(x, y)], full[(x, y + scroll)]);
                }
            }
        }
    }

    #[test]
    fn banners_match_native_widget_at_every_scroll_row() {
        for mode in [
            PixelSize::Full,
            PixelSize::HalfHeight,
            PixelSize::HalfWidth,
            PixelSize::Quadrant,
            PixelSize::ThirdHeight,
            PixelSize::Sextant,
            PixelSize::QuarterHeight,
            PixelSize::Octant,
        ] {
            for alignment in [Alignment::Left, Alignment::Center, Alignment::Right] {
                for width in [1, 7, 40, 100] {
                    let line = StoryLine {
                        text: "Abáé\nñZ?".into(),
                        style: Style::default().fg(Color::Yellow).bg(Color::Blue),
                        alignment,
                        banner: Some(mode),
                    };
                    let height = banner_height(mode) * 2;
                    let mut native = Buffer::empty(Rect::new(0, 0, width, height));
                    BigText::builder()
                        .lines(line.text.split('\n').map(Line::from).collect::<Vec<_>>())
                        .style(line.style)
                        .alignment(alignment)
                        .pixel_size(mode)
                        .build()
                        .render(native.area, &mut native);
                    for scroll in 0..usize::from(height) {
                        for view in [1, 3, 9] {
                            let mut actual = Buffer::empty(Rect::new(2, 3, width, view));
                            render_history(
                                core::slice::from_ref(&line),
                                actual.area,
                                scroll,
                                &mut actual,
                            );
                            for y in 0..view.min(height - scroll as u16) {
                                for x in 0..width {
                                    assert_eq!(
                                        actual[(2 + x, 3 + y)],
                                        native[(x, scroll as u16 + y)],
                                        "{mode:?} {alignment:?} width={width} scroll={scroll}"
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
