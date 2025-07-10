use solana_pda_analyzer_database::{
    DatabaseRepository, DatabaseConfig, DatabaseMigrator,
    CreateProgramRequest, CreateTransactionRequest, CreatePdaRequest, CreateAccountInteractionRequest,
    ProgramFilter, TransactionFilter, PdaFilter, AccountInteractionFilter,
};
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;

async fn setup_test_database() -> PgPool {
    let config = DatabaseConfig {
        host: "localhost".to_string(),
        port: 5432,
        database: format!("test_pda_analyzer_{}", Uuid::new_v4().to_string().replace('-', "")),
        username: "postgres".to_string(),
        password: "".to_string(),
        max_connections: 5,
        min_connections: 1,
        acquire_timeout: 30,
        idle_timeout: 600,
        max_lifetime: 1800,
    };

    let migrator = DatabaseMigrator::new(config.database_url());
    
    // Create test database
    if let Err(_) = migrator.ensure_database_exists().await {
        // If we can't create database, skip these tests
        panic!("Cannot create test database - ensure PostgreSQL is running");
    }
    
    let pool = config.create_pool().await.expect("Failed to create pool");
    migrator.run_migrations(&pool).await.expect("Failed to run migrations");
    
    pool
}

async fn cleanup_test_database(pool: &PgPool, database_name: &str) {
    pool.close().await;
    
    // Drop test database
    let config = DatabaseConfig {
        database: "postgres".to_string(),
        ..Default::default()
    };
    
    if let Ok(admin_pool) = config.create_pool().await {
        let _ = sqlx::query(&format!("DROP DATABASE IF EXISTS {}", database_name))
            .execute(&admin_pool)
            .await;
        admin_pool.close().await;
    }
}

#[tokio::test]
async fn test_program_operations() {
    let pool = setup_test_database().await;
    let repo = DatabaseRepository::new(pool.clone());
    
    // Test create program
    let request = CreateProgramRequest {
        program_id: "11111111111111111111111111111111".to_string(),
        name: Some("Test Program".to_string()),
        description: Some("A test program".to_string()),
    };
    
    let program = repo.create_program(request).await.expect("Failed to create program");
    assert_eq!(program.program_id, "11111111111111111111111111111111");
    assert_eq!(program.name, Some("Test Program".to_string()));
    
    // Test get program
    let retrieved = repo.get_program_by_id(&program.program_id).await.expect("Failed to get program");
    assert!(retrieved.is_some());
    let retrieved_program = retrieved.unwrap();
    assert_eq!(retrieved_program.id, program.id);
    
    // Test list programs
    let programs = repo.list_programs(ProgramFilter::default()).await.expect("Failed to list programs");
    assert!(programs.len() > 0);
    
    // Test update program (upsert)
    let update_request = CreateProgramRequest {
        program_id: "11111111111111111111111111111111".to_string(),
        name: Some("Updated Test Program".to_string()),
        description: Some("An updated test program".to_string()),
    };
    
    let updated_program = repo.create_program(update_request).await.expect("Failed to update program");
    assert_eq!(updated_program.name, Some("Updated Test Program".to_string()));
    
    cleanup_test_database(&pool, &pool.connect_options().get_database().unwrap()).await;
}

#[tokio::test]
async fn test_transaction_operations() {
    let pool = setup_test_database().await;
    let repo = DatabaseRepository::new(pool.clone());
    
    // Test create transaction
    let request = CreateTransactionRequest {
        signature: "test_signature_123".to_string(),
        slot: 12345,
        block_time: Some(Utc::now()),
        fee: Some(5000),
        success: true,
        error_message: None,
    };
    
    let transaction = repo.create_transaction(request).await.expect("Failed to create transaction");
    assert_eq!(transaction.signature, "test_signature_123");
    assert_eq!(transaction.slot, 12345);
    assert_eq!(transaction.success, true);
    
    // Test get transaction
    let retrieved = repo.get_transaction_by_signature(&transaction.signature).await.expect("Failed to get transaction");
    assert!(retrieved.is_some());
    let retrieved_tx = retrieved.unwrap();
    assert_eq!(retrieved_tx.id, transaction.id);
    
    // Test list transactions
    let transactions = repo.list_transactions(TransactionFilter::default()).await.expect("Failed to list transactions");
    assert!(transactions.len() > 0);
    
    // Test list with filters
    let filter = TransactionFilter {
        success: Some(true),
        limit: Some(10),
        ..Default::default()
    };
    let filtered = repo.list_transactions(filter).await.expect("Failed to list filtered transactions");
    assert!(filtered.iter().all(|tx| tx.success));
    
    cleanup_test_database(&pool, &pool.connect_options().get_database().unwrap()).await;
}

#[tokio::test]
async fn test_pda_operations() {
    let pool = setup_test_database().await;
    let repo = DatabaseRepository::new(pool.clone());
    
    // First create a program
    let program_request = CreateProgramRequest {
        program_id: "22222222222222222222222222222222".to_string(),
        name: Some("PDA Test Program".to_string()),
        description: None,
    };
    let program = repo.create_program(program_request).await.expect("Failed to create program");
    
    // Test create PDA
    let seeds_json = serde_json::json!([
        {"String": "metadata"},
        {"U64": 12345}
    ]);
    
    let request = CreatePdaRequest {
        address: "33333333333333333333333333333333".to_string(),
        program_id: program.id,
        seeds: seeds_json,
        bump: 254,
        first_seen_transaction: None,
        data_hash: Some("abcd1234".to_string()),
    };
    
    let pda = repo.create_pda(request).await.expect("Failed to create PDA");
    assert_eq!(pda.address, "33333333333333333333333333333333");
    assert_eq!(pda.bump, 254);
    
    // Test get PDA
    let retrieved = repo.get_pda_by_address(&pda.address).await.expect("Failed to get PDA");
    assert!(retrieved.is_some());
    let retrieved_pda = retrieved.unwrap();
    assert_eq!(retrieved_pda.id, pda.id);
    
    // Test list PDAs
    let pdas = repo.list_pdas(PdaFilter::default()).await.expect("Failed to list PDAs");
    assert!(pdas.len() > 0);
    
    // Test list with program filter
    let filter = PdaFilter {
        program_id: Some(program.id),
        ..Default::default()
    };
    let filtered = repo.list_pdas(filter).await.expect("Failed to list filtered PDAs");
    assert!(filtered.iter().all(|p| p.program_id == program.id));
    
    cleanup_test_database(&pool, &pool.connect_options().get_database().unwrap()).await;
}

#[tokio::test]
async fn test_account_interaction_operations() {
    let pool = setup_test_database().await;
    let repo = DatabaseRepository::new(pool.clone());
    
    // Setup: Create program, transaction, and PDA
    let program_request = CreateProgramRequest {
        program_id: "44444444444444444444444444444444".to_string(),
        name: Some("Interaction Test Program".to_string()),
        description: None,
    };
    let program = repo.create_program(program_request).await.expect("Failed to create program");
    
    let tx_request = CreateTransactionRequest {
        signature: "interaction_test_tx".to_string(),
        slot: 54321,
        block_time: Some(Utc::now()),
        fee: Some(5000),
        success: true,
        error_message: None,
    };
    let transaction = repo.create_transaction(tx_request).await.expect("Failed to create transaction");
    
    let pda_request = CreatePdaRequest {
        address: "55555555555555555555555555555555".to_string(),
        program_id: program.id,
        seeds: serde_json::json!([{"String": "test"}]),
        bump: 253,
        first_seen_transaction: Some(transaction.id),
        data_hash: None,
    };
    let pda = repo.create_pda(pda_request).await.expect("Failed to create PDA");
    
    // Test create account interaction
    let request = CreateAccountInteractionRequest {
        transaction_id: transaction.id,
        pda_id: pda.id,
        instruction_index: 0,
        interaction_type: "write".to_string(),
        data_before: Some(vec![1, 2, 3]),
        data_after: Some(vec![4, 5, 6]),
        lamports_before: Some(1000000),
        lamports_after: Some(2000000),
    };
    
    let interaction = repo.create_account_interaction(request).await.expect("Failed to create interaction");
    assert_eq!(interaction.interaction_type, "write");
    assert_eq!(interaction.instruction_index, 0);
    
    // Test list interactions
    let interactions = repo.list_account_interactions(AccountInteractionFilter::default()).await.expect("Failed to list interactions");
    assert!(interactions.len() > 0);
    
    // Test list with transaction filter
    let filter = AccountInteractionFilter {
        transaction_id: Some(transaction.id),
        ..Default::default()
    };
    let filtered = repo.list_account_interactions(filter).await.expect("Failed to list filtered interactions");
    assert!(filtered.iter().all(|i| i.transaction_id == transaction.id));
    
    // Test list with PDA filter
    let filter = AccountInteractionFilter {
        pda_id: Some(pda.id),
        ..Default::default()
    };
    let filtered = repo.list_account_interactions(filter).await.expect("Failed to list PDA interactions");
    assert!(filtered.iter().all(|i| i.pda_id == pda.id));
    
    cleanup_test_database(&pool, &pool.connect_options().get_database().unwrap()).await;
}

#[tokio::test]
async fn test_database_metrics() {
    let pool = setup_test_database().await;
    let repo = DatabaseRepository::new(pool.clone());
    
    // Add some test data
    let program_request = CreateProgramRequest {
        program_id: "66666666666666666666666666666666".to_string(),
        name: Some("Metrics Test Program".to_string()),
        description: None,
    };
    let program = repo.create_program(program_request).await.expect("Failed to create program");
    
    let tx_request = CreateTransactionRequest {
        signature: "metrics_test_tx".to_string(),
        slot: 98765,
        block_time: Some(Utc::now()),
        fee: Some(5000),
        success: true,
        error_message: None,
    };
    let _transaction = repo.create_transaction(tx_request).await.expect("Failed to create transaction");
    
    let pda_request = CreatePdaRequest {
        address: "77777777777777777777777777777777".to_string(),
        program_id: program.id,
        seeds: serde_json::json!([{"String": "metrics"}]),
        bump: 252,
        first_seen_transaction: None,
        data_hash: None,
    };
    let _pda = repo.create_pda(pda_request).await.expect("Failed to create PDA");
    
    // Test database metrics
    let metrics = repo.get_database_metrics().await.expect("Failed to get metrics");
    assert!(metrics.total_programs >= 1);
    assert!(metrics.total_transactions >= 1);
    assert!(metrics.total_pdas >= 1);
    
    cleanup_test_database(&pool, &pool.connect_options().get_database().unwrap()).await;
}

#[tokio::test]
async fn test_program_stats() {
    let pool = setup_test_database().await;
    let repo = DatabaseRepository::new(pool.clone());
    
    // Create test data for stats
    let program_request = CreateProgramRequest {
        program_id: "88888888888888888888888888888888".to_string(),
        name: Some("Stats Test Program".to_string()),
        description: None,
    };
    let program = repo.create_program(program_request).await.expect("Failed to create program");
    
    // Create multiple transactions
    for i in 0..5 {
        let tx_request = CreateTransactionRequest {
            signature: format!("stats_test_tx_{}", i),
            slot: 100000 + i as i64,
            block_time: Some(Utc::now()),
            fee: Some(5000),
            success: i % 2 == 0, // Alternate success/failure
            error_message: if i % 2 == 0 { None } else { Some("Test error".to_string()) },
        };
        let _transaction = repo.create_transaction(tx_request).await.expect("Failed to create transaction");
    }
    
    // Create multiple PDAs
    for i in 0..3 {
        let pda_request = CreatePdaRequest {
            address: format!("stats_pda_{:032}", i),
            program_id: program.id,
            seeds: serde_json::json!([{"String": format!("stats_{}", i)}]),
            bump: 250 + i as i16,
            first_seen_transaction: None,
            data_hash: None,
        };
        let _pda = repo.create_pda(pda_request).await.expect("Failed to create PDA");
    }
    
    // Test program stats
    let stats = repo.get_program_stats(program.id).await.expect("Failed to get program stats");
    assert_eq!(stats.program_id, program.id);
    // Note: Stats might be 0 because we haven't created the full relationship chain
    // In a real scenario, interactions would link transactions to PDAs
    
    cleanup_test_database(&pool, &pool.connect_options().get_database().unwrap()).await;
}

#[tokio::test]
async fn test_batch_operations() {
    let pool = setup_test_database().await;
    let repo = DatabaseRepository::new(pool.clone());
    
    // Create program for batch tests
    let program_request = CreateProgramRequest {
        program_id: "99999999999999999999999999999999".to_string(),
        name: Some("Batch Test Program".to_string()),
        description: None,
    };
    let program = repo.create_program(program_request).await.expect("Failed to create program");
    
    // Test batch create PDAs
    let pda_requests = vec![
        CreatePdaRequest {
            address: "batch_pda_1_00000000000000000000000".to_string(),
            program_id: program.id,
            seeds: serde_json::json!([{"String": "batch1"}]),
            bump: 249,
            first_seen_transaction: None,
            data_hash: None,
        },
        CreatePdaRequest {
            address: "batch_pda_2_00000000000000000000000".to_string(),
            program_id: program.id,
            seeds: serde_json::json!([{"String": "batch2"}]),
            bump: 248,
            first_seen_transaction: None,
            data_hash: None,
        },
    ];
    
    let created_pdas = repo.batch_create_pdas(pda_requests).await.expect("Failed to batch create PDAs");
    assert_eq!(created_pdas.len(), 2);
    
    cleanup_test_database(&pool, &pool.connect_options().get_database().unwrap()).await;
}

#[tokio::test]
async fn test_migration_system() {
    let config = DatabaseConfig {
        host: "localhost".to_string(),
        port: 5432,
        database: format!("migration_test_{}", Uuid::new_v4().to_string().replace('-', "")),
        username: "postgres".to_string(),
        password: "".to_string(),
        max_connections: 5,
        min_connections: 1,
        acquire_timeout: 30,
        idle_timeout: 600,
        max_lifetime: 1800,
    };

    let migrator = DatabaseMigrator::new(config.database_url());
    
    // Test database creation
    migrator.ensure_database_exists().await.expect("Failed to ensure database exists");
    
    // Test migration
    let pool = config.create_pool().await.expect("Failed to create pool");
    migrator.run_migrations(&pool).await.expect("Failed to run migrations");
    
    // Verify tables exist
    let table_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public'")
        .fetch_one(&pool)
        .await
        .expect("Failed to count tables");
    
    assert!(table_count.0 >= 6); // We should have at least 6 tables from our schema
    
    cleanup_test_database(&pool, &config.database).await;
}