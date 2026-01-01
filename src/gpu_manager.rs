use std::collections::HashMap;

use crate::{config::Config, tenant::Tenant};

pub struct GpuManager {
    gpu_slots: usize,
    per_tenant_limit: usize,
    tenant_resources: HashMap<String, usize>, // tenant_id -> current_count
}

impl GpuManager {
    pub fn new(config: &Config) -> Self {
        GpuManager {
            gpu_slots: config.gpu_slots,
            per_tenant_limit: config.per_tenant_limit,
            tenant_resources: HashMap::new(),
        }
    }

    pub fn try_reserve_slot(&mut self, tenant: &Tenant) -> Result<(), GpuError> {
        let total_used: usize = self.tenant_resources.values().sum();
        if total_used >= self.gpu_slots {
            return Err(GpuError::NoGlobalCapacity);
        }

        let current_count = self
            .tenant_resources
            .entry(tenant.tenant_id.to_string())
            .or_insert(0);

        if *current_count >= tenant.gpu_slot_limit {
            return Err(GpuError::TenantLimitReached);
        }

        *current_count += 1;

        Ok(())
    }

    pub fn release_slot(&mut self, tenant_id: &str) -> Result<(), GpuError> {
        let should_remove = {
            if let Some(count) = self.tenant_resources.get_mut(tenant_id) {
                if *count == 0 {
                    return Err(GpuError::TenantHasNoSlots);
                }
                *count -= 1;
                *count == 0
            } else {
                return Err(GpuError::TenantHasNoSlots);
            }
        };

        if should_remove {
            self.tenant_resources.remove(tenant_id);
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GpuError {
    #[error("No GPU slots available")]
    NoGlobalCapacity,
    #[error("Tenant reached capacity")]
    TenantLimitReached,
    #[error("Tenant has no active slots to free")]
    TenantHasNoSlots,
}
