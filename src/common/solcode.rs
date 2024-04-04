use std::path::Path;
use anyhow::{Result, anyhow};

use solana_ledger::blockstore::Blockstore;
use solana_sdk::{genesis_config::{GenesisConfig, DEFAULT_GENESIS_ARCHIVE, DEFAULT_GENESIS_FILE}, 
    signature::Keypair,
    hash::Hash};

pub fn create_new_ledger(
    ledger_path: &Path,
    genesis_config: &GenesisConfig,
    max_genesis_archive_unpacked_size: u64,
    column_options: String // LedgerColumnOptions,
) -> Result<Hash> {
    // Blockstore::destroy(ledger_path)?;
    // genesis_config.write(ledger_path)?;

    // Fill slot 0 with ticks that link back to the genesis_config to bootstrap the ledger.
    let blockstore_dir = column_options.shred_storage_type.blockstore_directory();
    let blockstore = Blockstore::open_with_options(
        ledger_path,
        BlockstoreOptions {
            access_type: AccessType::Primary,
            recovery_mode: None,
            enforce_ulimit_nofile: false,
            column_options: column_options.clone(),
        },
    )?;
    let ticks_per_slot = genesis_config.ticks_per_slot;
    let hashes_per_tick = genesis_config.poh_config.hashes_per_tick.unwrap_or(0);
    let entries = create_ticks(ticks_per_slot, hashes_per_tick, genesis_config.hash());
    let last_hash = entries.last().unwrap().hash;
    let version = solana_sdk::shred_version::version_from_hash(&last_hash);

    let shredder = Shredder::new(0, 0, 0, version).unwrap();
    let (shreds, _) = shredder.entries_to_shreds(
        &Keypair::new(),
        &entries,
        true, // is_last_in_slot
        0,    // next_shred_index
        0,    // next_code_index
        true, // merkle_variant
        &ReedSolomonCache::default(),
        &mut ProcessShredsStats::default(),
    );
    assert!(shreds.last().unwrap().last_in_slot());

    blockstore.insert_shreds(shreds, None, false)?;
    blockstore.set_roots(std::iter::once(&0))?;
    // Explicitly close the blockstore before we create the archived genesis file
    drop(blockstore);

    let archive_path = ledger_path.join(DEFAULT_GENESIS_ARCHIVE);
    let args = vec![
        "jcfhS",
        archive_path.to_str().unwrap(),
        "-C",
        ledger_path.to_str().unwrap(),
        DEFAULT_GENESIS_FILE,
        blockstore_dir,
    ];
    let output = std::process::Command::new("tar")
        .args(args)
        .output()
        .unwrap();
    if !output.status.success() {
        use std::str::from_utf8;
        anyhow!("tar stdout: {}", from_utf8(&output.stdout).unwrap_or("?"));
        anyhow!("tar stderr: {}", from_utf8(&output.stderr).unwrap_or("?"));

        // return Err(BlockstoreError::Io(IoError::new(
            // ErrorKind::Other,
            anyhow!(
                "Error trying to generate snapshot archive: {}",
                output.status
            );
        // )));
    }
    Ok(Hash::new_unique())

    // ensure the genesis archive can be unpacked and it is under
    // max_genesis_archive_unpacked_size, immediately after creating it above.
    // {
        // let temp_dir = tempfile::tempdir_in(ledger_path).unwrap();
        // unpack into a temp dir, while completely discarding the unpacked files
        // let unpack_check = unpack_genesis_archive(
        //     &archive_path,
        //     temp_dir.path(),
        //     max_genesis_archive_unpacked_size,
        // );
        // if let Err(unpack_err) = unpack_check {
            // stash problematic original archived genesis related files to
            // examine them later and to prevent validator and ledger-tool from
            // naively consuming them
            // let mut error_messages = String::new();

            // fs::rename(
            //     ledger_path.join(DEFAULT_GENESIS_ARCHIVE),
            //     ledger_path.join(format!("{DEFAULT_GENESIS_ARCHIVE}.failed")),
            // )
            // .unwrap_or_else(|e| {
            //     let _ = write!(
            //         &mut error_messages,
            //         "/failed to stash problematic {DEFAULT_GENESIS_ARCHIVE}: {e}"
            //     );
            // });
            // fs::rename(
            //     ledger_path.join(DEFAULT_GENESIS_FILE),
            //     ledger_path.join(format!("{DEFAULT_GENESIS_FILE}.failed")),
            // )
            // .unwrap_or_else(|e| {
            //     let _ = write!(
            //         &mut error_messages,
            //         "/failed to stash problematic {DEFAULT_GENESIS_FILE}: {e}"
            //     );
            // });
            // fs::rename(
            //     ledger_path.join(blockstore_dir),
            //     ledger_path.join(format!("{blockstore_dir}.failed")),
            // )
            // .unwrap_or_else(|e| {
            //     let _ = write!(
            //         &mut error_messages,
            //         "/failed to stash problematic {blockstore_dir}: {e}"
            //     );
            // });

            // return Err(BlockstoreError::Io(IoError::new(
            //     ErrorKind::Other,
            //     format!("Error checking to unpack genesis archive: {unpack_err}{error_messages}"),
            // )));
        // }
    // }

    // Ok(last_hash)
}

    // snapshot_config::SnapshotConfig,

pub struct Blockstore {
    ledger_path: PathBuf,
    db: Arc<Database>,
    meta_cf: LedgerColumn<cf::SlotMeta>,
    dead_slots_cf: LedgerColumn<cf::DeadSlots>,
    duplicate_slots_cf: LedgerColumn<cf::DuplicateSlots>,
    roots_cf: LedgerColumn<cf::Root>,
    erasure_meta_cf: LedgerColumn<cf::ErasureMeta>,
    orphans_cf: LedgerColumn<cf::Orphans>,
    index_cf: LedgerColumn<cf::Index>,
    data_shred_cf: LedgerColumn<cf::ShredData>,
    code_shred_cf: LedgerColumn<cf::ShredCode>,
    transaction_status_cf: LedgerColumn<cf::TransactionStatus>,
    address_signatures_cf: LedgerColumn<cf::AddressSignatures>,
    transaction_memos_cf: LedgerColumn<cf::TransactionMemos>,
    transaction_status_index_cf: LedgerColumn<cf::TransactionStatusIndex>,
    highest_primary_index_slot: RwLock<Option<Slot>>,
    rewards_cf: LedgerColumn<cf::Rewards>,
    blocktime_cf: LedgerColumn<cf::Blocktime>,
    perf_samples_cf: LedgerColumn<cf::PerfSamples>,
    block_height_cf: LedgerColumn<cf::BlockHeight>,
    program_costs_cf: LedgerColumn<cf::ProgramCosts>,
    bank_hash_cf: LedgerColumn<cf::BankHash>,
    optimistic_slots_cf: LedgerColumn<cf::OptimisticSlots>,
    max_root: AtomicU64,
    merkle_root_meta_cf: LedgerColumn<cf::MerkleRootMeta>,
    insert_shreds_lock: Mutex<()>,
    new_shreds_signals: Mutex<Vec<Sender<bool>>>,
    completed_slots_senders: Mutex<Vec<CompletedSlotsSender>>,
    pub shred_timing_point_sender: Option<PohTimingSender>,
    pub lowest_cleanup_slot: RwLock<Slot>,
    pub slots_stats: SlotsStats,
    rpc_api_metrics: BlockstoreRpcApiMetrics,
}