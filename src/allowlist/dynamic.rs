use std::sync::Arc;

use arc_swap::ArcSwap;

use crate::allowlist::{AllowList, HomogeneousAllowLists, PlainAllowList};

pub struct DynamicAllowList<T: AllowList, const N: usize> {
    fixed: HomogeneousAllowLists<T, N>,
    manual: ArcSwap<PlainAllowList>,
}

impl<T: AllowList, const N: usize> AllowList for DynamicAllowList<T, N> {
    fn contains(&self, domain: &str, suffix: &str) -> bool {
        self.fixed.contains(domain, suffix) || self.manual.load().contains(domain, suffix)
    }

    fn mutable(&self) -> bool {
        true
    }
}

impl<T: AllowList, const N: usize> DynamicAllowList<T, N> {
    pub fn allowlist(&self, domain: &str) {
        let mut manual = self.manual.load().as_ref().to_owned();
        manual.add(domain);
        self.manual.store(Arc::new(manual));
    }

    pub fn from_fixed(lists: [T; N]) -> Self {
        DynamicAllowList {
            fixed: HomogeneousAllowLists::new(lists),
            manual: ArcSwap::from_pointee(PlainAllowList::empty()),
        }
    }
}
