use crate::app::{App,Term};
use crate::modes::Mode;

pub fn handle_command(app: &mut App, terminal: &mut Term) {
    let command = app.command.split(" ").nth(0).unwrap();
    if command == "q" {
        app.mode = Mode::Quit;
    }   
}
