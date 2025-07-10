use solana_pda_analyzer_core::{PdaAnalyzerError, Result, AccountState};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use solana_sdk::transaction::Transaction;
use solana_rpc_client_api::config::{RpcTransactionConfig, RpcAccountInfoConfig};
use solana_transaction_status::{EncodedConfirmedTransactionWithStatusMeta, UiTransactionEncoding};
use std::str::FromStr;
use std::collections::HashMap;
use tracing::{info, warn, error};

#[derive(Debug, Clone)]
pub struct SolanaClient {
    rpc_client: RpcClient,
    commitment: solana_sdk::commitment_config::CommitmentConfig,
}

impl SolanaClient {
    pub fn new(rpc_url: &str) -> Self {
        let rpc_client = RpcClient::new(rpc_url.to_string());
        let commitment = solana_sdk::commitment_config::CommitmentConfig::confirmed();
        
        Self {
            rpc_client,
            commitment,
        }
    }

    pub async fn get_transaction_with_meta(
        &self,
        signature: &str,
    ) -> Result<EncodedConfirmedTransactionWithStatusMeta> {
        let sig = Signature::from_str(signature)
            .map_err(|e| PdaAnalyzerError::TransactionParsingError(e.to_string()))?;
        
        let config = RpcTransactionConfig {
            encoding: Some(UiTransactionEncoding::JsonParsed),
            commitment: Some(self.commitment),
            max_supported_transaction_version: Some(0),
        };
        
        let transaction = self.rpc_client
            .get_transaction_with_config(&sig, config)
            .map_err(|e| PdaAnalyzerError::NetworkError(e.to_string()))?;
        
        Ok(transaction)
    }

    pub async fn get_account_info(&self, pubkey: &Pubkey) -> Result<Option<AccountState>> {
        let config = RpcAccountInfoConfig {
            encoding: Some(solana_account_decoder::UiAccountEncoding::Base64),
            commitment: Some(self.commitment),
            data_slice: None,
            min_context_slot: None,
        };
        
        match self.rpc_client.get_account_with_config(pubkey, config) {
            Ok(account_info) => {
                if let Some(account) = account_info.value {
                    Ok(Some(AccountState {
                        pubkey: *pubkey,
                        lamports: account.lamports,
                        data: account.data.decode().unwrap_or_default(),
                        owner: account.owner,
                        executable: account.executable,
                        rent_epoch: account.rent_epoch,
                    }))
                } else {
                    Ok(None)
                }
            }
            Err(e) => {
                warn!("Failed to get account info for {}: {}", pubkey, e);
                Ok(None)
            }
        }
    }

    pub async fn get_multiple_accounts(&self, pubkeys: &[Pubkey]) -> Result<Vec<Option<AccountState>>> {
        let config = RpcAccountInfoConfig {
            encoding: Some(solana_account_decoder::UiAccountEncoding::Base64),
            commitment: Some(self.commitment),
            data_slice: None,
            min_context_slot: None,
        };
        
        let accounts = self.rpc_client
            .get_multiple_accounts_with_config(pubkeys, config)
            .map_err(|e| PdaAnalyzerError::NetworkError(e.to_string()))?;
        
        let mut result = Vec::new();
        for (i, account_opt) in accounts.value.iter().enumerate() {
            if let Some(account) = account_opt {
                result.push(Some(AccountState {
                    pubkey: pubkeys[i],
                    lamports: account.lamports,
                    data: account.data.decode().unwrap_or_default(),
                    owner: account.owner,
                    executable: account.executable,
                    rent_epoch: account.rent_epoch,
                }));
            } else {
                result.push(None);
            }
        }
        
        Ok(result)
    }

    pub async fn get_signatures_for_address(
        &self,
        address: &Pubkey,
        limit: Option<usize>,
        before: Option<&Signature>,
    ) -> Result<Vec<String>> {
        let config = solana_rpc_client_api::config::RpcGetConfirmedSignaturesForAddress2Config {
            before: before.map(|s| s.to_string()),
            until: None,
            limit,
            commitment: Some(self.commitment),
        };
        
        let signatures = self.rpc_client
            .get_signatures_for_address_with_config(address, config)
            .map_err(|e| PdaAnalyzerError::NetworkError(e.to_string()))?;
        
        Ok(signatures.into_iter().map(|s| s.signature).collect())
    }

    pub async fn get_slot(&self) -> Result<u64> {
        self.rpc_client
            .get_slot_with_commitment(self.commitment)
            .map_err(|e| PdaAnalyzerError::NetworkError(e.to_string()))
    }

    pub async fn get_block_time(&self, slot: u64) -> Result<Option<i64>> {
        match self.rpc_client.get_block_time(slot) {
            Ok(time) => Ok(Some(time)),
            Err(_) => Ok(None),
        }
    }

    pub fn parse_transaction_from_encoded(
        &self,
        encoded_transaction: &EncodedConfirmedTransactionWithStatusMeta,
    ) -> Result<(Transaction, Vec<AccountState>, Vec<AccountState>)> {
        // This is a simplified parser - in practice you'd need to handle all the
        // different encoding formats and extract pre/post account states
        
        // For now, return empty account states
        // A full implementation would parse the transaction meta to extract account states
        let pre_account_states = Vec::new();
        let post_account_states = Vec::new();
        
        // Parse the transaction - this is simplified
        // In practice, you'd need to handle the different encoding formats
        let transaction = Transaction::default(); // Placeholder
        
        Ok((transaction, pre_account_states, post_account_states))
    }
}

#[derive(Debug, Clone)]
pub struct TransactionFetcher {
    client: SolanaClient,
    batch_size: usize,
}

impl TransactionFetcher {
    pub fn new(client: SolanaClient, batch_size: usize) -> Self {
        Self {
            client,
            batch_size,
        }
    }

    pub async fn fetch_transactions_for_program(
        &self,
        program_id: &Pubkey,
        limit: Option<usize>,
    ) -> Result<Vec<String>> {
        info!("Fetching transactions for program: {}", program_id);
        
        let signatures = self.client
            .get_signatures_for_address(program_id, limit, None)
            .await?;
        
        info!("Found {} transactions for program {}", signatures.len(), program_id);
        Ok(signatures)
    }

    pub async fn fetch_transaction_batch(
        &self,
        signatures: &[String],
    ) -> Result<Vec<EncodedConfirmedTransactionWithStatusMeta>> {
        let mut transactions = Vec::new();
        
        for signature in signatures {
            match self.client.get_transaction_with_meta(signature).await {
                Ok(tx) => transactions.push(tx),
                Err(e) => {
                    error!("Failed to fetch transaction {}: {}", signature, e);
                    continue;
                }
            }
        }
        
        Ok(transactions)
    }

    pub async fn stream_transactions<F>(
        &self,
        program_id: &Pubkey,
        mut callback: F,
    ) -> Result<()>
    where
        F: FnMut(EncodedConfirmedTransactionWithStatusMeta) -> Result<()>,
    {
        let mut before_signature = None;
        let mut total_processed = 0;
        
        loop {
            let signatures = self.client
                .get_signatures_for_address(program_id, Some(self.batch_size), before_signature.as_ref())
                .await?;
            
            if signatures.is_empty() {
                break;
            }
            
            let transactions = self.fetch_transaction_batch(&signatures).await?;
            
            for transaction in transactions {
                callback(transaction)?;
                total_processed += 1;
            }
            
            // Update the before signature for the next batch
            if let Some(last_sig) = signatures.last() {
                before_signature = Some(Signature::from_str(last_sig)
                    .map_err(|e| PdaAnalyzerError::TransactionParsingError(e.to_string()))?);
            }
            
            info!("Processed {} transactions so far", total_processed);
        }
        
        info!("Finished processing {} transactions", total_processed);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_solana_client_creation() {
        let client = SolanaClient::new("https://api.mainnet-beta.solana.com");
        assert_eq!(client.commitment, solana_sdk::commitment_config::CommitmentConfig::confirmed());
    }
    
    #[test]
    fn test_transaction_fetcher_creation() {
        let client = SolanaClient::new("https://api.mainnet-beta.solana.com");
        let fetcher = TransactionFetcher::new(client, 100);
        assert_eq!(fetcher.batch_size, 100);
    }
}