use sysinfo::{
  CpuRefreshKind,
  Disks, 
  Networks,
  RefreshKind,
  System
};
use chrono::{DateTime, Utc};

pub fn get_info() -> String {
  let time: DateTime<Utc> = Utc::now();
  
  let sysinfo = format!(
    "{}\n{:.0}\n{}\n{}\n{}\n{}\n{}",
    host_name(),
    cpu_average(),
    memory(),
    bandwidth(),
    disk(),
    total_mem(),
    time // Dinamico
  );
  sysinfo
}

fn host_name() -> String { //Estatico
  System::host_name().unwrap()
}

fn cpu_average() -> f32 { //Dinamico
  let mut s = System::new_with_specifics(
    RefreshKind::new().with_cpu(CpuRefreshKind::everything())
  );

  // Wait a bit because CPU usage is based on diff.
  std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
  // Refresh CPUs again.
  s.refresh_cpu();

  let mut cpu_avrg = 0.0;
  let logic_cores = s.cpus().len() as f32;
  for cpu in s.cpus() {
    cpu_avrg += cpu.cpu_usage();
  }
  cpu_avrg /= logic_cores;
  cpu_avrg.trunc()
}

fn memory() -> u64 { // Dinamico
  let mut sys = System::new_all();
  sys.refresh_all();

  sys.used_memory() / 1_000_000
}

fn bandwidth() -> u64 { //Dinamico
  let mut bandwith: u64 = 0;
  let mut freebandwith: u64 = 0;

  let mut networks = Networks::new_with_refreshed_list();
  for (_interface_name, network) in &networks {
    bandwith = network.total_transmitted() + network.total_received();
  }

  networks.refresh();
  for (_interface_name, network) in &networks {
    freebandwith = bandwith - (network.transmitted() + network.received());
  }

  freebandwith / 1_000_000
}

fn disk() -> u64 { // Estatico
  let mut disk_space = 0;

  let disks = Disks::new_with_refreshed_list();
  if let Some(disk) = disks.list().iter().next(){
    disk_space = disk.available_space() / 1_000_000_000
  }

  disk_space
}

fn total_mem() -> u64 { // Estatico
  let mut sys = System::new_all();
  sys.refresh_all();

  sys.total_memory() / 1_000_000
}