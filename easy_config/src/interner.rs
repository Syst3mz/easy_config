use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct InternedString(Arc<str>);
impl InternedString {
    pub fn resolve(&self) -> &str {
        &self.0
    }
}
impl PartialEq for InternedString {
    fn eq(&self, other: &InternedString) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for InternedString {}
impl Hash for InternedString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0).hash(state);
    }
}
pub struct Interner {
    interned: HashSet<InternedString>,
}

impl Interner {
    pub fn new() -> Interner {
        Self {
            interned: HashSet::new(),
        }
    }

    pub fn intern(&mut self, string: impl AsRef<str>) -> InternedString {
        let arc: Arc<str> = Arc::from(string.as_ref());
        let interned = InternedString(arc);

        if let Some(interned) = self.interned.get(&interned) {
            return interned.clone();
        }

        self.interned.insert(interned.clone());
        interned
    }
}

pub(crate) trait OptionExtension {
    fn intern(self, interner: &mut Interner) -> Option<InternedString>;
}

impl<T: AsRef<str>> OptionExtension for Option<T> {
    fn intern(self, interner: &mut Interner) -> Option<InternedString> {
        self.map(|x| interner.intern(x.as_ref()))
    }
}