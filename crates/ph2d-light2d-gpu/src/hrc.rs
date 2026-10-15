//! ⭐ **A geometria da cascata holográfica** (W5c; `c2_hrc.wgsl`) — CPU, pura: os
//! quadrantes, os níveis, as sondas e os bytes dos buffers. A MESMA conta dá a `PassList` (G5) e o
//! plano dos passes do solver.

use crate::passlist::Buffer;

/// O máximo de níveis (2^19 texels de lado passa qualquer textura).
pub(crate) const NIVEIS_MAX: usize = 20;

/// Os rótulos dos buffers dos intervalos T_n (o G5 acha-os pelo nome).
pub(crate) const NOMES_T: [&str; NIVEIS_MAX] = [
    "luz2d/buf/hrc_t0",
    "luz2d/buf/hrc_t1",
    "luz2d/buf/hrc_t2",
    "luz2d/buf/hrc_t3",
    "luz2d/buf/hrc_t4",
    "luz2d/buf/hrc_t5",
    "luz2d/buf/hrc_t6",
    "luz2d/buf/hrc_t7",
    "luz2d/buf/hrc_t8",
    "luz2d/buf/hrc_t9",
    "luz2d/buf/hrc_t10",
    "luz2d/buf/hrc_t11",
    "luz2d/buf/hrc_t12",
    "luz2d/buf/hrc_t13",
    "luz2d/buf/hrc_t14",
    "luz2d/buf/hrc_t15",
    "luz2d/buf/hrc_t16",
    "luz2d/buf/hrc_t17",
    "luz2d/buf/hrc_t18",
    "luz2d/buf/hrc_t19",
];
/// O par de radiâncias por cone (pingue-pongue entre níveis).
pub(crate) const NOMES_R: [&str; 2] = ["luz2d/buf/hrc_r0", "luz2d/buf/hrc_r1"];
pub(crate) const NOME_ACUMULADO: &str = "luz2d/buf/hrc_acumulado";
pub(crate) const NOME_UNIFORME: &str = "luz2d/buf/hrc_uniforme";
/// ⭐ W5r — o arco de cada cone, `arcos[2ⁿ + m]` para n = 0..=N (a placa escreve-o uma vez, ao nascer).
pub(crate) const NOME_ARCOS: &str = "luz2d/buf/hrc_arcos";
/// ⭐ W5s — um `u32` por ladrilho [`LADO_DO_VAZIO`]² da grade: 1 onde a captura tem cobertura (a traça
/// salta o intervalo que só cruza ladrilhos a 0).
pub(crate) const NOME_COM_COBERTURA: &str = "luz2d/buf/hrc_com_cobertura";
/// O `LADO_DO_VAZIO` do `c2_hrc.wgsl`.
pub(crate) const LADO_DO_VAZIO: u32 = 8;

/// ⭐ W5s — os ladrilhos da máscara do vazio na grade `w × h`.
pub(crate) fn ladrilhos([w, h]: [u32; 2]) -> u64 {
    u64::from(w.div_ceil(LADO_DO_VAZIO)) * u64::from(h.div_ceil(LADO_DO_VAZIO))
}
/// Os buffers que ocupam as ligações que um passe não usa (leitura; escrita de T; escrita de R).
pub(crate) const NOMES_VAZIOS: [&str; 4] = [
    "luz2d/buf/hrc_vazio_l",
    "luz2d/buf/hrc_vazio_t",
    "luz2d/buf/hrc_vazio_r",
    "luz2d/buf/hrc_vazio_a",
];

/// Bytes de uma entrada de T ou R (4 × f16).
const ENTRADA: u64 = 8;

/// Um quadrante no canónico: `(X ao longo de u, Y ao longo de v, N níveis)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Quadrante {
    pub x: u32,
    pub y: u32,
    pub n: u32,
}

impl Quadrante {
    /// O quadrante `q` (0 = +x, 1 = +y, 2 = −x, 3 = −y) da grade `w × h`: o topo é o menor `N`
    /// com `2^N ≥ X` (a cascata de cima passa a grade; o R dela é o céu), pelo menos 3 (T0–T2 são
    /// traçados).
    pub(crate) fn de(q: u32, [w, h]: [u32; 2]) -> Self {
        let (x, y) = if q.is_multiple_of(2) { (w, h) } else { (h, w) };
        let n = x.max(1).next_power_of_two().trailing_zeros().max(3);
        Self { x, y, n }
    }

    /// As sondas ao longo de u na cascata `n` DENTRO da grade (`xi·2ⁿ < X`): a de fora tem o traço
    /// no vácuo e o cone no céu — o shader responde-os sem os guardar.
    pub(crate) fn nx(&self, n: u32) -> u32 {
        self.x.div_ceil(1 << n)
    }

    /// Entradas de T_n.
    pub(crate) fn entradas_t(&self, n: u32) -> u64 {
        u64::from(self.nx(n)) * u64::from(self.y) * ((1u64 << n) + 1)
    }

    /// Entradas de R_n.
    pub(crate) fn entradas_r(&self, n: u32) -> u64 {
        u64::from(self.nx(n)) * u64::from(self.y) * (1u64 << n)
    }
}

/// Os buffers da cascata holográfica para a grade `w × h`: T_0..T_N (o maior dos dois eixos), o par
/// R, o acumulado, o uniforme (um por passe) e os vazios.
/// `rgb`: o acumulado com 3 f32 por texel (`Config::hrc_acumulado_rgb`), não 4.
pub(crate) fn buffers([w, h]: [u32; 2], passes: u64, rgb: bool) -> Vec<Buffer> {
    use wgpu::BufferUsages as B;
    let quads = [Quadrante::de(0, [w, h]), Quadrante::de(1, [w, h])];
    let n_max = quads.iter().map(|q| q.n).max().unwrap_or(3);
    let armazenado = B::STORAGE | B::COPY_DST;
    let mut v: Vec<Buffer> = (0..=n_max)
        .map(|n| Buffer {
            nome: NOMES_T[n as usize],
            bytes: ENTRADA
                * quads
                    .iter()
                    .filter(|q| n <= q.n)
                    .map(|q| q.entradas_t(n))
                    .max()
                    .unwrap_or(1),
            usos: armazenado,
        })
        .collect();
    let r = quads
        .iter()
        .flat_map(|q| (0..q.n).map(|n| q.entradas_r(n)))
        .max()
        .unwrap_or(1);
    v.extend(NOMES_R.map(|nome| Buffer {
        nome,
        bytes: ENTRADA * r,
        usos: armazenado,
    }));
    v.push(Buffer {
        nome: NOME_ACUMULADO,
        bytes: if rgb { 12 } else { 16 } * u64::from(w) * u64::from(h),
        usos: armazenado,
    });
    v.push(Buffer {
        nome: NOME_ARCOS,
        bytes: 4 << (n_max + 1),
        usos: armazenado,
    });
    v.push(Buffer {
        nome: NOME_COM_COBERTURA,
        bytes: 4 * ladrilhos([w, h]),
        usos: armazenado,
    });
    v.push(Buffer {
        nome: NOME_UNIFORME,
        bytes: crate::passlist::PASSO_UNIFORME * passes,
        usos: B::UNIFORM | B::COPY_DST,
    });
    v.extend(NOMES_VAZIOS.map(|nome| Buffer {
        nome,
        bytes: 16,
        usos: armazenado,
    }));
    v
}

/// Quantos passes (despachos) a cascata holográfica grava por quadro: os ladrilhos com cobertura (W5s);
/// por quadrante T0–T2, T3..TN, R_{N−1}..R0 e a soma; mais a saída.
pub(crate) fn n_passes([w, h]: [u32; 2]) -> u64 {
    (0..4)
        .map(|q| {
            let n = u64::from(Quadrante::de(q, [w, h]).n);
            3 + (n - 2) + n + 1
        })
        .sum::<u64>()
        + 2
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn o_topo_passa_a_grade() {
        let q = Quadrante::de(0, [600, 340]);
        assert_eq!((q.x, q.y, q.n), (600, 340, 10));
        assert_eq!(Quadrante::de(1, [600, 340]).n, 9);
        assert_eq!(Quadrante::de(0, [5, 5]).n, 3);
        // Só as sondas de dentro da grade se guardam; a do topo é uma.
        assert_eq!((q.nx(0), q.nx(1), q.nx(10)), (600, 300, 1));
    }
}
