use std::{str::CharIndices, sync::Arc};

use anyhow::Result;
use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

pub type Tokens<'source> = Vec<(SourceSpan, Token<'source>)>;

#[derive(Debug, PartialEq, Eq)]
pub enum Token<'source> {
    ParamStart,
    ParamEnd,
    Identifier { name: &'source str },
    Text { text: &'source str },
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
}

impl<'source> Tokenizer<'source> {
    pub fn new(source: Arc<NamedSource>, text: &'source str) -> Self {
        Self {
            source,
            text,
            char_indices: text.char_indices(),
            tokens: Vec::new(),
        }
    }

    pub fn tokenize(mut self) -> Result<Tokens<'source>, TokenError> {
        while !self.peek_eof() {
            if self.peek_param_start() {
                self.try_tokenize_param()?;
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
        let mut chars = self.char_indices.clone();
        matches!(chars.next(), Some((_, '{'))) && matches!(chars.next(), Some((_, '{')))
    }

    fn tokenize_param_start(&mut self) {
        let (start, _) = self.char_indices.next().unwrap();
        self.char_indices.next().unwrap();
        self.push_token(Token::ParamStart, start, 2);
    }

    fn peek_param_end(&self) -> bool {
        let mut chars = self.char_indices.clone();
        matches!(chars.next(), Some((_, '}'))) && matches!(chars.next(), Some((_, '}')))
    }

    fn tokenize_param_end(&mut self) {
        let (start, _) = self.char_indices.next().unwrap();
        self.char_indices.next().unwrap();
        self.push_token(Token::ParamEnd, start, 2);
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

        while !self.peek_param_start() {
            if let Some((_, c)) = self.char_indices.next() {
                len += c.len_utf8();
            } else {
                break;
            }
        }

        let text = &self.text[start..start + len];
        let token = Token::Text { text };
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
            },
        };
        assert_eq!(tokens, vec![
            (SourceSpan::from((0, 1)), Token::Text { text: "A" }),
            (SourceSpan::from((1, 2)), Token::ParamStart),
            (SourceSpan::from((3, 2)), Token::Identifier { name: "p0" }),
            (SourceSpan::from((5, 2)), Token::ParamEnd),
            (SourceSpan::from((7, 1)), Token::Text { text: "B" }),
            (SourceSpan::from((8, 2)), Token::ParamStart),
            (SourceSpan::from((10, 2)), Token::Identifier { name: "p1" }),
            (SourceSpan::from((12, 2)), Token::ParamEnd),
            (SourceSpan::from((14, 1)), Token::Text { text: "C" }),
        ])
    }
}