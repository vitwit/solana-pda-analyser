use crate::{Result, TransactionAnalysis, PdaInteraction, InteractionType, PdaInfo};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::transaction::Transaction;
use solana_sdk::instruction::Instruction;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TransactionAnalyzer {
    known_programs: HashMap<Pubkey, String>,
}

impl TransactionAnalyzer {
    pub fn new() -> Self {
        let mut known_programs = HashMap::new();
        
        // Add known program IDs
        known_programs.insert(solana_sdk::system_program::id(), "System Program".to_string());
        // Note: In a real implementation, you would import and use actual program IDs
        // For now, using placeholder program IDs
        known_programs.insert(
            "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".parse().unwrap_or_default(),
            "SPL Token".to_string()
        );
        known_programs.insert(
            "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL".parse().unwrap_or_default(),
            "SPL Associated Token Account".to_string()
        );
        
        Self {
            known_programs,
        }
    }

    pub fn analyze_transaction(
        &self,
        signature: &str,
        transaction: &Transaction,
        slot: u64,
        block_time: Option<DateTime<Utc>>,
        success: bool,
        error_message: Option<String>,
        pre_account_states: &[AccountState],
        post_account_states: &[AccountState],
    ) -> Result<TransactionAnalysis> {
        let mut pda_interactions = Vec::new();
        let mut discovered_pdas = Vec::new();
        
        // Analyze each instruction
        for (instruction_index, instruction) in transaction.message.instructions.iter().enumerate() {
            let program_id = transaction.message.account_keys[instruction.program_id_index as usize];
            
            // Check if this instruction involves any PDAs
            for account_index in &instruction.accounts {
                let account_pubkey = transaction.message.account_keys[*account_index as usize];
                
                // Check if this account is a PDA
                if let Some(pda_info) = self.detect_pda(&account_pubkey, &program_id) {
                    discovered_pdas.push(pda_info);
                    
                    // Determine interaction type
                    let interaction_type = self.determine_interaction_type(
                        instruction_index,
                        *account_index as usize,
                        &transaction.message,
                        pre_account_states,
                        post_account_states,
                    );
                    
                    let (data_before, data_after, lamports_before, lamports_after) = 
                        self.get_account_changes(
                            &account_pubkey,
                            pre_account_states,
                            post_account_states,
                        );
                    
                    pda_interactions.push(PdaInteraction {
                        pda_address: account_pubkey,
                        instruction_index: instruction_index as u32,
                        interaction_type,
                        data_before,
                        data_after,
                        lamports_before,
                        lamports_after,
                    });
                }
            }
        }
        
        // Remove duplicate PDAs
        discovered_pdas.sort_by_key(|pda| pda.address);
        discovered_pdas.dedup_by_key(|pda| pda.address);
        
        Ok(TransactionAnalysis {
            signature: signature.to_string(),
            slot,
            block_time,
            success,
            error_message,
            pda_interactions,
            discovered_pdas,
        })
    }

    fn detect_pda(&self, address: &Pubkey, program_id: &Pubkey) -> Option<PdaInfo> {
        // Simple heuristic: check if the address is on-curve
        // Real PDAs are off-curve points
        if address.is_on_curve() {
            return None;
        }
        
        // For now, we'll assume any off-curve address associated with a program is a PDA
        // In a real implementation, we'd need to try to derive the seeds
        Some(PdaInfo {
            address: *address,
            program_id: *program_id,
            seeds: Vec::new(), // Would need to derive these
            bump: 0,           // Would need to derive this
            first_seen_slot: None,
            first_seen_transaction: None,
        })
    }

    fn determine_interaction_type(
        &self,
        _instruction_index: usize,
        _account_index: usize,
        message: &solana_sdk::message::Message,
        pre_states: &[AccountState],
        post_states: &[AccountState],
    ) -> InteractionType {
        // This is a simplified heuristic
        // In practice, you'd need to decode the instruction data and analyze the changes
        
        let account_pubkey = message.account_keys[_account_index];
        
        let pre_state = pre_states.iter().find(|s| s.pubkey == account_pubkey);
        let post_state = post_states.iter().find(|s| s.pubkey == account_pubkey);
        
        match (pre_state, post_state) {
            (None, Some(_)) => InteractionType::Create,
            (Some(_), None) => InteractionType::Close,
            (Some(pre), Some(post)) => {
                if pre.data != post.data {
                    InteractionType::Write
                } else {
                    InteractionType::Read
                }
            }
            (None, None) => InteractionType::Read, // Fallback
        }
    }

    fn get_account_changes(
        &self,
        account_pubkey: &Pubkey,
        pre_states: &[AccountState],
        post_states: &[AccountState],
    ) -> (Option<Vec<u8>>, Option<Vec<u8>>, Option<u64>, Option<u64>) {
        let pre_state = pre_states.iter().find(|s| s.pubkey == *account_pubkey);
        let post_state = post_states.iter().find(|s| s.pubkey == *account_pubkey);
        
        let data_before = pre_state.map(|s| s.data.clone());
        let data_after = post_state.map(|s| s.data.clone());
        let lamports_before = pre_state.map(|s| s.lamports);
        let lamports_after = post_state.map(|s| s.lamports);
        
        (data_before, data_after, lamports_before, lamports_after)
    }

    pub fn add_known_program(&mut self, program_id: Pubkey, name: String) {
        self.known_programs.insert(program_id, name);
    }
    
    pub fn get_program_name(&self, program_id: &Pubkey) -> Option<&String> {
        self.known_programs.get(program_id)
    }
}

impl Default for TransactionAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct AccountState {
    pub pubkey: Pubkey,
    pub lamports: u64,
    pub data: Vec<u8>,
    pub owner: Pubkey,
    pub executable: bool,
    pub rent_epoch: u64,
}

#[derive(Debug, Clone)]
pub struct InstructionAnalysis {
    pub program_id: Pubkey,
    pub instruction_data: Vec<u8>,
    pub accounts: Vec<Pubkey>,
    pub pda_accounts: Vec<Pubkey>,
}

impl InstructionAnalysis {
    pub fn from_instruction(instruction: &Instruction, _message: &solana_sdk::message::Message) -> Self {
        let program_id = instruction.program_id;
        let accounts: Vec<Pubkey> = instruction.accounts
            .iter()
            .map(|account_meta| account_meta.pubkey)
            .collect();
        
        // Filter for potential PDAs (off-curve addresses)
        let pda_accounts: Vec<Pubkey> = accounts
            .iter()
            .filter(|addr| !addr.is_on_curve())
            .cloned()
            .collect();
        
        Self {
            program_id,
            instruction_data: instruction.data.clone(),
            accounts,
            pda_accounts,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::signature::Keypair;
    use solana_sdk::signer::Signer;
    use solana_sdk::system_instruction;
    
    #[test]
    fn test_transaction_analyzer_creation() {
        let analyzer = TransactionAnalyzer::new();
        assert!(analyzer.known_programs.len() > 0);
    }
    
    #[test]
    fn test_account_state_handling() {
        let analyzer = TransactionAnalyzer::new();
        let keypair = Keypair::new();
        let account_state = AccountState {
            pubkey: keypair.pubkey(),
            lamports: 1000000,
            data: vec![1, 2, 3],
            owner: solana_sdk::system_program::id(),
            executable: false,
            rent_epoch: 0,
        };
        
        let pre_states = vec![account_state.clone()];
        let post_states = vec![AccountState {
            data: vec![1, 2, 3, 4],
            ..account_state
        }];
        
        let (data_before, data_after, _, _) = analyzer.get_account_changes(
            &keypair.pubkey(),
            &pre_states,
            &post_states,
        );
        
        assert_eq!(data_before, Some(vec![1, 2, 3]));
        assert_eq!(data_after, Some(vec![1, 2, 3, 4]));
    }
}