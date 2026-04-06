use crate::game::entity::{entity::Entity, player::Player};
use crate::zone::command::CommandBox;

pub struct Cluster {
    player: Player,
    name: String,

    command_buffers: Vec<CommandBox>,
}

impl Cluster {
    pub fn new(player: Player, name: String) -> Self {
        Cluster {
            player,
            name,
            command_buffers: Vec::new(),
        }
    }

    pub fn update(&mut self) {
        self.player.update();
    }

    pub fn take_commands(&mut self) -> Vec<CommandBox> {
        let commands = std::mem::take(&mut self.command_buffers);
        commands
    }

    pub fn id(&self) -> u64 {
        self.player.id()
    }

    pub fn player_name(&self) -> &String {
        &self.name
    }

    pub fn player(&self) -> &Player {
        &self.player
    }

    pub fn player_mut(&mut self) -> &mut Player {
        &mut self.player
    }
}
