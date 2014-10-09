use std::collections::HashMap;
use std::hash::Hash;

trait Index<K, V> {
    fn get_key(&self, &K) -> Option<&V>;
    fn set_key(&mut self, K, V) -> bool;
    fn del_key(&mut self, &K) -> bool;
}

pub struct HashMapIndex<K, V> {
    kv: HashMap<K, V>,
}


impl<K: Hash + Eq, V> HashMapIndex<K, V> {
    #[allow(unused_mut)]
    fn new() -> HashMapIndex<K, V> {
        let mut kv: HashMap<K, V> = HashMap::new();
        let mut index: HashMapIndex<K, V> = HashMapIndex {kv: kv};
        return index;
    }
}

impl<K: Hash + Eq, V: Clone> Index<K, V> for HashMapIndex<K, V> {
    fn get_key(&self, key: &K) -> Option<&V> {
        return self.kv.find(key);
    }
    fn set_key(&mut self, key: K, value: V) -> bool {
        return self.kv.insert(key, value);
    }
    fn del_key(&mut self, key: &K) -> bool {
        return self.kv.remove(key);
    }
}


#[cfg(test)]
mod test {
    use super::{HashMapIndex, Index};

    #[test]
    fn returns_none() {
        let key = "foo".to_string();
        let index: HashMapIndex<String, String> = HashMapIndex::new();
        assert_eq!(index.get_key(&key), None);
    }

    #[test]
    fn assigns_values() {
        let key = "key".to_string();
        let value = "value".to_string();
        let mut index: HashMapIndex<String, String> = HashMapIndex::new();
        index.set_key(key.clone(), value.clone());
        assert!(index.get_key(&key) == Some(&value));
    }

    #[test]
    fn removes_values() {
        let key = "key".to_string();
        let value = "value".to_string();
        let mut index: HashMapIndex<String, String> = HashMapIndex::new();
        index.set_key(key.clone(), value.clone());
        index.del_key(&key);
        assert!(index.get_key(&key) == None);
    }
}
