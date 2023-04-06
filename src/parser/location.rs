/// Holds the location of a token within a file
/// 
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Location<'name> {
    pub column: usize,
    pub line: usize,
    pub filename: &'name str
}


/// Refers to a particular length of data within a file
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span<'filedata> {
    pub location: Location<'filedata>,
    pub data: &'filedata str
}

impl<'name> Location<'name> {
    /// Construct a location from its components
    pub fn from_components(column: usize, line: usize, filename: &'name str) -> Self {
        Self {
            column, line, filename
        }
    }
}

impl<'name> std::cmp::PartialOrd for Location<'name> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.filename != other.filename {
            return None;
        }

        match self.line.partial_cmp(&other.line) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }

        self.column.partial_cmp(&other.column)
    }
}

impl<'name> std::fmt::Display for Location<'name> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "column {} line {} in {}", self.column + 1, self.line + 1, self.filename)
    }
}

impl <'filedata> Span<'filedata> {
    /// Construct a new span from its components
    pub fn from_components(location: Location<'filedata>, data: &'filedata str) -> Self {
        Self {
            location, data
        }
    }
}

impl <'filedata> std::fmt::Display for Span<'filedata> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.data)
    }
}