use std::collections::{Map, MutableMap, HashMap};
use std::hash::Hash;

pub trait Indexable<K, V> {
    fn get_key(&self, &K) -> Option<&V>;
    fn set_key(&mut self, K, V) -> bool;
    fn del_key(&mut self, &K) -> bool;
}

pub struct Index<T> {
    kv: T,
}

impl<K: Hash + Eq, V: Clone> Index<HashMap<K,V>> {
    #[allow(unused_mut)]
    fn new() -> Index<HashMap<K,V>> {
        let mut kv = HashMap::new();
        let mut index = Index {kv: kv};
        return index;
    }
}

impl<K: Hash + Eq, V: Clone, T: Map<K,V> + MutableMap<K,V>> Indexable<K, V> for Index<T> {
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
    use std::collections::HashMap;
    use super::{Indexable, Index};

    #[test]
    fn returns_none() {
        let key = "foo".to_string();
        let index: Index<HashMap<String, String>> = Index::new();

        assert_eq!(index.get_key(&key), None);
    }

    #[test]
    fn assigns_values() {
        let key = "key".to_string();
        let value = "value".to_string();
        let mut index: Index<HashMap<String, String>> = Index::new();

        index.set_key(key.clone(), value.clone());
        
        assert!(index.get_key(&key) == Some(&value));
    }

    #[test]
    fn removes_values() {
        let key = "key".to_string();
        let value = "value".to_string();
        let mut index: Index<HashMap<String, String>> = Index::new();

        index.set_key(key.clone(), value.clone());
        index.del_key(&key);

        assert!(index.get_key(&key) == None);
    }
}
