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

macro_rules! define_enum {
    (
        $(#[$doc:meta])*
        $name:ident : $type:ty,
        $(
            $(#[$vdoc:meta])*
            $variant:ident = $value:expr
        ),+ $(,)?
    ) => {
        $(#[$doc])*
        #[repr($type)]
        #[derive(Copy, Clone, Debug, PartialEq, Eq)]
        pub enum $name {
            $(
                $(#[$vdoc])*
                $variant = $value,
            )*
        }

        impl $name {
            #[inline(always)]
            pub const fn bits(self) -> $type {
                self as $type
            }

            #[inline(always)]
            pub const fn from_bits(v: $type) -> Option<Self> {
                match v {
                    $(
                        $value => Some(Self::$variant),
                    )*
                    _ => None,
                }
            }
        }

        impl From<$name> for $type {
            #[inline(always)]
            fn from(v: $name) -> $type {
                v.bits()
            }
        }

        impl core::convert::TryFrom<$type> for $name {
            type Error = ();
            #[inline(always)]
            fn try_from(v: $type) -> core::result::Result<Self, ()> {
                Self::from_bits(v).ok_or(())
            }
        }
    };
}
