use actix::prelude::*;

use crate::actors::worker;
use crate::types;

pub struct UnlockWallet {
    /// Wallet id
    pub id: String,
    /// Wallet password
    pub password: types::Password,
}

impl Message for UnlockWallet {
    type Result = worker::Result<types::UnlockedSessionWallet>;
}

impl Handler<UnlockWallet> for worker::Worker {
    type Result = <UnlockWallet as Message>::Result;

    fn handle(
        &mut self,
        UnlockWallet { id, password }: UnlockWallet,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.unlock_wallet(&id, password.as_ref())
    }
}
