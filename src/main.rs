mod memory;
mod degrees;

use std::collections::HashMap;
use crate::degrees::DegreeFactors;
use crate::memory::{get_module_base, get_process_pid_and_handle};

fn main() {
    let (pid, handle) = get_process_pid_and_handle().expect("BloonsTD6 not running");
    let game_assembly = get_module_base(pid).expect("GameAssembly module not found in process maps");
    let ingame = game_assembly.get_ingame_instance(handle).expect("failed to read InGame instance");
    let bridge = ingame.get_unity_to_simulation(handle).expect("failed to read UnityToSimulation");
    let simulation = bridge.get_simulation(handle).expect("failed to read Simulation");
    let gamemodel = simulation.get_gamemodel(handle).expect("failed to read GameModel");
    let paragon_costs = gamemodel.get_paragon_upgrades_costs(handle);
    let towers = bridge.get_all_towers(handle).expect("failed to read towers list");
    let mut map: HashMap<String, DegreeFactors> = HashMap::new();
    let mut total_totems = 0;
    for tower in towers {
        let tower = tower.get_tower(handle).unwrap();
        let tower_model = tower.get_tower_model(handle).unwrap();
        let base_id = tower_model.get_base_id(handle).unwrap();

        if base_id == "ParagonPowerTotem" {
            total_totems += 1;
            continue;
        }
        let paragon_upgrade_name = format!("{base_id} Paragon");
        // Paragon doesn't exist for tower
        if !paragon_costs.contains_key(&paragon_upgrade_name) {
            continue;
        }

        let worth = tower.get_worth(handle).unwrap();
        let total_upgrades = tower_model.get_total_upgrades(handle).unwrap();
        let damage_dealt = tower.get_damage_dealt(handle).unwrap();
        let cash_earned = tower.get_cash_earned(handle).unwrap();
        let is_tier_5 = tower_model.get_max_path_tier(handle).unwrap() == 5;
        let is_paragon = tower_model.get_is_paragon(handle).unwrap();
        let entry = map.entry(base_id.clone()).or_default();

        // Skip paragons. Useful if player wants to sell and rebuy paragon for higher degree
        if is_paragon {
            continue;
        }
        if is_tier_5 {
            entry.tier_5s += 1;
        } else {
            entry.worth += worth;
            entry.total_upgrades += total_upgrades;
        }
        entry.damage_dealt += damage_dealt;
        entry.cash_earned += cash_earned;
    }
    for (base_id, mut factors) in map {
        factors.totems = total_totems;
        factors.paragon_cost = paragon_costs.get(&format!("{} Paragon", base_id)).unwrap().clone() as f32;
        for (degree, cost) in factors.get_degree_requirements() {
            println!("{base_id} (D{degree}): {cost}");
        }
    }
}