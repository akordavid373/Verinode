use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::{clock::Clock, rent::Rent, Sysvar},
};
use std::collections::HashMap;
use sha2::{Sha256, Digest};

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct AtomicSwap {
    pub swap_id: String,
    pub initiator: Pubkey,
    pub participant: Option<Pubkey>,
    pub initiator_chain: u64,
    pub participant_chain: u64,
    pub initiator_asset: AssetInfo,
    pub participant_asset: AssetInfo,
    pub initiator_deposit: Option<DepositInfo>,
    pub participant_deposit: Option<DepositInfo>,
    pub status: SwapStatus,
    pub secret_hash: Vec<u8>,
    pub secret: Option<Vec<u8>>,
    pub timelock: u64,
    pub created_at: u64,
    pub expires_at: u64,
    pub refund_initiator: bool,
    pub refund_participant: bool,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct AssetInfo {
    pub token_address: Vec<u8>,
    pub amount: u64,
    pub decimals: u8,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct DepositInfo {
    pub transaction_hash: Vec<u8>,
    pub block_number: u64,
    pub confirmed_at: u64,
    pub proof: Vec<u8>,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum SwapStatus {
    Initiated,
    Deposited,
    Redeemed,
    Refunded,
    Expired,
    Cancelled,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct SwapState {
    pub is_initialized: bool,
    pub authority: Pubkey,
    pub active_swaps: HashMap<String, AtomicSwap>,
    pub completed_swaps: Vec<String>,
    pub swap_stats: SwapStats,
    pub fee_rate: u64,
    pub min_timelock: u64,
    pub max_timelock: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct SwapStats {
    pub total_swaps: u64,
    pub completed_swaps: u64,
    pub refunded_swaps: u64,
    pub expired_swaps: u64,
    pub total_volume: u64,
    pub average_swap_time: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct InitSwapArgs {
    pub swap_id: String,
    pub initiator_chain: u64,
    pub participant_chain: u64,
    pub initiator_asset: AssetInfo,
    pub participant_asset: AssetInfo,
    pub secret_hash: Vec<u8>,
    pub timelock: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct ParticipateSwapArgs {
    pub swap_id: String,
    pub participant: Pubkey,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct DepositArgs {
    pub swap_id: String,
    pub transaction_hash: Vec<u8>,
    pub block_number: u64,
    pub proof: Vec<u8>,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct RedeemArgs {
    pub swap_id: String,
    pub secret: Vec<u8>,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct RefundArgs {
    pub swap_id: String,
    pub is_initiator: bool,
}

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = AtomicSwapInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        AtomicSwapInstruction::InitializeSwapState(args) => {
            initialize_swap_state(program_id, accounts, args)
        }
        AtomicSwapInstruction::InitiateSwap(args) => {
            initiate_swap(program_id, accounts, args)
        }
        AtomicSwapInstruction::ParticipateSwap(args) => {
            participate_swap(program_id, accounts, args)
        }
        AtomicSwapInstruction::Deposit(args) => {
            deposit(program_id, accounts, args)
        }
        AtomicSwapInstruction::Redeem(args) => {
            redeem(program_id, accounts, args)
        }
        AtomicSwapInstruction::Refund(args) => {
            refund(program_id, accounts, args)
        }
        AtomicSwapInstruction::CancelSwap(swap_id) => {
            cancel_swap(program_id, accounts, swap_id)
        }
        AtomicSwapInstruction::UpdateFeeRate(new_rate) => {
            update_fee_rate(program_id, accounts, new_rate)
        }
    }
}

pub fn initialize_swap_state(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: InitSwapStateArgs,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let swap_account = next_account_info(accounts_iter)?;
    let authority_account = next_account_info(accounts_iter)?;

    if !authority_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut swap_data = SwapState::try_from_slice(&swap_account.data.borrow())
        .unwrap_or_else(|_| SwapState {
            is_initialized: false,
            authority: Pubkey::default(),
            active_swaps: HashMap::new(),
            completed_swaps: Vec::new(),
            swap_stats: SwapStats {
                total_swaps: 0,
                completed_swaps: 0,
                refunded_swaps: 0,
                expired_swaps: 0,
                total_volume: 0,
                average_swap_time: 0,
            },
            fee_rate: 1000, // 0.1%
            min_timelock: 3600, // 1 hour
            max_timelock: 86400 * 7, // 7 days
        });

    if swap_data.is_initialized {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    swap_data.is_initialized = true;
    swap_data.authority = args.authority;

    swap_data.serialize(&mut &mut swap_account.data.borrow_mut()[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;

    msg!("Atomic swap state initialized with authority: {:?}", args.authority);
    Ok(())
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct InitSwapStateArgs {
    pub authority: Pubkey,
}

pub fn initiate_swap(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: InitSwapArgs,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let swap_account = next_account_info(accounts_iter)?;
    let initiator_account = next_account_info(accounts_iter)?;

    if !initiator_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut swap_data = SwapState::try_from_slice(&swap_account.data.borrow())?;

    if swap_data.active_swaps.contains_key(&args.swap_id) {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    if args.timelock < swap_data.min_timelock || args.timelock > swap_data.max_timelock {
        return Err(ProgramError::InvalidArgument);
    }

    let clock = Clock::get()?;
    let current_time = clock.unix_timestamp as u64;

    let atomic_swap = AtomicSwap {
        swap_id: args.swap_id.clone(),
        initiator: *initiator_account.key,
        participant: None,
        initiator_chain: args.initiator_chain,
        participant_chain: args.participant_chain,
        initiator_asset: args.initiator_asset,
        participant_asset: args.participant_asset,
        initiator_deposit: None,
        participant_deposit: None,
        status: SwapStatus::Initiated,
        secret_hash: args.secret_hash.clone(),
        secret: None,
        timelock: args.timelock,
        created_at: current_time,
        expires_at: current_time + args.timelock,
        refund_initiator: false,
        refund_participant: false,
    };

    swap_data.active_swaps.insert(args.swap_id.clone(), atomic_swap);
    swap_data.swap_stats.total_swaps += 1;
    swap_data.swap_stats.total_volume += args.initiator_asset.amount;

    swap_data.serialize(&mut &mut swap_account.data.borrow_mut()[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;

    msg!("Initiated atomic swap: {}", args.swap_id);
    Ok(())
}

pub fn participate_swap(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: ParticipateSwapArgs,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let swap_account = next_account_info(accounts_iter)?;
    let participant_account = next_account_info(accounts_iter)?;

    if !participant_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut swap_data = SwapState::try_from_slice(&swap_account.data.borrow())?;

    let atomic_swap = swap_data.active_swaps.get_mut(&args.swap_id)
        .ok_or(ProgramError::InvalidArgument)?;

    if atomic_swap.participant.is_some() {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    if atomic_swap.status != SwapStatus::Initiated {
        return Err(ProgramError::InvalidAccountData);
    }

    atomic_swap.participant = Some(args.participant);
    atomic_swap.status = SwapStatus::Deposited;

    swap_data.serialize(&mut &mut swap_account.data.borrow_mut()[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;

    msg!("Participant joined swap: {}", args.swap_id);
    Ok(())
}

pub fn deposit(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: DepositArgs,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let swap_account = next_account_info(accounts_iter)?;
    let depositor_account = next_account_info(accounts_iter)?;

    if !depositor_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut swap_data = SwapState::try_from_slice(&swap_account.data.borrow())?;

    let atomic_swap = swap_data.active_swaps.get_mut(&args.swap_id)
        .ok_or(ProgramError::InvalidArgument)?;

    let clock = Clock::get()?;
    let current_time = clock.unix_timestamp as u64;

    let deposit_info = DepositInfo {
        transaction_hash: args.transaction_hash,
        block_number: args.block_number,
        confirmed_at: current_time,
        proof: args.proof,
    };

    if atomic_swap.initiator == *depositor_account.key {
        if atomic_swap.initiator_deposit.is_some() {
            return Err(ProgramError::AccountAlreadyInitialized);
        }
        atomic_swap.initiator_deposit = Some(deposit_info);
    } else if atomic_swap.participant == Some(*depositor_account.key) {
        if atomic_swap.participant_deposit.is_some() {
            return Err(ProgramError::AccountAlreadyInitialized);
        }
        atomic_swap.participant_deposit = Some(deposit_info);
    } else {
        return Err(ProgramError::InvalidAccountData);
    }

    if atomic_swap.initiator_deposit.is_some() && atomic_swap.participant_deposit.is_some() {
        atomic_swap.status = SwapStatus::Deposited;
    }

    swap_data.serialize(&mut &mut swap_account.data.borrow_mut()[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;

    msg!("Deposit confirmed for swap: {}", args.swap_id);
    Ok(())
}

pub fn redeem(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: RedeemArgs,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let swap_account = next_account_info(accounts_iter)?;
    let redeemer_account = next_account_info(accounts_iter)?;

    if !redeemer_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut swap_data = SwapState::try_from_slice(&swap_account.data.borrow())?;

    let atomic_swap = swap_data.active_swaps.get_mut(&args.swap_id)
        .ok_or(ProgramError::InvalidArgument)?;

    if atomic_swap.status != SwapStatus::Deposited {
        return Err(ProgramError::InvalidAccountData);
    }

    let mut hasher = Sha256::new();
    hasher.update(&args.secret);
    let computed_hash = hasher.finalize().to_vec();

    if computed_hash != atomic_swap.secret_hash {
        return Err(ProgramError::InvalidArgument);
    }

    atomic_swap.secret = Some(args.secret.clone());
    atomic_swap.status = SwapStatus::Redeemed;

    swap_data.completed_swaps.push(args.swap_id.clone());
    swap_data.active_swaps.remove(&args.swap_id);
    swap_data.swap_stats.completed_swaps += 1;

    let clock = Clock::get()?;
    let swap_duration = clock.unix_timestamp as u64 - atomic_swap.created_at;
    update_swap_stats(&mut swap_data.swap_stats, swap_duration);

    swap_data.serialize(&mut &mut swap_account.data.borrow_mut()[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;

    msg!("Redeemed atomic swap: {}", args.swap_id);
    Ok(())
}

pub fn refund(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: RefundArgs,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let swap_account = next_account_info(accounts_iter)?;
    let refunder_account = next_account_info(accounts_iter)?;

    if !refunder_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut swap_data = SwapState::try_from_slice(&swap_account.data.borrow())?;

    let atomic_swap = swap_data.active_swaps.get_mut(&args.swap_id)
        .ok_or(ProgramError::InvalidArgument)?;

    let clock = Clock::get()?;
    let current_time = clock.unix_timestamp as u64;

    if current_time < atomic_swap.expires_at {
        return Err(ProgramError::InvalidArgument);
    }

    if args.is_initiator {
        if atomic_swap.initiator != *refunder_account.key {
            return Err(ProgramError::InvalidAccountData);
        }
        atomic_swap.refund_initiator = true;
    } else {
        if atomic_swap.participant != Some(*refunder_account.key) {
            return Err(ProgramError::InvalidAccountData);
        }
        atomic_swap.refund_participant = true;
    }

    if atomic_swap.refund_initiator && atomic_swap.refund_participant {
        atomic_swap.status = SwapStatus::Refunded;
        swap_data.completed_swaps.push(args.swap_id.clone());
        swap_data.active_swaps.remove(&args.swap_id);
        swap_data.swap_stats.refunded_swaps += 1;
    }

    swap_data.serialize(&mut &mut swap_account.data.borrow_mut()[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;

    msg!("Refunded atomic swap: {}", args.swap_id);
    Ok(())
}

pub fn cancel_swap(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    swap_id: String,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let swap_account = next_account_info(accounts_iter)?;
    let authority_account = next_account_info(accounts_iter)?;

    if !authority_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut swap_data = SwapState::try_from_slice(&swap_account.data.borrow())?;
    
    if swap_data.authority != *authority_account.key {
        return Err(ProgramError::InvalidAccountData);
    }

    if let Some(atomic_swap) = swap_data.active_swaps.get_mut(&swap_id) {
        atomic_swap.status = SwapStatus::Cancelled;
        swap_data.completed_swaps.push(swap_id.clone());
        swap_data.active_swaps.remove(&swap_id);
        
        msg!("Cancelled atomic swap: {}", swap_id);
    } else {
        return Err(ProgramError::InvalidArgument);
    }

    swap_data.serialize(&mut &mut swap_account.data.borrow_mut()[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;

    Ok(())
}

pub fn update_fee_rate(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    new_rate: u64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let swap_account = next_account_info(accounts_iter)?;
    let authority_account = next_account_info(accounts_iter)?;

    if !authority_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut swap_data = SwapState::try_from_slice(&swap_account.data.borrow())?;
    
    if swap_data.authority != *authority_account.key {
        return Err(ProgramError::InvalidAccountData);
    }

    swap_data.fee_rate = new_rate;

    swap_data.serialize(&mut &mut swap_account.data.borrow_mut()[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;

    msg!("Updated atomic swap fee rate to: {}", new_rate);
    Ok(())
}

fn update_swap_stats(swap_stats: &mut SwapStats, duration: u64) {
    let total_completed = swap_stats.completed_swaps;
    swap_stats.average_swap_time = 
        (swap_stats.average_swap_time * total_completed + duration) / (total_completed + 1);
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum AtomicSwapInstruction {
    InitializeSwapState(InitSwapStateArgs),
    InitiateSwap(InitSwapArgs),
    ParticipateSwap(ParticipateSwapArgs),
    Deposit(DepositArgs),
    Redeem(RedeemArgs),
    Refund(RefundArgs),
    CancelSwap(String),
    UpdateFeeRate(u64),
}

impl AtomicSwapInstruction {
    pub fn try_from_slice(data: &[u8]) -> Result<Self, ProgramError> {
        let discriminator = u8::from_le_bytes(
            data.get(..1)
                .ok_or(ProgramError::InvalidInstructionData)?
                .try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)?,
        );

        match discriminator {
            0 => {
                let args = InitSwapStateArgs::try_from_slice(&data[1..])
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                Ok(AtomicSwapInstruction::InitializeSwapState(args))
            }
            1 => {
                let args = InitSwapArgs::try_from_slice(&data[1..])
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                Ok(AtomicSwapInstruction::InitiateSwap(args))
            }
            2 => {
                let args = ParticipateSwapArgs::try_from_slice(&data[1..])
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                Ok(AtomicSwapInstruction::ParticipateSwap(args))
            }
            3 => {
                let args = DepositArgs::try_from_slice(&data[1..])
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                Ok(AtomicSwapInstruction::Deposit(args))
            }
            4 => {
                let args = RedeemArgs::try_from_slice(&data[1..])
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                Ok(AtomicSwapInstruction::Redeem(args))
            }
            5 => {
                let args = RefundArgs::try_from_slice(&data[1..])
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                Ok(AtomicSwapInstruction::Refund(args))
            }
            6 => {
                let swap_id = String::try_from_slice(&data[1..])
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                Ok(AtomicSwapInstruction::CancelSwap(swap_id))
            }
            7 => {
                let new_rate = u64::try_from_slice(&data[1..])
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                Ok(AtomicSwapInstruction::UpdateFeeRate(new_rate))
            }
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
