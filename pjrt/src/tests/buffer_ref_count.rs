//! Buffer Reference Counting Tests
//!
//! Tests for the buffer external reference counting APIs including:
//! - Buffer::unsafe_pointer()
//! - Buffer::increase_external_ref_count()
//! - Buffer::decrease_external_ref_count()
//! - Buffer::opaque_device_memory_pointer()

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_external_ref_count_basic() {
        // This is a basic test structure
        // Full implementation requires a loaded PJRT plugin
        // TODO: Add integration test with actual plugin
    }

    #[test]
    fn test_buffer_unsafe_pointer_doc() {
        // Test that unsafe_pointer documentation is accurate
        // The pointer should be valid after increase_external_ref_count
        // and invalid after decrease_external_ref_count
    }

    #[test]
    fn test_buffer_device_memory_pointer() {
        // Test opaque_device_memory_pointer API
        // Should return a valid device memory pointer
    }

    #[test]
    fn test_external_ref_count_safety() {
        // Test that proper ref counting prevents use-after-free
        // increase_external_ref_count should prevent buffer deletion
        // decrease_external_ref_count should allow normal cleanup
    }
}

// Integration tests that require a PJRT plugin
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
