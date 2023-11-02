struct CountRemoveIf<I: Iterator, R: Fn(&I::Item) -> bool + ?Sized> {
    iter: I,
    remove: Box<R>,
    count: usize,
}

trait CountRemove: Iterator {
    fn count_remove<M: PartialEq<&'a Self::Item> + 'static>(self, what: M) -> CountRemoveIf<Self, dyn Fn(&Self::Item) -> bool> where Self: Sized {
        CountRemoveIf { iter: self, remove: Box::new(move |item| what == item), count: 0 }
    }
    fn count_remove_from<M: PartialEq<&'a Self::Item> + 'static>(self, what: M, count: usize) -> CountRemoveIf<Self, dyn Fn(&Self::Item) -> bool> where Self: Sized {
        CountRemoveIf { iter: self, remove: Box::new(move |item| what == item), count }
    }
    fn count_remove_if<F: Fn(&Self::Item) -> bool>(self, remove: F) -> CountRemoveIf<Self, F> where Self: Sized {
        CountRemoveIf { iter: self, remove: Box::new(remove), count: 0 }
    }
    fn count_remove_from_if<F: Fn(&Self::Item) -> bool>(self, remove: F, count: usize) -> CountRemoveIf<Self, F> where Self: Sized {
        CountRemoveIf { iter: self, remove: Box::new(remove), count }
    }
}

impl<I: Iterator, R: Fn(I::Item) -> bool> CountRemoveIf<I, R> {
    fn count(&self) -> usize {
        self.count
    }
}

impl<I: Iterator, R: Fn(I::Item) -> bool> Iterator for CountRemoveIf<I, R> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.next() {
                Some(v) if (self.remove)(&v) => self.count += 1,
                item => break item
            }
        }
    }
}

impl<I: Iterator + DoubleEndedIterator, R: Fn(I::Item) -> bool> DoubleEndedIterator for CountRemoveIf<I, R> {
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.next_back() {
                Some(v) if (self.remove)(&v) => self.count += 1,
                item => break item
            }
        }
    }
}