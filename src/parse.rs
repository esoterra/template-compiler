use anyhow::{Result, anyhow};
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
    let first_word = one_of(ALPHABET_LOWER)
        .then(one_of(ALPHANUM_LOWER).repeated())
        .ignored();
    let first_acronym = one_of(ALPHABET_UPPER)
        .then(one_of(ALPHANUM_UPPER).repeated())
        .ignored();
    let first_fragment = choice((first_word, first_acronym));

    let word = one_of(ALPHANUM_LOWER).repeated().at_least(1).ignored();
    let acronym = one_of(ALPHANUM_UPPER).repeated().at_least(1).ignored();
    let fragment = choice((word, acronym));

    let label = first_fragment
        .then(just("-").then(fragment).repeated())
        .to_slice()
        .labelled("Label (identifier)");

    // Define block delimiters
    let open_expr = just("{{").labelled("Start of expression");
    let close_expr = just("}}").labelled("End of expression");

    let open_statement = just("{%").labelled("Start of statement");
    let close_statement = just("%}").labelled("End of statement");

    let open_comment = just("{#").labelled("Start of comment");
    let close_comment = just("#}").labelled("End of comment");

    let plain_open_brace = just("{").then(none_of("{%#")).ignored().labelled(
        "Template text token beginning with \"{\" (must not match \"{{\", \"{%\", or \"{#\")",
    );
    let plain_close_brace = just("}")
        .then(none_of("}"))
        .ignored()
        .labelled("Template text token beginning with \"}\" (must not match \"}}\")");
    let plain_percent = just("%")
        .then(none_of("}"))
        .ignored()
        .labelled("Template text token beginning with \"%\" (must not match \"%}\")");
    let plain_hash = just("#")
        .then(none_of("}"))
        .ignored()
        .labelled("Template text token beginning with \"#\" (must not match \"#}\")");

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
        .labelled("Template text")
        .boxed();

    // Expressions
    let param_expression = label
        .clone()
        .map(|name| Node::Parameter { name })
        .labelled("Parameter expression");
    let expression = param_expression
        .padded()
        .delimited_by(open_expr, close_expr)
        .labelled("Expression")
        .boxed();

    // Statements
    let condition_expression = label.labelled("Condition expression");
    let if_start = just("if")
        .ignore_then(whitespace())
        .ignore_then(condition_expression)
        .padded()
        .delimited_by(open_statement, close_statement)
        .labelled("Start of 'if' statement")
        .boxed();
    let if_end = just("endif")
        .padded()
        .delimited_by(open_statement, close_statement)
        .labelled("Start of 'if' statement")
        .boxed();

    // Comments
    let comment_contents = choice((none_of("#").ignored(), plain_hash))
        .repeated()
        .ignored();
    let comment = open_comment
        .ignore_then(comment_contents)
        .ignore_then(close_comment)
        .ignored()
        .labelled("Comment");

    let template_block = recursive(|template| {
        let statement = if_start
            .then(template.padded_by(comment.repeated()).repeated().collect())
            .map(|(cond_ident, contents)| Node::Conditional {
                cond_ident,
                contents,
            })
            .then_ignore(if_end)
            .labelled("If statement")
            .boxed();
        choice((text, expression, statement)).labelled("Template block")
    });

    template_block
        .padded_by(comment.repeated())
        .repeated()
        .collect()
        .then_ignore(end())
}

pub fn parse_template<'src>(name: &str, input: &'src str) -> Result<Vec<Node<'src>>> {
    let parser = parser();
    let (output, errors) = parser.parse(input).into_output_errors();

    for error in errors {
        println!("{name}: {error:?}");
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
