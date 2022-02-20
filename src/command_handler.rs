use crate::app::{App, Term};
use crate::modes::Mode;
use crate::tabs::Tab;

fn handle_set(app: &mut App, option: &str) {
    match option {
        "be" | "bigendian" => app.options.big_endian = true,
        "le" | "littlendian" => app.options.big_endian = false,
        _ => {}
    }
}

#[allow(unused_variables, unused_assignments)]
pub fn handle_command(app: &mut App, terminal: &mut Term) {
    // Example usage: "q!" will force quit
    let mut force_command = false;
    let command = app.command.clone();
    let mut command_chars = command.chars();
    app.command = String::new();
    if let Some(data) = command.strip_prefix(":0x") {
        match i64::from_str_radix(data, 16) {
            Ok(x) => {
                let mut goto_address: usize = x as usize;
                if let Tab::File(current_file) = &mut app.tabs[app.tabs_index] {
                    let filesize = current_file.data.len();
                    if goto_address >= filesize {
                        if filesize > 0 {
                            goto_address = filesize - 1;
                        } else {
                            goto_address = 0;
                        }
                    }
                    current_file.cursor.goto(goto_address as usize);
                }
            }
            Err(_e) => {
                // TODO: Add error messages
            }
        }
    }
    if let Some(data) = command.strip_prefix(":e ") {
        if let Err(_e) = app.open(data) {
        } else {
            app.tabs_index = app.tabs.len() - 1;
        }
    }
    if let Some(data) = command.strip_prefix(":w ") {
        if let Err(_e) = app.write(data) {
            //TODO: handle errors
        }
        return;
    }
    if let Some(cmd) = command.strip_prefix(":set ") {
        handle_set(app, cmd)
    }
    match command.trim() {
        ":bnext" | ":bn" => {
            app.tab_next();
        }
        ":bprev" | ":bp" => {
            app.tab_previous();
        }
        ":bd" => {
            app.tabs.remove(app.tabs_index);
            if app.tabs_index == app.tabs.len() {
                app.tabs_index -= 1;
            }
            if app.tabs.is_empty() {
                app.mode = Mode::Quit;
            }
        }
        ":topen" => {
            app.options.type_inspector = true;
        }
        ":tclose" => {
            app.options.type_inspector = false;
        }
        ":ttoggle" => {
            app.options.type_inspector = !app.options.type_inspector;
        }
        ":help" => {
            if app.tabs.len() == 1 {
                if let Tab::Title = app.tabs[0] {
                    app.tabs.remove(0);
                }
            }
            app.tabs.push(Tab::Help);
            app.tabs_index = app.tabs.len() - 1;
        }
        cmd if cmd.starts_with(":b") => {
            if let Ok(num) = cmd[2..].parse::<usize>() {
                if num > 0 {
                    app.goto_tab(num - 1);
                }
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
                                } else {
                                    force_command = true;
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Some('/') => {
                    let search_query = &command[1..];
                }
                _ => {}
            }
        }
    }

    // TODO: Add checking if file needs to be saved + check for force quit
}
