use crate::file::File as NailFile;
use crate::nail::{get_title_view, get_help_view};
use crate::app::App;

use tui::widgets::Text;

pub enum Tab {
    Title,
    File(NailFile),
    Help,
}

impl Tab {
    pub fn view(&self, app: &App) -> Vec<Text<'static>> {
        match self {
            Tab::Title => {
                get_title_view()
            }
            Tab::Help => {
                get_help_view()
            }
            Tab::File(f) => {
                f.hex_view(app)
            }
        }
    }

    pub fn title(&self) -> &str {
        match self {
            Tab::Title => {
                "title"
            }
            Tab::Help => {
                "help"
            }
            Tab::File(f) => {
                &f.name[..]
            }
        }
    }

    pub fn long_title(&self) -> &str {
        match self {
            Tab::File(f) => {
                &f.path[..]
            }
            _ => self.title()
        }
    }
}
