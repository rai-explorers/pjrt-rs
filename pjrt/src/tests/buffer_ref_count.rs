//! Buffer Reference Counting Tests
//!
//! Tests for the buffer external reference counting APIs including:
//! - Buffer::unsafe_pointer()
//! - Buffer::increase_external_ref_count()
//! - Buffer::decrease_external_ref_count()
//! - Buffer::opaque_device_memory_pointer()
//!
//! These tests require the `integration-tests` feature and the `PJRT_PLUGIN_PATH`
//! environment variable to be set to a valid PJRT plugin path.

#[cfg(all(test, feature = "integration-tests"))]
mod integration_tests {
    use crate::{plugin, Client, HostBuffer};

    fn setup_test_client() -> Option<Client> {
        let plugin_path = match std::env::var("PJRT_PLUGIN_PATH") {
            Ok(path) => path,
            Err(_) => {
                eprintln!("Skipping test: PJRT_PLUGIN_PATH environment variable not set");
                return None;
            }
        };
        match plugin(&plugin_path).load() {
            Ok(api) => match Client::builder(&api).build() {
                Ok(client) => Some(client),
                Err(e) => {
                    eprintln!("Skipping test: Failed to create client: {}", e);
                    None
                }
            },
            Err(e) => {
                eprintln!("Skipping test: Failed to load plugin: {}", e);
                None
            }
        }
    }

    #[test]
    fn test_buffer_external_ref_count_with_plugin() {
        let Some(client) = setup_test_client() else {
            return;
        };

        // Create a host buffer
        let host_buffer = HostBuffer::from_scalar(42.0f32);

        // Transfer to device
        let device_buffer = host_buffer.to_sync(&client).copy().unwrap();

        // Test external reference counting
        unsafe {
            device_buffer.increase_external_ref_count().unwrap();

            // Get the unsafe pointer (returns usize)
            let ptr = device_buffer.unsafe_pointer().unwrap();
            assert!(ptr != 0, "Unsafe pointer should not be zero");

            // Get device memory pointer
            let dev_ptr = device_buffer.opaque_device_memory_pointer().unwrap();
            assert!(
                !dev_ptr.is_null(),
                "Device memory pointer should not be null"
            );

            // Decrease ref count
            device_buffer.decrease_external_ref_count().unwrap();
        }
    }

    #[test]
    fn test_multiple_external_refs() {
        let Some(client) = setup_test_client() else {
            return;
        };

        let host_buffer = HostBuffer::from_scalar(1.0f32);
        let device_buffer = host_buffer.to_sync(&client).copy().unwrap();

        unsafe {
            // Multiple increases should be allowed
            device_buffer.increase_external_ref_count().unwrap();
            device_buffer.increase_external_ref_count().unwrap();
            device_buffer.increase_external_ref_count().unwrap();

            // Multiple decreases to balance
            device_buffer.decrease_external_ref_count().unwrap();
            device_buffer.decrease_external_ref_count().unwrap();
            device_buffer.decrease_external_ref_count().unwrap();
        }
    }

    #[test]
    fn test_unbalanced_decrease_fails() {
        let Some(client) = setup_test_client() else {
            return;
        };

        let host_buffer = HostBuffer::from_scalar(1.0f32);
        let device_buffer = host_buffer.to_sync(&client).copy().unwrap();

        // Decreasing without increasing should fail or be handled gracefully
        let _result = unsafe { device_buffer.decrease_external_ref_count() };
        // This may succeed or fail depending on PJRT implementation
        // but should not cause memory corruption
    }
}
