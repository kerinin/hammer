use std::collections::{Map, MutableMap};
use std::hash::Hash;

pub trait Indexable<K,V>: Map<K,V> + MutableMap<K,V>;
pub trait Indexable<K, V> {
    fn find(&self, &K) -> Option<&V>;
    fn insert(&mut self, K, V) -> bool;
    fn remove(&mut self, &K) -> bool;

//    //fn get_key(&self, &K) -> Option<&V>;
//    //fn set_key(&mut self, K, V) -> bool;
//    //fn del_key(&mut self, &K) -> bool;
}


//impl<K: Hash + Eq, V: Clone, T: Map<K,V> + MutableMap<K,V>> Indexable<K, V> for T {
//    fn find(&self, key: &K) -> Option<&V> {
//        return self.find(key);
//    }
//    fn insert(&mut self, key: K, value: V) -> bool {
//        return self.insert(key, value);
//    }
//    fn remove(&mut self, key: &K) -> bool {
//        return self.remove(key);
//    }
//}

//impl<K: Hash + Eq, V: Clone, T: Map<K,V> + MutableMap<K,V>> Indexable<K, V> for T {
//    fn get_key(&self, key: &K) -> Option<&V> {
//        return self.find(key);
//    }
//    fn set_key(&mut self, key: K, value: V) -> bool {
//        return self.insert(key, value);
//    }
//    fn del_key(&mut self, key: &K) -> bool {
//        return self.remove(key);
//    }
//}


#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use super::{Indexable};

    fn indexable(i: Indexable<String, String>) -> bool { return true; }

    #[test]
    fn is_indexable() {
        let index: HashMap<String, String> = HashMap::new();
        assert!(indexable(index));
    }

    #[test]
    fn returns_none() {
        let key = "foo".to_string();
        let index: HashMap<String, String> = HashMap::new();

        assert_eq!(index.find(&key), None);
    }

    #[test]
    fn assigns_values() {
        let key = "key".to_string();
        let value = "value".to_string();
        let mut index: HashMap<String, String> = HashMap::new();

        index.insert(key.clone(), value.clone());
        
        assert!(index.find(&key) == Some(&value));
    }

    #[test]
    fn removes_values() {
        let key = "key".to_string();
        let value = "value".to_string();
        let mut index: HashMap<String, String> = HashMap::new();

        index.insert(key.clone(), value.clone());
        index.remove(&key);

        assert!(index.find(&key) == None);
    }
}
