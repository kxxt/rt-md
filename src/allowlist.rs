mod dynamic;
mod plain;

pub use dynamic::DynamicAllowList;
pub use plain::{AllowMode, PlainAllowList};

pub trait AllowList {
    fn contains(&self, domain: &str, suffix: &str) -> bool;

    fn mutable(&self) -> bool {
        false
    }
}

pub struct EmptyAllowList;

pub struct HeterogeneousAllowLists {
    lists: Vec<Box<dyn AllowList>>,
}

pub struct HomogeneousAllowLists<T, const N: usize> {
    lists: [T; N],
}

impl HeterogeneousAllowLists {
    #[allow(unused)]
    pub fn two<T: AllowList + 'static, U: AllowList + 'static>(first: T, second: U) -> Self {
        let lists = vec![
            Box::new(first) as Box<dyn AllowList>,
            Box::new(second) as Box<dyn AllowList>,
        ];
        Self { lists }
    }

    #[allow(unused)]
    pub fn three<T: AllowList + 'static, U: AllowList + 'static, K: AllowList + 'static>(
        first: T,
        second: U,
        third: K,
    ) -> Self {
        let lists = vec![
            Box::new(first) as Box<dyn AllowList>,
            Box::new(second) as Box<dyn AllowList>,
            Box::new(third) as Box<dyn AllowList>,
        ];
        Self { lists }
    }

    #[allow(unused)]
    pub fn empty() -> Self {
        Self { lists: Vec::new() }
    }

    #[allow(unused)]
    pub fn push(&mut self, allowlist: impl AllowList + 'static) {
        self.lists.push(Box::new(allowlist) as Box<dyn AllowList>);
    }
}

impl AllowList for HeterogeneousAllowLists {
    fn contains(&self, value: &str, suffix: &str) -> bool {
        for l in self.lists.iter() {
            if l.contains(&value.to_lowercase(), &suffix.to_lowercase()) {
                return true;
            }
        }
        false
    }
}

impl<T: AllowList, const N: usize> HomogeneousAllowLists<T, N> {
    pub fn new(lists: [T; N]) -> Self {
        Self { lists }
    }
}

impl<T: AllowList, const N: usize> AllowList for HomogeneousAllowLists<T, N> {
    fn contains(&self, domain: &str, suffix: &str) -> bool {
        for list in &self.lists {
            if list.contains(domain, suffix) {
                return true;
            }
        }
        false
    }
}

impl AllowList for EmptyAllowList {
    fn contains(&self, _domain: &str, _suffix: &str) -> bool {
        false
    }
}

impl<T: AllowList + ?Sized> AllowList for Box<T> {
    fn contains(&self, domain: &str, suffix: &str) -> bool {
        self.as_ref().contains(domain, suffix)
    }
}
