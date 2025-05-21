use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Mutex;

pub struct ConcurrentHashMap<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    pub num_shards: usize,
    pub shards: Vec<Mutex<HashMap<K, V>>>,
}

impl<K, V> ConcurrentHashMap<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    pub fn new(num_shards: usize) -> Self {
        assert!(num_shards > 0, "Number of shards must be positive");
        let mut shards = Vec::with_capacity(num_shards);
        for _ in 0..num_shards {
            shards.push(Mutex::new(HashMap::new()));
        }
        Self { num_shards, shards }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash_value = hasher.finish();
        let shard_index = (hash_value % self.num_shards as u64) as usize;

        let shard_lock = self.shards[shard_index].lock().unwrap();
        shard_lock.get(key).cloned()
    }

    pub fn insert(&self, key: K, value: V) -> Option<V> {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher); // K needs Hash, key is owned but hash takes &K
        let hash_value = hasher.finish();
        let shard_index = (hash_value % self.num_shards as u64) as usize;

        let mut shard_lock = self.shards[shard_index].lock().unwrap(); // MutexGuard needs to be mutable
        shard_lock.insert(key, value) // HashMap::insert takes K and V
    }

    pub fn remove(&self, key: &K) -> Option<V> {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash_value = hasher.finish();
        let shard_index = (hash_value % self.num_shards as u64) as usize;

        let mut shard_lock = self.shards[shard_index].lock().unwrap();
        shard_lock.remove(key)
    }
}

#[cfg(test)]
mod tests {
    use super::ConcurrentHashMap;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_insert_and_get_single_shard() {
        let map = ConcurrentHashMap::<String, i32>::new(1);

        map.insert("key1".to_string(), 10);
        assert_eq!(map.get(&"key1".to_string()), Some(10));

        map.insert("key2".to_string(), 20);
        assert_eq!(map.get(&"key2".to_string()), Some(20));
        assert_eq!(map.get(&"key1".to_string()), Some(10)); // Ensure previous key is still there
        assert_eq!(map.get(&"non_existent_key".to_string()), None);
    }

    #[test]
    fn test_insert_and_get_multiple_shards() {
        let map = ConcurrentHashMap::<String, i32>::new(4);
        let keys = [
            "key0", "key1", "key2", "key3", "key4", "key5", "key6", "key7",
            "key8", "key9",
        ];
        for (i, key) in keys.iter().enumerate() {
            map.insert(key.to_string(), i as i32);
        }

        for (i, key) in keys.iter().enumerate() {
            assert_eq!(map.get(&key.to_string()), Some(i as i32));
        }
        assert_eq!(map.get(&"non_existent_key".to_string()), None);
    }

    #[test]
    fn test_insert_overwrite() {
        let map = ConcurrentHashMap::<String, i32>::new(2);

        let initial_insert = map.insert("key1".to_string(), 20);
        assert_eq!(initial_insert, None);
        assert_eq!(map.get(&"key1".to_string()), Some(20));

        let overwrite_insert = map.insert("key1".to_string(), 30);
        assert_eq!(overwrite_insert, Some(20));
        assert_eq!(map.get(&"key1".to_string()), Some(30));
    }

    #[test]
    fn test_remove() {
        let map = ConcurrentHashMap::<String, i32>::new(3);

        map.insert("key1".to_string(), 40);
        assert_eq!(map.get(&"key1".to_string()), Some(40));

        let remove_existing = map.remove(&"key1".to_string());
        assert_eq!(remove_existing, Some(40));
        assert_eq!(map.get(&"key1".to_string()), None);

        let remove_again = map.remove(&"key1".to_string());
        assert_eq!(remove_again, None);

        let remove_non_existent = map.remove(&"non_existent_key".to_string());
        assert_eq!(remove_non_existent, None);
    }

    #[test]
    fn test_concurrent_insert_different_shards() {
        let num_shards = 4;
        let num_threads = 8;
        let ops_per_thread = 100;
        let map = Arc::new(ConcurrentHashMap::<String, usize>::new(num_shards));
        let mut handles = vec![];

        for i in 0..num_threads {
            let map_clone = Arc::clone(&map);
            let handle = thread::spawn(move || {
                for j in 0..ops_per_thread {
                    let key = format!("key_{}_{}", i, j);
                    let value = i * ops_per_thread + j;
                    map_clone.insert(key, value);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        for i in 0..num_threads {
            for j in 0..ops_per_thread {
                let key = format!("key_{}_{}", i, j);
                let expected_value = i * ops_per_thread + j;
                assert_eq!(map.get(&key), Some(expected_value));
            }
        }
        
        // Optional: Check total count if easy, otherwise rely on individual key checks.
        // For this specific test, all keys are unique and should be present.
        let mut total_items = 0;
        for shard_mutex in map.shards.iter() {
            let shard_lock = shard_mutex.lock().unwrap();
            total_items += shard_lock.len();
        }
        assert_eq!(total_items, num_threads * ops_per_thread);
    }

    #[test]
    fn test_concurrent_insert_same_shard_potential() {
        let num_shards = 2;
        let num_threads = 10;
        let num_ops_per_thread = 1000;
        let num_keys_to_contest = 5;
        let map = Arc::new(ConcurrentHashMap::<usize, usize>::new(num_shards));
        let mut handles = vec![];

        for thread_id in 0..num_threads {
            let map_clone = Arc::clone(&map);
            let handle = thread::spawn(move || {
                for j in 0..num_ops_per_thread {
                    let key = j % num_keys_to_contest;
                    map_clone.insert(key, thread_id);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        for k in 0..num_keys_to_contest {
            let value = map.get(&k);
            assert!(value.is_some(), "Key {} should have a value", k);
            // We can also check if the value is one of the thread_ids
            if let Some(thread_id_val) = value {
                assert!(thread_id_val < num_threads, "Value for key {} is an invalid thread_id", k);
            }
        }
    }
}
