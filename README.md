# Proof-of-Impact Funding Protocol (PIFP)

**Trust-minimized global funding. Money moves only when verifiable impact occurs.**

## Why PIFP?
Traditional donations rely on trust. PIFP uses **Stellar smart contracts** to lock funds until proof of impact is verified, replacing intermediaries with cryptographic accountability.

## How It Works
1.  **Project Created**: Creator sets funding goal and proof requirements.
2.  **Funded**: Donors deposit Bitcoin-backed assets into a Stellar pool (anonymously).
3.  **Proof**: Implementer submits proof (photos, data) to the Oracle.
4.  **Verified & Released**: Smart contract verifies proof hash and releases funds.

## Tech Stack
*   **Stellar/Soroban**: Smart contract logic for conditional release.
*   **Rust**: Backend oracle for proof hashing and verification.
*   **React**: Frontend for project creation and funding.

## Security
*   **Non-custodial**: Funds locked in contracts, not by us.
*   **Privacy**: Commitment schemes hide donor identity.
*   **Authentication**: Wallet signatures + OTP for critical actions.

## Impact
Zero corruption. 100% Transparency. Validated outcomes for charity and development.
