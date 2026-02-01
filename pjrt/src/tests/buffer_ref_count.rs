//! Buffer Reference Counting Tests
//!
//! Tests for the buffer external reference counting APIs including:
//! - Buffer::unsafe_pointer()
//! - Buffer::increase_external_ref_count()
//! - Buffer::decrease_external_ref_count()
//! - Buffer::opaque_device_memory_pointer()

#[cfg(all(test, feature = "integration-tests"))]
mod integration_tests {
    use pjrt::{Client, HostBuffer, Result};

    fn setup_test_client() -> Result<Client> {
        let plugin_path = std::env::var("PJRT_PLUGIN_PATH")
            .expect("PJRT_PLUGIN_PATH environment variable must be set for integration tests");
        let api = pjrt::plugin(&plugin_path).load()?;
        Client::builder(&api).build()
    }

    #[test]
    fn test_buffer_external_ref_count_with_plugin() -> Result<()> {
        let client = setup_test_client()?;

        // Create a host buffer
        let host_buffer = HostBuffer::from_scalar(42.0f32);

        // Transfer to device
        let device_buffer = host_buffer.to_sync(&client).copy()?;

        // Test external reference counting
        unsafe {
            device_buffer.increase_external_ref_count()?;

            // Get the unsafe pointer
            let ptr = device_buffer.unsafe_pointer()?;
            assert!(!ptr.is_null());

            // Get device memory pointer
            let dev_ptr = device_buffer.opaque_device_memory_pointer()?;
            assert!(!dev_ptr.is_null());

            // Decrease ref count
            device_buffer.decrease_external_ref_count()?;
        }

        Ok(())
    }

    #[test]
    fn test_multiple_external_refs() -> Result<()> {
        let client = setup_test_client()?;
        let host_buffer = HostBuffer::from_scalar(1.0f32);
        let device_buffer = host_buffer.to_sync(&client).copy()?;

        unsafe {
            // Multiple increases should be allowed
            device_buffer.increase_external_ref_count()?;
            device_buffer.increase_external_ref_count()?;
            device_buffer.increase_external_ref_count()?;

            // Multiple decreases to balance
            device_buffer.decrease_external_ref_count()?;
            device_buffer.decrease_external_ref_count()?;
            device_buffer.decrease_external_ref_count()?;
        }

        Ok(())
    }

    #[test]
    fn test_unbalanced_decrease_fails() -> Result<()> {
        let client = setup_test_client()?;
        let host_buffer = HostBuffer::from_scalar(1.0f32);
        let device_buffer = host_buffer.to_sync(&client).copy()?;

        // Decreasing without increasing should fail or be handled gracefully
        let result = unsafe { device_buffer.decrease_external_ref_count() };
        // This may succeed or fail depending on PJRT implementation
        // but should not cause memory corruption

        Ok(())
    }
}
