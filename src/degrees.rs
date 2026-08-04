#[derive(Default, Debug)]
pub struct DegreeFactors {
    pub worth: f32,
    pub damage_dealt: i64,
    pub cash_earned: f32,
    pub total_upgrades: i32,
    pub tier_5s: i32,
    pub paragon_cost: f32,
    pub totems: i32
}

impl DegreeFactors {
    pub fn get_degree_requirements(self) -> Vec<(i32, f32)> {
        let current_power = self.get_total_power();
        let current_degree = degree_from_power(current_power);
        let mut result = vec![(current_degree, 0.0)];
        let max_power_from_slider = 60000.0 - self.get_power_from_worth();
        for degree in (current_degree+1)..=100 {
            let power_required = power_required(degree) - current_power;
            if power_required > max_power_from_slider {
                break;
            }
            let cash_spent_per_power = self.paragon_cost * 1.05 / 20000.0;
            result.push((degree, power_required*cash_spent_per_power))
        }
        result
    }
    fn get_total_power(&self) -> f32 {
        let total_power = self.get_power_from_worth()
            + self.get_power_from_tier5s()
            + self.get_power_from_upgrades()
            + self.get_power_from_pops_and_cash_generated()
            + self.get_power_from_totems();
        total_power
    }
    fn get_power_from_worth(&self) -> f32 {
        let cost_power = ((self.worth) / (self.paragon_cost / 20000.0)).min(60000.0);
        cost_power
    }
    fn get_power_from_tier5s(&self) -> f32 {
        let t5_power = ((self.tier_5s - 3) as f32 * 6000.0).min(50000.0).max(0.0);
        t5_power
    }
    fn get_power_from_upgrades(&self) -> f32 {
        let total_upgrade_power = (self.total_upgrades as f32 * 100.0).min(10000.0);
        total_upgrade_power
    }
    fn get_power_from_pops_and_cash_generated(&self) -> f32 {
        let pop_and_income_power = (self.damage_dealt as f32 / 180.0 + self.cash_earned / 45.0).min(90000.0);
        pop_and_income_power
    }
    fn get_power_from_totems(&self) -> f32 {
        let totem_power = self.totems * 2000;
        totem_power as f32
    }
}

fn power_required(degree: i32) -> f32 {
    (50 * degree.pow(3) + 5025 * degree.pow(2) + 168324 * degree + 843000) as f32 / 600.0
}

fn degree_from_power(power: f32) -> i32 {
    for degree in 2..=100 {
        if power_required(degree) >= power {
            return degree - 1;
        }
    }
    100
}