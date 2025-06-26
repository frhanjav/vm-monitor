use chrono::{DateTime, Utc};
use serde::Serialize;
use sysinfo::{System, Disks, Networks};
use uuid::Uuid;

#[derive(Serialize, Debug)]
pub struct CpuMetrics {
    pub usage_percent: f32,
    pub core_count: usize,
    pub per_core_usage: Vec<f32>,
}

#[derive(Serialize, Debug)]
pub struct MemoryMetrics {
    pub total_memory: u64,
    pub used_memory: u64,
    pub available_memory: u64,
    pub total_swap: u64,
    pub used_swap: u64,
}

#[derive(Serialize, Debug)]
pub struct DiskMetric {
    pub name: String,
    pub mount_point: String,
    pub total_space: u64,
    pub available_space: u64,
    pub filesystem: String,
    pub total_written_bytes: u64,
    pub total_read_bytes: u64,
}

#[derive(Serialize, Debug)]
pub struct NetworkMetric {
    pub interface_name: String,
    pub received_bytes_total: u64,
    pub transmitted_bytes_total: u64,
}

#[derive(Serialize, Debug)]
pub struct SystemInfo {
    pub hostname: String,
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
    pub uptime: u64, // seconds
}

#[derive(Serialize, Debug)]
pub struct SystemMetrics {
    pub timestamp: DateTime<Utc>,
    pub instance_id: Uuid,
    pub cpu_metrics: CpuMetrics,
    pub memory_metrics: MemoryMetrics,
    pub disk_metrics: Vec<DiskMetric>,
    pub network_metrics: Vec<NetworkMetric>,
    pub system_info: SystemInfo,
}

pub fn collect_metrics(instance_id: Uuid, sys: &mut System) -> SystemMetrics {
    sys.refresh_cpu_all();
    sys.refresh_memory();

    let disks = Disks::new_with_refreshed_list();
    let networks = Networks::new_with_refreshed_list();

    let cpu_metrics = CpuMetrics {
        usage_percent: sys.global_cpu_usage(),
        core_count: sys.cpus().len(),
        per_core_usage: sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect(),
    };

    let memory_metrics = MemoryMetrics {
        total_memory: sys.total_memory(),
        used_memory: sys.used_memory(),
        available_memory: sys.available_memory(),
        total_swap: sys.total_swap(),
        used_swap: sys.used_swap(),
    };

    let disk_metrics: Vec<DiskMetric> = disks
        .iter()
        .map(|disk| DiskMetric {
            name: disk.name().to_string_lossy().into_owned(),
            mount_point: disk.mount_point().to_string_lossy().into_owned(),
            total_space: disk.total_space(),
            available_space: disk.available_space(),
            filesystem: disk.file_system().to_string_lossy().into_owned(),
            total_written_bytes: disk.usage().total_written_bytes,
            total_read_bytes: disk.usage().total_read_bytes,
        })
        .collect();
      
    let network_metrics: Vec<NetworkMetric> = networks
        .iter()
        .map(|(name, data)| NetworkMetric {
            interface_name: name.clone(),
            received_bytes_total: data.total_received(),
            transmitted_bytes_total: data.total_transmitted(),
        })
        .collect();

    let system_info = SystemInfo {
        hostname: System::host_name().unwrap_or_else(|| "N/A".to_string()),
        os_name: System::name().unwrap_or_else(|| "N/A".to_string()),
        os_version: System::os_version().unwrap_or_else(|| "N/A".to_string()),
        kernel_version: System::kernel_version().unwrap_or_else(|| "N/A".to_string()),
        uptime: System::uptime(),
    };

    SystemMetrics {
        timestamp: Utc::now(),
        instance_id,
        cpu_metrics,
        memory_metrics,
        disk_metrics,
        network_metrics,
        system_info,
    }
}