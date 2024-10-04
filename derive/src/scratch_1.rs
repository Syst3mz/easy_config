enum Mode {
    Unit,
    TupleLike(f32, i32),
    StructLike {a: f32, b: i32},
}
impl easy_config::serialization::Config for Mode {
    fn serialize(&self) -> easy_config::parser::expression::Expression {
        use easy_config::parser::expression::Expression;
        match self {
            Self::Unit => {
                easy_config::parser::expression::Expression::Collection(
                    vec![
                        Expression::Presence("Unit".to_string()),
                    ],
                )
                    .minimized()
            }
            Self::TupleLike(a, b) => {
                easy_config::parser::expression::Expression::Collection(
                    vec![
                        Expression::Presence("TupleLike".to_string()),
                         a.serialize(),
                         b.serialize(),
                    ]
                )
                    .minimized()
            }
            Self::StructLike { a, b } => {
                easy_config::parser::expression::Expression::Collection(vec![
                    Expression::Presence("StructLike".to_string()),
                    Expression::Pair(
                        String::from("a"),
                        Box::new(a.serialize()),
                    ),
                    Expression::Pair(String::from("b"), Box::new(b.serialize())),
                ]
                )
                    .minimized()
            }
        }
    }
    fn deserialize(
        expr: easy_config::parser::expression::Expression,
    ) -> Result<Self, easy_config::serialization::error::Error>
    where
        Self: Sized,
    {
        use easy_config::parser::expression::Expression;
        use easy_config::serialization::DeserializeExtension;
        use easy_config::serialization::error::Error;
        let mut fields = expr
            .clone()
            .into_deserialization_iterator()
            .ok_or(Error::ExpectedTypeGot("Mode".to_string(), expr.pretty()))?;
        let specifier_expr = fields
            .next()
            .ok_or(
                Error::ExpectedTypeGot("Mode".to_string(), "End of input".to_string()),
            )?;
        match &specifier_expr {
            Expression::Presence(p) => {
                return if p == "Unit" {
                    Ok(Mode::Unit)
                } else {
                    Err(
                        Error::ExpectedTypeGot(
                            "Unit".to_string(),
                            specifier_expr.pretty(),
                        ),
                    )
                };
            }
            _ => {}
        }
        let specifier = specifier_expr
            .release()
            .ok_or(Error::ExpectedTypeGot("Mode".to_string(), specifier_expr.pretty()))?;
        if specifier == "TupleLike" {
            return Ok(
                Mode::TupleLike(
                    <f32>::deserialize(
                        fields
                            .next()
                            .ok_or(
                                Error::ExpectedTypeGot(
                                    "f32".to_string(),
                                    "End of input".to_string(),
                                ),
                            )?,
                    )?,
                    <i32>::deserialize(
                        fields
                            .next()
                            .ok_or(
                                Error::ExpectedTypeGot(
                                    "i32".to_string(),
                                    "End of input".to_string(),
                                ),
                            )?,
                    )?,
                ),
            );
        }
        let specifier = specifier_expr
            .release()
            .ok_or(Error::ExpectedTypeGot("Mode".to_string(), specifier_expr.pretty()))?;
        if specifier == "StructLike" {
            return Ok(Mode::StructLike {
                a: <f32>::deserialize(
                    fields
                        .next()
                        .ok_or(
                            Error::ExpectedTypeGot(
                                "f32".to_string(),
                                "End of input".to_string(),
                            ),
                        )?
                        .get("a")
                        .ok_or(Error::UnableToFindKey("a".to_string()))?,
                )?,
                b: <i32>::deserialize(
                    fields
                        .next()
                        .ok_or(
                            Error::ExpectedTypeGot(
                                "i32".to_string(),
                                "End of input".to_string(),
                            ),
                        )?
                        .get("b")
                        .ok_or(Error::UnableToFindKey("b".to_string()))?,
                )?,
            });
        }
        panic!("not implemented")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_me() {}
}