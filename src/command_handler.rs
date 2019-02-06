use crate::app::{App,Term};
use crate::modes::Mode;

#[allow(unused_variables,unused_assignments)]
pub fn handle_command(app: &mut App, terminal: &mut Term) {
    // Example usage: "q!" will force quit
    let mut force_command = false;
    let command = app.command.clone();
    let mut command_chars = command.chars();
    if command.starts_with(":0x") {
        match i64::from_str_radix(&command[3..], 16) {
            Ok(x) => {
                let mut goto_address: usize = x as usize;
                let filesize = app.files[app.tabs_index].data.len();
                if goto_address >= filesize {
                    if filesize > 0 {
                        goto_address = filesize - 1;
                    }
                    else {
                        goto_address = 0;
                    }
                }
                app.files[app.tabs_index].cursor.goto(goto_address as usize);
            }
            Err(_e) => {
                // TODO: Add error messages
            }
        }
    }
    if command.starts_with(":e ") {
        if let Err(_e) = app.open(&command[3..]) {
        }
        else {
            app.tabs_index = app.files.len() - 1;
        }
    }
    if command.starts_with(":w ") {
        if let Err(_e) = app.write(&command[3..]) {
            //TODO: handle errors
        }
    }
    match command.trim() {
        ":bnext" => {
            app.tab_next();
        }
        ":bprev" => {
            app.tab_previous();
        }
        ":bd" => {
            app.files.remove(app.tabs_index);
            if app.tabs_index == app.files.len() {
                app.tabs_index -= 1;
            }
            if app.files.is_empty() {
                app.mode = Mode::Quit;
                return;
            }
        }
        _ => {
            match command_chars.next() {
                Some(':') => {
                    for (index, c) in command_chars.enumerate() {
                        match c {
                            'q' => {
                                // NOTE: Unless forced, quit may be undone later
                                app.mode = Mode::Quit;
                            }
                            'w' => {
                                if let Err(_e) = app.write(None) {
                                    //TODO: Add error logging
                                }
                            }
                            'a' => {
                                // TODO: Handle marking all
                            }
                            '!' => {
                                if index == 0 {
                                    app.mode = Mode::Bash;
                                    app.command = command[2..].to_string();
                                    return;
                                }
                                else {
                                    force_command = true;
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Some('/') => {

                }
                _ => {}
            }
        }
    }
    
    // TODO: Add checking if file needs to be saved + check for force quit
}
