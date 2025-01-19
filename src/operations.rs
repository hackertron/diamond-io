use crate::Parameters;
use phantom_zone_math::{
    prelude::ModulusOps,
    ring::{PrimeRing, RingOps},
};

pub fn bit_decompose(params: &Parameters, bu: &Vec<Vec<u64>>) -> Vec<Vec<Vec<u64>>> {
    let ring = params.ring();
    let m = *params.m();
    let ring_size = ring.ring_size();
    // Create a matrix of dimension m × m, where each element is a binary polynomial
    let mut tau = vec![vec![vec![ring.zero(); ring_size]; m]; m];

    // For each row h in the output matrix
    for h in 0..m {
        // For each column i in the output matrix
        for i in 0..m {
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

pub fn vec_vec_dot_product(
    ring: &PrimeRing,
    vec_a: &Vec<Vec<u64>>,
    vec_b: &Vec<Vec<u64>>,
) -> Vec<u64> {
    assert_eq!(
        vec_a.len(),
        vec_b.len(),
        "Vectors must have the same length"
    );
    let mut out = vec![ring.zero(); ring.ring_size()];
    for i in 0..vec_a.len() {
        let mut scratch = ring.allocate_scratch(1, 2, 0);
        let mut scratch = scratch.borrow_mut();
        let product = ring.take_poly(&mut scratch);
        ring.poly_mul(product, &vec_a[i], &vec_b[i], scratch.reborrow());
        out = poly_add(ring, &out, &product.to_vec());
    }
    out
}

pub fn vec_mat_mul(ring: &PrimeRing, vec: Vec<Vec<u64>>, mat: Vec<Vec<Vec<u64>>>) -> Vec<Vec<u64>> {
    let len = vec.len();
    let mat_rows = mat.len();
    let mat_cols = mat[0].len();
    assert_eq!(len, mat_rows);
    let mut out = vec![vec![ring.zero(); ring.ring_size()]; mat_cols];

    for i in 0..mat_cols {
        let col_i = mat.iter().map(|row| row[i].clone()).collect::<Vec<_>>();
        out[i] = vec_vec_dot_product(ring, &vec, &col_i);
    }
    out
}

#[cfg(test)]
mod tests {
    use phantom_zone_math::{prelude::ModulusOps, ring::RingOps};

    use crate::{bit_decompose, poly_add, Parameters, PublicKey};

    #[test]
    fn test_bit_decompose() {
        let params = Parameters::new(12, 51, 4);
        let pub_key = PublicKey::new(params);
        let b1 = &pub_key.b()[1];
        let ring = pub_key.params().ring();
        let m = *pub_key.params().m();
        let g = pub_key.params().g();
        let tau = bit_decompose(pub_key.params(), b1);

        // Reconstruct the original input by multiplying tau with G
        let mut reconstructed = vec![vec![ring.zero(); ring.ring_size()]; m];

        // For each column i of the output
        for i in 0..m {
            // For each row h of tau
            for h in 0..m {
                // Multiply tau[h][i] by g[h] and add to the result
                let mut scratch = ring.allocate_scratch(1, 2, 0);
                let mut scratch = scratch.borrow_mut();
                let product = ring.take_poly(&mut scratch);
                ring.poly_mul(product, &tau[h][i], &g[h], scratch.reborrow());
                reconstructed[i] = poly_add(ring, &reconstructed[i], &product.to_vec());
            }
        }

        for i in 0..m {
            assert_eq!(b1[i], reconstructed[i]);
        }
    }
}
