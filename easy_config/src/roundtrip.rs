#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
    use std::rc::Rc;
    use std::sync::Arc;
    use crate::interner::Interner;
    use crate::to_value::ToValue;
    use crate::from_value::FromValue;
    use crate::span::Span;
    use crate::value::Value;

    fn round_trip<T: ToValue + FromValue + PartialEq + std::fmt::Debug>(value: T) {
        let mut interner = Interner::new();
        let v = value.to_value(&mut interner, None);
        let source = ""; // no source needed for most types
        let result = T::from_value(v, source).unwrap();
        assert_eq!(value, result);
    }

    // Primitives
    #[test]
    fn test_bool() { round_trip(true); round_trip(false); }

    #[test]
    fn test_u8() { round_trip(0u8); round_trip(255u8); }

    #[test]
    fn test_u16() { round_trip(0u16); round_trip(65535u16); }

    #[test]
    fn test_u32() { round_trip(0u32); round_trip(u32::MAX); }

    #[test]
    fn test_u64() { round_trip(0u64); round_trip(u64::MAX); }

    #[test]
    fn test_i8() { round_trip(0i8); round_trip(-128i8); round_trip(127i8); }

    #[test]
    fn test_i16() { round_trip(0i16); round_trip(i16::MIN); round_trip(i16::MAX); }

    #[test]
    fn test_i32() { round_trip(0i32); round_trip(i32::MIN); round_trip(i32::MAX); }

    #[test]
    fn test_i64() { round_trip(0i64); round_trip(i64::MIN); round_trip(i64::MAX); }

    #[test]
    fn test_f32() { round_trip(0.0f32); round_trip(3.14f32); round_trip(-1.5f32); }

    #[test]
    fn test_f64() { round_trip(0.0f64); round_trip(3.14f64); round_trip(-1.5f64); }

    #[test]
    fn test_char() { round_trip('a'); round_trip('Z'); round_trip('0'); }

    // String
    #[test]
    fn test_string_empty() { round_trip(String::new()); }

    #[test]
    fn test_string_single_word() { round_trip("hello".to_string()); }

    #[test]
    fn test_string_multiple_words() {
        let mut interner = Interner::new();
        let source = "nuh uh";
        let value = Value::list(None, vec![
            Value::presence(interner.intern("nuh"), None).with_span(Span::new(0, 3)),
            Value::presence(interner.intern("uh"), None).with_span(Span::new(4, 6)),
        ], None);
        let result = String::from_value(value, source).unwrap();
        assert_eq!("nuh uh", result);
    }

    // Unit
    #[test]
    fn test_unit() { round_trip(()); }

    // Option
    #[test]
    fn test_option_none() { round_trip(Option::<u32>::None); }

    #[test]
    fn test_option_some() { round_trip(Some(42u32)); }

    #[test]
    fn test_option_nested() { round_trip(Some(Some(42u32))); round_trip(Option::<Option<u32>>::None); }

    // Box, Rc, Arc
    #[test]
    fn test_box() {
        let mut interner = Interner::new();
        let v = 42u32.to_value(&mut interner, None);
        let result = Box::<u32>::from_value(v, "").unwrap();
        assert_eq!(*result, 42u32);
    }

    #[test]
    fn test_rc() {
        let mut interner = Interner::new();
        let v = 42u32.to_value(&mut interner, None);
        let result = Rc::<u32>::from_value(v, "").unwrap();
        assert_eq!(*result, 42u32);
    }

    #[test]
    fn test_arc() {
        let mut interner = Interner::new();
        let v = 42u32.to_value(&mut interner, None);
        let result = Arc::<u32>::from_value(v, "").unwrap();
        assert_eq!(*result, 42u32);
    }

    // Vec
    #[test]
    fn test_vec_empty() { round_trip(Vec::<u32>::new()); }

    #[test]
    fn test_vec() { round_trip(vec![1u32, 2, 3, 4, 5]); }

    #[test]
    fn test_vec_nested() { round_trip(vec![vec![1u32, 2], vec![3u32, 4]]); }

    // Array
    #[test]
    fn test_array() {
        let mut interner = Interner::new();
        let value = [1u32, 2, 3];
        let v = value.to_value(&mut interner, None);
        let result = <[u32; 3]>::from_value(v, "").unwrap();
        assert_eq!(value, result);
    }

    #[test]
    fn test_array_wrong_length() {
        let mut interner = Interner::new();
        let value = [1u32, 2, 3];
        let v = value.to_value(&mut interner, None);
        let result = <[u32; 4]>::from_value(v, "");
        assert!(result.is_err());
    }

    // HashSet
    #[test]
    fn test_hashset() {
        let mut interner = Interner::new();
        let value: HashSet<u32> = [1, 2, 3].into_iter().collect();
        let v = value.to_value(&mut interner, None);
        let result = HashSet::<u32>::from_value(v, "").unwrap();
        assert_eq!(value, result);
    }

    // BTreeSet
    #[test]
    fn test_btreeset() {
        let mut interner = Interner::new();
        let value: BTreeSet<u32> = [1, 2, 3].into_iter().collect();
        let v = value.to_value(&mut interner, None);
        let result = BTreeSet::<u32>::from_value(v, "").unwrap();
        assert_eq!(value, result);
    }

    // HashMap
    #[test]
    fn test_hashmap() {
        let mut interner = Interner::new();
        let mut value: HashMap<String, u32> = HashMap::new();
        value.insert("a".to_string(), 1);
        value.insert("b".to_string(), 2);
        let v = value.to_value(&mut interner, None);
        let result = HashMap::<String, u32>::from_value(v, "").unwrap();
        assert_eq!(value, result);
    }

    // BTreeMap
    #[test]
    fn test_btreemap() {
        let mut interner = Interner::new();
        let mut value: BTreeMap<String, u32> = BTreeMap::new();
        value.insert("a".to_string(), 1);
        value.insert("b".to_string(), 2);
        let v = value.to_value(&mut interner, None);
        let result = BTreeMap::<String, u32>::from_value(v, "").unwrap();
        assert_eq!(value, result);
    }

    // Tuples
    #[test]
    fn test_tuple_1() { round_trip((42u32,)); }

    #[test]
    fn test_tuple_2() { round_trip((42u32, true)); }

    #[test]
    fn test_tuple_3() { round_trip((42u32, true, 3.14f32)); }

    #[test]
    fn test_tuple_4() { round_trip((1u32, 2u32, 3u32, 4u32)); }

    // References (ToValue only, no FromValue)
    #[test]
    fn test_ref() {
        let mut interner = Interner::new();
        let value = 42u32;
        let v = (&value).to_value(&mut interner, None);
        let result = u32::from_value(v, "").unwrap();
        assert_eq!(value, result);
    }

    #[test]
    fn test_box_to_value() {
        let mut interner = Interner::new();
        let value = Box::new(42u32);
        let v = value.to_value(&mut interner, None);
        let result = u32::from_value(v, "").unwrap();
        assert_eq!(*value, result);
    }

    #[test]
    fn test_multi_word_string() {
        let mut interner = Interner::new();
        let source = "(nuh\nuh)";
        let value = Value::list(None, vec![
            Value::presence(interner.intern("nuh"), None).with_span(Span::new(1, 4)),
            Value::presence(interner.intern("uh"), None).with_span(Span::new(5, 7)),
        ], None);
    }
}