use std::collections::HashSet;

use pjrt_sys::{
    PJRT_Buffer, PJRT_ExecuteContext, PJRT_ExecuteContext_Destroy_Args, PJRT_ExecuteOptions,
};

use crate::{Api, Buffer, LoadedExecutable, Result};

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
    pub(crate) fn wrap(api: &Api, ptr: *mut PJRT_ExecuteContext) -> Self {
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

pub struct ExecuteOptions {
    launch_id: i32,
    non_donatable_input_indices: Vec<i64>,
    // TODO:
    // send_callbacks
    // recv_callbacks
}

impl ExecuteOptions {
    pub fn new() -> Self {
        Self {
            launch_id: 0,
            non_donatable_input_indices: vec![],
        }
    }

    pub fn launch_id(mut self, launch_id: i32) -> Self {
        self.launch_id = launch_id;
        self
    }

    pub fn non_donatable_input_indices(mut self, indices: impl Into<Vec<i64>>) -> Self {
        self.non_donatable_input_indices = indices.into();
        self
    }
}

impl Default for ExecuteOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> From<&'a ExecuteOptions> for PJRT_ExecuteOptions {
    fn from(v: &'a ExecuteOptions) -> Self {
        let mut options = PJRT_ExecuteOptions::new();
        options.launch_id = v.launch_id;
        options.non_donatable_input_indices = v.non_donatable_input_indices.as_ptr();
        options.num_non_donatable_input_indices = v.non_donatable_input_indices.len();
        options
    }
}

pub struct Execution<'a, T> {
    pub loaded_executable: &'a LoadedExecutable,
    pub inputs: T,
    pub options: ExecuteOptions,
}

impl<'a, T> Execution<'a, T>
where
    T: ExecutionInputs,
{
    pub fn new(loaded_executable: &'a LoadedExecutable, inputs: T) -> Self {
        let options = ExecuteOptions {
            launch_id: 0,
            non_donatable_input_indices: inputs.non_donatable_input_indices(),
        };
        Self {
            loaded_executable,
            inputs,
            options,
        }
    }

    pub fn launch_id(mut self, launch_id: i32) -> Self {
        self.options.launch_id = launch_id;
        self
    }

    pub fn non_donatable_input_indices(mut self, indices: impl Into<Vec<i64>>) -> Self {
        self.options.non_donatable_input_indices = indices.into();
        self
    }

    pub async fn run(self) -> Result<Vec<Vec<Buffer>>> {
        let (events, outputs) = self
            .loaded_executable
            .call_execute(self.inputs, &self.options)?;
        for event in events {
            event.await?;
        }
        Ok(outputs)
    }

    pub fn run_sync(self) -> Result<Vec<Vec<Buffer>>> {
        let (events, outputs) = self
            .loaded_executable
            .call_execute(self.inputs, &self.options)?;
        for event in events {
            event.wait()?;
        }
        Ok(outputs)
    }
}

pub trait ExecutionInputs {
    fn buffer_ptrs(&self) -> Vec<Vec<*mut PJRT_Buffer>>;
    fn non_donatable_input_indices(&self) -> Vec<i64> {
        vec![]
    }
}

impl ExecutionInputs for () {
    fn buffer_ptrs(&self) -> Vec<Vec<*mut PJRT_Buffer>> {
        vec![vec![]]
    }
}

impl ExecutionInputs for Buffer {
    fn buffer_ptrs(&self) -> Vec<Vec<*mut PJRT_Buffer>> {
        vec![vec![self.ptr]]
    }
}

impl<const A: usize> ExecutionInputs for [Buffer; A] {
    fn buffer_ptrs(&self) -> Vec<Vec<*mut PJRT_Buffer>> {
        vec![self.iter().map(|b| b.ptr).collect()]
    }
}

impl<const D: usize, const A: usize> ExecutionInputs for [[Buffer; A]; D] {
    fn buffer_ptrs(&self) -> Vec<Vec<*mut PJRT_Buffer>> {
        let mut buffer_refs = Vec::with_capacity(D);
        for array in self.iter() {
            buffer_refs.push(array.iter().map(|b| b.ptr).collect());
        }
        buffer_refs
    }
}

impl ExecutionInputs for Vec<Buffer> {
    fn buffer_ptrs(&self) -> Vec<Vec<*mut PJRT_Buffer>> {
        vec![self.iter().map(|b| b.ptr).collect()]
    }
}

impl ExecutionInputs for Vec<Vec<Buffer>> {
    fn buffer_ptrs(&self) -> Vec<Vec<*mut PJRT_Buffer>> {
        let inner_size = self.iter().fold(HashSet::new(), |mut set, buffers| {
            set.insert(buffers.len());
            set
        });
        assert_eq!(
            inner_size.len(),
            1,
            "all inner vectors must have the same length"
        );
        self.iter()
            .map(|buffers| buffers.iter().map(|b| b.ptr).collect())
            .collect()
    }
}
