use std::fs;
use std::io;
use std::io::prelude::*;
use std::path::Path;

use termion::input::MouseTerminal;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::Terminal;
use tui::layout::Rect;

use crate::modes::Mode;
use crate::file::File;
use crate::util::HexCursor;

pub struct App {
    pub files: Vec<File>,
    pub mode: Mode,
    pub command: String,
    pub size: Rect,
    pub tabs_index: usize,
    pub line_count: usize,
}

impl App {
    pub fn open(&mut self, filename: &str) -> io::Result<()> {
        let mut f = fs::File::open(filename)?;
        let mut data: Vec<u8> = Vec::new();
        f.read_to_end(&mut data)?;
        let file =
            File {
                name: if let Some(s) = Path::new(filename).file_name(){
                        s.to_str().unwrap().to_string()
                    } else {
                        filename.to_string()
                    },
                path: filename.to_string(),
                cursor: HexCursor::new((0,0)),
                data,
                scroll_y: 0x10
            };
        self.files.push(file);
        Ok(())
    }

    pub fn tab_next(&mut self) {
        self.tabs_index = (self.tabs_index + 1) % self.files.len();
    }

    pub fn tab_previous(&mut self) {
        if self.tabs_index > 0 {
            self.tabs_index -= 1;
        } else {
            self.tabs_index = self.files.len() - 1;
        }
    }

    pub fn tab_titles(&mut self) -> Vec<&str> {
        self.files.iter()
             .map(|x| &x.name[..])
             .collect::<Vec<&str>>()
    }
}

pub type Term = Terminal<
                TermionBackend<
                AlternateScreen<
                MouseTerminal<
                termion::raw::RawTerminal<std::io::Stdout>>>>>;
