use tui::widgets::Text;
use tui::style::{Style,Color};
use std::str;
use crate::util::HexCursor;

pub struct File<'a> {
    pub name: &'a str,
    pub path: &'a str,
    pub data: Vec<u8>,
    pub cursor: HexCursor,
    pub scroll_y: usize,
}

impl<'a> File<'a> {
    pub fn hex_view(&self, num_lines : usize) -> Vec<Text<'static>> {
        let mut view = 
            self.data
            .chunks(0x10)
            .skip(self.scroll_y / 0x10)
            .take(num_lines)
            .enumerate()
            .map(|(i, data)| 
                vec![
                 Text::styled(
                     format!("{:08X} ", self.scroll_y + (i * 0x10)),
                     Style::default().fg(Color::Black)
                 ),
                 Text::raw(
                    format!(
                         "{:<47}",
                         data.iter()
                             .map(|byte : &u8| format!("{:02X}", byte))
                             .collect::<Vec<String>>()
                             .join(" ")
                    ).to_string()
                ),
                Text::raw("  "),
                Text::raw(format!("{}\n", data.iter()
                                  .map(|byte| (match *byte {
                                      0...0x1F | 0x80...0xA0 | 0x7F => ".".to_string(),
                                      _ => (*byte as char).to_string()
                                  }))
                              .collect::<Vec<String>>()
                              .join("")))
                ])
            .flatten()
            .collect::<Vec<Text<'static>>>();
        view.insert(0, Text::styled(
                "         00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F  0123456789ABCDEF\n",
                Style::default().fg(Color::Black)
            ));
        view
    }
    // cursorX = 10 + ((file.cursor.pos.0 / 2) * 3) + (file.cursor.pos.0 % 2)
}
