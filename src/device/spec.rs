use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use sysinfo::System;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceSpec {
    pub id: String,
    pub os: String,
    pub os_version: String,
    pub core_num: u8,
    pub ip_addr: String,
    pub port: u16,
    pub status: String,
    pub updated_at: String,
}

pub fn get_system_info() -> DeviceSpec {
    let mut system = System::new_all();
    system.refresh_all();

    let os = System::name().unwrap_or_else(|| "Unknown".to_string());
    let os_version = System::os_version().unwrap_or_else(|| "Unknown version".to_string());
    let core_num = system.cpus().len() as u8;

    let ip_addr = "".to_string();
    let port = 0;

    let status = "Active".to_string();
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let updated_at = format!("{:?}", since_the_epoch);

    DeviceSpec {
        id: uuid::Uuid::new_v4().to_string(),
        os,
        os_version,
        core_num,
        ip_addr,
        port,
        status,
        updated_at,
    }
}
