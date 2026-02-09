//! PJRT Event Handling
//!
//! This module provides the `Event` struct for managing asynchronous operations in PJRT.
//! Events are used to track the completion of asynchronous device operations such as:
//!
//! - Buffer transfers between host and device
//! - Buffer copies between devices
//! - Program execution
//! - Compilation operations
//!
//! The `Event` struct implements Rust's `Future` trait, allowing it to be used with
//! async/await syntax for convenient asynchronous programming.

use std::ffi::c_void;
use std::future::Future;
use std::mem;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Context, Poll, Waker};

use pjrt_sys::{
    PJRT_Error, PJRT_Error_Code, PJRT_Error_Destroy_Args, PJRT_Event, PJRT_Event_Await_Args,
    PJRT_Event_Create_Args, PJRT_Event_Destroy_Args, PJRT_Event_Error_Args,
    PJRT_Event_IsReady_Args, PJRT_Event_OnReady_Args, PJRT_Event_Set_Args,
};

use crate::{Api, ErrorCode, Result};

extern "C" fn on_ready_callback(err: *mut PJRT_Error, cb_data: *mut c_void) {
    let (api, waker) = unsafe { *Box::from_raw(cb_data as *mut (Api, Waker)) };
    let mut args = PJRT_Error_Destroy_Args::new();
    args.error = err;
    api.PJRT_Error_Destroy(&mut args)
        .expect("PJRT_Error_Destroy");
    waker.wake();
}

/// An asynchronous event that signals completion of a PJRT operation.
///
/// Events are used throughout PJRT to track the completion of asynchronous
/// operations such as buffer transfers, device copies, and executions.
///
/// This struct implements `Future`, allowing it to be awaited in async code:
///
/// # Example
///
/// ```rust,ignore
/// let event = buffer.ready_event()?;
/// event.await?; // Wait for the buffer to be ready
/// ```
/// # Thread Safety
///
/// `Event` is `!Send + !Sync` due to the raw `*mut PJRT_Event` pointer.
/// Although `Api` and `AtomicBool` are both `Send + Sync`, the raw pointer
/// prevents auto-derivation. Events must be awaited or waited on the same
/// thread that created them.
pub struct Event {
    api: Api,
    ptr: *mut PJRT_Event,
    registered_callback: AtomicBool,
}

impl Drop for Event {
    fn drop(&mut self) {
        let mut args = PJRT_Event_Destroy_Args::new();
        args.event = self.ptr;
        self.api
            .PJRT_Event_Destroy(args)
            .expect("PJRT_Event_Destroy");
    }
}

impl std::fmt::Debug for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let is_ready = self.is_ready().unwrap_or(false);
        f.debug_struct("Event")
            .field("is_ready", &is_ready)
            .field(
                "callback_registered",
                &self.registered_callback.load(Ordering::SeqCst),
            )
            .finish()
    }
}

impl Event {
    pub(crate) fn wrap(api: &Api, ptr: *mut PJRT_Event) -> Self {
        assert!(!ptr.is_null());
        Self {
            api: api.clone(),
            ptr,
            registered_callback: AtomicBool::new(false),
        }
    }

    pub fn api(&self) -> &Api {
        &self.api
    }

    fn is_ready(&self) -> Result<bool> {
        let mut args = PJRT_Event_IsReady_Args::new();
        args.event = self.ptr;
        let args = self.api.PJRT_Event_IsReady(args)?;
        Ok(args.is_ready)
    }

    fn error(&self) -> Result<()> {
        let mut args = PJRT_Event_Error_Args::new();
        args.event = self.ptr;
        self.api.PJRT_Event_Error(args).map(|_| ())
    }

    fn register_on_ready_callback(&self, waker: &Waker) -> Result<()> {
        let mut cb_data = Box::new((self.api.clone(), waker.clone()));
        let mut args = PJRT_Event_OnReady_Args::new();
        args.event = self.ptr;
        args.user_arg = cb_data.as_mut() as *mut _ as *mut c_void;
        args.callback = Some(on_ready_callback);
        let args = self.api.PJRT_Event_OnReady(args);
        mem::forget(cb_data);
        args.map(|_| self.registered_callback.store(true, Ordering::SeqCst))
    }

    #[must_use = "handle wait result"]
    pub fn wait(self) -> Result<()> {
        if self.is_ready()? {
            return Ok(());
        }
        let mut args = PJRT_Event_Await_Args::new();
        args.event = self.ptr;
        let _ = self.api.PJRT_Event_Await(args)?;
        Ok(())
    }

    /// Creates a new event.
    ///
    /// This creates an event that can be set later to signal completion.
    pub fn create(api: &Api) -> Result<Self> {
        let args = PJRT_Event_Create_Args::new();
        let args = api.PJRT_Event_Create(args)?;
        Ok(Self::wrap(api, args.event))
    }

    /// Sets this event with the given error code and message.
    ///
    /// This marks the event as complete. If error_code is OK, the event
    /// completes successfully. Otherwise, it completes with an error.
    pub fn set(&self, error_code: ErrorCode, error_message: Option<&str>) -> Result<()> {
        let mut args = PJRT_Event_Set_Args::new();
        args.event = self.ptr;
        args.error_code = error_code as PJRT_Error_Code;
        if let Some(msg) = error_message {
            args.error_message = msg.as_ptr() as *const i8;
            args.error_message_size = msg.len();
        }
        self.api.PJRT_Event_Set(args).map(|_| ())
    }
}

impl Future for Event {
    type Output = Result<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.is_ready() {
            Ok(is_ready) => {
                if is_ready {
                    Poll::Ready(self.error())
                } else {
                    if self.registered_callback.load(Ordering::SeqCst) {
                        return Poll::Pending;
                    }
                    match self.register_on_ready_callback(cx.waker()) {
                        Ok(_) => Poll::Pending,
                        Err(err) => Poll::Ready(Err(err)),
                    }
                }
            }
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}
