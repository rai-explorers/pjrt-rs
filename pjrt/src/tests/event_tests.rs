//! Unit Tests for Event Module
//!
//! These tests verify the Event type's trait implementations and FFI
//! compatibility. Full functional tests require a PJRT plugin — see
//! the `examples/` directory for integration tests.

#[cfg(test)]
mod trait_tests {
    use std::future::Future;

    use crate::{Event, Result};

    #[test]
    fn test_event_is_future() {
        fn assert_future<T: Future>() {}
        assert_future::<Event>();
    }

    #[test]
    fn test_event_future_output_type() {
        fn assert_future_output<T: Future<Output = Result<()>>>() {}
        assert_future_output::<Event>();
    }
}

#[cfg(test)]
mod error_code_tests {
    use crate::ErrorCode;

    #[test]
    fn test_error_code_values_are_distinct() {
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
}

#[cfg(test)]
mod callback_signature_tests {
    use std::ffi::c_void;

    use pjrt_sys::PJRT_Error;

    #[test]
    fn test_callback_signature_compatibility() {
        extern "C" fn test_callback(_err: *mut PJRT_Error, _user_data: *mut c_void) {}
        let _: Option<unsafe extern "C" fn(*mut PJRT_Error, *mut c_void)> = Some(test_callback);
    }

    #[test]
    fn test_callback_with_null_error() {
        extern "C" fn test_callback(err: *mut PJRT_Error, _user_data: *mut c_void) {
            let _ = err.is_null();
        }
        test_callback(std::ptr::null_mut(), std::ptr::null_mut());
    }
}

#[cfg(test)]
mod pjrt_args_struct_tests {
    use pjrt_sys::{
        PJRT_Event_Await_Args, PJRT_Event_Create_Args, PJRT_Event_Destroy_Args,
        PJRT_Event_Error_Args, PJRT_Event_IsReady_Args, PJRT_Event_OnReady_Args,
        PJRT_Event_Set_Args,
    };

    #[test]
    fn test_event_await_args_new() {
        let args = PJRT_Event_Await_Args::new();
        assert!(args.event.is_null());
    }

    #[test]
    fn test_event_create_args_new() {
        let args = PJRT_Event_Create_Args::new();
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
mod type_size_tests {
    use std::mem;
    use std::sync::atomic::AtomicBool;

    use pjrt_sys::PJRT_Event;

    use crate::Event;

    #[test]
    fn test_event_contains_expected_fields() {
        assert_eq!(mem::size_of::<AtomicBool>(), 1);
        assert_eq!(mem::size_of::<*mut PJRT_Event>(), mem::size_of::<usize>());
        let event_size = mem::size_of::<Event>();
        assert!(event_size > 0);
        assert!(event_size <= 64);
    }
}
