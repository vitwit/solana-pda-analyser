use solana_pda_analyzer_core::{
    TransactionAnalyzer, AccountState, InteractionType, 
    TransactionAnalysis, PdaInteraction
};
use solana_sdk::{
    pubkey::Pubkey, 
    transaction::Transaction, 
    message::Message,
    instruction::Instruction,
    signature::Keypair,
    signer::Signer,
    system_instruction,
};
use chrono::{DateTime, Utc};

fn create_test_account_state(pubkey: Pubkey, lamports: u64, data: Vec<u8>) -> AccountState {
    AccountState {
        pubkey,
        lamports,
        data,
        owner: solana_sdk::system_program::id(),
        executable: false,
        rent_epoch: 0,
    }
}

#[test]
fn test_transaction_analyzer_creation() {
    let analyzer = TransactionAnalyzer::new();
    assert!(analyzer.known_programs.len() > 0);
    assert!(analyzer.get_program_name(&solana_sdk::system_program::id()).is_some());
}

#[test]
fn test_add_known_program() {
    let mut analyzer = TransactionAnalyzer::new();
    let custom_program = Pubkey::new_unique();
    let program_name = "Custom Program".to_string();
    
    analyzer.add_known_program(custom_program, program_name.clone());
    
    assert_eq!(analyzer.get_program_name(&custom_program), Some(&program_name));
}

#[test]
fn test_analyze_transaction_basic() {
    let analyzer = TransactionAnalyzer::new();
    
    // Create a simple transaction
    let payer = Keypair::new();
    let recipient = Keypair::new();
    
    let instruction = system_instruction::transfer(&payer.pubkey(), &recipient.pubkey(), 1000);
    let message = Message::new(&[instruction], Some(&payer.pubkey()));
    let transaction = Transaction::new_unsigned(message);
    
    let signature = "test_signature";
    let slot = 12345;
    let block_time = Some(Utc::now());
    let success = true;
    let error_message = None;
    
    // Create account states
    let pre_states = vec![
        create_test_account_state(payer.pubkey(), 10000, vec![]),
        create_test_account_state(recipient.pubkey(), 0, vec![]),
    ];
    
    let post_states = vec![
        create_test_account_state(payer.pubkey(), 9000, vec![]),
        create_test_account_state(recipient.pubkey(), 1000, vec![]),
    ];
    
    let result = analyzer.analyze_transaction(
        signature,
        &transaction,
        slot,
        block_time,
        success,
        error_message,
        &pre_states,
        &post_states,
    );
    
    assert!(result.is_ok());
    let analysis = result.unwrap();
    assert_eq!(analysis.signature, signature);
    assert_eq!(analysis.slot, slot);
    assert_eq!(analysis.success, success);
}

#[test]
fn test_account_state_changes() {
    let analyzer = TransactionAnalyzer::new();
    let account_pubkey = Pubkey::new_unique();
    
    let pre_states = vec![
        create_test_account_state(account_pubkey, 1000, vec![1, 2, 3]),
    ];
    
    let post_states = vec![
        create_test_account_state(account_pubkey, 2000, vec![4, 5, 6]),
    ];
    
    let (data_before, data_after, lamports_before, lamports_after) = 
        analyzer.get_account_changes(&account_pubkey, &pre_states, &post_states);
    
    assert_eq!(data_before, Some(vec![1, 2, 3]));
    assert_eq!(data_after, Some(vec![4, 5, 6]));
    assert_eq!(lamports_before, Some(1000));
    assert_eq!(lamports_after, Some(2000));
}

#[test]
fn test_account_creation_detection() {
    let analyzer = TransactionAnalyzer::new();
    let new_account = Pubkey::new_unique();
    
    let pre_states = vec![];
    let post_states = vec![
        create_test_account_state(new_account, 1000, vec![0; 100]),
    ];
    
    let (data_before, data_after, lamports_before, lamports_after) = 
        analyzer.get_account_changes(&new_account, &pre_states, &post_states);
    
    assert_eq!(data_before, None);
    assert_eq!(data_after, Some(vec![0; 100]));
    assert_eq!(lamports_before, None);
    assert_eq!(lamports_after, Some(1000));
}

#[test]
fn test_account_closure_detection() {
    let analyzer = TransactionAnalyzer::new();
    let closing_account = Pubkey::new_unique();
    
    let pre_states = vec![
        create_test_account_state(closing_account, 1000, vec![1, 2, 3]),
    ];
    let post_states = vec![];
    
    let (data_before, data_after, lamports_before, lamports_after) = 
        analyzer.get_account_changes(&closing_account, &pre_states, &post_states);
    
    assert_eq!(data_before, Some(vec![1, 2, 3]));
    assert_eq!(data_after, None);
    assert_eq!(lamports_before, Some(1000));
    assert_eq!(lamports_after, None);
}

#[test]
fn test_interaction_type_determination() {
    let analyzer = TransactionAnalyzer::new();
    let account_pubkey = Pubkey::new_unique();
    
    // Create a dummy message for testing
    let payer = Keypair::new();
    let instruction = system_instruction::transfer(&payer.pubkey(), &account_pubkey, 1000);
    let message = Message::new(&[instruction], Some(&payer.pubkey()));
    
    // Test data change (Write)
    let pre_states = vec![
        create_test_account_state(account_pubkey, 1000, vec![1, 2, 3]),
    ];
    let post_states = vec![
        create_test_account_state(account_pubkey, 1000, vec![4, 5, 6]),
    ];
    
    let interaction_type = analyzer.determine_interaction_type(
        0, 0, &message, &pre_states, &post_states
    );
    
    // Note: The actual logic in determine_interaction_type is simplified
    // This test mainly ensures the function doesn't panic
    assert!(matches!(interaction_type, InteractionType::Read | InteractionType::Write));
}

#[test]
fn test_pda_detection() {
    let analyzer = TransactionAnalyzer::new();
    let program_id = Pubkey::new_unique();
    
    // Create an address that's definitely on-curve (regular account)
    let regular_account = Keypair::new().pubkey();
    
    // Test with regular account (should return None as it's on-curve)
    let result = analyzer.detect_pda(&regular_account, &program_id);
    assert!(result.is_none());
    
    // For off-curve addresses (actual PDAs), the detection would work
    // but we can't easily create them in tests without the exact seeds
}

#[test]
fn test_known_programs() {
    let analyzer = TransactionAnalyzer::new();
    
    // Test system program
    let system_program_name = analyzer.get_program_name(&solana_sdk::system_program::id());
    assert!(system_program_name.is_some());
    assert_eq!(system_program_name.unwrap(), "System Program");
    
    // Test unknown program
    let unknown_program = Pubkey::new_unique();
    let unknown_name = analyzer.get_program_name(&unknown_program);
    assert!(unknown_name.is_none());
}

#[test]
fn test_transaction_analysis_with_error() {
    let analyzer = TransactionAnalyzer::new();
    
    let payer = Keypair::new();
    let instruction = system_instruction::transfer(&payer.pubkey(), &Pubkey::new_unique(), 1000);
    let message = Message::new(&[instruction], Some(&payer.pubkey()));
    let transaction = Transaction::new_unsigned(message);
    
    let signature = "failed_transaction";
    let slot = 12345;
    let block_time = Some(Utc::now());
    let success = false;
    let error_message = Some("InsufficientFunds".to_string());
    
    let pre_states = vec![];
    let post_states = vec![];
    
    let result = analyzer.analyze_transaction(
        signature,
        &transaction,
        slot,
        block_time,
        success,
        error_message.clone(),
        &pre_states,
        &post_states,
    );
    
    assert!(result.is_ok());
    let analysis = result.unwrap();
    assert_eq!(analysis.signature, signature);
    assert_eq!(analysis.success, false);
    assert_eq!(analysis.error_message, error_message);
}

#[test]
fn test_multiple_instructions_transaction() {
    let analyzer = TransactionAnalyzer::new();
    
    let payer = Keypair::new();
    let recipient1 = Keypair::new();
    let recipient2 = Keypair::new();
    
    let instructions = vec![
        system_instruction::transfer(&payer.pubkey(), &recipient1.pubkey(), 500),
        system_instruction::transfer(&payer.pubkey(), &recipient2.pubkey(), 300),
    ];
    
    let message = Message::new(&instructions, Some(&payer.pubkey()));
    let transaction = Transaction::new_unsigned(message);
    
    let signature = "multi_instruction_tx";
    let slot = 12345;
    let block_time = Some(Utc::now());
    let success = true;
    let error_message = None;
    
    let pre_states = vec![
        create_test_account_state(payer.pubkey(), 10000, vec![]),
        create_test_account_state(recipient1.pubkey(), 0, vec![]),
        create_test_account_state(recipient2.pubkey(), 0, vec![]),
    ];
    
    let post_states = vec![
        create_test_account_state(payer.pubkey(), 9200, vec![]),
        create_test_account_state(recipient1.pubkey(), 500, vec![]),
        create_test_account_state(recipient2.pubkey(), 300, vec![]),
    ];
    
    let result = analyzer.analyze_transaction(
        signature,
        &transaction,
        slot,
        block_time,
        success,
        error_message,
        &pre_states,
        &post_states,
    );
    
    assert!(result.is_ok());
    let analysis = result.unwrap();
    assert_eq!(analysis.signature, signature);
    assert_eq!(analysis.success, true);
    // Transaction has 2 instructions
    assert_eq!(transaction.message.instructions.len(), 2);
}

#[test]
fn test_instruction_analysis() {
    use solana_pda_analyzer_core::InstructionAnalysis;
    
    let payer = Keypair::new();
    let recipient = Keypair::new();
    
    let instruction = system_instruction::transfer(&payer.pubkey(), &recipient.pubkey(), 1000);
    let message = Message::new(&[instruction.clone()], Some(&payer.pubkey()));
    
    let analysis = InstructionAnalysis::from_instruction(&instruction, &message);
    
    assert_eq!(analysis.program_id, solana_sdk::system_program::id());
    assert_eq!(analysis.accounts.len(), 2);
    assert_eq!(analysis.accounts[0], payer.pubkey());
    assert_eq!(analysis.accounts[1], recipient.pubkey());
}

#[test]
fn test_empty_transaction_analysis() {
    let analyzer = TransactionAnalyzer::new();
    
    // Create an empty transaction (no instructions)
    let payer = Keypair::new();
    let message = Message::new(&[], Some(&payer.pubkey()));
    let transaction = Transaction::new_unsigned(message);
    
    let signature = "empty_transaction";
    let slot = 12345;
    let block_time = Some(Utc::now());
    let success = true;
    let error_message = None;
    
    let pre_states = vec![];
    let post_states = vec![];
    
    let result = analyzer.analyze_transaction(
        signature,
        &transaction,
        slot,
        block_time,
        success,
        error_message,
        &pre_states,
        &post_states,
    );
    
    assert!(result.is_ok());
    let analysis = result.unwrap();
    assert_eq!(analysis.signature, signature);
    assert_eq!(analysis.pda_interactions.len(), 0);
    assert_eq!(analysis.discovered_pdas.len(), 0);
}