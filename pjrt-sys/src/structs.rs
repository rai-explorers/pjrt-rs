macro_rules! impl_new {
    ($t:ident, $s:ident) => {
        impl $crate::$t {
            pub const STRUCT_SIZE: usize = $crate::$s as usize;

            pub fn new() -> Self {
                let mut t = $crate::$t::default();
                t.struct_size = Self::STRUCT_SIZE;
                t
            }
        }
    };
    ($t:ident) => {
        impl $crate::$t {
            pub fn new() -> Self {
                $crate::$t::default()
            }
        }
    };
}

impl_new!(PJRT_Extension_Base, PJRT_Extension_Base_STRUCT_SIZE);

impl_new!(PJRT_Api_Version, PJRT_Api_Version_STRUCT_SIZE);

impl_new!(PJRT_Error_Destroy_Args, PJRT_Error_Destroy_Args_STRUCT_SIZE);

impl_new!(PJRT_Error_Message_Args, PJRT_Error_Message_Args_STRUCT_SIZE);

impl_new!(PJRT_Error_GetCode_Args, PJRT_Error_GetCode_Args_STRUCT_SIZE);

impl_new!(PJRT_NamedValue, PJRT_NamedValue_STRUCT_SIZE);

impl_new!(
    PJRT_Plugin_Initialize_Args,
    PJRT_Plugin_Initialize_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Plugin_Attributes_Args,
    PJRT_Plugin_Attributes_Args_STRUCT_SIZE
);

impl_new!(PJRT_Event_Destroy_Args, PJRT_Event_Destroy_Args_STRUCT_SIZE);

impl_new!(PJRT_Event_IsReady_Args, PJRT_Event_IsReady_Args_STRUCT_SIZE);

impl_new!(PJRT_Event_Error_Args, PJRT_Event_Error_Args_STRUCT_SIZE);

impl_new!(PJRT_Event_Await_Args, PJRT_Event_Await_Args_STRUCT_SIZE);

impl_new!(PJRT_Event_OnReady_Args, PJRT_Event_OnReady_Args_STRUCT_SIZE);

impl_new!(
    PJRT_KeyValueGetCallback_Args,
    PJRT_KeyValueGetCallback_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_KeyValuePutCallback_Args,
    PJRT_KeyValuePutCallback_Args_STRUCT_SIZE
);

impl_new!(PJRT_Client_Create_Args, PJRT_Client_Create_Args_STRUCT_SIZE);

impl_new!(
    PJRT_Client_Destroy_Args,
    PJRT_Client_Destroy_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Client_PlatformName_Args,
    PJRT_Client_PlatformName_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Client_ProcessIndex_Args,
    PJRT_Client_ProcessIndex_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Client_PlatformVersion_Args,
    PJRT_Client_PlatformVersion_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Client_TopologyDescription_Args,
    PJRT_Client_TopologyDescription_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Client_Devices_Args,
    PJRT_Client_Devices_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Client_AddressableDevices_Args,
    PJRT_Client_AddressableDevices_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Client_LookupDevice_Args,
    PJRT_Client_LookupDevice_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Client_LookupAddressableDevice_Args,
    PJRT_Client_LookupAddressableDevice_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Client_AddressableMemories_Args,
    PJRT_Client_AddressableMemories_Args_STRUCT_SIZE
);

impl_new!(PJRT_Program, PJRT_Program_STRUCT_SIZE);

impl_new!(
    PJRT_Client_Compile_Args,
    PJRT_Client_Compile_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Client_DefaultDeviceAssignment_Args,
    PJRT_Client_DefaultDeviceAssignment_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Buffer_MemoryLayout_Tiled,
    PJRT_Buffer_MemoryLayout_Tiled_STRUCT_SIZE
);

impl_new!(
    PJRT_Buffer_MemoryLayout_Strides,
    PJRT_Buffer_MemoryLayout_Strides_STRUCT_SIZE
);

impl_new!(
    PJRT_Buffer_MemoryLayout,
    PJRT_Buffer_MemoryLayout_STRUCT_SIZE
);

impl_new!(
    PJRT_Client_BufferFromHostBuffer_Args,
    PJRT_Client_BufferFromHostBuffer_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Client_CreateViewOfDeviceBuffer_Args,
    PJRT_Client_CreateViewOfDeviceBuffer_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_DeviceDescription_Id_Args,
    PJRT_DeviceDescription_Id_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_DeviceDescription_ProcessIndex_Args,
    PJRT_DeviceDescription_ProcessIndex_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_DeviceDescription_Attributes_Args,
    PJRT_DeviceDescription_Attributes_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_DeviceDescription_Kind_Args,
    PJRT_DeviceDescription_Kind_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_DeviceDescription_DebugString_Args,
    PJRT_DeviceDescription_DebugString_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_DeviceDescription_ToString_Args,
    PJRT_DeviceDescription_ToString_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Device_GetDescription_Args,
    PJRT_Device_GetDescription_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Device_IsAddressable_Args,
    PJRT_Device_IsAddressable_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Device_LocalHardwareId_Args,
    PJRT_Device_LocalHardwareId_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Device_AddressableMemories_Args,
    PJRT_Device_AddressableMemories_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Device_DefaultMemory_Args,
    PJRT_Device_DefaultMemory_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Device_MemoryStats_Args,
    PJRT_Device_MemoryStats_Args_STRUCT_SIZE
);

impl_new!(PJRT_Memory_Id_Args, PJRT_Memory_Id_Args_STRUCT_SIZE);

impl_new!(PJRT_Memory_Kind_Args, PJRT_Memory_Kind_Args_STRUCT_SIZE);

impl_new!(
    PJRT_Memory_Kind_Id_Args,
    PJRT_Memory_Kind_Id_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Memory_DebugString_Args,
    PJRT_Memory_DebugString_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Memory_ToString_Args,
    PJRT_Memory_ToString_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Memory_AddressableByDevices_Args,
    PJRT_Memory_AddressableByDevices_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_ExecuteContext_Create_Args,
    PJRT_ExecuteContext_Create_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_ExecuteContext_Destroy_Args,
    PJRT_ExecuteContext_Destroy_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Executable_Destroy_Args,
    PJRT_Executable_Destroy_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_LoadedExecutable_Destroy_Args,
    PJRT_LoadedExecutable_Destroy_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_LoadedExecutable_GetExecutable_Args,
    PJRT_LoadedExecutable_GetExecutable_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Executable_Name_Args,
    PJRT_Executable_Name_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Executable_NumReplicas_Args,
    PJRT_Executable_NumReplicas_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Executable_NumPartitions_Args,
    PJRT_Executable_NumPartitions_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_LoadedExecutable_AddressableDevices_Args,
    PJRT_LoadedExecutable_AddressableDevices_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Executable_OptimizedProgram_Args,
    PJRT_Executable_OptimizedProgram_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_LoadedExecutable_Delete_Args,
    PJRT_LoadedExecutable_Delete_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_LoadedExecutable_IsDeleted_Args,
    PJRT_LoadedExecutable_IsDeleted_Args_STRUCT_SIZE
);

impl_new!(PJRT_ExecuteOptions, PJRT_ExecuteOptions_STRUCT_SIZE);

impl_new!(
    PJRT_LoadedExecutable_Execute_Args,
    PJRT_LoadedExecutable_Execute_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Executable_NumOutputs_Args,
    PJRT_Executable_NumOutputs_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Executable_SizeOfGeneratedCodeInBytes_Args,
    PJRT_Executable_SizeOfGeneratedCodeInBytes_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Executable_Fingerprint_Args,
    PJRT_Executable_Fingerprint_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Executable_GetCostAnalysis_Args,
    PJRT_Executable_GetCostAnalysis_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Executable_GetCompiledMemoryStats_Args,
    PJRT_Executable_GetCompiledMemoryStats_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Executable_OutputElementTypes_Args,
    PJRT_Executable_OutputElementTypes_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Executable_OutputDimensions_Args,
    PJRT_Executable_OutputDimensions_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Executable_OutputMemoryKinds_Args,
    PJRT_Executable_OutputMemoryKinds_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Executable_Serialize_Args,
    PJRT_Executable_Serialize_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Executable_DeserializeAndLoad_Args,
    PJRT_Executable_DeserializeAndLoad_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_LoadedExecutable_Fingerprint_Args,
    PJRT_LoadedExecutable_Fingerprint_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Buffer_Destroy_Args,
    PJRT_Buffer_Destroy_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Buffer_ElementType_Args,
    PJRT_Buffer_ElementType_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Buffer_Dimensions_Args,
    PJRT_Buffer_Dimensions_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Buffer_UnpaddedDimensions_Args,
    PJRT_Buffer_UnpaddedDimensions_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Buffer_DynamicDimensionIndices_Args,
    PJRT_Buffer_DynamicDimensionIndices_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Buffer_GetMemoryLayout_Args,
    PJRT_Buffer_GetMemoryLayout_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Buffer_ToHostBuffer_Args,
    PJRT_Buffer_ToHostBuffer_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Buffer_OnDeviceSizeInBytes_Args,
    PJRT_Buffer_OnDeviceSizeInBytes_Args_STRUCT_SIZE
);

impl_new!(PJRT_Buffer_Delete_Args, PJRT_Buffer_Delete_Args_STRUCT_SIZE);

impl_new!(
    PJRT_Buffer_IsDeleted_Args,
    PJRT_Buffer_IsDeleted_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Buffer_CopyToDevice_Args,
    PJRT_Buffer_CopyToDevice_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Buffer_CopyToMemory_Args,
    PJRT_Buffer_CopyToMemory_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Buffer_IsOnCpu_Args,
    PJRT_Buffer_IsOnCpu_Args_STRUCT_SIZE
);

impl_new!(PJRT_Buffer_Device_Args, PJRT_Buffer_Device_Args_STRUCT_SIZE);

impl_new!(PJRT_Buffer_Memory_Args, PJRT_Buffer_Memory_Args_STRUCT_SIZE);

impl_new!(
    PJRT_Buffer_ReadyEvent_Args,
    PJRT_Buffer_ReadyEvent_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Buffer_UnsafePointer_Args,
    PJRT_Buffer_UnsafePointer_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Buffer_IncreaseExternalReferenceCount_Args,
    PJRT_Buffer_IncreaseExternalReferenceCount_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Buffer_DecreaseExternalReferenceCount_Args,
    PJRT_Buffer_DecreaseExternalReferenceCount_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Buffer_OpaqueDeviceMemoryDataPointer_Args,
    PJRT_Buffer_OpaqueDeviceMemoryDataPointer_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_CopyToDeviceStream_Destroy_Args,
    PJRT_CopyToDeviceStream_Destroy_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_CopyToDeviceStream_AddChunk_Args,
    PJRT_CopyToDeviceStream_AddChunk_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_CopyToDeviceStream_TotalBytes_Args,
    PJRT_CopyToDeviceStream_TotalBytes_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_CopyToDeviceStream_GranuleSize_Args,
    PJRT_CopyToDeviceStream_GranuleSize_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_CopyToDeviceStream_CurrentBytes_Args,
    PJRT_CopyToDeviceStream_CurrentBytes_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_TopologyDescription_Create_Args,
    PJRT_TopologyDescription_Create_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_TopologyDescription_Destroy_Args,
    PJRT_TopologyDescription_Destroy_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_TopologyDescription_PlatformVersion_Args,
    PJRT_TopologyDescription_PlatformVersion_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_TopologyDescription_PlatformName_Args,
    PJRT_TopologyDescription_PlatformName_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_TopologyDescription_GetDeviceDescriptions_Args,
    PJRT_TopologyDescription_GetDeviceDescriptions_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_TopologyDescription_Serialize_Args,
    PJRT_TopologyDescription_Serialize_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_TopologyDescription_Attributes_Args,
    PJRT_TopologyDescription_Attributes_Args_STRUCT_SIZE
);

impl_new!(PJRT_Compile_Args, PJRT_Compile_Args_STRUCT_SIZE);

// AsyncHostToDeviceTransferManager structs
impl_new!(
    PJRT_AsyncHostToDeviceTransferManager_Destroy_Args,
    PJRT_AsyncHostToDeviceTransferManager_Destroy_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_AsyncHostToDeviceTransferManager_TransferData_Args,
    PJRT_AsyncHostToDeviceTransferManager_TransferData_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_AsyncHostToDeviceTransferManager_RetrieveBuffer_Args,
    PJRT_AsyncHostToDeviceTransferManager_RetrieveBuffer_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_AsyncHostToDeviceTransferManager_Device_Args,
    PJRT_AsyncHostToDeviceTransferManager_Device_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_AsyncHostToDeviceTransferManager_BufferCount_Args,
    PJRT_AsyncHostToDeviceTransferManager_BufferCount_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_AsyncHostToDeviceTransferManager_BufferSize_Args,
    PJRT_AsyncHostToDeviceTransferManager_BufferSize_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_AsyncHostToDeviceTransferManager_SetBufferError_Args,
    PJRT_AsyncHostToDeviceTransferManager_SetBufferError_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_AsyncHostToDeviceTransferManager_AddMetadata_Args,
    PJRT_AsyncHostToDeviceTransferManager_AddMetadata_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_AsyncHostToDeviceTransferManager_TransferLiteral_Args,
    PJRT_AsyncHostToDeviceTransferManager_TransferLiteral_Args_STRUCT_SIZE
);

// Buffer CopyRawToHost structs
impl_new!(
    PJRT_Buffer_CopyRawToHost_Args,
    PJRT_Buffer_CopyRawToHost_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Buffer_CopyRawToHostFuture_Args,
    PJRT_Buffer_CopyRawToHostFuture_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Buffer_CopyRawToHostFuture_Callback_Args,
    PJRT_Buffer_CopyRawToHostFuture_Callback_Args_STRUCT_SIZE
);

// Buffer DonateWithControlDependency structs
impl_new!(
    PJRT_Buffer_DonateWithControlDependency_Args,
    PJRT_Buffer_DonateWithControlDependency_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Buffer_DonateWithControlDependency_Callback_Args,
    PJRT_Buffer_DonateWithControlDependency_Callback_Args_STRUCT_SIZE
);

// Client new structs
impl_new!(
    PJRT_Client_CreateBuffersForAsyncHostToDevice_Args,
    PJRT_Client_CreateBuffersForAsyncHostToDevice_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Client_CreateUninitializedBuffer_Args,
    PJRT_Client_CreateUninitializedBuffer_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Client_CreateErrorBuffer_Args,
    PJRT_Client_CreateErrorBuffer_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Client_CreateAliasBuffer_Args,
    PJRT_Client_CreateAliasBuffer_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Client_FulfillAliasBuffer_Args,
    PJRT_Client_FulfillAliasBuffer_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Client_UpdateGlobalProcessInfo_Args,
    PJRT_Client_UpdateGlobalProcessInfo_Args_STRUCT_SIZE
);

impl_new!(PJRT_Client_DmaMap_Args, PJRT_Client_DmaMap_Args_STRUCT_SIZE);

impl_new!(
    PJRT_Client_DmaUnmap_Args,
    PJRT_Client_DmaUnmap_Args_STRUCT_SIZE
);

// Device new structs
impl_new!(
    PJRT_Device_CreateAsyncTrackingEvent_Args,
    PJRT_Device_CreateAsyncTrackingEvent_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_Device_PoisonExecution_Args,
    PJRT_Device_PoisonExecution_Args_STRUCT_SIZE
);

// Event new structs
impl_new!(PJRT_Event_Create_Args, PJRT_Event_Create_Args_STRUCT_SIZE);

impl_new!(PJRT_Event_Set_Args, PJRT_Event_Set_Args_STRUCT_SIZE);

// Executable new structs
impl_new!(
    PJRT_Executable_GetCompileOptions_Args,
    PJRT_Executable_GetCompileOptions_Args_STRUCT_SIZE
);

impl_new!(
    PJRT_LoadedExecutable_GetDeviceAssignment_Args,
    PJRT_LoadedExecutable_GetDeviceAssignment_Args_STRUCT_SIZE
);

// KeyValueTryGetCallback
impl_new!(
    PJRT_KeyValueTryGetCallback_Args,
    PJRT_KeyValueTryGetCallback_Args_STRUCT_SIZE
);

// ProcessInfo
impl_new!(PJRT_ProcessInfo, PJRT_ProcessInfo_STRUCT_SIZE);

// Callback Info structs (no struct_size field, use simple form)
impl_new!(PJRT_RecvCallbackInfo);

impl_new!(PJRT_SendCallbackInfo);

// ShapeSpec
impl_new!(PJRT_ShapeSpec, PJRT_ShapeSpec_STRUCT_SIZE);

// TopologyDescription Deserialize
impl_new!(
    PJRT_TopologyDescription_Deserialize_Args,
    PJRT_TopologyDescription_Deserialize_Args_STRUCT_SIZE
);

// AsyncTrackingEvent
impl_new!(
    PJRT_AsyncTrackingEvent_Destroy_Args,
    PJRT_AsyncTrackingEvent_Destroy_Args_STRUCT_SIZE
);
