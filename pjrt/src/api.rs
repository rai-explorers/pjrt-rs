//! PJRT API Entry Point
//!
//! This module provides the main entry point to the PJRT runtime through the `Api` struct.
//! The `Api` represents the loaded PJRT library and provides methods to:
//!
//! - Load PJRT plugins
//! - Create runtime clients
//! - Compile programs to executables
//! - Create execution contexts
//! - Manage topology descriptions
//!
//! The `Api` struct is thread-safe and can be cloned to share between threads.
//!
//! # Examples
//!
//! ## Loading a Plugin and Creating a Client
//!
//! ```rust,ignore
//! use pjrt::{plugin, Client};
//!
//! // Load a PJRT plugin (e.g., CPU plugin)
//! let api = plugin("/path/to/pjrt_c_api_cpu_plugin.so").load()?;
//!
//! // Create a client to interact with devices
//! let client = Client::builder(&api).build()?;
//!
//! // Query API version
//! let version = api.version();
//! println!("PJRT API version: {}.{}", version.major(), version.minor());
//!
//! // Check available devices
//! println!("Devices: {}", client.devices().len());
//! ```
//!
//! ## Working with Extensions
//!
//! ```rust,ignore
//! use pjrt::{plugin, Client, StreamExtension};
//!
//! let api = plugin("/path/to/pjrt_c_api_gpu_plugin.so").load()?;
//!
//! // Query for available extensions
//! if let Some(stream_ext) = api.get_extension::<StreamExtension>() {
//!     println!("Stream extension is available");
//! }
//! ```
//!
//! ## Plugin with Options
//!
//! ```rust,ignore
//! use pjrt::{plugin, Client, NamedValue};
//!
//! let api = plugin("/path/to/plugin.so").load()?;
//!
//! // Create client with custom options
//! let options = vec![
//!     NamedValue::i64("device_count", 2),
//!     NamedValue::bool("async_execution", true),
//! ];
//!
//! let client = Client::builder(&api)
//!     .options(options)
//!     .build()?;
//! ```

use std::backtrace::Backtrace;
use std::sync::Arc;

use pjrt_sys::{
    PJRT_Api, PJRT_Api_Version, PJRT_Client_Create_Args, PJRT_Compile_Args, PJRT_Error,
    PJRT_Error_Destroy_Args, PJRT_Error_GetCode_Args, PJRT_Error_Message_Args,
    PJRT_ExecuteContext_Create_Args, PJRT_NamedValue, PJRT_Plugin_Attributes_Args,
    PJRT_Plugin_Initialize_Args, PJRT_Program, PJRT_TopologyDescription_Create_Args,
};

use crate::kv_store::{kv_get_callback, kv_put_callback, kv_try_get_callback};
use crate::named_value::NamedValueMap;
use crate::{
    utils, Client, CompileOptions, CompileToExecutable, Error, Executable, ExecuteContext,
    KeyValueStore, NamedValue, Program, Result, TopologyDescription,
};

/// The main entry point for interacting with a PJRT plugin.
///
/// `Api` represents a loaded PJRT plugin and provides methods to create clients,
/// compile programs, and access plugin metadata. It is the first object you need
/// to create when using pjrt-rs.
///
/// # Thread Safety
///
/// `Api` implements `Send` and `Sync`, allowing it to be shared across threads.
/// This is safe because:
///
/// 1. The PJRT C API is designed to be thread-safe. The API functions use proper
///    synchronization internally.
/// 2. All state is encapsulated in the PJRT runtime, which manages its own
///    thread safety.
/// 3. The `Arc<PJRT_Api>` ensures the API pointer remains valid as long as any
///    `Api` instance exists.
///
/// **Important**: The thread-safety guarantee depends on the PJRT plugin being
/// correctly implemented. The official PJRT plugins (CPU, GPU, TPU) are all
/// thread-safe. If you are using a third-party plugin, verify it is thread-safe
/// before sharing `Api` across threads.
///
/// # Example
///
/// ```rust,ignore
/// use pjrt::{Api, plugin};
///
/// // Load the CPU plugin
/// let api = plugin("path/to/pjrt_c_api_cpu_plugin.so")?;
///
/// // Create a client
/// let client = api.create_client(vec![], None)?;
///
/// // The Api can be cloned and shared across threads
/// let api_clone = api.clone();
/// std::thread::spawn(move || {
///     // Safe to use api_clone here
///     let version = api_clone.version();
/// });
/// ```
#[derive(Clone)]
pub struct Api {
    raw: Arc<PJRT_Api>,
    version: Version,
}

impl std::fmt::Debug for Api {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Api")
            .field("version", &self.version)
            .finish()
    }
}

// SAFETY: The PJRT C API is designed to be thread-safe. The PJRT_Api struct
// is essentially a vtable of function pointers that don't change after
// initialization. All state is managed by the PJRT runtime which handles
// its own synchronization.
//
// See: https://github.com/openxla/xla/blob/main/xla/pjrt/c/pjrt_c_api.h
// The PJRT API documentation states that the API is thread-safe.
unsafe impl Send for Api {}
unsafe impl Sync for Api {}

impl Api {
    #[allow(clippy::arc_with_non_send_sync)]
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub(crate) fn wrap(ptr: *const PJRT_Api) -> Self {
        assert!(!ptr.is_null());
        let raw = Arc::new(unsafe { *ptr });
        let version = Version::new(raw.pjrt_api_version);
        let api = Self { raw, version };
        let args = PJRT_Plugin_Initialize_Args::new();
        api.PJRT_Plugin_Initialize(args)
            .expect("PJRT_Plugin_Initialize");
        api
    }

    /// Create a minimal `Api` for unit testing.
    ///
    /// # Safety
    ///
    /// The returned `Api` has all function pointers set to `None` and must
    /// **not** be used to call any PJRT functions. It is only suitable for
    /// constructing types that hold an `Api` reference in tests.
    #[cfg(test)]
    #[allow(clippy::arc_with_non_send_sync)]
    pub(crate) unsafe fn empty_for_testing() -> Self {
        let raw = Arc::new(PJRT_Api::default());
        let version = Version {
            major_version: 0,
            minor_version: 0,
        };
        Self { raw, version }
    }

    /// Returns the PJRT API version supported by this plugin.
    pub fn version(&self) -> Version {
        self.version
    }

    /// Returns the head of the extension linked list, if any.
    pub(crate) fn extension_start(&self) -> *mut pjrt_sys::PJRT_Extension_Base {
        self.raw.extension_start
    }

    /// Returns plugin-specific attributes as key-value pairs.
    ///
    /// Common attributes include:
    /// - `xla_version`: The XLA version string
    /// - `stablehlo_current_version`: The StableHLO version supported
    /// - `stablehlo_minimum_version`: The minimum StableHLO version supported
    pub fn plugin_attributes(&self) -> Result<NamedValueMap> {
        let mut args = PJRT_Plugin_Attributes_Args::new();
        args = self.PJRT_Plugin_Attributes(args)?;
        utils::to_named_value_map(args.attributes, args.num_attributes)
    }

    pub fn create_execute_context(&self) -> Result<ExecuteContext> {
        let mut args = PJRT_ExecuteContext_Create_Args::new();
        args = self.PJRT_ExecuteContext_Create(args)?;
        Ok(ExecuteContext::wrap(self, args.context))
    }

    pub fn create_topology(
        &self,
        name: impl AsRef<str>,
        options: Vec<NamedValue>,
    ) -> Result<TopologyDescription> {
        let name = name.as_ref().as_bytes();
        let create_options: Vec<PJRT_NamedValue> = options.iter().map(Into::into).collect();
        let mut args = PJRT_TopologyDescription_Create_Args::new();
        args.topology_name = name.as_ptr() as *const i8;
        args.topology_name_size = name.len();
        args.create_options = create_options.as_ptr();
        args.num_options = create_options.len();
        args = self.PJRT_TopologyDescription_Create(args)?;
        Ok(TopologyDescription::wrap(self, args.topology, None))
    }

    #[allow(clippy::borrowed_box)]
    pub fn create_client(
        &self,
        options: Vec<NamedValue>,
        kv_store: Option<&Box<dyn KeyValueStore>>,
    ) -> Result<Client> {
        let create_options: Vec<PJRT_NamedValue> = options.iter().map(Into::into).collect();
        let mut args = PJRT_Client_Create_Args::new();
        args.create_options = create_options.as_ptr();
        args.num_options = create_options.len();
        if let Some(kv_store) = kv_store {
            args.kv_get_callback = Some(kv_get_callback);
            args.kv_get_user_arg = kv_store as *const _ as *mut _;
            args.kv_put_callback = Some(kv_put_callback);
            args.kv_put_user_arg = kv_store as *const _ as *mut _;
            args.kv_try_get_callback = Some(kv_try_get_callback);
            args.kv_try_get_user_arg = kv_store as *const _ as *mut _;
        }
        args = self.PJRT_Client_Create(args)?;
        Ok(Client::wrap(self, args.client))
    }

    pub fn compile<T>(
        &self,
        program: &T,
        topology: &TopologyDescription,
        options: CompileOptions,
        client: Option<&Client>,
    ) -> Result<Executable>
    where
        Self: CompileToExecutable<T>,
    {
        CompileToExecutable::<T>::compile(self, program, topology, &options, client)
    }

    /// Converts a PJRT error pointer to a Result, with the function name for context.
    #[allow(unused_assignments)]
    pub(crate) fn err_or_with_fn<T>(
        &self,
        err: *mut PJRT_Error,
        value: T,
        function: &'static str,
    ) -> Result<T> {
        if err.is_null() {
            Ok(value)
        } else {
            let mut args = PJRT_Error_Message_Args::new();
            args.error = err;
            self.PJRT_Error_Message(&mut args)?;
            let msg = utils::str_from_raw(args.message, args.message_size).into_owned();
            let mut args = PJRT_Error_GetCode_Args::new();
            args.error = err;
            args = self.PJRT_Error_GetCode(args)?;
            let code = args.code.try_into()?;
            let mut args = PJRT_Error_Destroy_Args::new();
            args.error = err;
            self.PJRT_Error_Destroy(&mut args)?;
            let backtrace = Backtrace::capture().to_string();
            Err(Error::PjrtError {
                function,
                msg,
                code,
                backtrace,
            })
        }
    }

    /// Converts a PJRT error pointer to a Result.
    ///
    /// This is a convenience method for code that doesn't have a function name.
    #[allow(unused_assignments)]
    pub(crate) fn err_or<T>(&self, err: *mut PJRT_Error, value: T) -> Result<T> {
        self.err_or_with_fn(err, value, "<unknown>")
    }
}

impl CompileToExecutable<Program> for Api {
    fn compile(
        &self,
        program: &Program,
        topology: &TopologyDescription,
        options: &CompileOptions,
        client: Option<&Client>,
    ) -> Result<Executable> {
        let options_encoded = options.encode();
        let mut args = PJRT_Compile_Args::new();
        args.topology = topology.ptr;
        args.program = &program.prog as *const PJRT_Program;
        args.compile_options = options_encoded.as_ptr() as *const i8;
        args.compile_options_size = options_encoded.len();
        if let Some(client) = client {
            args.client = client.ptr();
        }
        args = self.PJRT_Compile(args)?;
        Ok(Executable::wrap(self, args.executable))
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Version {
    pub major_version: i32,
    pub minor_version: i32,
}

impl Version {
    pub(crate) fn new(raw: PJRT_Api_Version) -> Self {
        let major_version = raw.major_version;
        let minor_version = raw.minor_version;
        Self {
            major_version,
            minor_version,
        }
    }
}

macro_rules! pjrt_api_fn_ret_err {
    ($fn:ident, $args_ty:ident) => {
        #[allow(dead_code)]
        impl Api {
            #[allow(non_snake_case)]
            #[allow(dead_code)]
            #[must_use = "get function result from returned value"]
            pub(crate) fn $fn(
                &self,
                mut args: pjrt_sys::$args_ty,
            ) -> $crate::Result<pjrt_sys::$args_ty> {
                let func = self
                    .raw
                    .$fn
                    .ok_or(Error::NullFunctionPointer(stringify!($fn)))?;
                let err = unsafe { func(&mut args as *mut _) };
                self.err_or_with_fn(err, args, stringify!($fn))
            }
        }
    };
}

macro_rules! pjrt_api_fn_ret_void {
    ($fn:ident, $args_ty:ident) => {
        #[allow(dead_code)]
        impl Api {
            #[allow(non_snake_case)]
            #[allow(dead_code)]
            pub(crate) fn $fn(&self, args: &mut pjrt_sys::$args_ty) -> Result<()> {
                let func = self
                    .raw
                    .$fn
                    .ok_or(Error::NullFunctionPointer(stringify!($fn)))?;
                unsafe { func(args as *mut _) };
                Ok(())
            }
        }
    };
}

pjrt_api_fn_ret_void!(PJRT_Error_Message, PJRT_Error_Message_Args);
pjrt_api_fn_ret_void!(PJRT_Error_Destroy, PJRT_Error_Destroy_Args);
pjrt_api_fn_ret_err!(PJRT_Error_GetCode, PJRT_Error_GetCode_Args);

pjrt_api_fn_ret_err!(PJRT_Plugin_Initialize, PJRT_Plugin_Initialize_Args);
pjrt_api_fn_ret_err!(PJRT_Plugin_Attributes, PJRT_Plugin_Attributes_Args);

pjrt_api_fn_ret_err!(PJRT_Event_Destroy, PJRT_Event_Destroy_Args);
pjrt_api_fn_ret_err!(PJRT_Event_IsReady, PJRT_Event_IsReady_Args);
pjrt_api_fn_ret_err!(PJRT_Event_Error, PJRT_Event_Error_Args);
pjrt_api_fn_ret_err!(PJRT_Event_Await, PJRT_Event_Await_Args);
pjrt_api_fn_ret_err!(PJRT_Event_OnReady, PJRT_Event_OnReady_Args);

pjrt_api_fn_ret_err!(PJRT_Client_Create, PJRT_Client_Create_Args);
pjrt_api_fn_ret_err!(PJRT_Client_Destroy, PJRT_Client_Destroy_Args);
pjrt_api_fn_ret_err!(PJRT_Client_PlatformName, PJRT_Client_PlatformName_Args);
pjrt_api_fn_ret_err!(PJRT_Client_ProcessIndex, PJRT_Client_ProcessIndex_Args);
pjrt_api_fn_ret_err!(
    PJRT_Client_PlatformVersion,
    PJRT_Client_PlatformVersion_Args
);
pjrt_api_fn_ret_err!(PJRT_Client_Devices, PJRT_Client_Devices_Args);
pjrt_api_fn_ret_err!(
    PJRT_Client_AddressableDevices,
    PJRT_Client_AddressableDevices_Args
);
pjrt_api_fn_ret_err!(PJRT_Client_LookupDevice, PJRT_Client_LookupDevice_Args);
pjrt_api_fn_ret_err!(
    PJRT_Client_LookupAddressableDevice,
    PJRT_Client_LookupAddressableDevice_Args
);
pjrt_api_fn_ret_err!(
    PJRT_Client_AddressableMemories,
    PJRT_Client_AddressableMemories_Args
);
pjrt_api_fn_ret_err!(PJRT_Client_Compile, PJRT_Client_Compile_Args);
pjrt_api_fn_ret_err!(
    PJRT_Client_DefaultDeviceAssignment,
    PJRT_Client_DefaultDeviceAssignment_Args
);
pjrt_api_fn_ret_err!(
    PJRT_Client_BufferFromHostBuffer,
    PJRT_Client_BufferFromHostBuffer_Args
);

pjrt_api_fn_ret_err!(PJRT_DeviceDescription_Id, PJRT_DeviceDescription_Id_Args);
pjrt_api_fn_ret_err!(
    PJRT_DeviceDescription_ProcessIndex,
    PJRT_DeviceDescription_ProcessIndex_Args
);
pjrt_api_fn_ret_err!(
    PJRT_DeviceDescription_Attributes,
    PJRT_DeviceDescription_Attributes_Args
);
pjrt_api_fn_ret_err!(
    PJRT_DeviceDescription_Kind,
    PJRT_DeviceDescription_Kind_Args
);
pjrt_api_fn_ret_err!(
    PJRT_DeviceDescription_DebugString,
    PJRT_DeviceDescription_DebugString_Args
);
pjrt_api_fn_ret_err!(
    PJRT_DeviceDescription_ToString,
    PJRT_DeviceDescription_ToString_Args
);

pjrt_api_fn_ret_err!(PJRT_Device_GetDescription, PJRT_Device_GetDescription_Args);
pjrt_api_fn_ret_err!(PJRT_Device_IsAddressable, PJRT_Device_IsAddressable_Args);
pjrt_api_fn_ret_err!(
    PJRT_Device_LocalHardwareId,
    PJRT_Device_LocalHardwareId_Args
);
pjrt_api_fn_ret_err!(
    PJRT_Device_AddressableMemories,
    PJRT_Device_AddressableMemories_Args
);
pjrt_api_fn_ret_err!(PJRT_Device_DefaultMemory, PJRT_Device_DefaultMemory_Args);
pjrt_api_fn_ret_err!(PJRT_Device_MemoryStats, PJRT_Device_MemoryStats_Args);

pjrt_api_fn_ret_err!(PJRT_Memory_Id, PJRT_Memory_Id_Args);
pjrt_api_fn_ret_err!(PJRT_Memory_Kind, PJRT_Memory_Kind_Args);
pjrt_api_fn_ret_err!(PJRT_Memory_DebugString, PJRT_Memory_DebugString_Args);
pjrt_api_fn_ret_err!(PJRT_Memory_ToString, PJRT_Memory_ToString_Args);
pjrt_api_fn_ret_err!(
    PJRT_Memory_AddressableByDevices,
    PJRT_Memory_AddressableByDevices_Args
);

pjrt_api_fn_ret_err!(PJRT_Executable_Destroy, PJRT_Executable_Destroy_Args);
pjrt_api_fn_ret_err!(PJRT_Executable_Name, PJRT_Executable_Name_Args);
pjrt_api_fn_ret_err!(
    PJRT_Executable_NumReplicas,
    PJRT_Executable_NumReplicas_Args
);
pjrt_api_fn_ret_err!(
    PJRT_Executable_NumPartitions,
    PJRT_Executable_NumPartitions_Args
);
pjrt_api_fn_ret_err!(PJRT_Executable_NumOutputs, PJRT_Executable_NumOutputs_Args);
pjrt_api_fn_ret_err!(
    PJRT_Executable_SizeOfGeneratedCodeInBytes,
    PJRT_Executable_SizeOfGeneratedCodeInBytes_Args
);
pjrt_api_fn_ret_err!(
    PJRT_Executable_GetCostAnalysis,
    PJRT_Executable_GetCostAnalysis_Args
);
pjrt_api_fn_ret_err!(
    PJRT_Executable_OutputMemoryKinds,
    PJRT_Executable_OutputMemoryKinds_Args
);
pjrt_api_fn_ret_err!(
    PJRT_Executable_OptimizedProgram,
    PJRT_Executable_OptimizedProgram_Args
);
pjrt_api_fn_ret_err!(PJRT_Executable_Serialize, PJRT_Executable_Serialize_Args);

pjrt_api_fn_ret_err!(
    PJRT_LoadedExecutable_Destroy,
    PJRT_LoadedExecutable_Destroy_Args
);
pjrt_api_fn_ret_err!(
    PJRT_LoadedExecutable_GetExecutable,
    PJRT_LoadedExecutable_GetExecutable_Args
);
pjrt_api_fn_ret_err!(
    PJRT_LoadedExecutable_AddressableDevices,
    PJRT_LoadedExecutable_AddressableDevices_Args
);
pjrt_api_fn_ret_err!(
    PJRT_LoadedExecutable_Delete,
    PJRT_LoadedExecutable_Delete_Args
);
pjrt_api_fn_ret_err!(
    PJRT_LoadedExecutable_IsDeleted,
    PJRT_LoadedExecutable_IsDeleted_Args
);
pjrt_api_fn_ret_err!(
    PJRT_LoadedExecutable_Execute,
    PJRT_LoadedExecutable_Execute_Args
);
pjrt_api_fn_ret_err!(
    PJRT_Executable_DeserializeAndLoad,
    PJRT_Executable_DeserializeAndLoad_Args
);
pjrt_api_fn_ret_err!(
    PJRT_LoadedExecutable_Fingerprint,
    PJRT_LoadedExecutable_Fingerprint_Args
);

pjrt_api_fn_ret_err!(PJRT_Buffer_Destroy, PJRT_Buffer_Destroy_Args);
pjrt_api_fn_ret_err!(PJRT_Buffer_ElementType, PJRT_Buffer_ElementType_Args);
pjrt_api_fn_ret_err!(PJRT_Buffer_Dimensions, PJRT_Buffer_Dimensions_Args);
pjrt_api_fn_ret_err!(
    PJRT_Buffer_UnpaddedDimensions,
    PJRT_Buffer_UnpaddedDimensions_Args
);
pjrt_api_fn_ret_err!(
    PJRT_Buffer_DynamicDimensionIndices,
    PJRT_Buffer_DynamicDimensionIndices_Args
);
pjrt_api_fn_ret_err!(
    PJRT_Buffer_GetMemoryLayout,
    PJRT_Buffer_GetMemoryLayout_Args
);
pjrt_api_fn_ret_err!(
    PJRT_Buffer_OnDeviceSizeInBytes,
    PJRT_Buffer_OnDeviceSizeInBytes_Args
);
pjrt_api_fn_ret_err!(PJRT_Buffer_Device, PJRT_Buffer_Device_Args);
pjrt_api_fn_ret_err!(PJRT_Buffer_Memory, PJRT_Buffer_Memory_Args);
pjrt_api_fn_ret_err!(PJRT_Buffer_Delete, PJRT_Buffer_Delete_Args);
pjrt_api_fn_ret_err!(PJRT_Buffer_IsDeleted, PJRT_Buffer_IsDeleted_Args);
pjrt_api_fn_ret_err!(PJRT_Buffer_CopyToDevice, PJRT_Buffer_CopyToDevice_Args);
pjrt_api_fn_ret_err!(PJRT_Buffer_ToHostBuffer, PJRT_Buffer_ToHostBuffer_Args);
pjrt_api_fn_ret_err!(PJRT_Buffer_IsOnCpu, PJRT_Buffer_IsOnCpu_Args);
pjrt_api_fn_ret_err!(PJRT_Buffer_ReadyEvent, PJRT_Buffer_ReadyEvent_Args);
pjrt_api_fn_ret_err!(PJRT_Buffer_UnsafePointer, PJRT_Buffer_UnsafePointer_Args);
pjrt_api_fn_ret_err!(
    PJRT_Buffer_IncreaseExternalReferenceCount,
    PJRT_Buffer_IncreaseExternalReferenceCount_Args
);
pjrt_api_fn_ret_err!(
    PJRT_Buffer_DecreaseExternalReferenceCount,
    PJRT_Buffer_DecreaseExternalReferenceCount_Args
);
pjrt_api_fn_ret_err!(
    PJRT_Buffer_OpaqueDeviceMemoryDataPointer,
    PJRT_Buffer_OpaqueDeviceMemoryDataPointer_Args
);

pjrt_api_fn_ret_err!(
    PJRT_CopyToDeviceStream_Destroy,
    PJRT_CopyToDeviceStream_Destroy_Args
);
pjrt_api_fn_ret_err!(
    PJRT_CopyToDeviceStream_AddChunk,
    PJRT_CopyToDeviceStream_AddChunk_Args
);
pjrt_api_fn_ret_err!(
    PJRT_CopyToDeviceStream_TotalBytes,
    PJRT_CopyToDeviceStream_TotalBytes_Args
);
pjrt_api_fn_ret_err!(
    PJRT_CopyToDeviceStream_GranuleSize,
    PJRT_CopyToDeviceStream_GranuleSize_Args
);
pjrt_api_fn_ret_err!(
    PJRT_CopyToDeviceStream_CurrentBytes,
    PJRT_CopyToDeviceStream_CurrentBytes_Args
);

pjrt_api_fn_ret_err!(
    PJRT_TopologyDescription_Create,
    PJRT_TopologyDescription_Create_Args
);
pjrt_api_fn_ret_err!(
    PJRT_TopologyDescription_Destroy,
    PJRT_TopologyDescription_Destroy_Args
);
pjrt_api_fn_ret_err!(
    PJRT_TopologyDescription_PlatformName,
    PJRT_TopologyDescription_PlatformName_Args
);
pjrt_api_fn_ret_err!(
    PJRT_TopologyDescription_PlatformVersion,
    PJRT_TopologyDescription_PlatformVersion_Args
);
pjrt_api_fn_ret_err!(
    PJRT_TopologyDescription_GetDeviceDescriptions,
    PJRT_TopologyDescription_GetDeviceDescriptions_Args
);
pjrt_api_fn_ret_err!(
    PJRT_TopologyDescription_Serialize,
    PJRT_TopologyDescription_Serialize_Args
);
pjrt_api_fn_ret_err!(
    PJRT_TopologyDescription_Attributes,
    PJRT_TopologyDescription_Attributes_Args
);

pjrt_api_fn_ret_err!(PJRT_Compile, PJRT_Compile_Args);

pjrt_api_fn_ret_err!(
    PJRT_Executable_OutputElementTypes,
    PJRT_Executable_OutputElementTypes_Args
);
pjrt_api_fn_ret_err!(
    PJRT_Executable_OutputDimensions,
    PJRT_Executable_OutputDimensions_Args
);

pjrt_api_fn_ret_err!(PJRT_Buffer_CopyToMemory, PJRT_Buffer_CopyToMemory_Args);

pjrt_api_fn_ret_err!(
    PJRT_Client_CreateViewOfDeviceBuffer,
    PJRT_Client_CreateViewOfDeviceBuffer_Args
);

pjrt_api_fn_ret_err!(
    PJRT_Executable_Fingerprint,
    PJRT_Executable_Fingerprint_Args
);

pjrt_api_fn_ret_err!(
    PJRT_Client_TopologyDescription,
    PJRT_Client_TopologyDescription_Args
);

pjrt_api_fn_ret_err!(
    PJRT_Executable_GetCompiledMemoryStats,
    PJRT_Executable_GetCompiledMemoryStats_Args
);

pjrt_api_fn_ret_err!(PJRT_Memory_Kind_Id, PJRT_Memory_Kind_Id_Args);

pjrt_api_fn_ret_err!(PJRT_ExecuteContext_Create, PJRT_ExecuteContext_Create_Args);
pjrt_api_fn_ret_err!(
    PJRT_ExecuteContext_Destroy,
    PJRT_ExecuteContext_Destroy_Args
);

// New APIs for XLA commit 68069613f91e354a1fc5a2e235da2ba44670e612

// Event Create and Set
pjrt_api_fn_ret_err!(PJRT_Event_Create, PJRT_Event_Create_Args);
pjrt_api_fn_ret_err!(PJRT_Event_Set, PJRT_Event_Set_Args);

// AsyncHostToDeviceTransferManager
pjrt_api_fn_ret_err!(
    PJRT_AsyncHostToDeviceTransferManager_Destroy,
    PJRT_AsyncHostToDeviceTransferManager_Destroy_Args
);
pjrt_api_fn_ret_err!(
    PJRT_AsyncHostToDeviceTransferManager_TransferData,
    PJRT_AsyncHostToDeviceTransferManager_TransferData_Args
);
pjrt_api_fn_ret_err!(
    PJRT_AsyncHostToDeviceTransferManager_RetrieveBuffer,
    PJRT_AsyncHostToDeviceTransferManager_RetrieveBuffer_Args
);
pjrt_api_fn_ret_err!(
    PJRT_AsyncHostToDeviceTransferManager_Device,
    PJRT_AsyncHostToDeviceTransferManager_Device_Args
);
pjrt_api_fn_ret_err!(
    PJRT_AsyncHostToDeviceTransferManager_BufferCount,
    PJRT_AsyncHostToDeviceTransferManager_BufferCount_Args
);
pjrt_api_fn_ret_err!(
    PJRT_AsyncHostToDeviceTransferManager_BufferSize,
    PJRT_AsyncHostToDeviceTransferManager_BufferSize_Args
);
pjrt_api_fn_ret_err!(
    PJRT_AsyncHostToDeviceTransferManager_SetBufferError,
    PJRT_AsyncHostToDeviceTransferManager_SetBufferError_Args
);
pjrt_api_fn_ret_err!(
    PJRT_AsyncHostToDeviceTransferManager_AddMetadata,
    PJRT_AsyncHostToDeviceTransferManager_AddMetadata_Args
);
pjrt_api_fn_ret_err!(
    PJRT_AsyncHostToDeviceTransferManager_TransferLiteral,
    PJRT_AsyncHostToDeviceTransferManager_TransferLiteral_Args
);

// Buffer CopyRawToHost
pjrt_api_fn_ret_err!(PJRT_Buffer_CopyRawToHost, PJRT_Buffer_CopyRawToHost_Args);
pjrt_api_fn_ret_err!(
    PJRT_Buffer_CopyRawToHostFuture,
    PJRT_Buffer_CopyRawToHostFuture_Args
);

// Buffer DonateWithControlDependency
pjrt_api_fn_ret_err!(
    PJRT_Buffer_DonateWithControlDependency,
    PJRT_Buffer_DonateWithControlDependency_Args
);

// Client CreateBuffersForAsyncHostToDevice
pjrt_api_fn_ret_err!(
    PJRT_Client_CreateBuffersForAsyncHostToDevice,
    PJRT_Client_CreateBuffersForAsyncHostToDevice_Args
);
pjrt_api_fn_ret_err!(
    PJRT_Client_CreateUninitializedBuffer,
    PJRT_Client_CreateUninitializedBuffer_Args
);
pjrt_api_fn_ret_err!(
    PJRT_Client_CreateErrorBuffer,
    PJRT_Client_CreateErrorBuffer_Args
);
pjrt_api_fn_ret_err!(
    PJRT_Client_CreateAliasBuffer,
    PJRT_Client_CreateAliasBuffer_Args
);
pjrt_api_fn_ret_err!(
    PJRT_Client_FulfillAliasBuffer,
    PJRT_Client_FulfillAliasBuffer_Args
);
pjrt_api_fn_ret_err!(
    PJRT_Client_UpdateGlobalProcessInfo,
    PJRT_Client_UpdateGlobalProcessInfo_Args
);
pjrt_api_fn_ret_err!(PJRT_Client_DmaMap, PJRT_Client_DmaMap_Args);
pjrt_api_fn_ret_err!(PJRT_Client_DmaUnmap, PJRT_Client_DmaUnmap_Args);

// Device CreateAsyncTrackingEvent and PoisonExecution
pjrt_api_fn_ret_err!(
    PJRT_Device_CreateAsyncTrackingEvent,
    PJRT_Device_CreateAsyncTrackingEvent_Args
);
pjrt_api_fn_ret_err!(
    PJRT_Device_PoisonExecution,
    PJRT_Device_PoisonExecution_Args
);

// TopologyDescription Deserialize
pjrt_api_fn_ret_err!(
    PJRT_TopologyDescription_Deserialize,
    PJRT_TopologyDescription_Deserialize_Args
);

// Executable GetCompileOptions and GetDeviceAssignment
pjrt_api_fn_ret_err!(
    PJRT_Executable_GetCompileOptions,
    PJRT_Executable_GetCompileOptions_Args
);
pjrt_api_fn_ret_err!(
    PJRT_LoadedExecutable_GetDeviceAssignment,
    PJRT_LoadedExecutable_GetDeviceAssignment_Args
);

// AsyncTrackingEvent Destroy
pjrt_api_fn_ret_err!(
    PJRT_AsyncTrackingEvent_Destroy,
    PJRT_AsyncTrackingEvent_Destroy_Args
);

// Note: PJRT_KeyValueTryGetCallback is a callback field in PJRT_Client_Create_Args,
// not a standalone API function. It's used when creating a client with KV store support.
