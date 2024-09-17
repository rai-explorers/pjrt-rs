use pjrt_sys::{
    PJRT_Chunk, PJRT_CopyToDeviceStream, PJRT_CopyToDeviceStream_AddChunk_Args,
    PJRT_CopyToDeviceStream_CurrentBytes_Args, PJRT_CopyToDeviceStream_Destroy_Args,
    PJRT_CopyToDeviceStream_GranuleSize_Args, PJRT_CopyToDeviceStream_TotalBytes_Args,
};

use crate::{Api, Chunk, Event, Result};

pub struct CopyToDeviceStream {
    api: Api,
    pub(crate) ptr: *mut PJRT_CopyToDeviceStream,
}

impl Drop for CopyToDeviceStream {
    fn drop(&mut self) {
        let mut args = PJRT_CopyToDeviceStream_Destroy_Args::new();
        args.stream = self.ptr;
        self.api
            .PJRT_CopyToDeviceStream_Destroy(args)
            .expect("PJRT_CopyToDeviceStream_Destroy");
    }
}

impl CopyToDeviceStream {
    pub fn wrap(api: &Api, ptr: *mut PJRT_CopyToDeviceStream) -> Self {
        assert!(!ptr.is_null());
        Self {
            api: api.clone(),
            ptr,
        }
    }

    pub fn api(&self) -> &Api {
        &self.api
    }

    pub fn call_add_chunk(&self, chunk: Chunk) -> Result<PJRT_CopyToDeviceStream_AddChunk_Args> {
        let mut args = PJRT_CopyToDeviceStream_AddChunk_Args::new();
        let mut chunk: PJRT_Chunk = chunk.into();
        args.stream = self.ptr;
        args.chunk = &mut chunk as *mut _;
        self.api.PJRT_CopyToDeviceStream_AddChunk(args)
    }

    pub fn add_chunk_sync(&self, chunk: Chunk) -> Result<()> {
        let args = self.call_add_chunk(chunk)?;
        let event = Event::wrap(&self.api, args.transfer_complete);
        event.wait()?;
        Ok(())
    }

    pub async fn add_chunk(&self, chunk: Chunk) -> Result<()> {
        let args = self.call_add_chunk(chunk)?;
        let event = Event::wrap(&self.api, args.transfer_complete);
        event.await?;
        Ok(())
    }

    pub fn total_bytes(&self) -> i64 {
        let mut args = PJRT_CopyToDeviceStream_TotalBytes_Args::new();
        args.stream = self.ptr;
        args = self
            .api
            .PJRT_CopyToDeviceStream_TotalBytes(args)
            .expect("PJRT_CopyToDeviceStream_TotalBytes");
        args.total_bytes
    }

    pub fn granule_size(&self) -> i64 {
        let mut args = PJRT_CopyToDeviceStream_GranuleSize_Args::new();
        args.stream = self.ptr;
        args = self
            .api
            .PJRT_CopyToDeviceStream_GranuleSize(args)
            .expect("PJRT_CopyToDeviceStream_GranuleSize");
        args.granule_size_in_bytes
    }

    pub fn current_bytes(&self) -> i64 {
        let mut args = PJRT_CopyToDeviceStream_CurrentBytes_Args::new();
        args.stream = self.ptr;
        args = self
            .api
            .PJRT_CopyToDeviceStream_CurrentBytes(args)
            .expect("PJRT_CopyToDeviceStream_CurrentBytes");
        args.current_bytes
    }
}
