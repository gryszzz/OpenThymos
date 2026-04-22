# Architecture

Thymos separates proposing work from executing work.

## Flow

`Cognition -> Intent -> Proposal -> Execution -> Result`

## What the runtime owns

- authority
- policy checks
- tool execution
- failure handling
- logging
- completion state

## Why that matters

Because the runtime owns execution truth, multiple interfaces can attach to the same run without diverging.

## Main pieces

- `thymos-cognition`
- `thymos-runtime`
- `thymos-tools`
- `thymos-policy`
- `thymos-ledger`
- `thymos-server`
- `thymos-cli`
