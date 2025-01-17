use crate::operations::{bit_decompose, poly_add};
use crate::Parameters;
use phantom_zone_math::{
    prelude::{ModulusOps, Sampler},
    ring::RingOps,
};
use rand::thread_rng;
pub struct PublicKey {
    pub b: Vec<Vec<Vec<u64>>>,
    pub params: Parameters,
}

impl PublicKey {
    /// Generate the public key `b` based on BGG+ RLWE attribute encoding
    /// where `b` is a matrix of ring elements of size `(ell + 1) x m`
    /// where `b[i][j]` is the polynomial at row i and column j
    pub fn new(params: &Parameters) -> Self {
        let mut rng = thread_rng();
        let ring = &params.ring;
        let mut b = vec![vec![vec![ring.zero(); ring.ring_size()]; params.m]; params.ell + 1];
        for i in 0..(params.ell + 1) {
            for j in 0..params.m {
                b[i][j] = ring.sample_uniform_vec(ring.ring_size(), &mut rng);
            }
        }
        Self {
            b,
            params: params.clone(),
        }
    }

    /// Perform a gate addition over the public key components at indices `idx_1` and `idx_2`
    pub fn add_gate(&self, idx_1: usize, idx_2: usize) -> Vec<Vec<u64>> {
        let ring = &self.params.ring;
        let m = self.params.m;
        let mut out = vec![vec![ring.zero(); ring.ring_size()]; m];
        for i in 0..m {
            out[i] = poly_add(&ring, &self.b[idx_1][i], &self.b[idx_2][i]);
        }
        out
    }

    pub fn mul_gate(&self, idx_1: usize, idx_2: usize) -> Vec<Vec<u64>> {
        let ring = &self.params.ring;
        let m = self.params.m;
        let mut out = vec![vec![ring.zero(); ring.ring_size()]; m];

        // Compute minus_b1 by multiplying each coefficient by -1
        let mut minus_b1 = vec![vec![ring.zero(); ring.ring_size()]; m];
        for i in 0..m {
            for j in 0..ring.ring_size() {
                // To get -1 * coefficient in the ring, we subtract the coefficient from 0
                minus_b1[i][j] = ring.sub(&ring.zero(), &self.b[idx_1][i][j]);
            }
        }

        let tau = bit_decompose(&self.params, &minus_b1);

        // Compute out = b2 * TAU
        for i in 0..m {
            for h in 0..m {
                let mut scratch = ring.allocate_scratch(1, 2, 0);
                let mut scratch = scratch.borrow_mut();
                let product = ring.take_poly(&mut scratch);

                // Multiply b2[h] by tau[h][i]
                ring.poly_mul(product, &self.b[idx_2][h], &tau[h][i], scratch.reborrow());

                out[i] = poly_add(ring, &out[i], &product.to_vec());
            }
        }
        out
    }
}
