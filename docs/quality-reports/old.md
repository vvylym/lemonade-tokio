# Aprender Quality Report

Generated: Sun Dec  7 11:19:55 AM WAT 2025

## Rust Project Score
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🦀  Rust Project Score v2.1
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📌  Summary
  Score: 82.5/134
  Percentage: 61.6%
  Grade: B+

📂  Categories
  ⚠️ Code Quality: 20.0/26 (76.9%)
  ❌ Dependency Health: 5.0/12 (41.7%)
  ❌ Documentation: 0.0/15 (0.0%)
  ❌ Formal Verification: 3.0/13 (23.1%)
  ✅ Known Defects: 20.0/20 (100.0%)
  ❌ Performance & Benchmarking: 0.0/10 (0.0%)
  ❌ Rust Tooling & CI/CD: 32.0/130 (24.6%)
  ❌ Testing Excellence: 2.5/20 (12.5%)

💡  Recommendations
  • Run 'cargo clippy --fix' to automatically fix clippy warnings
  • Run 'cargo fmt' to format code according to Rust style guidelines
  • Run 'cargo audit' and update vulnerable dependencies
  • Add deny.toml configuration for dependency policy enforcement
  • Enable high-value lint categories (unsafe_op_in_unsafe_fn, unreachable_pub, checked_conversions) for better code quality
  • Create .clippy.toml with disallowed-methods to enforce project-specific style preferences
  • Improve test quality: install cargo-mutants and aim for ≥80% mutation score
  • Improve test coverage: Install cargo-llvm-cov and aim for ≥85% line coverage
  • Add integration tests: Create tests/ directory with end-to-end test files
  • Add doc tests: Include runnable examples in /// documentation comments
  • Improve test quality: Install cargo-mutants and aim for ≥80% mutation score
  • Improve rustdoc coverage: Add /// documentation to public API items with examples
  • Improve README: Add Installation, Usage, Examples, and License sections
  • Add CHANGELOG.md: Document version history and changes between releases
  • Add [[bench]] sections: Configure benchmark targets in Cargo.toml with Criterion
  • Add benchmark CI: Create .github/workflows with 'cargo bench' for automated performance testing
  • Use custom harness: Add 'harness = false' to [[bench]] sections for Criterion integration
  • Add feature flags: Use [features] to make dependencies optional and enable modular builds
  • Optimize dependency tree: Use optional dependencies and disable default features to reduce bloat

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Quality Gates
# Quality Gate Report

**Timestamp**: 2025-12-07T10:19:55.724341696+00:00

**Status**: ✅ PASS

## Gate Results

- ✓ **clippy** (0.11s)
  ✓ Clippy passed
- ✓ **tests** (0.12s)
  ✓ Tests passed
- ✓ **coverage** (0.27s)
  ✓ Coverage: 94.2% (>= 80.0%)
- ✓ **complexity** (0.00s)
  ✓ Complexity: All functions <10

**Total Time**: 0.49s


## TDG Score
╭─────────────────────────────────────────────────╮
│  TDG Score Report                              │
├─────────────────────────────────────────────────┤
│  Overall Score: 98.1/100 (A+)                  │
│  Language: Rust (confidence: 100%)             │
│                                                 │
│  📊 Breakdown:                                  │
│  ├─ Structural:     25.0/25                    │
│  ├─ Semantic:       20.0/20                    │
│  ├─ Duplication:    19.7/20                    │
│  ├─ Coupling:       15.0/15                    │
│  ├─ Documentation:  10.0/10                    │
│  └─ Consistency:    10.0/10                    │
╰─────────────────────────────────────────────────╯

