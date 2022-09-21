// Consider using bitflags::bitflags! macro to create named values from a struct
#[repr(u32)]
pub enum NamedCollisionGroups {
	Everything = std::u32::MAX,
	Terrain = 0b0001,
	Projectile = 0b0010,
	Npc = 0b0100,
}