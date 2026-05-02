use crate::Result;

#[derive(Debug, Clone)]
pub struct CodeWriter {
    pub s: String,
    indent: usize,
}

impl Default for CodeWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeWriter {
    pub fn new() -> Self {
        Self {
            s: String::new(),
            indent: 0,
        }
    }

    pub fn indent(&mut self) {
        self.indent += 1;
    }

    pub fn dedent(&mut self) {
        self.indent = self.indent.saturating_sub(1);
    }

    pub fn writeln(&mut self, line: &str) -> Result<()> {
        for _ in 0..self.indent {
            self.s.push_str("    ");
        }
        self.s.push_str(line);
        self.s.push('\n');
        Ok(())
    }

    pub fn into_string(self) -> String {
        format!("{}\n", self.s.trim_end())
    }
}
