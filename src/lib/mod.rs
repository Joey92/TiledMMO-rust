pub mod components;
pub mod network;

pub fn calc_z_pos(y: f32) -> f32 {
    2. - y * 0.0001
}
