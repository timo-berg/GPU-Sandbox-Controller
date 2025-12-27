use serde::Deserialize;
use std::collections::HashMap;
use tokio::fs;

#[derive(Deserialize, Clone)]
pub struct Tenant {
    pub tenant_id: String,
    pub allowed_capabilities: Vec<String>,
    pub gpu_slot_limit: usize,
    pub rate_limit: usize, // #jobs / minute
    pub status: TenantStatus,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum TenantStatus {
    Active,
    Suspended,
}

#[derive(Deserialize)]
pub struct TenantFile {
    tenants: Vec<Tenant>,
}

impl Tenant {
    pub async fn load_all(
        path: &str,
    ) -> Result<HashMap<String, Tenant>, Box<dyn std::error::Error>> {
        let contents: String = fs::read_to_string(path).await?;
        let file: TenantFile = serde_json::from_str(&contents)?;
        let tenant_map = file
            .tenants
            .into_iter()
            .map(|t| (t.tenant_id.clone(), t))
            .collect();
        Ok(tenant_map)
    }
}
