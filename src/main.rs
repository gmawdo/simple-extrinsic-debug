
use sp_keyring::AccountKeyring;
use sp_runtime::generic::Era;
use substrate_api_client::{
	ac_compose_macros::{compose_call, compose_extrinsic_offline},
	ac_primitives::{config::Config, DefaultRuntimeConfig, PlainTip, ExtrinsicParams, ExtrinsicSigner,GenericAdditionalParams},
	rpc::JsonrpseeClient,Api, GetChainInfo, SubmitAndWatch, XtStatus
};

// Import the necessary types from gm.rs
mod gm;
use gm::api::template_module::calls::types::Disassembled;
//use gm::api::runtime_apis::template_module::calls::types::Disassembled;

use tokio;

type Hash = <DefaultRuntimeConfig as Config>::Hash;
/// Get the balance type from your node runtime and adapt it if necessary.
type Balance = <DefaultRuntimeConfig as Config>::Balance;
/// We need AssetTip here, because the kitchensink runtime uses the asset pallet. Change to PlainTip if your node uses the balance pallet only.
type AdditionalParams = GenericAdditionalParams<PlainTip<Balance>, Hash>;

// To test this example with CI we run it against the Substrate kitchensink node, which uses the asset pallet.
// Therefore, we need to use the `AssetRuntimeConfig` in this example.
// ????????
// ! However, most Substrate runtimes do not use the asset pallet at all. So if you run an example against your own node
// you most likely should use `DefaultRuntimeConfig` instead.

#[tokio::main]
async fn main() {
    let node_url = "ws://127.0.0.1:9944";  // Example WebSocket URL
    let client = JsonrpseeClient::new(node_url).await.unwrap();

    let mut api = Api::<DefaultRuntimeConfig, _>::new(client).await.unwrap();
    let signer = AccountKeyring::Alice.pair();
    let extrinsic_signer = ExtrinsicSigner::<DefaultRuntimeConfig>::new(signer);
    api.set_signer(extrinsic_signer.clone());

    let metadata = api.metadata();
    metadata.print_overview();

    let genesis_hash = api.genesis_hash();
    println!("Genesis Hash: {:?}", genesis_hash);  // Print genesis hash

    let runtime_version = api.runtime_version();
    println!("Runtime Version: {:?}", runtime_version);  // Print runtime version

    let transaction_version = api.runtime_version().transaction_version;
    println!("Transaction Version: {:?}", transaction_version);  // Print transaction version

    let spec_version = api.runtime_version().spec_version;
    println!("Spec Version: {:?}", spec_version);  // Print spec version

    let signer_nonce = api.get_nonce().await.unwrap();
    println!("Signer Nonce: {:?}", signer_nonce);  // Print nonce

    let disassembled_call = Disassembled {
        creation_time: b"2024GMTest".to_vec(),
        file_path: b"/path/test.txt".to_vec(),
        event_key: b"update".to_vec(),
    };

    let call = compose_call!(
        metadata,
        "TemplateModule",
        "disassembled",
        disassembled_call.creation_time,
        disassembled_call.file_path,
        disassembled_call.event_key
    ).unwrap();

    	// Get the last finalized header to retrieve information for Era for mortal transactions (online).
	let last_finalized_header_hash = api.get_finalized_head().await.unwrap().unwrap();
	let header = api.get_header(Some(last_finalized_header_hash)).await.unwrap().unwrap();
    println!("Last Finalized Header Hash: {:?}", last_finalized_header_hash);  // Print last finalized header hash
    println!("Header: {:?}", header);  // Print header details

	let period = 5; ////?????
    // Define additional parameters for the extrinsic
	let additional_extrinsic_params: AdditionalParams = GenericAdditionalParams::new()
		.era(Era::mortal(period, header.number.into()), last_finalized_header_hash)
		.tip(0);

    let extrinsic_params = <DefaultRuntimeConfig as Config>::ExtrinsicParams::new(
        spec_version,
        transaction_version,
        signer_nonce,
        genesis_hash,
        additional_extrinsic_params,
    );

    let xt = compose_extrinsic_offline!(extrinsic_signer, call, extrinsic_params);
    let hash = api.submit_and_watch_extrinsic_until(xt, XtStatus::InBlock).await.unwrap().block_hash.unwrap();
    println!("[+] Extrinsic got included in block {:?}", hash);
}