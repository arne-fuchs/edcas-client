use ethers::prelude::Abigen;

fn main() {
    print!("Building contracts...");
    Abigen::new("EDCAS", "./contracts/EDCAS.abi")
        .unwrap()
        .generate()
        .unwrap()
        .write_to_file("src/app/evm_interpreter/edcas_contract.rs")
        .unwrap();
    println!("Done!");
}
