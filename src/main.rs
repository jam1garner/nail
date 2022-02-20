#![allow(dead_code)]
mod app;
mod command_handler;
mod file;
mod modes;
mod nail;
mod tabs;
mod tilde_expand;
mod util;

use std::env;
use std::io;
use std::io::Write;
use std::process::Command;

use termion::cursor::Goto;
use termion::event::Key;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::text::Span;
use tui::text::Spans;
use tui::widgets::{Block, Borders, Paragraph, Tabs};
use tui::Terminal;

use crate::app::{App, AppOptions, Term};
use crate::modes::Mode;
use crate::tabs::Tab;
use crate::util::event::{Event, Events};

#[allow(unused_variables)]
fn default_mode(events: &Events, app: &mut App, terminal: &mut Term) -> Result<(), failure::Error> {
    if let Event::Input(input) = events.next()? {
        match input {
            Key::Char(':') => {
                app.mode = Mode::Command;
                app.command = String::from(":");
            }
            Key::Char('/') => {
                app.mode = Mode::Command;
                app.command = String::from("/");
            }
            Key::Char('i') => app.mode = Mode::Insert,
            Key::Char('R') | Key::Char('r') => app.mode = Mode::Replace,
            Key::Up | Key::Char('k') => {
                if let Tab::File(current_file) = &mut app.tabs[app.tabs_index] {
                    current_file.cursor.up();
                }
            }
            Key::PageUp => {
                if let Tab::File(current_file) = &mut app.tabs[app.tabs_index] {
                    let filesize = current_file.data.len();
                    for _bulk_action in 0..34 {
                        current_file.cursor.up();
                    }
                }
            }
            Key::Down | Key::Char('j') => {
                if let Tab::File(current_file) = &mut app.tabs[app.tabs_index] {
                    let filesize = current_file.data.len();
                    current_file.cursor.down(filesize);
                }
            }
            Key::PageDown => {
                if let Tab::File(current_file) = &mut app.tabs[app.tabs_index] {
                    let filesize = current_file.data.len();
                    for _bulk_action in 0..34 {
                        current_file.cursor.down(filesize);
                    }
                }
            }
            Key::Left | Key::Char('h') => {
                if let Tab::File(current_file) = &mut app.tabs[app.tabs_index] {
                    current_file.cursor.left();
                }
            }
            Key::Right | Key::Char('l') => {
                if let Tab::File(current_file) = &mut app.tabs[app.tabs_index] {
                    let filesize = current_file.data.len();
                    current_file.cursor.right(filesize);
                }
            }
            Key::Char('w') => {
                if let Tab::File(current_file) = &mut app.tabs[app.tabs_index] {
                    let filesize = current_file.data.len();
                    current_file.cursor.next_word(filesize);
                }
            }
            Key::Char('b') => {
                if let Tab::File(current_file) = &mut app.tabs[app.tabs_index] {
                    current_file.cursor.prev_word();
                }
            }
            Key::Char('G') => {
                if let Tab::File(current_file) = &mut app.tabs[app.tabs_index] {
                    current_file
                        .cursor
                        .goto(current_file.data.len().saturating_sub(1));
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn command_mode(events: &Events, app: &mut App, terminal: &mut Term) -> Result<(), failure::Error> {
    // Move cursor to proper position
    write!(
        terminal.backend_mut(),
        "{}",
        Goto(1 + app.command.len() as u16, app.size.height)
    )?;
    // Command mode event handling
    if let Event::Input(input) = events.next()? {
        match input {
            Key::Esc => app.mode = Mode::Default,
            Key::Char('\n') => {
                app.mode = Mode::Default;
                command_handler::handle_command(app, terminal);
                if let Mode::Default = app.mode {
                    if app.tabs.is_empty() {
                        app.mode = Mode::Title;
                    }
                }
            }
            Key::Char(c) => app.command.push(c),
            Key::Backspace => {
                if app.command.pop().unwrap() == ':' && app.command.is_empty() {
                    if app.tabs.is_empty() {
                        app.mode = Mode::Title;
                    } else {
                        app.mode = Mode::Default;
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

#[allow(unused_variables)]
fn write_mode(events: &Events, app: &mut App, terminal: &mut Term) -> Result<(), failure::Error> {
    if let Event::Input(input) = events.next()? {
        match input {
            Key::Esc => app.mode = Mode::Default,
            Key::Up => {
                if let Tab::File(current_file) = &mut app.tabs[app.tabs_index] {
                    current_file.cursor.up();
                }
            }
            Key::Down => {
                if let Tab::File(current_file) = &mut app.tabs[app.tabs_index] {
                    let filesize = current_file.data.len();
                    current_file.cursor.down(filesize);
                }
            }
            Key::Left => {
                if let Tab::File(current_file) = &mut app.tabs[app.tabs_index] {
                    current_file.cursor.left();
                }
            }
            Key::Right => {
                if let Tab::File(current_file) = &mut app.tabs[app.tabs_index] {
                    let filesize = current_file.data.len();
                    current_file.cursor.right(filesize);
                }
            }
            Key::PageUp => {
                if let Tab::File(current_file) = &mut app.tabs[app.tabs_index] {
                    let filesize = current_file.data.len();
                    for _bulk_action in 0..34 {
                        current_file.cursor.up();
                    }
                }
            }
            Key::PageDown => {
                if let Tab::File(current_file) = &mut app.tabs[app.tabs_index] {
                    let filesize = current_file.data.len();
                    for _bulk_action in 0..34 {
                        current_file.cursor.down(filesize);
                    }
                }
            }
            Key::Char(c) => {
                if c.is_ascii_hexdigit() {
                    if let Tab::File(current_file) = &mut app.tabs[app.tabs_index] {
                        let digit = u8::from_str_radix(&c.to_string()[..], 16)?;
                        let cursor_pos = current_file.cursor.pos;
                        let byte_pos = (cursor_pos.0 / 2) + (cursor_pos.1 * 0x10);
                        match app.mode {
                            Mode::Insert => {
                                //TODO: Implement insert
                            }
                            Mode::Replace => {
                                if cursor_pos.0 % 2 == 0 {
                                    // modify upper 4 bits
                                    current_file.data[byte_pos] =
                                        (current_file.data[byte_pos] & 0xF) | ((digit << 4) & 0xF0);
                                } else {
                                    // lower 4 bits
                                    current_file.data[byte_pos] =
                                        (current_file.data[byte_pos] & 0xF0) | (digit & 0xF);
                                }
                                let filesize = current_file.data.len();
                                current_file.cursor.right(filesize);
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

#[allow(unused_variables)]
fn title_mode(events: &Events, app: &mut App, terminal: &mut Term) -> Result<(), failure::Error> {
    if let Event::Input(Key::Char(':')) = events.next()? {
        app.mode = Mode::TitleCommand;
        app.command = String::from(":");
    }
    Ok(())
}

fn main() -> Result<(), failure::Error> {
    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    // App
    let mut app = App {
        tabs: Vec::new(),
        mode: Mode::Title,
        command: String::new(),
        size: Rect::new(0, 0, 0, 0),
        tabs_index: 0,
        line_count: 0,
        options: AppOptions::new(),
    };

    // Load files from args
    for arg in env::args().skip(1) {
        if app.open(&arg[..]).is_ok() {
            app.mode = Mode::Default;
        }
    }

    if app.tabs.is_empty() {
        app.tabs.push(Tab::Title);
    }

    let events = Events::new();
    let mut editor_rect = Rect::new(0, 0, 0, 0);

    // Main loop
    loop {
        if let Mode::Command = app.mode {
            terminal.show_cursor()?
        }

        if let Tab::Title = app.tabs[app.tabs_index] {
            match app.mode {
                Mode::Title | Mode::TitleCommand | Mode::Quit => {}
                _ => {
                    app.mode = Mode::Title;
                }
            }
        }

        match app.mode {
            Mode::Bash => {
                terminal.clear()?;
                write!(terminal.backend_mut(), "{}", Goto(1, 1))?;
                let _output = if cfg!(target_os = "windows") {
                    Command::new("cmd")
                        .args(&["/C", &app.command[..]])
                        .output()
                        .expect("failed to execute process")
                } else {
                    Command::new("sh")
                        .arg("-c")
                        .arg(&app.command[..])
                        .output()
                        .expect("failed to execute process")
                };
            }
            //            Mode::Title | Mode::TitleCommand => {
            //                terminal.draw(|mut f| {
            //                    app.size = f.size();
            //                    let chunks = Layout::default()
            //                        .direction(Direction::Vertical)
            //                        .constraints([Constraint::Min(23), Constraint::Length(1)].as_ref())
            //                        .split(app.size);
            //                    Paragraph::new(get_title_view().iter())
            //                        .block(Block::default().borders(Borders::ALL))
            //                        .render(&mut f, chunks[0]);
            //                    Paragraph::new(vec![Text::raw(app.command.clone())].iter())
            //                        .style(Style::default().bg(
            //                                match app.mode {
            //                                    Mode::TitleCommand => Color::Red,
            //                                    _ => Color::Cyan
            //                                }))
            //                        .render(&mut f, chunks[1]);
            //                })?;
            //            }
            _ => {
                terminal.draw(|f| {
                    app.size = f.size();
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints(
                            [
                                Constraint::Length(3),
                                Constraint::Min(3),
                                Constraint::Length(1),
                            ]
                            .as_ref(),
                        )
                        .split(app.size);
                    // -2 for the border, -1 for the top line
                    // calculate number of lines of hex we have room for
                    let reserved_lines = if app.options.type_inspector { 3 } else { 0 };
                    app.line_count = (chunks[1].height - (3 + reserved_lines)) as usize;

                    // If cursor is out of bounds, scroll
                    if let Tab::File(file) = &mut app.tabs[app.tabs_index] {
                        if file.cursor.pos.1 * 0x10 < file.scroll_y {
                            file.scroll_y = file.cursor.pos.1 * 0x10;
                        }

                        // +0 = +1 for "one past the end" -1 for "including the header line"
                        if (file.scroll_y / 0x10) + app.line_count <= file.cursor.pos.1 {
                            file.scroll_y = (file.cursor.pos.1 + 1 - app.line_count) * 0x10;
                        }
                    }

                    editor_rect = chunks[1];

                    let block = Block::default().style(Style::default().bg(match app.mode {
                        Mode::Command => Color::Red,
                        _ => Color::DarkGray,
                    }));
                    f.render_widget(block, app.size);
                    let index = app.tabs_index;
                    let tabs =
                        Tabs::new(app.tab_titles().iter().cloned().map(Spans::from).collect())
                            .block(Block::default().borders(Borders::ALL).title("Tabs"))
                            .select(index)
                            .style(Style::default().fg(Color::LightBlue))
                            .highlight_style(Style::default().fg(Color::Red));
                    f.render_widget(tabs, chunks[0]);
                    let view = app.current_tab().view(&app);
                    let p = Paragraph::new(view).block(
                        Block::default()
                            .title(app.current_tab().long_title())
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(match app.mode {
                                Mode::Insert | Mode::Replace => Color::Yellow,
                                _ => Color::White,
                            })),
                    );
                    f.render_widget(p, chunks[1]);
                    let p = Paragraph::new(vec![Spans::from(Span::raw(app.command.clone()))])
                        .style(Style::default().bg(match app.mode {
                            Mode::Command => Color::Red,
                            _ => Color::DarkGray,
                        }));
                    f.render_widget(p, chunks[2]);
                })?;
            }
        }

        match app.mode {
            Mode::Default | Mode::Insert | Mode::Replace => {
                terminal.show_cursor()?;
                editor_rect.x = 0;
                if let Tab::File(file) = &mut app.tabs[app.tabs_index] {
                    write!(
                        terminal.backend_mut(),
                        "{}",
                        Goto(
                            (editor_rect.x as usize
                                + 11
                                + ((file.cursor.pos.0 / 2) * 3)
                                + (file.cursor.pos.0 % 2)) as u16,
                            (editor_rect.y as usize + 3 + file.cursor.pos.1
                                - (file.scroll_y / 0x10)) as u16
                        )
                    )?;
                }
            }
            Mode::Command => {}
            _ => terminal.hide_cursor()?,
        }

        match app.mode {
            Mode::Default => default_mode(&events, &mut app, &mut terminal)?,
            Mode::Command | Mode::TitleCommand => command_mode(&events, &mut app, &mut terminal)?,
            Mode::Insert | Mode::Replace => write_mode(&events, &mut app, &mut terminal)?,
            Mode::Title => title_mode(&events, &mut app, &mut terminal)?,
            Mode::Quit => break,
            _ => {}
        };
    }
    Ok(())
}
