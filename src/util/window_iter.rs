use std::iter::{Skip, StepBy};

pub struct WindowIter<A, B> {
    iter_a: A,
    iter_b: B,
}

impl<A: Iterator, B: Iterator> Iterator for WindowIter<A, B> {
    type Item = (Option<A::Item>, Option<B::Item>);

    fn next(&mut self) -> Option<Self::Item> {
        let a = self.iter_a.next();
        let b = self.iter_b.next();
        if a.is_none() && b.is_none() {
            None
        } else {
            Some((a, b))
        }
    }
}

pub trait IntoWindowIter {
    fn into_window_iter(self) -> WindowIter<StepBy<Self>, StepBy<Skip<Self>>>
    where
        Self: Sized + Clone;
}

impl<T: Iterator> IntoWindowIter for T {
    fn into_window_iter(self) -> WindowIter<StepBy<Self>, StepBy<Skip<Self>>>
    where
        Self: Sized + Clone,
    {
        WindowIter {
            iter_b: self.clone().skip(1).step_by(2),
            iter_a: self.step_by(2),
        }
    }
}
