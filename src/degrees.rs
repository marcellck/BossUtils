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
    pub fn get_degree(self) -> i32 {
        let power = self.get_total_power();
        for i in 2_i64..=100 {
            let power_required = (50*i.pow(3)+5025*i.pow(2)+168324*i+843000)/600;
            if power_required >= power as i64 {
                return (i - 1) as i32;
            }
        }
        100
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
        dbg!(self.worth);
        dbg!(self.paragon_cost);

        let cost_power = ((self.worth) / (self.paragon_cost / 20000f32)).min(60000f32);
        dbg!(cost_power)
    }
    fn get_power_from_tier5s(&self) -> f32 {
        let t5_power = ((self.tier_5s - 3) as f32 * 6000f32).min(50000f32).max(0f32);
        dbg!(t5_power)
    }
    fn get_power_from_upgrades(&self) -> f32 {
        let total_upgrade_power = (self.total_upgrades as f32 * 100f32).min(10000f32);
        dbg!(total_upgrade_power)
    }
    fn get_power_from_pops_and_cash_generated(&self) -> f32 {
        let pop_and_income_power = (self.damage_dealt as f32 / 180f32 + self.cash_earned / 45f32).min(90000f32);
        dbg!(pop_and_income_power)
    }
    fn get_power_from_totems(&self) -> f32 {
        let totem_power = self.totems * 2000;
        dbg!(totem_power) as f32
    }
}