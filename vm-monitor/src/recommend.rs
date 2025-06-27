use serde::Deserialize;

// Struct to represent a row in our instances.csv dataset
#[derive(Debug, Deserialize, Clone)]
pub struct VmInstance {
    pub instance_name: String,
    pub provider: String,
    pub region: String,
    pub vcpus: u32,
    pub memory_gb: f32,
    pub hourly_cost: f32,
}

// Struct to hold the recommendation result, including the calculated score
#[derive(Debug, Clone)]
pub struct Recommendation {
    pub instance: VmInstance,
    pub cost_per_needed_resource: f32,
}

// Load and deserialize the embedded CSV data
pub fn load_vm_dataset() -> Result<Vec<VmInstance>, csv::Error> {
    const DATA: &str = include_str!("../instances.csv");
    let mut reader = csv::Reader::from_reader(DATA.as_bytes());
    reader.deserialize().collect()
}

pub fn recommend_vms(
    dataset: &[VmInstance],
    avg_cpu_usage_percent: f32,
    physical_cpu_cores: u32,
    avg_memory_used_gb: f32,
    region_pref: Option<&str>,
) -> Vec<Recommendation> {

    // Calculate user's effective resource usage
    // Cap at a minimum to avoid recommending tiny VMs for idle systems
    let needed_cpu_cores = (physical_cpu_cores as f32 * (avg_cpu_usage_percent / 100.0)).max(1.0);
    let needed_memory_gb = avg_memory_used_gb.max(2.0); // Assume at least 2GB is needed

    println!("Based on average usage, recommending for ~{:.2} vCPUs and {:.2} GB Memory...", needed_cpu_cores, needed_memory_gb);

    // 1. FILTER: Only VMs that can handle the workload
    let buffer = 1.25; // 25% safety buffer
    let mut suitable_vms: Vec<VmInstance> = dataset.iter()
        .filter(|vm| {
            let cpu_ok = vm.vcpus as f32 >= needed_cpu_cores * buffer;
            let mem_ok = vm.memory_gb >= needed_memory_gb * buffer;
            let region_ok = match region_pref {
                Some(pref) => vm.region.to_lowercase().contains(&pref.to_lowercase()),
                None => true,
            };
            cpu_ok && mem_ok && region_ok
        })
        .cloned() // Clone the data to make it mutable
        .collect();

    if suitable_vms.is_empty() {
        println!("No suitable VMs found in the dataset with the given criteria and buffer. Try a wider region or check usage stats.");
        return vec![];
    }
    
    // 2. SCORE: Calculate efficiency metrics and create Recommendation structs
    let total_needed_resources = (needed_cpu_cores * buffer) + (needed_memory_gb * buffer);
    let mut recommendations: Vec<Recommendation> = suitable_vms.iter_mut()
        .map(|vm| {
            Recommendation {
                cost_per_needed_resource: if total_needed_resources > 0.0 {
                    vm.hourly_cost / total_needed_resources
                } else {
                    f32::MAX // Avoid division by zero
                },
                instance: vm.clone(),
            }
        })
        .collect();

    // 3. RANK: Sort by cost efficiency
    recommendations.sort_by(|a, b| a.cost_per_needed_resource.partial_cmp(&b.cost_per_needed_resource).unwrap());
    
    // 4. GROUP & SELECT TOP 2 PER PROVIDER
    let mut final_recommendations = Vec::new();
    let mut seen_providers = std::collections::HashMap::new();

    for rec in recommendations {
        let count = seen_providers.entry(rec.instance.provider.clone()).or_insert(0);
        if *count < 2 {
            final_recommendations.push(rec);
            *count += 1;
        }
    }

    // Final sort of the top-N results
    final_recommendations.sort_by(|a, b| a.cost_per_needed_resource.partial_cmp(&b.cost_per_needed_resource).unwrap());

    final_recommendations
}