use crate::expression::Expression;
use crate::expression_iterator::ExpressionIterator;
use crate::serialization::EasyConfig;
use crate::serialization::serialization_error::SerializationError;

mod simple_test;
mod composite_tuple;
mod vec_of_structs;

#[derive(Debug, PartialEq, Eq)]
struct EzStruct {
    name: String,
    count: i32,
}

impl EasyConfig for EzStruct {
    const IS_STRUCT: bool = true;
    fn serialize(&self) -> Expression {
        Expression::list(vec![
            Expression::presence("EzStruct"),
            Expression::list(vec![
                Expression::binding("name", Expression::presence(self.name.clone())),
                Expression::binding("count", Expression::presence(self.count))
            ])
        ])
    }

    fn deserialize(expression_iterator: &mut ExpressionIterator, source_text: impl AsRef<str>) -> Result<Self, SerializationError> where Self: Sized {
        let source_text = source_text.as_ref();
        let mut normalized = expression_iterator
            .normalized_struct("EzStruct")?
            .into_iter();
        normalized.next();
        let mut fields= normalized.binding_map()?;

        Ok(Self {
            name: fields.get("name", source_text)?,
            count: fields.get("count", source_text)?,
        })
    }
}

fn ez_struct() -> EzStruct {
    EzStruct {
        name: "Momo".to_string(),
        count: 3,
    }
}