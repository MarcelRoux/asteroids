# ADR-0000: Engine Choice — macroquad over bevy

## Status

Accepted

## Context

This project aims to showcase:

- explicit performance ownership,
- bounded-cost simulation,
- deterministic game logic,
- and intentional system architecture.

While Rust game engines like Bevy provide powerful abstractions (ECS, scheduling, plugins), they also introduce:

- significant framework surface area,
- longer compile times,
- larger binaries,
- and non-trivial default behavior that can obscure performance tradeoffs.

The project prioritizes *engineering visibility* over engine-provided convenience.

## Decision

Use **macroquad** as the rendering/input framework.

macroquad is used strictly as:

- windowing
- input
- drawing
- audio

All gameplay, simulation, AI, and performance behavior is implemented explicitly in project code.

## Rationale

### Why macroquad

- Fast 0→1 development (first playable quickly)
- Small dependency graph → faster iteration
- Minimal abstractions → performance is the developer’s responsibility
- Straightforward WebAssembly (WASM) support
- Small final binary size
- Encourages engine-agnostic architecture

### Why not bevy (for this project)

- Larger compile times and binary sizes
- ECS + plugin model can hide performance costs
- Framework conventions dominate early design decisions
- Less suitable for demonstrating low-level tradeoffs early

This is not a rejection of Bevy as a technology, but a conscious choice aligned with the goals of this project.

## Consequences

- The game loop, scheduling, and system ordering are explicitly defined.
- Performance characteristics are transparent and measurable.
- The architecture remains portable to other engines if desired.
