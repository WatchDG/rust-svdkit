use core::marker::PhantomData;
use super::traits::RegValue;

#[repr(transparent)]
pub struct RO<T>(core::cell::UnsafeCell<T>);

#[repr(transparent)]
pub struct WO<T>(core::cell::UnsafeCell<T>);

#[repr(transparent)]
pub struct RW<T>(core::cell::UnsafeCell<T>);

#[repr(transparent)]
pub struct W1S<T>(core::cell::UnsafeCell<T>);

#[repr(transparent)]
pub struct W1C<T>(core::cell::UnsafeCell<T>);

#[repr(transparent)]
pub struct W0S<T>(core::cell::UnsafeCell<T>);

#[repr(transparent)]
pub struct W0C<T>(core::cell::UnsafeCell<T>);

#[repr(transparent)]
pub struct WT<T>(core::cell::UnsafeCell<T>);

unsafe impl<T> Send for RO<T> {}
unsafe impl<T> Sync for RO<T> {}
unsafe impl<T> Send for WO<T> {}
unsafe impl<T> Sync for WO<T> {}
unsafe impl<T> Send for RW<T> {}
unsafe impl<T> Sync for RW<T> {}
unsafe impl<T> Send for W1S<T> {}
unsafe impl<T> Sync for W1S<T> {}
unsafe impl<T> Send for W1C<T> {}
unsafe impl<T> Sync for W1C<T> {}
unsafe impl<T> Send for W0S<T> {}
unsafe impl<T> Sync for W0S<T> {}
unsafe impl<T> Send for W0C<T> {}
unsafe impl<T> Sync for W0C<T> {}
unsafe impl<T> Send for WT<T> {}
unsafe impl<T> Sync for WT<T> {}

impl<T: Copy> RO<T> {
    #[inline(always)]
    pub fn read(&self) -> T {
        unsafe { core::ptr::read_volatile(self.0.get()) }
    }
}

impl<T: Copy> WO<T> {
    #[inline(always)]
    pub fn write(&self, v: T) {
        unsafe { core::ptr::write_volatile(self.0.get(), v) }
    }
}

impl<T: Copy> RW<T> {
    #[inline(always)]
    pub fn read(&self) -> T {
        unsafe { core::ptr::read_volatile(self.0.get()) }
    }
    #[inline(always)]
    pub fn write(&self, v: T) {
        unsafe { core::ptr::write_volatile(self.0.get(), v) }
    }
}

impl<T: RegValue> W1S<T> {
    #[inline(always)]
    pub fn read(&self) -> T {
        unsafe { core::ptr::read_volatile(self.0.get()) }
    }
    #[inline(always)]
    pub fn write(&self, v: T) {
        unsafe { core::ptr::write_volatile(self.0.get(), v) }
    }
    #[inline(always)]
    pub fn set_bits(&self, mask: T) {
        self.write(mask)
    }
}

impl<T: RegValue> W1C<T> {
    #[inline(always)]
    pub fn read(&self) -> T {
        unsafe { core::ptr::read_volatile(self.0.get()) }
    }
    #[inline(always)]
    pub fn write(&self, v: T) {
        unsafe { core::ptr::write_volatile(self.0.get(), v) }
    }
    #[inline(always)]
    pub fn clear_bits(&self, mask: T) {
        self.write(mask)
    }
}

impl<T: RegValue> W0S<T> {
    #[inline(always)]
    pub fn read(&self) -> T {
        unsafe { core::ptr::read_volatile(self.0.get()) }
    }
    #[inline(always)]
    pub fn write(&self, v: T) {
        unsafe { core::ptr::write_volatile(self.0.get(), v) }
    }
    #[inline(always)]
    pub fn set_bits(&self, mask: T) {
        let m = mask.to_u64() & T::MASK;
        let v = (!m) & T::MASK;
        self.write(T::from_u64(v));
    }
}

impl<T: RegValue> W0C<T> {
    #[inline(always)]
    pub fn read(&self) -> T {
        unsafe { core::ptr::read_volatile(self.0.get()) }
    }
    #[inline(always)]
    pub fn write(&self, v: T) {
        unsafe { core::ptr::write_volatile(self.0.get(), v) }
    }
    #[inline(always)]
    pub fn clear_bits(&self, mask: T) {
        let m = mask.to_u64() & T::MASK;
        let v = (!m) & T::MASK;
        self.write(T::from_u64(v));
    }
}

impl<T: RegValue> WT<T> {
    #[inline(always)]
    pub fn read(&self) -> T {
        unsafe { core::ptr::read_volatile(self.0.get()) }
    }
    #[inline(always)]
    pub fn write(&self, v: T) {
        unsafe { core::ptr::write_volatile(self.0.get(), v) }
    }
    #[inline(always)]
    pub fn toggle_bits(&self, mask: T) {
        self.write(mask)
    }
}

pub struct Unwritten;
pub struct Written;
pub struct WOOnce<T, S> {
    pub base: usize,
    pub offset: usize,
    pub _state: PhantomData<S>,
    pub _t: PhantomData<T>,
}
pub struct RWOnce<T, S> {
    pub base: usize,
    pub offset: usize,
    pub _state: PhantomData<S>,
    pub _t: PhantomData<T>,
}

impl<T: Copy, S> RWOnce<T, S> {
    #[inline(always)]
    pub unsafe fn read(&self) -> T {
        let p = (self.base + self.offset) as *const RW<T>;
        (*p).read()
    }
}

impl<T: Copy> WOOnce<T, Unwritten> {
    #[inline(always)]
    pub unsafe fn write(self, v: T) -> WOOnce<T, Written> {
        let p = (self.base + self.offset) as *const WO<T>;
        (*p).write(v);
        WOOnce {
            base: self.base,
            offset: self.offset,
            _state: PhantomData,
            _t: PhantomData,
        }
    }
}

impl<T: Copy> RWOnce<T, Unwritten> {
    #[inline(always)]
    pub unsafe fn write(self, v: T) -> RWOnce<T, Written> {
        let p = (self.base + self.offset) as *const RW<T>;
        (*p).write(v);
        RWOnce {
            base: self.base,
            offset: self.offset,
            _state: PhantomData,
            _t: PhantomData,
        }
    }
}
