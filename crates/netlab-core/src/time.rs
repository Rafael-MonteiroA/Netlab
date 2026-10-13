//! Relógio virtual, independente do relógio de parede do computador.

use std::fmt;
use std::ops::{Add, AddAssign, Sub};

/// Instante virtual expresso em milissegundos.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SimulationTime(u64);

impl SimulationTime {
    pub const ZERO: Self = Self(0);

    pub const fn from_millis(millis: u64) -> Self {
        Self(millis)
    }

    pub const fn new(millis: u64) -> Self {
        Self::from_millis(millis)
    }

    pub const fn as_millis(self) -> u64 {
        self.0
    }

    pub const fn millis(self) -> u64 {
        self.as_millis()
    }
}

impl Add<u64> for SimulationTime {
    type Output = Self;

    fn add(self, rhs: u64) -> Self::Output {
        Self(self.0.saturating_add(rhs))
    }
}

impl AddAssign<u64> for SimulationTime {
    fn add_assign(&mut self, rhs: u64) {
        self.0 = self.0.saturating_add(rhs);
    }
}

impl Sub for SimulationTime {
    type Output = u64;

    fn sub(self, rhs: Self) -> Self::Output {
        self.0.saturating_sub(rhs.0)
    }
}

impl fmt::Display for SimulationTime {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}ms", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::SimulationTime;

    #[test]
    fn supports_virtual_arithmetic() {
        let start = SimulationTime::from_millis(10);
        assert_eq!((start + 5).as_millis(), 15);
        assert_eq!(start + 5 - start, 5);
    }
}
