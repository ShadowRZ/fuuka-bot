use crate::config::TagTriggers;

pub mod illust;

pub struct Context {
    pub r18: bool,
    pub tag_triggers: TagTriggers,
}
