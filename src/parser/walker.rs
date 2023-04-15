use crate::Span;
use super::Location;

/// Walks through a file, producing characters one at a time
#[derive(Debug, Clone)]
pub struct FileWalker<'filedata> {
    all_data: &'filedata str,
    filename: &'filedata str,
    current_byte_index: usize,
    column: usize,
    line: usize
}

/// A marker for a location within a file
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FileLocationMarker {
    index: usize,
    column: usize,
    line: usize
}

impl<'filedata> FileWalker<'filedata> {
    /// Construct a new `FileWalker` from a name and data
    pub fn from_data(data: &'filedata str, filename: &'filedata str) -> Self {
        Self {
            all_data: data,
            filename,
            current_byte_index: 0,
            column: 0,
            line: 0
        }
    }

    /// Construct a `FileWalker` from a `Span`
    pub fn from_span(span: &Span<'filedata>) -> Self {
        Self {
            all_data: span.data,
            filename: span.location.filename,
            current_byte_index: 0,
            column: span.location.column,
            line: span.location.line,
        }
    }

    /// Get the location of the currently referenced character
    pub fn current_location(&self) -> Location<'filedata> {
        Location::from_components(self.column, self.line, self.filename)
    }

    /// Get the location of the currently referenced character as a `FileLocationMaker`
    pub fn get_marker(&self) -> FileLocationMarker {
        FileLocationMarker {
            index: self.current_byte_index,
            line: self.line,
            column: self.column
        }
    }

    /// Get the string currently pointed to
    pub fn current_string(&self) -> &'filedata str {
        if self.current_byte_index >= self.all_data.len() {
            ""
        }
        else {
            unsafe { std::str::from_utf8_unchecked(&self.all_data.as_bytes()[self.current_byte_index..]) } // .expect("The unicode assumption was violated")
        }
    }

    /// Step forward by one character if possible, return the character stepped over, otherwise return None
    pub fn step(&mut self) -> Option<char> {
        // Get the first character
        let character = self.current_string().chars().next();

        if let Some(c) = character {
            self.current_byte_index += c.len_utf8();
            if c == '\n' {
                self.line += 1;
                self.column = 0;
            }
            else {
                self.column += 1;
            }
        }

        character
    }

    /// Return to a previous location in the file (using a `FileLocationMarker`) and return true, if the `FileLocationMarker` does not point to the boundary of a character, return false and do not move the current character back
    pub fn pop_back(&mut self, marker: FileLocationMarker) -> bool {
        if self.all_data.is_char_boundary(marker.index) {
            self.current_byte_index = marker.index;
            self.line = marker.line;
            self.column = marker.column;
            true
        }
        else {
            false
        }
    }

    /// Get the span representing a portion of the file from a given marker to the cursor (not including the character the cursor is pointing at), returns none if the marker does not point to a valid unicode boundary, or if the marker is after the current location.
    pub fn span_from_marker_to_here(&self, marker: FileLocationMarker) -> Option<Span<'filedata>> {
        if marker.index == self.current_byte_index {
            Some(Span::from_components(self.current_location(), ""))
        }
        else if !self.all_data.is_char_boundary(marker.index) || marker.index > self.current_byte_index {
            None
        }
        else {
            let location = Location::from_components(marker.column, marker.line, self.filename);
            let data = Some(&self.all_data[marker.index..self.current_byte_index]).expect("The unicode assumption was violated");

            Some(Span::from_components(location, data))
        }
    }

    /// Get the location of a marker in the file, or None if the marker is not pointing to a character
    pub fn get_location_of_marker(&self, marker: FileLocationMarker) -> Option<Location<'filedata>> {
        if self.all_data.is_char_boundary(marker.index) {
            Some(Location::from_components(marker.column, marker.line, self.filename))
        }
        else {
            None
        }
    }

    /// Get a span a certain number of lines (potentially) away from the line the span given is on
    pub fn expand_span(&self, span: &Span, lines_away: usize) -> Span {
        // Get the index of the span within the file
        assert!(span.data.as_ptr() as usize >= self.all_data.as_ptr() as usize);
        let span_byte_index = span.data.as_ptr() as usize - self.all_data.as_ptr() as usize;
        assert!(span_byte_index <= self.all_data.len());

        // We need to start counting back a number of lines... if doing so doesn't just bring us back to the beginning.
        let start_line_number = span.location.line.max(lines_away) - lines_away;
        
        // We can thus construct a location at the start of that line
        let location = Location::from_components(0, start_line_number, self.filename);

        // Now, we can walk back to the index of the start of the desired line
        let start_index = if start_line_number == 0 { 0 } else {
            let mut lines_remaining = span.location.line - start_line_number + 1;
            let mut current_index = span_byte_index;

            while current_index > 0 {
                current_index -= 1;
                while current_index > 0 && !self.all_data.is_char_boundary(current_index) {}
                if self.all_data[current_index.. current_index + 2].starts_with('\n') {
                    lines_remaining -= 1;
                    if lines_remaining == 0 {
                        current_index += 1;
                        break;
                    }
                }

            }

            current_index
        };

        // Next, we need to walk forward to find the ending index
        let mut lines_remaining = lines_away + 1;
        let mut current_index = span_byte_index;
        for c in self.all_data[span_byte_index..].chars() {
            if c == '\n' {
                lines_remaining -= 1;
                if lines_remaining == 0 {
                    break;
                }
            }
            current_index += c.len_utf8();
        }

        Span::from_components(location, &self.all_data[start_index..current_index])
    }
}

#[cfg(test)]
mod test {
    use crate::{FileWalker, Location, Span};

    #[test]
    pub fn simple_walk_step() {
        let data = "Möbius";
        let mut walker = FileWalker::from_data(data, "hello.txt");

        assert_eq!(walker.step(), Some('M'));
        assert_eq!(walker.step(), Some('ö'));
        assert_eq!(walker.step(), Some('b'));
        assert_eq!(walker.step(), Some('i'));
        assert_eq!(walker.step(), Some('u'));
        assert_eq!(walker.step(), Some('s'));
        assert_eq!(walker.step(), None);
        assert_eq!(walker.step(), None);
    }

    #[test]
    pub fn simple_walk_current_str() {
        let data = "Möbius";
        let mut walker = FileWalker::from_data(data, "hello.txt");

        assert_eq!(walker.current_string(), "Möbius");
        walker.step();
        assert_eq!(walker.current_string(), "öbius");
        walker.step();
        assert_eq!(walker.current_string(), "bius");
        walker.step();
        assert_eq!(walker.current_string(), "ius");
        walker.step();
        assert_eq!(walker.current_string(), "us");
        walker.step();
        assert_eq!(walker.current_string(), "s");
        walker.step();
        assert_eq!(walker.current_string(), "");
        walker.step();
        assert_eq!(walker.current_string(), "");
    }

    #[test]
    pub fn simple_walk_current_location() {
        let data = "Möbius";
        let mut walker = FileWalker::from_data(data, "hello.txt");

        assert_eq!(walker.current_location(), Location::from_components(0, 0, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_location(), Location::from_components(1, 0, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_location(), Location::from_components(2, 0, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_location(), Location::from_components(3, 0, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_location(), Location::from_components(4, 0, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_location(), Location::from_components(5, 0, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_location(), Location::from_components(6, 0, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_location(), Location::from_components(6, 0, "hello.txt"));
    }

    #[test]
    pub fn simple_walk_pop_back() {
        let data = "Möbius";
        let mut walker = FileWalker::from_data(data, "hello.txt");

        let start = walker.get_marker();
        assert_eq!(walker.current_string(), "Möbius");
        assert_eq!(walker.current_location(), Location::from_components(0, 0, "hello.txt"));
        walker.step();
        let at_unicode = walker.get_marker();
        assert_eq!(walker.current_string(), "öbius");
        assert_eq!(walker.current_location(), Location::from_components(1, 0, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_string(), "bius");
        assert_eq!(walker.current_location(), Location::from_components(2, 0, "hello.txt"));
        walker.step();
        let middle = walker.get_marker();
        assert_eq!(walker.current_string(), "ius");
        assert_eq!(walker.current_location(), Location::from_components(3, 0, "hello.txt"));

        walker.pop_back(at_unicode);
        assert_eq!(walker.current_string(), "öbius");
        assert_eq!(walker.current_location(), Location::from_components(1, 0, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_string(), "bius");
        assert_eq!(walker.current_location(), Location::from_components(2, 0, "hello.txt"));

        walker.pop_back(start);
        assert_eq!(walker.current_string(), "Möbius");
        assert_eq!(walker.current_location(), Location::from_components(0, 0, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_string(), "öbius");
        assert_eq!(walker.current_location(), Location::from_components(1, 0, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_string(), "bius");
        assert_eq!(walker.current_location(), Location::from_components(2, 0, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_string(), "ius");
        assert_eq!(walker.current_location(), Location::from_components(3, 0, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_string(), "us");
        assert_eq!(walker.current_location(), Location::from_components(4, 0, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_string(), "s");
        assert_eq!(walker.current_location(), Location::from_components(5, 0, "hello.txt"));

        walker.pop_back(middle);
        assert_eq!(walker.current_string(), "ius");
        assert_eq!(walker.current_location(), Location::from_components(3, 0, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_string(), "us");
        assert_eq!(walker.current_location(), Location::from_components(4, 0, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_string(), "s");
        assert_eq!(walker.current_location(), Location::from_components(5, 0, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_string(), "");
        assert_eq!(walker.current_location(), Location::from_components(6, 0, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_string(), "");
        assert_eq!(walker.current_location(), Location::from_components(6, 0, "hello.txt"));
    }

    #[test]
    pub fn line_break_walk_step() {
        let data = "Mö\nbi\r\nus";
        let mut walker = FileWalker::from_data(data, "hello.txt");

        assert_eq!(walker.step(), Some('M'));
        assert_eq!(walker.step(), Some('ö'));
        assert_eq!(walker.step(), Some('\n'));
        assert_eq!(walker.step(), Some('b'));
        assert_eq!(walker.step(), Some('i'));
        assert_eq!(walker.step(), Some('\r'));
        assert_eq!(walker.step(), Some('\n'));
        assert_eq!(walker.step(), Some('u'));
        assert_eq!(walker.step(), Some('s'));
        assert_eq!(walker.step(), None);
        assert_eq!(walker.step(), None);
    }

    #[test]
    pub fn line_break_walk_current_str() {
        let data = "Mö\nbi\r\nus";
        let mut walker = FileWalker::from_data(data, "hello.txt");

        assert_eq!(walker.current_string(), "Mö\nbi\r\nus");
        walker.step();
        assert_eq!(walker.current_string(), "ö\nbi\r\nus");
        walker.step();
        assert_eq!(walker.current_string(), "\nbi\r\nus");
        walker.step();
        assert_eq!(walker.current_string(), "bi\r\nus");
        walker.step();
        assert_eq!(walker.current_string(), "i\r\nus");
        walker.step();
        assert_eq!(walker.current_string(), "\r\nus");
        walker.step();
        assert_eq!(walker.current_string(), "\nus");
        walker.step();
        assert_eq!(walker.current_string(), "us");
        walker.step();
        assert_eq!(walker.current_string(), "s");
        walker.step();
        assert_eq!(walker.current_string(), "");
        walker.step();
        assert_eq!(walker.current_string(), "");
    }

    #[test]
    pub fn line_break_walk_current_location() {
        let data = "Mö\nbi\r\nus";
        let mut walker = FileWalker::from_data(data, "hello.txt");

        assert_eq!(walker.current_location(), Location::from_components(0, 0, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_location(), Location::from_components(1, 0, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_location(), Location::from_components(2, 0, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_location(), Location::from_components(0, 1, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_location(), Location::from_components(1, 1, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_location(), Location::from_components(2, 1, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_location(), Location::from_components(3, 1, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_location(), Location::from_components(0, 2, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_location(), Location::from_components(1, 2, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_location(), Location::from_components(2, 2, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_location(), Location::from_components(2, 2, "hello.txt"));
    }

    #[test]
    pub fn line_break_walk_pop_back() {
        let data = "Mö\nbi\r\nus";
        let mut walker = FileWalker::from_data(data, "hello.txt");

        let start = walker.get_marker();
        assert_eq!(walker.current_string(), "Mö\nbi\r\nus");
        assert_eq!(walker.current_location(), Location::from_components(0, 0, "hello.txt"));
        walker.step();
        let at_unicode = walker.get_marker();
        assert_eq!(walker.current_string(), "ö\nbi\r\nus");
        assert_eq!(walker.current_location(), Location::from_components(1, 0, "hello.txt"));
        walker.step();
        let before_line = walker.get_marker();
        assert_eq!(walker.current_string(), "\nbi\r\nus");
        assert_eq!(walker.current_location(), Location::from_components(2, 0, "hello.txt"));
        walker.step();
        let after_line = walker.get_marker();
        assert_eq!(walker.current_string(), "bi\r\nus");
        assert_eq!(walker.current_location(), Location::from_components(0, 1, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_string(), "i\r\nus");
        assert_eq!(walker.current_location(), Location::from_components(1, 1, "hello.txt"));
        walker.step();
        let at_carriage_return = walker.get_marker();
        assert_eq!(walker.current_string(), "\r\nus");
        assert_eq!(walker.current_location(), Location::from_components(2, 1, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_string(), "\nus");
        assert_eq!(walker.current_location(), Location::from_components(3, 1, "hello.txt"));

        walker.pop_back(before_line);
        assert_eq!(walker.current_string(), "\nbi\r\nus");
        assert_eq!(walker.current_location(), Location::from_components(2, 0, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_string(), "bi\r\nus");
        assert_eq!(walker.current_location(), Location::from_components(0, 1, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_string(), "i\r\nus");
        assert_eq!(walker.current_location(), Location::from_components(1, 1, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_string(), "\r\nus");
        assert_eq!(walker.current_location(), Location::from_components(2, 1, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_string(), "\nus");
        assert_eq!(walker.current_location(), Location::from_components(3, 1, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_string(), "us");
        assert_eq!(walker.current_location(), Location::from_components(0, 2, "hello.txt"));
        walker.step();
        let right_at_end = walker.get_marker();
        assert_eq!(walker.current_string(), "s");
        assert_eq!(walker.current_location(), Location::from_components(1, 2, "hello.txt"));

        walker.pop_back(start);
        assert_eq!(walker.current_string(), "Mö\nbi\r\nus");
        assert_eq!(walker.current_location(), Location::from_components(0, 0, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_string(), "ö\nbi\r\nus");
        assert_eq!(walker.current_location(), Location::from_components(1, 0, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_string(), "\nbi\r\nus");
        assert_eq!(walker.current_location(), Location::from_components(2, 0, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_string(), "bi\r\nus");
        assert_eq!(walker.current_location(), Location::from_components(0, 1, "hello.txt"));

        walker.pop_back(at_carriage_return);
        assert_eq!(walker.current_string(), "\r\nus");
        assert_eq!(walker.current_location(), Location::from_components(2, 1, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_string(), "\nus");
        assert_eq!(walker.current_location(), Location::from_components(3, 1, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_string(), "us");
        assert_eq!(walker.current_location(), Location::from_components(0, 2, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_string(), "s");
        assert_eq!(walker.current_location(), Location::from_components(1, 2, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_string(), "");
        assert_eq!(walker.current_location(), Location::from_components(2, 2, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_string(), "");
        assert_eq!(walker.current_location(), Location::from_components(2, 2, "hello.txt"));

        walker.pop_back(at_unicode);
        assert_eq!(walker.current_string(), "ö\nbi\r\nus");
        assert_eq!(walker.current_location(), Location::from_components(1, 0, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_string(), "\nbi\r\nus");
        assert_eq!(walker.current_location(), Location::from_components(2, 0, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_string(), "bi\r\nus");
        assert_eq!(walker.current_location(), Location::from_components(0, 1, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_string(), "i\r\nus");
        assert_eq!(walker.current_location(), Location::from_components(1, 1, "hello.txt"));

        walker.pop_back(right_at_end);
        assert_eq!(walker.current_string(), "s");
        assert_eq!(walker.current_location(), Location::from_components(1, 2, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_string(), "");
        assert_eq!(walker.current_location(), Location::from_components(2, 2, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_string(), "");
        assert_eq!(walker.current_location(), Location::from_components(2, 2, "hello.txt"));

        walker.pop_back(after_line);
        assert_eq!(walker.current_string(), "bi\r\nus");
        assert_eq!(walker.current_location(), Location::from_components(0, 1, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_string(), "i\r\nus");
        assert_eq!(walker.current_location(), Location::from_components(1, 1, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_string(), "\r\nus");
        assert_eq!(walker.current_location(), Location::from_components(2, 1, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_string(), "\nus");
        assert_eq!(walker.current_location(), Location::from_components(3, 1, "hello.txt"));
    }

    #[test]
    pub fn line_break_walk_span_from_marker_to_here() {
        let data = "Mö\nbi\r\nus";
        let mut walker = FileWalker::from_data(data, "hello.txt");

        assert_eq!(walker.current_string(), "Mö\nbi\r\nus");
        assert_eq!(walker.current_location(), Location::from_components(0, 0, "hello.txt"));
        walker.step();
        let at_unicode = walker.get_marker();
        assert_eq!(walker.current_string(), "ö\nbi\r\nus");
        assert_eq!(walker.current_location(), Location::from_components(1, 0, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_string(), "\nbi\r\nus");
        assert_eq!(walker.current_location(), Location::from_components(2, 0, "hello.txt"));
        walker.step();
        let later = walker.get_marker();
        assert_eq!(walker.current_string(), "bi\r\nus");
        assert_eq!(walker.current_location(), Location::from_components(0, 1, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_string(), "i\r\nus");
        assert_eq!(walker.current_location(), Location::from_components(1, 1, "hello.txt"));
        assert_eq!(walker.span_from_marker_to_here(at_unicode), Some(Span::from_components(Location::from_components(1, 0, "hello.txt"), "ö\nb")));
        walker.step();
        assert_eq!(walker.current_string(), "\r\nus");
        assert_eq!(walker.current_location(), Location::from_components(2, 1, "hello.txt"));
        walker.step();
        assert_eq!(walker.current_string(), "\nus");
        assert_eq!(walker.current_location(), Location::from_components(3, 1, "hello.txt"));
        assert_eq!(walker.span_from_marker_to_here(at_unicode), Some(Span::from_components(Location::from_components(1, 0, "hello.txt"), "ö\nbi\r")));
        assert_eq!(walker.span_from_marker_to_here(later), Some(Span::from_components(Location::from_components(0, 1, "hello.txt"), "bi\r")));
    }

    #[test]
    pub fn simple_expand_span() {
        let mut walker = FileWalker::from_data("abc\ndef\nghi\njkl\nmno\npqr\nstu\nvwx\nyz0", "input");

        let mut line_spans = Vec::new();

        for (i, line) in walker.all_data.lines().enumerate() {
            line_spans.push(Span::from_components(Location::from_components(0, i, "input"), line));
        }

        loop {
            let marker = walker.get_marker();
            walker.step();
            let Some(span) = walker.span_from_marker_to_here(marker) else { break; };
            
            if span.data.is_empty() { break; }

            for i in 0..10 {
                let expanded = walker.expand_span(&span, i);
                assert!(expanded.data.lines().count() <= 1 + 2 * i);
                assert_eq!(expanded.data.lines().count(), 1 + ((span.location.line + i).min(line_spans.len() - 1)) - (span.location.line.max(i) - i));
                
                if i == 0 {   
                    assert_eq!(expanded, line_spans[span.location.line]);
                }
            }
            
        }
    }
}