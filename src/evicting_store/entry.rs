trait Entry<T, V> {
    fn new(token: T) -> Self;
    fn fetch(&mut self) -> V;
    fn flush(&mut self);
}
