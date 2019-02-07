use std::cmp::max;
use std::io::prelude::*;
use std::io::Cursor;
use std::io::SeekFrom;

use byteorder::{LittleEndian, BigEndian, ReadBytesExt};
use tui::widgets::Text;
use tui::style::{Style,Color};

use crate::util::HexCursor;
use crate::app::App;

pub struct File {
    pub name: String,
    pub path: String,
    pub data: Vec<u8>,
    pub cursor: HexCursor,
    pub scroll_y: usize,
}

impl File {
    pub fn hex_view(&self, app: &App) -> Vec<Text<'static>> {
        let num_lines = app.line_count;
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
                Text::raw(format!("{}\n",
                              data.iter()
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
        if app.options.type_inspector {
            self.append_type_inspector(app, &mut view);
        }

        view
    }
    // cursorX = 10 + ((file.cursor.pos.0 / 2) * 3) + (file.cursor.pos.0 % 2)

    fn append_type_inspector(&self, app: &App, view: &mut Vec<Text<'static>>) {
        let filesize = self.data.len();
        let loc = self.cursor.loc();
        let mut rdr = Cursor::new(&self.data[..]);
        let sbyte : i8;
        let ubyte : u8;
        let mut sshort: i16 = 0;
        let mut ushort: u16 = 0;
        let mut sword : i32 = 0;
        let mut uword : u32 = 0;
        let mut float : f32 = 0.0;
        let mut sdword: i64 = 0;
        let mut udword: u64 = 0;
        let mut double: f64 = 0.0;
        // s8, u8, s16, u16, s32, u32, f32, s64, u64, f64
        if ((filesize as isize) - (loc as isize)) >= 1 {
            rdr.seek(SeekFrom::Start(loc as u64)).unwrap();
            sbyte = rdr.read_i8().unwrap_or(0);
            rdr.seek(SeekFrom::Start(loc as u64)).unwrap();
            ubyte = rdr.read_u8().unwrap_or(0);
        }
        else {
            // Out of bounds, quit rather than try
            return;
        }
        if app.options.big_endian {
            if filesize - loc >= 2 {
                rdr.seek(SeekFrom::Start(loc as u64)).unwrap();
                sshort = rdr.read_i16::<BigEndian>().unwrap_or(0);
                rdr.seek(SeekFrom::Start(loc as u64)).unwrap();
                ushort = rdr.read_u16::<BigEndian>().unwrap_or(0);
            }
            if filesize - loc >= 4 {
                rdr.seek(SeekFrom::Start(loc as u64)).unwrap();
                sword = rdr.read_i32::<BigEndian>().unwrap_or(0);
                rdr.seek(SeekFrom::Start(loc as u64)).unwrap();
                uword = rdr.read_u32::<BigEndian>().unwrap_or(0);
                rdr.seek(SeekFrom::Start(loc as u64)).unwrap();
                float = rdr.read_f32::<BigEndian>().unwrap_or(0.0);
            }
            if filesize - loc >= 8 {
                rdr.seek(SeekFrom::Start(loc as u64)).unwrap();
                sdword = rdr.read_i64::<BigEndian>().unwrap_or(0);
                rdr.seek(SeekFrom::Start(loc as u64)).unwrap();
                udword = rdr.read_u64::<BigEndian>().unwrap_or(0);
                rdr.seek(SeekFrom::Start(loc as u64)).unwrap();
                double = rdr.read_f64::<BigEndian>().unwrap_or(0.0);
            }
        }
        else {
            if filesize - loc >= 2 {
                rdr.seek(SeekFrom::Start(loc as u64)).unwrap();
                sshort = rdr.read_i16::<LittleEndian>().unwrap_or(0);
                rdr.seek(SeekFrom::Start(loc as u64)).unwrap();
                ushort = rdr.read_u16::<LittleEndian>().unwrap_or(0);
            }
            if filesize - loc >= 4 {
                rdr.seek(SeekFrom::Start(loc as u64)).unwrap();
                sword = rdr.read_i32::<LittleEndian>().unwrap_or(0);
                rdr.seek(SeekFrom::Start(loc as u64)).unwrap();
                uword = rdr.read_u32::<LittleEndian>().unwrap_or(0);
                rdr.seek(SeekFrom::Start(loc as u64)).unwrap();
                float = rdr.read_f32::<LittleEndian>().unwrap_or(0.0);
            }
            if filesize - loc >= 8 {
                rdr.seek(SeekFrom::Start(loc as u64)).unwrap();
                sdword = rdr.read_i64::<LittleEndian>().unwrap_or(0);
                rdr.seek(SeekFrom::Start(loc as u64)).unwrap();
                udword = rdr.read_u64::<LittleEndian>().unwrap_or(0);
                rdr.seek(SeekFrom::Start(loc as u64)).unwrap();
                double = rdr.read_f64::<LittleEndian>().unwrap_or(0.0);
            }
        }

        // Find dword padding amount
        let dword_size = max(max(format!("{}", sdword).len(), format!("{}", udword).len()), 3);
        let unsigned_size = max(format!("{}", uword).len(), 3);
        let signed_size = max(format!("{}", sword).len(), 3);

        // Line 1
        view.push(Text::styled(" u8: ", Style::default().fg(Color::Black)));
        view.push(Text::raw(format!("{:1$} ", ubyte, unsigned_size).to_string()));
        view.push(Text::styled(" i8: ", Style::default().fg(Color::Black)));
        view.push(Text::raw(format!("{:1$} ", sbyte, signed_size).to_string()));
        view.push(Text::styled("u64: ", Style::default().fg(Color::Black)));
        view.push(Text::raw(format!("{:1$} ", udword, dword_size).to_string()));
        view.push(Text::styled("f32: ", Style::default().fg(Color::Black)));
        view.push(Text::raw(format!("{:.4}\n", float).to_string()));

        // Line 2
        view.push(Text::styled("u16: ", Style::default().fg(Color::Black)));
        view.push(Text::raw(format!("{:1$} ", ushort, unsigned_size).to_string()));
        view.push(Text::styled("i16: ", Style::default().fg(Color::Black)));
        view.push(Text::raw(format!("{:1$} ", sshort, signed_size).to_string()));
        view.push(Text::styled("i64: ", Style::default().fg(Color::Black)));
        view.push(Text::raw(format!("{:1$} ", sdword, dword_size).to_string()));
        view.push(Text::styled("f64: ", Style::default().fg(Color::Black)));
        view.push(Text::raw(format!("{:.4}\n", double).to_string()));

        // Line 3
        view.push(Text::styled("u32: ", Style::default().fg(Color::Black)));
        view.push(Text::raw(format!("{:1$} ", uword, unsigned_size).to_string()));
        view.push(Text::styled("i32: ", Style::default().fg(Color::Black)));
        view.push(Text::raw(format!("{:1$} ", sword, signed_size).to_string()));
    }
}
