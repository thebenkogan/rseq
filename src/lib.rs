use std::{
    ops::{Add, Div, Mul, Sub},
    rc::Rc,
};

struct RSeq {
    pub head: i32,
    pub tail: Rc<dyn Fn() -> Self>,
}

impl Clone for RSeq {
    fn clone(&self) -> Self {
        Self {
            head: self.head,
            tail: Rc::clone(&self.tail),
        }
    }
}

impl RSeq {
    fn cnst(v: i32) -> Self {
        Self {
            head: v,
            tail: Rc::new(move || Self::cnst(v)),
        }
    }

    fn incr(start: i32) -> Self {
        Self {
            head: start,
            tail: Rc::new(move || Self::incr(start + 1)),
        }
    }

    fn thunk(&self) -> Self {
        (self.tail)()
    }

    fn take(&self, n: usize) -> Vec<i32> {
        let mut out = Vec::with_capacity(n);
        out.push(self.head);
        let mut curr = self.thunk();
        for _ in 0..n - 1 {
            out.push(curr.head);
            curr = curr.thunk();
        }
        out
    }

    fn map(&self, f: impl Fn(i32) -> i32 + Copy + 'static) -> Self {
        let tail = self.thunk();
        Self {
            head: f(self.head),
            tail: Rc::new(move || tail.map(f)),
        }
    }

    fn filter(&self, f: impl Fn(i32) -> bool + Copy + 'static) -> Self {
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
}

impl Add for &RSeq {
    type Output = RSeq;

    fn add(self, rhs: Self) -> Self::Output {
        let ltail = self.thunk();
        let rtail = rhs.thunk();
        RSeq {
            head: self.head + rhs.head,
            tail: Rc::new(move || &ltail + &rtail),
        }
    }
}

impl Mul for &RSeq {
    type Output = RSeq;

    fn mul(self, rhs: Self) -> Self::Output {
        let ltail = self.thunk();
        let rtail = rhs.thunk();
        RSeq {
            head: self.head * rhs.head,
            tail: Rc::new(move || &ltail * &rtail),
        }
    }
}

impl Sub for &RSeq {
    type Output = RSeq;

    fn sub(self, rhs: Self) -> Self::Output {
        let ltail = self.thunk();
        let rtail = rhs.thunk();
        RSeq {
            head: self.head - rhs.head,
            tail: Rc::new(move || &ltail - &rtail),
        }
    }
}

impl Div for &RSeq {
    type Output = RSeq;

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
}
