use chrono::{TimeZone, offset::LocalResult};

use crate::game::action::action_list_table::ACTION_LIST_TABLE;
use crate::zone::command::CommandTrait;

use crate::{
    game::entity::entity::PlayActionOk,
    generated::proto_client::{PayloadEntityDamaged, PayloadPlayAction},
    zone::zone,
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

        // 攻撃者の取得
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
        let Some(entity_target) = zone.entity_mut(&self.target_id) else {
            log::warn!("Target entity {} not found in zone.", self.target_id);
            return;
        };

        // targetにダメージを与える
        let PlayActionOk::Damage(dmg) = action_result;
        entity_target.on_damaged(dmg);

        // log
        log::info!(
            "Entity {} performed action {} and dealt {} damage.",
            self.id,
            self.action_id,
            dmg
        );

        let mut client = zone
            .tonic_client_mut()
            .zone_broadcast_service_client
            .clone();

        let play_action_message = PayloadPlayAction {
            id: self.id,
            target_id: self.target_id,
            action_id: self.action_id,
            timestamp: self.timestamp,
        };

        let entity_damaged_message = PayloadEntityDamaged {
            entity_id: self.target_id,
            damage: dmg,
        };

        tokio::spawn(async move {
            if let Err(e) = client.play_action(play_action_message).await {
                log::error!("Failed to send play action message: {}", e);
                return;
            }
            if let Err(e) = client.entity_damaged(entity_damaged_message).await {
                log::error!("Failed to send entity damaged message: {}", e);
            }
        });
    }
}
