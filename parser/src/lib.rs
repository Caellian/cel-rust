use lalrpop_util::lalrpop_mod;

pub mod ast;
pub use ast::*;

pub mod parse;
pub use parse::*;

pub mod error;
pub use error::ParseError;

lalrpop_mod!(#[allow(clippy::all)] pub parser, "/cel.rs");

/// Parses a CEL expression and returns it.
///
/// # Example
/// ```
/// use cel_parser::parse;
/// let expr = parse("1 + 1").unwrap();
/// println!("{:?}", expr);
/// ```
pub fn parse(input: &str) -> Result<Expression, ParseError> {
    // Wrap the internal parser function - whether larlpop or chumsky

    // Example for a possible new chumsky based parser...
    // parser().parse(input)
    //     .into_result()
    //     .map_err(|e|  {
    //         ParseError {
    //             msg: e.iter().map(|e| format!("{}", e)).collect::<Vec<_>>().join("\n")
    //         }
    //     })

    // Existing Larlpop Parser:
    crate::parser::ExpressionParser::new()
        .parse(input)
        .map_err(|e| ParseError::from_lalrpop(input, e))
}

#[cfg(test)]
mod tests {
    use crate::{
        ArithmeticOp, Atom, Atom::*, Expression, Expression::*, Member::*, RelationOp, UnaryOp,
    };

    fn parse(input: &str) -> Expression {
        crate::parse(input).unwrap_or_else(|e| panic!("{}", e))
    }

    fn assert_parse_eq(input: &str, expected: Expression) {
        assert_eq!(parse(input), expected);
    }

    #[test]
    fn ident() {
        assert_parse_eq("a", Ident("a".to_string().into()));
        assert_parse_eq("hello ", Ident("hello".to_string().into()));
    }

    #[test]
    fn simple_int() {
        assert_parse_eq("1", Atom(Int(1)))
    }

    #[test]
    fn simple_float() {
        assert_parse_eq("1.0", Atom(Float(1.0)))
    }

    #[test]
    fn other_floats() {
        assert_parse_eq("1e3", Expression::Atom(Atom::Float(1000.0)));
        assert_parse_eq("1e-3", Expression::Atom(Atom::Float(0.001)));
        assert_parse_eq("1.4e-3", Expression::Atom(Atom::Float(0.0014)));
    }

    #[test]
    fn single_quote_str() {
        assert_parse_eq("'foobar'", Atom(String("foobar".to_string().into())))
    }

    #[test]
    fn double_quote_str() {
        assert_parse_eq(r#""foobar""#, Atom(String("foobar".to_string().into())))
    }

    // #[test]
    // fn single_quote_raw_str() {
    //     assert_parse_eq(
    //         "r'\n'",
    //         Expression::Atom(String("\n".to_string().into())),
    //     );
    // }

    #[test]
    fn single_quote_bytes() {
        assert_parse_eq("b'foo'", Atom(Bytes(b"foo".to_vec().into())));
        assert_parse_eq("b''", Atom(Bytes(b"".to_vec().into())));
    }

    #[test]
    fn double_quote_bytes() {
        assert_parse_eq(r#"b"foo""#, Atom(Bytes(b"foo".to_vec().into())));
        assert_parse_eq(r#"b"""#, Atom(Bytes(b"".to_vec().into())));
    }

    #[test]
    fn bools() {
        assert_parse_eq("true", Atom(Bool(true)));
        assert_parse_eq("false", Atom(Bool(false)));
    }

    #[test]
    fn nulls() {
        assert_parse_eq("null", Atom(Null));
    }

    #[test]
    fn structure() {
        println!("{:+?}", parse("{1 + a: 3}"));
    }

    #[test]
    fn simple_str() {
        assert_parse_eq(r#"'foobar'"#, Atom(String("foobar".to_string().into())));
        println!("{:?}", parse(r#"1 == '1'"#))
    }

    #[test]
    fn test_parse_map_macro() {
        assert_parse_eq(
            "[1, 2, 3].map(x, x * 2)",
            FunctionCall(
                Box::new(Ident("map".to_string().into())),
                Some(Box::new(List(vec![
                    Atom(Int(1)),
                    Atom(Int(2)),
                    Atom(Int(3)),
                ]))),
                vec![
                    Ident("x".to_string().into()),
                    Arithmetic(
                        Box::new(Ident("x".to_string().into())),
                        ArithmeticOp::Multiply,
                        Box::new(Atom(Int(2))),
                    ),
                ],
            ),
        )
    }

    #[test]
    fn nested_attributes() {
        assert_parse_eq(
            "a.b[1]",
            Member(
                Member(
                    Ident("a".to_string().into()).into(),
                    Attribute("b".to_string().into()).into(),
                )
                .into(),
                Index(Atom(Int(1)).into()).into(),
            ),
        )
    }

    #[test]
    fn function_call_no_args() {
        assert_parse_eq(
            "a()",
            FunctionCall(Box::new(Ident("a".to_string().into())), None, vec![]),
        );
    }

    #[test]
    fn test_parser_bool_unary_ops() {
        assert_parse_eq(
            "!false",
            Unary(UnaryOp::Not, Box::new(Expression::Atom(Atom::Bool(false)))),
        );
        assert_parse_eq(
            "!true",
            Unary(UnaryOp::Not, Box::new(Expression::Atom(Atom::Bool(true)))),
        );
    }

    #[test]
    fn test_parser_binary_bool_expressions() {
        assert_parse_eq(
            "true == true",
            Relation(
                Box::new(Expression::Atom(Atom::Bool(true))),
                RelationOp::Equals,
                Box::new(Expression::Atom(Atom::Bool(true))),
            ),
        );
    }

    #[test]
    fn test_parser_bool_unary_ops_repeated() {
        assert_eq!(
            parse("!!true"),
            (Unary(
                UnaryOp::DoubleNot,
                Box::new(Expression::Atom(Atom::Bool(true))),
            ))
        );
    }

    #[test]
    fn delimited_expressions() {
        assert_parse_eq(
            "(-((1)))",
            Unary(UnaryOp::Minus, Box::new(Expression::Atom(Atom::Int(1)))),
        );
    }

    #[test]
    fn test_empty_list_parsing() {
        assert_eq!(parse("[]"), (List(vec![])));
    }

    #[test]
    fn test_int_list_parsing() {
        assert_parse_eq(
            "[1,2,3]",
            List(vec![
                Expression::Atom(Atom::Int(1)),
                Expression::Atom(Atom::Int(2)),
                Expression::Atom(Atom::Int(3)),
            ]),
        );
    }

    #[test]
    fn list_index_parsing() {
        assert_parse_eq(
            "[1,2,3][0]",
            Member(
                Box::new(List(vec![
                    Expression::Atom(Int(1)),
                    Expression::Atom(Int(2)),
                    Expression::Atom(Int(3)),
                ])),
                Box::new(Index(Box::new(Expression::Atom(Int(0))))),
            ),
        );
    }

    #[test]
    fn mixed_type_list() {
        assert_parse_eq(
            "['0', 1, 3.0, null]",
            //"['0', 1, 2u, 3.0, null]",
            List(vec![
                Expression::Atom(String("0".to_string().into())),
                Expression::Atom(Int(1)),
                //Expression::Atom(UInt(2)),
                Expression::Atom(Float(3.0)),
                Expression::Atom(Null),
            ]),
        );
    }

    #[test]
    fn test_nested_list_parsing() {
        assert_parse_eq(
            "[[], [], [[1]]]",
            List(vec![
                List(vec![]),
                List(vec![]),
                List(vec![List(vec![Expression::Atom(Int(1))])]),
            ]),
        );
    }

    #[test]
    fn test_in_list_relation() {
        assert_parse_eq(
            "2 in [2]",
            Relation(
                Box::new(Expression::Atom(Int(2))),
                RelationOp::In,
                Box::new(List(vec![Expression::Atom(Int(2))])),
            ),
        );
    }

    #[test]
    fn test_empty_map_parsing() {
        assert_eq!(parse("{}"), (Map(vec![])));
    }

    #[test]
    fn test_nonempty_map_parsing() {
        assert_parse_eq(
            "{'a': 1, 'b': 2}",
            Map(vec![
                (
                    Expression::Atom(String("a".to_string().into())),
                    Expression::Atom(Int(1)),
                ),
                (
                    Expression::Atom(String("b".to_string().into())),
                    Expression::Atom(Int(2)),
                ),
            ]),
        );
    }

    #[test]
    fn nonempty_map_index_parsing() {
        assert_parse_eq(
            "{'a': 1, 'b': 2}[0]",
            Member(
                Box::new(Map(vec![
                    (
                        Expression::Atom(String("a".to_string().into())),
                        Expression::Atom(Int(1)),
                    ),
                    (
                        Expression::Atom(String("b".to_string().into())),
                        Expression::Atom(Int(2)),
                    ),
                ])),
                Box::new(Index(Box::new(Expression::Atom(Int(0))))),
            ),
        );
    }

    #[test]
    fn integer_relations() {
        assert_parse_eq(
            "2 != 3",
            Relation(
                Box::new(Expression::Atom(Int(2))),
                RelationOp::NotEquals,
                Box::new(Expression::Atom(Int(3))),
            ),
        );
        assert_parse_eq(
            "2 == 3",
            Relation(
                Box::new(Expression::Atom(Int(2))),
                RelationOp::Equals,
                Box::new(Expression::Atom(Int(3))),
            ),
        );

        assert_parse_eq(
            "2 < 3",
            Relation(
                Box::new(Expression::Atom(Int(2))),
                RelationOp::LessThan,
                Box::new(Expression::Atom(Int(3))),
            ),
        );

        assert_parse_eq(
            "2 <= 3",
            Relation(
                Box::new(Expression::Atom(Int(2))),
                RelationOp::LessThanEq,
                Box::new(Expression::Atom(Int(3))),
            ),
        );
    }

    #[test]
    fn binary_product_expressions() {
        assert_parse_eq(
            "2 * 3",
            Arithmetic(
                Box::new(Expression::Atom(Atom::Int(2))),
                ArithmeticOp::Multiply,
                Box::new(Expression::Atom(Atom::Int(3))),
            ),
        );
    }

    // #[test]
    // fn binary_product_negated_expressions() {
    //     assert_parse_eq(
    //         "2 * -3",
    //         Arithmetic(
    //             Box::new(Expression::Atom(Atom::Int(2))),
    //             ArithmeticOp::Multiply,
    //             Box::new(Unary(
    //                 UnaryOp::Minus,
    //                 Box::new(Expression::Atom(Atom::Int(3))),
    //             )),
    //         ),
    //     );
    //
    //     assert_parse_eq(
    //         "2 / -3",
    //         Arithmetic(
    //             Box::new(Expression::Atom(Int(2))),
    //             ArithmeticOp::Divide,
    //             Box::new(Unary(
    //                 UnaryOp::Minus,
    //                 Box::new(Expression::Atom(Int(3))),
    //             )),
    //         ),
    //     );
    // }

    #[test]
    fn test_parser_sum_expressions() {
        assert_parse_eq(
            "2 + 3",
            Arithmetic(
                Box::new(Expression::Atom(Atom::Int(2))),
                ArithmeticOp::Add,
                Box::new(Expression::Atom(Atom::Int(3))),
            ),
        );

        // assert_parse_eq(
        //     "2 - -3",
        //     Arithmetic(
        //         Box::new(Expression::Atom(Atom::Int(2))),
        //         ArithmeticOp::Subtract,
        //         Box::new(Unary(
        //             UnaryOp::Minus,
        //             Box::new(Expression::Atom(Atom::Int(3))),
        //         )),
        //     ),
        // );
    }

    #[test]
    fn conditionals() {
        assert_parse_eq(
            "true && true",
            And(
                Box::new(Expression::Atom(Bool(true))),
                Box::new(Expression::Atom(Bool(true))),
            ),
        );
        assert_parse_eq(
            "false || true",
            Or(
                Box::new(Expression::Atom(Bool(false))),
                Box::new(Expression::Atom(Bool(true))),
            ),
        );
    }
    #[test]
    fn test_ternary_true_condition() {
        assert_parse_eq(
            "true ? 'result_true' : 'result_false'",
            Ternary(
                Box::new(Expression::Atom(Bool(true))),
                Box::new(Expression::Atom(String("result_true".to_string().into()))),
                Box::new(Expression::Atom(String("result_false".to_string().into()))),
            ),
        );

        assert_parse_eq(
            "true ? 100 : 200",
            Ternary(
                Box::new(Expression::Atom(Bool(true))),
                Box::new(Expression::Atom(Int(100))),
                Box::new(Expression::Atom(Int(200))),
            ),
        );
    }

    #[test]
    fn test_ternary_false_condition() {
        assert_parse_eq(
            "false ? 'result_true' : 'result_false'",
            Ternary(
                Box::new(Expression::Atom(Bool(false))),
                Box::new(Expression::Atom(String("result_true".to_string().into()))),
                Box::new(Expression::Atom(String("result_false".to_string().into()))),
            ),
        );
    }

    #[test]
    fn test_operator_precedence() {
        assert_parse_eq(
            "a && b == 'string'",
            And(
                Box::new(Ident("a".to_string().into())),
                Box::new(Relation(
                    Box::new(Ident("b".to_string().into())),
                    RelationOp::Equals,
                    Box::new(Expression::Atom(String("string".to_string().into()))),
                )),
            ),
        );
    }

    #[test]
    fn test_foobar() {
        println!("{:?}", parse("foo.bar.baz == 10 && size(requests) == 3"))
    }

    #[test]
    fn test_unrecognized_token_error() {
        let source = r#"
            account.balance == transaction.withdrawal
                || (account.overdraftProtection
                    account.overdraftLimit >= transaction.withdrawal  - account.balance)
        "#;

        let err = crate::parse(source).unwrap_err();

        assert_eq!(err.msg, "unrecognized token: 'account'");

        assert_eq!(err.span.start.as_ref().unwrap().line, 3);
        assert_eq!(err.span.start.as_ref().unwrap().column, 20);
        assert_eq!(err.span.end.as_ref().unwrap().line, 3);
        assert_eq!(err.span.end.as_ref().unwrap().column, 27);
    }

    #[test]
    fn test_unrecognized_eof_error() {
        let source = r#" "#;

        let err = crate::parse(source).unwrap_err();

        assert_eq!(err.msg, "unrecognized eof");

        assert_eq!(err.span.start.as_ref().unwrap().line, 0);
        assert_eq!(err.span.start.as_ref().unwrap().column, 0);
        assert_eq!(err.span.end.as_ref().unwrap().line, 0);
        assert_eq!(err.span.end.as_ref().unwrap().column, 0);
    }

    #[test]
    fn test_invalid_token_error() {
        let source = r#"
            account.balance == §
        "#;

        let err = crate::parse(source).unwrap_err();

        assert_eq!(err.msg, "invalid token");

        assert_eq!(err.span.start.as_ref().unwrap().line, 1);
        assert_eq!(err.span.start.as_ref().unwrap().column, 31);
        assert_eq!(err.span.end.as_ref().unwrap().line, 1);
        assert_eq!(err.span.end.as_ref().unwrap().column, 31);
    }
}
