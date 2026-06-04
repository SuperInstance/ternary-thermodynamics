# Future Integration: ternary-thermodynamics

## Current State
Models ternary agent systems through statistical mechanics: `TernaryDistribution` over Rock/Paper/Scissors strategies, Shannon entropy, Boltzmann distribution weighted by fitness, partition function Z, Helmholtz and Gibbs free energy, and phase transition detection (order-disorder transitions).

## Integration Opportunities

### With ternary-cell (Energy-Based Room Resources)
ternary-cell's conservation law (γ + H ≈ const, from the architecture doc) is exactly a thermodynamic conservation principle. ternary-thermodynamics provides the formal framework: cell energy is internal energy U, surprise is entropy S, the vibe phase is temperature T, and the conservation law is the first law of thermodynamics (ΔU = TΔS - work). Phase transitions in the cell grid correspond to order-disorder transitions — cells spontaneously synchronizing or desynchronizing.

### With ternary-thermodynamics → construct-core (Resource Budgets)
construct-core's skill loading consumes resources (memory, compute). ternary-thermodynamics models resource allocation as energy distribution: loading a skill costs energy (free energy decreases), unloading recovers energy. The partition function Z normalizes skill costs, and the Boltzmann distribution naturally weights skills by cost-benefit ratio at the current "temperature" (urgency level).

### With ternary-games (Thermodynamic Game Theory)
The cross-pollination report suggests Nash equilibrium finding via game theory for consensus. ternary-thermodynamics adds the energy dimension: Nash equilibria are thermodynamic ground states (minimum free energy). A game's payoff matrix becomes a Hamiltonian, and equilibrium finding is energy minimization. Phase transitions in the game correspond to strategy regime changes — from cooperative to competitive.

## Potential in Mature Systems
In room-as-codespace, each room has thermodynamic properties: entropy (information diversity), temperature (activity level), free energy (available compute). PLATO monitors these properties across all rooms. When a room's entropy drops below threshold (monoculture risk), PLATO injects diversity (random signals, curriculum lessons). When free energy is low (resources exhausted), PLATO triggers GC or suspends the Codespace. The thermodynamic framework unifies resource monitoring under a single physical analogy.

## Cross-Pollination Ideas
- **ternary-noise**: Noise injection as temperature — higher noise = higher temperature = more exploration. Anneal noise to cool the system into a ground state.
- **ternary-pareto**: Multi-objective free energy — minimize free energy across multiple objectives simultaneously using Pareto fronts.
- **ternary-ensemble**: Ensemble diversity as entropy — a diverse ensemble has high entropy, a homogeneous one has low entropy. Maximize ensemble entropy for robustness.

## Dependencies for Next Steps
- Define `RoomThermodynamics` type with entropy, temperature, free energy for rooms
- Add conservation law verification using thermodynamic formalism to ternary-cell
- Implement energy-based skill cost model for construct-core
- Benchmark phase transition detection on real cell grid data
