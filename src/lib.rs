//! # Ternary Thermodynamics
//!
//! Statistical mechanics analogs for ternary agent systems.
//!
//! This crate models a system of agents that each adopt one of three strategies
//! (Rock, Paper, Scissors) through the lens of statistical mechanics:
//!
//! - **Entropy** — Shannon entropy of the strategy distribution
//! - **Temperature** — inverse sensitivity to fitness differences
//! - **Boltzmann Distribution** — strategy probabilities weighted by fitness
//! - **Partition Function** — normalization constant Z
//! - **Free Energy** — Helmholtz and Gibbs analogs
//! - **Phase Transitions** — detect order-disorder transitions

#![warn(missing_docs)]
#![forbid(unsafe_code)]

use std::vec::Vec;

/// The three strategies in a ternary agent system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Strategy {
    /// Strategy 0 (analogous to "Rock")
    Rock,
    /// Strategy 1 (analogous to "Paper")
    Paper,
    /// Strategy 2 (analogous to "Scissors")
    Scissors,
}

/// A probability distribution over the three strategies.
///
/// Probabilities must be non-negative and sum to 1.0.
#[derive(Debug, Clone, PartialEq)]
pub struct TernaryDistribution {
    /// Probability of Rock
    pub p_rock: f64,
    /// Probability of Paper
    pub p_paper: f64,
    /// Probability of Scissors
    pub p_scissors: f64,
}

impl TernaryDistribution {
    /// Create a new distribution from three probabilities.
    ///
    /// # Panics
    /// Panics if probabilities are negative or don't sum to approximately 1.0.
    pub fn new(p_rock: f64, p_paper: f64, p_scissors: f64) -> Self {
        assert!(p_rock >= 0.0 && p_paper >= 0.0 && p_scissors >= 0.0);
        let sum = p_rock + p_paper + p_scissors;
        assert!((sum - 1.0).abs() < 1e-10, "probabilities must sum to 1.0, got {}", sum);
        Self { p_rock, p_paper, p_scissors }
    }

    /// Create a uniform distribution (1/3, 1/3, 1/3).
    pub fn uniform() -> Self {
        Self::new(1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0)
    }

    /// Create a deterministic distribution concentrated on one strategy.
    pub fn deterministic(strategy: Strategy) -> Self {
        match strategy {
            Strategy::Rock => Self::new(1.0, 0.0, 0.0),
            Strategy::Paper => Self::new(0.0, 1.0, 0.0),
            Strategy::Scissors => Self::new(0.0, 0.0, 1.0),
        }
    }

    /// Get probability for a given strategy.
    pub fn prob(&self, strategy: Strategy) -> f64 {
        match strategy {
            Strategy::Rock => self.p_rock,
            Strategy::Paper => self.p_paper,
            Strategy::Scissors => self.p_scissors,
        }
    }

    /// Return all three probabilities as a tuple.
    pub fn as_tuple(&self) -> (f64, f64, f64) {
        (self.p_rock, self.p_paper, self.p_scissors)
    }

    /// Return probabilities as a slice-friendly array.
    pub fn as_array(&self) -> [f64; 3] {
        [self.p_rock, self.p_paper, self.p_scissors]
    }
}

// ---------------------------------------------------------------------------
// SystemEntropy
// ---------------------------------------------------------------------------

/// Shannon entropy of a ternary distribution.
///
/// In statistical mechanics, entropy measures the disorder of a system.
/// For a ternary system with probabilities (p₁, p₂, p₃), the Shannon entropy is:
///
/// ```text
/// S = -Σ pᵢ ln(pᵢ)
/// ```
///
/// Maximum entropy is ln(3) ≈ 1.0986 (uniform distribution).
/// Minimum entropy is 0 (deterministic state).
pub struct SystemEntropy;

impl SystemEntropy {
    /// Compute Shannon entropy in nats (natural log base).
    pub fn entropy(dist: &TernaryDistribution) -> f64 {
        let probs = dist.as_array();
        let mut h = 0.0;
        for &p in &probs {
            if p > 0.0 {
                h -= p * p.ln();
            }
        }
        h
    }

    /// Maximum possible entropy for a 3-state system: ln(3).
    pub fn max_entropy() -> f64 {
        3.0_f64.ln()
    }

    /// Normalized entropy: S / S_max, in [0, 1].
    ///
    /// 0 = fully ordered (deterministic), 1 = fully disordered (uniform).
    pub fn normalized_entropy(dist: &TernaryDistribution) -> f64 {
        let h = Self::entropy(dist);
        let h_max = Self::max_entropy();
        if h_max == 0.0 { 0.0 } else { h / h_max }
    }

    /// Compute entropy from raw counts (unnormalized frequencies).
    ///
    /// Returns `None` if all counts are zero.
    pub fn from_counts(counts: [u64; 3]) -> Option<f64> {
        let total = counts[0] + counts[1] + counts[2];
        if total == 0 {
            return None;
        }
        let mut h = 0.0;
        for &c in &counts {
            if c > 0 {
                let p = c as f64 / total as f64;
                h -= p * p.ln();
            }
        }
        Some(h)
    }
}

// ---------------------------------------------------------------------------
// SystemTemperature
// ---------------------------------------------------------------------------

/// "Temperature" of the agent system.
///
/// In statistical mechanics, temperature controls how agents distribute across
/// strategies of different fitness:
///
/// - **High temperature** → agents spread uniformly (random play)
/// - **Low temperature** → agents concentrate on best strategy (ordered)
/// - **Zero temperature** → fully deterministic (all on fittest)
///
/// We define temperature as the inverse of the average sensitivity to fitness
/// differences, or equivalently from the entropy derivative.
pub struct SystemTemperature;

impl SystemTemperature {
    /// Estimate temperature from a distribution using the relationship
    /// between entropy and temperature.
    ///
    /// Uses: T = S / S_max, a simple normalized measure.
    /// Range: [0, 1] where 0 = frozen, 1 = hot/random.
    pub fn from_entropy(dist: &TernaryDistribution) -> f64 {
        SystemEntropy::normalized_entropy(dist)
    }

    /// Estimate temperature from fitness values using Boltzmann inversion.
    ///
    /// Given a distribution and corresponding fitnesses, infer the temperature
    /// that would produce that distribution: T ≈ ΔE / Δ(ln p).
    ///
    /// Returns `None` if the distribution is degenerate.
    pub fn from_boltzmann_fit(
        dist: &TernaryDistribution,
        fitnesses: [f64; 3],
    ) -> Option<f64> {
        // Find two non-zero probabilities and compute T = -(E2 - E1) / (ln p2 - ln p1)
        let probs = dist.as_array();
        let mut idx: Vec<usize> = (0..3).filter(|&i| probs[i] > 1e-15).collect();
        if idx.len() < 2 {
            return None;
        }
        // Use the two with largest probability gap
        idx.sort_by(|&a, &b| probs[b].partial_cmp(&probs[a]).unwrap());

        let i = idx[0];
        let j = idx[idx.len() - 1];
        let delta_e = fitnesses[j] - fitnesses[i];
        let delta_ln_p = probs[j].ln() - probs[i].ln();

        if delta_ln_p.abs() < 1e-15 {
            // All probs equal → infinite temperature
            return Some(f64::INFINITY);
        }

        // T = delta_fitness / delta_ln_p (from p ∝ exp(fitness/T))
        let temp = delta_e / delta_ln_p;
        if temp > 0.0 { Some(temp) } else { None }
    }

    /// Classify the system's thermal state.
    pub fn thermal_state(dist: &TernaryDistribution) -> ThermalState {
        let t = Self::from_entropy(dist);
        if t < 0.1 {
            ThermalState::Frozen
        } else if t < 0.5 {
            ThermalState::Cold
        } else if t < 0.9 {
            ThermalState::Warm
        } else {
            ThermalState::Hot
        }
    }
}

/// Thermal state classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThermalState {
    /// T ≈ 0 — fully ordered
    Frozen,
    /// Low temperature — mostly ordered
    Cold,
    /// Moderate temperature — mixed
    Warm,
    /// High temperature — nearly random
    Hot,
}

// ---------------------------------------------------------------------------
// PartitionFunction
// ---------------------------------------------------------------------------

/// Partition function Z for the ternary system.
///
/// The partition function normalizes the Boltzmann distribution:
///
/// ```text
/// Z = Σ exp(-Eᵢ / T)
/// ```
///
/// where Eᵢ are energies (negative fitnesses) and T is temperature.
pub struct PartitionFunction;

impl PartitionFunction {
    /// Compute the partition function Z = Σ exp(β · fitnessᵢ)
    ///
    /// We use `beta = 1/T` and treat fitness as negative energy,
    /// so higher fitness → higher probability.
    pub fn compute(fitnesses: [f64; 3], temperature: f64) -> f64 {
        let beta = if temperature > 0.0 { 1.0 / temperature } else { f64::MAX };
        fitnesses.iter().map(|&e| (beta * e).exp()).sum()
    }

    /// Compute Z for a series of fitness snapshots at different temperatures.
    pub fn compute_curve(fitnesses: [f64; 3], temperatures: &[f64]) -> Vec<f64> {
        temperatures.iter().map(|&t| Self::compute(fitnesses, t)).collect()
    }
}

// ---------------------------------------------------------------------------
// BoltzmannDistribution
// ---------------------------------------------------------------------------

/// Boltzmann distribution over strategies.
///
/// Given fitness values and a temperature, computes strategy probabilities:
///
/// ```text
/// pᵢ = exp(β · fitnessᵢ) / Z
/// ```
///
/// where β = 1/T and Z is the partition function.
pub struct BoltzmannDistribution;

impl BoltzmannDistribution {
    /// Compute the Boltzmann distribution from fitnesses and temperature.
    ///
    /// Returns `None` if temperature is non-positive.
    pub fn compute(fitnesses: [f64; 3], temperature: f64) -> Option<TernaryDistribution> {
        if temperature <= 0.0 {
            return None;
        }

        let beta = 1.0 / temperature;
        let boltzmann: [f64; 3] = fitnesses.map(|e| (beta * e).exp());
        let z = boltzmann[0] + boltzmann[1] + boltzmann[2];

        Some(TernaryDistribution::new(
            boltzmann[0] / z,
            boltzmann[1] / z,
            boltzmann[2] / z,
        ))
    }

    /// Compute the distribution at zero temperature (winner-takes-all).
    ///
    /// All probability mass goes to the strategy with highest fitness.
    /// In case of ties, probability is split equally among winners.
    pub fn compute_zero_temp(fitnesses: [f64; 3]) -> TernaryDistribution {
        let max_fitness = fitnesses.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let winners: Vec<usize> = (0..3).filter(|&i| (fitnesses[i] - max_fitness).abs() < 1e-15).collect();
        let share = 1.0 / winners.len() as f64;

        let mut probs = [0.0f64; 3];
        for &w in &winners {
            probs[w] = share;
        }

        TernaryDistribution::new(probs[0], probs[1], probs[2])
    }

    /// Compute a curve of distributions over a range of temperatures.
    pub fn compute_curve(fitnesses: [f64; 3], temperatures: &[f64]) -> Vec<Option<TernaryDistribution>> {
        temperatures.iter().map(|&t| Self::compute(fitnesses, t)).collect()
    }
}

// ---------------------------------------------------------------------------
// FreeEnergy
// ---------------------------------------------------------------------------

/// Free energy analogs for the ternary system.
///
/// **Helmholtz free energy**: F = -T · ln(Z) = ⟨E⟩ - T·S
///
/// **Gibbs free energy**: G = ⟨E⟩ - T·S + p·V analog
///
/// In our agent system, "energy" is negative fitness, and we interpret
/// the free energy as a measure of the system's capacity for useful work
/// (i.e., strategic adaptation).
pub struct FreeEnergy;

impl FreeEnergy {
    /// Compute Helmholtz free energy: F = -T · ln(Z)
    ///
    /// Lower free energy means a more stable configuration.
    pub fn helmholtz(fitnesses: [f64; 3], temperature: f64) -> f64 {
        let z = PartitionFunction::compute(fitnesses, temperature);
        -temperature * z.ln()
    }

    /// Compute average energy: ⟨E⟩ = -Σ pᵢ · fitnessᵢ
    ///
    /// (We use negative fitness as energy.)
    pub fn average_energy(dist: &TernaryDistribution, fitnesses: [f64; 3]) -> f64 {
        let probs = dist.as_array();
        -(probs[0] * fitnesses[0] + probs[1] * fitnesses[1] + probs[2] * fitnesses[2])
    }

    /// Compute Helmholtz free energy from distribution: F = ⟨E⟩ - T·S
    pub fn helmholtz_from_dist(
        dist: &TernaryDistribution,
        fitnesses: [f64; 3],
        temperature: f64,
    ) -> f64 {
        let avg_e = Self::average_energy(dist, fitnesses);
        let entropy = SystemEntropy::entropy(dist);
        avg_e - temperature * entropy
    }

    /// Compute Gibbs free energy analog: G = F + p·V analog.
    ///
    /// Here the p·V analog is the "pressure" of competition times
    /// the "volume" (number of available strategies). We use a simple
    /// model where pressure = variance of fitnesses.
    pub fn gibbs(
        dist: &TernaryDistribution,
        fitnesses: [f64; 3],
        temperature: f64,
    ) -> f64 {
        let f = Self::helmholtz_from_dist(dist, fitnesses, temperature);
        let mean_fit = (fitnesses[0] + fitnesses[1] + fitnesses[2]) / 3.0;
        let variance = fitnesses.iter().map(|&x| (x - mean_fit).powi(2)).sum::<f64>() / 3.0;
        let pressure = variance;
        let volume = 3.0; // number of strategies
        f + pressure * volume
    }

    /// Compute the free energy curve over a range of temperatures.
    pub fn helmholtz_curve(fitnesses: [f64; 3], temperatures: &[f64]) -> Vec<f64> {
        temperatures.iter().map(|&t| Self::helmholtz(fitnesses, t)).collect()
    }
}

// ---------------------------------------------------------------------------
// PhaseTransitionDetector
// ---------------------------------------------------------------------------

/// Detector for phase transitions in the ternary system.
///
/// A phase transition occurs when the system undergoes a qualitative change
/// in behavior — e.g., from ordered (one strategy dominates) to disordered
/// (uniform mixing). This is analogous to freezing/boiling in physical systems.
///
/// We detect transitions by tracking an order parameter and looking for
/// sharp changes.
pub struct PhaseTransitionDetector {
    /// Sensitivity threshold for detecting transitions.
    pub sensitivity: f64,
}

/// Result of phase transition detection.
#[derive(Debug, Clone)]
pub struct TransitionPoint {
    /// Index in the temperature array where transition occurs.
    pub index: usize,
    /// Temperature at which transition occurs.
    pub temperature: f64,
    /// Magnitude of the order parameter change.
    pub magnitude: f64,
    /// Type of transition detected.
    pub transition_type: TransitionType,
}

/// Classification of the transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionType {
    /// Order → Disorder (like melting)
    Melting,
    /// Disorder → Order (like freezing)
    Freezing,
    /// Gradual crossover, no sharp transition
    Crossover,
}

impl PhaseTransitionDetector {
    /// Create a new detector with given sensitivity.
    pub fn new(sensitivity: f64) -> Self {
        Self { sensitivity }
    }

    /// Create with default sensitivity (0.3).
    pub fn default_detector() -> Self {
        Self::new(0.3)
    }

    /// Order parameter: measures how "ordered" the system is.
    ///
    /// Defined as the deviation from uniform: max(pᵢ) - 1/3.
    /// Range: [0, 2/3]. 0 = uniform, 2/3 = deterministic.
    pub fn order_parameter(dist: &TernaryDistribution) -> f64 {
        let max_p = dist.as_array().into_iter().fold(0.0f64, f64::max);
        max_p - 1.0 / 3.0
    }

    /// Detect phase transitions by scanning a temperature sweep.
    ///
    /// Returns a list of transition points where the order parameter
    /// changes by more than `sensitivity` between adjacent temperatures.
    pub fn detect_transitions(
        &self,
        fitnesses: [f64; 3],
        temperatures: &[f64],
    ) -> Vec<TransitionPoint> {
        if temperatures.len() < 2 {
            return Vec::new();
        }

        let order_params: Vec<f64> = temperatures
            .iter()
            .map(|&t| {
                let dist = BoltzmannDistribution::compute(fitnesses, t)
                    .unwrap_or_else(|| TernaryDistribution::uniform());
                Self::order_parameter(&dist)
            })
            .collect();

        let mut transitions = Vec::new();

        for i in 1..order_params.len() {
            let delta = order_params[i] - order_params[i - 1];
            if delta.abs() > self.sensitivity {
                let t_type = if delta > 0.0 {
                    TransitionType::Freezing
                } else {
                    TransitionType::Melting
                };
                transitions.push(TransitionPoint {
                    index: i,
                    temperature: temperatures[i],
                    magnitude: delta.abs(),
                    transition_type: t_type,
                });
            }
        }

        transitions
    }

    /// Compute the order parameter curve over a temperature sweep.
    pub fn order_parameter_curve(
        fitnesses: [f64; 3],
        temperatures: &[f64],
    ) -> Vec<f64> {
        temperatures
            .iter()
            .map(|&t| {
                let dist = BoltzmannDistribution::compute(fitnesses, t)
                    .unwrap_or_else(|| TernaryDistribution::uniform());
                Self::order_parameter(&dist)
            })
            .collect()
    }

    /// Compute specific heat analog: C = d⟨E⟩/dT.
    ///
    /// Peaks in specific heat indicate phase transitions.
    pub fn specific_heat_curve(
        fitnesses: [f64; 3],
        temperatures: &[f64],
    ) -> Vec<f64> {
        let energies: Vec<f64> = temperatures
            .iter()
            .map(|&t| {
                let dist = BoltzmannDistribution::compute(fitnesses, t)
                    .unwrap_or_else(|| TernaryDistribution::uniform());
                FreeEnergy::average_energy(&dist, fitnesses)
            })
            .collect();

        let mut heat = Vec::with_capacity(energies.len().saturating_sub(1));
        for i in 1..energies.len() {
            let dt = temperatures[i] - temperatures[i - 1];
            if dt.abs() > 1e-15 {
                heat.push((energies[i] - energies[i - 1]) / dt);
            } else {
                heat.push(0.0);
            }
        }
        heat
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uniform_distribution_entropy_is_max() {
        let dist = TernaryDistribution::uniform();
        let h = SystemEntropy::entropy(&dist);
        let expected = 3.0_f64.ln();
        assert!((h - expected).abs() < 1e-10, "entropy = {}, expected = {}", h, expected);
    }

    #[test]
    fn test_deterministic_entropy_is_zero() {
        let dist = TernaryDistribution::deterministic(Strategy::Rock);
        let h = SystemEntropy::entropy(&dist);
        assert!(h.abs() < 1e-10, "entropy should be 0, got {}", h);
    }

    #[test]
    fn test_normalized_entropy_range() {
        let uniform = TernaryDistribution::uniform();
        let det = TernaryDistribution::deterministic(Strategy::Paper);
        assert!((SystemEntropy::normalized_entropy(&uniform) - 1.0).abs() < 1e-10);
        assert!(SystemEntropy::normalized_entropy(&det).abs() < 1e-10);
    }

    #[test]
    fn test_entropy_from_counts() {
        let h = SystemEntropy::from_counts([100, 100, 100]).unwrap();
        assert!((h - 3.0_f64.ln()).abs() < 1e-10);
    }

    #[test]
    fn test_entropy_from_counts_zero() {
        assert!(SystemEntropy::from_counts([0, 0, 0]).is_none());
    }

    #[test]
    fn test_temperature_uniform_is_hot() {
        let dist = TernaryDistribution::uniform();
        assert_eq!(SystemTemperature::thermal_state(&dist), ThermalState::Hot);
    }

    #[test]
    fn test_temperature_deterministic_is_frozen() {
        let dist = TernaryDistribution::deterministic(Strategy::Rock);
        assert_eq!(SystemTemperature::thermal_state(&dist), ThermalState::Frozen);
    }

    #[test]
    fn test_boltzmann_uniform_fitness() {
        let fitnesses = [1.0, 1.0, 1.0];
        let dist = BoltzmannDistribution::compute(fitnesses, 1.0).unwrap();
        assert!((dist.p_rock - 1.0 / 3.0).abs() < 1e-10);
        assert!((dist.p_paper - 1.0 / 3.0).abs() < 1e-10);
        assert!((dist.p_scissors - 1.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_boltzmann_zero_temp() {
        let fitnesses = [3.0, 1.0, 2.0];
        let dist = BoltzmannDistribution::compute_zero_temp(fitnesses);
        assert!((dist.p_rock - 1.0).abs() < 1e-10);
        assert!(dist.p_paper.abs() < 1e-10);
        assert!(dist.p_scissors.abs() < 1e-10);
    }

    #[test]
    fn test_boltzmann_zero_temp_tie() {
        let fitnesses = [2.0, 2.0, 1.0];
        let dist = BoltzmannDistribution::compute_zero_temp(fitnesses);
        assert!((dist.p_rock - 0.5).abs() < 1e-10);
        assert!((dist.p_paper - 0.5).abs() < 1e-10);
        assert!(dist.p_scissors.abs() < 1e-10);
    }

    #[test]
    fn test_boltzmann_low_temp_concentrates() {
        let fitnesses = [3.0, 1.0, 2.0];
        let dist = BoltzmannDistribution::compute(fitnesses, 0.01).unwrap();
        assert!(dist.p_rock > 0.99, "low T should concentrate on best: p_rock = {}", dist.p_rock);
    }

    #[test]
    fn test_boltzmann_high_temp_uniform() {
        let fitnesses = [3.0, 1.0, 2.0];
        let dist = BoltzmannDistribution::compute(fitnesses, 1000.0).unwrap();
        assert!((dist.p_rock - 1.0 / 3.0).abs() < 0.01);
    }

    #[test]
    fn test_boltzmann_negative_temp() {
        assert!(BoltzmannDistribution::compute([1.0, 2.0, 3.0], -1.0).is_none());
    }

    #[test]
    fn test_partition_function_value() {
        let fitnesses = [0.0, 0.0, 0.0];
        let z = PartitionFunction::compute(fitnesses, 1.0);
        assert!((z - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_partition_function_curve() {
        let fitnesses = [1.0, 2.0, 3.0];
        let temps = [0.1, 1.0, 10.0];
        let curve = PartitionFunction::compute_curve(fitnesses, &temps);
        assert_eq!(curve.len(), 3);
        // Higher temp → more uniform → Z approaches 3 * exp(avg_fitness * beta)
        assert!(curve[0] > curve[2]); // at low temp, Z dominated by best fitness
    }

    #[test]
    fn test_helmholtz_free_energy() {
        let fitnesses = [1.0, 1.0, 1.0];
        let t = 1.0;
        let z = PartitionFunction::compute(fitnesses, t);
        let expected = -t * z.ln();
        let f = FreeEnergy::helmholtz(fitnesses, t);
        assert!((f - expected).abs() < 1e-10);
    }

    #[test]
    fn test_helmholtz_from_dist_matches() {
        let fitnesses = [3.0, 1.0, 2.0];
        let t = 1.0;
        let dist = BoltzmannDistribution::compute(fitnesses, t).unwrap();
        let f1 = FreeEnergy::helmholtz(fitnesses, t);
        let f2 = FreeEnergy::helmholtz_from_dist(&dist, fitnesses, t);
        // These should be approximately equal
        assert!((f1 - f2).abs() < 1e-8, "F1={}, F2={}", f1, f2);
    }

    #[test]
    fn test_average_energy() {
        let dist = TernaryDistribution::deterministic(Strategy::Rock);
        let fitnesses = [3.0, 1.0, 2.0];
        let e = FreeEnergy::average_energy(&dist, fitnesses);
        assert!((e - (-3.0)).abs() < 1e-10);
    }

    #[test]
    fn test_order_parameter_uniform_is_zero() {
        let dist = TernaryDistribution::uniform();
        let op = PhaseTransitionDetector::order_parameter(&dist);
        assert!(op.abs() < 1e-10);
    }

    #[test]
    fn test_order_parameter_deterministic() {
        let dist = TernaryDistribution::deterministic(Strategy::Rock);
        let op = PhaseTransitionDetector::order_parameter(&dist);
        assert!((op - 2.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_phase_transition_detection() {
        let fitnesses = [5.0, 1.0, 1.0];
        // Sweep from high T to low T (disorder → order)
        let temps: Vec<f64> = (0..50).map(|i| 0.05 + (i as f64) * 0.1).rev().collect();
        let detector = PhaseTransitionDetector::new(0.01);
        let transitions = detector.detect_transitions(fitnesses, &temps);
        // Should detect transitions for highly skewed fitness across a fine sweep
        assert!(!transitions.is_empty(), "should detect at least one transition");
    }

    #[test]
    fn test_specific_heat_curve_length() {
        let fitnesses = [3.0, 1.0, 2.0];
        let temps: Vec<f64> = (1..11).map(|i| i as f64 * 0.1).collect();
        let heat = PhaseTransitionDetector::specific_heat_curve(fitnesses, &temps);
        assert_eq!(heat.len(), temps.len() - 1);
    }

    #[test]
    fn test_gibbs_free_energy() {
        let fitnesses = [3.0, 1.0, 2.0];
        let t = 1.0;
        let dist = BoltzmannDistribution::compute(fitnesses, t).unwrap();
        let g = FreeEnergy::gibbs(&dist, fitnesses, t);
        // Gibbs should be >= Helmholtz (we add a non-negative pV term)
        let f = FreeEnergy::helmholtz_from_dist(&dist, fitnesses, t);
        assert!(g >= f - 1e-10, "G={} should be >= F={}", g, f);
    }

    #[test]
    fn test_distribution_prob_accessor() {
        let dist = TernaryDistribution::new(0.5, 0.3, 0.2);
        assert!((dist.prob(Strategy::Rock) - 0.5).abs() < 1e-10);
        assert!((dist.prob(Strategy::Paper) - 0.3).abs() < 1e-10);
        assert!((dist.prob(Strategy::Scissors) - 0.2).abs() < 1e-10);
    }

    #[test]
    fn test_temperature_from_boltzmann_fit() {
        let fitnesses = [3.0, 1.0, 2.0];
        let t = 1.0;
        let dist = BoltzmannDistribution::compute(fitnesses, t).unwrap();
        let inferred = SystemTemperature::from_boltzmann_fit(&dist, fitnesses);
        assert!(inferred.is_some(), "should infer temperature from non-degenerate distribution");
        let inferred_t = inferred.unwrap();
        assert!((inferred_t - t).abs() < 0.5, "inferred T={} vs actual T={}", inferred_t, t);
    }
}
