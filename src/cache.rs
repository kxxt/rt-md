//! Time To Idle Eviction Cache
//!
//! The cache provided in this module evicts items based on Time-To-Idle policy.
//!
//! We provide two implementations for evaluation on offline datasets and online evaluation.

use std::{cmp::Reverse, net::IpAddr};

use hashbrown::{DefaultHashBuilder, HashMap};
use priority_queue::PriorityQueue;

pub trait TimeToIdleCache<V> {
    fn new(tti: u64) -> Self;

    fn insert(&mut self, key: IpAddr, value: V, now: u64);
    fn get_mut(&mut self, key: IpAddr, now: u64) -> Option<&mut V>;
}

pub struct SimulatedTimeToIdleCache<V> {
    /// Priority queue ordered by `last access time + IDLE timeout`, smallest first
    /// When accessing this element, we update its priority.
    /// We check this queue when inserting or get an element.
    timeout_queue: PriorityQueue<IpAddr, Reverse<u64>>,
    /// The real hash map storing last access time
    inner: HashMap<IpAddr, V, DefaultHashBuilder>,
    /// tti
    tti: u64,
}

impl<V> SimulatedTimeToIdleCache<V> {
    /// Try to evict
    fn try_evict(&mut self, now: u64) {
        while let Some((&k, &t)) = self.timeout_queue.peek()
            && t.0 < now
        {
            self.timeout_queue.pop();
            self.inner.remove(&k);
        }
    }
}

impl<V> TimeToIdleCache<V> for SimulatedTimeToIdleCache<V> {
    fn new(tti: u64) -> Self {
        Self {
            timeout_queue: PriorityQueue::new(),
            inner: HashMap::new(),
            tti,
        }
    }

    fn insert(&mut self, key: IpAddr, value: V, now: u64) {
        self.try_evict(now);
        self.inner.insert(key, value);
        self.timeout_queue.push(key, Reverse(now + self.tti));
    }

    fn get_mut(&mut self, key: IpAddr, now: u64) -> Option<&mut V> {
        self.try_evict(now);
        let v = self.inner.get_mut(&key);
        if v.is_some() {
            self.timeout_queue
                .push_decrease(key, Reverse(now + self.tti));
        }
        v
    }
}

// pub struct OnlineTimeToIdleCache<V: Clone + Send + Sync + 'static> {
//     inner: moka::sync::Cache<IpAddr, Arc<RwLock<V>>, DefaultHashBuilder>,
// }

// impl<V: Clone + Send + Sync + 'static> TimeToIdleCache<V> for OnlineTimeToIdleCache<V> {
//     fn new(tti: u64) -> Self {
//         Self {
//             inner: moka::sync::CacheBuilder::new(1_000_000_000)
//                 .time_to_idle(Duration::from_millis(tti))
//                 .build_with_hasher(DefaultHashBuilder::default()),
//         }
//     }

//     fn insert(&mut self, key: IpAddr, value: V, now: u64) {
//         self.inner.insert(key, Arc::new(RwLock::new(value)));
//     }

//     fn get_mut(&mut self, key: IpAddr, now: u64) -> Option<&mut V> {
//         self.inner.get(key)
//     }
// }
