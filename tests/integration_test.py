#!/usr/bin/env python3
"""
Integration Test Suite for Solana PDA Analyzer
This Python script provides comprehensive integration testing for the API and database
"""

import asyncio
import aiohttp
import json
import sys
import argparse
import time
from typing import Dict, List, Optional, Any
from dataclasses import dataclass
import subprocess
import os
import signal

@dataclass
class TestResult:
    name: str
    success: bool
    duration: float
    error: Optional[str] = None
    details: Optional[Dict[str, Any]] = None

class Colors:
    RED = '\033[0;31m'
    GREEN = '\033[0;32m'
    YELLOW = '\033[1;33m'
    BLUE = '\033[0;34m'
    NC = '\033[0m'  # No Color

class PdaAnalyzerTester:
    def __init__(self, base_url: str = "http://localhost:8080"):
        self.base_url = base_url
        self.session: Optional[aiohttp.ClientSession] = None
        self.results: List[TestResult] = []
        
    async def __aenter__(self):
        self.session = aiohttp.ClientSession()
        return self
        
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        if self.session:
            await self.session.close()
    
    def log_info(self, message: str):
        print(f"{Colors.BLUE}[INFO]{Colors.NC} {message}")
    
    def log_success(self, message: str):
        print(f"{Colors.GREEN}[SUCCESS]{Colors.NC} {message}")
    
    def log_warning(self, message: str):
        print(f"{Colors.YELLOW}[WARNING]{Colors.NC} {message}")
    
    def log_error(self, message: str):
        print(f"{Colors.RED}[ERROR]{Colors.NC} {message}")
    
    async def make_request(self, method: str, endpoint: str, **kwargs) -> aiohttp.ClientResponse:
        """Make HTTP request to the API"""
        url = f"{self.base_url}{endpoint}"
        async with self.session.request(method, url, **kwargs) as response:
            return response
    
    async def test_health_check(self) -> TestResult:
        """Test the health check endpoint"""
        start_time = time.time()
        try:
            response = await self.make_request('GET', '/health')
            duration = time.time() - start_time
            
            if response.status == 200:
                data = await response.json()
                if data.get('success') is True:
                    return TestResult("health_check", True, duration)
                else:
                    return TestResult("health_check", False, duration, "Invalid response format")
            else:
                return TestResult("health_check", False, duration, f"HTTP {response.status}")
        except Exception as e:
            duration = time.time() - start_time
            return TestResult("health_check", False, duration, str(e))
    
    async def test_pda_analysis(self) -> TestResult:
        """Test PDA analysis endpoint"""
        start_time = time.time()
        try:
            payload = {
                "address": "11111111111111111111111111111111",
                "program_id": "11111111111111111111111111111111"
            }
            
            response = await self.make_request(
                'POST', 
                '/api/v1/analyze/pda',
                json=payload,
                headers={'Content-Type': 'application/json'}
            )
            
            duration = time.time() - start_time
            
            if response.status == 200:
                data = await response.json()
                if data.get('success') is True and 'data' in data:
                    result_data = data['data']
                    expected_fields = ['address', 'program_id', 'derived_successfully']
                    
                    if all(field in result_data for field in expected_fields):
                        return TestResult("pda_analysis", True, duration, details=result_data)
                    else:
                        return TestResult("pda_analysis", False, duration, "Missing expected fields")
                else:
                    return TestResult("pda_analysis", False, duration, "Invalid response format")
            else:
                error_text = await response.text()
                return TestResult("pda_analysis", False, duration, f"HTTP {response.status}: {error_text}")
                
        except Exception as e:
            duration = time.time() - start_time
            return TestResult("pda_analysis", False, duration, str(e))
    
    async def test_batch_pda_analysis(self) -> TestResult:
        """Test batch PDA analysis endpoint"""
        start_time = time.time()
        try:
            payload = {
                "addresses": [
                    {
                        "address": "11111111111111111111111111111111",
                        "program_id": "11111111111111111111111111111111"
                    },
                    {
                        "address": "22222222222222222222222222222222",
                        "program_id": "11111111111111111111111111111111"
                    }
                ]
            }
            
            response = await self.make_request(
                'POST',
                '/api/v1/analyze/pda/batch',
                json=payload,
                headers={'Content-Type': 'application/json'}
            )
            
            duration = time.time() - start_time
            
            if response.status == 200:
                data = await response.json()
                if data.get('success') is True and isinstance(data.get('data'), list):
                    results = data['data']
                    if len(results) == 2:
                        return TestResult("batch_pda_analysis", True, duration, details={'count': len(results)})
                    else:
                        return TestResult("batch_pda_analysis", False, duration, f"Expected 2 results, got {len(results)}")
                else:
                    return TestResult("batch_pda_analysis", False, duration, "Invalid response format")
            else:
                error_text = await response.text()
                return TestResult("batch_pda_analysis", False, duration, f"HTTP {response.status}: {error_text}")
                
        except Exception as e:
            duration = time.time() - start_time
            return TestResult("batch_pda_analysis", False, duration, str(e))
    
    async def test_list_programs(self) -> TestResult:
        """Test programs listing endpoint"""
        start_time = time.time()
        try:
            response = await self.make_request('GET', '/api/v1/programs')
            duration = time.time() - start_time
            
            if response.status == 200:
                data = await response.json()
                if data.get('success') is True and isinstance(data.get('data'), list):
                    return TestResult("list_programs", True, duration, details={'count': len(data['data'])})
                else:
                    return TestResult("list_programs", False, duration, "Invalid response format")
            else:
                error_text = await response.text()
                return TestResult("list_programs", False, duration, f"HTTP {response.status}: {error_text}")
                
        except Exception as e:
            duration = time.time() - start_time
            return TestResult("list_programs", False, duration, str(e))
    
    async def test_list_transactions(self) -> TestResult:
        """Test transactions listing endpoint"""
        start_time = time.time()
        try:
            response = await self.make_request('GET', '/api/v1/transactions?limit=10')
            duration = time.time() - start_time
            
            if response.status == 200:
                data = await response.json()
                if data.get('success') is True and isinstance(data.get('data'), list):
                    return TestResult("list_transactions", True, duration, details={'count': len(data['data'])})
                else:
                    return TestResult("list_transactions", False, duration, "Invalid response format")
            else:
                error_text = await response.text()
                return TestResult("list_transactions", False, duration, f"HTTP {response.status}: {error_text}")
                
        except Exception as e:
            duration = time.time() - start_time
            return TestResult("list_transactions", False, duration, str(e))
    
    async def test_list_pdas(self) -> TestResult:
        """Test PDAs listing endpoint"""
        start_time = time.time()
        try:
            response = await self.make_request('GET', '/api/v1/pdas?limit=10')
            duration = time.time() - start_time
            
            if response.status == 200:
                data = await response.json()
                if data.get('success') is True and isinstance(data.get('data'), list):
                    return TestResult("list_pdas", True, duration, details={'count': len(data['data'])})
                else:
                    return TestResult("list_pdas", False, duration, "Invalid response format")
            else:
                error_text = await response.text()
                return TestResult("list_pdas", False, duration, f"HTTP {response.status}: {error_text}")
                
        except Exception as e:
            duration = time.time() - start_time
            return TestResult("list_pdas", False, duration, str(e))
    
    async def test_database_metrics(self) -> TestResult:
        """Test database metrics endpoint"""
        start_time = time.time()
        try:
            response = await self.make_request('GET', '/api/v1/analytics/database')
            duration = time.time() - start_time
            
            if response.status == 200:
                data = await response.json()
                if data.get('success') is True and isinstance(data.get('data'), dict):
                    metrics = data['data']
                    expected_fields = ['total_programs', 'total_transactions', 'total_pdas', 'total_interactions']
                    
                    if all(field in metrics for field in expected_fields):
                        return TestResult("database_metrics", True, duration, details=metrics)
                    else:
                        return TestResult("database_metrics", False, duration, "Missing expected metrics fields")
                else:
                    return TestResult("database_metrics", False, duration, "Invalid response format")
            else:
                error_text = await response.text()
                return TestResult("database_metrics", False, duration, f"HTTP {response.status}: {error_text}")
                
        except Exception as e:
            duration = time.time() - start_time
            return TestResult("database_metrics", False, duration, str(e))
    
    async def test_error_handling(self) -> TestResult:
        """Test error handling with invalid requests"""
        start_time = time.time()
        try:
            # Test invalid PDA address
            payload = {
                "address": "invalid_address",
                "program_id": "11111111111111111111111111111111"
            }
            
            response = await self.make_request(
                'POST',
                '/api/v1/analyze/pda',
                json=payload,
                headers={'Content-Type': 'application/json'}
            )
            
            duration = time.time() - start_time
            
            # Should return 400 for invalid address
            if response.status == 400:
                return TestResult("error_handling", True, duration, details={'status': 400})
            else:
                return TestResult("error_handling", False, duration, f"Expected 400, got {response.status}")
                
        except Exception as e:
            duration = time.time() - start_time
            return TestResult("error_handling", False, duration, str(e))
    
    async def test_nonexistent_endpoints(self) -> TestResult:
        """Test nonexistent endpoints return 404"""
        start_time = time.time()
        try:
            response = await self.make_request('GET', '/api/v1/nonexistent')
            duration = time.time() - start_time
            
            if response.status == 404:
                return TestResult("nonexistent_endpoints", True, duration, details={'status': 404})
            else:
                return TestResult("nonexistent_endpoints", False, duration, f"Expected 404, got {response.status}")
                
        except Exception as e:
            duration = time.time() - start_time
            return TestResult("nonexistent_endpoints", False, duration, str(e))
    
    async def test_concurrent_requests(self) -> TestResult:
        """Test handling of concurrent requests"""
        start_time = time.time()
        try:
            # Send 10 concurrent health check requests
            tasks = []
            for _ in range(10):
                task = asyncio.create_task(self.make_request('GET', '/health'))
                tasks.append(task)
            
            responses = await asyncio.gather(*tasks, return_exceptions=True)
            duration = time.time() - start_time
            
            # Check that all requests succeeded
            success_count = 0
            for response in responses:
                if isinstance(response, aiohttp.ClientResponse) and response.status == 200:
                    success_count += 1
            
            if success_count == 10:
                return TestResult("concurrent_requests", True, duration, details={'success_count': success_count})
            else:
                return TestResult("concurrent_requests", False, duration, f"Only {success_count}/10 requests succeeded")
                
        except Exception as e:
            duration = time.time() - start_time
            return TestResult("concurrent_requests", False, duration, str(e))
    
    async def run_all_tests(self) -> List[TestResult]:
        """Run all integration tests"""
        self.log_info("Starting integration tests...")
        
        # Define test suite
        tests = [
            ("Health Check", self.test_health_check),
            ("PDA Analysis", self.test_pda_analysis),
            ("Batch PDA Analysis", self.test_batch_pda_analysis),
            ("List Programs", self.test_list_programs),
            ("List Transactions", self.test_list_transactions),
            ("List PDAs", self.test_list_pdas),
            ("Database Metrics", self.test_database_metrics),
            ("Error Handling", self.test_error_handling),
            ("Nonexistent Endpoints", self.test_nonexistent_endpoints),
            ("Concurrent Requests", self.test_concurrent_requests),
        ]
        
        results = []
        
        for test_name, test_func in tests:
            self.log_info(f"Running test: {test_name}")
            try:
                result = await test_func()
                results.append(result)
                
                if result.success:
                    self.log_success(f"{test_name} passed ({result.duration:.3f}s)")
                    if result.details:
                        self.log_info(f"  Details: {result.details}")
                else:
                    self.log_error(f"{test_name} failed ({result.duration:.3f}s): {result.error}")
                    
            except Exception as e:
                result = TestResult(test_name, False, 0.0, str(e))
                results.append(result)
                self.log_error(f"{test_name} failed with exception: {e}")
        
        return results
    
    def print_summary(self, results: List[TestResult]):
        """Print test summary"""
        passed = sum(1 for r in results if r.success)
        total = len(results)
        
        print("\n" + "="*60)
        print("TEST SUMMARY")
        print("="*60)
        
        for result in results:
            status = "PASS" if result.success else "FAIL"
            color = Colors.GREEN if result.success else Colors.RED
            print(f"{color}{status}{Colors.NC} {result.name} ({result.duration:.3f}s)")
            if not result.success and result.error:
                print(f"      Error: {result.error}")
        
        print("\n" + "-"*60)
        overall_color = Colors.GREEN if passed == total else Colors.RED
        print(f"{overall_color}Overall: {passed}/{total} tests passed{Colors.NC}")
        
        if passed == total:
            print(f"{Colors.GREEN}All tests passed!{Colors.NC}")
            return True
        else:
            print(f"{Colors.RED}{total - passed} tests failed{Colors.NC}")
            return False

async def main():
    parser = argparse.ArgumentParser(description="Solana PDA Analyzer Integration Tests")
    parser.add_argument("--url", default="http://localhost:8080", help="Base URL for API server")
    parser.add_argument("--timeout", type=int, default=30, help="Request timeout in seconds")
    parser.add_argument("--verbose", action="store_true", help="Verbose output")
    
    args = parser.parse_args()
    
    # Test if server is accessible
    try:
        async with aiohttp.ClientSession(timeout=aiohttp.ClientTimeout(total=args.timeout)) as session:
            async with session.get(f"{args.url}/health") as response:
                if response.status != 200:
                    print(f"{Colors.RED}Server not accessible at {args.url}{Colors.NC}")
                    sys.exit(1)
    except Exception as e:
        print(f"{Colors.RED}Cannot connect to server at {args.url}: {e}{Colors.NC}")
        print(f"{Colors.YELLOW}Make sure the server is running with: cargo run --bin pda-analyzer serve{Colors.NC}")
        sys.exit(1)
    
    # Run tests
    async with PdaAnalyzerTester(args.url) as tester:
        results = await tester.run_all_tests()
        success = tester.print_summary(results)
        
        # Return appropriate exit code
        sys.exit(0 if success else 1)

if __name__ == "__main__":
    asyncio.run(main())