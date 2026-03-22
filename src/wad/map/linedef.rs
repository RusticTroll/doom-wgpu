use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct LineDef {
    pub starting_vertex: u16,
    pub ending_vertex: u16,
    pub flags: u16,
    pub special: u16,
    pub tag: u16,
    pub front_sidedef_index: u16,
    pub back_sidedef_index: u16,
}

impl LineDef {
    /// Returns `true` if this LineDef will block players and monsters, otherwise `false`
    pub fn blocking(&self) -> bool {
        self.flags & 0x1 == 1
    }

    /// Returns `true` if this LineDef will block only monsters, otherwise `false`
    pub fn monster_blocking(&self) -> bool {
        self.flags & 0x2 == 1
    }

    /// Returns `true` if this LineDef is two-sided, otherwise `false`
    pub fn two_sided(&self) -> bool {
        self.flags & 0x4 == 1
    }

    /// Returns `true` if this LineDef's upper texture should be unpegged, otherwise `false`
    pub fn top_unpegged(&self) -> bool {
        self.flags & 0x8 == 1
    }

    /// Returns `true` if this LineDef's lower texture should be unpegged, otherwise `false`
    pub fn bottom_unpegged(&self) -> bool {
        self.flags & 0x10 == 1
    }

    /// Returns `true` if this LineDef is secret, otherwise `false`
    pub fn secret(&self) -> bool {
        self.flags & 0x20 == 1
    }

    /// Returns `true` if this LineDef blocks sound, otherwise `false`
    pub fn sound_blocking(&self) -> bool {
        self.flags & 0x40 == 1
    }

    /// Returns `true` if this LineDef will never appear on the automap, otherwise `false`
    pub fn automap_invisible(&self) -> bool {
        self.flags & 0x80 == 1
    }

    /// Returns `true` if this LineDef will always appear on the automap, otherwise `false`
    pub fn automap_always_visible(&self) -> bool {
        self.flags & 0x100 == 1
    }
}
