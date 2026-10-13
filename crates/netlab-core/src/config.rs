//! Configuração global da simulação.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SimulationMode {
    Run,
    Step,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SimulationConfig {
    pub seed: u64,
    pub mode: SimulationMode,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            seed: 0,
            mode: SimulationMode::Run,
        }
    }
}
