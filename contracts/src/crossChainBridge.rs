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

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct ChainConfig {
    pub chain_id: u64,
    pub name: String,
    pub rpc_url: String,
    pub bridge_address: Vec<u8>,
    pub gas_price: u64,
    pub block_time: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct BridgeState {
    pub is_initialized: bool,
    pub authority: Pubkey,
    pub supported_chains: HashMap<u64, ChainConfig>,
    pub pending_transfers: HashMap<String, PendingTransfer>,
    pub completed_transfers: Vec<String>,
    pub total_volume: u64,
    pub fee_rate: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct PendingTransfer {
    pub transfer_id: String,
    pub from_chain: u64,
    pub to_chain: u64,
    pub sender: Vec<u8>,
    pub recipient: Vec<u8>,
    pub amount: u64,
    pub token_address: Vec<u8>,
    pub timestamp: u64,
    pub status: TransferStatus,
    pub proof_hash: Vec<u8>,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum TransferStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Refunded,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct InitBridgeArgs {
    pub authority: Pubkey,
    pub fee_rate: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct AddChainArgs {
    pub chain_id: u64,
    pub name: String,
    pub rpc_url: String,
    pub bridge_address: Vec<u8>,
    pub gas_price: u64,
    pub block_time: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct InitiateTransferArgs {
    pub transfer_id: String,
    pub from_chain: u64,
    pub to_chain: u64,
    pub recipient: Vec<u8>,
    pub amount: u64,
    pub token_address: Vec<u8>,
}

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = CrossChainBridgeInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        CrossChainBridgeInstruction::InitializeBridge(args) => {
            initialize_bridge(program_id, accounts, args)
        }
        CrossChainBridgeInstruction::AddSupportedChain(args) => {
            add_supported_chain(program_id, accounts, args)
        }
        CrossChainBridgeInstruction::InitiateTransfer(args) => {
            initiate_transfer(program_id, accounts, args)
        }
        CrossChainBridgeInstruction::CompleteTransfer(transfer_id) => {
            complete_transfer(program_id, accounts, transfer_id)
        }
        CrossChainBridgeInstruction::RefundTransfer(transfer_id) => {
            refund_transfer(program_id, accounts, transfer_id)
        }
        CrossChainBridgeInstruction::UpdateFeeRate(new_rate) => {
            update_fee_rate(program_id, accounts, new_rate)
        }
    }
}

pub fn initialize_bridge(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: InitBridgeArgs,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let bridge_account = next_account_info(accounts_iter)?;
    let authority_account = next_account_info(accounts_iter)?;

    if !authority_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut bridge_data = BridgeState::try_from_slice(&bridge_account.data.borrow())
        .unwrap_or_else(|_| BridgeState {
            is_initialized: false,
            authority: Pubkey::default(),
            supported_chains: HashMap::new(),
            pending_transfers: HashMap::new(),
            completed_transfers: Vec::new(),
            total_volume: 0,
            fee_rate: 0,
        });

    if bridge_data.is_initialized {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    bridge_data.is_initialized = true;
    bridge_data.authority = args.authority;
    bridge_data.fee_rate = args.fee_rate;

    bridge_data.serialize(&mut &mut bridge_account.data.borrow_mut()[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;

    msg!("Bridge initialized with authority: {:?}", args.authority);
    Ok(())
}

pub fn add_supported_chain(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: AddChainArgs,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let bridge_account = next_account_info(accounts_iter)?;
    let authority_account = next_account_info(accounts_iter)?;

    if !authority_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut bridge_data = BridgeState::try_from_slice(&bridge_account.data.borrow())?;
    
    if bridge_data.authority != *authority_account.key {
        return Err(ProgramError::InvalidAccountData);
    }

    let chain_config = ChainConfig {
        chain_id: args.chain_id,
        name: args.name,
        rpc_url: args.rpc_url,
        bridge_address: args.bridge_address,
        gas_price: args.gas_price,
        block_time: args.block_time,
    };

    bridge_data.supported_chains.insert(args.chain_id, chain_config);

    bridge_data.serialize(&mut &mut bridge_account.data.borrow_mut()[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;

    msg!("Added support for chain {}: {}", args.chain_id, args.name);
    Ok(())
}

pub fn initiate_transfer(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: InitiateTransferArgs,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let bridge_account = next_account_info(accounts_iter)?;
    let sender_account = next_account_info(accounts_iter)?;

    if !sender_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut bridge_data = BridgeState::try_from_slice(&bridge_account.data.borrow())?;

    if !bridge_data.supported_chains.contains_key(&args.from_chain) ||
       !bridge_data.supported_chains.contains_key(&args.to_chain) {
        return Err(ProgramError::InvalidArgument);
    }

    let clock = Clock::get()?;
    let proof_hash = generate_transfer_proof(&args, clock.unix_timestamp);

    let transfer = PendingTransfer {
        transfer_id: args.transfer_id.clone(),
        from_chain: args.from_chain,
        to_chain: args.to_chain,
        sender: sender_account.key.to_bytes().to_vec(),
        recipient: args.recipient,
        amount: args.amount,
        token_address: args.token_address,
        timestamp: clock.unix_timestamp as u64,
        status: TransferStatus::Pending,
        proof_hash,
    };

    bridge_data.pending_transfers.insert(args.transfer_id.clone(), transfer);
    bridge_data.total_volume += args.amount;

    bridge_data.serialize(&mut &mut bridge_account.data.borrow_mut()[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;

    msg!("Initiated transfer {} from chain {} to {}", 
          args.transfer_id, args.from_chain, args.to_chain);
    Ok(())
}

pub fn complete_transfer(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    transfer_id: String,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let bridge_account = next_account_info(accounts_iter)?;
    let authority_account = next_account_info(accounts_iter)?;

    if !authority_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut bridge_data = BridgeState::try_from_slice(&bridge_account.data.borrow())?;
    
    if bridge_data.authority != *authority_account.key {
        return Err(ProgramError::InvalidAccountData);
    }

    if let Some(mut transfer) = bridge_data.pending_transfers.remove(&transfer_id) {
        transfer.status = TransferStatus::Completed;
        bridge_data.completed_transfers.push(transfer_id.clone());
        
        msg!("Completed transfer: {}", transfer_id);
    } else {
        return Err(ProgramError::InvalidArgument);
    }

    bridge_data.serialize(&mut &mut bridge_account.data.borrow_mut()[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;

    Ok(())
}

pub fn refund_transfer(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    transfer_id: String,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let bridge_account = next_account_info(accounts_iter)?;
    let authority_account = next_account_info(accounts_iter)?;

    if !authority_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut bridge_data = BridgeState::try_from_slice(&bridge_account.data.borrow())?;
    
    if bridge_data.authority != *authority_account.key {
        return Err(ProgramError::InvalidAccountData);
    }

    if let Some(mut transfer) = bridge_data.pending_transfers.remove(&transfer_id) {
        transfer.status = TransferStatus::Refunded;
        bridge_data.completed_transfers.push(transfer_id.clone());
        
        msg!("Refunded transfer: {}", transfer_id);
    } else {
        return Err(ProgramError::InvalidArgument);
    }

    bridge_data.serialize(&mut &mut bridge_account.data.borrow_mut()[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;

    Ok(())
}

pub fn update_fee_rate(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    new_rate: u64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let bridge_account = next_account_info(accounts_iter)?;
    let authority_account = next_account_info(accounts_iter)?;

    if !authority_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut bridge_data = BridgeState::try_from_slice(&bridge_account.data.borrow())?;
    
    if bridge_data.authority != *authority_account.key {
        return Err(ProgramError::InvalidAccountData);
    }

    bridge_data.fee_rate = new_rate;

    bridge_data.serialize(&mut &mut bridge_account.data.borrow_mut()[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;

    msg!("Updated fee rate to: {}", new_rate);
    Ok(())
}

fn generate_transfer_proof(args: &InitiateTransferArgs, timestamp: i64) -> Vec<u8> {
    use sha2::{Sha256, Digest};
    
    let mut hasher = Sha256::new();
    hasher.update(args.transfer_id.as_bytes());
    hasher.update(args.from_chain.to_be_bytes());
    hasher.update(args.to_chain.to_be_bytes());
    hasher.update(&args.recipient);
    hasher.update(args.amount.to_be_bytes());
    hasher.update(&args.token_address);
    hasher.update(timestamp.to_be_bytes());
    
    hasher.finalize().to_vec()
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum CrossChainBridgeInstruction {
    InitializeBridge(InitBridgeArgs),
    AddSupportedChain(AddChainArgs),
    InitiateTransfer(InitiateTransferArgs),
    CompleteTransfer(String),
    RefundTransfer(String),
    UpdateFeeRate(u64),
}

impl CrossChainBridgeInstruction {
    pub fn try_from_slice(data: &[u8]) -> Result<Self, ProgramError> {
        let discriminator = u8::from_le_bytes(
            data.get(..1)
                .ok_or(ProgramError::InvalidInstructionData)?
                .try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)?,
        );

        match discriminator {
            0 => {
                let args = InitBridgeArgs::try_from_slice(&data[1..])
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                Ok(CrossChainBridgeInstruction::InitializeBridge(args))
            }
            1 => {
                let args = AddChainArgs::try_from_slice(&data[1..])
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                Ok(CrossChainBridgeInstruction::AddSupportedChain(args))
            }
            2 => {
                let args = InitiateTransferArgs::try_from_slice(&data[1..])
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                Ok(CrossChainBridgeInstruction::InitiateTransfer(args))
            }
            3 => {
                let transfer_id = String::try_from_slice(&data[1..])
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                Ok(CrossChainBridgeInstruction::CompleteTransfer(transfer_id))
            }
            4 => {
                let transfer_id = String::try_from_slice(&data[1..])
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                Ok(CrossChainBridgeInstruction::RefundTransfer(transfer_id))
            }
            5 => {
                let new_rate = u64::try_from_slice(&data[1..])
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                Ok(CrossChainBridgeInstruction::UpdateFeeRate(new_rate))
            }
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
