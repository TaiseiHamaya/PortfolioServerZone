use crate::zone::zone;

pub trait CommandTrait {
    fn execute(&self, zone: &mut zone::Zone);
}
