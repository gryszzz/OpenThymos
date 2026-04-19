---
layout: default
title: Secure Tool Fabric
---

# Secure Tool Fabric

Slice 2 moves Thymos away from trusted in-process execution for risky tools.

## Current shape

- `shell` and `http` execute through a worker request/response contract
- `thymos-worker` provides a subprocess isolation boundary
- shell execution is now THYMOS-native rather than a generic terminal wrapper

## THYMOS secure shell

The shell now carries:

- `purpose`
- `capability_profile`
- confined working directory roots
- isolated `HOME` in restricted mode
- real timeout enforcement with process kill
- receipt-bearing observations with command digest and execution metadata

Profiles:

- `inspect`
- `build`
- `mutate`
- `networked`

## Hardening now enforced

- forbidden shell chaining sequences like `&&`, `||`, and `;`
- profile-based command gating
- path confinement under allowed roots
- private and loopback host blocking for the HTTP tool by default

## Next hardening steps

- container or VM-backed workers
- egress enforcement below the process layer
- signed worker attestation
- browser/code worker classes with per-capability sandboxes
