use std::collections::HashMap;

use crate::{Error, GlobalDeviceId, Result};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LogicalId {
    pub replica_id: usize,
    pub partition_id: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeviceAssignment {
    num_replicas: usize,
    num_partitions: usize,
    assignments: Vec<Vec<GlobalDeviceId>>,
}

impl DeviceAssignment {
    pub fn new(
        num_replicas: usize,
        num_partitions: usize,
        assignments: Vec<GlobalDeviceId>,
    ) -> Self {
        assert_eq!(num_replicas * num_partitions, assignments.len());
        let mut assignments2d = Vec::with_capacity(num_replicas);
        for c in assignments.chunks_exact(num_partitions) {
            assignments2d.push(c.to_vec());
        }
        Self {
            num_replicas,
            num_partitions,
            assignments: assignments2d,
        }
    }

    pub fn num_replicas(&self) -> usize {
        self.num_replicas
    }

    pub fn num_partitions(&self) -> usize {
        self.num_partitions
    }

    pub fn lookup_logical_id(&self, global_device_id: GlobalDeviceId) -> Result<LogicalId> {
        for (replica, assignment) in self.assignments.iter().enumerate() {
            for (partition, id) in assignment.iter().enumerate() {
                if *id == global_device_id {
                    return Ok(LogicalId {
                        replica_id: replica,
                        partition_id: partition,
                    });
                }
            }
        }
        Err(Error::DeviceNotInDeviceAssignment(global_device_id))
    }

    pub fn get_lookup_map(&self) -> HashMap<GlobalDeviceId, LogicalId> {
        let mut map = HashMap::new();
        for (replica, assignment) in self.assignments.iter().enumerate() {
            for (partition, global_device_id) in assignment.iter().enumerate() {
                map.insert(
                    *global_device_id,
                    LogicalId {
                        replica_id: replica,
                        partition_id: partition,
                    },
                );
            }
        }
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logical_id_creation() {
        let id = LogicalId {
            replica_id: 0,
            partition_id: 1,
        };
        assert_eq!(id.replica_id, 0);
        assert_eq!(id.partition_id, 1);
    }

    #[test]
    fn test_logical_id_clone() {
        let id = LogicalId {
            replica_id: 2,
            partition_id: 3,
        };
        let cloned = id.clone();
        assert_eq!(id, cloned);
    }

    #[test]
    fn test_logical_id_equality() {
        let id1 = LogicalId {
            replica_id: 0,
            partition_id: 0,
        };
        let id2 = LogicalId {
            replica_id: 0,
            partition_id: 0,
        };
        let id3 = LogicalId {
            replica_id: 1,
            partition_id: 0,
        };

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_logical_id_ordering() {
        let id1 = LogicalId {
            replica_id: 0,
            partition_id: 0,
        };
        let id2 = LogicalId {
            replica_id: 0,
            partition_id: 1,
        };
        let id3 = LogicalId {
            replica_id: 1,
            partition_id: 0,
        };

        assert!(id1 < id2);
        assert!(id2 < id3);
        assert!(id1 < id3);
    }

    #[test]
    fn test_logical_id_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(LogicalId {
            replica_id: 0,
            partition_id: 0,
        });
        set.insert(LogicalId {
            replica_id: 0,
            partition_id: 1,
        });
        set.insert(LogicalId {
            replica_id: 0,
            partition_id: 0,
        }); // Duplicate

        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_logical_id_debug() {
        let id = LogicalId {
            replica_id: 5,
            partition_id: 10,
        };
        let debug = format!("{:?}", id);
        assert!(debug.contains("LogicalId"));
        assert!(debug.contains("5"));
        assert!(debug.contains("10"));
    }

    #[test]
    fn test_device_assignment_new_single_replica_single_partition() {
        // 1x1 device assignment
        let assignments = vec![0i32];
        let da = DeviceAssignment::new(1, 1, assignments);

        assert_eq!(da.num_replicas(), 1);
        assert_eq!(da.num_partitions(), 1);
    }

    #[test]
    fn test_device_assignment_new_multiple_replicas() {
        // 2x1 device assignment
        let assignments = vec![0i32, 1i32];
        let da = DeviceAssignment::new(2, 1, assignments);

        assert_eq!(da.num_replicas(), 2);
        assert_eq!(da.num_partitions(), 1);
    }

    #[test]
    fn test_device_assignment_new_multiple_partitions() {
        // 1x2 device assignment
        let assignments = vec![0i32, 1i32];
        let da = DeviceAssignment::new(1, 2, assignments);

        assert_eq!(da.num_replicas(), 1);
        assert_eq!(da.num_partitions(), 2);
    }

    #[test]
    fn test_device_assignment_new_2x2() {
        // 2x2 device assignment (4 devices)
        let assignments = vec![0i32, 1i32, 2i32, 3i32];
        let da = DeviceAssignment::new(2, 2, assignments);

        assert_eq!(da.num_replicas(), 2);
        assert_eq!(da.num_partitions(), 2);
    }

    #[test]
    fn test_device_assignment_lookup_logical_id() {
        // 2x2 grid:
        // Replica 0: [device 0, device 1]
        // Replica 1: [device 2, device 3]
        let assignments = vec![0i32, 1i32, 2i32, 3i32];
        let da = DeviceAssignment::new(2, 2, assignments);

        // Device 0 should be at replica 0, partition 0
        let logical_id = da.lookup_logical_id(0).unwrap();
        assert_eq!(logical_id.replica_id, 0);
        assert_eq!(logical_id.partition_id, 0);

        // Device 1 should be at replica 0, partition 1
        let logical_id = da.lookup_logical_id(1).unwrap();
        assert_eq!(logical_id.replica_id, 0);
        assert_eq!(logical_id.partition_id, 1);

        // Device 2 should be at replica 1, partition 0
        let logical_id = da.lookup_logical_id(2).unwrap();
        assert_eq!(logical_id.replica_id, 1);
        assert_eq!(logical_id.partition_id, 0);

        // Device 3 should be at replica 1, partition 1
        let logical_id = da.lookup_logical_id(3).unwrap();
        assert_eq!(logical_id.replica_id, 1);
        assert_eq!(logical_id.partition_id, 1);
    }

    #[test]
    fn test_device_assignment_lookup_not_found() {
        let assignments = vec![0i32, 1i32];
        let da = DeviceAssignment::new(1, 2, assignments);

        let result = da.lookup_logical_id(99);
        assert!(result.is_err());
        match result {
            Err(Error::DeviceNotInDeviceAssignment(id)) => assert_eq!(id, 99),
            _ => panic!("Expected DeviceNotInDeviceAssignment error"),
        }
    }

    #[test]
    fn test_device_assignment_get_lookup_map() {
        let assignments = vec![0i32, 1i32, 2i32, 3i32];
        let da = DeviceAssignment::new(2, 2, assignments);
        let map = da.get_lookup_map();

        assert_eq!(map.len(), 4);
        assert_eq!(
            map.get(&0),
            Some(&LogicalId {
                replica_id: 0,
                partition_id: 0
            })
        );
        assert_eq!(
            map.get(&1),
            Some(&LogicalId {
                replica_id: 0,
                partition_id: 1
            })
        );
        assert_eq!(
            map.get(&2),
            Some(&LogicalId {
                replica_id: 1,
                partition_id: 0
            })
        );
        assert_eq!(
            map.get(&3),
            Some(&LogicalId {
                replica_id: 1,
                partition_id: 1
            })
        );
    }

    #[test]
    fn test_device_assignment_clone() {
        let assignments = vec![0i32, 1i32];
        let da = DeviceAssignment::new(1, 2, assignments);
        let cloned = da.clone();

        assert_eq!(da.num_replicas(), cloned.num_replicas());
        assert_eq!(da.num_partitions(), cloned.num_partitions());
    }

    #[test]
    fn test_device_assignment_equality() {
        let da1 = DeviceAssignment::new(1, 2, vec![0i32, 1i32]);
        let da2 = DeviceAssignment::new(1, 2, vec![0i32, 1i32]);
        let da3 = DeviceAssignment::new(2, 1, vec![0i32, 1i32]);

        assert_eq!(da1, da2);
        assert_ne!(da1, da3);
    }

    #[test]
    fn test_device_assignment_debug() {
        let da = DeviceAssignment::new(1, 2, vec![0i32, 1i32]);
        let debug = format!("{:?}", da);
        assert!(debug.contains("DeviceAssignment"));
        assert!(debug.contains("1")); // num_replicas
        assert!(debug.contains("2")); // num_partitions
    }

    #[test]
    #[should_panic]
    fn test_device_assignment_new_wrong_length() {
        // Should panic because we expect 2*2=4 devices but only provide 3
        DeviceAssignment::new(2, 2, vec![0i32, 1i32, 2i32]);
    }
}
