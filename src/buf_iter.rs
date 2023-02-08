use std::slice::SliceIndex;

pub trait BufIterType<T: Copy> {
    fn buf(&self) -> &[T];
    fn idx(&self) -> usize;
    fn half_mut(&mut self) -> (&mut usize, &[T], &mut usize);
    fn apply_defered(&mut self) {
        let (idx, _, defered) = self.half_mut();
        *idx += *defered;
        *defered = 0;
    }
    fn len(&self) -> usize {
        self.buf().len()
    }
}

pub trait MutBufIterType<T: Copy>: BufIterType<T> {
    fn full_mut(&mut self) -> (&mut usize, &mut [T], &mut usize);
}

pub trait SliceIter<T: Copy>: BufIterType<T> {
    fn slide(&mut self, backward: usize, forward: usize) -> Option<&[T]> {
        let (idx, buf, _) = self.half_mut();
        let res = buf.get(*idx - backward..*idx - backward + forward);
        if res.is_some() {
            *idx += forward - backward;
        }
        res
    }

    fn slide_defered(&mut self, backward: usize, forward: usize) -> Option<&[T]> {
        let (idx, buf, defered) = self.half_mut();
        let res = buf.get(*idx - backward..*idx - backward + forward);
        if res.is_some() {
            *defered += forward - backward;
        }
        res
    }

    fn look_forward(&self, step: usize) -> Option<&[T]> {
        self.buf().get(self.idx()..self.idx() + step)
    }

    fn look_backward(&self, step: usize) -> Option<&[T]> {
        self.buf().get(self.idx() - step..self.idx())
    }

    fn step_forward(&mut self, step: usize) -> Option<&[T]> {
        self.slide(0, step)
    }

    fn step_forward_defered(&mut self, step: usize) -> Option<&[T]> {
        self.slide_defered(0, step)
    }

    fn step_backward(&mut self, step: usize) -> Option<&[T]> {
        self.slide(step, 0)
    }

    fn step_backward_defered(&mut self, step: usize) -> Option<&[T]> {
        self.slide_defered(step, 0)
    }

    fn look_one(&self) -> Option<&T> {
        self.buf().get(self.idx())
    }

    fn look_one_backward(&self) -> Option<&T> {
        if self.idx() == 0 {
            None
        } else {
            self.buf().get(self.idx() - 1)
        }
    }

    fn step_one(&mut self) -> Option<&T> {
        let (idx, buf, _) = self.half_mut();
        let res = buf.get(*idx);
        if res.is_some() {
            *idx += 1;
        }
        res
    }

    fn step_one_backward(&mut self) -> Option<&T> {
        let (idx, buf, _) = self.half_mut();
        let res = buf.get(*idx - 1);
        if res.is_some() {
            *idx -= 1;
        }
        res
    }
}

pub trait SliceIterMut<T: Copy>: SliceIter<T> + MutBufIterType<T> {
    fn slide_mut(&mut self, backward: usize, forward: usize) -> Option<&mut [T]> {
        let (idx, buf, _) = self.full_mut();
        let res = buf.get_mut(*idx - backward..*idx - backward + forward);
        if res.is_some() {
            *idx += forward - backward;
        }
        res
    }

    fn slide_mut_defered(&mut self, backward: usize, forward: usize) -> Option<&mut [T]> {
        let (idx, buf, defered) = self.full_mut();
        let res = buf.get_mut(*idx - backward..*idx - backward + forward);
        if res.is_some() {
            *defered += forward - backward;
        }
        res
    }

    fn look_forward_mut(&mut self, step: usize) -> Option<&mut [T]> {
        let (idx, buf, _) = self.full_mut();
        buf.get_mut(*idx..*idx + step)
    }

    fn look_backward_mut(&mut self, step: usize) -> Option<&mut [T]> {
        let (idx, buf, _) = self.full_mut();
        buf.get_mut(*idx - step..*idx)
    }

    fn step_forward_mut(&mut self, step: usize) -> Option<&mut [T]> {
        self.slide_mut(0, step)
    }

    fn step_forward_mut_defered(&mut self, step: usize) -> Option<&mut [T]> {
        self.slide_mut_defered(0, step)
    }

    fn step_backward_mut(&mut self, step: usize) -> Option<&mut [T]> {
        self.slide_mut(step, 0)
    }

    fn step_backward_mut_defered(&mut self, step: usize) -> Option<&mut [T]> {
        self.slide_mut_defered(step, 0)
    }

    fn copy_from_slice(&mut self, slice: &[T], slice_size: usize, to_step: bool) {
        let (idx, buf, _) = self.full_mut();
        buf[*idx..*idx + slice_size].copy_from_slice(slice);
        if to_step {
            *idx += slice_size;
        }
    }

    fn look_one_mut(&mut self) -> Option<&mut T> {
        let (idx, buf, _) = self.full_mut();
        buf.get_mut(*idx)
    }

    fn step_one_mut(&mut self) -> Option<&mut T> {
        let (idx, buf, _) = self.full_mut();
        let res = buf.get_mut(*idx);
        if res.is_some() {
            *idx += 1;
        }
        res
    }

    fn set_next(&mut self, next: &[T]) {
        let (idx, buf, _) = self.full_mut();
        buf[*idx..next.len()].copy_from_slice(next);
    }

    fn set_next_one(&mut self, next: T) -> Option<&mut T> {
        let el = self.step_one_mut()?;
        *el = next;
        Some(el)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BufIter<'a, T: Copy> {
    buf: &'a [T],
    idx: usize,
    defered: usize,
}

impl<'a, T: Copy> BufIter<'a, T> {
    pub fn new(buf: &'a [T]) -> Self {
        BufIter {
            buf,
            idx: 0,
            defered: 0,
        }
    }

    pub fn from<I>(buf: &'a [T], index: I) -> Option<Self>
    where
        I: SliceIndex<[T], Output = [T]>,
    {
        match buf.get(index) {
            Some(buf) => Some(BufIter {
                buf,
                idx: 0,
                defered: 0,
            }),
            None => None,
        }
    }
}

impl<'a, T: Copy> BufIterType<T> for BufIter<'a, T> {
    fn buf(&self) -> &'a [T] {
        self.buf
    }
    fn idx(&self) -> usize {
        self.idx
    }
    fn half_mut(&mut self) -> (&mut usize, &[T], &mut usize) {
        (&mut self.idx, self.buf, &mut self.defered)
    }
}

impl<'a, T: Copy> SliceIter<T> for BufIter<'a, T> {}

#[derive(Debug)]
pub struct MutBufIter<'a, T: Copy> {
    buf: &'a mut [T],
    idx: usize,
    defered: usize,
}

impl<'a, T: Copy> MutBufIter<'a, T> {
    pub fn new(buf: &'a mut [T]) -> Self {
        MutBufIter {
            buf,
            idx: 0,
            defered: 0,
        }
    }

    pub fn from<I>(buf: &'a mut [T], index: I) -> Option<Self>
    where
        I: SliceIndex<[T], Output = [T]>,
    {
        match buf.get_mut(index) {
            Some(buf) => Some(MutBufIter {
                buf,
                idx: 0,
                defered: 0,
            }),
            None => None,
        }
    }
}

impl<'a, T: Copy> BufIterType<T> for MutBufIter<'a, T> {
    fn buf(&self) -> &[T] {
        &self.buf
    }
    fn idx(&self) -> usize {
        self.idx
    }
    fn half_mut(&mut self) -> (&mut usize, &[T], &mut usize) {
        (&mut self.idx, self.buf, &mut self.defered)
    }
}

impl<'a, T: Copy> MutBufIterType<T> for MutBufIter<'a, T> {
    fn full_mut(&mut self) -> (&mut usize, &mut [T], &mut usize) {
        (&mut self.idx, &mut self.buf, &mut self.defered)
    }
}

impl<'a, T: Copy> SliceIter<T> for MutBufIter<'a, T> {}

impl<'a, T: Copy> SliceIterMut<T> for MutBufIter<'a, T> {}
