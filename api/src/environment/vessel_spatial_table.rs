use crate::utils::utils::Float;
use crate::vessel::{Vessel, VesselId};
use burbomath::Point;
use std::collections::BTreeMap;
use std::convert::Infallible;

/// Read-only snapshot of vessel positions and docking capacity.
/// Built once per environment iteration, so it may lag behind by one iteration.
#[derive(Debug, Default)]
pub struct VesselSpatialTable {
    records: BTreeMap<VesselId, VesselSpatialRecord>,
}

#[derive(Debug)]
pub struct VesselSpatialRecord {
    /// Position of the top-level vessel this one is (transitively) docked to, or its own position if not docked.
    pub pos: Point<Float>,
    pub parent: Option<VesselId>,
    /// compat type -> count of empty docking clamps
    pub free_clamps_by_compat_type: BTreeMap<usize, usize>,
}

impl VesselSpatialTable {
    pub fn build(vessels: &[Vessel]) -> Self {
        let mut records = BTreeMap::new();
        for root in vessels {
            let pos = root.pos();
            let _: Result<(), Infallible> = root.traverse(|path, vessel| {
                let mut free_clamps_by_compat_type: BTreeMap<usize, usize> = BTreeMap::new();
                for module in vessel.modules() {
                    for clamp in module.docking_clamps() {
                        if clamp.is_empty() {
                            *free_clamps_by_compat_type
                                .entry(clamp.compat_type())
                                .or_default() += 1;
                        }
                    }
                }
                records.insert(
                    vessel.id(),
                    VesselSpatialRecord {
                        pos,
                        parent: path.parent(),
                        free_clamps_by_compat_type,
                    },
                );
                Ok(())
            });
        }
        Self { records }
    }

    pub fn get(&self, id: &VesselId) -> Option<&VesselSpatialRecord> {
        self.records.get(id)
    }
}

impl VesselSpatialRecord {
    pub fn free_clamps_count(&self, compat_type: usize) -> usize {
        self.free_clamps_by_compat_type
            .get(&compat_type)
            .copied()
            .unwrap_or(0)
    }
}
