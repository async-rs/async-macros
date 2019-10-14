/// Declares task-local values.
///
/// The macro wraps any number of static declarations and makes them task-local. Attributes and
/// visibility modifiers are allowed.
///
/// Each declared value is of the accessor type [`LocalKey`].
///
/// [`LocalKey`]: task/struct.LocalKey.html
///
/// # Examples
///
/// ```ignore
/// #
/// use std::cell::Cell;
///
/// use async_std::task;
/// use async_std::prelude::*;
///
/// task_local! {
///     static VAL: Cell<u32> = Cell::new(5);
/// }
///
/// task::block_on(async {
///     let v = VAL.with(|c| c.get());
///     assert_eq!(v, 5);
/// });
/// ```
#[macro_export]
macro_rules! task_local {
    () => ();

    ($(#[$attr:meta])* $vis:vis static $name:ident: $t:ty = $init:expr) => (
        $(#[$attr])* $vis static $name: ::async_std::task::LocalKey<$t> = {
            #[inline]
            fn __init() -> $t {
                $init
            }

            ::async_std::task::LocalKey {
                __init,
                __key: ::std::sync::atomic::AtomicUsize::new(0),
            }
        };
    );

    ($(#[$attr:meta])* $vis:vis static $name:ident: $t:ty = $init:expr; $($rest:tt)*) => (
        $crate::task_local!($(#[$attr])* $vis static $name: $t = $init);
        $crate::task_local!($($rest)*);
    );
}
