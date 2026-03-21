use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct MapThing {
    pub x_position: i16,
    pub y_position: i16,
    pub angle_facing: i16,
    pub thing_type: i16,
    pub flags: i16,
}

impl MapThing {
    /// Returns `true` if this MapThing should spawn on the given skill level and singleplayer status, otherwise `false`
    pub fn should_spawn(&self, skill_level: i32, single_player: bool) -> bool {
        if single_player && self.flags & 0x10 != 0 {
            return false;
        }

        match skill_level {
            1 | 2 => return self.flags & 0x1 == 1,
            3 => return self.flags & 0x2 == 1,
            4 | 5 => return self.flags & 0x4 == 1,
            _ => panic!("Unknown skill level: {}", skill_level),
        }
    }

    /// Returns `true` if this MapThing has the ambush flag set, otherwise `false`
    pub fn in_ambush(&self) -> bool {
        self.flags & 0x8 == 1
    }
}
