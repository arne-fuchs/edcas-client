use ethers::prelude::*;

pub fn upload_logs() {
    println!("nothing here for now");
    return;
    //TODO: start evm uploader thread
    //TODO: start journal reader thread
    //TODO: feed data from journal reader to evm uploader

    //TODO: ask Frank what i have to do and how to actually do that.
}

pub fn set_sc_address(smart_contract_address: String) {
    let addr = smart_contract_address
        .parse::<Address>()
        .unwrap_or_else(|_| panic!("Address is incorrect"));
    println!("nothing here yet");
    //TODO: take that sc_address and put it either directly in json or in the client.settings and
    //then call save
    return;
}
