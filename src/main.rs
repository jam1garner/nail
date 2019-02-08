#![allow(dead_code)]
mod util;
mod file;
mod modes;
mod command_handler;
mod tilde_expand;
mod app;
mod nail;

use std::io;
use std::io::Write;
use std::process::Command;
use std::env;

use termion::cursor::Goto;
use termion::event::Key;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, Tabs, Text, Widget, Paragraph};
use tui::Terminal;

use crate::modes::Mode;
use crate::util::event::{Event, Events};
use crate::app::{App, AppOptions, Term};
use crate::nail::get_title_view;

#[allow(unused_variables)]
fn default_mode(events: &Events, app: &mut App, terminal: &mut Term) -> Result<(), failure::Error>  {
    match events.next()? {
        Event::Input(input) => match input {
            Key::Char(':') => {
               app.mode = Mode::Command;
               app.command = String::from(":");
            }
            Key::Char('i') =>
                app.mode = Mode::Insert,
            Key::Char('R') | Key::Char('r') =>
                app.mode = Mode::Replace,
            Key::Up | Key::Char('k') => {
                app.files[app.tabs_index].cursor.up();
            }
            Key::Down | Key::Char('j') => {
                let filesize = app.files[app.tabs_index].data.len();
                app.files[app.tabs_index].cursor.down(filesize);
            }
            Key::Left | Key::Char('h') => {
                app.files[app.tabs_index].cursor.left();
            }
            Key::Right | Key::Char('l') => {
                let filesize = app.files[app.tabs_index].data.len();
                app.files[app.tabs_index].cursor.right(filesize);
            }
            _ => {}
        },
        _ => {}
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
    match events.next()? {
        Event::Input(input) => match input {
            Key::Esc => app.mode = Mode::Default,
            Key::Char('\n') => {
                app.mode = Mode::Default;
                command_handler::handle_command(app, terminal);
                if let Mode::Default = app.mode {
                    if app.files.is_empty() {
                        app.mode = Mode::Title;
                    }
                }
            } 
            Key::Char(c) => app.command.push(c),
            Key::Backspace => {
                if app.command.pop().unwrap() == ':' && app.command.is_empty() {
                    if app.files.is_empty() {
                        app.mode = Mode::Title;
                    }
                    else {
                        app.mode = Mode::Default;
                    }
                }
            }
            _ => {}
        },
        _ => {}
    }
    Ok(())
}

#[allow(unused_variables)]
fn write_mode(events: &Events, app: &mut App, terminal: &mut Term) -> Result<(), failure::Error> {
    match events.next()? {
        Event::Input(input) => match input {
            Key::Esc =>
                app.mode = Mode::Default,
            Key::Up => {
                app.files[app.tabs_index].cursor.up();
            }
            Key::Down => {
                let filesize = app.files[app.tabs_index].data.len();
                app.files[app.tabs_index].cursor.down(filesize);
            }
            Key::Left => {
                app.files[app.tabs_index].cursor.left();
            }
            Key::Right => {
                let filesize = app.files[app.tabs_index].data.len();
                app.files[app.tabs_index].cursor.right(filesize);
            }
            Key::Char(c) => {
                if c.is_ascii_hexdigit() {
                    let digit = u8::from_str_radix(&c.to_string()[..], 16)?;
                    let cursor_pos = app.files[app.tabs_index].cursor.pos;
                    let byte_pos = (cursor_pos.0 / 2) + (cursor_pos.1 * 0x10);
                    match app.mode {
                        Mode::Insert => {
                            //TODO: Implement insert
                        }
                        Mode::Replace => {
                            if cursor_pos.0 % 2 == 0 {
                                // modify upper 4 bits
                                app.files[app.tabs_index].data[byte_pos] = 
                                    (app.files[app.tabs_index].data[byte_pos] & 0xF) | 
                                    ((digit << 4) & 0xF0);
                            }
                            else {
                                // lower 4 bits
                                app.files[app.tabs_index].data[byte_pos] = 
                                    (app.files[app.tabs_index].data[byte_pos] & 0xF0) | 
                                    (digit & 0xF);
                            }
                            let filesize = app.files[app.tabs_index].data.len();
                            app.files[app.tabs_index].cursor.right(filesize);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        },
        _ => {}
    }
    Ok(())
}

fn title_mode(events: &Events, app: &mut App, terminal: &mut Term) -> Result<(), failure::Error> {
    match events.next()? {
        Event::Input(input) => match input {
            Key::Char(':') => {
               app.mode = Mode::TitleCommand;
               app.command = String::from(":");
            }
            _ => {}
        },
        _ => {}
    }
    Ok(())
}

fn main() -> Result<(), failure::Error> {
    // Hardcoded files atm
    /*let filenames = vec!["File 0", "File 1", "File 2","File 3"];
    let filepaths = vec!["C:/path/to/file/0.txt",
                        "C:/path/to/file/1.txt",
                        "C:/path/to/file/2.txt",
                        "C:/path/to/file/3.txt"];
    let files : Vec<File> = 
        vec![b"Test 1 blha blah blah \x00 test adsasdasdas\xFF\x12dsadsad\n".to_vec(),
             b"Test 2\n".to_vec(),
             (0u8..=0xFFu8).collect(),
             b"Test 4\n".to_vec()]
            .into_iter()
            .enumerate()
            .map(|(x, y)| File {
                name: filenames[x].to_string(),
                path: filepaths[x].to_string(),
                data: y.to_vec(),
                cursor: HexCursor::new((0,0)),
                scroll_y: 0
            })
            .collect();*/

    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    // App
    let mut app = App {
        files: Vec::new(),
        mode: Mode::Title,
        command: String::new(),
        size: Rect::new(0,0,0,0),
        tabs_index: 0,
        line_count: 0,
        options: AppOptions::new(),
    };

    // Load files from args
    for arg in env::args().skip(1) {
        if let Ok(_x) = app.open(&arg[..]) {
            app.mode = Mode::Default;
        }
    }

    let events = Events::new();
    let mut editor_rect = Rect::new(0,0,0,0);
    
    // Main loop
    loop {
        match app.mode {
            Mode::Command => terminal.show_cursor()?,
            _ => {}
        }
        
        match app.mode {
            Mode::Bash => {
                terminal.clear()?;
                write!(
                    terminal.backend_mut(),
                    "{}",
                    Goto(1,1)
                )?;
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
                loop{} 
            }
            Mode::Title | Mode::TitleCommand => {
                terminal.draw(|mut f| {
                    app.size = f.size();
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Min(23), Constraint::Length(1)].as_ref())
                        .split(app.size);
                    Paragraph::new(get_title_view().iter())
                        .block(Block::default().borders(Borders::ALL))
                        .render(&mut f, chunks[0]);
                    Paragraph::new(vec![Text::raw(app.command.clone())].iter())
                        .style(Style::default().bg(
                                match app.mode {
                                    Mode::TitleCommand => Color::Red,
                                    _ => Color::Cyan
                                }))
                        .render(&mut f, chunks[1]);
                })?;
            }
            _ => {
                terminal.draw(|mut f| {
                    app.size = f.size();
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Length(3), Constraint::Min(3), Constraint::Length(1)].as_ref())
                        .split(app.size);
                    // -2 for the border, -1 for the top line
                    // calculate number of lines of hex we have room for
                    let reserved_lines = if app.options.type_inspector {3} else {0};
                    app.line_count = (chunks[1].height - (3 + reserved_lines)) as usize;
                    
                    // If cursor is out of bounds, scroll
                    let file = &mut app.files[app.tabs_index];
                    if file.cursor.pos.1 * 0x10 < file.scroll_y {
                        file.scroll_y = file.cursor.pos.1 * 0x10;
                    }
                    
                    // +0 = +1 for "one past the end" -1 for "including the header line"
                    if (file.scroll_y / 0x10) + app.line_count <= file.cursor.pos.1 {
                        file.scroll_y = (file.cursor.pos.1 + 1 - app.line_count) * 0x10; 
                    }

                    editor_rect = chunks[1];
                    Block::default()
                        .style(Style::default().bg(
                                match app.mode {
                                    Mode::Command => Color::Red,
                                    _ => Color::Cyan
                                }))
                        .render(&mut f, app.size);
                    let index = app.tabs_index;
                    Tabs::default()
                        .block(Block::default().borders(Borders::ALL).title("Tabs"))
                        .titles(&app.tab_titles())
                        .select(index)
                        .style(Style::default().fg(Color::LightBlue))
                        .highlight_style(Style::default().fg(Color::Red))
                        .render(&mut f, chunks[0]);

                    let view = app.files[app.tabs_index].hex_view(&app);
                    Paragraph::new(view.iter())
                    .block(
                        Block::default()
                        .title(&app.files[app.tabs_index].path)
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(
                            match app.mode {
                                Mode::Insert | Mode::Replace => Color::Yellow,
                                _ => Color::White
                            })))
                    .render(&mut f, chunks[1]);

                    Paragraph::new(vec![Text::raw(app.command.clone())].iter())
                        .style(Style::default().bg(
                                match app.mode {
                                    Mode::Command => Color::Red,
                                    _ => Color::Cyan
                                }))
                        .render(&mut f, chunks[2]);
                })?;
            }
        }
        
        match app.mode {
            Mode::Default | Mode::Insert | Mode::Replace => {
                terminal.show_cursor()?;
                editor_rect.x = 0;
                let file = &mut app.files[app.tabs_index];
                //if file.cursor.pos.1 * 0x10 < file.scroll_y {
                //    file.scroll_y = file.cursor.pos.1 * 0x10;
                //}
                // +0 = +1 for "one past the end" -1 for "including the header line"
                //if (file.scroll_y / 0x10) + app.line_count <= file.cursor.pos.1 {
                //    file.scroll_y = (file.cursor.pos.1 - app.line_count) * 0x10; 
                //}
                write!(
                    terminal.backend_mut(),
                    "{}",
                    Goto((editor_rect.x as usize + 11 + ((file.cursor.pos.0 / 2) * 3) + (file.cursor.pos.0 % 2)) as u16,
                         (editor_rect.y as usize + 3 + file.cursor.pos.1 - (file.scroll_y / 0x10)) as u16)
                )?;
            }
            Mode::Command => {}
            _ => {
                terminal.hide_cursor()?
            }
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
