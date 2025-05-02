# WRT Reorganization Agent Prompt

You are an AI agent tasked with implementing the WRT Decoder Reorganization Plan as outlined in the `decoder.plan.md` file.

## Your Task

You will be implementing the reorganization plan for the WRT (WebAssembly Runtime) crates. The plan involves addressing duplication of code, improving organization structure, and ensuring both std and no_std compatibility across all crates.

## Process to Follow

1. **Read and understand the `decoder.plan.md` file** to get a complete understanding of the plan and desired outcomes.

2. **Work through each phase sequentially**, implementing one phase at a time:
   - Phase 1: Consolidate Component Files
   - Phase 2: Establish Core WebAssembly Module
   - Phase 3: Deduplicate Resource Operations
   - Phase 4: Update Main Library Exports
   - Phase 5: Ensure Consistent Feature Flags

3. **After implementing each phase**:
   - Build with std features: `cargo build --features std`
   - Build with no_std features: `cargo build --no-default-features --features no_std,alloc`
   - Run tests: `cargo test --features std`
   - Check for linting issues: `cargo clippy -- -D warnings`
   - Ensure documentation builds: `cargo doc --no-deps`

4. **Report on completion of each phase**:
   - Summarize the changes you made
   - Confirm that all validation criteria are met
   - Identify any issues encountered and how they were resolved

5. **After completing all phases**:
   - Perform a final comprehensive validation
   - Create a summary report of all changes made
   - Confirm all success metrics are met

## Validation Requirements

For each phase and at the end of the implementation:

1. No build errors or warnings with std and no_std features
2. All tests pass successfully
3. No clippy warnings
4. Documentation builds without warnings

## Final Deliverable

A comprehensive report including:
1. Summary of all changes made
2. Confirmation that all validation criteria are met
3. Any issues encountered and how they were resolved
4. Recommendations for future improvements

Once you've completed each phase and validated that it meets all criteria, mark it as complete and move on to the next phase. When all phases are successfully implemented and validated, you can declare the reorganization plan complete. 