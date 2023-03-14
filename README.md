# Specifications
The documentation can be found here: https://docs.leancoin.io/swap-lean/

# Getting Started
Leancoin requires some dependencies to be built and deployed.

Dependencies required to build and deploy Leancoin program:
- Rust 1.6 or higher
- Solana CLI 1.14.7
- Anchor 0.27.0

Dependencies required to run TypeScript tests:
- NodeJS 18.2.0 or higher
- Yarn 1.22.19

You can either install these dependencies in your OS manually or install Docker and use Dockerfile provided in this repository to start pre-configured container with all the dependencies. The second way is recommended because it's much simpler and less error-prone.

## Using Docker

### Prerequisites
The only prerequisite is Docker installed in your OS.

### Usage
Go to `scripts` directory and build Dockerfile from this OS.

#### Using script
The simplest way for an OS with Bash scripts support (Linux, MacOS oraz Windows with terminal where Bash is supported like WSL, Git Bash, mingw), the recommended way is to execute `./run-docker-container.sh` script which performs everything automatically.

The script can also automatically build and deploy Leancoin in the container so it will be immediately ready to use.

#### Manually
The alternative way is to build Docker image and start it manually. It's necessary for OS without Bash scripts support.

Build Docker image using the following command:
`docker build --build-arg ANCHOR_VERSION=0.27.0 --build-arg SOLANA_VERSION=1.14.7 -t leancoin:1.0 -f Dockerfile ../`

Start Docker container:
`docker run -d -p 8899:8899 --name leancoin leancoin:1.0 bash -c "sleep infinity"`

Open terminal in Docker's container:
`docker exec -it leancoin bash`

## Without Docker
Install dependencies mentioned above. Please check the links below to find out on how to install the dependencies:
- Rust: https://www.rust-lang.org/tools/install
- Solana CLI: https://docs.solana.com/cli/install-solana-cli-tools
- Anchor: https://www.anchor-lang.com/docs/installation
- NodeJS: https://nodejs.org/en/download/
- Yarn: https://yarnpkg.com/getting-started/install

# Commands
Execute below commands in the environment where you set up the dependencies mentioned above in Getting Started section - either in the Docker's container (if you used one) or directly in your OS if you deployed Leancoin there.

- Build Leancoin (compiles Rust code): `anchor build && cargo build`
- Install NPM dependencies (required to run tests in TypeScript): `yarn install`
- Start test Solana validator: `solana-test-validator`
- Deploy Leancoin (it's deployed to test Solana validator by default): `anchor deploy`
- Run tests in Rust for Leancoin: `cargo test`
- Run tests in TypeScript for Leancoin: `anchor test`

# Project Structure 
The project structure is based on the standard Anchor's template which is composed of contracts, tests, and deploy instructions. The template provides a great starting point for developers to quickly get up and running and deploying smart contracts on the Solana blockchain.

## Main contract files
Main contract files are placed in the `programs\LeanManagementToken` directory.

Besides `Cargo.toml` and `Xargo.toml` files used by Rust to build the code, there is also `src` directory with the actual source code of the contract. It contains the following files:
- `lib.rs` - the main contract file with exposed functions,
- `account.rs` - contains structures of accounts used in `lib.rs`,
- `context.rs` - contains structures of contexts used in `lib.rs`,
- `error.rs` - contains all errors used in `lib.rs` and `utils.rs`,
- `utils.rs` - contains helper structures and functions used in `lib.rs`.

The files include also Rust tests (more details in [Tests section](#tests) ).

```rust
crate leancoin
  ├── mod account
  ├── mod context
  ├── mod error
  ├── mod leancoin
  └── mod utils
```

## TypeScript Tests
TypeScript tests are placed in the `tests` directory. It contains the following files:
- `LeanCoin.ts` file - integration tests for the contract (more details in [Tests section](#tests) ),
- `utils` directory - helper functions used in `LeanCoin.ts` file.

There are also some files related to TypeScript tests placed at root level:
- `.env` file - contains environment variables used by the tests,
- `package.json` file - declares dependencies and scripts used by TypeScript to run tests,
- `tsconfig.json` file - contains compiler options used to compile TypeScript files with tests.

## Deployment scripts
Deployment script is placed in `scripts` directory. It is described more thoroughly in [Using script section](#using-script).

## Other files
There are also few other files like Prettier configuration or Anchor configuration files at the root level.

# Documentation
Execute the following command to generate code documentation: `cargo doc --open`

# Tests
There are two types of tests created to test Leancoin:
- Rust tests - tests placed in Rust files (`*.rs`) testing the code directly or indirectly:
  - Indirect tests are provided for the exposed contract functions, i.e. functions available in the `lib.rs` file. They are tested using [solana-program-test crate](https://docs.rs/solana-program-test/latest/solana_program_test/) so it looks more like integration testing process. It is much more valuable than direct tests for this part of contract as the code is tested from the contract client's perspective.
  - Direct tests are provided for everything else, e.g. utils. It means that the functions are directly invoked in the tests.
- Anchor client tests - tests written in TypeScript placed in `tests` directory. This is the recommended approach for testing code written in Anchor. It is integration testing performed using the client provided by Anchor. It simulates the usual way of invoking the deployed smart contract.

## Code coverage
Code coverage has been checked using [cargo-tarpaulin crate](https://crates.io/crates/cargo-tarpaulin), but it doesn't provide reliable results. While it detects the coverage for functions tested directly (by invoking them in the tests), it fails to detect coverage for functions tested indirectly, i.e. those invoked using the `solana-program-test` crate.

That's why we decided to use it as a hint, rather than something that defines how well the code is covered. Since we haven't found any working solutions to properly detect code coverage, or to test the code in a better way that would be detected by `cargo-tarpaulin`, we were forced to rely mainly on the code review process to determine the completeness of tests.

## Running tests
Use the following commands to run tests:
- Rust tests: `cargo test`
- TypeScript tests: `anchor test`
