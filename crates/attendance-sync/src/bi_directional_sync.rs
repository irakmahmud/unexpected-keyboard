use attendance_common::Result;
use attendance_database::DbPool;
use attendance_domain::{Employee, Holiday, Shift};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterDataSyncPackage {
    pub sync_version: u64,
    pub generated_at: String,
    pub source_node_id: String,
    pub employees: Vec<Employee>,
    pub shifts: Vec<Shift>,
    pub holidays: Vec<Holiday>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResultSummary {
    pub employees_updated: usize,
    pub shifts_updated: usize,
    pub holidays_updated: usize,
    pub completed_at: String,
    pub has_conflicts: bool,
}

pub struct BiDirectionalSyncManager {
    db: DbPool,
}

impl BiDirectionalSyncManager {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    /// Ingest and reconcile downstream master data package from HQ
    pub fn ingest_master_data_package(
        &self,
        package: MasterDataSyncPackage,
    ) -> Result<SyncResultSummary> {
        let mut emp_count = 0;
        let mut shift_count = 0;
        let mut hld_count = 0;

        for emp in package.employees {
            self.db.save_employee(&emp)?;
            emp_count += 1;
        }

        for shift in package.shifts {
            self.db.create_shift(&shift)?;
            shift_count += 1;
        }

        for hld in package.holidays {
            self.db.add_holiday(&hld)?;
            hld_count += 1;
        }

        tracing::info!(
            "Master data package ingested: {} employees, {} shifts, {} holidays",
            emp_count,
            shift_count,
            hld_count
        );

        Ok(SyncResultSummary {
            employees_updated: emp_count,
            shifts_updated: shift_count,
            holidays_updated: hld_count,
            completed_at: chrono::Utc::now().to_rfc3339(),
            has_conflicts: false,
        })
    }
}
