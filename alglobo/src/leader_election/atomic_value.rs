use std::{
    sync::{Arc, Condvar, Mutex, MutexGuard},
    time::Duration,
};

#[derive(Clone)]
pub struct AtomicValue<V> {
    value: Arc<Mutex<V>>,
    condition: Arc<Condvar>,
}

impl<V> AtomicValue<V> {
    pub fn new(initial_value: V) -> Self {
        Self {
            value: Arc::new(Mutex::new(initial_value)),
            condition: Arc::new(Condvar::new()),
        }
    }

    pub fn load(&self) -> MutexGuard<V> {
        self.value.lock().expect("Mutex poisoned")
    }

    pub fn store(&self, new_value: V) {
        let mut value = self.value.lock().expect("Mutex poisoned");
        *value = new_value;
        self.condition.notify_all();
    }

    pub fn wait_timeout_while<F: FnMut(&mut V) -> bool>(
        &self,
        timeout: Duration,
        predicate: F,
    ) -> Option<MutexGuard<V>> {
        let lock = self.value.lock().expect("Mutex poison");
        let (lock, result) = self
            .condition
            .wait_timeout_while(lock, timeout, predicate)
            .expect("Mutex posion");

        if result.timed_out() {
            None
        } else {
            Some(lock)
        }
    }
}
