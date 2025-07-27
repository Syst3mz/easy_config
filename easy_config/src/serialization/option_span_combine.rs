use crate::lexical_span::LexicalSpan;

pub trait OptionSpanCombine {
    fn combine(&mut self, other: LexicalSpan);
}

impl OptionSpanCombine for Option<LexicalSpan> {
    fn combine(&mut self, other: LexicalSpan) {
        if let Some(acc) = self {
            *acc = acc.combine(other)
        } else {
            *self = Some(other);
            return;
        }
    }
}