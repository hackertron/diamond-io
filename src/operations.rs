use crate::{Parameters, PublicKey};
use phantom_zone_crypto::util::distribution::NoiseDistribution;
use phantom_zone_math::{
    prelude::{ElemFrom, Gaussian, ModulusOps, Sampler},
    ring::{PrimeRing, RingOps},
};
use rand::{thread_rng, Rng};

/// A ciphertext in the BGG+ RLWE encoding scheme
#[derive(Clone, Debug)]
pub struct Ciphertext {
    /// The inner component matrix of size (ell + 1) × m where the rows are equal to the vectors G+b_0, x_1G+b_1, ..., x_ellG+b_ell
    inner: Vec<Vec<Vec<u64>>>,

    /// The secret vector which is a polynomial in the ring
    secret: Vec<u64>,

    /// The error matrix of size (ell + 1) × m where each row is equal to e_aS_0, e_aS_1, ..., e_aS_ell
    error: Vec<Vec<Vec<u64>>>,
}

impl Ciphertext {
    /// Create a new ciphertext by encoding an attribute vector
    ///
    /// # Arguments
    /// * `public_key`: The public key matrix
    /// * `params`: System parameters
    /// * `x`: Attribute vector to encode
    pub fn new(public_key: &PublicKey, params: &Parameters, x: &Vec<u64>) -> Self {
        assert!(x.len() == params.ell + 1);

        let mut rng = thread_rng();
        let ring = &params.ring;
        let s = ring.sample_uniform_vec(ring.ring_size(), &mut rng);
        let mut ct_inner = public_key.b.clone();

        // Generate error vectors
        let mut err_a = vec![vec![ring.zero(); ring.ring_size()]; params.m];
        let gaussian: NoiseDistribution = Gaussian(3.19).into();

        for i in 0..params.m {
            ring.sample_into::<i64>(&mut err_a[i], gaussian, rng.clone());
        }

        // Initialize error matrix
        let mut error = vec![vec![vec![ring.zero(); ring.ring_size()]; params.m]; params.ell + 1];

        for i in 0..params.ell + 1 {
            for si in 0..params.m {
                for sj in 0..params.m {
                    let random_bit = if rng.gen_bool(0.5) { 1 } else { -1 };
                    if random_bit == 1 {
                        error[i][si] = poly_add(ring, &error[i][si], &err_a[sj]);
                    }
                    if random_bit == -1 {
                        error[i][si] = poly_sub(ring, &error[i][si], &err_a[sj]);
                    }
                }
            }

            // Add gadget vector to ct_inner if x[i] = 1
            for j in 0..params.m {
                if x[i] == 1 {
                    ct_inner[i][j] = poly_add(ring, &ct_inner[i][j], &params.g[j]);
                }
            }
        }

        Self {
            inner: ct_inner,
            secret: s,
            error,
        }
    }

    pub fn inner(&self) -> &Vec<Vec<Vec<u64>>> {
        &self.inner
    }

    pub fn secret(&self) -> &Vec<u64> {
        &self.secret
    }

    pub fn error(&self) -> &Vec<Vec<Vec<u64>>> {
        &self.error
    }

    /// Compute the ct_full as inner * secret + error
    pub fn compute_ct_full(&self, ring: &PrimeRing) -> Vec<Vec<Vec<u64>>> {
        let mut ct_full = self.inner.clone();

        for i in 0..self.inner.len() {
            for j in 0..self.inner[0].len() {
                let mut scratch = ring.allocate_scratch(1, 2, 0);
                let mut scratch = scratch.borrow_mut();
                let c = ring.take_poly(&mut scratch);

                // Compute inner * secret
                ring.poly_mul(c, &self.inner[i][j], &self.secret, scratch.reborrow());

                // Add error
                ct_full[i][j] = super::poly_add(ring, &c.to_vec(), &self.error[i][j]);
            }
        }

        ct_full
    }
}

pub fn m_eval_add_x(params: &Parameters) -> Vec<Vec<Vec<u64>>> {
    let ring = &params.ring;
    let mut h_add_x = vec![vec![vec![ring.zero(); ring.ring_size()]; params.m]; 2 * params.m];

    // Fill both identity matrices at once
    for i in 0..params.m {
        // Set constant polynomial 1 on diagonals of both identity matrices
        h_add_x[i][i][0] = ring.elem_from(1u64);
        h_add_x[i + params.m][i][0] = ring.elem_from(1u64);
    }

    h_add_x
}

pub fn m_eval_mul_x(
    params: &Parameters,
    pub_key_1: &Vec<Vec<u64>>,
    x_2: u64,
) -> Vec<Vec<Vec<u64>>> {
    let ring = &params.ring;
    let mut h_mul_x = vec![vec![vec![ring.zero(); ring.ring_size()]; params.m]; 2 * params.m];

    // First matrix: Identity matrix scaled by x_2
    let mut x = vec![ring.zero(); ring.ring_size()];
    x[0] = ring.elem_from(x_2);
    for i in 0..params.m {
        h_mul_x[i][i] = x.clone();
    }

    // Second matrix: Tau(b1)
    // First compute -b1
    let mut minus_pub_key_1 = vec![vec![ring.zero(); ring.ring_size()]; params.m];
    for i in 0..params.m {
        for j in 0..ring.ring_size() {
            minus_pub_key_1[i][j] = ring.sub(&ring.zero(), &pub_key_1[i][j]);
        }
    }

    // Compute tau of -b1
    let tau = bit_decompose(params, &minus_pub_key_1);

    // Copy tau into the bottom half of h_mul_x
    for h in 0..params.m {
        for i in 0..params.m {
            h_mul_x[h + params.m][i] = tau[h][i].clone();
        }
    }

    h_mul_x
}

pub fn bit_decompose(params: &Parameters, bu: &Vec<Vec<u64>>) -> Vec<Vec<Vec<u64>>> {
    let ring = &params.ring;
    let ring_size = ring.ring_size();
    // Create a matrix of dimension m × m, where each element is a binary polynomial
    let mut tau = vec![vec![vec![ring.zero(); ring_size]; params.m]; params.m];

    // For each row h in the output matrix
    for h in 0..params.m {
        // For each column i in the output matrix
        for i in 0..params.m {
            // For each coefficient j in the polynomial
            for j in 0..ring_size {
                // Get the h-th bit of the j-th coefficient of the i-th polynomial
                let coeff = bu[i][j];
                // Check if the h-th bit is set
                let bit = (coeff >> h) & 1;
                tau[h][i][j] = bit;
            }
        }
    }
    tau
}

pub fn poly_add(ring: &PrimeRing, a: &Vec<u64>, b: &Vec<u64>) -> Vec<u64> {
    assert_eq!(a.len(), b.len());
    let mut c = vec![ring.zero(); a.len()];
    for i in 0..a.len() {
        let elem = ring.add(&a[i], &b[i]);
        c[i] = elem;
    }
    c
}

pub fn poly_sub(ring: &PrimeRing, a: &Vec<u64>, b: &Vec<u64>) -> Vec<u64> {
    assert_eq!(a.len(), b.len());
    let mut c = vec![ring.zero(); a.len()];
    for i in 0..a.len() {
        let elem = ring.sub(&a[i], &b[i]);
        c[i] = elem;
    }
    c
}

#[cfg(test)]
mod tests {
    use phantom_zone_math::prelude::ElemFrom;

    use super::*;
    use crate::BggRlwe;

    #[test]
    fn test_matrix_encoding_homomorphism_add_gate() {
        let bgg_rlwe = BggRlwe::new(12, 51, 4);
        let pub_key = &bgg_rlwe.public_key;
        let mut rng = thread_rng();
        let params = &bgg_rlwe.params;
        let ring = &params.ring;
        let mut x = (0..params.ell + 1)
            .map(|_| rng.gen_range(0..2))
            .collect::<Vec<_>>();
        x[0] = 1; // The actual attribute vector is x[1..], the value set to the index 0 is just for easier arithmetic during encoding

        let ciphertext = Ciphertext::new(&bgg_rlwe.public_key, &params, &x);
        let ct_inner = ciphertext.inner();

        // Perform plus gate of b[1] and b[2]
        let b_1_plus_2 = pub_key.add_gate(1, 2);

        // Perform plus gate of ct_inner[1] and ct_inner[2] to obtain the matrix h_1_plus_2_x
        let h_1_plus_2_x = m_eval_add_x(&params);

        // Verify homomorphism of plus gate such that (ct_inner[1] | ct_inner[2]) * h_1_plus_2_x = b_1_plus_2 + (x1+x2)G
        let mut lhs = vec![vec![params.ring.zero(); params.ring.ring_size()]; params.m];
        for i in 0..params.m {
            for j in 0..params.m {
                let mut scratch = ring.allocate_scratch(1, 2, 0);
                let mut scratch = scratch.borrow_mut();
                let product = ring.take_poly(&mut scratch);
                ring.poly_mul(
                    product,
                    &ct_inner[1][j],
                    &h_1_plus_2_x[j][i],
                    scratch.reborrow(),
                );
                lhs[i] = poly_add(ring, &lhs[i], &product.to_vec());

                let mut scratch = ring.allocate_scratch(1, 2, 0);
                let mut scratch = scratch.borrow_mut();
                let product_2 = ring.take_poly(&mut scratch);
                ring.poly_mul(
                    product_2,
                    &ct_inner[2][j],
                    &h_1_plus_2_x[j + params.m][i],
                    scratch.reborrow(),
                );
                lhs[i] = poly_add(ring, &lhs[i], &product_2.to_vec());
            }
        }

        let mut rhs = b_1_plus_2.clone();
        let mut fx = vec![ring.zero(); ring.ring_size()];
        fx[0] = ring.elem_from(x[1] + x[2]);

        for i in 0..params.m {
            let mut scratch = ring.allocate_scratch(1, 2, 0);
            let mut scratch = scratch.borrow_mut();
            let gi_times_fx = ring.take_poly(&mut scratch);
            ring.poly_mul(gi_times_fx, &params.g[i], &fx, scratch.reborrow());
            let gi_times_fx_vec = gi_times_fx.to_vec();
            rhs[i] = poly_add(ring, &rhs[i], &gi_times_fx_vec);
        }
        for i in 0..params.m {
            assert_eq!(lhs[i], rhs[i]);
        }
    }

    #[test]
    fn test_matrix_encoding_homomorphism_mul_gate() {
        let bgg_rlwe = BggRlwe::new(12, 51, 4);
        let pub_key = &bgg_rlwe.public_key;
        let mut rng = thread_rng();
        let params = &bgg_rlwe.params;
        let ring = &params.ring;
        let mut x = (0..params.ell + 1)
            .map(|_| rng.gen_range(0..2))
            .collect::<Vec<_>>();
        x[0] = 1; // The actual attribute vector is x[1..], the value set to the index 0 is just for easier arithmetic during encoding

        let ciphertext = Ciphertext::new(&bgg_rlwe.public_key, &params, &x);
        let ct_inner = ciphertext.inner();

        // Perform multiplication gate of b[1] and b[2]
        let b_1_times_2 = pub_key.mul_gate(1, 2);

        // Perform plus gate of ct_inner[1] and ct_inner[2] to obtain the matrix h_1_plus_2_x
        let h_1_times_2_x = m_eval_mul_x(
            &bgg_rlwe.params,
            &bgg_rlwe.public_key.b[1],
            x[2], // Pass x[2] as the scalar multiplier
        );

        // Verify homomorphism of multiplication gate such that (ct_inner[1] | ct_inner[2]) * h_1_times_2_x = b_1_times_2 + (x1*x2)G
        let mut lhs = vec![vec![params.ring.zero(); params.ring.ring_size()]; params.m];
        for i in 0..params.m {
            for j in 0..params.m {
                let mut scratch = ring.allocate_scratch(1, 2, 0);
                let mut scratch = scratch.borrow_mut();
                let product = ring.take_poly(&mut scratch);
                ring.poly_mul(
                    product,
                    &ct_inner[1][j],
                    &h_1_times_2_x[j][i],
                    scratch.reborrow(),
                );
                lhs[i] = poly_add(ring, &lhs[i], &product.to_vec());

                let mut scratch = ring.allocate_scratch(1, 2, 0);
                let mut scratch = scratch.borrow_mut();
                let product_2 = ring.take_poly(&mut scratch);
                ring.poly_mul(
                    product_2,
                    &ct_inner[2][j],
                    &h_1_times_2_x[j + params.m][i],
                    scratch.reborrow(),
                );
                lhs[i] = poly_add(ring, &lhs[i], &product_2.to_vec());
            }
        }

        let mut rhs = b_1_times_2.clone();
        let mut fx = vec![ring.zero(); ring.ring_size()];
        fx[0] = ring.elem_from(x[1] * x[2]);

        for i in 0..params.m {
            let mut scratch = ring.allocate_scratch(1, 2, 0);
            let mut scratch = scratch.borrow_mut();
            let gi_times_fx = ring.take_poly(&mut scratch);
            ring.poly_mul(gi_times_fx, &params.g[i], &fx, scratch.reborrow());
            let gi_times_fx_vec = gi_times_fx.to_vec();
            rhs[i] = poly_add(ring, &rhs[i], &gi_times_fx_vec);
        }
        for i in 0..params.m {
            assert_eq!(lhs[i], rhs[i]);
        }
    }

    #[test]
    fn test_bit_decompose() {
        let bgg_rlwe = BggRlwe::new(12, 51, 4);
        let bu = bgg_rlwe.public_key.b[1].clone();
        let ring = &bgg_rlwe.params.ring;
        let tau = bit_decompose(&bgg_rlwe.params, &bu);

        // Reconstruct the original input by multiplying tau with G
        let mut reconstructed = vec![vec![ring.zero(); ring.ring_size()]; bgg_rlwe.params.m];

        // For each column i of the output
        for i in 0..bgg_rlwe.params.m {
            // For each row h of tau
            for h in 0..bgg_rlwe.params.m {
                // Multiply tau[h][i] by g[h] and add to the result
                let mut scratch = ring.allocate_scratch(1, 2, 0);
                let mut scratch = scratch.borrow_mut();
                let product = ring.take_poly(&mut scratch);
                ring.poly_mul(
                    product,
                    &tau[h][i],
                    &bgg_rlwe.params.g[h],
                    scratch.reborrow(),
                );
                reconstructed[i] = poly_add(ring, &reconstructed[i], &product.to_vec());
            }
        }

        for i in 0..bgg_rlwe.params.m {
            assert_eq!(bu[i], reconstructed[i]);
        }
    }
}
