pub mod ciphertext;
pub mod eval;
pub mod operations;
pub mod parameters;
pub mod pub_key;
pub mod utils;

#[cfg(test)]
mod tests {
    use crate::{
        ciphertext::Ciphertext,
        eval::{m_eval_add, m_eval_add_x, m_eval_mul, m_eval_mul_x},
        operations::{poly_add, vec_mat_mul},
        parameters::Parameters,
        pub_key::PublicKey,
    };
    use phantom_zone_math::{
        prelude::{ElemFrom, ModulusOps},
        ring::RingOps,
    };
    use rand::{thread_rng, Rng};

    #[test]
    fn test_matrix_encoding_homomorphism_add_gate() {
        let params = Parameters::new(12, 51, 7);
        let pub_key = PublicKey::new(params);
        let mut rng = thread_rng();
        let ring = pub_key.params().ring();
        let m = *pub_key.params().m();
        let ell = *pub_key.params().ell();
        let g = pub_key.params().g();
        let mut x = (0..ell + 1)
            .map(|_| rng.gen_range(0..2))
            .collect::<Vec<_>>();
        x[0] = 1; // The actual attribute vector is x[1..], the value set to the index 0 is just for easier arithmetic during encoding

        let ciphertext = Ciphertext::new(&pub_key, &x);
        let ct_inner = ciphertext.inner();

        // Perform add gate of b[1] and b[2]
        let b_1_plus_2 = m_eval_add(&pub_key, 1, 2);

        // Perform add gate of ct_inner[1] and ct_inner[2]
        let h_1_plus_2_x = m_eval_add_x(&pub_key, &x, 1, 2);

        // Verify homomorphism of add gate such that (ct_inner[1] | ct_inner[2]) * h_1_plus_2_x = b_1_plus_2 + (x1+x2)G
        let concat_vec = [ct_inner[1].clone(), ct_inner[2].clone()].concat();
        let lhs = vec_mat_mul(ring, concat_vec, h_1_plus_2_x);

        let mut rhs = b_1_plus_2;
        let mut fx = vec![ring.zero(); ring.ring_size()];
        fx[0] = ring.elem_from(x[1] + x[2]);

        for i in 0..m {
            let mut scratch = ring.allocate_scratch(1, 2, 0);
            let mut scratch = scratch.borrow_mut();
            let gi_times_fx = ring.take_poly(&mut scratch);
            ring.poly_mul(gi_times_fx, &g[i], &fx, scratch.reborrow());
            let gi_times_fx_vec = gi_times_fx.to_vec();
            rhs[i] = poly_add(ring, &rhs[i], &gi_times_fx_vec);
        }
        for i in 0..m {
            assert_eq!(lhs[i], rhs[i]);
        }
    }

    #[test]
    fn test_matrix_encoding_homomorphism_mul_gate() {
        let params = Parameters::new(12, 51, 7);
        let pub_key = PublicKey::new(params);
        let mut rng = thread_rng();
        let ring = pub_key.params().ring();
        let m = *pub_key.params().m();
        let ell = *pub_key.params().ell();
        let g = pub_key.params().g();
        let mut x = (0..ell + 1)
            .map(|_| rng.gen_range(0..2))
            .collect::<Vec<_>>();
        x[0] = 1; // The actual attribute vector is x[1..], the value set to the index 0 is just for easier arithmetic during encoding

        let ciphertext = Ciphertext::new(&pub_key, &x);
        let ct_inner = ciphertext.inner();

        // Perform mul gate of b[1] and b[2]
        let b_1_times_2 = m_eval_mul(&pub_key, 1, 2);

        // Perform mul gate of ct_inner[1] and ct_inner[2]
        let h_1_times_2_x = m_eval_mul_x(&pub_key, &x, 1, 2);

        // Verify homomorphism of mul gate such that (ct_inner[1] | ct_inner[2]) * h_1_times_2_x = b_1_times_2 + (x1*x2)G
        let concat_vec = [ct_inner[1].clone(), ct_inner[2].clone()].concat();
        let lhs = vec_mat_mul(ring, concat_vec, h_1_times_2_x);

        let mut rhs = b_1_times_2.clone();
        let mut fx = vec![ring.zero(); ring.ring_size()];
        fx[0] = ring.elem_from(x[1] * x[2]);

        for i in 0..m {
            let mut scratch = ring.allocate_scratch(1, 2, 0);
            let mut scratch = scratch.borrow_mut();
            let gi_times_fx = ring.take_poly(&mut scratch);
            ring.poly_mul(gi_times_fx, &g[i], &fx, scratch.reborrow());
            let gi_times_fx_vec = gi_times_fx.to_vec();
            rhs[i] = poly_add(ring, &rhs[i], &gi_times_fx_vec);
        }
        for i in 0..m {
            assert_eq!(lhs[i], rhs[i]);
        }
    }
}
