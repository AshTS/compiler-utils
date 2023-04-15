use std::str::Lines;

use crate::{Location, Span, FileWalker, ErrorLevel};

const CLEAR: &str = "\x1b[0m";
const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
const CYAN: &str = "\x1b[36m";
const WHITE: &str = "\x1b[37m";


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorDisplaySettings {
    pub colored: bool
}

impl std::default::Default for ErrorDisplaySettings {
    fn default() -> Self {
        Self { colored: true }
    }
}

#[derive(Debug, Clone)]
pub struct ErrorRender<'filedata, 'a> {
    level: ErrorLevel,
    settings: &'a ErrorDisplaySettings,
    message: &'a str,
    primary_location: &'a Location<'filedata>,
    notes: Vec<Note<'filedata, 'a>>,
    walker: &'a FileWalker<'filedata>
}

impl<'filedata, 'a> ErrorRender<'filedata, 'a> {
    pub fn new(level: ErrorLevel, settings: &'a ErrorDisplaySettings, message: &'a str, primary_location: &'a Location<'filedata>, mut notes: Vec<Note<'filedata, 'a>>, walker: &'a FileWalker<'filedata>) -> Self {
        // Now we need to rely on the notes being in sorted order, so we will need to do that first
        notes.sort_by(|a, b| match a.span.location.line.cmp(&b.span.location.line) {
            std::cmp::Ordering::Equal => b.span.location.column.cmp(&a.span.location.column),
            default => default
        });
        
        Self {
            level,
            settings,
            message,
            primary_location,
            notes,
            walker
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineDisplay<'filedata, 'a> {
    pub line_span: Span<'filedata>,
    settings: &'a ErrorDisplaySettings,
}

#[derive(Debug, Clone)]
pub struct RegionRender<'filedata, 'a> {
    settings: &'a ErrorDisplaySettings,
    pub location: Location<'filedata>,
    lines: Lines<'filedata>
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Note<'filedata, 'a> {
    span: &'a Span<'filedata>,
    note: &'a str,
    error_level: ErrorLevel
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoteDisplay<'filedata, 'a> {
    pub span: &'a Span<'filedata>,
    settings: &'a ErrorDisplaySettings,
    note: &'a str,
    color: ErrorLevel
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiNoteDisplay<'filedata, 'a> {
    notes: Vec<&'a Note<'filedata, 'a>>,
    settings: &'a ErrorDisplaySettings,
}

impl<'filedata, 'a> MultiNoteDisplay<'filedata, 'a> {
    pub fn new(settings: &'a ErrorDisplaySettings, notes: &'a [Note<'filedata, 'a>], line: usize) -> Self {
        let mut notes: Vec<_> = notes.iter().filter(|v| v.span.location.line == line).collect();
        notes.sort_by(|a, b| b.span.location.column.cmp(&a.span.location.column));

        Self {
            settings,
            notes
        }
    }
}


impl<'filedata, 'a> Note<'filedata, 'a> {
    pub fn new(span: &'a Span<'filedata>, text: &'a str, error_level: ErrorLevel) -> Self {
        Self {
            span,
            note: text,
            error_level
        }
    }
}

impl<'filedata, 'a> NoteDisplay<'filedata, 'a> {
    pub fn new(span: &'a Span<'filedata>, settings: &'a ErrorDisplaySettings, note: &'a str, color: ErrorLevel) -> Self {
        Self {
            span,
            settings,
            note,
            color
        }
    }

    pub fn from_note(settings: &'a ErrorDisplaySettings, note: &Note<'filedata, 'a>) -> Self {
        Self {
            span: note.span,
            settings,
            note: note.note,
            color: note.error_level
        }
    }
}

impl<'filedata, 'a: 'filedata> RegionRender<'filedata, 'a> {
    pub fn new(settings: &'a ErrorDisplaySettings, span: &'a Span<'filedata>, walker: &'a FileWalker<'filedata>, width: usize) -> Self {
        let region_span = walker.expand_span(span, width);

        Self {
            settings,
            location: region_span.location,
            lines: region_span.data.lines(),
        }
    }
}

impl<'filedata, 'a> std::iter::Iterator for RegionRender<'filedata, 'a> {
    type Item = LineDisplay<'filedata, 'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let this_location = self.location;
        self.location.line += 1;
        self.location.column = 0;

        self.lines.next().map(|line| 
            LineDisplay{
                line_span: Span::from_components(this_location, line),
                settings: self.settings
            })
    }
}

impl<'filedata, 'a> std::fmt::Display for LineDisplay<'filedata, 'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let clear: &str = if self.settings.colored { CLEAR } else { "" };
        let cyan: &str = if self.settings.colored { CYAN } else { "" };

        write!(f, "{cyan}{:3} |{clear}{}", self.line_span.location.line + 1, self.line_span.data)?;

        Ok(())
    }
}

impl<'filedata, 'a> std::fmt::Display for NoteDisplay<'filedata, 'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let clear: &str = if self.settings.colored { CLEAR } else { "" };
        let cyan: &str = if self.settings.colored { CYAN } else { "" };
        let red: &str = if self.settings.colored { RED } else { "" };
        let yellow: &str = if self.settings.colored { YELLOW } else { "" };

        let color = match self.color {
            ErrorLevel::Error => red,
            ErrorLevel::Warning => yellow,
            ErrorLevel::Info => cyan,
        };

        let length = self.span.location.column;

        write!(f, "{cyan}    |{:1$}{color}", "", length)?;

        for _ in 0..self.span.data.chars().count() {
            write!(f, "^")?;
        }

        write!(f, " {}{clear}", self.note)?;

        Ok(())
    }
}

impl<'filedata, 'a> std::fmt::Display for MultiNoteDisplay<'filedata, 'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let clear: &str = if self.settings.colored { CLEAR } else { "" };
        let cyan: &str = if self.settings.colored { CYAN } else { "" };
        let red: &str = if self.settings.colored { RED } else { "" };
        let yellow: &str = if self.settings.colored { YELLOW } else { "" };

        for (i, note) in self.notes.iter().enumerate() {
            if i != 0 {
                writeln!(f)?;
            }

            let color = match note.error_level {
                ErrorLevel::Error => red,
                ErrorLevel::Warning => yellow,
                ErrorLevel::Info => cyan,
            };
    
            let length = note.span.location.column;
    
            write!(f, "{cyan}    |{:1$}{color}", "", length)?;
    
            for _ in 0..note.span.data.chars().count() {
                write!(f, "^")?;
            }
    
            write!(f, " {}{clear}", note.note)?;
        }

        Ok(())
    }
}

impl<'filedata, 'a> std::fmt::Display for ErrorRender<'filedata, 'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let clear: &str = if self.settings.colored { CLEAR } else { "" };
        let cyan: &str = if self.settings.colored { CYAN } else { "" };
        let red: &str = if self.settings.colored { RED } else { "" };
        let yellow: &str = if self.settings.colored { YELLOW } else { "" };
        let white: &str = if self.settings.colored { WHITE } else { "" };

        match self.level {
            ErrorLevel::Error => write!(f, "{red}error{white}: "),
            ErrorLevel::Warning => write!(f, "{yellow}warning{white}: "),
            ErrorLevel::Info => write!(f, "{cyan}info{white}: "),
        }?;

        writeln!(f, "{}{cyan}", self.message)?;
        writeln!(f, "   --> {clear}{}", self.primary_location)?;

        let mut next_line_needed = 0;

        // We rely here on the notes being sorted, this is done by having the only way to construct this object be by sorting the notes
        for note in &self.notes {
            let current_renderer = RegionRender::new(self.settings, note.span, self.walker, 1);

            for line in current_renderer {
                if line.line_span.location.line < next_line_needed { continue; }
                writeln!(f, "{}", line)?;
                next_line_needed = line.line_span.location.line + 1;

                let mut line_note = None;

                for note in &self.notes {
                    if note.span.location.line == line.line_span.location.line {
                        if line_note.is_none() {
                            line_note = Some(note);
                        }
                        else {
                            line_note = None;
                            writeln!(f, "{}", MultiNoteDisplay::new(self.settings, &self.notes, note.span.location.line))?;
                            break;
                        }
                    }
                }

                if let Some(note) = line_note {
                    writeln!(f, "{}", NoteDisplay::from_note(self.settings,note))?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn line_display_no_color() {
        let settings = ErrorDisplaySettings {
            colored: false
        };

        let line_display = LineDisplay {
            line_span: Span {
                location: Location {
                    column: 0,
                    line: 41,
                    filename: "input.txt",
                },
                data: "Hello World!",
            },
            settings: &settings,
        };

        assert_eq!(line_display.to_string(), " 42 |Hello World!");
    }

    #[test]
    fn line_display_color() {
        let settings = ErrorDisplaySettings {
            colored: true
        };

        let line_display = LineDisplay {
            line_span: Span {
                location: Location {
                    column: 0,
                    line: 41,
                    filename: "input.txt",
                },
                data: "Hello World!",
            },
            settings: &settings,
        };

        assert_eq!(line_display.to_string(), format!("{CYAN} 42 |{CLEAR}Hello World!"));
    }

    #[test]
    fn region_render() {
        let settings = ErrorDisplaySettings::default();

        let input = "ABC\n DEF\nGHI\n JKL";
        let walker = FileWalker::from_data(input, "input.txt");

        let inner_span = Span {
            location: Location { column: 0, line: 2, filename: "input.txt" },
            data: &input[10..12],
        };

        let mut region_render0 = RegionRender::new(&settings, &inner_span, &walker, 0);

        assert_eq!(region_render0.next(), Some(LineDisplay { line_span: Span { location: 
            Location { column: 0, line: 2, filename: "input.txt" }, data: "GHI" }, settings: &settings }));
        assert_eq!(region_render0.next(), None);


        let mut region_render1 = RegionRender::new(&settings, &inner_span, &walker, 1);

        assert_eq!(region_render1.next(), Some(LineDisplay { line_span: Span { location: 
            Location { column: 0, line: 1, filename: "input.txt" }, data: " DEF" }, settings: &settings }));
        assert_eq!(region_render1.next(), Some(LineDisplay { line_span: Span { location: 
            Location { column: 0, line: 2, filename: "input.txt" }, data: "GHI" }, settings: &settings }));
        assert_eq!(region_render1.next(), Some(LineDisplay { line_span: Span { location: 
            Location { column: 0, line: 3, filename: "input.txt" }, data: " JKL" }, settings: &settings }));
        assert_eq!(region_render1.next(), None);

        let mut region_render2 = RegionRender::new(&settings, &inner_span, &walker, 2);

        assert_eq!(region_render2.next(), Some(LineDisplay { line_span: Span { location: 
            Location { column: 0, line: 0, filename: "input.txt" }, data: "ABC" }, settings: &settings }));
        assert_eq!(region_render2.next(), Some(LineDisplay { line_span: Span { location: 
            Location { column: 0, line: 1, filename: "input.txt" }, data: " DEF" }, settings: &settings }));
        assert_eq!(region_render2.next(), Some(LineDisplay { line_span: Span { location: 
            Location { column: 0, line: 2, filename: "input.txt" }, data: "GHI" }, settings: &settings }));
        assert_eq!(region_render2.next(), Some(LineDisplay { line_span: Span { location: 
            Location { column: 0, line: 3, filename: "input.txt" }, data: " JKL" }, settings: &settings }));
        assert_eq!(region_render2.next(), None);
    }
}
