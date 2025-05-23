#[derive(Debug, Clone)]
pub enum Slot {
    BODY,
    HEAD,
    HANDS,
    FEET,
    ROPE,
    SPHERE,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Stat {
    HP(u32),
    ATK(u32),
    DEF(u32),
    SPD(u32),
    HPP(f32),
    ATKP(f32),
    DEFP(f32),
    BE(f32),  // break effect
    EHR(f32), // effect hit rate
    ERR(f32), // energy regen
    OHB(f32), // healing
    PHYS(f32),
    FIRE(f32),
    ICE(f32),
    WIND(f32),
    LIGHTNING(f32),
    QUANTUM(f32),
    IMAGINARY(f32),
    CR(f32),
    CD(f32),
}

#[derive(Debug, Clone)]
pub enum Set {
    POET,
}

#[derive(Debug, Clone)]
pub struct Relic {
    pub name: String,
    pub set: Set,
    pub slot: Slot,
    pub mainstat: Stat,
    pub substats: Vec<Stat>
}

impl Relic {
    pub fn new(name: String, set: Set, slot: Slot, mainstat: Stat, substats: Vec<Stat>) -> Self {
        Self {
            name,
            set,
            slot,
            mainstat,
            substats,
        }
    }
}

impl PartialEq for Relic {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.mainstat == other.mainstat && self.substats == other.substats       
    }
}
impl Eq for Relic {}