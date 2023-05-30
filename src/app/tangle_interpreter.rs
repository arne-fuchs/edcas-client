use std::fs::File;
use std::ops::Add;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;
use bus::BusReader;
use iota_wallet::account::{AccountHandle, SyncOptions};
use iota_wallet::{ClientOptions, Error, NftOptions};
use iota_wallet::account::types::{AccountBalance, Transaction};
use iota_wallet::account_manager::AccountManager;
use iota_wallet::iota_client::api_types::core::response::OutputWithMetadataResponse;
use iota_wallet::iota_client::block::address::{Address, NftAddress};
use iota_wallet::iota_client::node_api::indexer::query_parameters::QueryParameter;
use iota_wallet::iota_client::generate_mnemonic;
use iota_wallet::iota_client::constants::SHIMMER_COIN_TYPE;
use iota_wallet::secret::stronghold::StrongholdSecretManager;
use json::JsonValue;
use log::{debug, error, info, warn};
use rustc_hex::ToHex;
use async_recursion::async_recursion;
use iota_wallet::iota_client::block::output::NftId;
use serde_json::{json, Value};
use crate::app::settings::Settings;

pub struct TangleInterpreter {
    queue: Vec<NftOptions>,
    bus: BusReader<JsonValue>,
    settings: Settings,
    account: AccountHandle,
    bech32_hrp: String,
    address: String,
    issuer_nft_id: Option<NftId>,
}

impl TangleInterpreter {
    pub fn run(&mut self) {
        let Self {
            queue,
            bus,
            settings,
            account,
            bech32_hrp,
            address,
            issuer_nft_id,
        } = self;

        match bus.recv() {
            Err(_) => {}
            Ok(json) => {
                let event = json["event"].as_str().unwrap();

                info!("Tangle event received: {}", event);

                match event {
                    "FSDJump" | "Location" => {
                        let irc = generate_irc27_from_json(json.clone());

                        let nft_option = NftOptions {
                            address: Some(address.clone()),
                            sender: Some(address.clone()),
                            metadata: None,
                            tag: Some(json["StarSystem"].to_string().as_bytes().to_vec()),
                            issuer: None,
                            immutable_metadata: Some(irc.get_json().to_string().as_bytes().to_vec()),
                        };

                        //Will be needed for logging later of a error occures because of borrowed self
                        let lcoal_bech32_hrp = bech32_hrp.clone();

                        //We need the nft id so it can be the issuer
                        let issuer_nft_id_option = tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                            .unwrap()
                            .block_on(async {
                                //TODO Look if there is already a system nft existing
                                let mut system = vec![nft_option];
                                //let duplicates = nft_adapter::remove_already_existing(&mut system,account.client()).await;
                                let duplicates = self.remove_already_existing(&mut system).await;
                                if !duplicates.is_empty() {
                                    // https://discord.com/channels/397872799483428865/399178054909296640/1060465105772486807
                                    let output_id = duplicates.first().unwrap().metadata.output_id().unwrap();
                                    let issuer_nft_id = NftId::from(&output_id);
                                    debug!("Issuer Nft Id Found: {}",&issuer_nft_id);
                                    //let faucet_response = request_funds_from_faucet(variables.settings.json["faucet-url"].to_string().as_str(), Address::Nft(NftAddress::new(issuer_nft_id)).to_bech32(variables.bech32_hrp.clone()).as_str()).await.unwrap();
                                    //debug!("Faucet Response: {}",faucet_response);
                                    return Some(issuer_nft_id);
                                }
                                if system.is_empty() {
                                    return None;
                                }
                                //let attached_nft = nft_adapter::attach_nft(account,system).await;
                                let attached_nft = self.attach_nft(system).await;
                                match attached_nft.clone() {
                                    None => {}
                                    Some(transaction) => {
                                        let issuer_nft_id = NftId::from_str(transaction.transaction_id.to_string().as_str()).unwrap();
                                        return Some(issuer_nft_id);
                                    }
                                }

                                return None;
                            });
                        self.issuer_nft_id = issuer_nft_id_option;

                        match issuer_nft_id_option {
                            Some(nftid) => {
                                info!("Issuer set: {:?}",nftid.to_string());
                                info!("NFT Address: {:?}",Address::Nft(NftAddress::new(nftid.clone())).to_bech32(lcoal_bech32_hrp));
                            }
                            None => {
                                warn!("No system found!");
                            }
                        }
                    }
                    "Scan" => {
                        //Check if system is known as nft id so additional nft's can be added as collection -> otherwise the nft will be lost at fist
                        //TODO What to do if the system is not known? Throw the data away?
                        match issuer_nft_id.clone() {
                            None => {
                                error!("No issuer for body nft found: {}", json["BodyName"].to_string());
                            }
                            Some(_nft_id) => {

                                //system found -> nft can be created and attached to collection
                                //issuer is bugged at the moment -> just sending as normal nft without issuer

                                let irc = generate_irc27_from_json(json.clone());

                                let _nft_option = NftOptions {
                                    address: Some(address.clone()),
                                    sender: Some(address.clone()),
                                    metadata: None,
                                    tag: Some(json["BodyName"].to_string().as_bytes().to_vec()),
                                    issuer: Some(Address::Nft(NftAddress::new(issuer_nft_id.unwrap().clone())).to_bech32(bech32_hrp.clone())),
                                    immutable_metadata: Some(json.to_string().as_bytes().to_vec()),
                                };

                                let _nft_option = NftOptions {
                                    address: Some(Address::Nft(NftAddress::new(issuer_nft_id.unwrap().clone())).to_bech32(bech32_hrp.clone())),
                                    sender: Some(address.clone()),
                                    metadata: None,
                                    tag: Some(json["BodyName"].to_string().as_bytes().to_vec()),
                                    issuer: None,
                                    immutable_metadata: Some(json.to_string().as_bytes().to_vec()),
                                };

                                let nft_option = NftOptions {
                                    address: Some(address.clone()),
                                    sender: Some(address.clone()),
                                    metadata: None,
                                    tag: Some(json["BodyName"].to_string().as_bytes().to_vec()),
                                    issuer: None,
                                    immutable_metadata: Some(irc.get_json().to_string().as_bytes().to_vec()),
                                };

                                info!("New body nft: {:?}",nft_option);

                                queue.push(nft_option);
                            }
                        }
                    }
                    "StartJump" | "ApproachBody" | "LeaveBody" | "FSSAllBodiesFound" => {
                        self.flush_queue();
                    }
                    &_ => {}
                }
            }
        }
    }

    fn flush_queue(&mut self) {
        if !self.queue.is_empty() {
            debug!("Flushing queue");
            let mut local_queue = self.queue.clone();
            self.queue.clear();

            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async move {
                    self.remove_already_existing(&mut local_queue).await;
                    if self.queue.len() > 0 {
                        debug!("Queue not empty -> attaching");
                        self.attach_nft(local_queue).await;
                    }
                    debug!("Flushing complete");
                });
        } else {
            debug!("Queue empty -> not flushing");
        }
    }

    async fn remove_already_existing(&mut self, queue: &mut Vec<NftOptions>) -> Vec<OutputWithMetadataResponse> {
        let mut i = 0;
        let initial_queue_len = queue.len();

        let mut duplicates: Vec<OutputWithMetadataResponse> = vec![];

        while i < queue.len() {
            let tag: String = queue[i].clone().tag.unwrap().clone().to_hex();
            let output_ids = self.account.client().nft_output_ids(vec![QueryParameter::Tag(String::from("0x").add(tag.as_str()))]).await.unwrap();
            let nft_outputs = self.account.client().get_outputs(output_ids).await.unwrap();
            match nft_outputs.first() {
                Some(nft) => {
                    //Nft exists -> removing
                    duplicates.push(nft.clone());
                    info!("Nft already existing:{}{} Local:{}","https://hornet.paesserver.de/dashboard/explorer/output/",nft.metadata.block_id,String::from_utf8(queue[i].clone().tag.unwrap()).unwrap());
                    queue.remove(i);
                }
                None => {}
            }
            i += 1;
        }
        if initial_queue_len != queue.len() {
            info!("Already existing found -> Before pruning: {} After pruning: {}",initial_queue_len,queue.len());
        }
        return duplicates;
    }

    //TODO Maybe change it to vec<Transaction> sp the multiple transactions can be processed
    async fn attach_nft(&mut self, nft_options: Vec<NftOptions>) -> Option<Transaction> {
        // Mint nfts in chunks, since the transaction size is limited
        let mut last_transaction: Option<Transaction> = None;
        for nfts in nft_options.chunks(30) {
            last_transaction = self.attach_nft_recursive(nfts.to_vec(), 0).await
        }
        return last_transaction;
    }

    #[async_recursion]
    async fn attach_nft_recursive(&mut self, nft_options: Vec<NftOptions>, iteration: u64) -> Option<Transaction> {
        let timeout_in_seconds = self.settings.n_timeout;
        let attempts = self.settings.n_attempts;

        let mut balance = self.account.sync(Some(SyncOptions {
            force_syncing: true,
            ..Default::default()
        })).await.unwrap_or_default().base_coin.available;

        while balance < 1000000 {
            warn!("Not enough balance: {}", balance);
            tokio::time::sleep(Duration::from_secs(timeout_in_seconds)).await;
            balance = self.account.sync(Some(SyncOptions {
                force_syncing: true,
                ..Default::default()
            })).await.unwrap_or_default().base_coin.available;
        }

        let transaction_result = self.account.mint_nfts(nft_options.clone(), None).await;

        return match transaction_result {
            Ok(transaction) => {
                info!(
                    "NFT: {} Block sent: https://edcas.paesserver.de:15265/api/core/v2/blocks/{}",
                    transaction.transaction_id,
                    transaction.block_id.expect("no block created yet")
                );

                Some(transaction)
            }
            Err(err) => {
                debug!("{:?}",&err);
                match err {
                    Error::Client(err) => {
                        let error = *err;
                        if error.to_string().contains("missing input") {
                            let error_string = error.to_string().clone();
                            let address = error_string.split_at(49).1.split_at(66).0;
                            let nft_address = Address::Nft(NftAddress::new(address.parse().unwrap())).to_bech32("tst");
                            debug!("Address: {}", &nft_address);
                        }
                    }
                    _ => {}
                }
                tokio::time::sleep(Duration::from_secs(timeout_in_seconds)).await;
                if iteration < attempts {
                    self.attach_nft_recursive(nft_options, iteration + 1).await
                } else {
                    error!("Couldn't attach nft");
                    debug!("nft_options:{:?}", nft_options.first().unwrap().clone());
                    debug!("Data:{}", String::from_utf8(nft_options.first().unwrap().clone().immutable_metadata.unwrap()).unwrap());
                    None
                }
            }
        };
    }
}

pub fn initialize(tangle_bus_reader: BusReader<JsonValue>, settings: Settings) -> TangleInterpreter {
    info!("Loading wallet");

    let mut node_url = settings.base_url.clone();
    let port = settings.port.to_string();

    node_url.push_str(":");
    node_url.push_str(port.as_str());

    info!("Using URL:{}", &node_url);
    info!("Local POW:{}", settings.local_pow);

    let client_options = ClientOptions::new()
        .with_local_pow(settings.local_pow)
        .with_node(node_url.as_str()).unwrap();

    //create stronghold account
    let wallet_file_result = File::open("wallet.stronghold");

    let account = match wallet_file_result {
        Ok(file) => {
            debug!("{:?}", file);
            info!("Stronghold file found");
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async {
                    // Create the account manager
                    let manager = AccountManager::builder()
                        .with_client_options(client_options)
                        .with_coin_type(SHIMMER_COIN_TYPE)
                        .finish().await.unwrap();

                    // Set the stronghold password
                    manager
                        .set_stronghold_password(settings.password.as_str())
                        .await.unwrap();

                    // Get the account we generated with `01_create_wallet`
                    let account = manager.get_account("User").await.unwrap();

                    let balance = account.sync(None).await.unwrap();

                    info!("[Total: {} : Available: {}]",balance.base_coin.total,balance.base_coin.available);
                    info!("[NFTS Count: {}]",balance.nfts.len());
                    info!("[Req. storage deposit (basic): {}]",balance.required_storage_deposit.basic());

                    return account;
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
                        .password(settings.password.as_str())
                        .build(PathBuf::from("wallet.stronghold")).unwrap();

                    // Only required the first time, can also be generated with `manager.generate_mnemonic()?`
                    let mnemonic = generate_mnemonic().unwrap();

                    // The mnemonic only needs to be stored the first time
                    secret_manager.store_mnemonic(mnemonic).await.unwrap();

                    let manager = AccountManager::builder()
                        .with_secret_manager(iota_wallet::secret::SecretManager::Stronghold(secret_manager))
                        .with_client_options(client_options)
                        .with_coin_type(SHIMMER_COIN_TYPE)
                        .finish()
                        .await.unwrap();

                    // Create a new account
                    manager
                        .create_account()
                        .with_alias("User".to_string())
                        .finish()
                        .await.unwrap()
                })
        }
    };

    let account_balance: AccountBalance = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            return account.sync(None).await.unwrap();
        });

    //get address one time so it doesn't have to be created each time
    let address = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed creating addresses")
        .block_on(async {
            let address = account.addresses().await.unwrap()[0].address().to_bech32();
            debug!("{}", &address);
            info!("Address: {}",&address);
            return address;
        });

    let bech32_hrp = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed creating addresses")
        .block_on(async {
            return account.client().get_bech32_hrp().await.unwrap();
        });

    info!("Bech32: {}",&bech32_hrp);
    info!("Done loading wallet");

    TangleInterpreter {
        queue: vec![],
        bus: tangle_bus_reader,
        settings,
        account,
        bech32_hrp,
        address,
        issuer_nft_id: None,
    }
}

// https://docs.opensea.io/docs/metadata-standards
struct IRC27 {
    standard: String,
    version: String,
    description: String,
    media_type: String,
    uri: String,
    name: String,
    collection_name: String,
    attributes: Vec<Attribute>,
}

struct Attribute {
    trait_type: String,
    value: String,
}

impl IRC27 {
    fn get_json(&self) -> Value {
        let Self {
            standard, version, description, media_type, uri: image, name, collection_name: collection, attributes
        } = self;
        let mut array = json!([]);
        for attribute in attributes {
            array.as_array_mut().unwrap().push(attribute.get_json());
        }
        json!(
            {
                "standard": standard,
                "version": version,
                "description": description,
                "type": media_type,
                "uri": image,
                "name": name,
                "collectionName": collection,
                "attributes": array
            }
        )
    }
}

impl Attribute {
    fn get_json(&self) -> Value {
        let Self {
            trait_type, value
        } = self;
        json!(
            {
                "trait_type": trait_type,
                "value": value
            }
        )
    }
}

fn generate_irc27_from_json(json: JsonValue) -> IRC27 {
    let mut attributes: Vec<Attribute> = vec![];
    for entry in json.entries() {
        attributes.push(
            Attribute {
                trait_type: entry.0.to_string(),
                value: entry.1.to_string(),
            }
        );
    }

    let mut name = json["BodyName"].to_string();
    if name == "null" {
        name = json["StarSystem"].to_string();
    }

    IRC27 {
        standard: "IRC27".to_string(),
        version: "v1.0".to_string(),
        description: "".to_string(),
        media_type: "image/png".to_string(),
        uri: "https://edassets.org/static/img/ed-logos/elite-dangerous-minimalistic.png".to_string(),
        name,
        collection_name: json["StarSystem"].to_string(),
        attributes,
    }
}