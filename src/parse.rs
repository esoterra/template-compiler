use std::sync::Arc;

use anyhow::Result;
use miette::{NamedSource, SourceSpan};

use crate::tokens::{Tokenizer, Token};

#[derive(Debug)]
pub struct FileData<'source> {
    pub source: Arc<NamedSource>,
    pub contents: Vec<(SourceSpan, Node<'source>)>,
}

#[derive(Debug)]
pub enum Node<'source> {
    Text { text: &'source str },
    Parameter { name: &'source str },
}

pub fn parse_file<'source>(source: Arc<NamedSource>, text: &'source str) -> Result<FileData<'source>> {
    let tokens = Tokenizer::new(source.clone(), text).tokenize()?;
    let mut token_iter = tokens.into_iter();

    let mut contents = Vec::new();

    while let Some((span, token)) = token_iter.next() {
        match token {
            Token::ParamStart => {
                match token_iter.next() {
                    Some((span, Token::Identifier { name })) => {
                        contents.push((span, Node::Parameter { name }));
                        match token_iter.next() {
                            Some((_, Token::ParamEnd)) => {},
                            Some((_i, _token)) => todo!(),
                            None => todo!(),
                        }
                    },
                    Some((_i, _token)) => todo!(),
                    None => todo!(),
                }
            },
            Token::Text { text } => {
                contents.push((span, Node::Text { text }));
            },
            _ => todo!()
        }
    }

    Ok(FileData { source, contents })
}
