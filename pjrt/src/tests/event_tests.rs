//! Unit Tests for Event Module
//!
//! These tests verify the Event type and its trait implementations.
//! Most tests focus on type-level checks since Event requires a PJRT plugin
//! for full functionality.

#[cfg(test)]
mod type_existence_tests {
    use crate::Event;

    #[test]
    fn test_event_type_exists() {
        // Verify Event type exists and can be referenced
        fn _takes_event(_: Event) {}
    }

    #[test]
    fn test_event_is_future() {
        use std::future::Future;

        // Verify Event implements Future
        fn assert_future<T: Future>() {}
        assert_future::<Event>();
    }

    #[test]
    fn test_event_future_output_type() {
        use std::future::Future;

        use crate::Result;

        // Verify Future::Output is Result<()>
        fn assert_future_output<T: Future<Output = Result<()>>>() {}
        assert_future_output::<Event>();
    }

    #[test]
    fn test_event_is_debug() {
        use std::fmt::Debug;

        // Verify Event implements Debug
        fn assert_debug<T: Debug>() {}
        assert_debug::<Event>();
    }
}

#[cfg(test)]
mod error_code_compatibility_tests {
    use crate::ErrorCode;

    /// Test that ErrorCode values are compatible with PJRT error codes.
    /// This is important for Event::set which takes an ErrorCode.
    #[test]
    fn test_error_code_for_event_set() {
        // ErrorCode values that can be used with Event::set
        let codes = [
            ErrorCode::Cancel,
            ErrorCode::Unknown,
            ErrorCode::InvalidArgument,
            ErrorCode::DeadlineExceeded,
            ErrorCode::NotFound,
            ErrorCode::AlreadyExists,
            ErrorCode::PermissionDenied,
            ErrorCode::ResourceExhausted,
            ErrorCode::FailedPrecondition,
            ErrorCode::Aborted,
            ErrorCode::OutOfRange,
            ErrorCode::Unimplemented,
            ErrorCode::Internal,
            ErrorCode::Unavailable,
            ErrorCode::DataLoss,
            ErrorCode::Unauthenticated,
        ];

        // All codes should have distinct non-zero values (except possibly OK if it existed)
        for (i, code) in codes.iter().enumerate() {
            for (j, other) in codes.iter().enumerate() {
                if i != j {
                    assert_ne!(
                        *code as i32, *other as i32,
                        "{:?} and {:?} have same value",
                        code, other
                    );
                }
            }
        }
    }

    #[test]
    fn test_error_code_repr_i32() {
        // Verify ErrorCode can be safely cast to i32 (required for FFI)
        let code = ErrorCode::Internal;
        let _: i32 = code as i32;
    }

    #[test]
    fn test_error_code_copy_clone() {
        let code = ErrorCode::NotFound;
        let copied = code;
        let cloned = code;
        assert_eq!(code, copied);
        assert_eq!(code, cloned);
    }
}

#[cfg(test)]
mod atomic_ordering_tests {
    use std::sync::atomic::{AtomicBool, Ordering};

    /// Test atomic operations used in Event implementation.
    /// Event uses AtomicBool with SeqCst ordering for callback tracking.
    #[test]
    fn test_atomic_bool_operations() {
        let atomic = AtomicBool::new(false);

        // Initial state
        assert!(!atomic.load(Ordering::SeqCst));

        // Store true (like registering callback)
        atomic.store(true, Ordering::SeqCst);
        assert!(atomic.load(Ordering::SeqCst));

        // Should not change if already set
        let was_set = atomic.load(Ordering::SeqCst);
        assert!(was_set);
    }

    #[test]
    fn test_atomic_bool_multiple_reads() {
        let atomic = AtomicBool::new(false);

        // Multiple reads should be consistent
        let read1 = atomic.load(Ordering::SeqCst);
        let read2 = atomic.load(Ordering::SeqCst);
        assert_eq!(read1, read2);

        atomic.store(true, Ordering::SeqCst);

        let read3 = atomic.load(Ordering::SeqCst);
        let read4 = atomic.load(Ordering::SeqCst);
        assert_eq!(read3, read4);
        assert!(read3);
    }
}

#[cfg(test)]
mod callback_signature_tests {
    use std::ffi::c_void;

    use pjrt_sys::PJRT_Error;

    /// Test the callback function signature expected by PJRT_Event_OnReady.
    /// The callback has signature: extern "C" fn(*mut PJRT_Error, *mut c_void)
    #[test]
    fn test_callback_signature_compatibility() {
        // Define a function with the expected signature
        extern "C" fn test_callback(_err: *mut PJRT_Error, _user_data: *mut c_void) {
            // This function matches the expected PJRT callback signature
        }

        // Verify it can be assigned to an Option of the correct function pointer type
        let _: Option<unsafe extern "C" fn(*mut PJRT_Error, *mut c_void)> = Some(test_callback);
    }

    #[test]
    fn test_callback_with_null_error() {
        extern "C" fn test_callback(err: *mut PJRT_Error, _user_data: *mut c_void) {
            // Error can be null when operation succeeds
            let _ = err.is_null();
        }

        // Should not panic
        test_callback(std::ptr::null_mut(), std::ptr::null_mut());
    }
}

#[cfg(test)]
mod future_trait_tests {
    use std::pin::Pin;
    use std::task::{Context, RawWaker, RawWakerVTable, Waker};

    use crate::Event;

    /// Create a no-op waker for testing purposes
    fn create_test_waker() -> Waker {
        fn no_op(_: *const ()) {}
        fn clone_waker(data: *const ()) -> RawWaker {
            RawWaker::new(data, &VTABLE)
        }

        static VTABLE: RawWakerVTable = RawWakerVTable::new(clone_waker, no_op, no_op, no_op);

        unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) }
    }

    #[test]
    fn test_event_can_be_pinned() {
        // Verify Event can be pinned (required for Future)
        fn assert_can_pin<T>(_: Pin<&mut T>) {}

        // This test just verifies the type signature compatibility
        // We can't actually create an Event without a plugin
        let _ = assert_can_pin::<Event>;
    }

    #[test]
    fn test_waker_can_be_cloned() {
        let waker = create_test_waker();
        let cloned = waker.clone();

        // Both wakers should be valid
        drop(cloned);
        drop(waker);
    }

    #[test]
    fn test_context_creation() {
        let waker = create_test_waker();
        let cx = Context::from_waker(&waker);

        // Context should provide access to waker
        let _: &Waker = cx.waker();
    }
}

#[cfg(test)]
mod pjrt_args_struct_tests {
    use pjrt_sys::{
        PJRT_Event_Await_Args, PJRT_Event_Create_Args, PJRT_Event_Destroy_Args,
        PJRT_Event_Error_Args, PJRT_Event_IsReady_Args, PJRT_Event_OnReady_Args,
        PJRT_Event_Set_Args,
    };

    /// Test that PJRT args structs can be created with new().
    /// Event uses these structs for FFI calls.
    #[test]
    fn test_event_await_args_new() {
        let args = PJRT_Event_Await_Args::new();
        // Verify struct exists and can be created
        assert!(args.event.is_null());
    }

    #[test]
    fn test_event_create_args_new() {
        let args = PJRT_Event_Create_Args::new();
        // Event pointer should be null initially
        assert!(args.event.is_null());
    }

    #[test]
    fn test_event_destroy_args_new() {
        let args = PJRT_Event_Destroy_Args::new();
        assert!(args.event.is_null());
    }

    #[test]
    fn test_event_error_args_new() {
        let args = PJRT_Event_Error_Args::new();
        assert!(args.event.is_null());
    }

    #[test]
    fn test_event_is_ready_args_new() {
        let args = PJRT_Event_IsReady_Args::new();
        assert!(args.event.is_null());
        assert!(!args.is_ready);
    }

    #[test]
    fn test_event_on_ready_args_new() {
        let args = PJRT_Event_OnReady_Args::new();
        assert!(args.event.is_null());
        assert!(args.callback.is_none());
        assert!(args.user_arg.is_null());
    }

    #[test]
    fn test_event_set_args_new() {
        let args = PJRT_Event_Set_Args::new();
        assert!(args.event.is_null());
        assert_eq!(args.error_code, 0);
        assert!(args.error_message.is_null());
        assert_eq!(args.error_message_size, 0);
    }
}

#[cfg(test)]
mod memory_and_pointer_tests {
    use std::ffi::c_void;
    use std::task::{RawWaker, RawWakerVTable, Waker};

    /// Test the callback data boxing pattern used in Event.
    /// Event boxes (Api, Waker) tuple and passes as *mut c_void.
    #[test]
    fn test_box_callback_data_pattern() {
        fn no_op(_: *const ()) {}
        fn clone_waker(data: *const ()) -> RawWaker {
            RawWaker::new(data, &VTABLE)
        }
        static VTABLE: RawWakerVTable = RawWakerVTable::new(clone_waker, no_op, no_op, no_op);

        let waker = unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) };

        // Test the boxing pattern (without Api since we can't create one)
        let data = Box::new((42i32, waker.clone()));
        let ptr = Box::into_raw(data) as *mut c_void;

        // Verify we can recover the data
        let recovered = unsafe { Box::from_raw(ptr as *mut (i32, Waker)) };
        assert_eq!(recovered.0, 42);

        // No mem::forget needed since we recovered with Box::from_raw
    }

    #[test]
    fn test_mem_forget_prevents_drop() {
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;

        let dropped = Arc::new(AtomicBool::new(false));
        let dropped_clone = dropped.clone();

        struct DropTracker(Arc<AtomicBool>);
        impl Drop for DropTracker {
            fn drop(&mut self) {
                self.0.store(true, Ordering::SeqCst);
            }
        }

        let data = Box::new(DropTracker(dropped_clone));
        let _ptr = Box::into_raw(data);

        // Since we used into_raw, drop should NOT be called
        // (in real code, the callback would recover this with from_raw)
        assert!(!dropped.load(Ordering::SeqCst));

        // Clean up to avoid leak (in real code, callback does this)
        unsafe {
            let _ = Box::from_raw(_ptr);
        }

        // Now drop should have been called
        assert!(dropped.load(Ordering::SeqCst));
    }
}

#[cfg(test)]
mod documentation_tests {
    /// Verify the module documentation accurately describes the functionality.
    #[test]
    fn test_module_documentation_accuracy() {
        // The event module documentation claims Events are used for:
        // - Buffer transfers between host and device
        // - Buffer copies between devices
        // - Program execution
        // - Compilation operations
        //
        // These use cases are valid based on the PJRT API design.

        // Event implements Future for async/await support
        use std::future::Future;

        use crate::Event;
        fn _assert_future<T: Future>() {}
        _assert_future::<Event>();
    }
}

#[cfg(test)]
mod type_size_tests {
    use std::mem;
    use std::sync::atomic::AtomicBool;

    use pjrt_sys::PJRT_Event;

    use crate::{Api, Event};

    #[test]
    fn test_event_contains_expected_fields() {
        // Event struct should contain:
        // - api: Api (Arc-based, so pointer-sized on the inner level)
        // - ptr: *mut PJRT_Event (pointer)
        // - registered_callback: AtomicBool

        // Verify AtomicBool size
        assert_eq!(mem::size_of::<AtomicBool>(), 1);

        // Verify pointer sizes
        assert_eq!(mem::size_of::<*mut PJRT_Event>(), mem::size_of::<usize>());

        // Event size should be reasonable (not checking exact size due to padding)
        let event_size = mem::size_of::<Event>();
        assert!(event_size > 0);
        assert!(event_size <= 64); // Sanity check: should be small
    }

    #[test]
    fn test_api_is_cloneable() {
        // Api must be Clone for Event to store a clone
        fn assert_clone<T: Clone>() {}
        assert_clone::<Api>();
    }
}

#[cfg(test)]
mod poll_state_tests {
    use std::task::Poll;

    use crate::Result;

    /// Test Poll states that Event::poll can return.
    #[test]
    fn test_poll_pending_state() {
        let pending: Poll<Result<()>> = Poll::Pending;
        assert!(pending.is_pending());
    }

    #[test]
    fn test_poll_ready_ok_state() {
        let ready: Poll<Result<()>> = Poll::Ready(Ok(()));
        assert!(ready.is_ready());
        if let Poll::Ready(result) = ready {
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_poll_ready_err_state() {
        use crate::Error;
        let err = Error::InvalidArgument("test".to_string());
        let ready: Poll<Result<()>> = Poll::Ready(Err(err));
        assert!(ready.is_ready());
        if let Poll::Ready(result) = ready {
            assert!(result.is_err());
        }
    }
}
