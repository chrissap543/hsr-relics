use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Slot {
    BODY,
    HEAD,
    HANDS,
    FEET,
    ROPE,
    SPHERE,
}

impl From<Slot> for usize {
    fn from(slot: Slot) -> Self {
        match slot {
            Slot::BODY => 0,
            Slot::HEAD => 1,
            Slot::HANDS => 2,
            Slot::FEET => 3,
            Slot::ROPE => 4,
            Slot::SPHERE => 5,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Set {
    // relic set
    BAND,
    CHAMPION,
    EAGLE,
    FIRESMITH,
    GENIUS,
    GUARD,
    HERO,
    HUNTER,
    IRON,
    KNIGHT,
    DISCIPLE,
    MESSENGER,
    MUSKETEER,
    PASSERBY,
    PIONEER,
    POET,
    PRISONER,
    ORDEAL,
    SCHOLAR,
    DUKE,
    SOAR,
    THIEF,
    GODDESS,
    WASTELANDER,
    WATCHMAKER,
    WAVESTRIDER,
    // planar
    BELOBOG,
    BONE,
    KEEL,
    CELESTIAL,
    DURAN,
    GLAMOTH,
    AGELESS,
    FORGE,
    TREE,
    INERT,
    REALM,
    SUNKEN,
    ENTERPRISE,
    PENACONY,
    ARENA,
    DESOLATION,
    STATION,
    SPRIGHTLY,
    BANDITRY,
    PARK,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relic {
    pub name: String,
    pub set: Set,
    pub slot: Slot,
    pub mainstat: Stat,
    pub substats: Vec<Stat>,
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
        self.name == other.name
            && self.mainstat == other.mainstat
            && self.substats == other.substats
    }
}
impl Eq for Relic {}
