pub trait Describe {
    fn describe(&self, source_text: impl AsRef<str>) -> String;
}