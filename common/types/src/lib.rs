#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

use alloc::vec::Vec;
use codec::{Decode, Encode};
use core::fmt::Debug;

// Messages: Phase Wallet

pub mod messaging {
    use alloc::string::String;
    use alloc::vec::Vec;
    use codec::{Decode, Encode};
    use core::fmt::Debug;
    use sp_core::U256;

    use super::{WorkerEcdhPublicKey, WorkerPublicKey};
    pub use phala_mq::bind_topic;
    pub use phala_mq::types::*;

    #[derive(Encode, Decode, Debug)]
    pub struct PushCommand<Cmd> {
        pub command: Cmd,
        pub number: u64,
    }

    impl<Cmd: BindTopic> BindTopic for PushCommand<Cmd> {
        const TOPIC: &'static [u8] = <Cmd as BindTopic>::TOPIC;
    }

    // TODO.kevin:
    //    We should create a crate for each contract just like developing apps.
    //    Then the following types should be put in their own crates.
    // Messages: Lottery

    bind_topic!(Lottery, b"^phala/BridgeTransfer");
    #[derive(Encode, Decode, Clone, Debug)]
    pub enum Lottery {
        SignedTx {
            round_id: u32,
            token_id: Vec<u8>,
            tx: Vec<u8>,
        },
        BtcAddresses {
            address_set: Vec<Vec<u8>>,
        },
    }

    bind_topic!(LotteryCommand, b"phala/lottery/command");
    #[derive(Encode, Decode, Debug)]
    pub enum LotteryCommand {
        SubmitUtxo {
            round_id: u32,
            address: String,
            utxo: (Txid, u32, u64),
        },
        SetAdmin {
            new_admin: String,
        },
    }

    pub type Txid = [u8; 32];

    // Messages for Balances

    bind_topic!(BalanceEvent<AccountId, Balance>, b"phala/balances/event");
    #[derive(Debug, Clone, Encode, Decode)]
    pub enum BalanceEvent<AccountId, Balance> {
        TransferToTee(AccountId, Balance),
    }

    bind_topic!(BalanceCommand<AccountId, Balance>, b"phala/balances/command");
    #[derive(Debug, Clone, Encode, Decode)]
    pub enum BalanceCommand<AccountId, Balance> {
        Transfer { dest: AccountId, value: Balance },
        TransferToChain { dest: AccountId, value: Balance },
    }

    bind_topic!(BalanceTransfer<AccountId, Balance>, b"^phala/balances/transfer");
    #[derive(Encode, Decode)]
    pub struct BalanceTransfer<AccountId, Balance> {
        pub dest: AccountId,
        pub amount: Balance,
    }

    // Messages for Assets

    bind_topic!(AssetCommand<AccountId, Balance>, b"phala/assets/command");
    #[derive(Encode, Decode, Debug)]
    pub enum AssetCommand<AccountId, Balance> {
        Issue {
            symbol: String,
            total: Balance,
        },
        Destroy {
            id: AssetId,
        },
        Transfer {
            id: AssetId,
            dest: AccountId,
            value: Balance,
        },
    }

    pub type AssetId = u32;

    // Messages for Web3Analytics

    bind_topic!(Web3AnalyticsCommand, b"phala/web3analytics/command");
    #[derive(Encode, Decode, Debug)]
    pub enum Web3AnalyticsCommand {
        SetConfiguration { skip_stat: bool },
    }

    // Messages for diem

    bind_topic!(DiemCommand, b"phala/diem/command");
    #[derive(Encode, Decode, Debug)]
    pub enum DiemCommand {
        /// Sets the whitelisted accounts, in bcs encoded base64
        AccountInfo {
            account_info_b64: String,
        },
        /// Verifies a transactions
        VerifyTransaction {
            account_address: String,
            transaction_with_proof_b64: String,
        },
        /// Sets the trusted state. The owner can only initialize the bridge with the genesis state
        /// once.
        SetTrustedState {
            trusted_state_b64: String,
            chain_id: u8,
        },
        VerifyEpochProof {
            ledger_info_with_signatures_b64: String,
            epoch_change_proof_b64: String,
        },

        NewAccount {
            seq_number: u64,
        },
        TransferXUS {
            to: String,
            amount: u64,
        },
    }

    // Messages for Kitties

    bind_topic!(KittyEvent<AccountId, Hash>, b"phala/kitties/event");
    #[derive(Encode, Decode, Debug)]
    pub enum KittyEvent<AccountId, Hash> {
        Created(AccountId, Hash),
    }

    bind_topic!(KittyTransfer<AccountId>, b"^phala/kitties/transfer");
    #[derive(Debug, Clone, Encode, Decode, PartialEq)]
    pub struct KittyTransfer<AccountId> {
        pub dest: AccountId,
        pub kitty_id: Vec<u8>,
    }

    // Messages: System
    #[derive(Encode, Decode, Debug)]
    pub struct WorkerEventWithKey {
        pub pubkey: WorkerPublicKey,
        pub event: WorkerEvent,
    }

    #[derive(Encode, Decode, Debug)]
    pub enum WorkerEvent {
        Registered,
        BenchStart { start_time: u64 },
        MiningStart { start_time: u64 },
        MiningStop,
        MiningEnterUnresponsive,
        MiningExitUnresponsive,
    }

    bind_topic!(SystemEvent, b"phala/system/event");
    #[derive(Encode, Decode, Debug)]
    pub enum SystemEvent {
        WorkerEvent(WorkerEventWithKey),
        HeartbeatChallenge(HeartbeatChallenge),
    }

    impl SystemEvent {
        pub fn new_worker_event(pubkey: WorkerPublicKey, event: WorkerEvent) -> SystemEvent {
            SystemEvent::WorkerEvent(WorkerEventWithKey { pubkey, event })
        }
    }

    #[derive(Encode, Decode, Debug, Default, Clone, PartialEq, Eq)]
    pub struct HeartbeatChallenge {
        pub seed: U256,
        pub online_target: U256,
    }

    bind_topic!(MiningReportEvent, b"phala/mining/report");
    #[derive(Encode, Decode, Clone, Debug)]
    pub enum MiningReportEvent {
        Heartbeat {
            /// The block number that this message been sent at
            block_num: u32,
            /// Mining start timestamp, extracted from WorkerEvent::MiningStart.
            mining_start_time: u64,
            /// Benchmark iterations since mining_start_time.
            iterations: u64,
        }
    }

    bind_topic!(MiningInfoUpdateEvent, b"^phala/mining/update");
    #[derive(Encode, Decode, Clone, Debug, Default)]
    pub struct MiningInfoUpdateEvent {
        // Workers that do not responce the heartbeat challenge in time. Each delay only report once.
        pub offline: Vec<WorkerPublicKey>,

        // V update and payout info
        pub settle: Vec<SettleInfo>

        // NOTE: Take care of the is_empty method when adding fields
    }

    impl MiningInfoUpdateEvent {
        pub fn is_empty(&self) -> bool {
            self.offline.is_empty() && self.settle.is_empty()
        }
    }

    #[derive(Encode, Decode, Clone, Debug)]
    pub struct SettleInfo {
        pub pubkey: WorkerPublicKey,
        pub v: u32,
        pub payout: u32,
    }

    // Messages: Gatekeeper
    // TODO.shelven: place them under a universial GatekeeperEvent
    bind_topic!(NewGatekeeperEvent, b"phala/gatekeeper/new");
    #[derive(Encode, Decode, Debug)]
    pub struct NewGatekeeperEvent {
        pub pubkey: WorkerPublicKey,
        pub ecdh_pubkey: WorkerEcdhPublicKey,
        pub gatekeeper_count: u32,
    }

    bind_topic!(DispatchMasterKeyEvent, b"phala/gatekeeper/masterkey");
    #[derive(Encode, Decode, Debug)]
    pub struct DispatchMasterKeyEvent {
        pub dest: WorkerPublicKey,
        pub ecdh_pubkey: WorkerEcdhPublicKey,
        pub encrypted_master_key: Vec<u8>,
        pub iv: phala_crypto::aead::IV,
    }

    pub type RandomNumber = [u8; 32];

    bind_topic!(RandomNumberEvent, b"phala/gatekeeper/random");
    #[derive(Encode, Decode, Clone, Debug)]
    pub struct RandomNumberEvent {
        pub block_number: u32,
        pub random_number: RandomNumber,
        pub last_random_number: RandomNumber,
    }
}

// Types used in storage

#[derive(Encode, Decode, PartialEq, Eq, Debug, Clone)]
pub enum WorkerStateEnum<BlockNumber> {
    Empty,
    Free,
    Gatekeeper,
    MiningPending,
    Mining(BlockNumber),
    MiningStopping,
}

impl<BlockNumber> Default for WorkerStateEnum<BlockNumber> {
    fn default() -> Self {
        WorkerStateEnum::Empty
    }
}

#[derive(Encode, Decode, Debug, Default, Clone)]
pub struct WorkerInfo<BlockNumber> {
    // identity
    pub machine_id: Vec<u8>,
    pub pubkey: Vec<u8>,
    pub last_updated: u64,
    // mining
    pub state: WorkerStateEnum<BlockNumber>,
    // performance
    pub score: Option<Score>,
    // confidence-level
    pub confidence_level: u8,
    // version
    pub runtime_version: u32,
}

#[derive(Encode, Decode, Default)]
pub struct StashInfo<AccountId: Default> {
    pub controller: AccountId,
    pub payout_prefs: PayoutPrefs<AccountId>,
}

#[derive(Encode, Decode, Default)]
pub struct PayoutPrefs<AccountId: Default> {
    pub commission: u32,
    pub target: AccountId,
}

#[derive(Encode, Decode, Debug, Default, Clone)]
pub struct Score {
    pub overall_score: u32,
    pub features: Vec<u32>,
}

type MachineId = [u8; 16];
pub type WorkerPublicKey = sp_core::ecdsa::Public;
// This is only true for ECDH_P256
pub type WorkerEcdhPublicKey = [u8; 65];
pub type ContractPublicKey = sp_core::ecdsa::Public;

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq)]
pub struct PRuntimeInfo<AccountId> {
    pub version: u32,
    pub machine_id: MachineId,
    pub pubkey: WorkerPublicKey,
    pub features: Vec<u32>,
    pub operator: Option<AccountId>,
}

#[derive(Encode, Decode, Debug, Default)]
pub struct RoundInfo<BlockNumber> {
    pub round: u32,
    pub start_block: BlockNumber,
}

#[derive(Encode, Decode, Debug, Default)]
pub struct StashWorkerStats<Balance> {
    pub slash: Balance,
    pub compute_received: Balance,
    pub online_received: Balance,
}

#[derive(Encode, Decode, Debug, Default, Clone, PartialEq, Eq)]
pub struct RoundStats {
    pub round: u32,
    pub online_workers: u32,
    pub compute_workers: u32,
    /// The targeted online reward counts in fraction (base: 100_000)
    pub frac_target_online_reward: u32,
    pub total_power: u32,
    /// The targeted compute reward counts in fraction (base: 100_000)
    pub frac_target_compute_reward: u32,
}

#[derive(Encode, Decode, Debug, Default, Clone, PartialEq, Eq)]
pub struct MinerStatsDelta {
    pub num_worker: i32,
    pub num_power: i32,
}

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq)]
pub enum PayoutReason {
    OnlineReward,
    ComputeReward,
}

impl Default for PayoutReason {
    fn default() -> Self {
        PayoutReason::OnlineReward
    }
}
