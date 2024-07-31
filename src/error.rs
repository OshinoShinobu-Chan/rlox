pub struct Error {
    pub line: usize,
    pub loc: String,
    pub message: String,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[line {}] Error {}, message: {}",
            self.line, self.loc, self.message
        )
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Error {
    pub fn new(line: usize, loc: String, message: String) -> Self {
        Self { line, loc, message }
    }
}
