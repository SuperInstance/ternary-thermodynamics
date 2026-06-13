# Ternary Thermodynamics

Statistical mechanics for ternary agent systems — modeling entropy, temperature, Boltzmann distributions, partition functions, free energy, and phase transitions in populations where each agent adopts one of three strategies (Rock, Paper, Scissors).

## Why It Matters

Systems of agents competing on a ternary strategy space — whether GPU kernels competing for resources, traders in a market, or organisms in an ecosystem — exhibit deep parallels with physical thermodynamic systems. The mathematics of statistical mechanics (developed by Boltzmann, Gibbs, and Maxwell in the late 1800s) directly describe:

- **How disorder grows** (entropy) when agents randomize
- **How sensitivity to payoff differences** controls the distribution (temperature)
- **When abrupt transitions occur** (phase transitions) — e.g., all agents suddenly switch to one strategy
- **What configurations are stable** (free energy minimization)

This lets you borrow 150 years of physics intuition: "is my fleet hot or cold?", "what's the system's free energy?", "are we near a phase transition?"

## How It Works

### Shannon Entropy

For a ternary distribution $\mathbf{p} = (p_R, p_P, p_S)$ where $p_R + p_P + p_S = 1$:

$$S(\mathbf{p}) = -\sum_{i} p_i \ln p_i$$

| Distribution | $S$ (nats) | Meaning |
|---|---|---|
| $(1, 0, 0)$ — deterministic | $0$ | Fully ordered |
| $(1/3, 1/3, 1/3)$ — uniform | $\ln 3 \approx 1.099$ | Maximum disorder |

**Normalized entropy**: $\hat{S} = S / \ln 3 \in [0, 1]$

### Temperature

Temperature $T$ controls how agents distribute across strategies of different fitness. Using the **Boltzmann distribution**:

$$p_i = \frac{e^{E_i / T}}{Z}, \quad Z = \sum_j e^{E_j / T}$$

where $E_i$ is the fitness of strategy $i$ and $Z$ is the **partition function**.

| Temperature | Behavior |
|---|---|
| $T \to 0$ | Agents freeze onto the highest-fitness strategy |
| $T \to \infty$ | Agents spread uniformly (random play) |
| $T \sim 1$ | Intermediate — diversity with bias |

**Estimating $T$ from data**: Given an observed distribution and known fitnesses, infer $T$ via Boltzmann inversion:

$$T \approx \frac{E_j - E_i}{\ln p_j - \ln p_i}$$

### Partition Function and Free Energy

The **Helmholtz free energy**:

$$F = -T \ln Z$$

The **Gibbs free energy** (at fixed pressure):

$$G = F + pV$$

In the agent-system analogy, $F$ measures the "usable work" available — low $F$ means the system is near equilibrium and stable.

### Phase Transitions

A **phase transition** occurs when the system jumps between ordered and disordered states. For ternary systems, the **order parameter** is:

$$m = \sqrt{\left(p_R - \frac{1}{3}\right)^2 + \left(p_P - \frac{1}{3}\right)^2 + \left(p_S - \frac{1}{3}\right)^2}$$

$m = 0$ for uniform (disordered), $m$ large for concentrated (ordered). The **susceptibility** $\chi = \partial m / \partial T$ diverges at the critical temperature.

### Thermal State Classification

| State | Normalized $T$ | Interpretation |
|---|---|---|
| 🧊 Frozen | $T < 0.1$ | All agents on one strategy |
| ❄️ Cold | $0.1 \leq T < 0.5$ | Mostly ordered |
| 🌡️ Warm | $0.5 \leq T < 0.9$ | Mixed |
| 🔥 Hot | $T \geq 0.9$ | Near-random |

### Complexity

All operations are $O(1)$ — the system has exactly 3 states, so all sums, logs, and comparisons are constant-time.

## Quick Start

```rust
use ternary_thermodynamics::{
    TernaryDistribution, SystemEntropy, SystemTemperature,
    Strategy,
};

// Observe a population: 50% Rock, 30% Paper, 20% Scissors
let dist = TernaryDistribution::new(0.5, 0.3, 0.2);

// Entropy
let s = SystemEntropy::entropy(&dist);
let s_norm = SystemEntropy::normalized_entropy(&dist);
println!("Entropy: {:.4} nats (normalized: {:.4})", s, s_norm);

// Temperature
let temp = SystemTemperature::from_entropy(&dist);
println!("Temperature: {:.4}", temp);
println!("Thermal state: {:?}", SystemTemperature::thermal_state(&dist));

// Infer temperature from fitness
let fitnesses = [1.0, 0.5, 0.0]; // Rock=1.0, Paper=0.5, Scissors=0.0
let t_fit = SystemTemperature::from_boltzmann_fit(&dist, fitnesses);
println!("Boltzmann-fitted T: {:?}", t_fit);

// Uniform distribution = maximum entropy
let uniform = TernaryDistribution::uniform();
assert!((SystemEntropy::entropy(&uniform) - 3f64.ln()).abs() < 1e-10);
```

## API

### `TernaryDistribution`
Fields: `p_rock: f64`, `p_paper: f64`, `p_scissors: f64`. Constructor validates that probabilities are non-negative and sum to 1.0. Methods: `uniform()`, `deterministic(strategy)`, `prob(strategy)`, `as_tuple()`, `as_array()`.

### `SystemEntropy`
- `entropy(&dist) → f64` — Shannon entropy in nats
- `max_entropy() → f64` — Returns $\ln 3$
- `normalized_entropy(&dist) → f64` — $S / S_{\max} \in [0, 1]$
- `from_counts([u64; 3]) → Option<f64>` — Entropy from raw counts

### `SystemTemperature`
- `from_entropy(&dist) → f64` — Normalized entropy as temperature proxy
- `from_boltzmann_fit(&dist, fitnesses) → Option<f64>` — Infer $T$ from fitness-probability pairs
- `thermal_state(&dist) → ThermalState` — Frozen/Cold/Warm/Hot classification

### `ThermalState`
Enum: `Frozen`, `Cold`, `Warm`, `Hot`.

## Architecture Notes

Within the **γ + η = C** framework:

- **γ (gamma)** — the strategy distribution: the population-level *signal* of what agents are doing
- **η (eta)** — temperature and fitness landscape: the *environment* that shapes which strategies survive
- **C** — **configuration equilibrium**: when $\gamma$ and $\eta$ are in balance, the system sits at a free-energy minimum — the most stable, predictable state. Phase transitions occur when external perturbations push the system past a free-energy barrier.

The crate is `#![forbid(unsafe_code)]` with zero external dependencies.

## References

1. Boltzmann, L. (1877). "Über die Beziehung zwischen dem zweiten Hauptsatze der mechanischen Wärmetheorie und der Wahrscheinlichkeitsrechnung." *Wiener Berichte*, 76, 373-435. — Original definition of entropy and the Boltzmann distribution.
2. Shannon, C. E. (1948). "A Mathematical Theory of Communication." *Bell System Technical Journal*, 27, 379-423. — Information-theoretic entropy.
3. Jaynes, E. T. (1957). "Information Theory and Statistical Mechanics." *Physical Review*, 106(4), 620-630. — Maximum entropy principle linking information theory to thermodynamics.
4. Szabó, G., & Fath, G. (2007). "Evolutionary Games on Graphs." *Physics Reports*, 446(4-6), 97-216. — Three-strategy games and phase transitions.

## License

MIT
