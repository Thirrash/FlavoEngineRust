mod test_tic_tac_toe;
use flavo_engine::ecs::entity_manager::EntityManager;
use flavo_engine::logger::assert::assert_error;
use flavo_engine::{log_error, log_debug};

fn main() {
    flavo_engine::logger::initialize().expect("Couldn't initialize flavo_engine::logger");

    let entity_mgr = EntityManager::new();
    let test_val = entity_mgr.get_test();
    log_debug!("Retrieved value from flavo_engine: {}", &test_val);
    log_error!("Just another shit");

    assert_error(|| false, || format!("Giga Dupsko {}", &test_val));

    test_tic_tac_toe::game::run_tic_tac_toe();
}
