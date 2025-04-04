use std::{
    fmt::Display,
    hash::Hash,
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT_ID: AtomicUsize = AtomicUsize::new(1);
static DUMMY_ID: UniqueId = UniqueId(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UniqueId(usize);

impl UniqueId {
    pub fn new() -> Self {
        UniqueId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for UniqueId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, Eq)]
pub struct IdentifiedSource {
    id: UniqueId,
    name: Option<&'static str>,
}

impl IdentifiedSource {
    pub fn new() -> Self {
        IdentifiedSource {
            id: UniqueId::new(),
            name: None,
        }
    }

    pub fn dummy() -> Self {
        IdentifiedSource {
            id: DUMMY_ID,
            name: None,
        }
    }

    pub fn set_name(&mut self, name: &'static str) {
        self.name = Some(name);
    }

    pub fn id(&self) -> UniqueId {
        self.id
    }

    pub fn name(&self) -> Option<&'static str> {
        self.name
    }
}

impl Default for IdentifiedSource {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for IdentifiedSource {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for IdentifiedSource {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Display for IdentifiedSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(name) = self.name {
            write!(f, "{}", name)
        } else {
            write!(f, "<anônimo {}>", self.id.0)
        }
    }
}
