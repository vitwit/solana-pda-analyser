// Main application logic for Solana PDA Analyzer

class PdaAnalyzerApp {
    constructor() {
        this.charts = {};
        this.refreshInterval = null;
        this.init();
    }

    async init() {
        this.setupEventListeners();
        await this.loadInitialData();
        this.startAutoRefresh();
    }

    setupEventListeners() {
        // PDA Analysis
        document.getElementById('analyze-btn').addEventListener('click', () => this.analyzePda());
        document.getElementById('clear-btn').addEventListener('click', () => this.clearAnalysis());
        
        // Enter key support for analysis
        document.getElementById('pda-address').addEventListener('keypress', (e) => {
            if (e.key === 'Enter') this.analyzePda();
        });
        document.getElementById('program-id').addEventListener('keypress', (e) => {
            if (e.key === 'Enter') this.analyzePda();
        });

        // Refresh buttons
        document.getElementById('refresh-programs').addEventListener('click', () => this.loadPrograms());
        document.getElementById('refresh-transactions').addEventListener('click', () => this.loadTransactions());
        
        // Navigation
        document.querySelectorAll('a[href^="#"]').forEach(anchor => {
            anchor.addEventListener('click', (e) => {
                e.preventDefault();
                const target = document.querySelector(anchor.getAttribute('href'));
                if (target) {
                    target.scrollIntoView({ behavior: 'smooth' });
                }
            });
        });
    }

    async loadInitialData() {
        try {
            await this.checkApiStatus();
            await this.loadDatabaseMetrics();
            await this.loadPrograms();
            await this.loadTransactions();
            await this.loadAnalytics();
        } catch (error) {
            console.error('Failed to load initial data:', error);
        }
    }

    async checkApiStatus() {
        try {
            await apiClient.healthCheck();
            document.getElementById('api-status').textContent = 'Connected';
            document.getElementById('api-status').previousElementSibling.className = 'fas fa-circle text-success me-2';
        } catch (error) {
            document.getElementById('api-status').textContent = 'Disconnected';
            document.getElementById('api-status').previousElementSibling.className = 'fas fa-circle text-danger me-2';
        }
    }

    async loadDatabaseMetrics() {
        try {
            const response = await apiClient.getDatabaseMetrics();
            if (response.success && response.data) {
                const metrics = response.data;
                document.getElementById('total-pdas').textContent = formatNumber(metrics.total_pdas);
                document.getElementById('total-programs').textContent = formatNumber(metrics.total_programs);
                document.getElementById('total-transactions').textContent = formatNumber(metrics.total_transactions);
            }
        } catch (error) {
            console.error('Failed to load database metrics:', error);
        }
    }

    async analyzePda() {
        const address = document.getElementById('pda-address').value.trim();
        const programId = document.getElementById('program-id').value.trim();
        
        // Validate input
        const errors = validatePdaAnalysisInput(address, programId);
        if (errors.length > 0) {
            showError(errors.join('\n'));
            return;
        }

        const analyzeBtn = document.getElementById('analyze-btn');
        const originalText = analyzeBtn.innerHTML;
        
        try {
            // Show loading state
            analyzeBtn.innerHTML = '<i class="fas fa-spinner fa-spin me-2"></i>Analyzing...';
            analyzeBtn.disabled = true;
            
            const response = await apiClient.analyzePda(address, programId);
            
            if (response.success && response.data) {
                this.displayAnalysisResults(response.data);
            } else {
                showError('Analysis failed: ' + (response.error || 'Unknown error'));
            }
        } catch (error) {
            handleApiError(error);
        } finally {
            // Restore button state
            analyzeBtn.innerHTML = originalText;
            analyzeBtn.disabled = false;
        }
    }

    displayAnalysisResults(result) {
        const resultsSection = document.getElementById('analysis-results');
        const resultsContent = document.getElementById('results-content');
        
        let html = '';
        
        if (result.derived_successfully) {
            html = `
                <div class="alert alert-success analysis-success">
                    <h6><i class="fas fa-check-circle me-2"></i>PDA Analysis Successful</h6>
                    <div class="mt-3">
                        <strong>Address:</strong> 
                        <span class="text-truncate-address">${result.address}</span>
                        <i class="fas fa-copy copy-button ms-2" onclick="copyToClipboard('${result.address}')" title="Copy address"></i>
                    </div>
                    <div class="mt-2">
                        <strong>Program ID:</strong> 
                        <span class="text-truncate-address">${result.program_id}</span>
                        <i class="fas fa-copy copy-button ms-2" onclick="copyToClipboard('${result.program_id}')" title="Copy program ID"></i>
                    </div>
                    <div class="mt-2">
                        <strong>Bump:</strong> ${result.bump}
                    </div>
                    ${result.seeds ? this.formatSeeds(result.seeds) : ''}
                </div>
            `;
        } else {
            html = `
                <div class="alert alert-warning analysis-failure">
                    <h6><i class="fas fa-exclamation-triangle me-2"></i>PDA Analysis Failed</h6>
                    <p>Could not derive seeds for the given PDA address and program ID combination.</p>
                    <div class="mt-3">
                        <strong>Address:</strong> 
                        <span class="text-truncate-address">${result.address}</span>
                        <i class="fas fa-copy copy-button ms-2" onclick="copyToClipboard('${result.address}')" title="Copy address"></i>
                    </div>
                    <div class="mt-2">
                        <strong>Program ID:</strong> 
                        <span class="text-truncate-address">${result.program_id}</span>
                        <i class="fas fa-copy copy-button ms-2" onclick="copyToClipboard('${result.program_id}')" title="Copy program ID"></i>
                    </div>
                    <div class="mt-3">
                        <small class="text-muted">
                            This could mean the address is not a valid PDA for this program, or the seed derivation pattern is not in our database.
                        </small>
                    </div>
                </div>
            `;
        }
        
        resultsContent.innerHTML = html;
        resultsSection.style.display = 'block';
        resultsSection.scrollIntoView({ behavior: 'smooth' });
    }

    formatSeeds(seeds) {
        if (!seeds || seeds.length === 0) {
            return '<div class="mt-2"><strong>Seeds:</strong> None</div>';
        }
        
        let html = '<div class="mt-3"><strong>Seeds:</strong></div>';
        seeds.forEach((seed, index) => {
            const seedType = this.getSeedType(seed);
            const seedValue = this.formatSeedValue(seed);
            
            html += `
                <div class="seed-display">
                    <span class="seed-type">${seedType}</span>
                    <span class="seed-value">${seedValue}</span>
                </div>
            `;
        });
        
        return html;
    }

    getSeedType(seed) {
        // This is a simplified type detection
        // In a real implementation, you'd have proper type information from the API
        if (typeof seed === 'string') {
            return 'String';
        } else if (typeof seed === 'number') {
            return 'Number';
        } else if (seed && typeof seed === 'object') {
            if (seed.String) return 'String';
            if (seed.Bytes) return 'Bytes';
            if (seed.Pubkey) return 'Pubkey';
            if (seed.U64) return 'U64';
            if (seed.U32) return 'U32';
            if (seed.U16) return 'U16';
            if (seed.U8) return 'U8';
        }
        return 'Unknown';
    }

    formatSeedValue(seed) {
        if (typeof seed === 'string') {
            return seed;
        } else if (typeof seed === 'number') {
            return seed.toString();
        } else if (seed && typeof seed === 'object') {
            if (seed.String) return seed.String;
            if (seed.Bytes) return `[${seed.Bytes.join(', ')}]`;
            if (seed.Pubkey) return seed.Pubkey;
            if (seed.U64) return seed.U64.toString();
            if (seed.U32) return seed.U32.toString();
            if (seed.U16) return seed.U16.toString();
            if (seed.U8) return seed.U8.toString();
        }
        return JSON.stringify(seed);
    }

    clearAnalysis() {
        document.getElementById('pda-address').value = '';
        document.getElementById('program-id').value = '';
        document.getElementById('analysis-results').style.display = 'none';
    }

    async loadPrograms() {
        try {
            const response = await apiClient.getPrograms({ limit: 10 });
            if (response.success && response.data) {
                this.displayPrograms(response.data);
            }
        } catch (error) {
            console.error('Failed to load programs:', error);
            document.getElementById('programs-table').innerHTML = `
                <tr><td colspan="6" class="text-center text-danger">Failed to load programs</td></tr>
            `;
        }
    }

    displayPrograms(programs) {
        const tbody = document.getElementById('programs-table');
        
        if (programs.length === 0) {
            tbody.innerHTML = '<tr><td colspan="6" class="text-center">No programs found</td></tr>';
            return;
        }
        
        tbody.innerHTML = programs.map(program => `
            <tr>
                <td>
                    <span class="text-truncate-address">${program.program_id}</span>
                    <i class="fas fa-copy copy-button ms-2" onclick="copyToClipboard('${program.program_id}')" title="Copy program ID"></i>
                </td>
                <td>${program.name || 'Unknown'}</td>
                <td>-</td>
                <td>-</td>
                <td>-</td>
                <td>
                    <button class="btn btn-sm btn-outline-primary" onclick="app.viewProgramDetails('${program.program_id}')">
                        <i class="fas fa-eye"></i>
                    </button>
                </td>
            </tr>
        `).join('');
    }

    async loadTransactions() {
        try {
            const response = await apiClient.getTransactions({ limit: 10 });
            if (response.success && response.data) {
                this.displayTransactions(response.data);
            }
        } catch (error) {
            console.error('Failed to load transactions:', error);
            document.getElementById('transactions-table').innerHTML = `
                <tr><td colspan="6" class="text-center text-danger">Failed to load transactions</td></tr>
            `;
        }
    }

    displayTransactions(transactions) {
        const tbody = document.getElementById('transactions-table');
        
        if (transactions.length === 0) {
            tbody.innerHTML = '<tr><td colspan="6" class="text-center">No transactions found</td></tr>';
            return;
        }
        
        tbody.innerHTML = transactions.map(tx => `
            <tr>
                <td>
                    <span class="text-truncate-address">${tx.signature}</span>
                    <i class="fas fa-copy copy-button ms-2" onclick="copyToClipboard('${tx.signature}')" title="Copy signature"></i>
                </td>
                <td>${formatNumber(tx.slot)}</td>
                <td>
                    <span class="badge ${tx.success ? 'badge-success' : 'badge-danger'}">
                        ${tx.success ? 'Success' : 'Failed'}
                    </span>
                </td>
                <td>${formatTimestamp(tx.block_time)}</td>
                <td>-</td>
                <td>
                    <button class="btn btn-sm btn-outline-primary" onclick="app.viewTransactionDetails('${tx.signature}')">
                        <i class="fas fa-eye"></i>
                    </button>
                </td>
            </tr>
        `).join('');
    }

    async loadAnalytics() {
        try {
            // Load analytics data and create charts
            this.createPdaDistributionChart();
            this.createSuccessRateChart();
            this.createActivityTimelineChart();
        } catch (error) {
            console.error('Failed to load analytics:', error);
        }
    }

    createPdaDistributionChart() {
        const ctx = document.getElementById('pda-distribution-chart').getContext('2d');
        
        this.charts.pdaDistribution = new Chart(ctx, {
            type: 'doughnut',
            data: {
                labels: ['Token Program', 'NFT Program', 'DeFi Program', 'Other'],
                datasets: [{
                    data: [45, 25, 20, 10],
                    backgroundColor: [
                        '#9945FF',
                        '#14F195',
                        '#FF6B6B',
                        '#4ECDC4'
                    ]
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: {
                    legend: {
                        position: 'bottom'
                    }
                }
            }
        });
    }

    createSuccessRateChart() {
        const ctx = document.getElementById('success-rate-chart').getContext('2d');
        
        this.charts.successRate = new Chart(ctx, {
            type: 'bar',
            data: {
                labels: ['Success', 'Failed'],
                datasets: [{
                    data: [85, 15],
                    backgroundColor: ['#14F195', '#FF6B6B']
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: {
                    legend: {
                        display: false
                    }
                },
                scales: {
                    y: {
                        beginAtZero: true,
                        max: 100
                    }
                }
            }
        });
    }

    createActivityTimelineChart() {
        const ctx = document.getElementById('activity-timeline-chart').getContext('2d');
        
        // Generate sample data for the last 7 days
        const days = [];
        const transactions = [];
        const pdas = [];
        
        for (let i = 6; i >= 0; i--) {
            const date = new Date();
            date.setDate(date.getDate() - i);
            days.push(date.toLocaleDateString());
            transactions.push(Math.floor(Math.random() * 100) + 50);
            pdas.push(Math.floor(Math.random() * 20) + 5);
        }
        
        this.charts.activityTimeline = new Chart(ctx, {
            type: 'line',
            data: {
                labels: days,
                datasets: [{
                    label: 'Transactions',
                    data: transactions,
                    borderColor: '#9945FF',
                    backgroundColor: 'rgba(153, 69, 255, 0.1)',
                    tension: 0.4
                }, {
                    label: 'New PDAs',
                    data: pdas,
                    borderColor: '#14F195',
                    backgroundColor: 'rgba(20, 241, 149, 0.1)',
                    tension: 0.4
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: {
                    legend: {
                        position: 'top'
                    }
                },
                scales: {
                    y: {
                        beginAtZero: true
                    }
                }
            }
        });
    }

    async viewProgramDetails(programId) {
        // In a real implementation, this would show a modal or navigate to a detailed view
        console.log('Viewing program details for:', programId);
        showError('Program details view not implemented yet');
    }

    async viewTransactionDetails(signature) {
        // In a real implementation, this would show a modal or navigate to a detailed view
        console.log('Viewing transaction details for:', signature);
        showError('Transaction details view not implemented yet');
    }

    startAutoRefresh() {
        // Refresh data every 30 seconds
        this.refreshInterval = setInterval(() => {
            this.loadDatabaseMetrics();
            this.checkApiStatus();
        }, 30000);
    }

    stopAutoRefresh() {
        if (this.refreshInterval) {
            clearInterval(this.refreshInterval);
            this.refreshInterval = null;
        }
    }
}

// Initialize the app when the page loads
document.addEventListener('DOMContentLoaded', () => {
    window.app = new PdaAnalyzerApp();
});

// Handle page unload
window.addEventListener('beforeunload', () => {
    if (window.app) {
        window.app.stopAutoRefresh();
    }
});