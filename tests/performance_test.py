#!/usr/bin/env python3
"""
Performance and Load Testing Suite for Solana PDA Analyzer
Tests API performance under various load conditions
"""

import asyncio
import aiohttp
import time
import statistics
import argparse
import json
import sys
from typing import List, Dict, Tuple
from dataclasses import dataclass
import numpy as np

@dataclass
class PerformanceResult:
    test_name: str
    total_requests: int
    duration: float
    successful_requests: int
    failed_requests: int
    avg_response_time: float
    min_response_time: float
    max_response_time: float
    percentile_95: float
    percentile_99: float
    requests_per_second: float
    errors: List[str]

class Colors:
    RED = '\033[0;31m'
    GREEN = '\033[0;32m'
    YELLOW = '\033[1;33m'
    BLUE = '\033[0;34m'
    NC = '\033[0m'

class PerformanceTester:
    def __init__(self, base_url: str = "http://localhost:8080"):
        self.base_url = base_url
        self.session: aiohttp.ClientSession = None
        
    async def __aenter__(self):
        connector = aiohttp.TCPConnector(limit=1000, limit_per_host=100)
        timeout = aiohttp.ClientTimeout(total=30)
        self.session = aiohttp.ClientSession(connector=connector, timeout=timeout)
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
    
    async def make_request(self, method: str, endpoint: str, **kwargs) -> Tuple[bool, float, str]:
        """Make HTTP request and return (success, response_time, error)"""
        start_time = time.time()
        try:
            url = f"{self.base_url}{endpoint}"
            async with self.session.request(method, url, **kwargs) as response:
                await response.read()  # Ensure response is fully read
                response_time = time.time() - start_time
                return response.status < 400, response_time, ""
        except Exception as e:
            response_time = time.time() - start_time
            return False, response_time, str(e)
    
    async def run_concurrent_requests(self, 
                                    method: str, 
                                    endpoint: str, 
                                    concurrent_users: int, 
                                    requests_per_user: int,
                                    **kwargs) -> PerformanceResult:
        """Run concurrent requests and measure performance"""
        
        total_requests = concurrent_users * requests_per_user
        self.log_info(f"Running {total_requests} requests ({concurrent_users} concurrent users, {requests_per_user} requests each)")
        
        # Create semaphore to limit concurrent requests
        semaphore = asyncio.Semaphore(concurrent_users)
        
        async def make_request_with_semaphore():
            async with semaphore:
                return await self.make_request(method, endpoint, **kwargs)
        
        # Create all tasks
        tasks = []
        for _ in range(total_requests):
            task = asyncio.create_task(make_request_with_semaphore())
            tasks.append(task)
        
        # Run all requests and measure time
        start_time = time.time()
        results = await asyncio.gather(*tasks, return_exceptions=True)
        total_duration = time.time() - start_time
        
        # Process results
        successful_requests = 0
        failed_requests = 0
        response_times = []
        errors = []
        
        for result in results:
            if isinstance(result, Exception):
                failed_requests += 1
                errors.append(str(result))
            else:
                success, response_time, error = result
                response_times.append(response_time)
                if success:
                    successful_requests += 1
                else:
                    failed_requests += 1
                    if error:
                        errors.append(error)
        
        # Calculate statistics
        if response_times:
            avg_response_time = statistics.mean(response_times)
            min_response_time = min(response_times)
            max_response_time = max(response_times)
            percentile_95 = np.percentile(response_times, 95)
            percentile_99 = np.percentile(response_times, 99)
        else:
            avg_response_time = min_response_time = max_response_time = 0
            percentile_95 = percentile_99 = 0
        
        requests_per_second = total_requests / total_duration if total_duration > 0 else 0
        
        return PerformanceResult(
            test_name=f"{method} {endpoint}",
            total_requests=total_requests,
            duration=total_duration,
            successful_requests=successful_requests,
            failed_requests=failed_requests,
            avg_response_time=avg_response_time,
            min_response_time=min_response_time,
            max_response_time=max_response_time,
            percentile_95=percentile_95,
            percentile_99=percentile_99,
            requests_per_second=requests_per_second,
            errors=errors[:10]  # Keep only first 10 errors
        )
    
    async def test_health_endpoint_load(self, concurrent_users: int = 50, requests_per_user: int = 10) -> PerformanceResult:
        """Test health endpoint under load"""
        return await self.run_concurrent_requests(
            'GET', '/health', concurrent_users, requests_per_user
        )
    
    async def test_pda_analysis_load(self, concurrent_users: int = 20, requests_per_user: int = 5) -> PerformanceResult:
        """Test PDA analysis endpoint under load"""
        payload = {
            "address": "11111111111111111111111111111111",
            "program_id": "11111111111111111111111111111111"
        }
        
        return await self.run_concurrent_requests(
            'POST', '/api/v1/analyze/pda', concurrent_users, requests_per_user,
            json=payload,
            headers={'Content-Type': 'application/json'}
        )
    
    async def test_batch_analysis_load(self, concurrent_users: int = 10, requests_per_user: int = 3) -> PerformanceResult:
        """Test batch PDA analysis under load"""
        # Create batch with varying sizes
        addresses = []
        for i in range(5):  # 5 addresses per batch
            addresses.append({
                "address": f"{i:044d}",
                "program_id": "11111111111111111111111111111111"
            })
        
        payload = {"addresses": addresses}
        
        return await self.run_concurrent_requests(
            'POST', '/api/v1/analyze/pda/batch', concurrent_users, requests_per_user,
            json=payload,
            headers={'Content-Type': 'application/json'}
        )
    
    async def test_database_queries_load(self, concurrent_users: int = 30, requests_per_user: int = 10) -> PerformanceResult:
        """Test database query endpoints under load"""
        return await self.run_concurrent_requests(
            'GET', '/api/v1/analytics/database', concurrent_users, requests_per_user
        )
    
    async def test_list_endpoints_load(self, concurrent_users: int = 25, requests_per_user: int = 8) -> PerformanceResult:
        """Test list endpoints under load"""
        return await self.run_concurrent_requests(
            'GET', '/api/v1/programs?limit=20', concurrent_users, requests_per_user
        )
    
    async def test_sustained_load(self, duration_seconds: int = 60) -> PerformanceResult:
        """Test sustained load over time"""
        self.log_info(f"Running sustained load test for {duration_seconds} seconds")
        
        start_time = time.time()
        end_time = start_time + duration_seconds
        
        successful_requests = 0
        failed_requests = 0
        response_times = []
        errors = []
        
        # Run requests continuously for the specified duration
        tasks = []
        while time.time() < end_time:
            if len(tasks) < 50:  # Maintain up to 50 concurrent requests
                task = asyncio.create_task(self.make_request('GET', '/health'))
                tasks.append(task)
            
            # Check completed tasks
            completed_tasks = [task for task in tasks if task.done()]
            for task in completed_tasks:
                try:
                    success, response_time, error = await task
                    response_times.append(response_time)
                    if success:
                        successful_requests += 1
                    else:
                        failed_requests += 1
                        if error:
                            errors.append(error)
                except Exception as e:
                    failed_requests += 1
                    errors.append(str(e))
                
                tasks.remove(task)
            
            await asyncio.sleep(0.01)  # Small delay to prevent CPU spinning
        
        # Wait for remaining tasks
        for task in tasks:
            try:
                success, response_time, error = await task
                response_times.append(response_time)
                if success:
                    successful_requests += 1
                else:
                    failed_requests += 1
                    if error:
                        errors.append(error)
            except Exception as e:
                failed_requests += 1
                errors.append(str(e))
        
        total_duration = time.time() - start_time
        total_requests = successful_requests + failed_requests
        
        # Calculate statistics
        if response_times:
            avg_response_time = statistics.mean(response_times)
            min_response_time = min(response_times)
            max_response_time = max(response_times)
            percentile_95 = np.percentile(response_times, 95)
            percentile_99 = np.percentile(response_times, 99)
        else:
            avg_response_time = min_response_time = max_response_time = 0
            percentile_95 = percentile_99 = 0
        
        requests_per_second = total_requests / total_duration if total_duration > 0 else 0
        
        return PerformanceResult(
            test_name=f"Sustained Load ({duration_seconds}s)",
            total_requests=total_requests,
            duration=total_duration,
            successful_requests=successful_requests,
            failed_requests=failed_requests,
            avg_response_time=avg_response_time,
            min_response_time=min_response_time,
            max_response_time=max_response_time,
            percentile_95=percentile_95,
            percentile_99=percentile_99,
            requests_per_second=requests_per_second,
            errors=errors[:10]
        )
    
    async def test_memory_usage_under_load(self) -> PerformanceResult:
        """Test with large payloads to check memory usage"""
        # Create large batch payload
        addresses = []
        for i in range(100):  # 100 addresses per batch
            addresses.append({
                "address": f"{i:044d}",
                "program_id": "11111111111111111111111111111111"
            })
        
        payload = {"addresses": addresses}
        
        return await self.run_concurrent_requests(
            'POST', '/api/v1/analyze/pda/batch', 5, 3,  # Lower concurrency for large payloads
            json=payload,
            headers={'Content-Type': 'application/json'}
        )
    
    def print_result(self, result: PerformanceResult):
        """Print detailed performance result"""
        print(f"\n{Colors.BLUE}{'='*60}{Colors.NC}")
        print(f"{Colors.BLUE}Test: {result.test_name}{Colors.NC}")
        print(f"{Colors.BLUE}{'='*60}{Colors.NC}")
        
        # Basic metrics
        print(f"Total Requests:      {result.total_requests}")
        print(f"Duration:            {result.duration:.2f}s")
        print(f"Successful:          {result.successful_requests}")
        print(f"Failed:              {result.failed_requests}")
        print(f"Success Rate:        {(result.successful_requests/result.total_requests*100):.1f}%")
        print(f"Requests/Second:     {result.requests_per_second:.2f}")
        
        # Response time metrics
        print(f"\nResponse Times:")
        print(f"  Average:           {result.avg_response_time*1000:.2f}ms")
        print(f"  Minimum:           {result.min_response_time*1000:.2f}ms")
        print(f"  Maximum:           {result.max_response_time*1000:.2f}ms")
        print(f"  95th Percentile:   {result.percentile_95*1000:.2f}ms")
        print(f"  99th Percentile:   {result.percentile_99*1000:.2f}ms")
        
        # Error summary
        if result.errors:
            print(f"\nErrors (first 10):")
            for i, error in enumerate(result.errors[:10], 1):
                print(f"  {i}. {error}")
        
        # Performance assessment
        self.assess_performance(result)
    
    def assess_performance(self, result: PerformanceResult):
        """Assess if performance is acceptable"""
        print(f"\n{Colors.YELLOW}Performance Assessment:{Colors.NC}")
        
        # Success rate assessment
        success_rate = result.successful_requests / result.total_requests * 100
        if success_rate >= 99:
            print(f"{Colors.GREEN}✓{Colors.NC} Excellent success rate ({success_rate:.1f}%)")
        elif success_rate >= 95:
            print(f"{Colors.YELLOW}⚠{Colors.NC} Good success rate ({success_rate:.1f}%)")
        else:
            print(f"{Colors.RED}✗{Colors.NC} Poor success rate ({success_rate:.1f}%)")
        
        # Response time assessment
        avg_time_ms = result.avg_response_time * 1000
        if avg_time_ms <= 100:
            print(f"{Colors.GREEN}✓{Colors.NC} Excellent average response time ({avg_time_ms:.1f}ms)")
        elif avg_time_ms <= 500:
            print(f"{Colors.YELLOW}⚠{Colors.NC} Good average response time ({avg_time_ms:.1f}ms)")
        else:
            print(f"{Colors.RED}✗{Colors.NC} Poor average response time ({avg_time_ms:.1f}ms)")
        
        # Throughput assessment
        if result.requests_per_second >= 100:
            print(f"{Colors.GREEN}✓{Colors.NC} Excellent throughput ({result.requests_per_second:.1f} req/s)")
        elif result.requests_per_second >= 50:
            print(f"{Colors.YELLOW}⚠{Colors.NC} Good throughput ({result.requests_per_second:.1f} req/s)")
        else:
            print(f"{Colors.RED}✗{Colors.NC} Poor throughput ({result.requests_per_second:.1f} req/s)")
        
        # 99th percentile assessment
        p99_ms = result.percentile_99 * 1000
        if p99_ms <= 1000:
            print(f"{Colors.GREEN}✓{Colors.NC} Good 99th percentile ({p99_ms:.1f}ms)")
        else:
            print(f"{Colors.RED}✗{Colors.NC} High 99th percentile ({p99_ms:.1f}ms)")

async def main():
    parser = argparse.ArgumentParser(description="Solana PDA Analyzer Performance Tests")
    parser.add_argument("--url", default="http://localhost:8080", help="Base URL for API server")
    parser.add_argument("--quick", action="store_true", help="Run quick performance tests")
    parser.add_argument("--sustained", type=int, default=0, help="Run sustained load test for N seconds")
    parser.add_argument("--users", type=int, default=50, help="Number of concurrent users")
    parser.add_argument("--requests", type=int, default=10, help="Requests per user")
    
    args = parser.parse_args()
    
    # Check server availability
    try:
        async with aiohttp.ClientSession() as session:
            async with session.get(f"{args.url}/health") as response:
                if response.status != 200:
                    print(f"{Colors.RED}Server not accessible at {args.url}{Colors.NC}")
                    sys.exit(1)
    except Exception as e:
        print(f"{Colors.RED}Cannot connect to server at {args.url}: {e}{Colors.NC}")
        sys.exit(1)
    
    async with PerformanceTester(args.url) as tester:
        print(f"{Colors.BLUE}Starting Performance Tests for Solana PDA Analyzer{Colors.NC}")
        print(f"Target URL: {args.url}")
        print(f"Concurrent Users: {args.users}")
        print(f"Requests per User: {args.requests}")
        
        results = []
        
        if args.quick:
            # Quick performance tests
            tester.log_info("Running quick performance tests...")
            
            # Health endpoint
            result = await tester.test_health_endpoint_load(20, 5)
            tester.print_result(result)
            results.append(result)
            
            # PDA analysis
            result = await tester.test_pda_analysis_load(10, 3)
            tester.print_result(result)
            results.append(result)
            
        elif args.sustained > 0:
            # Sustained load test
            result = await tester.test_sustained_load(args.sustained)
            tester.print_result(result)
            results.append(result)
            
        else:
            # Full performance test suite
            
            # 1. Health endpoint load test
            tester.log_info("Testing health endpoint under load...")
            result = await tester.test_health_endpoint_load(args.users, args.requests)
            tester.print_result(result)
            results.append(result)
            
            # 2. PDA analysis load test
            tester.log_info("Testing PDA analysis under load...")
            result = await tester.test_pda_analysis_load(min(args.users, 20), min(args.requests, 5))
            tester.print_result(result)
            results.append(result)
            
            # 3. Batch analysis load test
            tester.log_info("Testing batch analysis under load...")
            result = await tester.test_batch_analysis_load(min(args.users, 10), min(args.requests, 3))
            tester.print_result(result)
            results.append(result)
            
            # 4. Database queries load test
            tester.log_info("Testing database queries under load...")
            result = await tester.test_database_queries_load(min(args.users, 30), args.requests)
            tester.print_result(result)
            results.append(result)
            
            # 5. List endpoints load test
            tester.log_info("Testing list endpoints under load...")
            result = await tester.test_list_endpoints_load(min(args.users, 25), args.requests)
            tester.print_result(result)
            results.append(result)
            
            # 6. Memory usage test
            tester.log_info("Testing memory usage under load...")
            result = await tester.test_memory_usage_under_load()
            tester.print_result(result)
            results.append(result)
        
        # Overall summary
        print(f"\n{Colors.BLUE}{'='*60}{Colors.NC}")
        print(f"{Colors.BLUE}OVERALL PERFORMANCE SUMMARY{Colors.NC}")
        print(f"{Colors.BLUE}{'='*60}{Colors.NC}")
        
        total_requests = sum(r.total_requests for r in results)
        total_successful = sum(r.successful_requests for r in results)
        overall_success_rate = (total_successful / total_requests * 100) if total_requests > 0 else 0
        
        print(f"Total Tests:         {len(results)}")
        print(f"Total Requests:      {total_requests}")
        print(f"Total Successful:    {total_successful}")
        print(f"Overall Success Rate: {overall_success_rate:.1f}%")
        
        avg_rps = statistics.mean([r.requests_per_second for r in results])
        avg_response_time = statistics.mean([r.avg_response_time for r in results]) * 1000
        
        print(f"Average RPS:         {avg_rps:.2f}")
        print(f"Average Response:    {avg_response_time:.2f}ms")
        
        if overall_success_rate >= 95 and avg_response_time <= 500:
            print(f"{Colors.GREEN}✓ Overall performance is GOOD{Colors.NC}")
            sys.exit(0)
        else:
            print(f"{Colors.YELLOW}⚠ Overall performance needs IMPROVEMENT{Colors.NC}")
            sys.exit(1)

if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print(f"\n{Colors.YELLOW}Performance tests interrupted by user{Colors.NC}")
        sys.exit(130)