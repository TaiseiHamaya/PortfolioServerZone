use std::collections::HashMap;

use crate::{
    game::entity::{enemy::Enemy, entity::Entity},
    zone::command::{CommandBox, sync_service::broadcast::spawn_enemy_command::SpawnEnemyCommand},
};

pub struct ContaintsDirector {
    enemies: HashMap<u64, Enemy>,

    commands: Vec<CommandBox>,
}

impl ContaintsDirector {
    pub fn new() -> Self {
        ContaintsDirector {
            enemies: HashMap::new(),
            commands: Vec::new(),
        }
    }

    pub fn spawn_enemy(&mut self, entity_id: u64) {
        let enemy = Enemy::new(entity_id, nalgebra::Point3::new(0.0, 0.0, 8.0), 1.0);
        self.commands
            .push(Box::new(SpawnEnemyCommand::new(enemy.entity_id())));
        self.enemies.insert(enemy.entity_id(), enemy);
    }

    pub fn take_commands(&mut self) -> Vec<CommandBox> {
        std::mem::take(&mut self.commands)
    }

    pub fn update(&mut self) {
        self.enemies.iter_mut().for_each(|(_, enemy)| {
            enemy.update();
        });
    }

    pub fn enemies(&self) -> &HashMap<u64, Enemy> {
        &self.enemies
    }

    pub fn enemies_mut(&mut self) -> &mut HashMap<u64, Enemy> {
        &mut self.enemies
    }
}
