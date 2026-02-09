//! PJRT Loaded Executable
//!
//! This module provides the `LoadedExecutable` struct for managing executables
//! that have been loaded onto devices and are ready for execution.
//!
//! A `LoadedExecutable` represents the final stage in the compilation pipeline:
//! 1. Compile a `Program` to an `Executable`
//! 2. Load the `Executable` to create a `LoadedExecutable`
//! 3. Execute the `LoadedExecutable` with input buffers
//!
//! The loaded executable can be executed multiple times with different inputs,
//! making it efficient for inference and training loops.

use std::mem::MaybeUninit;
use std::slice;

use bon::bon;
use pjrt_sys::{
    PJRT_Buffer, PJRT_Event, PJRT_ExecuteOptions, PJRT_LoadedExecutable,
    PJRT_LoadedExecutable_AddressableDevices_Args, PJRT_LoadedExecutable_Delete_Args,
    PJRT_LoadedExecutable_Destroy_Args, PJRT_LoadedExecutable_Execute_Args,
    PJRT_LoadedExecutable_GetExecutable_Args, PJRT_LoadedExecutable_IsDeleted_Args,
};

use crate::execute::ExecuteOptionsRaw;
use crate::{
    event, utils, Buffer, Client, CompileOptions, CompileToLoadedExecutable, Device, Event,
    Executable, ExecuteOptions, Execution, ExecutionInputs, Result,
};

/// An executable loaded onto devices and ready for execution.
///
/// A `LoadedExecutable` represents a compiled program that has been loaded
/// onto one or more devices and is ready to execute. Unlike `Executable`,
/// which is device-agnostic, a `LoadedExecutable` is bound to specific devices.
///
/// The loaded executable can be executed multiple times with different inputs,
/// making it efficient for repeated inference or training steps.
///
/// # Example
///
/// ```rust,ignore
/// // Compile and load a program
/// let loaded_exec = client.compile(&program, options)?;
///
/// // Execute with inputs
/// let outputs = loaded_exec.execute(&input_buffers, &options).await?;
///
/// // Execute again with different inputs
/// let more_outputs = loaded_exec.execute(&other_inputs, &options).await?;
/// ```
/// # Thread Safety
///
/// `LoadedExecutable` is `!Send + !Sync` because it holds a [`Client`]
/// reference (which uses `Rc` internally). Compile once per thread, or
/// compile on the client's thread and execute there.
pub struct LoadedExecutable {
    client: Client,
    pub(crate) ptr: *mut PJRT_LoadedExecutable,
}

impl Drop for LoadedExecutable {
    fn drop(&mut self) {
        let mut args = PJRT_LoadedExecutable_Destroy_Args::new();
        args.executable = self.ptr;
        self.client
            .api()
            .PJRT_LoadedExecutable_Destroy(args)
            .expect("PJRT_LoadedExecutable_Destroy");
    }
}

impl std::fmt::Debug for LoadedExecutable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let executable = self.executable();
        f.debug_struct("LoadedExecutable")
            .field("name", &executable.name())
            .field("is_deleted", &self.is_deleted())
            .field("num_addressable_devices", &self.addressable_devices().len())
            .finish()
    }
}

#[bon]
impl LoadedExecutable {
    pub(crate) fn wrap(client: &Client, ptr: *mut PJRT_LoadedExecutable) -> Self {
        assert!(!ptr.is_null());
        Self {
            client: client.clone(),
            ptr,
        }
    }

    #[builder(finish_fn = build)]
    pub fn builder<T>(
        #[builder(start_fn)] client: &Client,
        #[builder(start_fn)] program: &T,
        #[builder(default)] options: CompileOptions,
    ) -> Result<Self>
    where
        Client: CompileToLoadedExecutable<T>,
    {
        client.compile(program, options)
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn executable(&self) -> Executable {
        let mut args = PJRT_LoadedExecutable_GetExecutable_Args::new();
        args.loaded_executable = self.ptr;
        args = self
            .client
            .api()
            .PJRT_LoadedExecutable_GetExecutable(args)
            .expect("PJRT_LoadedExecutable_GetExecutable");
        Executable::wrap(self.client.api(), args.executable)
    }

    pub fn addressable_devices(&self) -> Vec<Device> {
        let mut args = PJRT_LoadedExecutable_AddressableDevices_Args::new();
        args.executable = self.ptr;
        args = self
            .client
            .api()
            .PJRT_LoadedExecutable_AddressableDevices(args)
            .expect("PJRT_LoadedExecutable_AddressableDevices");
        let raw_devices = unsafe {
            slice::from_raw_parts(args.addressable_devices, args.num_addressable_devices)
        };
        raw_devices
            .iter()
            .cloned()
            .map(|d| Device::wrap(&self.client, d))
            .collect()
    }

    pub fn delete(self) {
        let mut args = PJRT_LoadedExecutable_Delete_Args::new();
        args.executable = self.ptr;
        self.client
            .api()
            .PJRT_LoadedExecutable_Delete(args)
            .expect("PJRT_LoadedExecutable_Delete");
    }

    pub fn is_deleted(&self) -> bool {
        let mut args = PJRT_LoadedExecutable_IsDeleted_Args::new();
        args.executable = self.ptr;
        args = self
            .client
            .api()
            .PJRT_LoadedExecutable_IsDeleted(args)
            .expect("PJRT_LoadedExecutable_IsDeleted");
        args.is_deleted
    }

    pub fn call_execute<'a, I>(
        &self,
        inputs: I,
        options: &'a ExecuteOptions<'a>,
    ) -> Result<(Vec<Event>, Vec<Vec<Buffer>>)>
    where
        I: ExecutionInputs,
    {
        let executable = self.executable();
        let num_outputs = executable.num_outputs();
        let input_buffers = inputs.buffer_ptrs();
        let mut args = PJRT_LoadedExecutable_Execute_Args::new();
        args.executable = self.ptr;
        args.num_devices = input_buffers.len();
        args.num_args = input_buffers[0].len();
        // allocate argument lists pass to pjrt runtime
        let mut argument_lists = Vec::with_capacity(input_buffers.len());
        for d in input_buffers.iter() {
            argument_lists.push(Some(d.as_slice()));
        }
        args.argument_lists = argument_lists.as_ptr() as *const *const *mut PJRT_Buffer;
        // allocate output buffers and complete_events and let pjrt runtime to fill it
        let output_inner = vec![MaybeUninit::<*mut PJRT_Buffer>::uninit(); num_outputs];
        let output_lists = vec![Some(output_inner.as_slice()); args.num_devices];
        args.output_lists = output_lists.as_ptr() as *const *mut *mut PJRT_Buffer;
        // allocate complete_events and let pjrt runtime to fill it
        let complete_events = vec![MaybeUninit::<*mut PJRT_Event>::uninit(); args.num_devices];
        args.device_complete_events = complete_events.as_ptr() as *mut *mut PJRT_Event;
        // options - use ExecuteOptionsRaw to handle callback lifetimes
        let mut raw_options = PJRT_ExecuteOptions::new();
        let _options_raw = ExecuteOptionsRaw::new(options, &mut raw_options);
        args.options = &mut raw_options as *mut PJRT_ExecuteOptions;
        args = self.client.api().PJRT_LoadedExecutable_Execute(args)?;
        let events =
            unsafe { slice::from_raw_parts(args.device_complete_events, args.num_devices) };
        let events = events
            .iter()
            .cloned()
            .map(|ptr| event::Event::wrap(self.client.api(), ptr))
            .collect::<Vec<_>>();
        let output_buffers = unsafe {
            utils::slice_to_vec2d(args.output_lists, args.num_devices, num_outputs, |ptr| {
                Buffer::wrap(&self.client, ptr)
            })
        };
        Ok((events, output_buffers))
    }

    pub fn execute_sync<'a, I>(
        &self,
        inputs: I,
        options: &'a ExecuteOptions<'a>,
    ) -> Result<Vec<Vec<Buffer>>>
    where
        I: ExecutionInputs,
    {
        let (events, outputs) = self.call_execute(inputs, options)?;
        for event in events {
            event.wait()?;
        }
        Ok(outputs)
    }

    pub async fn execute<'a, I>(
        &self,
        inputs: I,
        options: &'a ExecuteOptions<'a>,
    ) -> Result<Vec<Vec<Buffer>>>
    where
        I: ExecutionInputs,
    {
        let (events, outputs) = self.call_execute(inputs, options)?;
        for event in events {
            event.await?;
        }
        Ok(outputs)
    }

    pub fn execution<I>(&self, inputs: I) -> Execution<'_, I>
    where
        I: ExecutionInputs,
    {
        Execution::new(self, inputs)
    }
}
