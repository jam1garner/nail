use tui::widgets::Text;

pub static OPEN_TEXT: &str =
"
                     `7MN.   `7MF'     db      `7MMF'`7MMF'
                       MMN.    M      ;MM:       MM    MM
                       M YMb   M     ,V^MM.      MM    MM
                       M  `MN. M    ,M  `MM      MM    MM
                       M   `MM.M    AbmmmqMA     MM    MM      ,
                       M     YMM   A'     VML    MM    MM     ,M
                     .JML.    YM .AMA.   .AMMA..JMML..JMMmmmmMMM `
                     ===========================================
                              Hex Editor - Version 0.1
                     ===========================================
                                                                   ,`,
                                                                  /##/
                                                                 /##|
                                                                /##/\\
                                                               /##/ '\\
                                                               /#/ \\ :\\
                                                               ''   \\  \\
                                                                     \\' \\
                                                                      \\ .\\
                                                                       '  '
                                                                       |' |
                                                                       |.`|
                                                                       |;:|";

pub fn get_title_view() -> Vec<Text<'static>> {
    vec![Text::raw(OPEN_TEXT)]
}
