use rand::Rng;

pub fn percentage_chance(chance: u8) -> bool {
    let mut rng = rand::thread_rng();
    let roll = rng.gen_range(0..=100);
    roll <= chance
}
