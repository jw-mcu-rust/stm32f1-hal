#![allow(unused_macros)]
#![allow(unused_imports)]

macro_rules! impl_wrap_func {
    (fn $func:ident(&$($life:lifetime)? self $(, $arg:ident: $arg_ty:ty)*) $(-> $ret:ty)?; $($rest_func:tt)*) => {
        #[inline(always)]
        fn $func(&$($life)? self $(, $arg: $arg_ty),*) $(-> $ret)? {
            self.$func($($arg),*)
        }
        impl_wrap_func!($($rest_func)*);
    };
    (fn $func:ident(&$($life:lifetime)? mut self $(, $arg:ident: $arg_ty:ty)*) $(-> $ret:ty)?; $($rest_func:tt)*) => {
        #[inline(always)]
        fn $func(&$($life)? mut self $(, $arg: $arg_ty),*) $(-> $ret)? {
             self.$func($($arg),*)
        }
        impl_wrap_func!($($rest_func)*);
    };
    () => {};
}
pub(crate) use impl_wrap_func;

macro_rules! impl_wrap_deref_func {
    (fn $func:ident(&$($life:lifetime)? self $(, $arg:ident: $arg_ty:ty)*) $(-> $ret:ty)?; $($rest_func:tt)*) => {
        #[inline(always)]
        fn $func(&$($life)? self $(, $arg: $arg_ty),*) $(-> $ret)? {
            self.deref().$func($($arg),*)
        }
        impl_wrap_deref_func!($($rest_func)*);
    };
    (fn $func:ident(&$($life:lifetime)? mut self $(, $arg:ident: $arg_ty:ty)*) $(-> $ret:ty)?; $($rest_func:tt)*) => {
        #[inline(always)]
        fn $func(&$($life)? mut self $(, $arg: $arg_ty),*) $(-> $ret)? {
            self.deref().$func($($arg),*)
        }
        impl_wrap_deref_func!($($rest_func)*);
    };
    () => {};
}
pub(crate) use impl_wrap_deref_func;

macro_rules! impl_wrap_trait {
    (
        $vis:vis trait $trait_name:ident {
            $($func:tt)*
        }
        $type:ty
    ) => {
        impl $trait_name for $type {
            impl_wrap_func!($($func)*);
        }
    };
}
pub(crate) use impl_wrap_trait;

macro_rules! impl_wrap_deref_trait {
    (
        $vis:vis trait $trait_name:ident {
            $($func:tt)*
        }
        $type:ty
    ) => {
        impl $trait_name for $type {
            impl_wrap_deref_func!($($func)*);
        }
    };
}
pub(crate) use impl_wrap_deref_trait;

macro_rules! wrap_trait {
    (
        ($type:ty, $($rest_type:ty,)*),
        $($trait_body:tt)+
    ) => {
        wrap_trait!(($($rest_type,)*), $($trait_body)+);
        impl_wrap_trait!($($trait_body)+ $type);
    };

    ((), $($trait_body:tt)+) => {
        // Declare the trait
        $($trait_body)+
    };
}
pub(crate) use wrap_trait;

macro_rules! wrap_deref_trait {
    (
        ($type:ty, $($rest_type:ty,)*),
        $($trait_body:tt)+
    ) => {
        wrap_deref_trait!(($($rest_type,)*), $($trait_body)+);
        impl_wrap_deref_trait!($($trait_body)+ $type);
    };

    ((), $($trait_body:tt)+) => {
        // Declare the trait
        $($trait_body)+
    };
}
pub(crate) use wrap_deref_trait;
