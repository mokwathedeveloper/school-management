# School Management System — Soroban Smart Contract

A Soroban smart contract on the Stellar network for managing student registrations,
class assignments, and fee payments. Built with the Soroban SDK for the Stellar testnet.

---

## Table of Contents

- [Overview](#overview)
- [Contract Functions](#contract-functions)
- [Storage Design](#storage-design)
- [Events](#events)
- [Error Codes](#error-codes)
- [Project Structure](#project-structure)
- [Build](#build)
- [Test](#test)
- [Deploy to Testnet](#deploy-to-testnet)
- [Deployed Contract](#deployed-contract)

---

## Overview

The School Management System tracks students across three class levels — **Grade**,
**HighSchool**, and **College**. It supports:

- Student registration with a wallet address
- School fee payments using a Stellar asset (e.g. USDC)
- Admin-controlled class updates and student removal
- Full payment history per student preserved even after removal

---

## Contract Functions

### Constructor

Initialises the contract once at deployment.

```rust
pub fn __constructor(env: &Env, admin: Address, token: Address)
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `admin` | `Address` | Admin wallet — controls class updates and student removal |
| `token` | `Address` | Stellar asset contract used for fee payments |

---

### Register Student

Registers a new student and returns their assigned `student_id`.

```rust
pub fn register_student(env: &Env, student_wallet: Address, name: String, class_name: Class) -> u64
```

- Requires auth from `student_wallet`.
- Auto-increments `student_id` starting from 1.
- Initialises an empty payment history.

---

### Get Student

Returns full details of a student.

```rust
pub fn get_student(env: &Env, student_id: u64) -> StudentDetails
```

---

### Make Payment

Transfers school fees from the student to the admin using the configured token.

```rust
pub fn make_payment(env: &Env, student_id: u64, amount: i128) -> Result<(), ContractError>
```

- Requires auth from the student's wallet.
- Records each payment and updates `total_paid`.
- Emits a `PaymentEvent`.

---

### Update Student Class *(Assignment requirement)*

Changes a student's class level. Only the admin may call this.

```rust
pub fn update_student_class(env: &Env, student_id: u64, new_class: Class) -> Result<(), ContractError>
```

- Returns `StudentNotFound` if the ID does not exist.
- Returns `StudentNotRegistered` if the student has been removed.
- Emits a `ClassUpdateEvent`.

---

### Get Payment History *(Assignment requirement)*

Returns the full list of payments made by a student.

```rust
pub fn get_payment_history(env: &Env, student_id: u64) -> Vec<Payment>
```

- Returns an empty list if no payments have been made.
- Each `Payment` contains `student_id`, `amount`, and `timestamp`.
- History is preserved even after the student is removed.

---

### Remove Student *(Assignment requirement)*

Marks a student as no longer registered. Only the admin may call this.

```rust
pub fn remove_student(env: &Env, student_id: u64) -> Result<(), ContractError>
```

- Returns `StudentNotFound` if the ID does not exist.
- Returns `StudentNotRegistered` if already removed (prevents double removal).
- Sets `is_registered = false` — payment history is kept for auditing.
- Emits a `StudentRemovedEvent`.

---

## Storage Design

| Key | Type | Description |
|-----|------|-------------|
| `DataKey::Admin` | `Address` | Admin wallet |
| `DataKey::Token` | `Address` | Payment token contract |
| `DataKey::StudentCount` | `u64` | Auto-incrementing ID counter |
| `DataKey::Student(id)` | `StudentDetails` | Full student record |
| `DataKey::StudentPayments(id)` | `Vec<Payment>` | Payment history per student |

### StudentDetails

```rust
pub struct StudentDetails {
    pub student_id: u64,
    pub name: String,
    pub wallet_address: Address,
    pub class_name: Class,     // Grade | HighSchool | College
    pub total_paid: i128,
    pub is_registered: bool,
}
```

### Payment

```rust
pub struct Payment {
    pub student_id: u64,
    pub amount: i128,
    pub timestamp: u64,
}
```

---

## Events

| Event | Topics | Data |
|-------|--------|------|
| `PaymentEvent` | `[wallet_address]` | `student_id`, `amount` |
| `ClassUpdateEvent` | `[wallet_address]` | `student_id` |
| `StudentRemovedEvent` | `[admin]` | `student_id` |

---

## Error Codes

| Error | Value | Meaning |
|-------|-------|---------|
| `InsufficientFunds` | 1 | Payment amount is zero or negative |
| `StudentNotFound` | 2 | No student with the given ID |
| `NotAdmin` | 3 | Caller is not the admin |
| `StudentNotRegistered` | 4 | Student has already been removed |

---

## Project Structure

```
schoool-management/
├── Cargo.toml
├── contracts/
│   └── school-management/
│       ├── Cargo.toml
│       ├── Makefile
│       └── src/
│           ├── lib.rs                  # Module declarations
│           ├── school_management.rs    # Contract implementation
│           ├── storage.rs              # Storage types
│           ├── events.rs               # On-chain events
│           ├── error.rs                # Error codes
│           └── test.rs                 # 12 tests
└── README.md
```

---

## Build

```bash
stellar contract build
```

WASM output: `target/wasm32v1-none/release/school_management.wasm`

---

## Test

```bash
cargo test
```

### Test Coverage

| Test | Function Tested |
|------|----------------|
| `test_register_student` | `register_student` |
| `test_get_student` | `get_student` |
| `test_make_payment` | `make_payment` |
| `test_update_student_class` | `update_student_class` happy path |
| `test_update_student_class_to_college` | `update_student_class` Grade → College |
| `test_update_class_fails_for_removed_student` | `update_student_class` error path |
| `test_get_payment_history_empty_on_registration` | `get_payment_history` empty state |
| `test_get_payment_history_records_payments` | `get_payment_history` multiple payments |
| `test_payment_history_total_matches_student_total_paid` | `get_payment_history` invariant |
| `test_remove_student_marks_as_not_registered` | `remove_student` happy path |
| `test_remove_student_preserves_payment_history` | `remove_student` history preserved |
| `test_remove_student_twice_fails` | `remove_student` error path |

---

## Deploy to Testnet

### 1. Build

```bash
make build
```

### 2. Fund a testnet account

```bash
stellar keys generate alice --network testnet --fund
```

### 3. Deploy

```bash
stellar contract deploy \
  --wasm target/wasm32v1-none/release/school_management.wasm \
  --network testnet \
  --source alice \
  -- \
  --admin $(stellar keys address alice) \
  --token <TOKEN_CONTRACT_ID>
```

### 4. Example invocations

**Register a student:**
```bash
stellar contract invoke --id <CONTRACT_ID> --network testnet --source alice \
  -- register_student --student_wallet <WALLET> --name "Alice" --class_name College
```

**Update student class:**
```bash
stellar contract invoke --id <CONTRACT_ID> --network testnet --source alice \
  -- update_student_class --student_id 1 --new_class HighSchool
```

**Get payment history:**
```bash
stellar contract invoke --id <CONTRACT_ID> --network testnet --source alice \
  -- get_payment_history --student_id 1
```

**Remove student:**
```bash
stellar contract invoke --id <CONTRACT_ID> --network testnet --source alice \
  -- remove_student --student_id 1
```

---

## Deployed Contract

| Network | Contract ID |
|---------|-------------|
| Testnet | `CBOCON3D72JX25XQUP3R2KB7X6BUVGITHOX2GLAWPISKSFIHPMGJ2CCY` |

---

## Assignment Reference

**Stellar Impact Bootcamp — Week 3, Day 1**

- [x] Implement `update_student_class`
- [x] Implement `get_payment_history`
- [x] Implement `remove_student`
- [x] Test all implemented functions (12 tests)
- [x] Deploy to Stellar testnet
