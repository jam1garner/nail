pub struct File<'a> {
    pub name: &'a str,
    pub path: &'a str,
    pub data: Vec<u8>
}
