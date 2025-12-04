use std::fmt::Debug;

use super::error::Error;

/// Represents a Chip-8 Assemby Token
#[derive(Debug)]
pub enum Token<'src> {
  Label(&'src str),
  Identifier(&'src str),
  Register(u8),
  Number(u16),
  Comma,
  LineBreak,
  Eof,
}

/// Represents a line of Chip-8 Assembly
/// Its easier to evaluate a line at a time
///
/// # Fields
///
/// * `tokens` - The tokens that make up the line
/// * `line` - The line number
pub struct Line<'src> {
  pub tokens: Vec<Token<'src>>,
  pub line: usize,
}

/// Transform the assembly into tokens
pub struct Tokenizer<'src> {
  src: &'src str,
  line: usize,
}

impl<'src> Tokenizer<'src> {
  /// Create a new tokenizer from the source code
  pub fn new(src: &'src str) -> Self {
    Tokenizer { src, line: 1 }
  }

  /// Get the next line of tokens
  ///
  /// # Returns
  ///
  /// * `Ok(Line)` - The next line of tokens
  /// * `Err(Error)` - If there was an error tokenizing any of the tokens
  pub fn get_line(&mut self) -> Result<Line<'src>, Error> {
    let mut tokens: Vec<Token<'src>> = Vec::new();
    let line = self.line;
    loop {
      let token = self.next_token()?;
      if matches!(token, Token::LineBreak) {
        break;
      }

      if let Token::Eof = token {
        if tokens.is_empty() {
          return Ok(Line {
            tokens: vec![Token::Eof],
            line: self.line,
          });
        }
        break;
      }
      tokens.push(token);
    }
    Ok(Line { tokens, line })
  }

  /// Get the next token
  ///
  /// # Returns
  ///
  /// * `Ok(Token)` - The next token
  /// * `Err(Error)` - If there was an error tokenizing the token
  fn next_token(&mut self) -> Result<Token<'src>, Error> {
    loop {
      match self.src.as_bytes() {
        [] => return Ok(Token::Eof),
        [b' ', ..] | [b'\t', ..] | [b'\r', ..] => self.src = &self.src[1..],
        [b'\n', ..] => {
          self.line += 1;
          self.src = &self.src[1..];
          return Ok(Token::LineBreak);
        }
        [b'V', b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F', ..] => {
          self.src = &self.src[1..];
          let space = self.next_space();
          let reg_str = self.consume(space);
          let reg = u8::from_str_radix(reg_str, 16);
          match reg {
            Ok(reg) => {
              if reg > 0xF {
                return Error::InvalidRegister(reg as _, self.line).warp();
              }
              return Ok(Token::Register(reg));
            }
            Err(err) => {
              return Error::InvalidNumber(err, reg_str.to_string(), self.line).warp();
            }
          }
        }
        [b'#', b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F', ..] => {
          self.src = &self.src[1..];
          let space = self.next_space();
          let num_str = self.consume(space);
          let num = u16::from_str_radix(num_str, 16);
          match num {
            Ok(num) => return Ok(Token::Number(num)),
            Err(err) => {
              return Error::InvalidNumber(err, num_str.to_string(), self.line).warp();
            }
          }
        }
        [b'0'..=b'9', ..] => {
          let space = self.next_space();
          let num_str = self.consume(space);
          let num = num_str.parse::<u16>();
          match num {
            Ok(num) => return Ok(Token::Number(num)),
            Err(err) => {
              return Error::InvalidNumber(err, num_str.to_string(), self.line).warp();
            }
          }
        }
        [b',', ..] => {
          let _ = self.consume(1);
          return Ok(Token::Comma);
        }
        [b'a'..=b'z' | b'A'..=b'Z' | b'[' | b']', ..] => {
          let space = self.next_space();
          let id = self.consume(space);
          if id.ends_with(':') {
            return Ok(Token::Label(id.strip_suffix(':').unwrap()));
          }
          return Ok(Token::Identifier(id));
        }
        // Ignore comments ; to end of line
        [b';', ..] => {
          let line_break = self
            .src
            .as_bytes()
            .iter()
            .position(|&c| c == b'\n')
            .unwrap_or(self.src.len());
          self.src = &self.src[line_break..];
        }
        _ => {
          let space = self.next_space();
          return Error::InvalidToken(self.consume(space).to_string(), self.line).warp();
        }
      }
    }
  }

  /// Get the next space in the input
  ///
  /// # Returns
  ///
  /// * `usize` - The index of the next space
  fn next_space(&mut self) -> usize {
    self
      .src
      .as_bytes()
      .iter()
      .position(|&c| c == b' ' || c == b'\t' || c == b'\n' || c == b'\r' || c == b',')
      .unwrap_or(self.src.len())
  }

  /// Consume the next n characters from the input and return them
  ///
  /// # Arguments
  ///
  /// * `n` - The number of characters to consume
  ///
  /// # Returns
  ///
  /// * `&str` - The consumed characters
  fn consume(&mut self, n: usize) -> &'src str {
    // save the lexeme before consuming
    let lexeme = &self.src[..n];
    // consume lexeme from the input
    self.src = &self.src[n..];
    lexeme
  }
}

impl<'src> Iterator for Tokenizer<'src> {
  type Item = Result<Line<'src>, Error>;

  /// Get the next line of tokens
  fn next(&mut self) -> Option<Self::Item> {
    let line: Result<Line<'src>, Error> = self.get_line();
    if let Ok(Line { tokens, line }) = line {
      if let [Token::Eof] = tokens.as_slice() {
        None
      } else {
        Some(Ok(Line { tokens, line }))
      }
    } else {
      Some(line)
    }
  }
}

// Helper types for error handling
/// Is the heap allocated version of Token
#[derive(Debug, Clone)]
pub enum OwnedToken {
  Label(String),
  Identifier(String),
  Register(u8),
  Number(u16),
  Comma,
  LineBreak,
  Eof,
}

impl<'src> From<&Token<'src>> for OwnedToken {
  fn from(token: &Token<'src>) -> Self {
    match token {
      Token::Label(s) => OwnedToken::Label(s.to_string()),
      Token::Identifier(s) => OwnedToken::Identifier(s.to_string()),
      Token::Register(u) => OwnedToken::Register(*u),
      Token::Number(u) => OwnedToken::Number(*u),
      Token::Comma => OwnedToken::Comma,
      Token::Eof => OwnedToken::Eof,
      Token::LineBreak => OwnedToken::LineBreak,
    }
  }
}

/// A line of OwnedTokens
#[derive(Debug, Clone)]
pub struct OwnedLine {
  pub tokens: Vec<OwnedToken>,
  pub line: usize,
}

impl<'src> From<&Line<'src>> for OwnedLine {
  fn from(line: &Line<'src>) -> Self {
    OwnedLine {
      tokens: line.tokens.iter().map(|t| t.into()).collect(),
      line: line.line,
    }
  }
}

impl OwnedToken {
  pub fn to_token(&self) -> Token<'_> {
    match self {
      OwnedToken::Label(s) => Token::Label(s.as_str()),
      OwnedToken::Identifier(s) => Token::Identifier(s.as_str()),
      OwnedToken::Register(u) => Token::Register(*u),
      OwnedToken::Number(u) => Token::Number(*u),
      OwnedToken::Comma => Token::Comma,
      OwnedToken::LineBreak => Token::LineBreak,
      OwnedToken::Eof => Token::Eof,
    }
  }
}

impl OwnedLine {
  pub fn to_line(&self) -> Line<'_> {
    Line {
      tokens: self.tokens.iter().map(|t| t.to_token()).collect(),
      line: self.line,
    }
  }
}
