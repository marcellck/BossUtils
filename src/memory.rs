use proc_maps::{Pid, get_process_maps};
use process_memory::{DataMember, Memory, ProcessHandle, TryIntoProcessHandle};
use std::collections::HashMap;
use sysinfo::System;

pub struct GameAssembly(pub usize);
#[derive(Copy, Clone, Debug)]
pub struct InGame(usize);
#[derive(Copy, Clone, Debug)]
pub struct Simulation(usize);
#[derive(Copy, Clone, Debug)]
pub struct GameModel(usize);
#[derive(Copy, Clone, Debug)]
pub struct UnityToSimulation(usize);
#[derive(Copy, Clone, Debug)]
pub struct TowerToSimulation(usize);
#[derive(Copy, Clone, Debug)]
pub struct Tower(usize);
#[derive(Copy, Clone, Debug)]
pub struct TowerModel(usize);

// Process name differs by platform
// Module file name differs by platform
#[cfg(any(target_os = "windows", target_os = "linux"))]
const PROCESS_NAME: &str = "BloonsTD6.exe";
#[cfg(any(target_os = "windows", target_os = "linux"))]
const MODULE_NAME: &str = "GameAssembly.dll";
#[cfg(any(target_os = "windows", target_os = "linux"))]
// NOT re-derived for v56.3. I can't test on windows :/
const INGAME_OFFSET: usize = 0x4A5BF20;
#[cfg(target_os = "macos")]
const PROCESS_NAME: &str = "BloonsTD6";
#[cfg(target_os = "macos")]
const MODULE_NAME: &str = "GameAssembly.dylib";
#[cfg(target_os = "macos")]
const INGAME_OFFSET: usize = 0x60C1870;

pub fn get_process_pid_and_handle() -> Option<(Pid, ProcessHandle)> {
    let sys = System::new_all();
    let process = sys.processes_by_exact_name(PROCESS_NAME.as_ref()).next()?;
    let pid = process.pid().as_u32() as Pid;
    let handle = pid.try_into_process_handle().ok()?;
    Some((pid, handle))
}

#[cfg(any(target_os = "windows", target_os = "linux"))]
pub fn get_module_base(pid: Pid) -> Option<GameAssembly> {
    let maps = get_process_maps(pid).ok()?;
    maps.into_iter()
        .filter(|map| {
            map.filename()
                .map(|path| path.to_string_lossy().contains(MODULE_NAME))
                .unwrap_or(false)
        })
        .map(|map| map.start())
        .min()
        .map(GameAssembly)
}

// A dylib/DLL is mapped as many separate memory regions on macOS
// filter to executable regions only and take the minimum address
#[cfg(target_os = "macos")]
pub fn get_module_base(pid: Pid) -> Option<GameAssembly> {
    let maps = get_process_maps(pid).ok()?;
    maps.into_iter()
        .filter(|map| {
            map.is_exec()
                && map
                    .filename()
                    .map(|path| path.to_string_lossy().contains(MODULE_NAME))
                    .unwrap_or(false)
        })
        .map(|map| map.start())
        .min()
        .map(GameAssembly)
}

fn read_memory<T: Copy>(handle: ProcessHandle, offsets: Vec<usize>) -> Option<T> {
    let mut val = DataMember::<T>::new(handle);
    val.set_offset(offsets);
    unsafe { val.read().ok() }
}

impl GameAssembly {
    pub fn get_ingame_instance(self, handle: ProcessHandle) -> Option<InGame> {
        read_memory(handle, vec![self.0 + INGAME_OFFSET, 0xB8, 0x0])
    }
}

impl InGame {
    pub fn get_unity_to_simulation(&self, handle: ProcessHandle) -> Option<UnityToSimulation> {
        read_memory(handle, vec![self.0 + 0xC0])
    }
}

impl UnityToSimulation {
    pub fn get_simulation(&self, handle: ProcessHandle) -> Option<Simulation> {
        read_memory(handle, vec![self.0 + 0x28])
    }
    pub fn get_all_towers(&self, handle: ProcessHandle) -> Option<Vec<TowerToSimulation>> {
        let list: usize = read_memory(handle, vec![self.0 + 0x58]).unwrap();
        let items: usize = read_memory(handle, vec![list + 0x10]).unwrap();
        let length: i32 = read_memory(handle, vec![list + 0x18]).unwrap();

        let mut result = vec![];
        for i in 0..length as usize {
            let tts = read_memory(handle, vec![items + 0x20 + i * 8]).unwrap();
            result.push(tts);
        }
        Some(result)
    }
}

impl Simulation {
    pub fn get_gamemodel(&self, handle: ProcessHandle) -> Option<GameModel> {
        read_memory(handle, vec![self.0 + 0x20])
    }
}

impl GameModel {
    pub fn get_paragon_upgrades_costs(&self, handle: ProcessHandle) -> HashMap<String, i32> {
        let upgrades: usize = read_memory(handle, vec![self.0 + 0x110]).unwrap();
        let array = read_il2cpp_pointer_array(handle, upgrades);
        let mut result = HashMap::new();
        for entry in array {
            let name: usize = read_memory(handle, vec![entry + 0x20]).unwrap();
            let name: String = read_il2cpp_string(handle, name).unwrap();
            if name.contains("Paragon") && !name.contains("Sentry") {
                let cost: i32 = read_memory(handle, vec![entry + 0x28]).unwrap();
                result.insert(name, cost);
            }
        }
        result
    }
}

impl TowerToSimulation {
    pub fn get_tower(&self, handle: ProcessHandle) -> Option<Tower> {
        read_memory(handle, vec![self.0 + 0x18])
    }
}

impl Tower {
    pub fn get_worth(&self, handle: ProcessHandle) -> Option<f32> {
        read_memory(handle, vec![self.0 + 0x158])
    }
    pub fn get_damage_dealt(&self, handle: ProcessHandle) -> Option<i64> {
        read_memory(handle, vec![self.0 + 0x160])
    }
    pub fn get_cash_earned(&self, handle: ProcessHandle) -> Option<f32> {
        read_memory(handle, vec![self.0 + 0x170])
    }
    pub fn get_tower_model(&self, handle: ProcessHandle) -> Option<TowerModel> {
        read_memory(handle, vec![self.0 + 0x1A0])
    }
}

impl TowerModel {
    // the single scalar tier field reads as 0 for every tower on macOS
    pub fn get_max_path_tier(&self, handle: ProcessHandle) -> Option<i32> {
        let array: usize = read_memory(handle, vec![self.0 + 0x68])?;
        if array == 0 {
            return None;
        }
        let length: i32 = read_memory(handle, vec![array + 0x18])?;
        let mut max_tier = 0;
        for i in 0..length as usize {
            let tier: i32 = read_memory(handle, vec![array + 0x20 + i * 4])?;
            if tier > max_tier {
                max_tier = tier;
            }
        }
        Some(max_tier)
    }

    pub fn get_is_paragon(&self, handle: ProcessHandle) -> Option<bool> {
        let value: u8 = read_memory(handle, vec![self.0 + 0x12C])?;
        Some(value != 0)
    }
    pub fn get_total_upgrades(&self, handle: ProcessHandle) -> Option<i32> {
        read_memory(handle, vec![self.0 + 0xE8, 0x18])
    }
    pub fn get_base_id(&self, handle: ProcessHandle) -> Option<String> {
        let str: usize = read_memory(handle, vec![self.0 + 0x28]).unwrap();
        read_il2cpp_string(handle, str)
    }
}

fn read_il2cpp_string(handle: ProcessHandle, str: usize) -> Option<String> {
    let len: i32 = read_memory(handle, vec![str + 0x10]).unwrap_or(0);
    let mut bytes = vec![];
    for i in 0..len as usize {
        let byte = read_memory(handle, vec![str + 0x14 + i * 2]).unwrap();
        bytes.push(byte);
    }
    String::from_utf8(bytes).ok()
}

fn read_il2cpp_pointer_array(handle: ProcessHandle, array: usize) -> Vec<usize> {
    let length: i32 = read_memory(handle, vec![array + 0x18]).unwrap();
    let mut result = vec![];
    for i in 0..length as usize {
        let elem: usize = read_memory(handle, vec![array + 0x20 + i * size_of::<usize>()]).unwrap();
        result.push(elem);
    }
    result
}
