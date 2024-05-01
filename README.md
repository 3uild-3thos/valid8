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
    run : Opens the same interactive menu as no arguments
    ledger (arg: overwrite): Generates a local ledger compatible with solana-test-validator. 
        Overwrite directory if already exists with the `-y` option.
    compose: Compose multiple valid8 config files into one.

## Interactive Menu:

In the interactive menu you can choose from the following options:

    Clone Account: Clone an account from the Solana blockchain.
    Clone Program: Clone a program from the Solana blockchain.
    Edit Accounts: Manage accounts in your local ledger.
    Edit Programs: Manage programs in your local ledger.
    Compose configs: Compose multiple valid8 config files into one.
    Generate Ledger: Generates a local ledger compatible with solana-test-validator.
    Exit: Exit the interactive menu.

Clone Account:

    Select "Clone Account" from the menu.
    Choose a network (mainnet or devnet) or select "Custom Network" to specify a custom RPC endpoint.
        If you choose "Custom Network," provide the RPC endpoint URL when prompted.
    Enter the public key of the account you want to clone.
    valid8 will clone the account and store it locally.

Edit Account:

    Select "Edit Account" from the menu.
    Enter the public key of the account you want to edit when prompted.
    Change the owner, or the amount of lamports in the account.
    valid8 will edit the account and store it locally with the changed value.

Clone Program:

    Select "Clone Program" from the menu.
    Choose a network (mainnet or devnet) or select "Custom Network" to specify a custom RPC endpoint.
        If you choose "Custom Network," provide the RPC endpoint URL when prompted.
    Enter the public key of the program you want to clone, the program data account will automatically cloned.


Edit Program:

    Select "Edit Program" from the menu.
    Enter the public key of the program you want to edit when prompted.
    Change the owner, the amount of lamports, or the upgrade authority of the program, or select Unpack PDA to edit a program related pda account
    valid8 will edit the program and store it locally with the changed value(s).

Ledger Command:

`valid8 ledger`

Generates a local ledger compatible with solana-test-validator. 
You can use this ledger with solana-test-validator to create a test environment and ledger with your cloned accounts and programs pre-loaded.
(use `-y` to automatically overwrite test-ledger if already exists)

Compose Command:

`valid8 compose`

Composes multiple valid8 configs together, for an even bigger dev environment. 
To add an extra valid8 config and compose it with your own, just add a filename to the `compose: ` field in your `valid8.json` file.

# Example:

## Open the interactive menu
`valid8`

1. Select "Clone Account"
2. Choose "devnet" network
3. Enter the public key of the account to clone

## Generate a local ledger
`valid8 ledger -y`

## Start solana-test-validator
`solana-test-validator` 

This will create a local ledger and run it with the cloned accounts available for testing.