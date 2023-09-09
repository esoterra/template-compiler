use std::{str::CharIndices, sync::Arc};

use anyhow::Result;
use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

pub type Tokens<'source> = Vec<(SourceSpan, Token<'source>)>;

#[derive(Debug, PartialEq, Eq)]
pub enum Token<'source> {
    ParamStart,
    ParamEnd,
    CommandStart,
    CommandEnd,
    If,
    EndIf,
    Identifier { name: &'source str },
    Text { index: usize, text: &'source str },
}

#[derive(Error, Debug, Diagnostic)]
#[error("Expected {expected}")]
#[diagnostic()]
pub struct TokenError {
    #[source_code]
    src: Arc<NamedSource>,
    #[label("Here")]
    span: SourceSpan,
    // Expected characters
    expected: &'static str,
}

pub struct Tokenizer<'source> {
    source: Arc<NamedSource>,
    text: &'source str,
    char_indices: CharIndices<'source>,
    tokens: Tokens<'source>,
    text_count: usize
}

impl<'source> Tokenizer<'source> {
    pub fn new(source: Arc<NamedSource>, text: &'source str) -> Self {
        Self {
            source,
            text,
            char_indices: text.char_indices(),
            tokens: Vec::new(),
            text_count: 0
        }
    }

    pub fn tokenize(mut self) -> Result<Tokens<'source>, TokenError> {
        while !self.peek_eof() {
            if self.peek_param_start() {
                self.try_tokenize_param()?;
            } else if self.peek_command_start() {
                self.try_tokenize_command()?;
            } else {
                self.tokenize_text();
            }
        }

        Ok(self.tokens)
    }

    fn peek_eof(&self) -> bool {
        let mut chars = self.char_indices.clone();
        chars.next().is_none()
    }

    fn try_tokenize_command(&mut self) -> Result<(), TokenError> {
        self.tokenize_command_start();
        self.skip_whitespace();
        if !self.peek_eof() {
            match self.try_tokenize_keyword()? {
                Token::If => {
                    self.skip_whitespace();
                    self.try_tokenize_ident()?
                }
                Token::EndIf => {}
                _ => unreachable!(),
            }
        }
        self.skip_whitespace();
        if self.peek_command_end() {
            self.tokenize_command_end();
            Ok(())
        } else {
            let index = self
                .char_indices
                .clone()
                .next()
                .map(|(i, _c)| i)
                .unwrap_or(self.text.len());
            Err(TokenError {
                src: self.source.to_owned(),
                span: SourceSpan::from(index),
                expected: "Command End \"%}\"",
            })
        }
    }

    fn try_tokenize_param(&mut self) -> Result<(), TokenError> {
        self.tokenize_param_start();
        self.skip_whitespace();
        if !self.peek_eof() {
            self.try_tokenize_ident()?;
        }
        self.skip_whitespace();
        if self.peek_param_end() {
            self.tokenize_param_end();
            Ok(())
        } else {
            let index = self
                .char_indices
                .clone()
                .next()
                .map(|(i, _c)| i)
                .unwrap_or(self.text.len());
            Err(TokenError {
                src: self.source.to_owned(),
                span: SourceSpan::from(index),
                expected: "Parameter End \"}}\"",
            })
        }
    }

    fn skip_whitespace(&mut self) {
        let mut chars = self.char_indices.clone();
        while let Some((_, c)) = chars.next() {
            if c.is_whitespace() {
                self.char_indices = chars.clone();
            } else {
                break;
            }
        }
    }

    fn peek_param_start(&self) -> bool {
        self.peek_check("{{")
    }

    fn tokenize_param_start(&mut self) {
        self.consume_as(Token::ParamStart, 2);
    }

    fn peek_param_end(&self) -> bool {
        self.peek_check("}}")
    }

    fn tokenize_param_end(&mut self) {
        self.consume_as(Token::ParamEnd, 2);
    }

    fn peek_command_start(&self) -> bool {
        self.peek_check("{%")
    }

    fn tokenize_command_start(&mut self) {
        self.consume_as(Token::CommandStart, 2);
    }

    fn peek_command_end(&self) -> bool {
        self.peek_check("%}")
    }

    fn tokenize_command_end(&mut self) {
        self.consume_as(Token::CommandEnd, 2);
    }

    fn peek_check(&self, s: &str) -> bool {
        let mut chars = self.char_indices.clone();

        for s_char in s.chars() {
            if let Some((_, next_char)) = chars.next() {
                if next_char != s_char {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }

    /// Requires that there are enough code points left.
    fn consume_as(&mut self, token: Token<'source>, chars: usize) {
        let (start, c) = self.char_indices.next().unwrap();
        let mut len = c.len_utf8();
        for _ in 1..chars {
            let (_, c) = self.char_indices.next().unwrap();
            len += c.len_utf8();
        }
        self.push_token(token, start, len);
    }

    fn try_tokenize_keyword(&mut self) -> Result<Token<'source>, TokenError> {
        if self.peek_check("if") {
            self.consume_as(Token::If, 2);
            Ok(Token::If)
        } else if self.peek_check("endif") {
            self.consume_as(Token::EndIf, 5);
            Ok(Token::EndIf)
        } else {
            let (index, _) = self.char_indices.next().unwrap();
            Err(TokenError {
                src: self.source.to_owned(),
                span: SourceSpan::from((index, 0)),
                expected: "Command must contain keyword",
            })
        }
    }

    fn try_tokenize_ident(&mut self) -> Result<(), TokenError> {
        let mut chars = self.char_indices.clone();
        let (i, c) = chars.next().unwrap();
        let start = i;
        let mut len = c.len_utf8();

        if !(c.is_alphabetic() || c == '_') {
            return Err(TokenError {
                src: self.source.to_owned(),
                span: SourceSpan::from((start, len)),
                expected: "Identifier with pattern /[a-zA-Z_][a-zA-Z0-9_]*/",
            });
        }

        while let Some((_, c)) = chars.next() {
            if c.is_alphanumeric() || c == '_' {
                self.char_indices = chars.clone();
                len += c.len_utf8();
            } else {
                break;
            }
        }

        let name = &self.text[start..start + len];
        let token = Token::Identifier { name };
        self.push_token(token, start, len);
        Ok(())
    }

    fn tokenize_text(&mut self) {
        let (i, c) = self.char_indices.next().unwrap();
        let start = i;
        let mut len = c.len_utf8();

        while !(self.peek_param_start() || self.peek_command_start()) {
            if let Some((_, c)) = self.char_indices.next() {
                len += c.len_utf8();
            } else {
                break;
            }
        }

        let text = &self.text[start..start + len];
        let token = Token::Text { index: self.text_count, text };
        self.text_count += 1;
        self.push_token(token, start, len);
    }

    fn push_token(&mut self, token: Token<'source>, start: usize, len: usize) {
        let span = SourceSpan::from((start, len));
        self.tokens.push((span, token));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use miette::Report;
    use pretty_assertions::assert_eq;

    #[test]
    fn basic_test() {
        let text = "A{{p0}}B{{p1}}C";
        let source = Arc::new(NamedSource::new("basic_test", text));
        let tokenizer = Tokenizer::new(source, text);
        let tokens = match tokenizer.tokenize() {
            Ok(tokens) => tokens,
            Err(error) => {
                println!("{:?}", Report::new(error));
                panic!("test failed");
            }
        };
        assert_eq!(
            tokens,
            vec![
                (SourceSpan::from((0, 1)), Token::Text { index: 0, text: "A" }),
                (SourceSpan::from((1, 2)), Token::ParamStart),
                (SourceSpan::from((3, 2)), Token::Identifier { name: "p0" }),
                (SourceSpan::from((5, 2)), Token::ParamEnd),
                (SourceSpan::from((7, 1)), Token::Text { index: 1, text: "B" }),
                (SourceSpan::from((8, 2)), Token::ParamStart),
                (SourceSpan::from((10, 2)), Token::Identifier { name: "p1" }),
                (SourceSpan::from((12, 2)), Token::ParamEnd),
                (SourceSpan::from((14, 1)), Token::Text { index: 2, text: "C" }),
            ]
        )
    }

    #[test]
    fn command_test() {
        let text = "A {% if foo %}Bar{% endif %} C";
        let source = Arc::new(NamedSource::new("basic_test", text));
        let tokenizer = Tokenizer::new(source, text);
        let tokens = match tokenizer.tokenize() {
            Ok(tokens) => tokens,
            Err(error) => {
                println!("{:?}", Report::new(error));
                panic!("test failed");
            }
        };
        assert_eq!(
            tokens,
            vec![
                (SourceSpan::from((0, 2)), Token::Text { index: 0, text: "A " }),
                (SourceSpan::from((2, 2)), Token::CommandStart),
                (SourceSpan::from((5, 2)), Token::If),
                (SourceSpan::from((8, 3)), Token::Identifier { name: "foo" }),
                (SourceSpan::from((12, 2)), Token::CommandEnd),
                (SourceSpan::from((14, 3)), Token::Text { index: 1, text: "Bar" }),
                (SourceSpan::from((17, 2)), Token::CommandStart),
                (SourceSpan::from((20, 5)), Token::EndIf),
                (SourceSpan::from((26, 2)), Token::CommandEnd),
                (SourceSpan::from((28, 2)), Token::Text { index: 2, text: " C" }),
            ]
        )
    }
}
