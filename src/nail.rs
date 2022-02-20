use tui::style::{Color, Style};
use tui::text::{Span, Spans};

static OPEN_TEXT: &str = r#"
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
                                                                /##/\
                                                               /##/ '\
                                                               /#/ \ :\
                                                               ''   \  \
                                                                     \' \
                                                                      \ .\
                                                                       '  '
                                                                       |' |
                                                                       |.`|
                                                                       |;:|"#;

static HELP_TEXT: &str = r#"Commands:
---------
^:e [file]^ - open [file] as new buffer, creates a new file if it doesn't exist
^:q^ - quit
^:w^ [file] - write to [file], default is the path opened from
^:bnext/:bprev^ - next/previous "buffer" (tab)
^:bd^ - buffer delete
^:topen/:tclose/:ttoggle^ - open, close or toggle type inspector
^:0x[hex number]^ - goto offset [hex number] in the current file
^:help^ - open help menu

Keybinds:
---------
^h/j/k/l^ - left/down/up/right (arrows keys also work)
^shift+r^ - enter replace mode (from default mode)
^w/b^ - move forwards/backwards to nearest 4-byte boundary
^shift+g^ - jump to bottom of buffer
^:^ - enter command mode
^i^ - enter insert mode (WIP)
^/^ - enter command mode (for search)
"#;

pub fn get_title_view() -> Vec<Spans<'static>> {
    vec![Spans::from(Span::raw(OPEN_TEXT))]
}

pub fn get_help_view() -> Vec<Spans<'static>> {
    HELP_TEXT
        .split('^')
        .enumerate()
        .map(|(i, text)| {
            if i % 2 == 0 {
                Spans::from(Span::raw(text))
            } else {
                Spans::from(Span::styled(text, Style::default().fg(Color::Red)))
            }
        })
        .collect()
}
