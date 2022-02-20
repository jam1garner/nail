use std::fs;
use std::io;
use std::io::prelude::*;
use std::path::Path;

use termion::input::MouseTerminal;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::Rect;
use tui::Terminal;

use crate::file::File;
use crate::modes::Mode;
use crate::tabs::Tab;
use crate::tilde_expand::tilde_expand;
use crate::util::HexCursor;

pub struct App {
    pub tabs: Vec<Tab>,
    pub mode: Mode,
    pub command: String,
    pub size: Rect,
    pub tabs_index: usize,
    pub line_count: usize,
    pub options: AppOptions,
}

impl App {
    pub fn open(&mut self, filename: &str) -> io::Result<()> {
        let expanded_path = tilde_expand(filename).unwrap_or_default();
        let mut data: Vec<u8>;
        match fs::File::open(expanded_path) {
            Ok(mut f) => {
                data = Vec::new();
                f.read_to_end(&mut data)?;
            }
            Err(_e) => {
                data = vec![];
            }
        }
        let file = File {
            name: if let Some(s) = Path::new(filename).file_name() {
                s.to_str().unwrap().to_string()
            } else {
                filename.to_string()
            },
            path: filename.to_string(),
            cursor: HexCursor::new((0, 0)),
            data,
            scroll_y: 0x10,
        };
        if self.tabs.len() == 1 {
            if let Tab::Title = self.tabs[0] {
                self.tabs.remove(0);
            }
        }
        self.tabs.push(Tab::File(file));
        Ok(())
    }

    pub fn write<'a, T: Into<Option<&'a str>>>(&mut self, filename: T) -> io::Result<()> {
        if let Tab::File(current_file) = &self.tabs[self.tabs_index] {
            let mut f = fs::File::create(filename.into().unwrap_or(&current_file.path[..]))?;
            f.write_all(&current_file.data[..])?;
            f.sync_all()?;
        }
        Ok(())
    }

    pub fn goto_tab(&mut self, idx: usize) {
        if idx < self.tabs.len() {
            self.tabs_index = idx;
        }
    }

    pub fn tab_next(&mut self) {
        self.tabs_index = (self.tabs_index + 1) % self.tabs.len();
    }

    pub fn tab_previous(&mut self) {
        if self.tabs_index > 0 {
            self.tabs_index -= 1;
        } else {
            self.tabs_index = self.tabs.len() - 1;
        }
    }

    pub fn tab_titles(&mut self) -> Vec<&str> {
        self.tabs.iter().map(|x| x.title()).collect::<Vec<&str>>()
    }

    pub fn current_tab(&self) -> &Tab {
        &self.tabs[self.tabs_index]
    }
}

pub struct AppOptions {
    pub big_endian: bool,
    pub type_inspector: bool,
}

impl AppOptions {
    pub fn new() -> AppOptions {
        AppOptions {
            big_endian: false,
            type_inspector: true,
        }
    }
}

pub type Term = Terminal<
    TermionBackend<AlternateScreen<MouseTerminal<termion::raw::RawTerminal<std::io::Stdout>>>>,
>;
