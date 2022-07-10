#[derive(Default)]
pub struct EntityManager {
    test: i32
}

impl EntityManager {
    pub fn new() -> EntityManager {
        return EntityManager {
            test: 2137 
        };
    }

    pub fn get_test(&self) -> i32 {
        return self.test;
    }
}
