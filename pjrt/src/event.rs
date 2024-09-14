use std::ffi::c_void;
use std::future::Future;
use std::mem;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};

use pjrt_sys::{
    PJRT_Error, PJRT_Error_Destroy_Args, PJRT_Event, PJRT_Event_Await_Args,
    PJRT_Event_Destroy_Args, PJRT_Event_Error_Args, PJRT_Event_IsReady_Args,
    PJRT_Event_OnReady_Args,
};

use crate::{Api, Result};

extern "C" fn on_ready_callback(err: *mut PJRT_Error, cb_data: *mut c_void) {
    let cb_data = cb_data as *mut Option<(Api, Waker)>;
    let cb_data: &mut Option<(Api, Waker)> = unsafe { mem::transmute(cb_data) };
    if let Some((api, waker)) = cb_data.take() {
        let mut args = PJRT_Error_Destroy_Args::new();
        args.error = err;
        api.PJRT_Error_Destroy(&mut args)
            .expect("PJRT_Error_Destroy");
        waker.wake();
    }
}

pub struct Event {
    api: Api,
    ptr: *mut PJRT_Event,

    // Note: as we need to free error in the callback, we need to keep a reference to the api and waker
    // if PJRT_Event_OnReady_Args can configure not to return error, we can drop the cb_data
    // and directly use the pass waker to callback
    cb_data: Option<(Api, Waker)>,
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

impl Event {
    pub fn new(api: &Api, ptr: *mut PJRT_Event) -> Self {
        assert!(!ptr.is_null());
        Self {
            api: api.clone(),
            ptr,
            cb_data: None,
        }
    }

    // TODO: fix sometimes hand in macos, don't know why
    #[must_use = "handle wait result"]
    pub fn wait(self) -> Result<()> {
        let mut args = PJRT_Event_IsReady_Args::new();
        args.event = self.ptr;
        args = self.api.PJRT_Event_IsReady(args)?;
        if args.is_ready {
            return Ok(());
        }
        let mut args = PJRT_Event_Await_Args::new();
        args.event = self.ptr;
        let _ = self.api.PJRT_Event_Await(args)?;
        Ok(())
    }
}

impl Future for Event {
    type Output = Result<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut args = PJRT_Event_IsReady_Args::new();
        args.event = self.ptr;
        let args = self.api.PJRT_Event_IsReady(args);
        match args {
            Ok(args) => {
                if args.is_ready {
                    let mut args = PJRT_Event_Error_Args::new();
                    args.event = self.ptr;
                    let args = self.api.PJRT_Event_Error(args);
                    match args {
                        Ok(_) => Poll::Ready(Ok(())),
                        Err(err) => Poll::Ready(Err(err)),
                    }
                } else {
                    // TODO: it mgiht not be atomic, use AtomicBool also in cb_data?
                    if self.cb_data.is_none() {
                        let event = self.get_mut();
                        event.cb_data = Some((event.api.clone(), cx.waker().clone()));
                        let mut args = PJRT_Event_OnReady_Args::new();
                        args.event = event.ptr;
                        args.user_arg = &mut event.cb_data as *mut _ as *mut c_void;
                        args.callback = Some(on_ready_callback);
                        let args = event.api.PJRT_Event_OnReady(args);
                        match args {
                            Ok(_) => Poll::Pending,
                            Err(err) => Poll::Ready(Err(err)),
                        }
                    } else {
                        Poll::Pending
                    }
                }
            }
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}
