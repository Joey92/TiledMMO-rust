use self::talker::get_talker_thinker;

pub mod talker;

pub fn get_thinker(name: &str) -> Result<big_brain::thinker::ThinkerBuilder, String> {
    match name {
        "Talker" => Ok(get_talker_thinker()),
        _ => Err(format!("Script {} not implemented", name)),
    }
}
