pub trait Entry<T, V> {
    fn new() -> Self;
    fn fetch(&mut self, token: T) -> V;
    fn value(self) -> Option<V>;
}
