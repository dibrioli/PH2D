//! ⭐⭐⭐ **FASE 1b — SANEAR: tornar EXACTAS as leis que o mapa só satisfaz até um
//! erro pequeno.** É esta fase que permite que todo o resto seja discreto.
//!
//! ⛔⛔ **É o passo que ninguém adivinha, e sem ele o resto não fecha.** As
//! coordenadas do mesmo vértice em cartas diferentes têm expoentes diferentes;
//! aplicar a transição de uma carta para outra perde bits baixos, e ao dar a volta
//! ao leque **não se regressa ao valor de partida**. Um ponto de grade cai então na
//! fenda numérica entre dois triângulos — ou **nos dois**. ⚠️ E não é raro: onde há
//! alinhamento a uma feição, pontos de grade caem **necessariamente** sobre arestas
//! da malha de entrada.
//!
//! # As duas leis desta fase
//!
//! 1. **A precisão iguala-se** ([`crate::ingest`] já truncou tudo para uma grade
//!    comum). Aqui a imagem de cada vértice é **propagada** a partir de um canto
//!    só, aplicando as transições — deixa de haver uma imagem por carta que possa
//!    discordar das outras.
//! 2. **Numa singularidade, arredondar não basta.** A composição das transições à
//!    volta do leque não é a identidade, e o valor não regressa ao fechar a volta.
//!    A imagem tem de ir para o **PONTO FIXO** dessa composição, que se resolve em
//!    forma fechada.
//!
//! ⚠️ **É a segunda lei que torna a primeira bem-posta:** propagar a partir de um
//! canto só dá o mesmo resultado por qualquer caminho **exactamente** quando o
//! valor é invariante pela holonomia — identidade num vértice regular, ponto fixo
//! numa singularidade.

use crate::exact::{P, Xf};
use crate::fan::{Corner, fan_of, seed_corners};
use crate::ingest::{Topo, build_twins, div_round};

/// O que o saneamento mediu de si próprio.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct SanitizeStats {
    /// Vértices pregados no ponto fixo de uma holonomia com rotação.
    pub pinned_fixed: usize,
    /// Singularidades de rotação nula (valência múltipla de 4) pregadas no inteiro.
    pub pinned_integer: usize,
    /// Vértices de bordo — leque aberto, sem holonomia.
    pub open_fans: usize,
    /// ⛔ Leques que se dizem regulares e cuja holonomia **não** é a identidade —
    /// o mapa não é de grade inteira ali.
    pub holonomy_broken: usize,
    /// Arestas que degeneraram **depois** do saneamento e foram colapsadas.
    pub late_collapsed: usize,
    /// ⛔ Arestas interiores cuja transição não se deixou reler **exactamente** dos
    /// valores saneados. Zero é o único resultado bom.
    pub inexact_transitions: usize,
    /// ⛔⛔⛔ **Recursos de transição que NÃO aproximam** — a imagem de `a1` cai a mais de
    /// meia célula de `a2`. `> 0` é vermelho: uma transição dessas manda o traçado para
    /// outra parte da peça, e era exactamente o que a identidade fazia.
    ///
    /// ⭐ Medido 2026-08-25 na peça do artista: **`4`** com o recurso antigo, **`1`** com o
    /// novo. Nas peças do corpus sem o defeito: `0` nos dois.
    pub far_fallbacks: usize,
    /// A distribuição de valências, indexada por valência (`0..=15`).
    pub valence: [usize; 16],
    /// ⭐ Quantas vezes o ângulo e a **holonomia** discordaram sobre a valência, e a
    /// holonomia — que é exacta — decidiu. Acontece à volta de uma dobra.
    pub valence_adjusted: usize,
}

/// ⭐ **O SANEAMENTO.** Devolve o que mediu; muda `topo` no lugar.
pub(crate) fn sanitize(topo: &mut Topo, valence_hint: Option<&[u8]>) -> SanitizeStats {
    let mut st = SanitizeStats::default();
    let seeds = seed_corners(topo);
    for (v, seed) in seeds.iter().enumerate() {
        let Some(seed) = *seed else { continue };
        let fan = fan_of(topo, seed);
        let first = fan.corners[0];
        let here = topo.uv[first.f()][first.kk()];
        let val = match valence_hint.and_then(|h| h.get(v).copied()) {
            Some(v) => v,
            None => {
                let (v, adjusted) = valence_of(topo, &fan.corners, fan.holonomy);
                if adjusted {
                    st.valence_adjusted += 1;
                }
                v
            }
        };
        if let Some(v16) = st.valence.get_mut(val as usize) {
            *v16 += 1;
        }
        let root = match fan.holonomy {
            None => {
                st.open_fans += 1;
                here
            }
            Some(h) if h.r != 0 => {
                st.pinned_fixed += 1;
                h.fixed_point().unwrap_or(here)
            }
            Some(h) => {
                // ⚠️ **Rotação nula NÃO prova regularidade** — uma singularidade de
                // valência múltipla de 4 dá zero também, e é por isso que a
                // valência entra aqui. Confiar só na rotação deixaria um nó de
                // valência 8 fora da grade, e a grade não fecha à volta dele.
                if val == 4 {
                    if h.t != [0, 0] {
                        st.holonomy_broken += 1;
                    }
                    here
                } else {
                    st.pinned_integer += 1;
                    nearest_integer(here, topo.one)
                }
            }
        };
        for (i, c) in fan.corners.iter().enumerate() {
            topo.uv[c.f()][c.kk()] = fan.to_here[i].apply(root);
        }
    }
    st.late_collapsed = late_collapse(topo);
    let (bad, far) = rederive_exact(topo);
    st.inexact_transitions = bad;
    st.far_fallbacks = far;
    st
}

/// O ponto de grade inteiro mais próximo.
fn nearest_integer(p: P, one: i64) -> P {
    [div_round(p[0], one) * one, div_round(p[1], one) * one]
}

/// ⭐ **A VALÊNCIA** — a parte exacta vem da holonomia, a grosseira vem do ângulo.
///
/// A valência de uma singularidade é precisa no saneamento e só ficaria disponível
/// depois dele. ⚠️ **Mas só é preciso distinguir `4` de `≥ 8`**, e para isso o ângulo
/// total dos cantos, medido nas coordenadas ainda não saneadas, **serve**.
///
/// ⛔⛔ **A primeira redacção somava `|ângulo|` e discordava da holonomia num
/// vértice do toro** (medido: pregámos `10` pontos fixos para `9` vértices
/// não-regulares). A causa é a dobra: num canto **dobrado** o domínio percorre a
/// cunha ao contrário, e a contribuição dele é **negativa** — tomar o módulo canto a
/// canto conta-a como positiva e a soma sai `4` sobre um vértice que a rotação
/// acumulada diz ser singular.
///
/// ⭐⭐ **A cura não é medir melhor o ângulo: é usar a parte que já é exacta.** A
/// holonomia dá a valência **módulo 4** sem erro nenhum (`v ≡ r`, medido); o ângulo
/// dá a **grandeza**. Encostar a grandeza ao resíduo certo faz as duas concordarem
/// por construção, e o instrumento fica a contar quantas vezes o encosto foi preciso
/// — que é exactamente o número de vértices em que o ângulo sozinho estava errado.
fn valence_of(topo: &Topo, corners: &[Corner], holonomy: Option<Xf>) -> (u8, bool) {
    let mut total = 0.0f64;
    for c in corners {
        let p = topo.uv[c.f()][c.kk()];
        let q = topo.uv[c.f()][(c.kk() + 1) % 3];
        let r = topo.uv[c.f()][(c.kk() + 2) % 3];
        let a = sub64(q, p);
        let b = sub64(r, p);
        let cross = a[0].mul_add(b[1], -(a[1] * b[0]));
        let dot = a[0].mul_add(b[0], a[1] * b[1]);
        if cross == 0.0 && dot == 0.0 {
            continue;
        }
        // ⚠️ **EM MÓDULO, e a alternativa foi MEDIDA e é pior.** Somar com sinal
        // faz um canto dobrado subtrair, e no toro isso lê `1` quarto de volta sobre
        // um vértice cuja holonomia diz `5`: a superfície dá a volta inteira, é o
        // **domínio** que dobra para trás. O módulo acerta em 9 dos 10 vértices
        // singulares do toro; o décimo é o da dobra, e é a holonomia que o resolve.
        total += cross.atan2(dot).abs();
    }
    let quarters = total / core::f64::consts::FRAC_PI_2;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let coarse = quarters.round().clamp(0.0, 15.0) as u8;
    let Some(h) = holonomy else {
        // Leque aberto: não há holonomia, e a valência de bordo não é uma volta
        // inteira. A grandeza é tudo o que há.
        return (coarse, false);
    };
    // ⭐⭐ **A RELAÇÃO FOI MEDIDA, não assumida** (`PH2D_QX_DEBUG_VALENCE`, 2026-08-24):
    // `coarse=3 ⇒ r=3` · `coarse=5 ⇒ r=1` · `coarse=4 ⇒ r=0` ⇒ **`v ≡ r (mod 4)`**.
    // ⛔ A primeira redacção escreveu `v ≡ 4 − r`, que é o oposto, e o preço foi
    // visível a olho: os cinco vértices de valência 5 do toro viraram `1`, o
    // saneamento pregou os vértices errados, e a saída do gancho perdeu a casca
    // (`χ = 1` com 34 arestas de bordo). *Um sinal trocado numa congruência não dá
    // erro de compilação nem de tipo — dá um buraco na malha.*
    let want = h.r & 3;
    if coarse & 3 == want {
        return (coarse, false);
    }
    // Encosta ao valor mais próximo com o resíduo que a holonomia exige.
    //
    // ⚠️ **O desempate é pelo mais perto de `4`, e não é decoração:** com `coarse=2`
    // e resíduo `0`, os candidatos `0` e `4` estão à MESMA distância, e ficar com o
    // primeiro faz nascer um vértice de «valência 0» — que não é uma valência. O
    // caso apareceu no gancho.
    let mut best = 4u8;
    let mut key = (i32::MAX, i32::MAX);
    for v in 1..16u8 {
        if v & 3 != want {
            continue;
        }
        let k = (
            (i32::from(v) - i32::from(coarse)).abs(),
            (i32::from(v) - 4).abs(),
        );
        if k < key {
            key = k;
            best = v;
        }
    }
    (best, true)
}

fn sub64(a: P, b: P) -> [f64; 2] {
    #[allow(clippy::cast_precision_loss)]
    [(a[0] - b[0]) as f64, (a[1] - b[1]) as f64]
}

/// **§2.5 — o COLAPSO FINAL.** Uma singularidade pregada no ponto fixo move-se até
/// meia célula, e isso pode zerar uma aresta curta que estava viva antes.
///
/// ⚠️ **Isto não muda a parametrização** — serve só para o resto não ter casos
/// especiais.
fn late_collapse(topo: &mut Topo) -> usize {
    let mut merge: Vec<(u32, u32)> = Vec::new();
    for (f, tri) in topo.tris.iter().enumerate() {
        for k in 0..3 {
            if topo.uv[f][k] == topo.uv[f][(k + 1) % 3] {
                merge.push((tri[k], tri[(k + 1) % 3]));
            }
        }
    }
    if merge.is_empty() {
        return 0;
    }
    let mut parent: Vec<u32> = (0..u32::try_from(topo.verts).unwrap_or(u32::MAX)).collect();
    fn find(p: &mut [u32], mut a: u32) -> u32 {
        while p[a as usize] != a {
            let g = p[p[a as usize] as usize];
            p[a as usize] = g;
            a = g;
        }
        a
    }
    let mut n = 0usize;
    for (a, b) in merge {
        let (ra, rb) = (find(&mut parent, a), find(&mut parent, b));
        if ra != rb {
            let (lo, hi) = if ra < rb { (ra, rb) } else { (rb, ra) };
            parent[hi as usize] = lo;
            n += 1;
        }
    }
    let mut label = vec![u32::MAX; topo.verts];
    let mut verts = 0u32;
    let mut tris = Vec::with_capacity(topo.tris.len());
    let mut uv = Vec::with_capacity(topo.uv.len());
    let mut p3 = Vec::with_capacity(topo.p3.len());
    for (f, tri) in topo.tris.iter().enumerate() {
        let c = [
            find(&mut parent, tri[0]),
            find(&mut parent, tri[1]),
            find(&mut parent, tri[2]),
        ];
        if c[0] == c[1] || c[1] == c[2] || c[2] == c[0] {
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
        uv.push(topo.uv[f]);
        p3.push(topo.p3[f]);
    }
    topo.tris = tris;
    topo.uv = uv;
    topo.p3 = p3;
    topo.verts = verts as usize;
    let (twin, _) = build_twins(&topo.tris);
    topo.twin = twin;
    topo.xf = vec![[Xf::IDENTITY; 3]; topo.tris.len()];
    n
}

/// ⭐⭐⭐ **A PROVA de que o saneamento fechou: reler cada transição dos valores
/// saneados, EXACTAMENTE.**
///
/// ⚠️ **É uma releitura e não uma repetição.** A primeira derivação
/// ([`crate::ingest`]) saiu de coordenadas com resíduo e teve de arredondar; depois
/// do saneamento a mesma leitura tem de dar **inteiro ao bit**, para os dois
/// vértices da aresta ao mesmo tempo. Uma que não dê nomeia o sítio exacto onde o
/// mapa não é de grade inteira — e é ela, e não um `assert` de compilação, que
/// separa um mapa saneado de um mapa quase saneado.
fn rederive_exact(topo: &mut Topo) -> (usize, usize) {
    let mut bad = 0usize;
    let mut far = 0usize;
    let n = topo.tris.len();
    let mut out = vec![[Xf::IDENTITY; 3]; n];
    // ⚠️ Mesma razão do irmão em [`crate::ingest`]: o `k` indexa o canto `k` e o
    // `k+1` da mesma face, em quatro tabelas paralelas.
    #[allow(clippy::needless_range_loop)]
    for f in 0..n {
        for k in 0..3usize {
            let Some((g, j)) = topo.twin[f][k] else {
                continue;
            };
            let (g, j) = (g as usize, j as usize);
            let a1 = topo.uv[f][k];
            let b1 = topo.uv[f][(k + 1) % 3];
            let a2 = topo.uv[g][(j + 1) % 3];
            let b2 = topo.uv[g][j];
            let d1 = [b1[0] - a1[0], b1[1] - a1[1]];
            let d2 = [b2[0] - a2[0], b2[1] - a2[1]];
            let mut found = None;
            for r in 0..4u8 {
                if Xf::rot(r, d1) != d2 {
                    continue;
                }
                let rot = Xf::rot(r, a1);
                let t = [a2[0] - rot[0], a2[1] - rot[1]];
                let cand = Xf { r, t };
                if cand.apply(b1) == b2 {
                    found = Some(cand);
                    break;
                }
            }
            match found {
                Some(x) => out[f][k] = x,
                None => {
                    bad += 1;
                    // ⛔⛔⛔ **O RECURSO ERA `topo.xf[f][k]`, E ISSO PODE SER A IDENTIDADE.**
                    //
                    // Ele lia o valor derivado no [`crate::ingest`]. ⚠️ **Mas a
                    // [`late_collapse`] renumera as faces e repõe o `xf` inteiro a
                    // `IDENTITY`** antes desta função correr — e ela corre logo acima.
                    // Nas peças em que ela mexe, este ramo devolvia **a identidade** para
                    // uma aresta que precisa de rotação e translação. *Uma transição
                    // identidade entre duas cartas que não coincidem manda o traçado para
                    // outra parte da peça, e nenhum gate de integralidade a apanha.*
                    //
                    // ⭐ O recurso passa a ser a **melhor transição disponível**, com a
                    // mesma lei do ingest — mas calculada sobre o `uv` **de agora**, que a
                    // propagação desta fase já corrigiu. *Um recurso que é a identidade é
                    // uma mentira; um que é a melhor aproximação é uma aproximação.*
                    let (r, _) = crate::ingest::best_rotation(d1, d2);
                    let rot = Xf::rot(r, a1);
                    let (t, _) =
                        crate::ingest::round_to_cells([a2[0] - rot[0], a2[1] - rot[1]], topo.one);
                    out[f][k] = Xf { r, t };
                }
            }
            // ⭐⭐⭐ **O RECURSO TEM DE APROXIMAR, e isto mede-o.**
            //
            // Uma transição correcta leva `a1` a `a2`; um recurso honesto leva-o a menos de
            // **meia célula** — é o que o arredondamento a células pode custar. ⚠️ *A
            // identidade não tem essa propriedade, e é assim que ela se apanha* sem depender
            // de a peça exibir a combinação rara que a produzia.
            let img = out[f][k].apply(a1);
            let d = [(img[0] - a2[0]) as f64, (img[1] - a2[1]) as f64];
            let half = topo.one as f64 * 0.5;
            if d[0].mul_add(d[0], d[1] * d[1]) > half * half {
                far += 1;
            }
        }
    }
    topo.xf = out;
    (bad, far)
}
