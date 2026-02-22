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
pub struct CrossChainMessage {
    pub message_id: String,
    pub source_chain: u64,
    pub target_chain: u64,
    pub sender: Vec<u8>,
    pub recipient: Vec<u8>,
    pub message_type: MessageType,
    pub payload: Vec<u8>,
    pub nonce: u64,
    pub timestamp: u64,
    pub gas_limit: u64,
    pub gas_price: u64,
    pub status: MessageStatus,
    pub signature: Vec<u8>,
    pub relay_proof: Option<RelayProof>,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum MessageType {
    Transfer,
    Swap,
    ContractCall,
    Data,
    Proof,
    Confirmation,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum MessageStatus {
    Pending,
    InTransit,
    Delivered,
    Executed,
    Failed,
    Expired,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct RelayProof {
    pub relayer: Pubkey,
    pub relay_transaction: Vec<u8>,
    pub relay_block: u64,
    pub relay_signature: Vec<u8>,
    pub relay_timestamp: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct MessagePassingState {
    pub is_initialized: bool,
    pub authority: Pubkey,
    pub supported_chains: HashMap<u64, ChainMessageConfig>,
    pub pending_messages: HashMap<String, CrossChainMessage>,
    pub delivered_messages: HashMap<String, DeliveredMessage>,
    pub message_stats: MessageStats,
    pub fee_rate: u64,
    pub relayers: HashMap<Pubkey, RelayerInfo>,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct ChainMessageConfig {
    pub chain_id: u64,
    pub name: String,
    pub message_router: Vec<u8>,
    pub gas_price_oracle: Vec<u8>,
    pub block_time: u64,
    pub confirmation_blocks: u64,
    pub max_message_size: u64,
    pub supported_message_types: Vec<MessageType>,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct DeliveredMessage {
    pub message_id: String,
    pub original_message: CrossChainMessage,
    pub delivered_at: u64,
    pub execution_result: ExecutionResult,
    pub gas_used: u64,
    pub relayer: Pubkey,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct ExecutionResult {
    pub success: bool,
    pub return_data: Vec<u8>,
    pub error_message: Option<String>,
    pub gas_used: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct RelayerInfo {
    pub address: Pubkey,
    pub stake_amount: u64,
    pub reputation: u32,
    pub total_messages: u64,
    pub success_rate: f64,
    pub is_active: bool,
    pub supported_chains: Vec<u64>,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct MessageStats {
    pub total_messages: u64,
    pub delivered_messages: u64,
    pub failed_messages: u64,
    pub average_delivery_time: u64,
    pub total_gas_used: u64,
    pub chain_stats: HashMap<u64, ChainMessageStats>,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct ChainMessageStats {
    pub chain_id: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub average_gas_used: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct InitMessagePassingArgs {
    pub authority: Pubkey,
    pub fee_rate: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct AddChainMessageConfigArgs {
    pub chain_id: u64,
    pub name: String,
    pub message_router: Vec<u8>,
    pub gas_price_oracle: Vec<u8>,
    pub block_time: u64,
    pub confirmation_blocks: u64,
    pub max_message_size: u64,
    pub supported_message_types: Vec<MessageType>,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct SendMessageArgs {
    pub message_id: String,
    pub source_chain: u64,
    pub target_chain: u64,
    pub recipient: Vec<u8>,
    pub message_type: MessageType,
    pub payload: Vec<u8>,
    pub gas_limit: u64,
    pub gas_price: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct RelayMessageArgs {
    pub message_id: String,
    pub relay_transaction: Vec<u8>,
    pub relay_block: u64,
    pub relay_signature: Vec<u8>,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct ExecuteMessageArgs {
    pub message_id: String,
    pub execution_data: Vec<u8>,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct RegisterRelayerArgs {
    pub stake_amount: u64,
    pub supported_chains: Vec<u64>,
}

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = MessagePassingInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        MessagePassingInstruction::InitializeMessagePassing(args) => {
            initialize_message_passing(program_id, accounts, args)
        }
        MessagePassingInstruction::AddChainMessageConfig(args) => {
            add_chain_message_config(program_id, accounts, args)
        }
        MessagePassingInstruction::SendMessage(args) => {
            send_message(program_id, accounts, args)
        }
        MessagePassingInstruction::RelayMessage(args) => {
            relay_message(program_id, accounts, args)
        }
        MessagePassingInstruction::ExecuteMessage(args) => {
            execute_message(program_id, accounts, args)
        }
        MessagePassingInstruction::RegisterRelayer(args) => {
            register_relayer(program_id, accounts, args)
        }
        MessagePassingInstruction::UpdateRelayerStatus(relayer_address, is_active) => {
            update_relayer_status(program_id, accounts, relayer_address, is_active)
        }
        MessagePassingInstruction::UpdateFeeRate(new_rate) => {
            update_fee_rate(program_id, accounts, new_rate)
        }
    }
}

pub fn initialize_message_passing(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: InitMessagePassingArgs,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let message_account = next_account_info(accounts_iter)?;
    let authority_account = next_account_info(accounts_iter)?;

    if !authority_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut message_data = MessagePassingState::try_from_slice(&message_account.data.borrow())
        .unwrap_or_else(|_| MessagePassingState {
            is_initialized: false,
            authority: Pubkey::default(),
            supported_chains: HashMap::new(),
            pending_messages: HashMap::new(),
            delivered_messages: HashMap::new(),
            message_stats: MessageStats {
                total_messages: 0,
                delivered_messages: 0,
                failed_messages: 0,
                average_delivery_time: 0,
                total_gas_used: 0,
                chain_stats: HashMap::new(),
            },
            fee_rate: 0,
            relayers: HashMap::new(),
        });

    if message_data.is_initialized {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    message_data.is_initialized = true;
    message_data.authority = args.authority;
    message_data.fee_rate = args.fee_rate;

    message_data.serialize(&mut &mut message_account.data.borrow_mut()[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;

    msg!("Message passing system initialized with authority: {:?}", args.authority);
    Ok(())
}

pub fn add_chain_message_config(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: AddChainMessageConfigArgs,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let message_account = next_account_info(accounts_iter)?;
    let authority_account = next_account_info(accounts_iter)?;

    if !authority_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut message_data = MessagePassingState::try_from_slice(&message_account.data.borrow())?;
    
    if message_data.authority != *authority_account.key {
        return Err(ProgramError::InvalidAccountData);
    }

    let chain_config = ChainMessageConfig {
        chain_id: args.chain_id,
        name: args.name,
        message_router: args.message_router,
        gas_price_oracle: args.gas_price_oracle,
        block_time: args.block_time,
        confirmation_blocks: args.confirmation_blocks,
        max_message_size: args.max_message_size,
        supported_message_types: args.supported_message_types,
    };

    message_data.supported_chains.insert(args.chain_id, chain_config);

    message_data.message_stats.chain_stats.insert(args.chain_id, ChainMessageStats {
        chain_id: args.chain_id,
        messages_sent: 0,
        messages_received: 0,
        average_gas_used: 0,
    });

    message_data.serialize(&mut &mut message_account.data.borrow_mut()[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;

    msg!("Added message config for chain {}: {}", args.chain_id, args.name);
    Ok(())
}

pub fn send_message(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: SendMessageArgs,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let message_account = next_account_info(accounts_iter)?;
    let sender_account = next_account_info(accounts_iter)?;

    if !sender_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut message_data = MessagePassingState::try_from_slice(&message_account.data.borrow())?;

    if !message_data.supported_chains.contains_key(&args.source_chain) ||
       !message_data.supported_chains.contains_key(&args.target_chain) {
        return Err(ProgramError::InvalidArgument);
    }

    let source_config = message_data.supported_chains.get(&args.source_chain).unwrap();
    if !source_config.supported_message_types.contains(&args.message_type) {
        return Err(ProgramError::InvalidArgument);
    }

    let clock = Clock::get()?;
    let nonce = generate_nonce(&args.message_id, clock.unix_timestamp);

    let signature = sign_message(&args, sender_account.key, &nonce);

    let message = CrossChainMessage {
        message_id: args.message_id.clone(),
        source_chain: args.source_chain,
        target_chain: args.target_chain,
        sender: sender_account.key.to_bytes().to_vec(),
        recipient: args.recipient,
        message_type: args.message_type,
        payload: args.payload,
        nonce,
        timestamp: clock.unix_timestamp as u64,
        gas_limit: args.gas_limit,
        gas_price: args.gas_price,
        status: MessageStatus::Pending,
        signature,
        relay_proof: None,
    };

    message_data.pending_messages.insert(args.message_id.clone(), message);
    message_data.message_stats.total_messages += 1;

    if let Some(chain_stats) = message_data.message_stats.chain_stats.get_mut(&args.source_chain) {
        chain_stats.messages_sent += 1;
    }

    message_data.serialize(&mut &mut message_account.data.borrow_mut()[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;

    msg!("Sent cross-chain message: {} from chain {} to {}", 
          args.message_id, args.source_chain, args.target_chain);
    Ok(())
}

pub fn relay_message(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: RelayMessageArgs,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let message_account = next_account_info(accounts_iter)?;
    let relayer_account = next_account_info(accounts_iter)?;

    if !relayer_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut message_data = MessagePassingState::try_from_slice(&message_account.data.borrow())?;

    let relayer_info = message_data.relayers.get(relayer_account.key)
        .ok_or(ProgramError::InvalidAccountData)?;

    if !relayer_info.is_active {
        return Err(ProgramError::InvalidAccountData);
    }

    let message = message_data.pending_messages.get_mut(&args.message_id)
        .ok_or(ProgramError::InvalidArgument)?;

    let clock = Clock::get()?;
    let relay_proof = RelayProof {
        relayer: *relayer_account.key,
        relay_transaction: args.relay_transaction,
        relay_block: args.relay_block,
        relay_signature: args.relay_signature,
        relay_timestamp: clock.unix_timestamp as u64,
    };

    message.relay_proof = Some(relay_proof);
    message.status = MessageStatus::InTransit;

    message_data.serialize(&mut &mut message_account.data.borrow_mut()[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;

    msg!("Relayed message: {} by relayer: {:?}", args.message_id, relayer_account.key);
    Ok(())
}

pub fn execute_message(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: ExecuteMessageArgs,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let message_account = next_account_info(accounts_iter)?;
    let executor_account = next_account_info(accounts_iter)?;

    if !executor_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut message_data = MessagePassingState::try_from_slice(&message_account.data.borrow())?;

    let message = message_data.pending_messages.get(&args.message_id)
        .ok_or(ProgramError::InvalidArgument)?;

    if message.status != MessageStatus::InTransit {
        return Err(ProgramError::InvalidAccountData);
    }

    let execution_result = execute_cross_chain_message(message, &args.execution_data);

    let clock = Clock::get()?;
    let delivered_message = DeliveredMessage {
        message_id: args.message_id.clone(),
        original_message: message.clone(),
        delivered_at: clock.unix_timestamp as u64,
        execution_result: execution_result.clone(),
        gas_used: execution_result.gas_used,
        relayer: message.relay_proof.as_ref().unwrap().relayer,
    };

    message_data.delivered_messages.insert(args.message_id.clone(), delivered_message);
    message_data.pending_messages.remove(&args.message_id);

    if execution_result.success {
        message_data.message_stats.delivered_messages += 1;
    } else {
        message_data.message_stats.failed_messages += 1;
    }

    message_data.message_stats.total_gas_used += execution_result.gas_used;

    if let Some(chain_stats) = message_data.message_stats.chain_stats.get_mut(&message.target_chain) {
        chain_stats.messages_received += 1;
        chain_stats.average_gas_used = 
            (chain_stats.average_gas_used * chain_stats.messages_received + execution_result.gas_used) / 
            (chain_stats.messages_received + 1);
    }

    message_data.serialize(&mut &mut message_account.data.borrow_mut()[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;

    msg!("Executed message: {} with result: {}", args.message_id, execution_result.success);
    Ok(())
}

pub fn register_relayer(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: RegisterRelayerArgs,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let message_account = next_account_info(accounts_iter)?;
    let relayer_account = next_account_info(accounts_iter)?;

    if !relayer_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut message_data = MessagePassingState::try_from_slice(&message_account.data.borrow())?;

    if message_data.relayers.contains_key(relayer_account.key) {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    let relayer_info = RelayerInfo {
        address: *relayer_account.key,
        stake_amount: args.stake_amount,
        reputation: 100,
        total_messages: 0,
        success_rate: 1.0,
        is_active: true,
        supported_chains: args.supported_chains,
    };

    message_data.relayers.insert(*relayer_account.key, relayer_info);

    message_data.serialize(&mut &mut message_account.data.borrow_mut()[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;

    msg!("Registered relayer: {:?}", relayer_account.key);
    Ok(())
}

pub fn update_relayer_status(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    relayer_address: Pubkey,
    is_active: bool,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let message_account = next_account_info(accounts_iter)?;
    let authority_account = next_account_info(accounts_iter)?;

    if !authority_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut message_data = MessagePassingState::try_from_slice(&message_account.data.borrow())?;
    
    if message_data.authority != *authority_account.key {
        return Err(ProgramError::InvalidAccountData);
    }

    if let Some(relayer_info) = message_data.relayers.get_mut(&relayer_address) {
        relayer_info.is_active = is_active;
        msg!("Updated relayer status for {:?} to: {}", relayer_address, is_active);
    } else {
        return Err(ProgramError::InvalidArgument);
    }

    message_data.serialize(&mut &mut message_account.data.borrow_mut()[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;

    Ok(())
}

pub fn update_fee_rate(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    new_rate: u64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let message_account = next_account_info(accounts_iter)?;
    let authority_account = next_account_info(accounts_iter)?;

    if !authority_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut message_data = MessagePassingState::try_from_slice(&message_account.data.borrow())?;
    
    if message_data.authority != *authority_account.key {
        return Err(ProgramError::InvalidAccountData);
    }

    message_data.fee_rate = new_rate;

    message_data.serialize(&mut &mut message_account.data.borrow_mut()[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;

    msg!("Updated message passing fee rate to: {}", new_rate);
    Ok(())
}

fn generate_nonce(message_id: &str, timestamp: i64) -> u64 {
    let mut hasher = Sha256::new();
    hasher.update(message_id.as_bytes());
    hasher.update(timestamp.to_be_bytes());
    let hash = hasher.finalize();
    u64::from_be_bytes([
        hash[0], hash[1], hash[2], hash[3],
        hash[4], hash[5], hash[6], hash[7],
    ])
}

fn sign_message(args: &SendMessageArgs, sender: &Pubkey, nonce: &u64) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(args.message_id.as_bytes());
    hasher.update(args.source_chain.to_be_bytes());
    hasher.update(args.target_chain.to_be_bytes());
    hasher.update(&args.recipient);
    hasher.update(&(args.message_type as u8).to_be_bytes());
    hasher.update(&args.payload);
    hasher.update(nonce.to_be_bytes());
    hasher.update(sender.as_ref());
    
    let hash = hasher.finalize();
    hash.to_vec()
}

fn execute_cross_chain_message(message: &CrossChainMessage, execution_data: &[u8]) -> ExecutionResult {
    let mut gas_used = message.gas_limit / 2; // Simulate gas usage
    
    match message.message_type {
        MessageType::Transfer => {
            ExecutionResult {
                success: true,
                return_data: execution_data.to_vec(),
                error_message: None,
                gas_used,
            }
        }
        MessageType::ContractCall => {
            gas_used = message.gas_limit * 3 / 4;
            ExecutionResult {
                success: true,
                return_data: vec![1, 2, 3, 4],
                error_message: None,
                gas_used,
            }
        }
        MessageType::Data => {
            ExecutionResult {
                success: true,
                return_data: message.payload.clone(),
                error_message: None,
                gas_used: 21000,
            }
        }
        _ => {
            ExecutionResult {
                success: false,
                return_data: vec![],
                error_message: Some("Unsupported message type".to_string()),
                gas_used,
            }
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum MessagePassingInstruction {
    InitializeMessagePassing(InitMessagePassingArgs),
    AddChainMessageConfig(AddChainMessageConfigArgs),
    SendMessage(SendMessageArgs),
    RelayMessage(RelayMessageArgs),
    ExecuteMessage(ExecuteMessageArgs),
    RegisterRelayer(RegisterRelayerArgs),
    UpdateRelayerStatus(Pubkey, bool),
    UpdateFeeRate(u64),
}

impl MessagePassingInstruction {
    pub fn try_from_slice(data: &[u8]) -> Result<Self, ProgramError> {
        let discriminator = u8::from_le_bytes(
            data.get(..1)
                .ok_or(ProgramError::InvalidInstructionData)?
                .try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)?,
        );

        match discriminator {
            0 => {
                let args = InitMessagePassingArgs::try_from_slice(&data[1..])
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                Ok(MessagePassingInstruction::InitializeMessagePassing(args))
            }
            1 => {
                let args = AddChainMessageConfigArgs::try_from_slice(&data[1..])
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                Ok(MessagePassingInstruction::AddChainMessageConfig(args))
            }
            2 => {
                let args = SendMessageArgs::try_from_slice(&data[1..])
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                Ok(MessagePassingInstruction::SendMessage(args))
            }
            3 => {
                let args = RelayMessageArgs::try_from_slice(&data[1..])
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                Ok(MessagePassingInstruction::RelayMessage(args))
            }
            4 => {
                let args = ExecuteMessageArgs::try_from_slice(&data[1..])
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                Ok(MessagePassingInstruction::ExecuteMessage(args))
            }
            5 => {
                let args = RegisterRelayerArgs::try_from_slice(&data[1..])
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                Ok(MessagePassingInstruction::RegisterRelayer(args))
            }
            6 => {
                let relayer_address = Pubkey::try_from_slice(&data[1..])
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                let is_active = bool::try_from_slice(&data[1 + 32..])
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                Ok(MessagePassingInstruction::UpdateRelayerStatus(relayer_address, is_active))
            }
            7 => {
                let new_rate = u64::try_from_slice(&data[1..])
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                Ok(MessagePassingInstruction::UpdateFeeRate(new_rate))
            }
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
