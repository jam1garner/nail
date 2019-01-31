use tui::widgets::Text;

pub struct File<'a> {
    pub name: &'a str,
    pub path: &'a str,
    pub data: Vec<u8>,
    pub pos: usize
}

impl<'a> File<'a> {
    pub fn hex_view(&self, num_lines : usize) -> Vec<Text<'static>> {
        self.data
            .chunks(0x10)
            .skip(self.pos / 0x10)
            .take(num_lines)
            .enumerate()
            .map(|(i, data)| Text::raw(format!(
                "{:08X}: {}\n",
                self.pos + (i * 0x10),
                data.iter()
                    .map(|byte : &u8| format!("{:02X}", byte))
                    .collect::<Vec<String>>()
                    .join(" ")
            ).to_string()))
            .collect()
    }
}
