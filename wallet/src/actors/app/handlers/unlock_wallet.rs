use actix::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{actors::app, model, types};

#[derive(Debug, Serialize, Deserialize)]
pub struct UnlockWalletRequest {
    pub wallet_id: String,
    pub password: types::Password,
}

#[derive(Serialize)]
pub struct UnlockWalletResponse {
    session_id: types::SessionId,
    name: Option<String>,
    description: Option<String>,
    available_accounts: Vec<u32>,
    current_account: u32,
    account_balance: model::WalletBalance,
    session_expiration_secs: u64,
}

impl Message for UnlockWalletRequest {
    type Result = Result<UnlockWalletResponse, app::Error>;
}

impl Handler<UnlockWalletRequest> for app::App {
    type Result = app::ResponseActFuture<UnlockWalletResponse>;

    fn handle(&mut self, msg: UnlockWalletRequest, _ctx: &mut Self::Context) -> Self::Result {
        let f = self.unlock_wallet(msg.wallet_id, msg.password).map(
            |types::UnlockedWallet { data, session_id }, slf, ctx| {
                slf.set_session_to_expire(session_id.clone()).spawn(ctx);

                UnlockWalletResponse {
                    session_id,
                    name: data.name,
                    description: data.description,
                    current_account: data.current_account,
                    available_accounts: data.available_accounts,
                    account_balance: data.balance,
                    session_expiration_secs: slf.params.session_expires_in.as_secs(),
                }
            },
        );

        Box::new(f)
    }
}
