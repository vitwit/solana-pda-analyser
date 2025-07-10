// API client for Solana PDA Analyzer

class ApiClient {
    constructor(baseUrl = 'http://localhost:8080') {
        this.baseUrl = baseUrl;
    }

    async request(endpoint, options = {}) {
        const url = `${this.baseUrl}${endpoint}`;
        const config = {
            headers: {
                'Content-Type': 'application/json',
                ...options.headers,
            },
            ...options,
        };

        try {
            const response = await fetch(url, config);
            const data = await response.json();

            if (!response.ok) {
                throw new Error(data.error || data.message || 'Request failed');
            }

            return data;
        } catch (error) {
            console.error('API request failed:', error);
            throw error;
        }
    }

    // Health check
    async healthCheck() {
        return this.request('/health');
    }

    // PDA Analysis
    async analyzePda(address, programId) {
        return this.request('/api/v1/analyze/pda', {
            method: 'POST',
            body: JSON.stringify({ address, program_id: programId }),
        });
    }

    async batchAnalyzePda(addresses) {
        return this.request('/api/v1/analyze/pda/batch', {
            method: 'POST',
            body: JSON.stringify({ addresses }),
        });
    }

    // Programs
    async getPrograms(params = {}) {
        const queryString = new URLSearchParams(params).toString();
        const endpoint = `/api/v1/programs${queryString ? `?${queryString}` : ''}`;
        return this.request(endpoint);
    }

    async getProgram(programId) {
        return this.request(`/api/v1/programs/${programId}`);
    }

    async getProgramStats(programId) {
        return this.request(`/api/v1/programs/${programId}/stats`);
    }

    async getProgramPatterns(programId) {
        return this.request(`/api/v1/programs/${programId}/patterns`);
    }

    // Transactions
    async getTransactions(params = {}) {
        const queryString = new URLSearchParams(params).toString();
        const endpoint = `/api/v1/transactions${queryString ? `?${queryString}` : ''}`;
        return this.request(endpoint);
    }

    async getTransaction(signature) {
        return this.request(`/api/v1/transactions/${signature}`);
    }

    // PDAs
    async getPdas(params = {}) {
        const queryString = new URLSearchParams(params).toString();
        const endpoint = `/api/v1/pdas${queryString ? `?${queryString}` : ''}`;
        return this.request(endpoint);
    }

    async getPda(address) {
        return this.request(`/api/v1/pdas/${address}`);
    }

    // Analytics
    async getDatabaseMetrics() {
        return this.request('/api/v1/analytics/database');
    }
}

// Export the client instance
const apiClient = new ApiClient();

// Utility functions
function handleApiError(error) {
    console.error('API Error:', error);
    
    let message = 'An unexpected error occurred';
    
    if (error.message) {
        message = error.message;
    } else if (typeof error === 'string') {
        message = error;
    }
    
    showError(message);
}

function showError(message) {
    const errorModal = new bootstrap.Modal(document.getElementById('errorModal'));
    document.getElementById('error-message').textContent = message;
    errorModal.show();
}

function showLoading() {
    const loadingModal = new bootstrap.Modal(document.getElementById('loadingModal'));
    loadingModal.show();
}

function hideLoading() {
    const loadingModal = bootstrap.Modal.getInstance(document.getElementById('loadingModal'));
    if (loadingModal) {
        loadingModal.hide();
    }
}

// Format utilities
function formatAddress(address, length = 8) {
    if (!address || address.length < length * 2) {
        return address;
    }
    return `${address.slice(0, length)}...${address.slice(-length)}`;
}

function formatTimestamp(timestamp) {
    if (!timestamp) return 'N/A';
    return new Date(timestamp).toLocaleString();
}

function formatNumber(num) {
    if (num === null || num === undefined) return 'N/A';
    return num.toLocaleString();
}

function formatPercentage(num) {
    if (num === null || num === undefined) return 'N/A';
    return `${num.toFixed(1)}%`;
}

function copyToClipboard(text) {
    navigator.clipboard.writeText(text).then(() => {
        // Show a brief success message
        const toast = document.createElement('div');
        toast.className = 'toast position-fixed top-0 end-0 m-3';
        toast.innerHTML = `
            <div class="toast-body">
                <i class="fas fa-check text-success me-2"></i>
                Copied to clipboard!
            </div>
        `;
        document.body.appendChild(toast);
        
        const bsToast = new bootstrap.Toast(toast);
        bsToast.show();
        
        setTimeout(() => {
            toast.remove();
        }, 3000);
    }).catch(err => {
        console.error('Failed to copy text: ', err);
    });
}

// Validation utilities
function isValidSolanaAddress(address) {
    // Basic validation for Solana addresses (base58, 32-44 characters)
    const base58Regex = /^[1-9A-HJ-NP-Za-km-z]{32,44}$/;
    return base58Regex.test(address);
}

function validatePdaAnalysisInput(address, programId) {
    const errors = [];
    
    if (!address || address.trim() === '') {
        errors.push('PDA address is required');
    } else if (!isValidSolanaAddress(address.trim())) {
        errors.push('Invalid PDA address format');
    }
    
    if (!programId || programId.trim() === '') {
        errors.push('Program ID is required');
    } else if (!isValidSolanaAddress(programId.trim())) {
        errors.push('Invalid Program ID format');
    }
    
    return errors;
}

// Export utilities for use in other files
window.apiClient = apiClient;
window.handleApiError = handleApiError;
window.showError = showError;
window.showLoading = showLoading;
window.hideLoading = hideLoading;
window.formatAddress = formatAddress;
window.formatTimestamp = formatTimestamp;
window.formatNumber = formatNumber;
window.formatPercentage = formatPercentage;
window.copyToClipboard = copyToClipboard;
window.isValidSolanaAddress = isValidSolanaAddress;
window.validatePdaAnalysisInput = validatePdaAnalysisInput;