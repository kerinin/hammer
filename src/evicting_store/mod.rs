mod eviction;
mod arc;

trait EvictingStore<V, T> {
    fn insert(&mut self, value: V) -> (token: T, evictions: Vec<Eviction>); 

    fn get(&mut self, token: T) -> Option<V>;
}
