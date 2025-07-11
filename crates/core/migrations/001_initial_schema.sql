-- Initial schema for Solana PDA Analyzer

-- Programs table
CREATE TABLE programs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    program_id TEXT NOT NULL UNIQUE,
    name TEXT,
    description TEXT,
    total_pdas BIGINT DEFAULT 0,
    last_analyzed TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- PDAs table
CREATE TABLE pdas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    address TEXT NOT NULL,
    program_id TEXT NOT NULL,
    seeds JSONB NOT NULL,
    bump SMALLINT NOT NULL,
    pattern TEXT,
    confidence DOUBLE PRECISION,
    analysis_time_ms BIGINT,
    first_seen_slot BIGINT,
    first_seen_transaction TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(address, program_id)
);

-- Transactions table
CREATE TABLE transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    signature TEXT NOT NULL UNIQUE,
    slot BIGINT NOT NULL,
    block_time TIMESTAMP WITH TIME ZONE,
    program_ids TEXT[] NOT NULL,
    pda_interactions JSONB NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Pattern statistics table
CREATE TABLE pattern_stats (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pattern TEXT NOT NULL,
    program_id TEXT NOT NULL,
    count BIGINT DEFAULT 0,
    avg_confidence DOUBLE PRECISION,
    last_updated TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(pattern, program_id)
);

-- Indexes for better performance
CREATE INDEX idx_pdas_program_id ON pdas(program_id);
CREATE INDEX idx_pdas_pattern ON pdas(pattern);
CREATE INDEX idx_pdas_created_at ON pdas(created_at);
CREATE INDEX idx_pdas_confidence ON pdas(confidence);
CREATE INDEX idx_pdas_address ON pdas(address);

CREATE INDEX idx_programs_program_id ON programs(program_id);
CREATE INDEX idx_programs_name ON programs(name);
CREATE INDEX idx_programs_total_pdas ON programs(total_pdas);

CREATE INDEX idx_transactions_signature ON transactions(signature);
CREATE INDEX idx_transactions_slot ON transactions(slot);
CREATE INDEX idx_transactions_program_ids ON transactions USING GIN(program_ids);
CREATE INDEX idx_transactions_created_at ON transactions(created_at);

CREATE INDEX idx_pattern_stats_pattern ON pattern_stats(pattern);
CREATE INDEX idx_pattern_stats_program_id ON pattern_stats(program_id);

-- Foreign key constraints
ALTER TABLE pdas ADD CONSTRAINT fk_pdas_programs 
    FOREIGN KEY (program_id) REFERENCES programs(program_id) ON DELETE CASCADE;

-- Functions for maintaining statistics
CREATE OR REPLACE FUNCTION update_program_stats()
RETURNS TRIGGER AS $$
BEGIN
    -- Update total PDAs count
    UPDATE programs 
    SET total_pdas = (
        SELECT COUNT(*) FROM pdas WHERE program_id = NEW.program_id
    ),
    updated_at = NOW()
    WHERE program_id = NEW.program_id;
    
    -- Update pattern statistics
    INSERT INTO pattern_stats (pattern, program_id, count, avg_confidence)
    VALUES (
        NEW.pattern,
        NEW.program_id,
        1,
        NEW.confidence
    )
    ON CONFLICT (pattern, program_id) DO UPDATE SET
        count = pattern_stats.count + 1,
        avg_confidence = (pattern_stats.avg_confidence * pattern_stats.count + NEW.confidence) / (pattern_stats.count + 1),
        last_updated = NOW();
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Triggers
CREATE TRIGGER trigger_update_program_stats
    AFTER INSERT OR UPDATE ON pdas
    FOR EACH ROW
    EXECUTE FUNCTION update_program_stats();

-- Views for common queries
CREATE VIEW pda_summary AS
SELECT 
    p.program_id,
    pr.name as program_name,
    p.pattern,
    COUNT(*) as count,
    AVG(p.confidence) as avg_confidence,
    MIN(p.created_at) as first_seen,
    MAX(p.created_at) as last_seen
FROM pdas p
LEFT JOIN programs pr ON p.program_id = pr.program_id
GROUP BY p.program_id, pr.name, p.pattern;

CREATE VIEW recent_analyses AS
SELECT 
    p.*,
    pr.name as program_name
FROM pdas p
LEFT JOIN programs pr ON p.program_id = pr.program_id
WHERE p.created_at > NOW() - INTERVAL '24 hours'
ORDER BY p.created_at DESC;

-- Insert known programs
INSERT INTO programs (program_id, name, description) VALUES
('11111111111111111111111111111111', 'System Program', 'The Solana system program'),
('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA', 'SPL Token', 'SPL Token program'),
('ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL', 'SPL Associated Token Account', 'SPL Associated Token Account program'),
('metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s', 'Metaplex Token Metadata', 'Metaplex Token Metadata program'),
('CndyV3LdqHUfDLmE5naZjVN8rBZz4tqhdefbAnjHG3JR', 'Metaplex Candy Machine', 'Metaplex Candy Machine program'),
('hausS13jsjafwWwGqZTUQRmWyvyxn9EQpqMwV1PBBmk', 'Metaplex Auction House', 'Metaplex Auction House program'),
('9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin', 'Serum DEX', 'Serum decentralized exchange'),
('675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8', 'Raydium AMM', 'Raydium automated market maker'),
('MarBmsSgKXdrN1egZf5sqe1TMai9K1rChYNDJgjq7aD', 'Marinade Finance', 'Marinade liquid staking'),
('namesLPneVptA9Z5rqUDD9tMTWEJwofgaYwp8cawRkX', 'Solana Name Service', 'Solana Name Service program'),
('GovER5Lthms3bLBqWub97yVrMmEogzX7xNjdXpPPCVZw', 'SPL Governance', 'SPL Governance program')
ON CONFLICT (program_id) DO NOTHING;