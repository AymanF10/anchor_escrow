## Overview
This project implements an escrow program on the Solana blockchain using the Anchor framework. The program allows users to create escrow accounts where they can deposit tokens securely, facilitating a two-party exchange. The program supports operations for making an escrow, taking from an escrow, and refunding deposits.

## Table of Contents
- [Installation](#installation)
- [Project Structure](#project-structure)
- [Program Logic](#program-logic)
  - [Making an Escrow](#making-an-escrow)
  - [Taking from an Escrow](#taking-from-an-escrow)
  - [Refunding an Escrow](#refunding-an-escrow)
- [Testing](#testing)
- [Usage](#usage)
- [Error Handling](#error-handling)
- [Contributing](#contributing)
- [License](#license)

## Installation
1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/solana-anchor-escrow.git
   cd solana-anchor-escrow
   ```

2. Install the required dependencies:
   ```bash
   anchor build
   ```

3. Ensure your Solana wallet is set up and funded on the devnet.

## Project Structure
The project consists of several key files:

| File                  | Description                                                   |
|-----------------------|---------------------------------------------------------------|
| `lib.rs`              | Contains the main logic of the escrow program, including methods for making, taking, and refunding escrows. |
| `state/escrow.rs`     | Defines the data structure for the escrow account.           |
| `instructions/make.rs`| Contains logic for creating an escrow and depositing tokens. |
| `instructions/take.rs`| Contains logic for taking tokens from an escrow.             |
| `instructions/refund.rs`| Handles refunding tokens back to the maker.                |
| `mod.rs`              | Module management for organizing related functionalities.    |

## Program Logic

### Making an Escrow
The `make` function allows a user to create a new escrow account and deposit tokens into it.

```rust
pub fn make(ctx: Context<Make>, seed: u64, deposit: u64, receive: u64) -> Result<()> {
    ctx.accounts.deposit(deposit);
    ctx.accounts.init_escrow(seed, receive, &ctx.bumps)
}
```
#### Key Components:
- **Depositing Tokens**: The user deposits tokens into the escrow account.
- **Initializing Escrow**: Sets up the escrow account with necessary parameters.

### Taking from an Escrow
The `take` function allows a user to take tokens from an existing escrow account.

```rust
pub fn take(ctx: Context<Take>) -> Result<()> {
    ctx.accounts.deposit()?;
    ctx.accounts.withdraw()?;
    ctx.accounts.close()?;
    Ok(())
}
```
#### Key Components:
- **Depositing Tokens**: The taker deposits their tokens into the escrow.
- **Withdrawing Tokens**: The taker withdraws tokens from the vault.
- **Closing Escrow**: Finalizes and closes the escrow account.

### Refunding an Escrow
The `refund` function allows a user to refund their tokens back from the escrow account.

```rust
pub fn refund(ctx: Context<Refund>) -> Result<()> {
    ctx.accounts.refund()?;
    ctx.accounts.close_refund()?;
    Ok(())
}
```
#### Key Components:
- **Refunding Tokens**: Transfers tokens back to the maker's associated token account.
- **Closing Escrow**: Closes the escrow account after refunding.

## Testing
To run tests for the escrow program, execute the following command:

```bash
anchor test
```
## Usage
1. **Deploy the Program**: Deploy your program to the Solana devnet using:
   ```bash
   anchor deploy --provider.cluster devnet
   ```

2. **Interact with the Program**: You can interact with your deployed program using client-side scripts or through a frontend application that connects to Solana.

3. **Example Commands**:
   - Making an Escrow:
     ```javascript
     const tx = await program.methods.make(seed, depositAmount, receiveAmount).rpc();
     console.log("Your transaction signature", tx);
     ```
   - Taking from an Escrow:
     ```javascript
     const tx = await program.methods.take().rpc();
     console.log("Take transaction signature", tx);
     ```
   - Refunding an Escrow:
     ```javascript
     const tx = await program.methods.refund().rpc();
     console.log("Refund transaction signature", tx);
     ```
## License
This project is licensed under the MIT License - see the LICENSE file for details.
