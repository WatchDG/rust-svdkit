#[macro_export]
macro_rules! impl_ro_register {
    ($name:ident, $ty:ty) => {
        impl $name {
            #[inline(always)]
            pub fn read(&self) -> $ty {
                self.0.read()
            }
        }
    };
}

#[macro_export]
macro_rules! impl_wo_register {
    ($name:ident, $ty:ty) => {
        impl $name {
            #[inline(always)]
            pub fn write(&self, v: $ty) {
                self.0.write(v)
            }
        }
    };
}

#[macro_export]
macro_rules! impl_rw_register {
    ($name:ident, $ty:ty) => {
        impl $name {
            #[inline(always)]
            pub fn read(&self) -> $ty {
                self.0.read()
            }
            #[inline(always)]
            pub fn write(&self, v: $ty) {
                self.0.write(v)
            }
        }
    };
}

#[macro_export]
macro_rules! impl_w1s_register {
    ($name:ident, $ty:ty) => {
        impl $name {
            #[inline(always)]
            pub fn read(&self) -> $ty {
                self.0.read()
            }
            #[inline(always)]
            pub fn write(&self, v: $ty) {
                self.0.write(v)
            }
            #[inline(always)]
            pub fn set_bits(&self, mask: $ty) {
                self.0.set_bits(mask)
            }
        }
    };
}

#[macro_export]
macro_rules! impl_w1c_register {
    ($name:ident, $ty:ty) => {
        impl $name {
            #[inline(always)]
            pub fn read(&self) -> $ty {
                self.0.read()
            }
            #[inline(always)]
            pub fn write(&self, v: $ty) {
                self.0.write(v)
            }
            #[inline(always)]
            pub fn clear_bits(&self, mask: $ty) {
                self.0.clear_bits(mask)
            }
        }
    };
}

#[macro_export]
macro_rules! impl_w0s_register {
    ($name:ident, $ty:ty) => {
        impl $name {
            #[inline(always)]
            pub fn read(&self) -> $ty {
                self.0.read()
            }
            #[inline(always)]
            pub fn write(&self, v: $ty) {
                self.0.write(v)
            }
            #[inline(always)]
            pub fn set_bits(&self, mask: $ty) {
                self.0.set_bits(mask)
            }
        }
    };
}

#[macro_export]
macro_rules! impl_w0c_register {
    ($name:ident, $ty:ty) => {
        impl $name {
            #[inline(always)]
            pub fn read(&self) -> $ty {
                self.0.read()
            }
            #[inline(always)]
            pub fn write(&self, v: $ty) {
                self.0.write(v)
            }
            #[inline(always)]
            pub fn clear_bits(&self, mask: $ty) {
                self.0.clear_bits(mask)
            }
        }
    };
}

#[macro_export]
macro_rules! impl_wt_register {
    ($name:ident, $ty:ty) => {
        impl $name {
            #[inline(always)]
            pub fn read(&self) -> $ty {
                self.0.read()
            }
            #[inline(always)]
            pub fn write(&self, v: $ty) {
                self.0.write(v)
            }
            #[inline(always)]
            pub fn toggle_bits(&self, mask: $ty) {
                self.0.toggle_bits(mask)
            }
        }
    };
}
