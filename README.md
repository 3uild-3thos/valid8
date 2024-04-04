valid8: Manage your Solana ledger locally

valid8 is a cli tool that simplifies managing Solana test ledgers for development and testing purposes. It allows you to clone accounts from the Solana blockchain and store them in a local ledger compatible with the solana-test-validator.

# Installation

(Assuming you have Rust and Cargo installed)

Clone the repository:

`git clone https://github.com/.../valid8.git`


Navigate to the project directory:
`cd valid8`

Build valid8:
`cargo build --release`

This will create a binary named valid8 in the target/release directory. You can copy the binary to a location in your PATH for easier access.

# Usage

Basic Usage:

valid8 [command]

## Available Commands:

    (no argument): Opens an interactive menu for managing accounts and programs.
    ledger (no argument): Generates a local ledger compatible with solana-test-validator.

## Interactive Menu:

When you run valid8 with no arguments, it will launch an interactive menu where you can choose from the following options:

    Clone Account: Clone an account from the Solana blockchain.
    Clone Program: Clone a program from the Solana blockchain.
    Edit Accounts: Manage accounts in your local ledger (future functionality).
    Edit Programs: Manage programs in your local ledger (future functionality).
    Exit: Exit the interactive menu.

Clone Account:

    Select "Clone Account" from the menu.
    Choose a network (mainnet or devnet) or select "Custom Network" to specify a custom RPC endpoint.
        If you choose "Custom Network," provide the RPC endpoint URL when prompted.
    Enter the public key of the account you want to clone.
    valid8 will clone the account and store it locally.

Ledger Command:

`valid8 ledger`

Generates a local ledger compatible with solana-test-validator. You can use this ledger with solana-test-validator to create a test environment and ledger with your cloned accounts and programs pre-loaded.
(use `-y` to automatically overwrite test-ledger if already exists)

# Example:

## Open the interactive menu
`valid8`

1. Select "Clone Account"
2. Choose "devnet" network
3. Enter the public key of the account to clone

## Generate a local ledger
`valid8 ledger -y`

## Start a testnet with solana-test-validator
`solana-test-validator` 

This will create a local ledger and run it with the cloned accounts available for testing.