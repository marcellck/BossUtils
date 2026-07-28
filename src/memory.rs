use std::collections::HashMap;
use proc_maps::{get_process_maps, Pid};
use process_memory::{DataMember, Memory, ProcessHandle, TryIntoProcessHandle};
use sysinfo::System;

pub struct GameAssembly(pub usize);
#[derive(Copy, Clone)]
pub struct InGame(usize);
#[derive(Copy, Clone)]
pub struct Simulation(usize);
#[derive(Copy, Clone)]
pub struct GameModel(usize);
#[derive(Copy, Clone)]
pub struct UnityToSimulation(usize);
#[derive(Copy, Clone)]
pub struct TowerToSimulation(usize);
#[derive(Copy, Clone)]
pub struct Tower(usize);
#[derive(Copy, Clone)]
pub struct TowerModel(usize);

pub fn get_process_pid_and_handle() -> Option<(Pid, ProcessHandle)> {
    let sys = System::new_all();
    let process = sys.processes_by_exact_name("BloonsTD6.exe".as_ref()).next()?;
    let pid = process.pid().as_u32() as Pid;
    let handle = pid.try_into_process_handle().ok()?;
    Some((pid, handle))
}


pub fn get_module_base(pid: Pid) -> Option<GameAssembly> {
    let maps = get_process_maps(pid).ok()?;
    for map in maps {
        if let Some(path) = map.filename() {
            if path.to_string_lossy().contains("GameAssembly.dll") {
                return Some(GameAssembly(map.start()));
            }
        }
    }
    None
}

fn read_memory<T: Copy>(handle: ProcessHandle, offsets: Vec<usize>) -> Option<T> {
    let mut val = DataMember::<T>::new(handle);
    val.set_offset(offsets);
    unsafe { val.read().ok() }
}

impl GameAssembly {
    pub fn get_ingame_instance(self, handle: ProcessHandle) -> Option<InGame> {
        read_memory(handle, vec![
            self.0 + 0x4949E50,
            0xB8,
            0x0,
        ])
    }
}

impl InGame {
    pub fn get_unity_to_simulation(&self, handle: ProcessHandle) -> Option<UnityToSimulation> {
        read_memory(handle, vec![self.0 + 0xD0])
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
        let upgrades: usize = read_memory(handle, vec![self.0 + 0x118]).unwrap();
        let array = read_il2cpp_pointer_array(handle, upgrades);
        let mut result = HashMap::new();
        for entry in array {
            let name: usize = read_memory(handle, vec![entry + 0x10]).unwrap();
            let name: String = read_il2cpp_string(handle, name).unwrap();
            if name.contains("Paragon") && !name.contains("Sentry") {
                let cost: i32 = read_memory(handle, vec![entry + 0x30]).unwrap();
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
        read_memory(handle, vec![self.0 + 0x198])
    }
}

impl TowerModel {
    pub fn get_tier(&self, handle: ProcessHandle) -> Option<i32> {
        read_memory(handle, vec![self.0 + 0x6C])
    }
    pub fn get_total_upgrades(&self, handle: ProcessHandle) -> Option<i32> {
        read_memory(handle, vec![self.0 + 0xF0, 0x18])
    }
    pub fn get_base_id(&self, handle: ProcessHandle) -> Option<String> {
        let str: usize = read_memory(handle, vec![self.0 + 0x30]).unwrap();
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