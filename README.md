# ternary-thermodynamics

Statistical mechanics analogs for ternary agent systems.

This crate studies ternary agent systems (think Rock-Paper-Scissors dynamics) through the lens of **statistical mechanics** — entropy, temperature, phase transitions, and free energy.

## Why?

In game theory, agents in a Rock-Paper-Scissors system cycle through strategies based on fitness. Statistical mechanics gives us powerful tools to understand these dynamics:

- **Entropy** measures how "mixed" the strategies are
- **Temperature** measures how random vs. ordered the population is
- **Phase transitions** mark sudden shifts in behavior (like freezing/melting)
- **Free energy** quantifies the system's capacity for strategic adaptation

## Statistical Mechanics Analogies

| Physics | Agent System |
|---------|-------------|
| Microstates | Individual agent strategies |
| Energy levels | Negative fitness values |
| Temperature | Inverse selection pressure |
| Entropy | Strategy diversity |
| Partition function | Normalization of Boltzmann distribution |
| Helmholtz free energy | Balance between fitness and diversity |
| Gibbs free energy | Fitness-diversity under competitive pressure |
| Phase transition | Order ↔ disorder shift in population |
| Specific heat | Sensitivity of energy to temperature change |
| Boltzmann distribution | Fitness-weighted strategy probabilities |

## Core Components

### `SystemEntropy`
Shannon entropy of the ternary distribution:
```
S = -Σ pᵢ ln(pᵢ)
```
- Maximum entropy = ln(3) ≈ 1.099 (uniform distribution)
- Minimum entropy = 0 (deterministic state)

### `SystemTemperature`
"Temperature" of the agent system:
- **High T** → agents spread uniformly (random play)
- **Low T** → agents concentrate on best strategy (ordered)
- **T = 0** → fully deterministic (winner-takes-all)

### `BoltzmannDistribution`
Strategy probabilities weighted by fitness:
```
pᵢ = exp(β · fitnessᵢ) / Z
```
where β = 1/T and Z is the partition function.

### `PartitionFunction`
The partition function Z normalizes the Boltzmann distribution:
```
Z = Σ exp(β · fitnessᵢ)
```

### `FreeEnergy`
- **Helmholtz**: F = -T · ln(Z) = ⟨E⟩ - T·S
- **Gibbs**: G = F + p·V (with competitive pressure as "pressure")

Lower free energy → more stable configuration.

### `PhaseTransitionDetector`
Detects order-disorder transitions by tracking the order parameter across temperature sweeps. Peaks in the specific heat analog indicate phase transitions.

## Example

```rust
use ternary_thermodynamics::*;

let fitnesses = [3.0, 1.0, 2.0];

// Compute Boltzmann distribution at T=1.0
let dist = BoltzmannDistribution::compute(fitnesses, 1.0).unwrap();
println!("P(Rock)={}, P(Paper)={}, P(Scissors)={}", 
    dist.p_rock, dist.p_paper, dist.p_scissors);

// Entropy
let entropy = SystemEntropy::entropy(&dist);
let normalized = SystemEntropy::normalized_entropy(&dist);
println!("Entropy: {:.4} (normalized: {:.4})", entropy, normalized);

// Free energy
let f = FreeEnergy::helmholtz(fitnesses, 1.0);
println!("Helmholtz free energy: {:.4}", f);

// Phase transition detection
let temps: Vec<f64> = (1..100).map(|i| i as f64 * 0.05).collect();
let detector = PhaseTransitionDetector::new(0.1);
let transitions = detector.detect_transitions(fitnesses, &temps);
for t in &transitions {
    println!("Transition at T={:.3}: {:?}", t.temperature, t.transition_type);
}
```

## Features

- **Pure Rust** — no unsafe code, no external dependencies, `#![no_std]` compatible
- **Well-tested** — 25+ unit tests
- **Documented** — full rustdoc with statistical mechanics context

## License

MIT
