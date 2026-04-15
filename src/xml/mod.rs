//! Minimal XML parser sufficient for CMSIS-SVD.
//!
//! Limitations (intentional):
//! - No namespaces support (we keep prefixes as part of the name).
//! - No DTD/ENTITY declarations (DOCTYPE is rejected).
//! - Supports comments, processing instructions, CDATA, and standard entities (&lt;...).

use crate::error::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Location {
    pub offset: usize,
    pub line: usize,
    pub column: usize,
}

impl core::fmt::Display for Location {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}:{} (off {})", self.line, self.column, self.offset)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribute {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    Element(Element),
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Element {
    pub name: String,
    pub attrs: Vec<Attribute>,
    pub children: Vec<Node>,
    pub loc: Location,
}

impl Element {
    pub fn attr(&self, name: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|a| a.name == name)
            .map(|a| a.value.as_str())
    }

    pub fn children_elements<'a>(&'a self) -> impl Iterator<Item = &'a Element> + 'a {
        self.children.iter().filter_map(|n| match n {
            Node::Element(e) => Some(e),
            _ => None,
        })
    }

    pub fn child(&self, name: &str) -> Option<&Element> {
        self.children_elements().find(|e| e.name == name)
    }

    pub fn child_text(&self, name: &str) -> Option<&str> {
        self.child(name).and_then(|c| c.text())
    }

    /// Returns concatenated text content (if any).
    pub fn text(&self) -> Option<&str> {
        // Optimization: fast path when there is a single text child.
        if self.children.len() == 1 {
            if let Node::Text(t) = &self.children[0] {
                return Some(t.as_str());
            }
        }
        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub root: Element,
}

impl Document {
    pub fn parse(input: &str) -> Result<Self> {
        Parser::new(input).parse_document()
    }
}

struct Parser<'a> {
    s: &'a [u8],
    i: usize,
    line: usize,
    col: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            s: input.as_bytes(),
            i: 0,
            line: 1,
            col: 1,
        }
    }

    fn loc(&self) -> Location {
        Location {
            offset: self.i,
            line: self.line,
            column: self.col,
        }
    }

    fn eof(&self) -> bool {
        self.i >= self.s.len()
    }

    fn peek(&self) -> Option<u8> {
        self.s.get(self.i).copied()
    }

    fn starts_with(&self, pat: &[u8]) -> bool {
        self.s
            .get(self.i..)
            .map(|tail| tail.starts_with(pat))
            .unwrap_or(false)
    }

    fn bump(&mut self) -> Option<u8> {
        let b = self.peek()?;
        self.i += 1;
        if b == b'\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(b)
    }

    fn expect(&mut self, b: u8) -> Result<()> {
        let loc = self.loc();
        match self.bump() {
            Some(x) if x == b => Ok(()),
            Some(x) => Err(Error::xml(
                loc,
                format!("expected byte {:?}, got {:?}", b as char, x as char),
            )),
            None => Err(Error::xml(loc, "unexpected end of input")),
        }
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.bump();
        }
    }

    fn parse_document(mut self) -> Result<Document> {
        self.skip_ws();
        // XML declaration / comments / PI
        loop {
            if self.starts_with(b"<?") {
                self.parse_processing_instruction()?;
                self.skip_ws();
                continue;
            }
            if self.starts_with(b"<!--") {
                self.parse_comment()?;
                self.skip_ws();
                continue;
            }
            if self.starts_with(b"<!DOCTYPE") || self.starts_with(b"<!doctype") {
                let loc = self.loc();
                return Err(Error::xml(loc, "DOCTYPE is not supported"));
            }
            break;
        }

        let root = self.parse_element()?;

        self.skip_ws();
        while !self.eof() {
            if self.starts_with(b"<!--") {
                self.parse_comment()?;
                self.skip_ws();
                continue;
            }
            if self.starts_with(b"<?") {
                self.parse_processing_instruction()?;
                self.skip_ws();
                continue;
            }
            let loc = self.loc();
            return Err(Error::xml(loc, "unexpected data after the root element"));
        }

        Ok(Document { root })
    }

    fn parse_processing_instruction(&mut self) -> Result<()> {
        let loc = self.loc();
        self.expect(b'<')?;
        self.expect(b'?')?;
        while !self.eof() && !self.starts_with(b"?>") {
            self.bump();
        }
        if self.starts_with(b"?>") {
            self.bump();
            self.bump();
            Ok(())
        } else {
            Err(Error::xml(loc, "unterminated processing instruction"))
        }
    }

    fn parse_comment(&mut self) -> Result<()> {
        let loc = self.loc();
        if !self.starts_with(b"<!--") {
            return Err(Error::xml(loc, "expected comment"));
        }
        // consume "<!--"
        self.bump();
        self.bump();
        self.bump();
        self.bump();
        while !self.eof() && !self.starts_with(b"-->") {
            self.bump();
        }
        if self.starts_with(b"-->") {
            self.bump();
            self.bump();
            self.bump();
            Ok(())
        } else {
            Err(Error::xml(loc, "unterminated comment"))
        }
    }

    fn parse_cdata(&mut self) -> Result<String> {
        let loc = self.loc();
        if !self.starts_with(b"<![CDATA[") {
            return Err(Error::xml(loc, "expected CDATA"));
        }
        for _ in 0..9 {
            self.bump();
        }
        let start = self.i;
        while !self.eof() && !self.starts_with(b"]]>") {
            self.bump();
        }
        if self.eof() {
            return Err(Error::xml(loc, "unterminated CDATA"));
        }
        let end = self.i;
        self.bump();
        self.bump();
        self.bump();
        Ok(String::from_utf8_lossy(&self.s[start..end]).into_owned())
    }

    fn parse_element(&mut self) -> Result<Element> {
        let loc = self.loc();
        self.expect(b'<')?;
        if matches!(self.peek(), Some(b'/' | b'!' | b'?')) {
            return Err(Error::xml(loc, "expected element start tag"));
        }
        let name = self.parse_name()?;
        let mut attrs = Vec::new();

        loop {
            self.skip_ws();
            if self.starts_with(b"/>") {
                self.bump();
                self.bump();
                return Ok(Element {
                    name,
                    attrs,
                    children: Vec::new(),
                    loc,
                });
            }
            if self.starts_with(b">") {
                self.bump();
                break;
            }
            // attribute
            let attr_name = self.parse_name()?;
            self.skip_ws();
            self.expect(b'=')?;
            self.skip_ws();
            let v = self.parse_quoted_value()?;
            attrs.push(Attribute {
                name: attr_name,
                value: v,
            });
        }

        // children until end tag
        let mut children = Vec::new();
        loop {
            if self.eof() {
                return Err(Error::xml(loc, format!("unterminated element <{}>", name)));
            }
            if self.starts_with(b"</") {
                self.bump();
                self.bump();
                let end_name = self.parse_name()?;
                self.skip_ws();
                self.expect(b'>')?;
                if end_name != name {
                    return Err(Error::xml(
                        self.loc(),
                        format!("expected </{}>, got </{}>", name, end_name),
                    ));
                }
                break;
            }
            if self.starts_with(b"<!--") {
                self.parse_comment()?;
                continue;
            }
            if self.starts_with(b"<?") {
                self.parse_processing_instruction()?;
                continue;
            }
            if self.starts_with(b"<![CDATA[") {
                let t = self.parse_cdata()?;
                push_text(&mut children, t);
                continue;
            }
            if self.starts_with(b"<") {
                let child = self.parse_element()?;
                children.push(Node::Element(child));
                continue;
            }
            // text node
            let t = self.parse_text()?;
            push_text(&mut children, t);
        }

        Ok(Element {
            name,
            attrs,
            children,
            loc,
        })
    }

    fn parse_text(&mut self) -> Result<String> {
        let start = self.i;
        while !self.eof() && !self.starts_with(b"<") {
            self.bump();
        }
        let raw = String::from_utf8_lossy(&self.s[start..self.i]).into_owned();
        decode_entities(&raw).map_err(|m| Error::xml(self.loc(), m))
    }

    fn parse_name(&mut self) -> Result<String> {
        let loc = self.loc();
        let start = self.i;
        while let Some(b) = self.peek() {
            let ok =
                matches!(b, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b':' | b'-' | b'.');
            if !ok {
                break;
            }
            self.bump();
        }
        if self.i == start {
            return Err(Error::xml(loc, "expected name"));
        }
        Ok(String::from_utf8_lossy(&self.s[start..self.i]).into_owned())
    }

    fn parse_quoted_value(&mut self) -> Result<String> {
        let loc = self.loc();
        let quote = match self.bump() {
            Some(b'"') => b'"',
            Some(b'\'') => b'\'',
            Some(_) => return Err(Error::xml(loc, "expected attribute quote")),
            None => return Err(Error::xml(loc, "unexpected end of input")),
        };
        let start = self.i;
        while !self.eof() && self.peek() != Some(quote) {
            self.bump();
        }
        if self.eof() {
            return Err(Error::xml(loc, "unterminated attribute value"));
        }
        let raw = String::from_utf8_lossy(&self.s[start..self.i]).into_owned();
        self.bump(); // closing quote
        decode_entities(&raw).map_err(|m| Error::xml(loc, m))
    }
}

fn push_text(children: &mut Vec<Node>, mut t: String) {
    if t.is_empty() {
        return;
    }
    // We do not strip whitespace aggressively: descriptions/text may rely on formatting.
    // But we drop purely-whitespace nodes that typically appear between elements.
    if t.trim().is_empty() {
        return;
    }
    if let Some(Node::Text(prev)) = children.last_mut() {
        prev.push_str(&t);
        return;
    }
    children.push(Node::Text(t));
}

fn decode_entities(s: &str) -> core::result::Result<String, String> {
    if !s.contains('&') {
        return Ok(s.to_string());
    }
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'&' {
            out.push(bytes[i] as char);
            i += 1;
            continue;
        }
        let semi = bytes[i..]
            .iter()
            .position(|&b| b == b';')
            .ok_or_else(|| "unterminated entity".to_string())?
            + i;
        let ent = &s[i + 1..semi];
        let ch = match ent {
            "lt" => '<',
            "gt" => '>',
            "amp" => '&',
            "apos" => '\'',
            "quot" => '"',
            _ if ent.starts_with("#x") => {
                let v = u32::from_str_radix(&ent[2..], 16)
                    .map_err(|_| format!("invalid entity: &{};", ent))?;
                char::from_u32(v).ok_or_else(|| format!("invalid code point: &{};", ent))?
            }
            _ if ent.starts_with('#') => {
                let v = ent[1..]
                    .parse::<u32>()
                    .map_err(|_| format!("invalid entity: &{};", ent))?;
                char::from_u32(v).ok_or_else(|| format!("invalid code point: &{};", ent))?
            }
            _ => return Err(format!("unsupported entity: &{};", ent)),
        };
        out.push(ch);
        i = semi + 1;
    }
    Ok(out)
}
