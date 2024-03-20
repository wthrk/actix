//! Actix is an actor framework.
//!
//! [Actors](Actor) are objects which encapsulate state and behavior, they communicate exclusively
//! by exchanging messages. Actix actors are implemented on top of [Tokio](https://tokio.rs).
//!
//! Multiple actors can run in same thread. Actors can run in multiple threads using the [`Arbiter`]
//! API. Actors exchange typed messages.
//!
//! ## Features
//!
//! - Async or sync actors
//! - Actor communication in a local/thread context
//! - Using Futures for asynchronous message handling
//! - Actor supervision
//! - Typed messages (no [`Any`](std::any::Any) type) and generic messages are allowed
//! - Runs on stable Rust 1.68+
//!
//! ## Other Documentation
//!
//! - [User Guide](https://actix.rs/docs/actix)
//! - [Community Chat on Discord](https://discord.gg/NWpN5mmg3x)

#![deny(rust_2018_idioms, nonstandard_style, future_incompatible)]
#![doc(html_logo_url = "https://actix.rs/img/logo.png")]
#![doc(html_favicon_url = "https://actix.rs/favicon.ico")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

#[cfg(doctest)]
doc_comment::doctest!("../README.md");

mod actor;
mod address;
mod context;
mod context_impl;
mod context_items;
mod handler;
mod mailbox;
mod stream;
mod supervisor;

pub mod actors;
pub mod clock;
pub mod fut;
pub mod registry;
pub mod sync;
pub mod utils;

//#[cfg(all(feature = "rt-tokio", not(feature = "rt-wasm-bindgen")))]
pub mod io;

#[cfg(feature = "macros")]
pub use actix_derive::{main, test, Message, MessageResponse};
pub use actix_rt::{spawn, Arbiter, ArbiterHandle, System, SystemRunner};

#[doc(hidden)]
pub mod __private {
    #[cfg(feature = "macros")]
    pub use actix_macros::{main, test};
}

#[doc(hidden)]
pub use crate::context::ContextFutureSpawner;
pub use crate::{
    actor::{Actor, ActorContext, ActorState, AsyncContext, Running, SpawnHandle, Supervised},
    address::{Addr, MailboxError, Recipient, WeakAddr, WeakRecipient},
    context::Context,
    fut::{
        ActorFuture, ActorFutureExt, ActorStream, ActorStreamExt, ActorTryFuture,
        ActorTryFutureExt, WrapFuture, WrapStream,
    },
    handler::{
        ActorResponse, AtomicResponse, Handler, Message, MessageResult, Response,
        ResponseActFuture, ResponseFuture,
    },
    registry::{ArbiterService, Registry, SystemRegistry, SystemService},
    stream::StreamHandler,
    supervisor::Supervisor,
    sync::{SyncArbiter, SyncContext},
};

pub mod prelude {
    //! The `actix` prelude.
    //!
    //! The purpose of this module is to alleviate imports of many common actix
    //! traits by adding a glob import to the top of actix heavy modules:
    //!
    //! ```
    //! # #![allow(unused_imports)]
    //! use actix::prelude::*;
    //! ```

    #[doc(hidden)]
    #[cfg(feature = "macros")]
    pub use actix_derive::{Message, MessageResponse};
    pub use actix_rt::{Arbiter, ArbiterHandle, System, SystemRunner};
    pub use futures_core::stream::Stream;

    #[allow(deprecated)]
    pub use crate::utils::Condition;
    pub use crate::{
        actor::{Actor, ActorContext, ActorState, AsyncContext, Running, SpawnHandle, Supervised},
        actors,
        address::{Addr, MailboxError, Recipient, RecipientRequest, Request, SendError},
        context::{Context, ContextFutureSpawner},
        dev, fut,
        fut::{
            ActorFuture, ActorFutureExt, ActorStream, ActorStreamExt, ActorTryFuture,
            ActorTryFutureExt, WrapFuture, WrapStream,
        },
        handler::{
            ActorResponse, AtomicResponse, Handler, Message, MessageResult, Response,
            ResponseActFuture, ResponseFuture,
        },
        registry::{ArbiterService, SystemService},
        stream::StreamHandler,
        supervisor::Supervisor,
        sync::{SyncArbiter, SyncContext},
        utils::{IntervalFunc, TimerFunc},
    };
}

pub mod dev {
    //! The `actix` prelude for library developers.
    //!
    //! The purpose of this module is to alleviate imports of many common actix
    //! traits by adding a glob import to the top of actix heavy modules:
    //!
    //! ```
    //! # #![allow(unused_imports)]
    //! use actix::dev::*;
    //! ```

    pub use crate::{
        address::{Envelope, EnvelopeProxy, RecipientRequest, Request, ToEnvelope},
        prelude::*,
    };
    pub mod channel {
        pub use crate::address::channel::{channel, AddressReceiver, AddressSender};
    }
    pub use crate::{
        context_impl::{AsyncContextParts, ContextFut, ContextParts},
        handler::{MessageResponse, OneshotSender},
        mailbox::Mailbox,
        registry::{Registry, SystemRegistry},
    };
}

/// Starts the system and executes the supplied future.
///
/// This function does the following:
///
/// * Creates and starts the actix system with default configuration.
/// * Spawns the given future onto the current arbiter.
/// * Blocks the current thread until the system shuts down.
///
/// The `run` function returns when the `System::current().stop()`
/// method gets called.
///
/// # Examples
///
/// ```
/// use std::time::{Duration, Instant};
/// use actix_rt::time::sleep;
///
/// fn main() {
///   actix::run(async move {
///       sleep(Duration::from_millis(100)).await;
///       actix::System::current().stop();
///   });
/// }
/// ```
///
/// # Panics
///
/// This function panics if the actix system is already running.
#[cfg(all(feature = "rt-tokio", not(feature = "rt-wasm-bindgen")))]
#[allow(clippy::unit_arg, clippy::needless_doctest_main)]
pub fn run<R>(f: R) -> std::io::Result<()>
where
    R: std::future::Future<Output = ()> + 'static,
{
    Ok(actix_rt::System::new().block_on(f))
}

#[cfg(all(feature = "rt-tokio", not(feature = "rt-wasm-bindgen")))]
pub use par1king_lot::{Mutex, RawMutex};

/// Mutex for wasm-single threaded applications.
/// 要はロックを掛ける必要がなくて実質 Refcell と同じ
//#[cfg(all(feature = "rt-wasm-bindgen", not(feature = "rt-tokio")))]
mod wasm {
    use parking_lot::lock_api::{Mutex as Mutex_, RawMutex as RawMutex_};
    use std::cell::RefCell;

    pub struct RawMutex {
        inner: RefCell<bool>,
    }

    unsafe impl RawMutex_ for RawMutex {
        const INIT: Self = Self {
            inner: RefCell::new(false),
        };

        type GuardMarker = parking_lot::lock_api::GuardSend;

        fn lock(&self) {
            let mut guard = self.inner.borrow_mut();
            if *guard {
                panic!("Cannot lock a mutex recursively");
            }
            *guard = true;
        }

        fn try_lock(&self) -> bool {
            let mut guard = self.inner.borrow_mut();
            if *guard {
                false
            } else {
                *guard = true;
                true
            }
        }

        unsafe fn unlock(&self) {
            *self.inner.borrow_mut() = false;
        }
    }

    unsafe impl Send for RawMutex {}
    unsafe impl Sync for RawMutex {}

    pub type Mutex<T> = Mutex_<RawMutex, T>;
}

#[cfg(all(feature = "rt-wasm-bindgen", not(feature = "rt-tokio")))]
pub use wasm::{Mutex, RawMutex};
