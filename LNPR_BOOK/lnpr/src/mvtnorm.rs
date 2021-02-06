use ndarray::{Array1, Array2};
use ndarray_linalg::cholesky::*;
use ndarray_linalg::lapack::UPLO;
use ndarray_rand::rand_distr::StandardNormal;
use ndarray_rand::RandomExt;

pub fn mvtnorm<R: ndarray_rand::rand::RngCore>(
    rng: &mut R,
    mu: &Array1<f64>,
    cov: &Array2<f64>,
) -> Array1<f64> {
    let l = cov.cholesky(UPLO::Lower).unwrap();
    let x = Array1::<f64>::random_using(mu.len(), StandardNormal, rng);

    l.dot(&x) + mu
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::{arr2, Array1, Array2};
    use ndarray_rand;

    #[test]
    fn test_mvtnorm_f64() {
        let mu: Array1<f64> = Array1::from(vec![0.0, 1.0, 0.1]);
        let cov: Array2<f64> = arr2(&[[4., 12., -16.], [12., 37., -43.], [-16., -43., 98.]]);
        let mut rng = ndarray_rand::rand::prelude::thread_rng();
        let generated = mvtnorm(&mut rng, &mu, &cov);
        println!("{:?}", generated);
    }
}
