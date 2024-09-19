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
