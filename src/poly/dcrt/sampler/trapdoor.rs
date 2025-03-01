use std::sync::Arc;

use crate::{
    poly::{
        dcrt::{DCRTPoly, DCRTPolyMatrix, DCRTPolyParams},
        sampler::PolyTrapdoorSampler,
        PolyMatrix, PolyParams,
    },
    utils::ceil_log2,
};

use openfhe::{
    cxx::UniquePtr,
    ffi::{DCRTPolyTrapdoorGen, RLWETrapdoorPair},
};

pub struct DCRTPolyTrapdoorSampler {
    params: DCRTPolyParams,
    base: usize,
}

impl DCRTPolyTrapdoorSampler {
    pub fn new(params: DCRTPolyParams, base: usize) -> Self {
        Self { params, base }
    }
}

impl PolyTrapdoorSampler for DCRTPolyTrapdoorSampler {
    type M = DCRTPolyMatrix;
    type Trapdoor = Arc<UniquePtr<RLWETrapdoorPair>>;

    fn trapdoor(&self) -> (Self::Trapdoor, Self::M) {
        let output = DCRTPolyTrapdoorGen(self.params.get_params(), self.base as i64, false);
        let trapdoor = output.GetTrapdoorPtr();
        let ncol = ceil_log2(&self.params.modulus()) + 2;

        let mut matrix_inner = Vec::with_capacity(1);
        let mut row = Vec::with_capacity(ncol);
        for i in 0..ncol {
            let poly = output.GetPolyAtIndex(i);
            let dcrt_poly = DCRTPoly::new(poly);
            row.push(dcrt_poly);
        }
        matrix_inner.push(row);
        let row_matrix = DCRTPolyMatrix::from_poly_vec(&self.params, matrix_inner);
        (trapdoor.into(), row_matrix)
    }

    fn preimage(&self, _trapdoor: &Self::Trapdoor, _target: &Self::M, _sigma: f64) -> Self::M {
        todo!()
        // let n_row = target.row_size();
        // let n_col = target.col_size();
        // let mut preimages = Vec::with_capacity(n_row);
        // for i in 0..n_row {
        //     let mut row_preimages = Vec::with_capacity(n_col);
        //     for j in 0..n_col {
        //         let target_poly = target.entry(i, j).clone();
        //         let preimage =
        //             DCRTPolyGaussSamp(12, 5, trapdoor.get_trapdoor(), &target_poly.get_poly(), 10);
        //         row_preimages.push(preimage);
        //     }
        //     preimages.push(row_preimages);
        // }
        // Self::M::from_poly_vec(&self.params, preimages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::poly::dcrt::DCRTPolyParams;

    #[test]
    fn test_trapdoor_generation() {
        let params = DCRTPolyParams::new(16, 4, 51);
        let base = 2;
        let sampler = DCRTPolyTrapdoorSampler::new(params, base);

        let (trapdoor, public_matrix) = sampler.trapdoor();

        assert!(!trapdoor.is_null());

        let expected_cols = ceil_log2(&sampler.params.modulus()) + 2;

        // Check dimensions of the public matrix
        assert_eq!(public_matrix.row_size(), 1, "Public matrix should have 1 row");
        assert_eq!(
            public_matrix.col_size(),
            expected_cols,
            "Public matrix should have ceil_log2(q) + 2 (m) columns"
        );

        // Verify that all entries in the matrix are valid DCRTPolys
        for i in 0..public_matrix.row_size() {
            for j in 0..public_matrix.col_size() {
                let poly = public_matrix.entry(i, j);
                assert!(!poly.get_poly().is_null(), "Matrix entry should be a valid DCRTPoly");
            }
        }
    }

    #[test]
    fn test_trapdoor_with_different_bases() {
        // Test with different base values
        let params = DCRTPolyParams::new(16, 4, 51);

        for base in [2, 3, 4, 8] {
            let sampler = DCRTPolyTrapdoorSampler { params: params.clone(), base };
            let (trapdoor, public_matrix) = sampler.trapdoor();

            // Check that the trapdoor is not null
            assert!(!trapdoor.is_null());

            // Calculate expected dimensions
            let expected_cols = ceil_log2(&params.modulus()) + 2;

            // Check dimensions of the public matrix
            assert_eq!(public_matrix.row_size(), 1);
            assert_eq!(public_matrix.col_size(), expected_cols);
        }
    }
}
