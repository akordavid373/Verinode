use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::{clock::Clock, Sysvar},
};
use std::collections::HashMap;
use sha2::{Sha256, Digest};
use hmac::{Hmac, Mac};
use ripemd::Ripemd160;

type HmacSha256 = Hmac<Sha256>;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct ChainProof {
    pub chain_id: u64,
    pub block_number: u64,
    pub transaction_hash: Vec<u8>,
    pub proof_data: Vec<u8>,
    pub merkle_root: Vec<u8>,
    pub merkle_proof: Vec<Vec<u8>>,
    pub timestamp: u64,
    pub verifier_signature: Vec<u8>,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct VerificationState {
    pub is_initialized: bool,
    pub authority: Pubkey,
    pub supported_chains: HashMap<u64, ChainVerificationConfig>,
    pub verified_proofs: HashMap<String, VerifiedProof>,
    pub pending_verifications: HashMap<String, PendingVerification>,
    pub verification_stats: VerificationStats,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct ChainVerificationConfig {
    pub chain_id: u64,
    pub name: String,
    pub rpc_url: String,
    pub block_time: u64,
    pub confirmation_blocks: u64,
    pub trust_level: u8,
    pub verifier_addresses: Vec<Vec<u8>>,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct VerifiedProof {
    pub proof_id: String,
    pub chain_proof: ChainProof,
    pub verification_result: VerificationResult,
    pub verified_at: u64,
    pub verifier: Pubkey,
    pub cross_chain_reference: Option<String>,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct PendingVerification {
    pub proof_id: String,
    pub chain_proof: ChainProof,
    pub submitted_at: u64,
    pub status: VerificationStatus,
    pub attempts: u32,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum VerificationResult {
    Valid,
    Invalid,
    Pending,
    Expired,
    InsufficientConfirmations,
    MalformedProof,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum VerificationStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct VerificationStats {
    pub total_verified: u64,
    pub total_failed: u64,
    pub success_rate: f64,
    pub average_verification_time: u64,
    pub chain_stats: HashMap<u64, ChainStats>,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct ChainStats {
    pub chain_id: u64,
    pub verified_count: u64,
    pub failed_count: u64,
    pub average_time: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct InitVerifierArgs {
    pub authority: Pubkey,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct AddChainConfigArgs {
    pub chain_id: u64,
    pub name: String,
    pub rpc_url: String,
    pub block_time: u64,
    pub confirmation_blocks: u64,
    pub trust_level: u8,
    pub verifier_addresses: Vec<Vec<u8>>,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct SubmitProofArgs {
    pub proof_id: String,
    pub chain_proof: ChainProof,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct VerifyCrossChainProofArgs {
    pub proof_id: String,
    pub source_chain_id: u64,
    pub target_chain_id: u64,
    pub original_proof_id: String,
}

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = ChainVerifierInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        ChainVerifierInstruction::InitializeVerifier(args) => {
            initialize_verifier(program_id, accounts, args)
        }
        ChainVerifierInstruction::AddChainConfig(args) => {
            add_chain_config(program_id, accounts, args)
        }
        ChainVerifierInstruction::SubmitProof(args) => {
            submit_proof(program_id, accounts, args)
        }
        ChainVerifierInstruction::VerifyProof(proof_id) => {
            verify_proof(program_id, accounts, proof_id)
        }
        ChainVerifierInstruction::VerifyCrossChainProof(args) => {
            verify_cross_chain_proof(program_id, accounts, args)
        }
        ChainVerifierInstruction::UpdateVerificationStatus(proof_id, result) => {
            update_verification_status(program_id, accounts, proof_id, result)
        }
    }
}

pub fn initialize_verifier(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: InitVerifierArgs,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let verifier_account = next_account_info(accounts_iter)?;
    let authority_account = next_account_info(accounts_iter)?;

    if !authority_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut verifier_data = VerificationState::try_from_slice(&verifier_account.data.borrow())
        .unwrap_or_else(|_| VerificationState {
            is_initialized: false,
            authority: Pubkey::default(),
            supported_chains: HashMap::new(),
            verified_proofs: HashMap::new(),
            pending_verifications: HashMap::new(),
            verification_stats: VerificationStats {
                total_verified: 0,
                total_failed: 0,
                success_rate: 0.0,
                average_verification_time: 0,
                chain_stats: HashMap::new(),
            },
        });

    if verifier_data.is_initialized {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    verifier_data.is_initialized = true;
    verifier_data.authority = args.authority;

    verifier_data.serialize(&mut &mut verifier_account.data.borrow_mut()[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;

    msg!("Chain verifier initialized with authority: {:?}", args.authority);
    Ok(())
}

pub fn add_chain_config(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: AddChainConfigArgs,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let verifier_account = next_account_info(accounts_iter)?;
    let authority_account = next_account_info(accounts_iter)?;

    if !authority_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut verifier_data = VerificationState::try_from_slice(&verifier_account.data.borrow())?;
    
    if verifier_data.authority != *authority_account.key {
        return Err(ProgramError::InvalidAccountData);
    }

    let chain_config = ChainVerificationConfig {
        chain_id: args.chain_id,
        name: args.name,
        rpc_url: args.rpc_url,
        block_time: args.block_time,
        confirmation_blocks: args.confirmation_blocks,
        trust_level: args.trust_level,
        verifier_addresses: args.verifier_addresses,
    };

    verifier_data.supported_chains.insert(args.chain_id, chain_config);

    verifier_data.verification_stats.chain_stats.insert(args.chain_id, ChainStats {
        chain_id: args.chain_id,
        verified_count: 0,
        failed_count: 0,
        average_time: 0,
    });

    verifier_data.serialize(&mut &mut verifier_account.data.borrow_mut()[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;

    msg!("Added verification config for chain {}: {}", args.chain_id, args.name);
    Ok(())
}

pub fn submit_proof(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: SubmitProofArgs,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let verifier_account = next_account_info(accounts_iter)?;
    let submitter_account = next_account_info(accounts_iter)?;

    if !submitter_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut verifier_data = VerificationState::try_from_slice(&verifier_account.data.borrow())?;

    if !verifier_data.supported_chains.contains_key(&args.chain_proof.chain_id) {
        return Err(ProgramError::InvalidArgument);
    }

    let clock = Clock::get()?;
    let pending_verification = PendingVerification {
        proof_id: args.proof_id.clone(),
        chain_proof: args.chain_proof.clone(),
        submitted_at: clock.unix_timestamp as u64,
        status: VerificationStatus::Pending,
        attempts: 0,
    };

    verifier_data.pending_verifications.insert(args.proof_id.clone(), pending_verification);

    verifier_data.serialize(&mut &mut verifier_account.data.borrow_mut()[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;

    msg!("Submitted proof for verification: {}", args.proof_id);
    Ok(())
}

pub fn verify_proof(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    proof_id: String,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let verifier_account = next_account_info(accounts_iter)?;
    let verifier_authority = next_account_info(accounts_iter)?;

    if !verifier_authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut verifier_data = VerificationState::try_from_slice(&verifier_account.data.borrow())?;
    
    if verifier_data.authority != *verifier_authority.key {
        return Err(ProgramError::InvalidAccountData);
    }

    let pending_verification = verifier_data.pending_verifications.get(&proof_id)
        .ok_or(ProgramError::InvalidArgument)?;

    let verification_result = perform_verification(&pending_verification.chain_proof, &verifier_data.supported_chains);

    let clock = Clock::get()?;
    let verified_proof = VerifiedProof {
        proof_id: proof_id.clone(),
        chain_proof: pending_verification.chain_proof.clone(),
        verification_result: verification_result.clone(),
        verified_at: clock.unix_timestamp as u64,
        verifier: *verifier_authority.key,
        cross_chain_reference: None,
    };

    verifier_data.verified_proofs.insert(proof_id.clone(), verified_proof);
    verifier_data.pending_verifications.remove(&proof_id);

    update_verification_stats(&mut verifier_data, pending_verification.chain_proof.chain_id, &verification_result);

    verifier_data.serialize(&mut &mut verifier_account.data.borrow_mut()[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;

    msg!("Verified proof: {} with result: {:?}", proof_id, verification_result);
    Ok(())
}

pub fn verify_cross_chain_proof(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: VerifyCrossChainProofArgs,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let verifier_account = next_account_info(accounts_iter)?;
    let verifier_authority = next_account_info(accounts_iter)?;

    if !verifier_authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut verifier_data = VerificationState::try_from_slice(&verifier_account.data.borrow())?;
    
    if verifier_data.authority != *verifier_authority.key {
        return Err(ProgramError::InvalidAccountData);
    }

    let original_proof = verifier_data.verified_proofs.get(&args.original_proof_id)
        .ok_or(ProgramError::InvalidArgument)?;

    if !verifier_data.supported_chains.contains_key(&args.source_chain_id) ||
       !verifier_data.supported_chains.contains_key(&args.target_chain_id) {
        return Err(ProgramError::InvalidArgument);
    }

    let cross_chain_result = verify_cross_chain_validity(
        &original_proof.chain_proof,
        args.source_chain_id,
        args.target_chain_id,
        &verifier_data.supported_chains,
    );

    let clock = Clock::get()?;
    let mut verified_proof = original_proof.clone();
    verified_proof.proof_id = args.proof_id.clone();
    verified_proof.verified_at = clock.unix_timestamp as u64;
    verified_proof.verifier = *verifier_authority.key;
    verified_proof.cross_chain_reference = Some(args.original_proof_id.clone());
    verified_proof.verification_result = cross_chain_result.clone();

    verifier_data.verified_proofs.insert(args.proof_id.clone(), verified_proof);

    update_verification_stats(&mut verifier_data, args.target_chain_id, &cross_chain_result);

    verifier_data.serialize(&mut &mut verifier_account.data.borrow_mut()[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;

    msg!("Cross-chain verified proof: {} from chain {} to {} with result: {:?}", 
          args.proof_id, args.source_chain_id, args.target_chain_id, cross_chain_result);
    Ok(())
}

pub fn update_verification_status(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    proof_id: String,
    result: VerificationResult,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let verifier_account = next_account_info(accounts_iter)?;
    let authority_account = next_account_info(accounts_iter)?;

    if !authority_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut verifier_data = VerificationState::try_from_slice(&verifier_account.data.borrow())?;
    
    if verifier_data.authority != *authority_account.key {
        return Err(ProgramError::InvalidAccountData);
    }

    if let Some(verified_proof) = verifier_data.verified_proofs.get_mut(&proof_id) {
        verified_proof.verification_result = result.clone();
        verified_proof.verified_at = Clock::get()?.unix_timestamp as u64;
        
        msg!("Updated verification status for proof: {} to {:?}", proof_id, result);
    } else {
        return Err(ProgramError::InvalidArgument);
    }

    verifier_data.serialize(&mut &mut verifier_account.data.borrow_mut()[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;

    Ok(())
}

fn perform_verification(
    chain_proof: &ChainProof,
    supported_chains: &HashMap<u64, ChainVerificationConfig>,
) -> VerificationResult {
    let chain_config = supported_chains.get(&chain_proof.chain_id);
    
    if chain_config.is_none() {
        return VerificationResult::Invalid;
    }

    if !verify_merkle_proof(&chain_proof.merkle_root, &chain_proof.merkle_proof) {
        return VerificationResult::MalformedProof;
    }

    if !verify_transaction_signature(&chain_proof.transaction_hash, &chain_proof.verifier_signature) {
        return VerificationResult::Invalid;
    }

    let clock = Clock::get().unwrap();
    let current_time = clock.unix_timestamp as u64;
    
    if current_time > chain_proof.timestamp + (chain_config.unwrap().block_time * 100) {
        return VerificationResult::Expired;
    }

    VerificationResult::Valid
}

fn verify_cross_chain_validity(
    original_proof: &ChainProof,
    source_chain_id: u64,
    target_chain_id: u64,
    supported_chains: &HashMap<u64, ChainVerificationConfig>,
) -> VerificationResult {
    let source_config = supported_chains.get(&source_chain_id);
    let target_config = supported_chains.get(&target_chain_id);

    if source_config.is_none() || target_config.is_none() {
        return VerificationResult::Invalid;
    }

    let source_trust = source_config.unwrap().trust_level;
    let target_trust = target_config.unwrap().trust_level;

    if source_trust < 5 || target_trust < 5 {
        return VerificationResult::InsufficientConfirmations;
    }

    VerificationResult::Valid
}

fn verify_merkle_proof(root: &[u8], proof: &[Vec<u8>]) -> bool {
    if proof.is_empty() {
        return false;
    }

    let mut computed_hash = root.to_vec();
    
    for proof_element in proof {
        let mut hasher = Sha256::new();
        hasher.update(&computed_hash);
        hasher.update(proof_element);
        computed_hash = hasher.finalize().to_vec();
    }

    computed_hash == root.to_vec()
}

fn verify_transaction_signature(transaction_hash: &[u8], signature: &[u8]) -> bool {
    if signature.len() != 65 {
        return false;
    }

    let mut hasher = Sha256::new();
    hasher.update(transaction_hash);
    let hash = hasher.finalize();

    signature.len() == 65 && !signature.iter().all(|&x| x == 0)
}

fn update_verification_stats(
    verifier_data: &mut VerificationState,
    chain_id: u64,
    result: &VerificationResult,
) {
    match result {
        VerificationResult::Valid => {
            verifier_data.verification_stats.total_verified += 1;
        }
        _ => {
            verifier_data.verification_stats.total_failed += 1;
        }
    }

    let total = verifier_data.verification_stats.total_verified + verifier_data.verification_stats.total_failed;
    verifier_data.verification_stats.success_rate = 
        verifier_data.verification_stats.total_verified as f64 / total as f64;

    if let Some(chain_stats) = verifier_data.verification_stats.chain_stats.get_mut(&chain_id) {
        match result {
            VerificationResult::Valid => chain_stats.verified_count += 1,
            _ => chain_stats.failed_count += 1,
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum ChainVerifierInstruction {
    InitializeVerifier(InitVerifierArgs),
    AddChainConfig(AddChainConfigArgs),
    SubmitProof(SubmitProofArgs),
    VerifyProof(String),
    VerifyCrossChainProof(VerifyCrossChainProofArgs),
    UpdateVerificationStatus(String, VerificationResult),
}

impl ChainVerifierInstruction {
    pub fn try_from_slice(data: &[u8]) -> Result<Self, ProgramError> {
        let discriminator = u8::from_le_bytes(
            data.get(..1)
                .ok_or(ProgramError::InvalidInstructionData)?
                .try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)?,
        );

        match discriminator {
            0 => {
                let args = InitVerifierArgs::try_from_slice(&data[1..])
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                Ok(ChainVerifierInstruction::InitializeVerifier(args))
            }
            1 => {
                let args = AddChainConfigArgs::try_from_slice(&data[1..])
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                Ok(ChainVerifierInstruction::AddChainConfig(args))
            }
            2 => {
                let args = SubmitProofArgs::try_from_slice(&data[1..])
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                Ok(ChainVerifierInstruction::SubmitProof(args))
            }
            3 => {
                let proof_id = String::try_from_slice(&data[1..])
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                Ok(ChainVerifierInstruction::VerifyProof(proof_id))
            }
            4 => {
                let args = VerifyCrossChainProofArgs::try_from_slice(&data[1..])
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                Ok(ChainVerifierInstruction::VerifyCrossChainProof(args))
            }
            5 => {
                let proof_id = String::try_from_slice(&data[1..])
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                let result = VerificationResult::try_from_slice(&data[1 + proof_id.len()..])
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                Ok(ChainVerifierInstruction::UpdateVerificationStatus(proof_id, result))
            }
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
