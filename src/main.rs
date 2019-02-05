#![allow(dead_code)]
mod util;
mod file;
mod modes;
mod command_handler;
mod app;

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
use tui::widgets::{Block, Borders, Tabs, Text, Widget, Paragraph};
use tui::Terminal;
use crate::file::File;
use crate::modes::Mode;

use crate::util::event::{Event, Events};
use crate::util::HexCursor;
use crate::app::{App, Term};

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
            Key::Up | Key::Char('j') => {
                app.files[app.tabs_index].cursor.up();
            }
            Key::Down | Key::Char('k') => {
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
                app.command = String::new();
            } 
            Key::Char(c) => app.command.push(c),
            Key::Backspace => {
                if app.command.pop().unwrap() == ':' && app.command.is_empty() {
                    app.mode = Mode::Default;
                }
            }
            _ => {}
        },
        _ => {}
    }
    Ok(())
}

#[allow(unused_variables)]
fn insert_mode(events: &Events, app: &mut App, terminal: &mut Term) -> Result<(), failure::Error> {
    match events.next()? {
        Event::Input(input) => match input {
            Key::Esc => app.mode = Mode::Default,
            _ => {}
        },
        _ => {}
    }
    Ok(())
} 

fn main() -> Result<(), failure::Error> {
    // Hardcoded files atm
    let filenames = vec!["File 0", "File 1", "File 2","File 3"];
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
            .collect();

    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    // App
    let mut app = App {
        files,
        mode: Mode::Default,
        command: String::new(),
        size: Rect::new(0,0,0,0),
        tabs_index: 0
    };

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
            _ => {
                terminal.draw(|mut f| {
                    app.size = f.size();
                    let line_count = app.size.height as usize;
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Length(3), Constraint::Min(3), Constraint::Length(1)].as_ref())
                        .split(app.size);
                    editor_rect = chunks[1].clone();
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

                    let view = app.files[app.tabs_index].hex_view(line_count);
                    Paragraph::new(view.iter())
                    .block(
                        Block::default()
                        .title(&app.files[app.tabs_index].path)
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(
                            match app.mode {
                                Mode::Insert => Color::Yellow,
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
            Mode::Default | Mode::Insert => {
                terminal.show_cursor()?;
                editor_rect.x = 0;
                let file = &app.files[app.tabs_index];
                write!(
                    terminal.backend_mut(),
                    "{}",
                    Goto((editor_rect.x as usize + 11 + ((file.cursor.pos.0 / 2) * 3) + (file.cursor.pos.0 % 2)) as u16,
                         (editor_rect.y as usize + 3 + file.cursor.pos.1 - file.scroll_y) as u16)
                )?;
            }
            Mode::Command => {}
            _ => {
                terminal.hide_cursor()?
            }
        }

        match app.mode {
            Mode::Default => default_mode(&events, &mut app, &mut terminal)?,
            Mode::Command => command_mode(&events, &mut app, &mut terminal)?,
            Mode::Insert => insert_mode(&events, &mut app, &mut terminal)?,
            Mode::Quit => break,
            _ => {}
        };
    }
    Ok(())
}
