#[allow(dead_code)]
mod util;
mod file;

use std::io;

use termion::event::Key;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, Tabs, Widget, Paragraph};
use tui::Terminal;
use crate::file::File;

use crate::util::event::{Event, Events};
use crate::util::TabsState;

struct App<'a> {
    tabs: TabsState<'a>,
    files: Vec<File<'a>>,
}

fn main() -> Result<(), failure::Error> {
    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let events = Events::new();
    
    let filenames = vec!["File 0", "File 1", "File 2","File 3"];
    let filepaths = vec!["C:/path/to/file/0.txt",
                        "C:/path/to/file/1.txt",
                        "C:/path/to/file/2.txt",
                        "C:/path/to/file/3.txt"];
    let files : Vec<File> = 
        vec![b"Test 1 blha blah blah \x00 test adsasdasdasdsadsad\n".to_vec(),
             b"Test 2\n".to_vec(),
             b"Test 3\n".to_vec(),
             b"Test 4\n".to_vec()]
            .into_iter()
            .enumerate()
            .map(|(x, y)| File {
                name: filenames[x],
                path: filepaths[x],
                data: y.to_vec(),
                pos: 0
            })
            .collect();

    // App
    let mut app = App {
        files,
        tabs: TabsState::new(filenames),
    };

    // Main loop
    loop {
        terminal.draw(|mut f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                //.margin(5)
                .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                .split(size);

            Block::default()
                .style(Style::default().bg(Color::White))
                .render(&mut f, size);
            Tabs::default()
                .block(Block::default().borders(Borders::ALL).title("Tabs"))
                .titles(&app.tabs.titles)
                .select(app.tabs.index)
                .style(Style::default().fg(Color::LightBlue))
                .highlight_style(Style::default().fg(Color::Red))
                .render(&mut f, chunks[0]);
            match app.tabs.index {
                0...4 => {
                    let view = app.files[app.tabs.index].hex_view(10);
                    Paragraph::new(view.iter())
                    .block(
                        Block::default()
                        .title(app.files[app.tabs.index].path)
                        .borders(Borders::ALL))
                    .render(&mut f, chunks[1]);
                }
                _ => {}
            }
        })?;

        match events.next()? {
            Event::Input(input) => match input {
                Key::Char('q') => {
                    break;
                }
                Key::Right => app.tabs.next(),
                Key::Left => app.tabs.previous(),
                _ => {}
            },
            _ => {}
        }
    }
    Ok(())
}
