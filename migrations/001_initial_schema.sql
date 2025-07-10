-- Create initial database schema for Solana PDA analyzer

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Programs table to store program information
CREATE TABLE programs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    program_id VARCHAR(44) UNIQUE NOT NULL,
    name VARCHAR(255),
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- PDA patterns table to store known PDA derivation patterns
CREATE TABLE pda_patterns (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    program_id UUID NOT NULL REFERENCES programs(id) ON DELETE CASCADE,
    pattern_name VARCHAR(255) NOT NULL,
    seeds_template JSONB NOT NULL,
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Transactions table to store analyzed transactions
CREATE TABLE transactions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    signature VARCHAR(88) UNIQUE NOT NULL,
    slot BIGINT NOT NULL,
    block_time TIMESTAMP WITH TIME ZONE,
    fee BIGINT,
    success BOOLEAN NOT NULL DEFAULT false,
    error_message TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- PDAs table to store discovered PDAs
CREATE TABLE pdas (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    address VARCHAR(44) UNIQUE NOT NULL,
    program_id UUID NOT NULL REFERENCES programs(id) ON DELETE CASCADE,
    seeds JSONB NOT NULL,
    bump SMALLINT NOT NULL,
    first_seen_transaction UUID REFERENCES transactions(id),
    data_hash VARCHAR(64),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Account interactions table to track PDA usage in transactions
CREATE TABLE account_interactions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    transaction_id UUID NOT NULL REFERENCES transactions(id) ON DELETE CASCADE,
    pda_id UUID NOT NULL REFERENCES pdas(id) ON DELETE CASCADE,
    instruction_index INTEGER NOT NULL,
    interaction_type VARCHAR(50) NOT NULL, -- 'read', 'write', 'create', 'close'
    data_before BYTEA,
    data_after BYTEA,
    lamports_before BIGINT,
    lamports_after BIGINT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Seed derivation attempts table for tracking analysis attempts
CREATE TABLE seed_derivation_attempts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    pda_address VARCHAR(44) NOT NULL,
    program_id UUID NOT NULL REFERENCES programs(id) ON DELETE CASCADE,
    attempted_seeds JSONB NOT NULL,
    success BOOLEAN NOT NULL DEFAULT false,
    attempted_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX idx_programs_program_id ON programs(program_id);
CREATE INDEX idx_pda_patterns_program_id ON pda_patterns(program_id);
CREATE INDEX idx_transactions_signature ON transactions(signature);
CREATE INDEX idx_transactions_slot ON transactions(slot);
CREATE INDEX idx_transactions_block_time ON transactions(block_time);
CREATE INDEX idx_pdas_address ON pdas(address);
CREATE INDEX idx_pdas_program_id ON pdas(program_id);
CREATE INDEX idx_account_interactions_transaction_id ON account_interactions(transaction_id);
CREATE INDEX idx_account_interactions_pda_id ON account_interactions(pda_id);
CREATE INDEX idx_seed_derivation_attempts_pda_address ON seed_derivation_attempts(pda_address);
CREATE INDEX idx_seed_derivation_attempts_program_id ON seed_derivation_attempts(program_id);

-- Create updated_at trigger function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Add triggers to update updated_at automatically
CREATE TRIGGER update_programs_updated_at BEFORE UPDATE ON programs FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_pda_patterns_updated_at BEFORE UPDATE ON pda_patterns FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_transactions_updated_at BEFORE UPDATE ON transactions FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_pdas_updated_at BEFORE UPDATE ON pdas FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();