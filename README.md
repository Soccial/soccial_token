# Soccial Token (SCTK) - Smart Contract

## 🧱 Overview

The **Soccial Token (SCTK)** is a utility token built on the **Solana blockchain**, serving as the economic foundation of the [Soccial](https://www.soccial.com/thetoken) social network.

It enables users to:

- Purchase services within the Soccial platform.
- Buy/sell Collaboration Cards (gamification).
- Participate in governance through on-chain voting.
- Earn rewards by staking or contributing to the ecosystem.
- Engage in a hybrid decentralized economy.
- Over time, the goal is to make the token a practical payment method not only within Soccial, but also for third-party products and services that align with the platform's ecosystem.

Soccial Token (SCTK) has a fixed supply of 500 million tokens, which means its value has the potential to grow hand in hand with the platform’s expansion — especially as the volume of transactions and business activity within Soccial increases.


## 🚀 Soccial Token Presale is Live!

The **Soccial Token (SCTK)** is currently in **presale** on Pinksale:

🔗 [Join the Presale Now](https://www.pinksale.finance/solana/launchpad/GtEFV77icPFn25ymroNnR9yRJhWL5AtZfevhbV1WZAvy)

Don't miss your chance to be part of the ecosystem from the very beginning.
- 20% discount on token price during presale.
- Guaranteed whitelist access for the private and public sale.
- Bonus governance credits to boost early participation.

If you believe in this vision and want to help the project take its next big steps, this is the moment to contribute. By participating in the presale, you’re not just supporting a new kind of social and economic platform - you're also making an investment that, together, we can work to make truly valuable in the future.

---

### 💡 A Project Born from Years of Work and a lot Passion

The **Soccial platform** - a new kind of social network - was built over the course of many years, completely from scratch, with no investors, no funding, and no team. Just one developer and one other invaluable founder (Isabel Pinto), both driven by an idea and a deep desire to create something different.

The Soccial Token (SCTK) was developed later,  to serve as the economic engine of that same network - and to open up a fair, community-driven economy around it.

This project isn’t powered by hype - it’s powered by work, persistence, resilience and the belief that something meaningful for the community can be built without permission, without millions - it's not just a product, but a foundation for true community-owned value to emerge.

We don’t have marketing funds, so reaching people is hard. But if you’ve found us now, you’re not just early - you’re part of how this story begins.

This isn’t about hype. We’re not buying bots or chasing empty numbers. We’re choosing the long road - one built on real belief, real people, and real progress.

**Thank you for being part of it.**


---

### 🤝 Open Source

This smart contract powers the core SCTK logic and is fully open-source.

It was developed by **Paulo Rodrigues**, co-founder of Soccial, with the goal of promoting transparency, trust, and community collaboration.

I believe open source is about sharing, collaborating, and building something greater together. Feel free to fork it, adapt it, and integrate it into your own project - just make it count — build something that matters.

> 💡 **For developers**
> 
> The code is made publicly available for full transparency on Soccial Token and to encourage developers to learn from it, reuse it, help improve it and share their ideas.
>
> Even if you don’t plan to use the full contract or its modules, I strongly recommend checking out the test helpers - they can save you months of work when building and validating your own Solana programs.
> 
>This contract wasn't designed to be used directly as-is, but with a few copy-pastes and minor adjustments, it can easily turn 3–4 months of development into just 2–3 days of integration.

 After years developing [Soccial](https://www.soccial.com) – the social network behind the Soccial Token
, I was never able to open-source any of the previous systems, even though that was always something I dreamed of doing.

So when I began building the Soccial Token contract, I decided from day one that it had to be fully open source. This project is my personal contribution to the Web3 community and the Solana ecosystem - a way of giving back and empowering others to build faster, smarter, and more transparently.

> ⚠️ This contract is provided **"as is"**, without any warranty of functionality, security, or suitability for any purpose. Use at your own risk.



## 🧩 Main Features
- **Initialize:** Initializes the token and allocates supply into contract-controlled vaults.
- **Tokenomics:** Controlled supply and automated distribution only through vaults.
- **Governance:** On-chain proposal and voting system for protocol-level decisions.
- **Airdrop Management:** Vault-controlled airdrops with cooldown and limited logic and eligibility checks.
- **Staking:** Stake SCTK to earn rewards through customizable plans.
- **Vesting:** Time-based vesting for team, advisors, and early contributors.
- **Vault System:** Segregated, contract-controlled vaults for airdrop, insurance, liquidity, off-chain reserve, reserved supply, revenue, rewards, staking, treasury, and vesting. Each vault operates with its own set of permissions and access rules. Some vaults require governance approval before they can be used.
- **Offchain market system** Token marketplace designed for use within the Soccial platform. Supports token purchases, deposits from off-chain Soccial vaults (with fees), and direct acquisitions using staked tokens.
- **Rewards Vault**  A special vault that can only be filled through contract-controlled fee revenue. It’s used to reward users in the Soccial Cards game, where players can unpack rare cards with a chance to win between 1% and 100% of the vault total balance. Users must choose how long they’re willing to wait before someone else has a chance to claim it - adding strategy, risk, and excitement to the reward system.
- **Fees System** A fully configurable fee system managed by the smart contract and governance controlled.
- **Access Control:** Role-based permissions (owner, admin, staff, user, early adopter) with secure enforcement.
- **Emergency Stop:** System pause and rollback capabilities for critical incident handling.
- **Event Logging:** Uses `emit!` and `msg!` to log all major on-chain actions, ensuring full traceability, transparency, and ease of debugging and auditing.
- **Centralized Permission Logic:** All permission checks are defined in `lib.rs` for clarity and ease of review. Only the `vaults` module


## 🧠 Logic and Security Guide
### Architecture of the Contract
The contract is fully modular, with each feature implemented in an isolated module. This ensures clean separation of concerns, extensibility, and testability.
- **lib.rs:** Main entry point, linking all modules.
- **airdrop.rs:** Manages controlled airdrops with cooldown periods.  
- **auth.rs:** Handles role-based access control (owner, admin, staff, user).  
- **economics.rs:** Controls token distribution logic and vault-based economics.  
- **governance.rs:** Handles voting and proposal creation.  
- **market.rs:** Manages the internal off-chain token market within the Soccial platform.  
- **staking.rs:** Allows users to stake tokens and earn rewards.  
- **token.rs:** Core logic related to supply and token state.  
- **utils.rs:** Provides shared helper functions, math utilities, and macros.  
- **vaults.rs:** Handles all contract-controlled vaults, each with specific permissions and usage (e.g. rewards, treasury, liquidity).  
- **vesting.rs:** Manages token vesting for team, investors, and marketing partnerships.

### Security Measures
1. **Access Control:** Only the contract owner or authorized addresses can execute critical functions.
2. **Rate Limiting:** Airdrop functions include maximum usage limits and cooldown periods to prevent abuse and ensure fair distribution.
3. **Governance Participation:** Community voting ensures transparency in major decisions.
4. **Data Integrity:** All state changes are recorded on-chain and are verifiable.
5. **Emergency Stop:** Allows halting of all critical operations to prevent further damage during an attack.
6. **Audit Trails:** The contract automatically logs andall actions for transparency.
7. **API-Gated Execution:** Most instructions must be triggered via Soccial's off-chain API, with only user-owned token recovery functions exposed publicly. This reduces the attack surface and helps mitigate DDoS and similar abuse attempts by routing logic through an adaptive off-chain gateway.


### Community Governance
The contract uses a decentralized governance model where token holders can vote on important changes. Founders are excluded from voting on financial decisions to maintain fairness.
- **Quorum:** A minimum percentage of the circulating supply must participate for a vote to be valid. Initially set to 0% to allow early adoption, with plans to increase it as the community grows.
- **Voting Period:** Votes are open for a fixed period of 7 days (configurable).

### Audit and Verification
- **Automated Reporting:** The contract generates reports after critical actions.
- **On-Chain Data:** Key metrics will be available on Soccial platform for public verification.
- **Code Audits:** Open source to allow third-party reviews.

## 🧪 Running Tests

We provide both **Rust-based** and **TypeScript-based** test suites to ensure contract reliability and correctness.

### ⚙️ Rust Tests

To run **all** Rust integration tests:

```bash
yarn test
```

To run a specific Rust test file:

```bash
yarn test add_admin         # Will run tests in `test_add_admin.rs`
```

To run a specific function inside a test file:

```bash
yarn test init_vesting twice_should_fail
```

To enable logging for a specific module:

```bash
yarn test vote governance_fails_without_tokens --log soccial_token=debug
```

> ℹ️ All Rust tests run via `cargo test-sbf` and are located under `programs/soccial_token/tests/`.

---

### 🧪 TypeScript Tests

To run **all** TypeScript tests on the default localnet:

```bash
yarn ts
```

To run a specific TypeScript test file:

```bash
yarn ts init_spl_token
```

To run tests on a different Solana cluster (e.g. devnet, testnet, mainnet):

```bash
yarn ts devnet
```

To run a specific test file on a specific network:

```bash
yarn ts add_admin testnet
```

> 📁 TypeScript tests are located in the `tests_ts/` folder and use the `tsx` test runner.

### 🧪 Tests Utils
To list available tests:

```bash
yarn testlist        # Rust
yarn tslist          # TypeScript
```

To run a selected test (interactive):

```bash
yarn testselect      # Rust
yarn tsselect        # TypeScript
```

### 🧪 Tests Configure
To generate network test configs (will use .env configured vars):

```bash
yarn tests:configure
```

Regenerate the test index to ensure test code is excluded from the final Anchor binary:

```bash
yarn tests:toml
```

To run a local test validator (localnet):

```bash
yarn testserver
```

## 🚀 Deployment

To deploy the contract:

```bash
yarn deploy
```

To text the deploy environment:

```bash
yarn deploytest
```


## 📦 Build & Analysis

To build the contract:

```bash
yarn build
```

To check the binary size:

```bash
yarn size
```

For detailed binary breakdown:

```bash
yarn size:detailed
```

To analyze crate size using `cargo bloat`:

```bash
yarn analyze
```

For detailed `.so` binary section analysis:

```bash
yarn analyze:detailed
```


## 🧹 Linting

To check code formatting:

```bash
yarn lint
```

To auto-fix formatting issues:

```bash
yarn lint:fix
```


## Project Structure
```
programs/soccial_token/
├── src/
│   ├── airdrop/                       – Controlled airdrops with cooldowns and eligibility checks
│   │   ├── airdrop.rs                – Airdrop core logic
│   │   ├── context.rs                – Context definitions for airdrop instructions
│   │   ├── error.rs                  – Module-specific error codes
│   │   └── mod.rs                    – Module entry point
│
│   ├── auth/                          – Role-based access control (admin, staff, owner)
│   │   ├── context.rs
│   │   ├── error.rs
│   │   ├── mod.rs
│   │   ├── permissions.rs
│   │   └── user.rs
│
│   ├── economics/                     – Token economics and automated fund distribution
│   │   ├── error.rs
│   │   ├── mod.rs
│   │   └── state.rs
│
│   ├── governance/                    – Proposals, voting, and decision logic
│   │   ├── approval.rs
│   │   ├── context.rs
│   │   ├── error.rs
│   │   ├── mod.rs
│   │   ├── proposal.rs
│   │   ├── proposal_type.rs
│   │   ├── state.rs
│   │   └── voting.rs
│
│   ├── initialize/                    – One-time bootstrap logic for setting up accounts and vaults
│   │   ├── context.rs
│   │   ├── error.rs
│   │   ├── initialize.rs
│   │   └── mod.rs
│
│   ├── market/                        – Marketplace and trading-related logic
│   │   ├── context.rs
│   │   ├── error.rs
│   │   ├── market.rs
│   │   └── mod.rs
│
│   ├── staking/                       – Staking system with rewards and lock periods
│   │   ├── claim.rs
│   │   ├── context.rs
│   │   ├── error.rs
│   │   ├── manage.rs
│   │   ├── mod.rs
│   │   ├── staking.rs
│   │   └── state.rs
│
│   ├── token/                         – Token minting, burning, and authority management
│   │   ├── context.rs
│   │   ├── error.rs
│   │   ├── metadata.rs
│   │   ├── mod.rs
│   │   └── state.rs
│
│   ├── utils/                         – Utility functions and shared helpers & macros
│   │   ├── error.rs
│   │   ├── macros.rs
│   │   ├── math.rs
│   │   ├── mod.rs
│   │   └── system.rs
│
│   ├── vaults/                        – Liquidity, treasury, and reward vaults
│   │   ├── context.rs
│   │   ├── error.rs
│   │   ├── mod.rs
│   │   └── vaults.rs
│
│   ├── vesting/                       – Team & investor vesting schedules with cliff and cycle rules
│   │   ├── context.rs
│   │   ├── error.rs
│   │   ├── mod.rs
│   │   ├── release.rs
│   │   ├── schedule.rs
│   │   └── state.rs
│
│   └── lib.rs                         – Main program entry point (Anchor #[program])

│
├── tests/
│   ├── testutils/                      – Reusable utilities, environment setup, and test macros
│   │   ├── basics.rs                   – Helpers for common PDA derivations and default values
│   │   ├── environment.rs              – Environment bootstrapping: wallet funding, airdrops, keypairs
│   │   ├── macros.rs                   – Custom macros for asserting results and simplifying test logic
│   │   └── mod.rs                      – Centralized exports for the test utilities module
│   │
│   ├── trymethods/                     – Lower-level tests for core logic, bypassing full integration
│   │   ├── tryairdrop.rs               – Direct calls to airdrop logic for isolated verification
│   │   ├── trygovernance.rs            – Direct invocation of governance instruction logic
│   │   ├── trymarket.rs                – Low-level tests for market interaction logic
│   │   ├── trystaking.rs               – Tests for core staking logic without full flow
│   │   ├── trysystem.rs                – Tests related to global system behaviour and logging
│   │   ├── trytoken.rs                 – Token module raw calls (mint, burn, authorities)
│   │   ├── tryuser.rs                  – Raw logic for user creation, permissioning, flags
│   │   ├── tryvaults.rs                – Vault logic execution without end-to-end context
│   │   ├── tryvesting.rs               – Vesting logic direct calls and edge validations
│   │   └── mod.rs                      – Entry point for trymethods module
│   │
│   ├── test_airdrop.rs                         – Tests for airdrop deposits, transfers, withdraws
│   ├── test_balance.rs                         – Balance verification and assertions
│   ├── test_governance_proposal_create.rs      – Proposal creation logic
│   ├── test_governance_proposal_finalize.rs    – Finalizing and executing proposals
│   ├── test_governance_update_settings.rs      – Governance config updates
│   ├── test_governance_vote.rs                 – Voting flow and validations
│   ├── test_initialize.rs                      – Basic system initialization
│   ├── test_initialize_economy.rs              – Initialization of economics and vaults
│   ├── test_initialize_spl_token.rs            – Token mint creation and authority setup
│   ├── test_list_contexts.rs                   – Context inspection helper
│   ├── test_market_buy.rs                      – Buy flow in market module
│   ├── test_market_deposit.rs                  – Depositing assets into market
│   ├── test_market_transfer.rs                 – Market token transfers
│   ├── test_permissions_api_authority.rs       – API-level permission logic
│   ├── test_staking_add.rs                     – Adding staking configs
│   ├── test_staking_buy.rs                     – Purchasing stake (if supported)
│   ├── test_staking_plan_add.rs                – Creating new staking plans
│   ├── test_staking_plan_deactivate.rs         – Deactivating plans
│   ├── test_staking_plan_edit.rs               – Editing plan parameters
│   ├── test_staking_rewards_claim.rs           – Reward claims from staking
│   ├── test_staking_stake.rs                   – Initial staking
│   ├── test_staking_stake_withdraw.rs          – Withdrawing staked tokens
│   ├── test_system_emit_log.rs                 – System-level log emission
│   ├── test_token_api_authority_set.rs         – Token authority changes
│   ├── test_token_pause.rs                     – Pausing token activity
│   ├── test_token_resume.rs                    – Resuming token activity
│   ├── test_token_update_airdrop_fee.rs        – Adjusting airdrop fee
│   ├── test_token_update_rewards_fee.rs        – Adjusting rewards fee
│   ├── test_user_admin_add.rs                  – Granting admin role
│   ├── test_user_admin_remove.rs               – Removing admin role
│   ├── test_user_early_adopter_add.rs          – Adding early adopter flag
│   ├── test_user_flag_add.rs                   – Flagging user
│   ├── test_user_flag_remove.rs                – Unflagging user
│   ├── test_user_permissions_assign.rs         – Assigning permissions
│   ├── test_user_permissions_unsign.rs         – Removing signature-based permissions
│   ├── test_vault_airdrop_deposit.rs           – Airdrop vault deposit
│   ├── test_vault_airdrop_transfer.rs          – Airdrop vault transfer
│   ├── test_vault_airdrop_withdraw.rs          – Airdrop vault withdrawal
│   ├── test_vault_insurance_deposit.rs         – Insurance vault deposit
│   ├── test_vault_insurance_transfer.rs        – Insurance vault transfer
│   ├── test_vault_insurance_withdraw.rs        – Insurance vault withdrawal
│   ├── test_vault_liquidity_deposit.rs         – Liquidity vault deposit
│   ├── test_vault_liquidity_transfer.rs        – Liquidity vault transfer
│   ├── test_vault_liquidity_withdraw.rs        – Liquidity vault withdrawal
│   ├── test_vault_offchain_reserve_deposit.rs  – Off-chain reserve deposit
│   ├── test_vault_offchain_reserve_transfer.rs – Off-chain reserve transfer
│   ├── test_vault_offchain_reserve_withdraw.rs – Off-chain reserve withdrawal
│   ├── test_vault_reserved_supply_deposit.rs   – Reserved supply vault deposit
│   ├── test_vault_reserved_supply_transfer.rs  – Reserved supply transfer
│   ├── test_vault_reservedsupply_withdraw.rs   – Reserved supply withdrawal
│   ├── test_vault_revenue_deposit.rs           – Revenue vault deposit
│   ├── test_vault_revenue_transfer.rs          – Revenue transfer
│   ├── test_vault_revenue_withdraw.rs          – Revenue withdrawal
│   ├── test_vault_rewards_deposit.rs           – Rewards vault deposit
│   ├── test_vault_rewards_transfer.rs          – Rewards transfer
│   ├── test_vault_rewards_withdraw.rs          – Rewards withdrawal
│   ├── test_vault_staking_deposit.rs           – Staking vault deposit
│   ├── test_vault_staking_transfer.rs          – Staking vault transfer
│   ├── test_vault_staking_withdraw.rs          – Staking vault withdrawal
│   ├── test_vault_treasury_transfer.rs         – Treasury transfer
│   ├── test_vault_treasury_withdraw.rs         – Treasury withdrawal
│   ├── test_vault_vesting_deposit.rs           – Vesting vault deposit
│   ├── test_vault_vesting_transfer.rs          – Vesting vault transfer
│   ├── test_vault_vesting_withdraw.rs          – Vesting vault withdrawal
│   ├── test_vesting_schedule_cancel.rs         – Cancelling vesting schedules
│   ├── test_vesting_schedule_create.rs         – Creating new vesting schedules
│   ├── test_vesting_schedule_set_immutable.rs  – Marking a schedule as immutable
│   ├── test_vesting_schedule_update.rs         – Updating vesting schedules
│   └── test_vesting_vested_claim.rs            – Claiming vested tokens
│
├── Cargo.toml     – Rust project configuration (dependencies, metadata, build settings)
├── Xargo.toml     – Configuration for building custom standard libraries with Xargo
│
scripts/
├── configure_tests.sh              – Generates test configuration files (e.g. accounts.json per network)
├── deploy.sh                       – Deploys the contract to the selected environment
├── deploy_test.sh                  – Deploys specifically for test purposes
├── generate_tests_toml.sh          – Builds toml index for test discovery/selection menus
├── test.sh                         – Runs full Rust test suite
├── test_ts.sh                      – Runs full TypeScript test suite
├── test_list.sh                    – Lists available Rust tests
├── test_select.sh                  – Interactive test selector (Rust)
├── test_ts_list.sh                 – Lists available TypeScript tests
├── test_ts_select.sh               – Interactive test selector (TypeScript)
├── update.sh                       – Updates dependencies or rebuilds the project cleanly
│
tests_ts/
├── accounts/                      – Builders for Anchor-compatible account setups
│   ├── airdrop.ts                 – Airdrop account initialization
│   ├── governance.ts              – Governance PDAs and data setup
│   ├── initialize.ts              – Full bootstrap account generation
│   ├── market.ts                  – Market-related PDAs
│   ├── staking.ts                 – Stake vaults, plans, and user data
│   ├── token.ts                   – TokenState, Mint authority and metadata accounts
│   ├── user.ts                    – UserAccount and permission flags
│   ├── vaults.ts                  – All system vaults (treasury, liquidity, etc.)
│   └── vesting.ts                 – Vesting account initialization
│
├── utils/                         – Shared test utilities and helpers
│   ├── env.ts                     – Anchor provider, program, wallet setup
│   ├── helpers.ts                 – General test helpers (e.g., delay, log, assert)
│   ├── math.ts                    – Fixed point math, precision tools
│   ├── serializers.ts             – Custom borsh/anchor-compatible serializers
│   └── test_vars.ts               – Reusable constants and PDA seeds
│ 
├── test_balance.ts                – Balance assertions across accounts
├── test_initialize.ts             – Full system setup verification
├── test_state.ts                  – State account assertions (TokenState, SystemControl, etc.)
├── test_vars.ts                   – Test Environment Test Variables
│
test_config/                      - [Not in the packet] Generate with "yarn tests:configure"
├── devnet.accounts.json          – Generated account list for Devnet
├── devnet.config.ts              – Devnet-specific test configuration
├── localnet.accounts.json        – Account list for Localnet
├── localnet.config.ts            – Localnet test setup (e.g. ledger paths, providers)
├── mainnet.accounts.json         – Accounts used in read-only or dry-run mainnet tests
├── mainnet.config.ts             – Optional settings for mainnet-safe testing
├── testnet.accounts.json         – Testnet account mapping
├── testnet.config.ts             – Testnet-specific configuration
│
├── .gitignore                – Ignore unnecessary files
├── Cargo.toml                - Rust project configuration
├── Cargo.lock                - Dependency lock file
└── README.md                 - Project documentation
```

## Next version
In the next version:
- Configurable Vesting on airdrops
- Command to make the mint immutable - and make it! It is controlled by a vault, but it is possible to mint with a non transparent contract upgrade.
- Command to make metadata immutable
- Remove initialize modules - The token is one, we don't need it anymore

## 📄 License

**License:** MIT  
**Author:** Paulo Rodrigues  
**Project:** Soccial Token (Soccial)

**More info:** [https://www.soccial.com/thetoken](https://www.soccial.com/thetoken)

**Whitepaper:** [https://d3gc3uk2xoluw7.cloudfront.net/download/soccialtoken_whitepaper_v1.pdf?v=1](https://www.soccial.com/thetoken)

---

## 📬 Contact

For support or questions, please contact the **Soccial team** via:
- [Telegram](https://t.me/+gPFV38cZi2RiNmJk)  
- [Soccial](https://www.soccial.com/)  
- [X (Twitter)](https://www.x.com/soccialtoken)


> ⚠️ As the solo developer behind Soccial, I’m not always able to offer full support to everyone using this code. However, we encourage you to use the community forum - we dream of seeing a space where developers collaborate, share, and help each other.
>
> I’ll do my best to help whenever I can.
