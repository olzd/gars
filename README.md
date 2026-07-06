# GARS - Genetic Algorithm Runtime System

A generic, deterministic, parallel genetic algorithm engine in Rust.

## Overview

Reusable framework for evolutionary optimization. It is
problem-agnostic: you define your genotype, evaluator, genetic
operators, and selection strategy, then the engine manages the entire evolution
loop.

Key properties:

- **Generic**: works with any genotype.
- **Deterministic**: given a master seed, a run is completely reproducible.
- **Parallel**: fitness evaluations are distributed across CPU cores via Rayon,
  using deterministic per-individual ChaCha8 RNG streams.
- **Observable**: register multiple observers to log progress, display live
  statistics, ...

The `examples/` directory contains implementation of:

- a cleaning robot
- a snake game

## Design

The framework provides the following core traits:

- `Genotype`: defines how to create random individuals
- `Evaluator`: computes the fitness of a genotype
- `Operator`: crossover and mutation of genotypes
- `Selector`: parents selection for reproduction
- `GenerationObserver`: called after each generation evaluation with statistics

The `Engine` is the only concrete implementation provided using those traits to drive the evolution loop which
basically consists of:

1. evaluating the current generation
2. computing statistics
3. notifying the registered observers
4. evolving the population to produce a new generation

# License

This project is available under the GPL 3.0 license. See the LICENSE file for details.