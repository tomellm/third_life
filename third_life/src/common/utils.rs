use rand::Rng;

pub fn roll_chance(chance: u8) -> bool {
    let mut rng = rand::thread_rng();
    let roll = rng.gen_range(0..=100);
    if chance == 0 {
        false
    } else {
        roll <= chance
    }
}
