use frame_benchmarking_cli::{BenchmarkCmd, ExtrinsicBuilder, OpaqueBlock};
use clap::Parser;
use subxt::{Config, OfflineClient};
use subxt::client::RuntimeVersion;
use subxt::config::substrate::{BlakeTwo256, SubstrateExtrinsicParamsBuilder, SubstrateHeader};
use subxt::config::SubstrateExtrinsicParams;
use subxt::utils::H256;
use sp_core::H256 as SubstrateHash;
use subxt_signer::eth::AccountId20;

#[derive(Parser, Debug)]
pub struct Command {
    #[command(subcommand)]
    pub sub: BenchmarkCmd,
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
        let params = SubstrateExtrinsicParamsBuilder::<MythicalConfig>::new().nonce(nonce.into()).build();

        let signer = subxt_signer::eth::dev::alith();
        let dynamic_tx = subxt::dynamic::tx("System", "remark", vec![Vec::<u8>::new()]);
        let transaction = self
            .offline_client
            .tx()
            .create_signed_offline(&dynamic_tx, &signer, params)
            .unwrap();
        sp_runtime::OpaqueExtrinsic::from_bytes(&transaction.into_encoded()).map_err(|_| "Unable to construct OpaqueExtrinsic")
    }
}

fn main() {
    env_logger::init();
    let cli = Command::parse();
    if let BenchmarkCmd::Overhead(cmd) = cli.sub {
        let extrinsic_builder_provider = Box::new(|metadata, genesis_hash: SubstrateHash, runtime_version| {
            let genesis_hash = H256::from(genesis_hash.to_fixed_bytes());
            Box::new(EthExtrinsicBuilder::new(metadata, genesis_hash, runtime_version)) as Box<_>
        });
         if let Err(e) = cmd.run_with_extrinsic_builder::<OpaqueBlock, ()>(Some(extrinsic_builder_provider)) {
                tracing::error!("Failed to run benchmark: {:?}", e);
         };
    }
}
