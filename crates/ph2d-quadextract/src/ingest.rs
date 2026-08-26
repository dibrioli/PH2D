//! **FASE 1a — a ENTRADA vira topologia exacta**: a escala comum, a quantização, o
//! colapso do que degenerou no domínio, e as funções de transição.
//!
//! ⚠️ **A ordem aqui é dependência de dados, não gosto:** a escala tem de sair
//! **antes** da quantização (é ela que a define), a quantização antes do colapso
//! (uma aresta que degenera *depois* de truncada tem de ser colapsada, e não
//! descoberta a meio da extracção), e o colapso antes das transições (uma face
//! morta não tem carta).

use crate::CornerMap;
use crate::ExtractError;
use crate::exact::{COORD_MAX, P, Xf};

/// A malha, o domínio e as costuras — tudo em inteiros exactos.
pub(crate) struct Topo {
    /// Por face sobrevivente, os três vértices (já em **classes**, pós-colapso).
    pub tris: Vec<[u32; 3]>,
    /// Por face, por canto, a imagem no domínio.
    pub uv: Vec<[P; 3]>,
    /// Por face, por **lado** `k` (a aresta do canto `k` para o canto `k+1`), a
    /// face vizinha e o lado dela — ou `None` se for bordo.
    pub twin: Twins,
    /// Por face, por lado `k`, a transição **desta carta para a da vizinha**.
    pub xf: Vec<[Xf; 3]>,
    /// ⭐ **A posição em `R³` guardada nos CANTOS, nunca no vértice.**
    ///
    /// ⚠️ É o que impede a geometria da superfície de se perder quando dois
    /// vértices se fundem no domínio: o vértice fundido tem uma posição só, mas
    /// cada canto continua a lembrar-se de onde a sua metade estava.
    pub p3: Vec<[[f64; 3]; 3]>,
    /// Quantas classes de vértice sobreviveram.
    pub verts: usize,
    /// Uma célula da grade, em unidades internas — `2^Q`.
    pub one: i64,
}

/// O que a ingestão mediu de si própria.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct IngestStats {
    /// O expoente da grade comum — ver [`crate::ExtractReport::grid_exponent`].
    pub grid_exponent: u32,
    /// Arestas cuja imagem no domínio tinha comprimento zero e foram colapsadas.
    pub collapsed: usize,
    /// Faces que morreram no colapso (menos de três vértices distintos).
    pub dead_faces: usize,
    /// ⭐⭐ **O pior resíduo da ROTAÇÃO** lido das cartas, em quartos de volta.
    pub rot_residual: f64,
    /// A MEDIANA do resíduo da translação, em células.
    ///
    /// ⚠️ **Ela está ao lado do máximo porque as duas contam histórias
    /// diferentes:** um máximo de meia célula com mediana `0` é *uma* costura má; com
    /// mediana `0,03` é o mapa inteiro a não fechar. *Um extremo global não distingue
    /// as duas, e foi a mediana que nomeou qual delas a cadeia da casa tem.*
    pub shift_residual_p50: f64,
    /// ⭐⭐⭐ **Quantas transições ficaram FRACCIONÁRIAS** (`> 1e-3` de célula) — a
    /// contagem, não o extremo. Ver [`derive_transitions`].
    pub shift_fractional: usize,
    /// ⭐⭐⭐ **O pior desencontro RELATIVO de comprimento** entre as duas imagens da mesma
    /// aresta, entre as transições fraccionárias. Uma rotação preserva comprimento ⇒ um
    /// valor não-nulo aqui diz que o defeito é do MAPA, não da translação.
    pub seam_length_gap: f64,
    /// ⭐⭐⭐ **O pior resíduo da TRANSLAÇÃO**, em células. ⛔ A extracção **assume**
    /// que ele é zero: um mapa cuja translação de costura não seja inteira tem as
    /// duas grades desalinhadas, e o saneamento só arredonda o erro para dentro.
    pub shift_residual: f64,
    /// Arestas interiores.
    pub interior_edges: usize,
    /// Arestas de bordo.
    pub boundary_edges: usize,
    /// ⚠️ Arestas com mais de duas faces — a malha não é uma superfície ali.
    pub non_manifold: usize,
}

/// **A ESCALA COMUM** — `Q` tal que toda coordenada do domínio vira um inteiro
/// exacto de menos de `2^52` passos.
///
/// ⭐⭐⭐ **É uma grade GLOBAL, e isso é deliberadamente mais forte do que uma
/// grade por vértice.** A lei da precisão exige que a imagem de um vértice seja
/// truncada aos bits que **toda** carta incidente consegue representar; uma grade
/// por vértice satisfaz cada vértice isoladamente, e uma grade global satisfaz
/// **todos ao mesmo tempo** — porque ela é sempre igual ou mais grossa do que a
/// que cada vértice exigiria.
///
/// ⚠️ **E é isso que compra a aritmética exacta:** com uma grade só, o domínio
/// inteiro é `i64` sem perder um bit e o predicado de orientação é um `i128`
/// exacto, sem filtro e sem biblioteca de precisão múltipla.
fn common_grid(map: &CornerMap) -> Result<u32, ExtractError> {
    let mut hi = i32::MIN;
    for tri in map.uv {
        for c in tri {
            for &v in c {
                if !v.is_finite() {
                    return Err(ExtractError::NotFinite);
                }
                if v != 0.0 {
                    // `frexp`: v = f · 2^e com f ∈ [0.5, 1) ⇒ expoente binário = e−1.
                    let e = ((v.abs().to_bits() >> 52) & 0x7ff) as i32 - 1023;
                    hi = hi.max(e);
                }
            }
        }
    }
    if hi == i32::MIN {
        return Err(ExtractError::EmptyDomain);
    }
    if hi > 50 {
        return Err(ExtractError::DomainTooLarge);
    }
    // ⚠️ O tecto de 62 não é folga inventada: acima dele `1i64 << q` transborda, e
    // um domínio pequeno o bastante para o pedir já cabe de sobra na grade de 62.
    Ok(u32::try_from((51 - hi).clamp(1, 62)).unwrap_or(1))
}

/// **A TRUNCAGEM** de uma coordenada para a grade — o passo que o §2.3 pede.
///
/// ⚠️ **A multiplicação por `2^q` é exacta** (é um expoente, não uma mantissa), e
/// por isso quem trunca é o corte para inteiro que vem a seguir, e só ele. *Um
/// não-operador algébrico (`(x+S)−S`) faria o mesmo e o otimizador tem licença
/// para o apagar; um corte para inteiro é uma instrução que ele não pode inventar.*
fn to_grid(x: f64, scale: f64) -> Option<i64> {
    let s = x * scale;
    if !s.is_finite() {
        return None;
    }
    #[allow(clippy::cast_precision_loss)]
    if s.abs() >= COORD_MAX as f64 {
        return None;
    }
    #[allow(clippy::cast_possible_truncation)]
    Some(s.trunc() as i64)
}

/// `2^q`, exacto.
fn pow2(q: u32) -> f64 {
    f64::from_bits(u64::from(1023 + q) << 52)
}

/// Union-find sobre vértices — o registo do colapso.
struct Uf(Vec<u32>);

impl Uf {
    fn new(n: usize) -> Self {
        Self((0..u32::try_from(n).unwrap_or(u32::MAX)).collect())
    }
    fn find(&mut self, mut a: u32) -> u32 {
        while self.0[a as usize] != a {
            let g = self.0[self.0[a as usize] as usize];
            self.0[a as usize] = g;
            a = g;
        }
        a
    }
    fn union(&mut self, a: u32, b: u32) -> bool {
        let (ra, rb) = (self.find(a), self.find(b));
        if ra == rb {
            return false;
        }
        // Ordem determinística: o menor índice fica raiz.
        let (lo, hi) = if ra < rb { (ra, rb) } else { (rb, ra) };
        self.0[hi as usize] = lo;
        true
    }
}

/// ⭐ **A INGESTÃO** — de mapa de cantos a topologia exacta.
pub(crate) fn ingest(map: &CornerMap) -> Result<(Topo, IngestStats), ExtractError> {
    let q = common_grid(map)?;
    let scale = pow2(q);
    let one = 1i64 << q;
    let mut st = IngestStats {
        grid_exponent: q,
        ..IngestStats::default()
    };

    // ── Quantizar TODOS os cantos para a grade comum.
    let nf = map.tris.len();
    if nf != map.uv.len() {
        return Err(ExtractError::Mismatched);
    }
    let mut raw: Vec<[P; 3]> = Vec::with_capacity(nf);
    for tri in map.uv {
        let mut out = [[0i64; 2]; 3];
        for (k, c) in tri.iter().enumerate() {
            out[k] = [
                to_grid(c[0], scale).ok_or(ExtractError::DomainTooLarge)?,
                to_grid(c[1], scale).ok_or(ExtractError::DomainTooLarge)?,
            ];
        }
        raw.push(out);
    }

    // ── §2.1 O COLAPSO do que degenerou no domínio, sobre os valores JÁ truncados.
    //
    // ⚠️ **Truncar primeiro e colapsar depois, e não o contrário.** Uma aresta com
    // comprimento não-nulo mas menor que o passo da grade degenera *na truncagem*;
    // descobri-la só na extracção seria descobri-la como um caso especial no meio
    // de um predicado.
    let mut uf = Uf::new(map.pos.len());
    for (f, tri) in map.tris.iter().enumerate() {
        for k in 0..3 {
            let (a, b) = (tri[k], tri[(k + 1) % 3]);
            if raw[f][k] == raw[f][(k + 1) % 3] && uf.union(a, b) {
                st.collapsed += 1;
            }
        }
    }

    // ── Renumerar as classes e deixar cair as faces que morreram.
    let mut label = vec![u32::MAX; map.pos.len()];
    let mut verts = 0u32;
    let mut tris: Vec<[u32; 3]> = Vec::with_capacity(nf);
    let mut uv: Vec<[P; 3]> = Vec::with_capacity(nf);
    let mut p3: Vec<[[f64; 3]; 3]> = Vec::with_capacity(nf);
    for (f, tri) in map.tris.iter().enumerate() {
        let c = [uf.find(tri[0]), uf.find(tri[1]), uf.find(tri[2])];
        if c[0] == c[1] || c[1] == c[2] || c[2] == c[0] {
            st.dead_faces += 1;
            continue;
        }
        let mut out = [0u32; 3];
        for k in 0..3 {
            let s = &mut label[c[k] as usize];
            if *s == u32::MAX {
                *s = verts;
                verts += 1;
            }
            out[k] = *s;
        }
        tris.push(out);
        uv.push(raw[f]);
        p3.push([
            pos64(map.pos[tri[0] as usize]),
            pos64(map.pos[tri[1] as usize]),
            pos64(map.pos[tri[2] as usize]),
        ]);
    }

    let (twin, nm) = build_twins(&tris);
    st.non_manifold = nm;
    for t in &twin {
        for s in t {
            if s.is_some() {
                st.interior_edges += 1;
            } else {
                st.boundary_edges += 1;
            }
        }
    }
    st.interior_edges /= 2;

    let mut topo = Topo {
        tris,
        uv,
        twin,
        xf: Vec::new(),
        p3,
        verts: verts as usize,
        one,
    };
    let (xf, rot_res, sh_res, sh_p50, sh_frac, len_gap) = derive_transitions(&topo, map, one);
    st.rot_residual = rot_res;
    st.shift_residual = sh_res;
    st.shift_residual_p50 = sh_p50;
    st.shift_fractional = sh_frac;
    st.seam_length_gap = len_gap;
    topo.xf = xf;
    Ok((topo, st))
}

fn pos64(p: [f32; 3]) -> [f64; 3] {
    [f64::from(p[0]), f64::from(p[1]), f64::from(p[2])]
}

/// Por face, por lado, a face vizinha e o lado dela — ou `None` se for bordo.
pub(crate) type Twins = Vec<[Option<(u32, u8)>; 3]>;

/// Os gémeos de cada lado, por chave de aresta ordenada.
pub(crate) fn build_twins(tris: &[[u32; 3]]) -> (Twins, usize) {
    use std::collections::BTreeMap;
    let mut side: BTreeMap<(u32, u32), Vec<(u32, u8)>> = BTreeMap::new();
    for (f, t) in tris.iter().enumerate() {
        for k in 0..3u8 {
            let (a, b) = (t[k as usize], t[(k as usize + 1) % 3]);
            let key = if a < b { (a, b) } else { (b, a) };
            #[allow(clippy::cast_possible_truncation)]
            side.entry(key).or_default().push((f as u32, k));
        }
    }
    let mut twin: Twins = vec![[None; 3]; tris.len()];
    let mut non_manifold = 0usize;
    for (_, v) in side {
        match v.len() {
            2 => {
                let (f0, k0) = v[0];
                let (f1, k1) = v[1];
                twin[f0 as usize][k0 as usize] = Some((f1, k1));
                twin[f1 as usize][k1 as usize] = Some((f0, k0));
            }
            1 => {}
            _ => non_manifold += 1,
        }
    }
    (twin, non_manifold)
}

/// ⭐ **§2.2 — AS FUNÇÕES DE TRANSIÇÃO**, lidas das duas imagens de cada aresta.
///
/// ⛔ **A rotação primeiro, a translação depois** — nesta ordem, porque a segunda
/// sai por substituição *depois* de a primeira estar fixada. O inverso obtém-se com
/// a rotação complementar aplicada à diferença, e é o que [`Xf::inverse`] faz.
///
/// ⚠️ **A rotação sai da razão dos dois vetores de aresta** e não de um ângulo
/// absoluto: é a razão que é invariante à moldura de cada carta.
fn derive_transitions(
    topo: &Topo,
    map: &CornerMap,
    one: i64,
) -> (Vec<[Xf; 3]>, f64, f64, f64, usize, f64) {
    let mut xf = vec![[Xf::IDENTITY; 3]; topo.tris.len()];
    let mut rot_res = 0.0f64;
    let mut sh_res = 0.0f64;
    let mut all: Vec<f64> = Vec::new();
    let mut len_gap = 0.0f64;
    let mut len_gaps: Vec<f64> = Vec::new();
    let _ = map;
    // ⚠️ Os índices percorrem QUATRO tabelas paralelas (`twin`, `uv`, `xf` e os
    // cantos rodados), e nenhuma delas se deixa iterar junto das outras: o `k` de uma
    // face indexa o canto `k` e o canto `k+1` ao mesmo tempo.
    #[allow(clippy::needless_range_loop)]
    for f in 0..topo.tris.len() {
        for k in 0..3usize {
            let Some((g, j)) = topo.twin[f][k] else {
                continue;
            };
            let (g, j) = (g as usize, j as usize);
            // A MESMA aresta nos dois sentidos: o canto `k` de `f` é o canto `j+1`
            // de `g`, e o canto `k+1` de `f` é o canto `j` de `g`.
            let a1 = topo.uv[f][k];
            let b1 = topo.uv[f][(k + 1) % 3];
            let a2 = topo.uv[g][(j + 1) % 3];
            let b2 = topo.uv[g][j];
            let d1 = [b1[0] - a1[0], b1[1] - a1[1]];
            let d2 = [b2[0] - a2[0], b2[1] - a2[1]];
            let (r, rr) = best_rotation(d1, d2);
            rot_res = rot_res.max(rr);
            let rot = Xf::rot(r, a1);
            let t = [a2[0] - rot[0], a2[1] - rot[1]];
            let (ti, tr) = round_to_cells(t, one);
            sh_res = sh_res.max(tr);
            all.push(tr);
            // ⭐⭐⭐ **O DESENCONTRO DE COMPRIMENTO da MESMA aresta nas duas cartas.**
            //
            // ⚠️ **É ele que decide entre duas curas opostas.** Uma rotação preserva o
            // comprimento: se `|d1| ≠ |d2|`, **nenhuma** das quatro rotações leva uma
            // aresta na outra, e a translação sai fraccionária por construção — o defeito
            // é a MONTANTE, no mapa. Se os comprimentos batem e só a translação falha, o
            // defeito é da própria translação. *Sem esta coluna as duas leem-se igual.*
            if tr > 1.0e-3 {
                let l1 = ((d1[0] as f64).hypot(d1[1] as f64)) / one as f64;
                let l2 = ((d2[0] as f64).hypot(d2[1] as f64)) / one as f64;
                len_gap = len_gap.max((l1 - l2).abs() / l1.max(1.0e-12));
                len_gaps.push((l1 - l2).abs() / l1.max(1.0e-12));
            }
            xf[f][k] = Xf { r, t: ti };
        }
    }
    all.sort_by(f64::total_cmp);
    let p50 = if all.is_empty() {
        0.0
    } else {
        all[all.len() / 2]
    };
    // ⭐⭐⭐ **QUANTAS transições são de facto fraccionárias** — não o pior, a CONTAGEM.
    //
    // ⚠️ **O `max` sozinho não escolhe uma cura:** `0,48` pode ser um sítio único (e aí a
    // pergunta é *qual*) ou metade da peça (e aí é sistémico). *Um extremo diz que existe;
    // só a contagem diz o tamanho.*
    let fractional = all.iter().filter(|r| **r > 1.0e-3).count();
    let _ = len_gaps;
    (xf, rot_res, sh_res, p50, fractional, len_gap)
}

/// A rotação de quarto de volta que melhor leva `d1` a `d2`, e o resíduo dela.
///
/// ⚠️ **O resíduo é medido em quartos de volta** e não em radianos: é a grandeza em
/// que o arredondamento decide, e reportá-la noutra unidade obrigaria quem lê a
/// converter para saber se está perto de meio passo.
fn best_rotation(d1: P, d2: P) -> (u8, f64) {
    #[allow(clippy::cast_precision_loss)]
    let (x1, y1) = (d1[0] as f64, d1[1] as f64);
    #[allow(clippy::cast_precision_loss)]
    let (x2, y2) = (d2[0] as f64, d2[1] as f64);
    // arg(d2 / d1) / (π/2)
    let den = x1.mul_add(x1, y1 * y1);
    if den == 0.0 {
        return (0, 0.0);
    }
    let re = x2.mul_add(x1, y2 * y1) / den;
    let im = y2.mul_add(x1, -(x2 * y1)) / den;
    let quarters = im.atan2(re) / core::f64::consts::FRAC_PI_2;
    let k = quarters.round();
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let r = (k.rem_euclid(4.0)) as u8;
    (r, (quarters - k).abs())
}

/// Arredonda uma translação para o múltiplo de célula mais próximo, devolvendo o
/// resíduo em células.
fn round_to_cells(t: P, one: i64) -> (P, f64) {
    let mut out = [0i64; 2];
    let mut worst = 0.0f64;
    for i in 0..2 {
        let n = div_round(t[i], one);
        out[i] = n * one;
        #[allow(clippy::cast_precision_loss)]
        let d = ((t[i] - out[i]) as f64 / one as f64).abs();
        worst = worst.max(d);
    }
    (out, worst)
}

/// Divisão inteira com arredondamento ao mais próximo (empate para longe de zero).
pub(crate) fn div_round(a: i64, b: i64) -> i64 {
    let (q, r) = (a / b, a % b);
    if 2 * r.abs() >= b.abs() {
        q + if (a < 0) == (b < 0) { 1 } else { -1 }
    } else {
        q
    }
}
