#![allow(dead_code, unused_imports)]

use crate::zone::command::CommandTrait;

//use crate::{generated::proto_client::PayloadZoneExitNotification, zone::zone};

pub struct DisconnectForceCommand {
    player_id: u64,
}

impl DisconnectForceCommand {
    pub fn new(player_id: u64) -> Self {
        DisconnectForceCommand { player_id }
    }
}

// impl CommandTrait for DisconnectForceCommand {
//     fn execute(self: Box<Self>, zone: &mut zone::Zone) {
//         log::info!("Forcefully disconnecting player {}.", self.player_id);
//         // ログアウト要求をチャッシュに追加
//         zone.zone_request_chash_mut().push_logout(self.player_id);

//         let clients = zone.tonic_client_mut().get_gateway_clients();

//         let message = PayloadZoneExitNotification { id: self.player_id };

//         tokio::spawn(async move {
//             for mut client in clients {
//                 let _ = client.player_exit(message.clone()).await;
//             }
//         });
//     }
// }
