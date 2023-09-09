use std::{sync::Arc, iter::Peekable};

use anyhow::Result;
use miette::{NamedSource, SourceSpan};

use crate::tokens::{Token, Tokenizer};

#[derive(Debug)]
pub struct M<T> {
    pub span: SourceSpan,
    pub value: T,
}

impl<T> M<T> {
    pub fn new(value: T, span: SourceSpan) -> Self {
        M { span, value }
    }
}

#[derive(Debug)]
pub struct FileData<'source> {
    pub source: Arc<NamedSource>,
    pub contents: Vec<Node<'source>>,
}

#[derive(Debug)]
pub enum Node<'source> {
    Text {
        index: usize,
        text: M<&'source str>,
    },
    Parameter {
        name: M<&'source str>,
    },
    Conditional {
        if_kwd: SourceSpan,
        cond_ident: M<&'source str>,
        contents: Vec<Node<'source>>,
        endif_kwd: SourceSpan,
    },
}

pub fn parse_file<'source>(
    source: Arc<NamedSource>,
    text: &'source str,
) -> Result<FileData<'source>> {
    let tokens = Tokenizer::new(source.clone(), text).tokenize()?;
    let mut token_iter = tokens.into_iter().peekable();

    let contents = parse_tokens(source.clone(), &mut token_iter)?;

    Ok(FileData { source, contents })
}

fn parse_tokens<'source, Iter>(source: Arc<NamedSource>, token_iter: &mut Peekable<Iter>) -> Result<Vec<Node<'source>>>
where
    Iter: Iterator<Item = (SourceSpan, Token<'source>)>,
{
    let mut contents = Vec::new();

    while let Some((span, token)) = token_iter.next() {
        match token {
            Token::CommandStart => {
                if token_iter.peek().unwrap().1 == Token::EndIf {
                    return Ok(contents);
                }

                let if_kwd = match_token(token_iter, Token::If)?;
                let cond_ident = match_ident(token_iter)?;
                match_token(token_iter, Token::CommandEnd)?;

                let if_contents = parse_tokens(source.clone(), token_iter)?;

                let endif_kwd = match_token(token_iter, Token::EndIf)?;
                match_token(token_iter, Token::CommandEnd)?;

                contents.push(Node::Conditional {
                    if_kwd,
                    cond_ident,
                    contents: if_contents,
                    endif_kwd,
                })
            }
            Token::ParamStart => {
                let name = match_ident(token_iter)?;
                contents.push(Node::Parameter { name });
                match_token(token_iter, Token::ParamEnd)?;
            }
            Token::Text { index, text } => {
                contents.push(Node::Text {
                    index,
                    text: M::new(text, span),
                });
            }
            _ => todo!(),
        }
    }

    Ok(contents)
}

fn match_token<'source, Iter>(token_iter: &mut Iter, token: Token<'source>) -> Result<SourceSpan>
where
    Iter: Iterator<Item = (SourceSpan, Token<'source>)>,
{
    match token_iter.next() {
        Some((span, t)) if t == token => Ok(span),
        Some(st) => { dbg!(st); todo!() },
        None => todo!("error"),
    }
}

fn match_ident<'source, Iter>(token_iter: &mut Iter) -> Result<M<&'source str>>
where
    Iter: Iterator<Item = (SourceSpan, Token<'source>)>,
{
    match token_iter.next() {
        Some((span, Token::Identifier { name })) => Ok(M::new(name, span)),
        Some(_) => todo!("error"),
        None => todo!("error"),
    }
}
