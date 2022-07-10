pub mod game_core;
mod test_tic_tac_toe;

fn main() {
    let entity_mgr = flavo_engine::ecs::EntityManager::new();
    let test_val = entity_mgr.get_test();
    println!("Retrieved value from flavo_engine: {}", &test_val);
    test_tic_tac_toe::run_tic_tac_toe();
}
