use actix::prelude::*;
use serde::{Deserialize, Serialize};

use crate::actors::app;
use crate::types;

#[derive(Debug, Serialize, Deserialize)]
pub struct ResyncWalletRequest {
    session_id: types::SessionId,
    wallet_id: String,
}

#[derive(Serialize)]
pub struct ResyncWalletResponse {
    success: bool,
}

impl Message for ResyncWalletRequest {
    type Result = app::Result<ResyncWalletResponse>;
}

impl Handler<ResyncWalletRequest> for app::App {
    type Result = app::ResponseActFuture<ResyncWalletResponse>;

    fn handle(&mut self, msg: ResyncWalletRequest, _ctx: &mut Self::Context) -> Self::Result {
        // All the resync methods use `Result<bool>` for convenience when bubbling up whether the
        // resync process is successful, and it only gets mapped to `Result<ResyncWalletRequest>`
        // here.
        let f = self
            .clear_chain_data_and_resync(msg.session_id, msg.wallet_id)
            .map(|success, _, _| ResyncWalletResponse { success });

        Box::new(f)
    }
}
