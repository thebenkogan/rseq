use std::{
    ops::{Add, Div, Mul, Sub},
    rc::Rc,
};

use num_traits::Num;

struct RSeq<T> {
    pub head: T,
    pub tail: Rc<dyn Fn() -> Self>,
}

struct RSeqIter<T> {
    curr: RSeq<T>,
}

impl<T> RSeqIter<T> {
    fn new(start: RSeq<T>) -> Self {
        Self { curr: start }
    }
}

impl<T: Copy + 'static> Iterator for RSeqIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let out = self.curr.head;
        self.curr = self.curr.thunk();
        Some(out)
    }
}

impl<T: Copy + 'static> IntoIterator for RSeq<T> {
    type Item = T;

    type IntoIter = RSeqIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        RSeqIter::new(self)
    }
}

impl<T: Copy> Clone for RSeq<T> {
    fn clone(&self) -> Self {
        Self {
            head: self.head,
            tail: Rc::clone(&self.tail),
        }
    }
}

impl<T> RSeq<T>
where
    T: Num + Copy + 'static,
{
    fn incr(start: T) -> Self {
        let next = start + T::one();
        Self {
            head: start,
            tail: Rc::new(move || Self::incr(next)),
        }
    }
}

impl<T> RSeq<T>
where
    T: Copy + 'static,
{
    fn cnst(v: T) -> Self {
        Self {
            head: v,
            tail: Rc::new(move || Self::cnst(v)),
        }
    }

    fn thunk(&self) -> Self {
        (self.tail)()
    }

    fn take(&self, n: usize) -> Vec<T> {
        let mut out = Vec::with_capacity(n);
        out.push(self.head);
        let mut curr = self.thunk();
        for _ in 0..n - 1 {
            out.push(curr.head);
            curr = curr.thunk();
        }
        out
    }

    fn map<M>(&self, f: impl Fn(T) -> M + Copy + 'static) -> RSeq<M> {
        let tail = self.thunk();
        RSeq {
            head: f(self.head),
            tail: Rc::new(move || tail.map(f)),
        }
    }

    fn filter(&self, f: impl Fn(T) -> bool + Copy + 'static) -> Self {
        let tail = self.thunk();
        if f(self.head) {
            Self {
                head: self.head,
                tail: Rc::new(move || tail.filter(f)),
            }
        } else {
            tail.filter(f)
        }
    }

    fn interleave(left: &Self, right: &Self) -> Self {
        let ltail = left.thunk();
        let rclone = right.clone();
        Self {
            head: left.head,
            tail: Rc::new(move || Self::interleave(&rclone, &ltail)),
        }
    }

    fn unfold(start: T, f: impl Fn(T) -> T + Copy + 'static) -> Self {
        let next = f(start);
        Self {
            head: start,
            tail: Rc::new(move || Self::unfold(next, f)),
        }
    }
}

impl<T> Add for &RSeq<T>
where
    T: Add<Output = T> + Copy + 'static,
{
    type Output = RSeq<T>;

    fn add(self, rhs: Self) -> Self::Output {
        let ltail = self.thunk();
        let rtail = rhs.thunk();
        RSeq {
            head: self.head + rhs.head,
            tail: Rc::new(move || &ltail + &rtail),
        }
    }
}

impl<T> Mul for &RSeq<T>
where
    T: Mul<Output = T> + Copy + 'static,
{
    type Output = RSeq<T>;

    fn mul(self, rhs: Self) -> Self::Output {
        let ltail = self.thunk();
        let rtail = rhs.thunk();
        RSeq {
            head: self.head * rhs.head,
            tail: Rc::new(move || &ltail * &rtail),
        }
    }
}

impl<T> Sub for &RSeq<T>
where
    T: Sub<Output = T> + Copy + 'static,
{
    type Output = RSeq<T>;

    fn sub(self, rhs: Self) -> Self::Output {
        let ltail = self.thunk();
        let rtail = rhs.thunk();
        RSeq {
            head: self.head - rhs.head,
            tail: Rc::new(move || &ltail - &rtail),
        }
    }
}

impl<T> Div for &RSeq<T>
where
    T: Div<Output = T> + Copy + 'static,
{
    type Output = RSeq<T>;

    fn div(self, rhs: Self) -> Self::Output {
        let ltail = self.thunk();
        let rtail = rhs.thunk();
        RSeq {
            head: self.head / rhs.head,
            tail: Rc::new(move || &ltail / &rtail),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cnst() {
        let s = RSeq::cnst(1);
        assert_eq!(s.take(5), vec![1, 1, 1, 1, 1]);
    }

    #[test]
    fn incr() {
        let s = RSeq::incr(2);
        assert_eq!(s.take(5), vec![2, 3, 4, 5, 6]);
    }

    #[test]
    fn ops() {
        let s = RSeq::cnst(8);
        let t = RSeq::cnst(2);
        assert_eq!((&s + &t).take(5), vec![10, 10, 10, 10, 10]);
        assert_eq!((&s - &t).take(5), vec![6, 6, 6, 6, 6]);
        assert_eq!((&s * &t).take(5), vec![16, 16, 16, 16, 16]);
        assert_eq!((&s / &t).take(5), vec![4, 4, 4, 4, 4]);
    }

    #[test]
    fn map() {
        let s = RSeq::incr(2);
        assert_eq!(s.map(|n| n * 2).take(5), vec![4, 6, 8, 10, 12]);
    }

    #[test]
    fn filter() {
        let s = RSeq::incr(2);
        assert_eq!(s.filter(|n| n % 2 == 0).take(5), vec![2, 4, 6, 8, 10]);
        assert_eq!(s.filter(|n| n % 2 != 0).take(5), vec![3, 5, 7, 9, 11]);
    }

    #[test]
    fn interleave() {
        let s = RSeq::incr(0).filter(|n| n % 2 == 0);
        let t = RSeq::incr(0).filter(|n| n % 2 != 0);
        assert_eq!(RSeq::interleave(&s, &t).take(5), vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn iterator() {
        let s = RSeq::incr(0).filter(|n| n % 2 == 0);
        let iter = s.into_iter();
        assert_eq!(
            iter.map(|n| n * n).take(10).collect::<Vec<i32>>(),
            vec![0, 4, 16, 36, 64, 100, 144, 196, 256, 324]
        );
    }

    #[test]
    fn unfold() {
        let s = RSeq::unfold((0, 1), |(x, y)| (y, x + y)).map(|(x, _)| x);
        assert_eq!(s.take(10), vec![0, 1, 1, 2, 3, 5, 8, 13, 21, 34]);
    }
}
