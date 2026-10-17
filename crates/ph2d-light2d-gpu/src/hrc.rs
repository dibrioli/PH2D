//! ⭐ **A geometria da cascata holográfica** (W5c; `c2_hrc.wgsl`) — CPU, pura: os
//! quadrantes, os níveis, as sondas e os bytes dos buffers. A MESMA conta dá a `PassList` (G5) e o
//! plano dos passes do solver.

use crate::passlist::{Buffer, Textura};

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
pub(crate) const NOMES_VAZIOS: [&str; 3] = [
    "luz2d/buf/hrc_vazio_l",
    "luz2d/buf/hrc_vazio_t",
    "luz2d/buf/hrc_vazio_r",
];

/// Bytes de uma entrada de T ou R (4 × f16).
const ENTRADA: u64 = 8;

/// ⭐ W5z — a largura (texels) das texturas dos T e dos R: o lado que todo dispositivo WebGPU dá, em
/// ladrilhos de 8 × 8 (o `LADRILHOS_POR_LINHA_HRC` do `hrc_niveis_textura.wgsl` × 8).
pub(crate) const LARGURA_DOS_NIVEIS: u32 = 8192;
const _: () = assert!(LARGURA_DOS_NIVEIS == 1024 * 8);
/// ⭐ W5z — as entradas que cabem numa textura de nível (`LARGURA_DOS_NIVEIS²`: a altura também é o lado
/// garantido).
const ENTRADAS_POR_TEXTURA: u64 = LARGURA_DOS_NIVEIS as u64 * LARGURA_DOS_NIVEIS as u64;

/// ⭐ W5u — a grade (texels) a partir da qual a holográfica faz `Config::hrc_cones_por_fio` cones e
/// `Config::hrc_pares_na_subida` pares por fio. O recurso é o PARALELISMO da placa: numa grade pequena,
/// juntar entradas deixa fios a menos. Medido (RTX 5060 Ti, `c2` por processo, ms, 1 entrada por fio · 2/2
/// · 4/4): 256×160 0,229 · 0,239 · 0,275; 512×320 0,451 · 0,436 · 0,443; 640×384 0,522 · — · 0,500;
/// 992×576 1,407 · 1,371 · 1,367; 2496×1472 7,32 · — · 6,04 (handoff W5u).
pub(crate) const POR_FIO_DESDE: u64 = 1 << 17;

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

/// Os T_0..T_N (o maior dos dois eixos) e o par R da grade `w × h`: `(nome, entradas)`.
fn niveis([w, h]: [u32; 2]) -> Vec<(&'static str, u64)> {
    let quads = [Quadrante::de(0, [w, h]), Quadrante::de(1, [w, h])];
    let n_max = quads.iter().map(|q| q.n).max().unwrap_or(3);
    let r = quads
        .iter()
        .flat_map(|q| (0..q.n).map(|n| q.entradas_r(n)))
        .max()
        .unwrap_or(1);
    (0..=n_max)
        .map(|n| {
            let t = quads
                .iter()
                .filter(|q| n <= q.n)
                .map(|q| q.entradas_t(n))
                .max()
                .unwrap_or(1);
            (NOMES_T[n as usize], t)
        })
        .chain(NOMES_R.map(|nome| (nome, r)))
        .collect()
}

/// ⭐ W5z — os T e os R da grade `w × h` cabem em texturas de nível?
pub(crate) fn niveis_cabem_em_textura(grade: [u32; 2]) -> bool {
    niveis(grade)
        .iter()
        .all(|&(_, e)| e <= ENTRADAS_POR_TEXTURA)
}

/// ⭐ W5z — os T, o par R e os vazios em TEXTURAS `Rg32Uint` (com os rótulos dos buffers que substituem):
/// [`LARGURA_DOS_NIVEIS`] de largura, a altura em ladrilhos de 8 linhas; os vazios 8 × 8.
pub(crate) fn texturas(grade: [u32; 2]) -> Vec<Textura> {
    use wgpu::TextureUsages as T;
    let textura = |nome, tamanho| Textura {
        nome,
        formato: wgpu::TextureFormat::Rg32Uint,
        tamanho,
        usos: T::STORAGE_BINDING | T::TEXTURE_BINDING,
        mips: 1,
    };
    let por_ladrilho = 8 * u64::from(LARGURA_DOS_NIVEIS);
    niveis(grade)
        .into_iter()
        .map(|(nome, e)| {
            let altura = 8 * e.div_ceil(por_ladrilho).max(1);
            textura(nome, [LARGURA_DOS_NIVEIS, altura as u32])
        })
        .chain(NOMES_VAZIOS.map(|nome| textura(nome, [8, 8])))
        .collect()
}

/// Os buffers da cascata holográfica para a grade `w × h`: o acumulado, os arcos, a cobertura, o
/// uniforme (um por passe) e — sem `niveis_em_textura` ([`texturas`]) — os T, o par R e os vazios.
pub(crate) fn buffers([w, h]: [u32; 2], passes: u64, niveis_em_textura: bool) -> Vec<Buffer> {
    use wgpu::BufferUsages as B;
    let n_max = Quadrante::de(0, [w, h]).n.max(Quadrante::de(1, [w, h]).n);
    let armazenado = B::STORAGE | B::COPY_DST;
    let mut v: Vec<Buffer> = if niveis_em_textura {
        Vec::new()
    } else {
        niveis([w, h])
            .into_iter()
            .map(|(nome, e)| Buffer {
                nome,
                bytes: ENTRADA * e,
                usos: armazenado,
            })
            .collect()
    };
    v.push(Buffer {
        nome: NOME_ACUMULADO,
        bytes: 16 * u64::from(w) * u64::from(h),
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
    if !niveis_em_textura {
        v.extend(NOMES_VAZIOS.map(|nome| Buffer {
            nome,
            bytes: 16,
            usos: armazenado,
        }));
    }
    v
}

impl crate::Config {
    /// ⭐ W5z — os T e os R da holográfica vão em textura: há holográfica, [`Self::hrc_em_texturas`],
    /// cabem ([`LARGURA_DOS_NIVEIS`]² entradas por nível) e o módulo por cima ([`Self::modulo_rc`], das
    /// réguas) traz o prelúdio de textura — um módulo de antes da W5z liga buffers.
    #[must_use]
    pub fn hrc_niveis_em_textura(&self) -> bool {
        let g = self.grade_efetiva();
        self.holografica.is_some()
            && self.hrc_em_texturas
            && niveis_cabem_em_textura([g.largura, g.altura])
            && self
                .modulo_rc
                .is_none_or(|m| m.contains("fn texel_do_nivel("))
    }
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
