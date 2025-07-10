# Contributing to Solana PDA Analyzer

Thank you for your interest in contributing to the Solana PDA Analyzer! This document provides guidelines and information for contributors.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [How to Contribute](#how-to-contribute)
- [Pull Request Process](#pull-request-process)
- [Coding Standards](#coding-standards)
- [Testing Guidelines](#testing-guidelines)
- [Documentation](#documentation)
- [Issue Reporting](#issue-reporting)
- [Feature Requests](#feature-requests)

## Code of Conduct

This project adheres to a code of conduct that we expect all contributors to follow. Please be respectful and constructive in all interactions.

### Our Pledge

- Use welcoming and inclusive language
- Be respectful of differing viewpoints and experiences
- Accept constructive criticism gracefully
- Focus on what is best for the community
- Show empathy towards other community members

## Getting Started

### Prerequisites

- Rust 1.70 or later
- PostgreSQL 12 or later
- Git
- Basic knowledge of Solana blockchain and PDAs

### Development Setup

1. **Fork and Clone**
   ```bash
   git clone https://github.com/your-username/solana-pda-analyzer.git
   cd solana-pda-analyzer
   ```

2. **Install Dependencies**
   ```bash
   # Install Rust if you haven't already
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # Install PostgreSQL (macOS)
   brew install postgresql
   brew services start postgresql
   
   # Install PostgreSQL (Ubuntu)
   sudo apt-get install postgresql postgresql-contrib
   sudo systemctl start postgresql
   ```

3. **Setup Environment**
   ```bash
   cp .env.example .env
   # Edit .env with your database credentials
   ```

4. **Initialize Database**
   ```bash
   make db-init
   ```

5. **Build and Test**
   ```bash
   make build
   make test-all
   ```

6. **Run the Application**
   ```bash
   make run
   ```

## How to Contribute

### Types of Contributions

We welcome several types of contributions:

- **Bug fixes**
- **Feature implementations**
- **Documentation improvements**
- **Test additions**
- **Performance optimizations**
- **Security enhancements**
- **Example additions**

### Before You Start

1. **Check existing issues** to see if your idea is already being worked on
2. **Open an issue** to discuss major changes before implementing
3. **Read the codebase** to understand the current architecture
4. **Run the test suite** to ensure everything works

## Pull Request Process

### 1. Create a Feature Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/issue-number-description
```

### 2. Make Your Changes

- Follow the [coding standards](#coding-standards)
- Add tests for new functionality
- Update documentation as needed
- Ensure all tests pass

### 3. Commit Your Changes

Use clear, descriptive commit messages:

```bash
git commit -m "feat: add PDA pattern recognition for SPL tokens"
git commit -m "fix: resolve database connection pooling issue"
git commit -m "docs: update API documentation for batch analysis"
```

**Commit Message Format:**
- `feat:` for new features
- `fix:` for bug fixes
- `docs:` for documentation changes
- `test:` for test additions
- `refactor:` for code refactoring
- `perf:` for performance improvements
- `chore:` for maintenance tasks

### 4. Push and Create Pull Request

```bash
git push origin feature/your-feature-name
```

Create a pull request with:
- **Clear title** describing the change
- **Detailed description** of what was changed and why
- **Links to related issues**
- **Screenshots** if applicable
- **Breaking changes** noted clearly

### 5. Code Review Process

- At least one maintainer will review your PR
- Address any feedback or requested changes
- Keep the PR updated with the latest main branch
- Be responsive to comments and questions

## Coding Standards

### Rust Code Style

- **Use `rustfmt`**: `cargo fmt` before committing
- **Use `clippy`**: `cargo clippy` and address warnings
- **Follow Rust naming conventions**:
  - `snake_case` for functions and variables
  - `PascalCase` for types and traits
  - `SCREAMING_SNAKE_CASE` for constants

### Code Organization

- **Modular design**: Keep modules focused and cohesive
- **Clear interfaces**: Well-defined public APIs
- **Error handling**: Use `Result` types and proper error propagation
- **Documentation**: Document public APIs with `///` comments

### Example Code Style

```rust
/// Analyzes a PDA to determine its seed derivation pattern.
/// 
/// # Arguments
/// * `address` - The PDA address to analyze
/// * `program_id` - The program that owns this PDA
/// 
/// # Returns
/// * `Ok(Some(PdaInfo))` if analysis succeeds and seeds are found
/// * `Ok(None)` if no pattern could be determined
/// * `Err(PdaAnalyzerError)` if analysis fails
pub async fn analyze_pda(
    &mut self,
    address: &Pubkey,
    program_id: &Pubkey,
) -> Result<Option<PdaInfo>> {
    // Implementation here
}
```

### Database Guidelines

- **Use migrations** for schema changes
- **Write efficient queries** with proper indexing
- **Handle transactions** appropriately
- **Test database operations** thoroughly

### API Guidelines

- **RESTful design** with consistent endpoints
- **Proper HTTP status codes**
- **JSON response format** consistency
- **Input validation** on all endpoints
- **Error handling** with meaningful messages

## Testing Guidelines

### Test Coverage Requirements

- **Unit tests** for all new functions
- **Integration tests** for database operations
- **API tests** for new endpoints
- **Performance tests** for critical paths

### Writing Tests

```rust
#[tokio::test]
async fn test_pda_analysis_success() {
    let mut analyzer = PdaAnalyzer::new();
    let program_id = Pubkey::from_str("Program_ID_HERE").unwrap();
    let pda_address = Pubkey::from_str("PDA_ADDRESS_HERE").unwrap();
    
    let result = analyzer.analyze_pda(&pda_address, &program_id).await;
    
    assert!(result.is_ok());
    let pda_info = result.unwrap();
    assert!(pda_info.is_some());
}
```

### Test Naming

- Use descriptive test names: `test_pda_analysis_with_invalid_address`
- Group related tests in modules
- Use setup/teardown functions for common test data

### Running Tests

```bash
# Run all tests
make test-all

# Run specific test categories
make test-unit
make test-integration
make test-api

# Run with coverage
make test-coverage
```

## Documentation

### Code Documentation

- **Public APIs**: Must have comprehensive documentation
- **Complex algorithms**: Explain the approach and rationale
- **Examples**: Include usage examples where helpful

### User Documentation

- **README updates**: For new features or setup changes
- **API documentation**: Keep OpenAPI specs current
- **Tutorials**: Add examples for new functionality

### Documentation Style

```rust
/// Brief description of what the function does.
/// 
/// More detailed explanation if needed, including:
/// - Important behavior notes
/// - Performance characteristics
/// - Error conditions
/// 
/// # Arguments
/// * `param1` - Description of first parameter
/// * `param2` - Description of second parameter
/// 
/// # Returns
/// Description of return value and possible variants
/// 
/// # Errors
/// When this function will return an error
/// 
/// # Examples
/// ```rust
/// let result = function_name(arg1, arg2)?;
/// assert_eq!(result.field, expected_value);
/// ```
pub fn function_name(param1: Type1, param2: Type2) -> Result<ReturnType> {
    // Implementation
}
```

## Issue Reporting

### Bug Reports

When reporting bugs, please include:

1. **Environment information**:
   - OS and version
   - Rust version
   - PostgreSQL version
   - Application version

2. **Steps to reproduce**:
   - Clear, numbered steps
   - Expected vs actual behavior
   - Error messages or logs

3. **Additional context**:
   - Screenshots if applicable
   - Configuration files (sanitized)
   - Related issues or PRs

### Bug Report Template

```markdown
**Describe the bug**
A clear description of what the bug is.

**To Reproduce**
Steps to reproduce the behavior:
1. Go to '...'
2. Click on '....'
3. Scroll down to '....'
4. See error

**Expected behavior**
What you expected to happen.

**Environment:**
- OS: [e.g. macOS 13.0]
- Rust version: [e.g. 1.75.0]
- PostgreSQL version: [e.g. 15.2]

**Additional context**
Any other context about the problem.
```

## Feature Requests

### Feature Request Guidelines

- **Clear use case**: Explain why this feature is needed
- **Detailed description**: What should the feature do?
- **Alternative solutions**: Have you considered other approaches?
- **Implementation ideas**: Any thoughts on how to implement it?

### Feature Request Template

```markdown
**Is your feature request related to a problem?**
A clear description of what the problem is.

**Describe the solution you'd like**
A clear description of what you want to happen.

**Describe alternatives you've considered**
Other solutions or features you've considered.

**Additional context**
Any other context, screenshots, or examples.
```

## Development Workflow

### Branch Naming

- `feature/description-of-feature`
- `fix/issue-number-short-description`
- `docs/what-docs-are-updated`
- `test/what-tests-are-added`

### Release Process

1. **Version bumping** follows semantic versioning
2. **Changelog updates** for each release
3. **Testing** on multiple environments
4. **Documentation** updates as needed

## Performance Considerations

- **Profile before optimizing**: Use `cargo bench` to measure
- **Memory usage**: Be mindful of large data structures
- **Database queries**: Optimize for common use cases
- **API response times**: Keep endpoints fast and responsive

## Security Guidelines

- **Input validation**: Sanitize all user inputs
- **SQL injection prevention**: Use parameterized queries
- **Error messages**: Don't leak sensitive information
- **Dependencies**: Keep dependencies updated

## Community

### Getting Help

- **GitHub Issues**: For bugs and feature requests
- **GitHub Discussions**: For questions and general discussion
- **Documentation**: Check existing docs first

### Communication

- Be respectful and professional
- Use clear, concise language
- Provide context and examples
- Be patient with responses

## Recognition

Contributors will be recognized in:
- **CONTRIBUTORS.md** file
- **Release notes** for significant contributions
- **README.md** acknowledgments section

## Questions?

If you have questions about contributing that aren't covered here:

1. Check existing documentation
2. Search closed issues and PRs
3. Open a new issue with the `question` label
4. Reach out to maintainers

Thank you for contributing to Solana PDA Analyzer! 🚀