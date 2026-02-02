//! PJRT Execution Context
//!
//! This module provides types and traits for executing compiled PJRT programs.
//! It includes:
//!
//! - `ExecuteContext`: Context for execution operations
//! - `ExecuteOptions`: Configuration options for execution
//! - `Execution`: A builder pattern for configuring and running executions
//! - `ExecutionInputs`: Trait for types that can be used as execution inputs
//! - `SendCallbackInfo` / `RecvCallbackInfo`: Callbacks for distributed communication
//! - `CallLocation`: Source location information for debugging
//!
//! The module provides both synchronous and asynchronous execution patterns,
//! supporting various input types including single buffers, arrays, and vectors.

use std::collections::HashSet;
use std::ffi::{c_void, CString};
use std::marker::PhantomData;

use pjrt_sys::{
    PJRT_Buffer, PJRT_ExecuteContext, PJRT_ExecuteContext_Destroy_Args, PJRT_ExecuteOptions,
    PJRT_RecvCallbackInfo, PJRT_SendCallbackInfo,
};

use crate::{Api, Buffer, LoadedExecutable, Result};

/// Context for PJRT execution operations.
///
/// An `ExecuteContext` provides the environment for executing compiled programs.
/// It can be used to share state across multiple executions and manage
/// resources for execution operations.
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

/// Options for configuring PJRT execution.
///
/// `ExecuteOptions` allows you to configure various aspects of execution,
/// including launch identifiers, input donation behavior, send/recv callbacks
/// for distributed communication, and debugging information.
///
/// # Example
///
/// ```rust,ignore
/// let options = ExecuteOptions::new()
///     .launch_id(1)
///     .non_donatable_input_indices(vec![0, 2])
///     .call_location(CallLocation::new("my_function", "example.py", 42));
/// ```
pub struct ExecuteOptions<'a> {
    launch_id: i32,
    non_donatable_input_indices: Vec<i64>,
    call_location: Option<CallLocation>,
    task_ids: Vec<i32>,
    incarnation_ids: Vec<i64>,
    /// Send callbacks per device. Outer vec is per device, inner vec is per send op.
    send_callbacks: Vec<Vec<SendCallbackInfo<'a>>>,
    /// Recv callbacks per device. Outer vec is per device, inner vec is per recv op.
    recv_callbacks: Vec<Vec<RecvCallbackInfo<'a>>>,
}

impl<'a> ExecuteOptions<'a> {
    pub fn new() -> Self {
        Self {
            launch_id: 0,
            non_donatable_input_indices: vec![],
            call_location: None,
            task_ids: vec![],
            incarnation_ids: vec![],
            send_callbacks: vec![],
            recv_callbacks: vec![],
        }
    }

    /// Sets the launch ID for this execution.
    ///
    /// If non-zero, identifies this execution as part of a potentially
    /// multi-device launch. This can be used to detect scheduling errors,
    /// e.g., if multi-host programs are launched in different orders on
    /// different hosts.
    pub fn launch_id(mut self, launch_id: i32) -> Self {
        self.launch_id = launch_id;
        self
    }

    /// Sets the indices of inputs that should not be donated.
    ///
    /// An input buffer may be non-donable, for example, if it is referenced
    /// more than once. Since such runtime information is not available at
    /// compile time, the compiler might mark the input as `may-alias`, which
    /// could lead PjRt to donate the input buffer when it should not.
    ///
    /// By defining this list of indices, a higher-level PJRT caller can
    /// instruct PJRT client not to donate specific input buffers.
    pub fn non_donatable_input_indices(mut self, indices: impl Into<Vec<i64>>) -> Self {
        self.non_donatable_input_indices = indices.into();
        self
    }

    /// Sets the call location for debugging and error reporting.
    ///
    /// The call location stores the source location (e.g., file:line) of the
    /// code that triggered the execution. This can be used for debugging and
    /// error reporting, allowing users to pinpoint which program execution
    /// led to an issue.
    pub fn call_location(mut self, location: CallLocation) -> Self {
        self.call_location = Some(location);
        self
    }

    /// Sets the task and incarnation IDs for distributed execution.
    ///
    /// For every `0 <= i < num_tasks`, task `task_ids[i]` has incarnation
    /// `incarnation_ids[i]`. These are used in distributed execution scenarios
    /// to track task identities and their incarnations.
    ///
    /// # Panics
    ///
    /// Panics if `task_ids` and `incarnation_ids` have different lengths.
    pub fn task_incarnation_ids(
        mut self,
        task_ids: impl Into<Vec<i32>>,
        incarnation_ids: impl Into<Vec<i64>>,
    ) -> Self {
        let task_ids = task_ids.into();
        let incarnation_ids = incarnation_ids.into();
        assert_eq!(
            task_ids.len(),
            incarnation_ids.len(),
            "task_ids and incarnation_ids must have the same length"
        );
        self.task_ids = task_ids;
        self.incarnation_ids = incarnation_ids;
        self
    }

    /// Returns the launch ID.
    pub fn get_launch_id(&self) -> i32 {
        self.launch_id
    }

    /// Returns the non-donatable input indices.
    pub fn get_non_donatable_input_indices(&self) -> &[i64] {
        &self.non_donatable_input_indices
    }

    /// Returns the call location if set.
    pub fn get_call_location(&self) -> Option<&CallLocation> {
        self.call_location.as_ref()
    }

    /// Sets the send callbacks for distributed execution.
    ///
    /// The outer vector corresponds to each device (length `num_devices`).
    /// The inner vector contains callback info for each send op; the order
    /// doesn't matter as channel IDs are used to match callbacks to operations.
    ///
    /// # Safety
    ///
    /// The callback functions must outlive the execution. The user_arg pointers
    /// in each SendCallbackInfo must remain valid for the duration of execution.
    pub fn send_callbacks(mut self, callbacks: Vec<Vec<SendCallbackInfo<'a>>>) -> Self {
        self.send_callbacks = callbacks;
        self
    }

    /// Sets the recv callbacks for distributed execution.
    ///
    /// The outer vector corresponds to each device (length `num_devices`).
    /// The inner vector contains callback info for each recv op; the order
    /// doesn't matter as channel IDs are used to match callbacks to operations.
    ///
    /// # Safety
    ///
    /// The callback functions must outlive the execution. The user_arg pointers
    /// in each RecvCallbackInfo must remain valid for the duration of execution.
    pub fn recv_callbacks(mut self, callbacks: Vec<Vec<RecvCallbackInfo<'a>>>) -> Self {
        self.recv_callbacks = callbacks;
        self
    }

    /// Returns the send callbacks.
    pub fn get_send_callbacks(&self) -> &[Vec<SendCallbackInfo<'a>>] {
        &self.send_callbacks
    }

    /// Returns the recv callbacks.
    pub fn get_recv_callbacks(&self) -> &[Vec<RecvCallbackInfo<'a>>] {
        &self.recv_callbacks
    }
}

impl Default for ExecuteOptions<'_> {
    fn default() -> Self {
        Self::new()
    }
}

/// Source location information for debugging and error reporting.
///
/// `CallLocation` stores information about where a PJRT execution was triggered
/// from, typically the source location in a higher-level framework like JAX or PyTorch.
/// This differs from operation metadata which refers to the origin of individual
/// operations within the compiled module.
///
/// The plugin can use this information for debugging and error reporting,
/// allowing users to pinpoint which program execution led to an issue.
///
/// # Example
///
/// ```rust
/// use pjrt::CallLocation;
///
/// let location = CallLocation::new("train_step", "model.py", 42);
/// assert_eq!(location.function_name(), Some("train_step"));
/// assert_eq!(location.file_name(), Some("model.py"));
/// assert_eq!(location.line_number(), Some(42));
/// ```
#[derive(Debug, Clone)]
pub struct CallLocation {
    // Stored as "function_name:file_name:line" or similar format
    location_string: CString,
}

impl CallLocation {
    /// Creates a new call location from function name, file name, and line number.
    ///
    /// The information is combined into a string format suitable for PJRT plugins.
    pub fn new(function_name: &str, file_name: &str, line: u32) -> Self {
        let location = format!("{}:{}:{}", function_name, file_name, line);
        Self {
            location_string: CString::new(location).expect("location contains null bytes"),
        }
    }

    /// Creates a call location from a pre-formatted location string.
    ///
    /// The format should be understood by the PJRT plugin being used.
    /// Common formats include "file:line" or "function:file:line".
    pub fn from_string(location: &str) -> Self {
        Self {
            location_string: CString::new(location).expect("location contains null bytes"),
        }
    }

    /// Returns the raw location string as a C string pointer.
    pub(crate) fn as_ptr(&self) -> *const i8 {
        self.location_string.as_ptr()
    }

    /// Parses the function name from the location string if available.
    pub fn function_name(&self) -> Option<&str> {
        let s = self.location_string.to_str().ok()?;
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() >= 3 {
            Some(parts[0])
        } else {
            None
        }
    }

    /// Parses the file name from the location string if available.
    pub fn file_name(&self) -> Option<&str> {
        let s = self.location_string.to_str().ok()?;
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() >= 3 {
            Some(parts[1])
        } else if parts.len() >= 2 {
            Some(parts[0])
        } else {
            None
        }
    }

    /// Parses the line number from the location string if available.
    pub fn line_number(&self) -> Option<u32> {
        let s = self.location_string.to_str().ok()?;
        let parts: Vec<&str> = s.split(':').collect();
        if let Some(last) = parts.last() {
            last.parse().ok()
        } else {
            None
        }
    }
}

/// Metadata for data transfers between host and device.
///
/// `TransferMetadata` contains information about data being transferred,
/// which is used by send/recv callbacks during distributed execution.
/// This includes the shape, type, and layout of the data being transferred.
#[derive(Debug)]
pub struct TransferMetadata {
    /// The dimensions of the tensor being transferred.
    pub dims: Vec<i64>,
    /// The element type of the tensor.
    pub element_type: crate::PrimitiveType,
    /// Optional memory layout information.
    pub layout: Option<crate::MemoryLayout>,
}

impl TransferMetadata {
    /// Creates new transfer metadata.
    pub fn new(dims: Vec<i64>, element_type: crate::PrimitiveType) -> Self {
        Self {
            dims,
            element_type,
            layout: None,
        }
    }

    /// Sets the memory layout for this transfer.
    pub fn with_layout(mut self, layout: crate::MemoryLayout) -> Self {
        self.layout = Some(layout);
        self
    }

    /// Returns the total number of elements.
    pub fn num_elements(&self) -> i64 {
        self.dims.iter().product()
    }

    /// Returns the total size in bytes.
    ///
    /// Returns `None` if the element type doesn't have a known size.
    pub fn size_in_bytes(&self) -> Option<usize> {
        let dtype = self.element_type.try_into_dtype().ok()?;
        Some(self.num_elements() as usize * dtype.size())
    }
}

/// Callback error for send/recv operations.
///
/// This type is used to report errors from within send/recv callbacks
/// back to the PJRT runtime.
pub type CallbackError = pjrt_sys::PJRT_CallbackError;

/// Type alias for the send callback function.
///
/// The send callback is invoked when data is ready to be sent to the host.
/// It receives:
/// - `chunk`: The data chunk to process
/// - `callback_error`: Error reporting mechanism
/// - `total_size_in_bytes`: Total size of the transfer
/// - `done`: Whether this is the last chunk
/// - `user_arg`: User-provided context
///
/// Returns `nullptr` on success, or a PJRT_Error* on failure.
pub type SendCallback = pjrt_sys::PJRT_SendCallback;

/// Type alias for the recv callback function.
///
/// The recv callback is invoked when the device is ready to receive data.
/// It receives:
/// - `stream`: A copy-to-device stream for writing data
/// - `user_arg`: User-provided context
///
/// The callback takes ownership of the stream and must call
/// `PJRT_CopyToDeviceStream_Destroy` when done.
pub type RecvCallback = pjrt_sys::PJRT_RecvCallback;

/// Information for configuring a send callback.
///
/// `SendCallbackInfo` associates a send callback with a specific channel ID
/// and user-provided context. This is used during execution to handle
/// outbound data transfers in distributed execution scenarios.
///
/// # Safety
///
/// The `user_arg` pointer must remain valid for the duration of the execution.
/// The `send_callback` function must be safe to call from PJRT's internal threads.
pub struct SendCallbackInfo<'a> {
    /// Channel ID to associate this callback with the correct send operation.
    pub channel_id: i64,
    /// User-provided argument passed to the callback.
    pub user_arg: *mut c_void,
    /// The callback function to invoke.
    pub send_callback: SendCallback,
    _marker: PhantomData<&'a ()>,
}

impl<'a> SendCallbackInfo<'a> {
    /// Creates a new send callback info.
    ///
    /// # Safety
    ///
    /// - `user_arg` must remain valid for the duration of the execution
    /// - `send_callback` must be safe to call from any thread
    pub unsafe fn new(channel_id: i64, user_arg: *mut c_void, send_callback: SendCallback) -> Self {
        Self {
            channel_id,
            user_arg,
            send_callback,
            _marker: PhantomData,
        }
    }

    pub(crate) fn to_raw(&self) -> PJRT_SendCallbackInfo {
        let mut info = PJRT_SendCallbackInfo::new();
        info.channel_id = self.channel_id;
        info.user_arg = self.user_arg;
        info.send_callback = self.send_callback;
        info
    }
}

impl std::fmt::Debug for SendCallbackInfo<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SendCallbackInfo")
            .field("channel_id", &self.channel_id)
            .field("user_arg", &self.user_arg)
            .finish()
    }
}

/// Information for configuring a recv callback.
///
/// `RecvCallbackInfo` associates a recv callback with a specific channel ID
/// and user-provided context. This is used during execution to handle
/// inbound data transfers in distributed execution scenarios.
///
/// # Safety
///
/// The `user_arg` pointer must remain valid for the duration of the execution.
/// The `recv_callback` function must be safe to call from PJRT's internal threads.
pub struct RecvCallbackInfo<'a> {
    /// Channel ID to associate this callback with the correct recv operation.
    pub channel_id: i64,
    /// User-provided argument passed to the callback.
    pub user_arg: *mut c_void,
    /// The callback function to invoke.
    pub recv_callback: RecvCallback,
    _marker: PhantomData<&'a ()>,
}

impl<'a> RecvCallbackInfo<'a> {
    /// Creates a new recv callback info.
    ///
    /// # Safety
    ///
    /// - `user_arg` must remain valid for the duration of the execution
    /// - `recv_callback` must be safe to call from any thread
    pub unsafe fn new(channel_id: i64, user_arg: *mut c_void, recv_callback: RecvCallback) -> Self {
        Self {
            channel_id,
            user_arg,
            recv_callback,
            _marker: PhantomData,
        }
    }

    pub(crate) fn to_raw(&self) -> PJRT_RecvCallbackInfo {
        let mut info = PJRT_RecvCallbackInfo::new();
        info.channel_id = self.channel_id;
        info.user_arg = self.user_arg;
        info.recv_callback = self.recv_callback;
        info
    }
}

impl std::fmt::Debug for RecvCallbackInfo<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RecvCallbackInfo")
            .field("channel_id", &self.channel_id)
            .field("user_arg", &self.user_arg)
            .finish()
    }
}

/// Helper struct to hold the raw callback info arrays during execution.
///
/// This struct manages the lifetime of the raw PJRT callback info structures
/// that need to be passed to the C API. The raw PJRT_ExecuteOptions references
/// data owned by this struct, so this must be kept alive for the duration
/// of the PJRT API call.
pub(crate) struct ExecuteOptionsRaw {
    send_callbacks_raw: Vec<Vec<PJRT_SendCallbackInfo>>,
    recv_callbacks_raw: Vec<Vec<PJRT_RecvCallbackInfo>>,
    send_callbacks_ptrs: Vec<*mut PJRT_SendCallbackInfo>,
    recv_callbacks_ptrs: Vec<*mut PJRT_RecvCallbackInfo>,
}

impl ExecuteOptionsRaw {
    /// Creates a new ExecuteOptionsRaw from ExecuteOptions and populates the raw PJRT_ExecuteOptions.
    pub fn new<'a>(options: &'a ExecuteOptions<'a>, raw: &mut PJRT_ExecuteOptions) -> Self {
        // Convert send callbacks to raw format
        let mut send_callbacks_raw: Vec<Vec<PJRT_SendCallbackInfo>> = options
            .send_callbacks
            .iter()
            .map(|device_callbacks| device_callbacks.iter().map(|cb| cb.to_raw()).collect())
            .collect();

        // Convert recv callbacks to raw format
        let mut recv_callbacks_raw: Vec<Vec<PJRT_RecvCallbackInfo>> = options
            .recv_callbacks
            .iter()
            .map(|device_callbacks| device_callbacks.iter().map(|cb| cb.to_raw()).collect())
            .collect();

        // Create pointer arrays for send callbacks
        let send_callbacks_ptrs: Vec<*mut PJRT_SendCallbackInfo> = send_callbacks_raw
            .iter_mut()
            .map(|v| v.as_mut_ptr())
            .collect();

        // Create pointer arrays for recv callbacks
        let recv_callbacks_ptrs: Vec<*mut PJRT_RecvCallbackInfo> = recv_callbacks_raw
            .iter_mut()
            .map(|v| v.as_mut_ptr())
            .collect();

        // Populate the raw options
        raw.launch_id = options.launch_id;
        raw.non_donatable_input_indices = options.non_donatable_input_indices.as_ptr();
        raw.num_non_donatable_input_indices = options.non_donatable_input_indices.len();

        if let Some(ref location) = options.call_location {
            raw.call_location = location.as_ptr();
        }

        if !options.task_ids.is_empty() {
            raw.num_tasks = options.task_ids.len();
            raw.task_ids = options.task_ids.as_ptr() as *mut i32;
            raw.incarnation_ids = options.incarnation_ids.as_ptr() as *mut i64;
        }

        let result = Self {
            send_callbacks_raw,
            recv_callbacks_raw,
            send_callbacks_ptrs,
            recv_callbacks_ptrs,
        };

        // Set callback pointers after creating the result so the Vecs won't move
        if !result.send_callbacks_ptrs.is_empty() {
            raw.send_callbacks =
                result.send_callbacks_ptrs.as_ptr() as *mut *mut PJRT_SendCallbackInfo;
            raw.num_send_ops = result
                .send_callbacks_raw
                .first()
                .map(|v| v.len())
                .unwrap_or(0);
        }

        if !result.recv_callbacks_ptrs.is_empty() {
            raw.recv_callbacks =
                result.recv_callbacks_ptrs.as_ptr() as *mut *mut PJRT_RecvCallbackInfo;
            raw.num_recv_ops = result
                .recv_callbacks_raw
                .first()
                .map(|v| v.len())
                .unwrap_or(0);
        }

        result
    }
}

impl<'a> From<&'a ExecuteOptions<'a>> for PJRT_ExecuteOptions {
    fn from(v: &'a ExecuteOptions<'a>) -> Self {
        let mut options = PJRT_ExecuteOptions::new();
        options.launch_id = v.launch_id;
        options.non_donatable_input_indices = v.non_donatable_input_indices.as_ptr();
        options.num_non_donatable_input_indices = v.non_donatable_input_indices.len();

        if let Some(ref location) = v.call_location {
            options.call_location = location.as_ptr();
        }

        if !v.task_ids.is_empty() {
            options.num_tasks = v.task_ids.len();
            options.task_ids = v.task_ids.as_ptr() as *mut i32;
            options.incarnation_ids = v.incarnation_ids.as_ptr() as *mut i64;
        }

        // Note: send/recv callbacks require ExecuteOptionsRaw to be kept alive
        // This simple From impl doesn't support callbacks - use ExecuteOptionsRaw::new() instead

        options
    }
}

/// A builder for configuring and running PJRT executions.
///
/// `Execution` provides a fluent interface for configuring execution options
/// and then running the execution either synchronously or asynchronously.
///
/// # Example
///
/// ```rust,ignore
/// let execution = loaded_executable.execution(inputs)
///     .launch_id(1)
///     .non_donatable_input_indices(vec![0]);
///
/// // Run asynchronously
/// let outputs = execution.run().await?;
///
/// // Or run synchronously
/// let outputs = execution.run_sync()?;
/// ```
pub struct Execution<'a, T> {
    pub loaded_executable: &'a LoadedExecutable,
    pub inputs: T,
    pub options: ExecuteOptions<'a>,
}

impl<'a, T> Execution<'a, T>
where
    T: ExecutionInputs,
{
    pub fn new(loaded_executable: &'a LoadedExecutable, inputs: T) -> Self {
        let options = ExecuteOptions {
            launch_id: 0,
            non_donatable_input_indices: inputs.non_donatable_input_indices(),
            call_location: None,
            task_ids: vec![],
            incarnation_ids: vec![],
            send_callbacks: vec![],
            recv_callbacks: vec![],
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

/// Trait for types that can be used as inputs to PJRT executions.
///
/// This trait is implemented for various buffer collection types, allowing
/// them to be used directly as execution inputs.
///
/// # Implementors
///
/// - `()`: Empty input (no arguments)
/// - `Buffer`: Single buffer input
/// - `[Buffer; N]`: Array of buffers
/// - `[[Buffer; A]; D]`: 2D array of buffers (multi-device)
/// - `Vec<Buffer>`: Vector of buffers
/// - `Vec<Vec<Buffer>>`: Vector of vectors (multi-device)
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
