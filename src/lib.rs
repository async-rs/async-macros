//! Macros for async-std.
//!
//! # Examples
//!
//! ```
//! #![feature(async_await)]
//! # futures::executor::block_on(async {
//! use async_macros::join;
//! use futures::future;
//!
//! let a = future::ready(1u8);
//! let b = future::ready(2u8);
//!
//! assert_eq!(join!(a, b).await, (1, 2));
//! # });
//! ```

#![forbid(future_incompatible, rust_2018_idioms)]
#![deny(missing_debug_implementations, nonstandard_style)]
#![warn(missing_docs, unreachable_pub)]
#![cfg_attr(test, deny(warnings))]

mod join;
mod maybe_done;
mod poll_fn;
mod ready;
mod select;
mod try_join;

pub use maybe_done::{maybe_done, MaybeDone};

/// Helper re-exports for use in macros.
pub mod utils {
    pub use super::poll_fn::poll_fn;
    pub use core::{future, pin, result, task};
}
