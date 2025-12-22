# 🧠 Project Context: Rust GPU Sandbox Controller

## Purpose of This Document

This document defines the **baseline context** for an ongoing personal project.
It should be treated as **persistent background knowledge** when answering future questions.

The goal is to avoid re-explaining fundamentals and instead focus on **deep, specific, implementation-level guidance**.

---

## High-Level Project Goal

Design and implement a **Rust-based GPU job orchestration microservice** that models core ideas behind hyperscaler AI infrastructure, including:

* multi-tenant job submission
* GPU resource modeling and scheduling
* sandboxed execution
* mocked security attestation
* observability and metrics

The project is **educational but realistic**, designed to demonstrate system design thinking relevant to AI cloud platforms (e.g. hyperscalers).

---

## Scope & Non-Goals

### In Scope

* Job queueing and dispatch
* Virtual GPU resource management (simulated, MIG-inspired)
* Scheduling policies (FIFO, fair-share)
* WASM-based sandboxing (Wasmtime)
* Mocked attestation flow
* Rust async service architecture
* Observability (tracing + metrics)
* Optional small ML inference (CPU or lightweight GPU)

### Explicitly Out of Scope

* Full Kubernetes integration
* Real container runtimes
* Real confidential computing (SGX/SEV/TDX)
* Full CUDA driver management
* Distributed consensus (Raft, etcd)
* Production-scale fault tolerance

Mocking is acceptable **if the architecture matches real systems**.

---

## Target Architecture (Conceptual)

**Single-node async service** with the following components:

* **API Layer**

  * REST or gRPC (Axum or Tonic)
  * Job submission, status queries, health, metrics

* **Attestation Layer**

  * Validates job signatures (mocked)
  * Verifies model hashes
  * Produces structured attestation reports

* **Scheduler**

  * Pulls from job queue
  * Selects jobs based on scheduling policy
  * Coordinates with GPU resource manager

* **GPU Resource Manager**

  * Tracks virtual GPU slots
  * Enforces per-tenant quotas
  * Reserves/releases slots

* **Sandbox Executor**

  * Executes jobs in isolated WASM runtime
  * Enforces capability restrictions
  * Supports pluggable backends (WASM / CPU / optional CUDA)

* **Observability**

  * Tracing spans across job lifecycle
  * Metrics for queue length, utilization, latency

---

## Core Concepts Being Modeled

### 1. GPU Multi-Tenancy

* GPUs are represented as **virtual slots**, not physical devices
* Slots approximate MIG-style partitions
* Scheduling is policy-driven, not hardware-driven

### 2. Scheduling

* Jobs flow through a bounded queue
* Scheduler decides *which job runs next*
* Multiple policies supported:

  * FIFO
  * Per-tenant fair-share (round robin)

### 3. Isolation

* WASM is used as a **lightweight sandbox**
* Jobs cannot access host resources unless explicitly granted
* Isolation is **capability-based**, not syscall-based

### 4. Security & Attestation (Mocked)

* Jobs are “validated” before execution
* Validation includes:

  * tenant identity
  * model integrity
  * declared capabilities
* Attestation reports are produced for auditing

### 5. Observability

* Every job has a lifecycle trace
* Metrics are first-class
* Failures should be observable and explainable

---

## Job Lifecycle (Conceptual)

1. Client submits job
2. API validates request
3. Attestation layer verifies job
4. Job enters queue
5. Scheduler selects job
6. GPU slot reserved
7. Sandbox executes workload
8. Result returned
9. Slot released
10. Metrics + trace finalized

Failures can occur at any stage and must be handled cleanly.

---

## Expected Failure Modes

The system should explicitly handle:

* Queue overflow
* GPU slot exhaustion
* Job timeouts
* Sandbox execution failure
* Attestation failure
* Cancellation / shutdown during execution

Failure handling correctness is more important than performance.

---

## Implementation Constraints

* Language: **Rust**
* Async runtime: **Tokio**
* API: Axum or Tonic
* WASM runtime: Wasmtime
* Observability: tracing + Prometheus-style metrics
* State storage: in-memory (no DB required)
* Target runtime: single process, single node

Code should prioritize:

* clarity
* explicit state transitions
* correctness under concurrency

---

## Author Background Assumptions

When answering questions, assume:

* Strong background in:

  * GPU architecture
  * CUDA programming (C)
  * parallel computation concepts

* Weak or no background in:

  * networking
  * containerization
  * cloud schedulers
  * distributed systems theory
  * service observability

Therefore:

* Skip GPU fundamentals
* Explain *why* infra patterns exist
* Focus on architectural intent, not buzzwords

---

## Desired Answer Style for Future Questions

When responding to questions about this project:

* Be **precise and technical**
* Explain trade-offs explicitly
* Prefer concrete Rust patterns over abstract theory
* Assume the question refers to *this architecture*
* Avoid re-explaining the entire system unless asked
* Provide diagrams or pseudocode when helpful

---

## Ultimate Outcome

The finished project should be:

* defensible in a technical interview
* easy to explain at a system-design level
* small enough to reason about end-to-end
* realistic enough to map onto real hyperscaler components

This is **not a toy**, but also **not a production system** — it is a *compressed learning environment*.