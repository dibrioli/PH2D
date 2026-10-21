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

/// ⭐ §44 — o alinhamento de cada faixa de quadrante (um ladrilho 8 × 8 do `hrc_niveis_textura.wgsl`).
const ALINHA_FAIXA: u64 = 64;

/// ⭐ §44 (handoff W6g) — com os quadrantes JUNTOS ([`crate::Config::hrc_juntos`]) cada um tem a sua faixa em
/// cada nível: `t[q][n]` é o começo da do T_n do quadrante `q` e `r[q]` o da dele em cada R do par (as `bases`
/// do `Hrc`, somadas ao índice pelos acessores dos níveis).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Faixas {
    pub t: [[u32; NIVEIS_MAX]; 4],
    pub r: [u32; 4],
}

/// As faixas da grade `w × h` e as entradas de cada T_n (`0..=N`) e de cada R com elas.
pub(crate) fn faixas([w, h]: [u32; 2]) -> (Faixas, Vec<u64>, u64) {
    let quads = [0, 1, 2, 3].map(|q| Quadrante::de(q, [w, h]));
    let n_max = quads.iter().map(|q| q.n).max().unwrap_or(3);
    let mut f = Faixas {
        t: [[0; NIVEIS_MAX]; 4],
        r: [0; 4],
    };
    let t = (0..=n_max)
        .map(|n| {
            let mut fim = 0u64;
            for (q, quad) in quads.iter().enumerate() {
                f.t[q][n as usize] = fim as u32;
                if n <= quad.n {
                    fim += quad.entradas_t(n).next_multiple_of(ALINHA_FAIXA);
                }
            }
            fim.max(1)
        })
        .collect();
    let mut r = 0u64;
    for (q, quad) in quads.iter().enumerate() {
        f.r[q] = r as u32;
        r += (0..quad.n).map(|n| quad.entradas_r(n)).max().unwrap_or(1).next_multiple_of(ALINHA_FAIXA);
    }
    (f, t, r)
}

/// Os T_0..T_N (o maior dos dois eixos) e o par R da grade `w × h`: `(nome, entradas)`; ⭐ §44 `juntos` = as
/// faixas dos quatro quadrantes ([`faixas`]).
fn niveis([w, h]: [u32; 2], juntos: bool) -> Vec<(&'static str, u64)> {
    if juntos {
        let (_, t, r) = faixas([w, h]);
        return t
            .into_iter()
            .enumerate()
            .map(|(n, e)| (NOMES_T[n], e))
            .chain(NOMES_R.map(|nome| (nome, r)))
            .collect();
    }
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
pub(crate) fn niveis_cabem_em_textura(grade: [u32; 2], juntos: bool) -> bool {
    niveis(grade, juntos)
        .iter()
        .all(|&(_, e)| e <= ENTRADAS_POR_TEXTURA)
}

/// ⭐ W5z — os T, o par R e os vazios em TEXTURAS `Rg32Uint` (com os rótulos dos buffers que substituem):
/// [`LARGURA_DOS_NIVEIS`] de largura, a altura em ladrilhos de 8 linhas; os vazios 8 × 8.
pub(crate) fn texturas(grade: [u32; 2], juntos: bool) -> Vec<Textura> {
    use wgpu::TextureUsages as T;
    let textura = |nome, tamanho| Textura {
        nome,
        formato: wgpu::TextureFormat::Rg32Uint,
        tamanho,
        usos: T::STORAGE_BINDING | T::TEXTURE_BINDING,
        mips: 1,
    };
    let por_ladrilho = 8 * u64::from(LARGURA_DOS_NIVEIS);
    niveis(grade, juntos)
        .into_iter()
        .map(|(nome, e)| {
            let altura = 8 * e.div_ceil(por_ladrilho).max(1);
            textura(nome, [LARGURA_DOS_NIVEIS, altura as u32])
        })
        .chain(NOMES_VAZIOS.map(|nome| textura(nome, [8, 8])))
        .collect()
}

/// ⭐ bug 6 (handoff W6g §32) — os R de todos os níveis de um ENTORNO, que a vista lê pela borda.
pub(crate) const NOME_ARQUIVO: &str = "luz2d/buf/hrc_arquivo";

/// ⭐ bug 6 — a JANELA do arquivo no quadrante (canónico do entorno, texels: `[U0, U1, V0, V1]`, fins
/// exclusivos): as sondas sob a grade da vista; a MESMA conta do `indice_na_janela` do `c2_hrc.wgsl`.
pub(crate) type Janela = [u32; 4];

/// As entradas do nível `m` na janela `j` (as colunas arredondadas para fora, como o `indice_na_janela`).
fn entradas_na_janela(j: Janela, m: u32) -> u64 {
    let k0 = j[0] >> m;
    let colunas = j[1].div_ceil(1 << m) + 1 - k0;
    u64::from(colunas) * u64::from(j[3] - j[2]) * (1u64 << m)
}

/// ⭐ bug 6 — a base de cada nível `0..N` de cada quadrante no arquivo (as janelas na ordem dos passes) e o
/// total de entradas.
pub(crate) fn bases_na_janela(
    grade: [u32; 2],
    janelas: [Janela; 4],
) -> ([[u32; NIVEIS_MAX]; 4], u64) {
    let mut bases = [[0u32; NIVEIS_MAX]; 4];
    let mut total = 0u64;
    for (q, b) in bases.iter_mut().enumerate() {
        for m in 0..Quadrante::de(q as u32, grade).n {
            b[m as usize] = total as u32;
            total += entradas_na_janela(janelas[q], m);
        }
    }
    (bases, total)
}

/// ⭐ bug 6 — as entradas do arquivo para uma janela de no máximo `[w, h]` texels (da grade do entorno) em
/// qualquer posição: o pior alinhamento das colunas de cada nível.
pub(crate) fn entradas_do_arquivo(grade: [u32; 2], [w, h]: [u32; 2]) -> u64 {
    (0..4u32)
        .map(|q| {
            let (lu, lv) = if q % 2 == 0 { (w, h) } else { (h, w) };
            (0..Quadrante::de(q, grade).n)
                .map(|m| (u64::from(lu.div_ceil(1 << m)) + 2) * u64::from(lv) * (1u64 << m))
                .sum::<u64>()
        })
        .sum()
}

/// ⭐ bug 6 — a janela de cada quadrante para o miolo `[x0, y0, x1, y1]` (texels da grade `w × h`): o
/// retângulo no canónico, uma linha a mais de cada lado (a bilinear do `de_fora`), dentro da grade, com
/// no máximo `max` texels de lado.
pub(crate) fn janelas_do_miolo([w, h]: [u32; 2], miolo: [f32; 4], max: [u32; 2]) -> [Janela; 4] {
    std::array::from_fn(|q| {
        let (gw, gh) = (w as f32, h as f32);
        let canonico = |x: f32, y: f32| match q {
            1 => (y, gw - x),
            2 => (gw - x, gh - y),
            3 => (gh - y, x),
            _ => (x, y),
        };
        let (a, b) = (canonico(miolo[0], miolo[1]), canonico(miolo[2], miolo[3]));
        let (lu, lv) = if q % 2 == 0 { (w, h) } else { (h, w) };
        let (mu, mv) = if q % 2 == 0 {
            (max[0], max[1])
        } else {
            (max[1], max[0])
        };
        let faixa = |p: f32, r: f32, lado: u32, max: u32| {
            let i0 = (p.min(r).floor() as i64 - 1).clamp(0, i64::from(lado)) as u32;
            let i1 = (p.max(r).ceil() as i64 + 1).clamp(0, i64::from(lado)) as u32;
            (i0, i1.min(i0 + max))
        };
        let (u0, u1) = faixa(a.0, b.0, lu, mu);
        let (v0, v1) = faixa(a.1, b.1, lv, mv);
        [u0, u1, v0, v1]
    })
}

/// Os buffers da cascata holográfica para a grade `w × h`: o acumulado, os arcos, a cobertura, o
/// uniforme (um por passe), o arquivo dos R (⭐ bug 6: só o ENTORNO o enche; os outros, 16 bytes) e — sem
/// `niveis_em_textura` ([`texturas`]) — os T, o par R e os vazios.
pub(crate) fn buffers(
    [w, h]: [u32; 2],
    passes: u64,
    niveis_em_textura: bool,
    juntos: bool,
    arquivo_da_borda: Option<[u32; 2]>,
) -> Vec<Buffer> {
    use wgpu::BufferUsages as B;
    let n_max = Quadrante::de(0, [w, h]).n.max(Quadrante::de(1, [w, h]).n);
    let armazenado = B::STORAGE | B::COPY_DST;
    let mut v: Vec<Buffer> = if niveis_em_textura {
        Vec::new()
    } else {
        niveis([w, h], juntos)
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
    v.push(Buffer {
        nome: NOME_ARQUIVO,
        bytes: match arquivo_da_borda {
            Some(max) => ENTRADA * entradas_do_arquivo([w, h], max),
            None => 16,
        },
        usos: armazenado,
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
            && niveis_cabem_em_textura([g.largura, g.altura], self.hrc_juntos())
            && self
                .modulo_rc
                .is_none_or(|m| m.contains("fn texel_do_nivel("))
    }

    /// ⭐ §44 — os quadrantes JUNTOS ([`Self::hrc_quadrantes_juntos`]): só com os despachos em 3D e um fio por
    /// sonda na traça (o despacho junto decompõe grupos `[⌈Y/64⌉, nx, k]`; o 1D dobrado não tem quadrante).
    #[must_use]
    pub fn hrc_juntos(&self) -> bool {
        self.hrc_quadrantes_juntos
            && self.holografica.is_some()
            && self.hrc_grade_3d
            && self.hrc_por_colunas
            && self.hrc_fio_por_sonda
    }
}

/// Quantas fatias do uniforme a cascata holográfica usa: uma por passe — os ladrilhos com cobertura (W5s);
/// por quadrante T0–T2, T3..TN, R_{N−1}..R0 e a soma; mais a saída. ⭐ bug 6: um ENTORNO que arquiva
/// (`arquivo_da_borda`) grava de novo, por quadrante, os T e os R (a 2.ª passagem, sem soma). ⭐ §44: com os
/// quadrantes `juntos`, quatro por etapa (uma por quadrante, até o nível mais alto dos quatro); e a ligação
/// de todo passe lê quatro fatias (o `array<Hrc, 4>`), logo mais três no fim.
pub(crate) fn n_passes([w, h]: [u32; 2], arquivo_da_borda: bool, juntos: bool) -> u64 {
    let rodadas = 1 + u64::from(arquivo_da_borda);
    let por_quadrante = if juntos {
        let n = u64::from(Quadrante::de(0, [w, h]).n.max(Quadrante::de(1, [w, h]).n));
        4 * (2 * n + 1) * rodadas + 4
    } else {
        (0..4)
            .map(|q| {
                let n = u64::from(Quadrante::de(q, [w, h]).n);
                let tr = 3 + (n - 2) + n;
                tr + 1 + if arquivo_da_borda { tr } else { 0 }
            })
            .sum::<u64>()
    };
    por_quadrante + 2 + 3
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
