use termion::input::MouseTerminal;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::Terminal;
use tui::layout::Rect;

use crate::modes::Mode;
use crate::file::File;
use crate::util::TabsState;

pub struct App<'a> {
    pub tabs: TabsState<'a>,
    pub files: Vec<File<'a>>,
    pub mode: Mode,
    pub command: String,
    pub size: Rect
}

pub type Term = Terminal<
                TermionBackend<
                AlternateScreen<
                MouseTerminal<
                termion::raw::RawTerminal<std::io::Stdout>>>>>;
