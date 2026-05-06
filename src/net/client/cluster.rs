use crate::game::entity::{entity::Entity, player::Player};
use crate::zone::command::CommandBox;

pub struct Cluster {
    player: Player,
    name: String,

    #[allow(dead_code)]
    user_id: u64,
    gateway_id: u64,

    command_buffers: Vec<CommandBox>,
}

impl Cluster {
    pub fn new(player: Player, name: String, user_id: u64, gateway_id: u64) -> Self {
        Cluster {
            player,
            name,
            user_id,
            gateway_id,
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

    pub fn entity_id(&self) -> u64 {
        self.player.entity_id()
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

    #[allow(dead_code)]
    pub fn user_id(&self) -> u64 {
        self.user_id
    }

    pub fn gateway_id(&self) -> u64 {
        self.gateway_id
    }
}
