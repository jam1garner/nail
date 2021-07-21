pub fn tilde_expand(path: &str) -> Option<String> {
    if &path[0..1] == "~" {
        if let Some(home) = dirs::home_dir() {
            let mut expanded = home.to_string_lossy().into_owned();
            expanded.push_str(&path[1..]);
            Some(expanded)
        } else {
            None
        }
    } else {
        Some(String::from(path))
    }
}
