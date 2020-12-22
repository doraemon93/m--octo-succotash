use actix::prelude::*;

use crate::actors::worker;

pub struct UpdateWalletInfo {
    /// Wallet id
    pub wallet_id: String,
    /// Wallet name
    pub name: Option<String>,
}

impl Message for UpdateWalletInfo {
    type Result = worker::Result<()>;
}

impl Handler<UpdateWalletInfo> for worker::Worker {
    type Result = <UpdateWalletInfo as Message>::Result;

    fn handle(
        &mut self,
        UpdateWalletInfo { wallet_id, name }: UpdateWalletInfo,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.update_wallet_info(&wallet_id, name)
    }
}
