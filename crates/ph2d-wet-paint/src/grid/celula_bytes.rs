//! **Um plano de células em bytes, ao bit**: cada componente e cada byte dele num plano contíguo, para um compressor ver
//! as mantissas ruidosas longe dos expoentes. A tinta adormecida em RAM e a viva no arquivo partem daqui.

use super::aparencia::AparenciaCelula;
use super::oleo_celula::OleoCelula;
use crate::material::MaterialCelula;

/// O mais de componentes que uma célula tem.
const MAX_COMPONENTES: usize = 16;

/// Uma célula é `N` componentes de 32 bits. O construtor sem `..` obriga um campo novo a entrar aqui.
pub trait Celula: Copy + Send {
    const N: usize;
    fn para(&self, f: &mut [u32]);
    fn de(f: &[u32]) -> Self;
}

impl Celula for f32 {
    const N: usize = 1;
    fn para(&self, f: &mut [u32]) {
        f[0] = self.to_bits();
    }
    fn de(f: &[u32]) -> Self {
        f32::from_bits(f[0])
    }
}

impl Celula for [f32; 3] {
    const N: usize = 3;
    fn para(&self, f: &mut [u32]) {
        for k in 0..3 {
            f[k] = self[k].to_bits();
        }
    }
    fn de(f: &[u32]) -> Self {
        [
            f32::from_bits(f[0]),
            f32::from_bits(f[1]),
            f32::from_bits(f[2]),
        ]
    }
}

macro_rules! celula_de_f32 {
    ($t:ty, $n:expr, [$($c:ident),*]) => {
        impl Celula for $t {
            const N: usize = $n;
            fn para(&self, f: &mut [u32]) {
                let mut k = 0;
                $(f[k] = self.$c.to_bits(); k += 1;)*
                debug_assert_eq!(k, Self::N);
            }
            fn de(f: &[u32]) -> Self {
                let mut k = 0;
                $(let $c = f32::from_bits(f[k]); k += 1;)*
                debug_assert_eq!(k, Self::N);
                Self { $($c),* }
            }
        }
    };
}

celula_de_f32!(
    MaterialCelula,
    13,
    [
        metal,
        rugosidade,
        nivel_ior,
        verniz,
        verniz_rugosidade,
        veludo,
        subsuperficie,
        raio_subsuperficie,
        transparencia,
        transmissao,
        relevo_papel,
        vidro_claro,
        amassa_papel
    ]
);
celula_de_f32!(
    AparenciaCelula,
    7,
    [papel, grao, vidrado, textura, opaca, suave, borda]
);

impl Celula for OleoCelula {
    const N: usize = 7;
    fn para(&self, f: &mut [u32]) {
        let OleoCelula {
            oleo,
            profundidade,
            brilho,
            soluvel,
            meio,
            corpo,
            pincel_de_oleo,
        } = *self;
        f[0] = oleo.to_bits();
        f[1] = profundidade.to_bits();
        f[2] = brilho.to_bits();
        f[3] = soluvel.to_bits();
        f[4] = meio.to_bits();
        f[5] = corpo.to_bits();
        f[6] = u32::from(pincel_de_oleo);
    }
    fn de(f: &[u32]) -> Self {
        OleoCelula {
            oleo: f32::from_bits(f[0]),
            profundidade: f32::from_bits(f[1]),
            brilho: f32::from_bits(f[2]),
            soluvel: f32::from_bits(f[3]),
            meio: f32::from_bits(f[4]),
            corpo: f32::from_bits(f[5]),
            pincel_de_oleo: f[6] != 0,
        }
    }
}

/// Os bytes de `v`, acrescentados a `saida`: `N × 4 × len` deles.
pub fn para_bytes<T: Celula>(v: &[T], saida: &mut Vec<u8>) {
    let n = v.len();
    let o = saida.len();
    saida.resize(o + n * T::N * 4, 0);
    let buf = &mut saida[o..];
    let mut f = [0u32; MAX_COMPONENTES];
    for (i, c) in v.iter().enumerate() {
        c.para(&mut f[..T::N]);
        for (k, palavra) in f[..T::N].iter().enumerate() {
            for (b, byte) in palavra.to_le_bytes().into_iter().enumerate() {
                buf[(k * 4 + b) * n + i] = byte;
            }
        }
    }
}

/// As células de volta de `buf`, os bytes de [`para_bytes`].
#[must_use]
pub fn de_bytes<T: Celula>(buf: &[u8]) -> Vec<T> {
    let n = buf.len() / (T::N * 4);
    let mut f = [0u32; MAX_COMPONENTES];
    (0..n)
        .map(|i| {
            for (k, palavra) in f[..T::N].iter_mut().enumerate() {
                let mut q = [0u8; 4];
                for (j, byte) in q.iter_mut().enumerate() {
                    *byte = buf[(k * 4 + j) * n + i];
                }
                *palavra = u32::from_le_bytes(q);
            }
            T::de(&f[..T::N])
        })
        .collect()
}
