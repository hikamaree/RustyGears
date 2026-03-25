use std::cmp::Ordering;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PassId(&'static str);

impl PassId {
    #[inline]
    pub const fn new(name: &'static str) -> Self {
        Self(name)
    }

    #[inline]
    pub fn from_type<T: 'static>() -> Self {
        Self(std::any::type_name::<T>())
    }

    #[inline]
    pub fn as_str(&self) -> &'static str {
        self.0
    }
}

impl Display for PassId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PartialOrd for PassId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PassId {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(other.0)
    }
}

impl PassId {
    pub const SHADOW: Self = Self("shadow");
    pub const OPAQUE: Self = Self("opaque");
    pub const TRANSPARENT: Self = Self("transparent");
    pub const WEIGHTED: Self = Self("weighted");
}
