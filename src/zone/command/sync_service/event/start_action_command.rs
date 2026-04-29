use chrono::{TimeZone, offset::LocalResult};

use crate::game::action::action_list_table::ACTION_LIST_TABLE;
use crate::zone::command::CommandTrait;

use crate::zone::command::sync_service::broadcast::entity_damaged_command::DamagedEntityCommand;
use crate::{
    game::entity::entity::PlayActionOk, generated::proto_client::PayloadPlayAction, zone::zone,
};

pub struct StartActionCommand {
    id: u64,
    target_id: u64,
    action_id: u32,
    timestamp: i64,
}

impl StartActionCommand {
    pub fn new(id: u64, target_id: u64, action_id: u32, timestamp: i64) -> Self {
        StartActionCommand {
            id,
            target_id,
            action_id,
            timestamp,
        }
    }
}

impl CommandTrait for StartActionCommand {
    fn execute(self: Box<Self>, zone: &mut zone::Zone) {
        log::info!("Entity {} begins action {}.", self.id, self.action_id);

        // 使用者の取得
        let Some(entity_player) = zone.entity_mut(&self.id) else {
            log::warn!("Entity {} not found in zone.", self.id);
            return;
        };

        // Timestampの変換
        let LocalResult::Single(timestamp) = chrono::Utc.timestamp_micros(self.timestamp) else {
            log::warn!("Invalid timestamp for action: {}", self.timestamp);
            return;
        };

        // プレイヤーのアクション実行
        let job_id = 0; // 仮のjob_id
        let Some(action_list) = ACTION_LIST_TABLE.wait().get_action_list(job_id) else {
            log::warn!("Action list for job {} not found.", job_id);
            return;
        };
        let Ok(action_result) = entity_player.play_action(self.action_id, &timestamp, action_list)
        else {
            log::warn!(
                "Failed to play action {} for entity {}.",
                self.action_id,
                self.id
            );
            return;
        };

        // ターゲットの取得
        if zone.entity_mut(&self.target_id).is_none() {
            log::warn!("Target entity {} not found in zone.", self.target_id);
            return;
        }

        let action_result_command = match action_result {
            PlayActionOk::Damage(dmg) => {
                Box::new(DamagedEntityCommand::new(self.id, self.target_id, dmg))
                    as Box<dyn CommandTrait>
            }
            PlayActionOk::Heal(heal) => {
                // Healの処理はここに追加
                log::info!("Entity {} healed for {} hit points.", self.target_id, heal);
                return;
            }
        };

        let clients = zone.gateway_clients().clients_vec();

        let play_action_message = PayloadPlayAction {
            id: self.id,
            target_id: self.target_id,
            action_id: self.action_id,
            timestamp: self.timestamp,
        };

        for mut client in clients {
            let message_clone = play_action_message.clone();
            tokio::spawn(async move {
                if let Err(e) = client.play_action(message_clone).await {
                    log::error!("Failed to send play action message: {}", e);
                }
            });
        }

        zone.add_zone_command(action_result_command);
    }
}
