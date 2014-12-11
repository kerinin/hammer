enum CacheState<V,T> {
    T1,
    T2,
    B1,
    B2,
}

struct Page<T, V> {
    state: CacheState
    link: Option<Page<T>>,
    data: Option<&V>,
}

impl Page<T, V> {
    fn new() -> Page<T,V> {
        Page {state: T1, link: None, data: None}
    }

    fn fetch(&mut self, token: T) {
        // No-op, becasue we don't anticipate having a backing store...
    }
    fn flush(&mut self) {
        // No-op, becasue we don't anticipate having a backing store...
    }
}
