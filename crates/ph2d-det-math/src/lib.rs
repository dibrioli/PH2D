//! **A matemática do motor de pintura, a mesma em toda máquina** (Bug #58). Os métodos transcendentais de `f32`/`f64`
//! chamam a libm do sistema, cujo último bit muda com a versão da glibc e com a CPU. Aqui, cada um vai ao `libm`
//! fixado; `sqrt`, `floor` e `mul_add` não precisam disto (exatos); o `powi` tem `det_powi` (B43.1).

/// Os transcendentais do motor. O gate `o_motor_nao_chama_a_libm_do_sistema` recusa o método de `std` nas crates dele.
pub trait DetMath: Sized + Copy {
    #[must_use]
    fn det_sin(self) -> Self;
    #[must_use]
    fn det_cos(self) -> Self;
    #[must_use]
    fn det_tan(self) -> Self;
    #[must_use]
    fn det_atan(self) -> Self;
    #[must_use]
    fn det_atan2(self, x: Self) -> Self;
    #[must_use]
    fn det_sin_cos(self) -> (Self, Self);
    #[must_use]
    fn det_exp(self) -> Self;
    #[must_use]
    fn det_ln(self) -> Self;
    #[must_use]
    fn det_log10(self) -> Self;
    #[must_use]
    fn det_powf(self, e: Self) -> Self;
    #[must_use]
    fn det_cbrt(self) -> Self;
    #[must_use]
    fn det_hypot(self, o: Self) -> Self;
    #[must_use]
    fn det_exp2(self) -> Self;
    #[must_use]
    fn det_log2(self) -> Self;
    /// Quadrados em ordem fixa (a do `__powisf2`): o `powi` de `std` é dobrado pelo LLVM em compilação com outro bit (#65088).
    #[must_use]
    fn det_powi(self, n: i32) -> Self;
}

impl DetMath for f32 {
    #[inline]
    fn det_sin(self) -> Self {
        libm::sinf(self)
    }
    #[inline]
    fn det_cos(self) -> Self {
        libm::cosf(self)
    }
    #[inline]
    fn det_tan(self) -> Self {
        libm::tanf(self)
    }
    #[inline]
    fn det_atan(self) -> Self {
        libm::atanf(self)
    }
    #[inline]
    fn det_atan2(self, x: Self) -> Self {
        libm::atan2f(self, x)
    }
    #[inline]
    fn det_sin_cos(self) -> (Self, Self) {
        libm::sincosf(self)
    }
    #[inline]
    fn det_exp(self) -> Self {
        libm::expf(self)
    }
    #[inline]
    fn det_ln(self) -> Self {
        libm::logf(self)
    }
    #[inline]
    fn det_log10(self) -> Self {
        libm::log10f(self)
    }
    #[inline]
    fn det_powf(self, e: Self) -> Self {
        libm::powf(self, e)
    }
    #[inline]
    fn det_cbrt(self) -> Self {
        libm::cbrtf(self)
    }
    #[inline]
    fn det_hypot(self, o: Self) -> Self {
        libm::hypotf(self, o)
    }
    #[inline]
    fn det_exp2(self) -> Self {
        libm::exp2f(self)
    }
    #[inline]
    fn det_log2(self) -> Self {
        libm::log2f(self)
    }
    #[inline]
    fn det_powi(self, n: i32) -> Self {
        let mut a = self;
        let mut b = n.unsigned_abs();
        let mut r = 1.0;
        loop {
            if b & 1 != 0 {
                r *= a;
            }
            b >>= 1;
            if b == 0 {
                break;
            }
            a *= a;
        }
        if n < 0 { 1.0 / r } else { r }
    }
}

impl DetMath for f64 {
    #[inline]
    fn det_sin(self) -> Self {
        libm::sin(self)
    }
    #[inline]
    fn det_cos(self) -> Self {
        libm::cos(self)
    }
    #[inline]
    fn det_tan(self) -> Self {
        libm::tan(self)
    }
    #[inline]
    fn det_atan(self) -> Self {
        libm::atan(self)
    }
    #[inline]
    fn det_atan2(self, x: Self) -> Self {
        libm::atan2(self, x)
    }
    #[inline]
    fn det_sin_cos(self) -> (Self, Self) {
        libm::sincos(self)
    }
    #[inline]
    fn det_exp(self) -> Self {
        libm::exp(self)
    }
    #[inline]
    fn det_ln(self) -> Self {
        libm::log(self)
    }
    #[inline]
    fn det_log10(self) -> Self {
        libm::log10(self)
    }
    #[inline]
    fn det_powf(self, e: Self) -> Self {
        libm::pow(self, e)
    }
    #[inline]
    fn det_cbrt(self) -> Self {
        libm::cbrt(self)
    }
    #[inline]
    fn det_hypot(self, o: Self) -> Self {
        libm::hypot(self, o)
    }
    #[inline]
    fn det_exp2(self) -> Self {
        libm::exp2(self)
    }
    #[inline]
    fn det_log2(self) -> Self {
        libm::log2(self)
    }
    #[inline]
    fn det_powi(self, n: i32) -> Self {
        let mut a = self;
        let mut b = n.unsigned_abs();
        let mut r = 1.0;
        loop {
            if b & 1 != 0 {
                r *= a;
            }
            b >>= 1;
            if b == 0 {
                break;
            }
            a *= a;
        }
        if n < 0 { 1.0 / r } else { r }
    }
}
