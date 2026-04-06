use crate::zone::zone;

pub trait CommandTrait: Send {
    fn execute(self: Box<Self>, zone: &mut zone::Zone);
}

pub type CommandBox = Box<dyn CommandTrait + Send>;
