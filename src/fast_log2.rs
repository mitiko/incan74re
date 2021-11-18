use std::arch::x86_64::*;

pub struct Log2Globals {
    pub double_exp_mask: __m256i,
    pub double_exp0: __m256i,
    pub to32bit_exp: __m256i,
    pub exp_normalizer: __m128i,
    pub comm_mul: __m256d,
    pub coeff1: __m256d,
    pub coeff2: __m256d,
    pub coeff3: __m256d,
    pub coeff4: __m256d,
    pub vect1: __m256d
}

impl Log2Globals {
    pub fn new() -> Log2Globals {
        Log2Globals {
            double_exp_mask: unsafe { _mm256_set1_epi64x(0x7ff << 52) },
            double_exp0: unsafe { _mm256_set1_epi64x(1023 << 52) },
            to32bit_exp: unsafe { _mm256_set_epi32(0, 0, 0, 0, 6, 4, 2, 0) },
            exp_normalizer: unsafe { _mm_set1_epi32(1023) },
            comm_mul: unsafe { _mm256_set1_pd(2.0 / (2.0f64).ln()) },
            coeff1: unsafe { _mm256_set1_pd(1.0 / 3.0) },
            coeff2: unsafe { _mm256_set1_pd(1.0 / 5.0) },
            coeff3: unsafe { _mm256_set1_pd(1.0 / 7.0) },
            coeff4: unsafe { _mm256_set1_pd(1.0 / 9.0) },
            vect1: unsafe { _mm256_set1_pd(1.0) }
        }
    }
}

// AVX2 computation of x log2(x)
// Based on https://www.coder.work/article/6462830
pub fn entropy(a: i32, b: i32, c: i32, d: i32, g: &Log2Globals) -> [f64; 4] {
    unsafe {
        // TODO: Are those loads optimal?
        let x_pd = _mm256_loadu_pd([a as f64, b as f64, c as f64, d as f64].as_ptr());
        let x = _mm256_castpd_si256(x_pd);

        let m = _mm256_and_si256(g.double_exp_mask, x);
        let exps64 = _mm256_srli_epi64::<52>(m);
        let exps32_avx = _mm256_permutevar8x32_epi32(exps64, g.to32bit_exp);
        let exps32_sse = _mm256_castsi256_si128(exps32_avx);
        let norm_exps = _mm_sub_epi32(exps32_sse, g.exp_normalizer);
        let exps_pd = _mm256_cvtepi32_pd(norm_exps);
        let y = _mm256_or_pd(_mm256_castsi256_pd(g.double_exp0),
            _mm256_andnot_pd(_mm256_castsi256_pd(g.double_exp_mask), x_pd));

        // Calculate t=(y-1)/(y+1) and t**2
        let t_num = _mm256_sub_pd(y, g.vect1);
        let t_den = _mm256_add_pd(y, g.vect1);
        let t = _mm256_div_pd(t_num, t_den);
        let t2 = _mm256_mul_pd(t, t); // t**2

        let t3 = _mm256_mul_pd(t, t2); // t**3
        let terms01 = _mm256_fmadd_pd(g.coeff1, t3, t);
        let t5 = _mm256_mul_pd(t3, t2); // t**5
        let terms012 = _mm256_fmadd_pd(g.coeff2, t5, terms01);
        let t7 = _mm256_mul_pd(t5, t2); // t**7
        let terms0123 = _mm256_fmadd_pd(g.coeff3, t7, terms012);
        let t9 = _mm256_mul_pd(t7, t2); // t**9
        let terms01234 = _mm256_fmadd_pd(g.coeff4, t9, terms0123);

        let log2_y = _mm256_mul_pd(terms01234, g.comm_mul);
        let log2_x = _mm256_add_pd(log2_y, exps_pd);
        let entropy = _mm256_mul_pd(log2_x, x_pd);
        let mut result = [0f64; 4];
        _mm256_storeu_pd(result.as_mut_ptr(), entropy);
        return result;
    }
}