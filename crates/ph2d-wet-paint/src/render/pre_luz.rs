//! [`PreLuz`]: o que a linha do óleo tem de uma célula antes da luz (filho de [`super`] por LOC).

use crate::jsmath::clamp_u8;

/// O que a linha do óleo tem de UMA célula antes da luz (A.1, [`crate::oleo_vista`]): a cor pré-multiplicada da pilha,
/// a cobertura antes (`pa_sem`) e depois da orla (`pa`), a do filme (`tinta`), o emboss da parte sem óleo (`pe`) e a
/// fracção com óleo (`w`). A vista interpola-o e fecha cada pixel com [`PreLuz::fecha_oleo`], a conta da linha.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PreLuz {
    pub pr: f64,
    pub pg: f64,
    pub pb: f64,
    pub pa_sem: f64,
    pub pa: f64,
    pub tinta: f64,
    pub pe: f64,
    pub w: f64,
}

impl PreLuz {
    /// O pixel RGBA da tinta com óleo de cor iluminada `[r, g, b]` (sRGB 0..255): a mistura com a parte sem óleo.
    #[must_use]
    pub fn fecha_oleo(&self, [r, g, b]: [f64; 3]) -> [u8; 4] {
        let PreLuz {
            pr,
            pg,
            pb,
            pa_sem,
            pa,
            pe,
            w: w_oleo,
            ..
        } = *self;
        if w_oleo < 1.0 {
            // A mistura com a tinta SEM óleo (a de baixo da fronteira, ou molhada com ela): a cor e a
            // cobertura de cada uma pela fracção.
            let (q, sem) = (1.0 - w_oleo, |c: f64| (c + pe) / pa_sem);
            [
                clamp_u8(w_oleo * r + q * sem(pr)),
                clamp_u8(w_oleo * g + q * sem(pg)),
                clamp_u8(w_oleo * b + q * sem(pb)),
                clamp_u8((w_oleo * pa + q * pa_sem) * 255.0),
            ]
        } else {
            [clamp_u8(r), clamp_u8(g), clamp_u8(b), clamp_u8(pa * 255.0)]
        }
    }

    /// O pixel RGBA da tinta SEM óleo (o ramo `pa > 0` da linha), ou transparente.
    #[must_use]
    pub fn fecha_sem_oleo(&self) -> [u8; 4] {
        let pa = self.pa_sem;
        if pa > 0.0 {
            [
                clamp_u8(self.pr / pa),
                clamp_u8(self.pg / pa),
                clamp_u8(self.pb / pa),
                clamp_u8(pa * 255.0),
            ]
        } else {
            [0; 4]
        }
    }
}
