use crate::file::File as NailFile;
use crate::nail::get_title_view;
use crate::app::App;

use tui::widgets::Text;

enum Tab {
    Title,
    File(NailFile),
    Help,
}

impl Tab {
    fn view(&self, app: &mut App) -> Vec<Text<'static>> {
        match self {
            Tab::Title => {
                get_title_view()
            }
            Tab::Help => {
                vec![]
            }
            Tab::File(f) => {
                f.hex_view(app)
            }
        }
    }

    fn title(&self) -> &str {
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
}
