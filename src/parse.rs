use anyhow::{Result, anyhow};
use ariadne::{Color, Label, Report, ReportKind, sources};
use chumsky::{prelude::*, text::whitespace};

const ALPHABET_LOWER: &'static str = "abcdefghijklmnopqrstuvwxyz";
const ALPHABET_UPPER: &'static str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const ALPHANUM_LOWER: &'static str = "abcdefghijklmnopqrstuvwxyz01234567890";
const ALPHANUM_UPPER: &'static str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ01234567890";

#[derive(Debug, PartialEq, Eq)]
pub enum Node<'src> {
    Text {
        text: &'src str,
    },
    Parameter {
        name: &'src str,
    },
    Conditional {
        cond_ident: &'src str,
        contents: Vec<Node<'src>>,
    },
}

fn parser<'src>() -> impl Parser<'src, &'src str, Vec<Node<'src>>, extra::Err<Rich<'src, char>>> {
    // Create parser for kebab-case identifier
    let alphabet_lower = one_of(ALPHABET_LOWER).labelled("lowercase letter (a-z)");
    let alphabet_upper = one_of(ALPHABET_UPPER).labelled("uppercase letter (A-Z)");
    let alphanum_lower = one_of(ALPHANUM_LOWER).labelled("lowercase letter or digit (a-z, 0-9)");
    let alphanum_upper = one_of(ALPHANUM_UPPER).labelled("uppercase letter or digit (A-Z, 0-9)");

    let first_word = alphabet_lower
        .then(alphanum_lower.repeated())
        .ignored();
    let first_acronym = alphabet_upper
        .then(alphanum_upper.repeated())
        .ignored();
    let first_fragment = choice((first_word, first_acronym));

    let word = alphanum_lower.repeated().at_least(1).ignored();
    let acronym = alphanum_upper.repeated().at_least(1).ignored();
    let fragment = choice((word, acronym));

    let label = first_fragment
        .then(just("-").then(fragment).repeated())
        .to_slice()
        .labelled("label (kebab-case identifier)");

    // Define block delimiters
    let open_expr = just("{{").labelled("start of expression (\"{{\")");
    let close_expr = just("}}").labelled("end of expression (\"}}\")");

    let open_statement = just("{%").labelled("start of statement (\"{%\")");
    let close_statement = just("%}").labelled("end of statement (\"%}\")");

    let open_comment = just("{#").labelled("start of comment (\"{#\")");
    let close_comment = just("#}").labelled("end of comment (\"#}\")");

    let plain_open_brace = just("{").then(none_of("{%#")).ignored().labelled(
        "template text token beginning with \"{\" (must not match \"{{\", \"{%\", or \"{#\")",
    );
    let plain_close_brace = just("}")
        .then(none_of("}"))
        .ignored()
        .labelled("template text token beginning with \"}\" (must not match \"}}\")");
    let plain_percent = just("%")
        .then(none_of("}"))
        .ignored()
        .labelled("template text token beginning with \"%\" (must not match \"%}\")");
    let plain_hash = just("#")
        .then(none_of("}"))
        .ignored()
        .labelled("template text token beginning with \"#\" (must not match \"#}\")");

    // Basic text block
    let text_token = choice((
        none_of("{}%#").ignored(),
        plain_open_brace,
        plain_close_brace,
        plain_percent,
        plain_hash,
    ));

    let text = text_token
        .repeated()
        .at_least(1)
        .to_slice()
        .map(|text| Node::Text { text })
        .labelled("template text")
        .boxed();

    // Expressions
    let param_expression = label
        .clone()
        .map(|name| Node::Parameter { name })
        .labelled("parameter name (starts with a-z or A-Z)");
    let expression = param_expression
        .padded()
        .delimited_by(open_expr, close_expr)
        .labelled("expression")
        .boxed();

    // Statements
    let condition_expression = label.labelled("condition expression");
    let if_start = just("if")
        .ignore_then(whitespace())
        .ignore_then(condition_expression)
        .padded()
        .delimited_by(open_statement, close_statement)
        .labelled("start of 'if' statement")
        .boxed();
    let if_end = just("endif")
        .padded()
        .delimited_by(open_statement, close_statement)
        .labelled("start of 'if' statement")
        .boxed();

    // Comments
    let comment_contents = choice((none_of("#").ignored(), plain_hash))
        .repeated()
        .ignored();
    let comment = open_comment
        .ignore_then(comment_contents)
        .ignore_then(close_comment)
        .ignored()
        .labelled("comment");

    let template_block = recursive(|template| {
        let statement = if_start
            .then(template.padded_by(comment.repeated()).repeated().collect())
            .map(|(cond_ident, contents)| Node::Conditional {
                cond_ident,
                contents,
            })
            .then_ignore(if_end)
            .labelled("if statement")
            .boxed();
        choice((text, expression, statement)).labelled("template block")
    });

    template_block
        .padded_by(comment.repeated())
        .repeated()
        .collect()
        .then_ignore(end())
}

pub fn parse_template<'src>(filename: &str, src: &'src str) -> Result<Vec<Node<'src>>> {
    let parser = parser();
    let (output, errors) = parser.parse(src).into_output_errors();

    for e in errors {
        Report::build(ReportKind::Error, (filename.to_owned(), e.span().into_range()))
            .with_config(ariadne::Config::new().with_index_type(ariadne::IndexType::Byte))
            .with_message(e.to_string())
            .with_label(
                Label::new((filename.to_owned(), e.span().into_range()))
                    .with_message(e.reason().to_string())
                    .with_color(Color::Red),
            )
            .with_labels(e.contexts().map(|(label, span)| {
                Label::new((filename.to_owned(), span.into_range()))
                    .with_message(format!("while parsing this {label}"))
                    .with_color(Color::Yellow)
            }))
            .finish()
            .print(sources([(filename.to_owned(), src)]))
            .unwrap()
    }

    output.ok_or_else(|| anyhow!("Failed to parse template"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comments_are_skipped() {
        // Comments in root
        let input = "{# asdlfa #}foo{# asdlkj #}{##}";
        let template = parse_template("comments_are_skipped:1", input).unwrap();
        assert_eq!(vec![Node::Text { text: "foo" }], template);
        // Comments in if
        let input = "{% if foo %}{# asdlfa #}foo{# asdlkj #}{##}{% endif %}";
        let template = parse_template("comments_are_skipped:2", input).unwrap();
        assert_eq!(
            vec![Node::Conditional {
                cond_ident: "foo",
                contents: vec![Node::Text { text: "foo" }]
            }],
            template
        );
    }
}
