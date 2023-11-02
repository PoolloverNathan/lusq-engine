pub trait SignallingFilterMapExt: Iterator {
    /// Similar to [filter_map], but sets `signal` to true once the filter removes an element.
    /// As iterators are lazily evaluated, it is only safe to check the value of `signal` after completely consuming the iterator.
    fn signalling_filter_map<'a, T, F>(self, map: F, signal: &'a mut bool) -> SignallingFilterMap<'a, Self, T, F> where Self: Sized, F: FnMut(Self::Item) -> Option<T> {
        SignallingFilterMap(self, map, signal)
    }
}
impl<I> SignallingFilterMapExt for I where I: Iterator {}

pub struct SignallingFilterMap<'a, I, R, F>(I, F, &'a mut bool) where I: 'a + Iterator, F: 'a + FnMut(I::Item) -> Option<R>;
impl<'a, I, R, F> Iterator for SignallingFilterMap<'a, I, R, F> where I: Iterator, F: FnMut(I::Item) -> Option<R> {
    type Item = R;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.0.next() {
                Some(v) => {
                    match self.1(v) {
                        Some(v) => break Some(v),
                        None => *self.2 = true
                    }
                }
                None => break None
            }
        }
    }
}