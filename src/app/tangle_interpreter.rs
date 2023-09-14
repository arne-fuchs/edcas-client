use std::{env, fs};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use base64::Engine;
use base64::engine::general_purpose;
use base64::prelude::BASE64_STANDARD;
use bus::BusReader;
use json::JsonValue;
use log::{debug, error, info, warn};
use flate2::Compression;
use flate2::write::ZlibEncoder;

use iota_sdk::crypto::keys::bip44::Bip44;
use iota_sdk::client::constants::SHIMMER_COIN_TYPE;
use iota_sdk::wallet::{Account, ClientOptions};
use iota_sdk::client::secret::SecretManager;
use iota_sdk::client::secret::stronghold::StrongholdSecretManager;
use iota_sdk::crypto::ciphers::traits::consts::UInt;
use iota_sdk::crypto::hashes::blake2b::Blake2b256;
use iota_sdk::crypto::hashes::{Digest, Output};
use iota_sdk::Wallet;
use iota_sdk::wallet::account::types::Balance;
use iota_sdk::client::secret::SecretManage;
use iota_sdk::types::block::signature::Ed25519Signature;
use rustc_hex::ToHex;

use serde_json::json;
use serde::Serialize;

use crate::app::settings::Settings;

pub struct TangleInterpreter {
    bus: BusReader<JsonValue>,
    settings: Arc<Settings>,
    account: Account,
    stronghold: StrongholdSecretManager,
    bech32_hrp: String,
    address: String,
}

impl TangleInterpreter {
    pub fn run(&mut self) {
        let Self {
            bus,
            settings: _,
            account: _,
            stronghold: _,
            bech32_hrp,
            address,
        } = self;

        match bus.recv() {
            Err(_) => {}
            Ok(json) => {
                let json_clone = json.clone();
                let event = json["event"].as_str().unwrap();

                info!("Tangle event received: {}", event);

                let mut encoder = ZlibEncoder::new(Vec::new(), Compression::fast());
                encoder.write_all(json_clone.to_string().as_bytes()).unwrap();
                let compressed_data = encoder.finish().unwrap();

                match event {
                    _ => {
                        tokio::runtime::Builder::new_multi_thread()
                            .enable_all()
                            .build()
                            .unwrap()
                            .block_on(async move {
                                //Hash the input
                                //let mut hasher = Blake2b256::new();
                                //hasher.update(compressed_data.clone());
                                //let hash = hasher.finalize();

                                //let bkake2b256: [u8; 32] = hash.as_slice().try_into().expect("Invalid length");


                                let bip44_chain = Bip44::new(SHIMMER_COIN_TYPE)
                                    .with_account(0)
                                    .with_change(false as _)
                                    .with_address_index(0);

                                let ed255195_signature = self.stronghold
                                    .sign_ed25519(&compressed_data, bip44_chain)
                                    .await.unwrap();

                                let mut signature: String = "0x".to_string();
                                let tmp_sig: String = ed255195_signature.signature().to_bytes().to_hex();
                                signature.push_str(tmp_sig.as_str());

                                let mut public_key: String = "0x".to_string();
                                let tmp_pk: String = ed255195_signature.public_key().to_bytes().to_hex();
                                public_key.push_str(tmp_pk.as_str());

                                let message_data = general_purpose::STANDARD.encode(compressed_data);


                                let message = json!(
                                    {
                                        "message": message_data,
                                        "signature": signature,
                                        "public_key": public_key
                                    }
                                );

                                let result = self.account.client().build_block()
                                    .with_tag(event.to_uppercase().as_bytes().to_vec())
                                    .with_data(message.to_string().as_bytes().to_vec())
                                    .finish()
                                    .await;

                                match result {
                                    Ok(block) => {
                                        debug!("Block send: {}",block.id());
                                    }
                                    Err(err) => {
                                        error!("Couldn't send block: {:?}", err);
                                    }
                                }
                            });
                    }
                }
            }
        }
    }
}

pub fn initialize(tangle_bus_reader: BusReader<JsonValue>, settings: Arc<Settings>) -> TangleInterpreter {
    info!("Loading wallet");

    let mut node_url = settings.iota_settings.base_url.clone();
    let port = settings.iota_settings.port.to_string();

    node_url.push(':');
    node_url.push_str(port.as_str());

    info!("Using URL:{}", &node_url);
    info!("Local POW:{}", settings.iota_settings.local_pow);

    let client_options = ClientOptions::new()
        .with_pow_worker_count(1)
        .with_local_pow(settings.iota_settings.local_pow)
        .with_node(node_url.as_str()).unwrap();

    //create stronghold account
    let mut wallet_path = PathBuf::from("wallet.stronghold");
    let mut storage_path = "walletdb".to_string();
    match env::var("HOME") {
        Ok(home) => {
            match fs::create_dir_all(format!("{}/.local/share/edcas-client/walletdb", home)) {
                Ok(_) => {
                    storage_path = format!("{}/.local/share/edcas-client/walletdb", home);
                    wallet_path = PathBuf::from(format!("{}/.local/share/edcas-client/wallet.stronghold", home));
                }
                Err(_) => {}
            }
        }
        Err(_) => {}
    }

    let wallet_file_result = File::open(&wallet_path);

    info!("Wallet path: {:?}", &wallet_path);
    info!("Wallet storage path: {}", &storage_path);

    info!("Existing wallet found?: {}",wallet_file_result.is_ok());

    let account = match wallet_file_result {
        Ok(file) => {
            debug!("{:?}", file);
            info!("Stronghold file found");
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async {
                    let secret_manager = StrongholdSecretManager::builder()
                        .password(settings.iota_settings.password.to_string())
                        .build(&wallet_path).unwrap();

                    let stronghold = SecretManager::Stronghold(secret_manager);

                    let wallet = Wallet::builder()
                        .with_secret_manager(stronghold)
                        .with_client_options(client_options)
                        .with_coin_type(SHIMMER_COIN_TYPE)
                        .finish().await.unwrap();

                    let account = wallet
                        .get_account("User").await.unwrap();

                    let balance = account.sync(None).await.unwrap();

                    info!("[Total: {} : Available: {}]",balance.base_coin().total(),balance.base_coin().available());
                    info!("[NFTS Count: {}]",balance.nfts().len());
                    info!("[Req. storage deposit (basic): {}]",balance.required_storage_deposit().basic());

                    account
                })
        }
        Err(err) => {
            debug!("{}", &err);
            warn!("{}",err);
            info!("Stronghold file not found -> creating");

            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async {
                    // Setup Stronghold secret_manager
                    let mut secret_manager = StrongholdSecretManager::builder()
                        .password(settings.iota_settings.password.to_string())
                        .build(&wallet_path).unwrap();

                    // Only required the first time, can also be generated with `manager.generate_mnemonic()?`
                    let wallet = Wallet::builder()
                        .with_secret_manager(SecretManager::Stronghold(secret_manager))
                        .with_client_options(client_options)
                        .with_coin_type(SHIMMER_COIN_TYPE)
                        .finish().await.unwrap();

                    // The mnemonic only needs to be stored the first time
                    let mnemonic = wallet.generate_mnemonic().unwrap();
                    wallet.store_mnemonic(mnemonic).await.unwrap();

                    // Create a new account
                    wallet
                        .create_account()
                        .with_alias("User".to_string())
                        .finish()
                        .await.unwrap()
                })
        }
    };

    let _account_balance: Balance = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            account.sync(None).await.unwrap()
        });

    //get address one time so it doesn't have to be created each time
    let address = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed creating addresses")
        .block_on(async {
            let address = account.addresses().await.unwrap()[0].address().to_string();
            debug!("{}", &address);
            info!("Address: {}",&address);
            address
        });

    let bech32_hrp = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed creating addresses")
        .block_on(async {
            account.client().get_bech32_hrp().await.unwrap().to_string()
        });

    assert_eq!(&bech32_hrp, "edcas");

    info!("Bech32: {}",&bech32_hrp);
    info!("Done loading wallet");

    TangleInterpreter {
        stronghold: StrongholdSecretManager::builder()
            .password(settings.iota_settings.password.to_string())
            .build(&wallet_path).unwrap(),
        bus: tangle_bus_reader,
        settings,
        account,
        bech32_hrp,
        address,
    }
}