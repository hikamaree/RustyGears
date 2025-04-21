use std::fs;
use std::path::Path;

#[cfg(target_os = "linux")]
fn detect_active_gpu() -> Option<String> {
    let drm_path = "/sys/class/drm";
    
    if let Ok(entries) = fs::read_dir(drm_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = entry.file_name().into_string().unwrap();
                
                if name.starts_with("card") {
                    let gpu_busy_path = format!("{}/device/gpu_busy_percent", path.display());
                    if Path::new(&gpu_busy_path).exists() {
                        return Some(name);
                    }
                }
            }
        }
    }
    None
}

#[cfg(target_os = "linux")]
fn get_gpu_usage(gpu: Option<String>) -> u32 {
    if let Some(active_gpu) = gpu {
        let usage_path = format!("/sys/class/drm/{}/device/gpu_busy_percent", active_gpu);
        if let Ok(usage) = fs::read_to_string(usage_path) {
            return usage.trim().parse().unwrap_or(0);
        }
    }
    0
}

#[cfg(target_os = "linux")]
fn get_gpu_vendor(gpu: Option<String>) -> String {
    if let Some(active_gpu) = gpu {
        let vendor_path = format!("/sys/class/drm/{}/device/vendor", active_gpu);
        if let Ok(vendor_id) = fs::read_to_string(vendor_path) {
            match vendor_id.trim() {
                "0x10de" => return "NVIDIA".to_string(),
                "0x1002" => return "AMD".to_string(),
                "0x8086" => return "Intel".to_string(),
                _ => return "Unknown".to_string(),
            }
        }
    }
    "Unknown".to_string()
}

#[cfg(target_os = "linux")]
fn get_gpu_temperature(gpu: Option<String>) -> u32 {
    if let Some(active_gpu) = gpu {
        let hwmon_base_path = format!("/sys/class/drm/{}/device/hwmon", active_gpu);

        if let Ok(entries) = fs::read_dir(&hwmon_base_path) {
            for entry in entries.flatten() {
                let hwmon_path = entry.path();
                let temp_path = hwmon_path.join("temp1_input");

                if temp_path.exists() {
                    if let Ok(temp) = fs::read_to_string(temp_path) {
                        return temp.trim().parse::<u32>().map(|t| t / 1000).unwrap_or(0);
                    }
                }
            }
        }
    }
    0
}

#[cfg(target_os = "linux")]
fn get_gpu_vram(gpu: Option<String>) -> String {
    if let Some(active_gpu) = gpu {
        let vram_used_path = format!("/sys/class/drm/{}/device/mem_info_vram_used", active_gpu);
        let vram_total_path = format!("/sys/class/drm/{}/device/mem_info_vram_total", active_gpu);

        let used = fs::read_to_string(vram_used_path)
            .ok()
            .and_then(|s| s.trim().parse::<u64>().ok())
            .map(|v| v / 1024 / 1024)
            .unwrap_or(0);

        let total = fs::read_to_string(vram_total_path)
            .ok()
            .and_then(|s| s.trim().parse::<u64>().ok())
            .map(|v| v / 1024 / 1024)
            .unwrap_or(0);

        return format!("{} / {} MB", used, total);
    }
    "N/A".to_string()
}

pub(super) struct GpuInfo {
    pub gpu: Option<String>,
    pub vendor: String,
    pub usage: u32,
    pub temp: u32,
    pub vram: String,
    pub lastupdate: f32,
}

impl GpuInfo {
    pub fn new() -> Self {
        let gpu = detect_active_gpu();
        let vendor = get_gpu_vendor(gpu.clone());
        let usage = get_gpu_usage(gpu.clone());
        let temp = get_gpu_temperature(gpu.clone());
        let vram = get_gpu_vram(gpu.clone());
        Self {
            gpu,
            vendor,
            usage,
            temp,
            vram,
            lastupdate: 0.0,
        }
    }

    pub fn update(&mut self, curr_time: f32) {
        if curr_time - self.lastupdate <= 1.0 {
            return;
        }

        self.lastupdate = curr_time;
        self.usage = get_gpu_usage(self.gpu.clone());
        self.temp = get_gpu_temperature(self.gpu.clone());
        self.vram = get_gpu_vram(self.gpu.clone());
    }

    pub fn display(&self) -> String {
        return format!("GPU: {}\n  Usage: {}\n  Temp: {}\n  Vram: {}",self.vendor, self.usage, self.temp, self.vram);
    }
}
