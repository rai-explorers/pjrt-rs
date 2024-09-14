use pjrt_sys::{PJRT_ExecuteContext, PJRT_ExecuteContext_Destroy_Args};

use crate::Api;

pub struct ExecuteContext {
    api: Api,
    pub(crate) ptr: *mut PJRT_ExecuteContext,
}

impl Drop for ExecuteContext {
    fn drop(&mut self) {
        let mut args = PJRT_ExecuteContext_Destroy_Args::new();
        args.context = self.ptr;
        self.api
            .PJRT_ExecuteContext_Destroy(args)
            .expect("PJRT_ExecuteContext_Destroy");
    }
}

impl ExecuteContext {
    pub fn new(api: &Api, ptr: *mut PJRT_ExecuteContext) -> Self {
        assert!(!ptr.is_null());
        Self {
            api: api.clone(),
            ptr,
        }
    }

    pub fn api(&self) -> &Api {
        &self.api
    }
}
