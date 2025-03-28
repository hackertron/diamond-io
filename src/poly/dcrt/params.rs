use crate::poly::params::PolyParams;
use num_bigint::BigUint;
use num_traits::Num;
use openfhe::ffi;
use std::{fmt::Debug, sync::Arc};

#[derive(Clone)]
pub struct DCRTPolyParams {
    /// polynomial ring dimension
    ring_dimension: u32,
    /// size of the tower
    crt_depth: usize,
    /// number of bits of each tower's modulus
    crt_bits: usize,
    /// ring modulus
    modulus: Arc<BigUint>,
}

impl Debug for DCRTPolyParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DCRTPolyParams")
            .field("modulus", &self.modulus)
            .field("ring_dimension", &self.ring_dimension())
            .field("crt_depth", &self.crt_depth())
            .field("crt_bits", &self.crt_bits())
            .finish()
    }
}

impl PolyParams for DCRTPolyParams {
    type Modulus = Arc<BigUint>;

    fn ring_dimension(&self) -> u32 {
        self.ring_dimension
    }

    fn modulus(&self) -> Self::Modulus {
        self.modulus.clone()
    }

    fn modulus_bits(&self) -> usize {
        self.modulus.bits() as usize
    }
}

#[cfg(feature = "test")]
impl Default for DCRTPolyParams {
    fn default() -> Self {
        Self::new(4, 2, 17)
    }
}

impl DCRTPolyParams {
    pub fn new(ring_dimension: u32, crt_depth: usize, crt_bits: usize) -> Self {
        // assert that ring_dimension is a power of 2
        assert!(ring_dimension.is_power_of_two(), "ring_dimension must be a power of 2");
        let modulus = ffi::GenModulus(ring_dimension, crt_depth, crt_bits);
        Self {
            ring_dimension,
            crt_depth,
            crt_bits,
            modulus: Arc::new(BigUint::from_str_radix(&modulus, 10).expect("invalid string")),
        }
    }

    pub fn crt_depth(&self) -> usize {
        self.crt_depth
    }

    pub fn crt_bits(&self) -> usize {
        self.crt_bits
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_params_initiation_ring_dimension() {
        let ring_dimension = 16;
        let crt_depth = 4;
        let crt_bits = 51;
        let p = DCRTPolyParams::new(ring_dimension, crt_depth, crt_bits);
        assert_eq!(p.ring_dimension(), ring_dimension);
        assert_eq!(p.modulus_bits(), 204);

        let ring_dimension = 2;
        let crt_depth = 4;
        let crt_bits = 51;
        let p = DCRTPolyParams::new(ring_dimension, crt_depth, crt_bits);
        assert_eq!(p.ring_dimension(), 2);
        assert_eq!(p.modulus_bits(), 204);

        let ring_dimension = 1;
        let crt_depth = 4;
        let crt_bits = 51;
        let p = DCRTPolyParams::new(ring_dimension, crt_depth, crt_bits);
        assert_eq!(p.ring_dimension(), 1);
        assert_eq!(p.modulus_bits(), 204);
    }

    #[test]
    fn test_params_initiation_crt_depth() {
        let ring_dimension = 16;
        let crt_depth = 4;
        let crt_bits = 51;
        let p = DCRTPolyParams::new(ring_dimension, crt_depth, crt_bits);
        assert_eq!(p.ring_dimension(), ring_dimension);
        assert_eq!(p.modulus_bits() as u32, (crt_depth * crt_bits) as u32);

        let ring_dimension = 16;
        let crt_depth = 5;
        let crt_bits = 51;
        let p = DCRTPolyParams::new(ring_dimension, crt_depth, crt_bits);
        assert_eq!(p.ring_dimension(), ring_dimension);
        assert_eq!(p.modulus_bits() as u32, (crt_depth * crt_bits) as u32);

        let ring_dimension = 16;
        let crt_depth = 6;
        let crt_bits = 51;
        let p = DCRTPolyParams::new(ring_dimension, crt_depth, crt_bits);
        assert_eq!(p.ring_dimension(), ring_dimension);
        assert_eq!(p.modulus_bits() as u32, (crt_depth * crt_bits) as u32);

        let ring_dimension = 16;
        let crt_depth = 7;
        let crt_bits = 20;
        let p = DCRTPolyParams::new(ring_dimension, crt_depth, crt_bits);
        assert_eq!(p.ring_dimension(), ring_dimension);
        assert_eq!(p.modulus_bits() as u32, (crt_depth * crt_bits) as u32);
    }

    #[test]
    #[should_panic(expected = "n must be a power of 2")]
    fn test_params_initiation_non_power_of_two() {
        let ring_dimension = 20;
        let crt_depth = 4;
        let crt_bits = 51;
        let _p = DCRTPolyParams::new(ring_dimension, crt_depth, crt_bits); // This should panic

        let ring_dimension = 0;
        let crt_depth = 4;
        let crt_bits = 51;
        let _p = DCRTPolyParams::new(ring_dimension, crt_depth, crt_bits); // This should panic
    }
}
