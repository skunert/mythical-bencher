use frame_benchmarking_cli::{BenchmarkCmd, ExtrinsicBuilder, OpaqueBlock, fetch_latest_metadata_from_blob, fetch_relevant_runtime_data};
use clap::Parser;
use subxt::ext::sp_runtime::OpaqueExtrinsic;
use subxt::{Config, OfflineClient};
use subxt::client::RuntimeVersion;
use subxt::config::substrate::{BlakeTwo256, SubstrateExtrinsicParamsBuilder, SubstrateHeader};
use subxt::config::SubstrateExtrinsicParams;
use subxt::utils::H256;
use subxt_signer::eth::AccountId20;

#[derive(Parser, Debug)]
pub struct Command {
    #[command(subcommand)]
    pub sub: BenchmarkCmd,
    #[arg(long)]
    pub genesis_hash: String,
}
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum MythicalConfig {}
impl Config for MythicalConfig {
    type Hash = H256;
    type AccountId = AccountId20;
    // type Address = MultiAddress<Self::AccountId, u32>;
    type Address = Self::AccountId;
    type Signature = subxt_signer::eth::Signature;
    type Hasher = BlakeTwo256;
    type Header = SubstrateHeader<u32, BlakeTwo256>;
    type ExtrinsicParams = SubstrateExtrinsicParams<Self>;
    type AssetId = u32;
}

impl EthExtrinsicBuilder {
    pub fn new(
        metadata: subxt::Metadata,
        genesis_hash: H256,
        runtime_version: RuntimeVersion,
    ) -> Self {
        Self { offline_client: OfflineClient::new(genesis_hash, runtime_version, metadata) }
    }
}

struct EthExtrinsicBuilder {
    pub offline_client: OfflineClient<MythicalConfig>
}

impl ExtrinsicBuilder for EthExtrinsicBuilder {
    fn pallet(&self) -> &str {
        "system"
    }

    fn extrinsic(&self) -> &str {
        "remark"
    }

    fn build(&self, nonce: u32) -> Result<sp_runtime::OpaqueExtrinsic, &'static str> {
        // let signer = MySigner(sp_keyring::Sr25519Keyring::Bob.pair());
        let signer = subxt_signer::eth::dev::alith();
        tracing::info!("Signing with account: {}, nonce: {}", hex::encode(signer.account_id().0), nonce);
        let dynamic_tx = subxt::dynamic::tx("System", "remark", vec![vec!['a', 'b', 'b']]);
        let params = SubstrateExtrinsicParamsBuilder::<MythicalConfig>::new().nonce(nonce.into()).build();
        // Default transaction parameters assume a nonce of 0.
        let transaction = self
            .offline_client
            .tx()
            .create_signed_offline(&dynamic_tx, &signer, params)
            .unwrap();
        let mut encoded = transaction.into_encoded();
        sp_runtime::OpaqueExtrinsic::from_bytes(&mut encoded).map_err(|_| "Unable to construct OpaqueExtrinsic")
    }
}

fn main() {
    env_logger::init();
    let mut cli = Command::parse();
    if let BenchmarkCmd::Overhead(cmd) = cli.sub {
        let provider = Box::new(|metadata, genesis_hash, runtime_version| {
            EthExtrinsicBuilder::new(metadata, genesis_hash, runtime_version)
        });
         if let Err(e) = cmd.run_with_extrinsic_builder::<OpaqueBlock, ()>(Some(provider)) {
                tracing::error!("Failed to run benchmark: {:?}", e);
         };
    }
}
