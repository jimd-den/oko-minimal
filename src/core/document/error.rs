//! One refusal from the document parser, with the line it came from.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError {
    pub line: usize,
    pub problem: String,
}

impl ParseError {
    pub(crate) fn new(line: usize, problem: impl Into<String>) -> Self {
        Self {
            line,
            problem: problem.into(),
        }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "line {}: {}", self.line, self.problem)
    }
}
