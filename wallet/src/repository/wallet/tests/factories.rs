use witnet_data_structures::chain::Hash;

use super::*;
use crate::db::HashMapDb;

pub fn wallet(data: Option<HashMapDb>) -> (Wallet<db::HashMapDb>, db::HashMapDb) {
    wallet_inner(data, true)
}

pub fn wallet_with_args(
    data: Option<HashMapDb>,
    store_master_key: bool,
) -> (Wallet<db::HashMapDb>, db::HashMapDb) {
    wallet_inner(data, store_master_key)
}

fn wallet_inner(
    data: Option<HashMapDb>,
    store_master_key: bool,
) -> (Wallet<db::HashMapDb>, db::HashMapDb) {
    let id = "example-wallet";
    let params = params::Params {
        testnet: false,
        seed_password: "".into(),
        master_key_salt: b"Bitcoin seed".to_vec(),
        id_hash_iterations: 4096,
        id_hash_function: types::HashFunction::Sha256,
        db_hash_iterations: 10_000,
        db_iv_length: 16,
        db_salt_length: 32,
        epoch_constants: EpochConstants::default(),
        node_sync_batch_size: 100,
        genesis_hash: Hash::default(),
        genesis_prev_hash: Hash::default(),
        sync_address_batch_length: 10,
        max_vt_weight: 20_000,
        max_dr_weight: 80_000,
    };
    let mnemonic = types::MnemonicGen::new()
        .with_len(types::MnemonicLength::Words12)
        .generate();
    let source = types::SeedSource::Mnemonics(mnemonic);
    let master_key = crypto::gen_master_key(
        params.seed_password.as_ref(),
        params.master_key_salt.as_ref(),
        &source,
    )
    .unwrap();
    let engine = types::CryptoEngine::new();
    let default_account_index = 0;
    let default_account =
        account::gen_account(&engine, default_account_index, &master_key).unwrap();

    let mut rng = rand::rngs::OsRng;
    let salt = crypto::salt(&mut rng, params.db_salt_length);
    let iv = crypto::salt(&mut rng, params.db_iv_length);

    let db = data.unwrap_or_default();
    let wallets = Wallets::new(db.clone());

    let master_key_to_store = if store_master_key {
        Some(master_key)
    } else {
        None
    };
    // Create the initial data required by the wallet
    wallets
        .create(
            &db,
            types::CreateWalletData {
                iv,
                salt,
                id,
                name: None,
                description: None,
                account: &default_account,
                master_key: master_key_to_store,
            },
        )
        .unwrap();

    let session_id = types::SessionId::from(String::from(id));
    let wallet = Wallet::unlock(id, session_id, db.clone(), params, engine).unwrap();

    (wallet, db)
}

pub fn pkh() -> PublicKeyHash {
    let bytes: [u8; 20] = rand::random();
    PublicKeyHash::from_bytes(&bytes).expect("PKH of 20 bytes failed")
}

pub fn transaction_id() -> types::Hash {
    let bytes: [u8; 32] = rand::random();

    types::Hash::SHA256(bytes)
}

pub fn vtt_from_body(body: types::VTTransactionBody) -> model::ExtendedTransaction {
    model::ExtendedTransaction {
        transaction: types::Transaction::ValueTransfer(VTTransaction {
            body,
            signatures: vec![],
        }),
        metadata: None,
    }
}

#[derive(Default)]
pub struct BlockInfo {
    hash: Option<Vec<u8>>,
    epoch: Option<u32>,
}

impl BlockInfo {
    pub fn create(self) -> model::Beacon {
        let block_hash = Hash::from(
            self.hash
                .unwrap_or_else(|| transaction_id().as_ref().to_vec()),
        );
        let epoch = self.epoch.unwrap_or_else(rand::random);

        model::Beacon { block_hash, epoch }
    }
}
