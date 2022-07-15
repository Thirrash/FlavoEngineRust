pub mod assert;
pub mod log;

pub fn initialize() -> Result<(), std::io::Error> {
    return match log::initialize() {
        Ok(_) => Ok(()),
        Err(err) => Err(err)
    };
}
