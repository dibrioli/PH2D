//! ⭐ **A LADEIRA DO ÓLEO para a água** (Bug 2 do smoke do dono 30/10: «a água sobe a ladeira do óleo»). O nivelamento
//! corria só pela altura da ÁGUA (`film`) e o capilar pelo dente do papel; a altura do óleo assentado não entrava em
//! passe nenhum e a água espalhava-se por cima dele como por papel liso. A superfície que a água nivela é água + óleo: a
//! do pé da parede corre para longe dela, a que está em cima do óleo desce. Só a tinta ASSENTADA com o Real Oil tem
//! altura (`sett × espessura × Depth·Real Oil`, a massa do campo de `oleo_real`); sem o campo do óleo na grade, `None`
//! e o fluxo de sempre, ao bit.
//!
//! Smoke do dono 30/10: com o Real Oil na mão, a ladeira é a do
//! óleo de ANTES do traço ([`Grid::oleo_antes`], fotografada no início do traço) — o óleo que se pinta corre como
//! sempre e a água do traço novo respeita o relevo dos anteriores. Sonda: `queda_na_ladeira_tests`.

use crate::grid::Grid;
use crate::grid::faixa_tiles::{TILE_FAIXA, span_runs, tile_words_for};
use ph2d_det_math::DetMath;

/// A rota da ladeira, em execução (sondas `sonda_oleo_derrete`, `sonda_oleo_furos`): `0` = o produto (= `7`), `1` = a de 30/10 sem peso,
/// `2` = desligada, `3` = fotografia com a massa inteira (assente + suspensa), sem peso, `4` = peso pela fracção
/// assente de AGORA, `5` = peso pela fracção assente da FOTOGRAFIA, `6` = `3` + `5`, `7` = `5` com a ladeira que segura
/// presa a `|dfilm|`.
#[doc(hidden)]
pub static LADEIRA_ROTA: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

/// ⭐ **A LEI DE BINGHAM do óleo grosso** (E7, queixa do dono 31/10: «mesmo com pouca água se comportam como líquidos e
/// provocam ondas e crateras»). A tinta com óleo tem TENSÃO DE CEDÊNCIA (Orchard 1962, F2 §2: escoa só se a força
/// passar de `τ_y`, e `τ_y` cresce com a concentração `φ` e o óleo `m`): abaixo dela nada escoa, acima escoa o excesso.
/// `τ_y` vem da RECEITA que está na célula, não do `film` instantâneo:
///
/// `φ = M ÷ (M + K_W·film)` (`M = susp + sett`, a mesma grandeza de `painter::receita`) ·
/// `τ_y = τ₀ · m · ((φ − φ_c) ÷ (1 − φ_c))₊ⁿ` (`m` = o MEIO da célula, [`Knob::Medium`] desde a E10;
/// era a fracção de Real Oil).
///
/// [`Knob::Medium`]: crate::tuning::Knob::Medium
///
/// Sem óleo (`m = 0`: aquarela, guache, tudo com Medium 0) `τ_y = 0` e o fluxo é o de sempre, ao bit; a tinta aguada
/// (`φ ≤ φ_c`) também. Constantes MEDIDAS na régua `sonda_relevo_oleo`:
/// `docs/Painter/Arquitetura/RELEVO_oleo_ondas_e_crateras.md` §6.
pub const TAU0: f64 = 3.0;
/// O limiar de percolação da rede de partículas (abaixo dele a tinta não tem estrutura: escoa como líquido).
pub const PHI_C: f64 = 0.2;
/// O expoente da lei de potência `τ_y ∝ φⁿ` das suspensões floculadas.
pub(crate) const EXPOENTE: i32 = 2;

/// Ferramenta de MEDIÇÃO (régua `sonda_relevo_oleo`), nunca o produto: os bits de um `f32` — `0` = `TAU0` (o produto),
/// `> 0` = outro `τ₀`, `< 0` = sem tensão de cedência (a lei de antes da E7, para o antes × depois).
#[doc(hidden)]
pub static PAPEL_BINGHAM: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
/// Ferramenta de MEDIÇÃO: os bits de um `f32` — `0` = `PHI_C`, senão outro `φ_c`.
#[doc(hidden)]
pub static BINGHAM_PHI_C: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// Ferramenta de MEDIÇÃO: `1` = `M` conta também o `sett` (o óleo assentado por baixo); `0` = só o `susp` (o produto).
#[doc(hidden)]
pub static BINGHAM_COM_SETT: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

/// A massa de pigmento da tinta que ESCOA na célula (`M` de `φ`): o `susp`. O assentado é o chão — não dá tensão de
/// cedência à tinta fresca por cima (o Wet Flat aguado com Real Oil sobre a crista seca do Dabber escorre, J11).
#[inline]
#[must_use]
pub(crate) fn massa_movel(susp: f32, sett: f32) -> f64 {
    let m = f64::from(susp);
    if BINGHAM_COM_SETT.load(std::sync::atomic::Ordering::Relaxed) != 0 {
        m + f64::from(sett)
    } else {
        m
    }
}

fn medida(a: &std::sync::atomic::AtomicU32, omissao: f64) -> f64 {
    match a.load(std::sync::atomic::Ordering::Relaxed) {
        0 => omissao,
        b => f64::from(f32::from_bits(b)),
    }
}

/// O `τ₀` em vigor (`≤ 0` = desligado, só pela régua).
#[inline]
pub(crate) fn tau0() -> f64 {
    medida(&PAPEL_BINGHAM, TAU0)
}

/// A tensão de cedência da célula, em unidades de velocidade: `m` o óleo, `massa` o pigmento (`susp + sett`), `f` a água.
/// Unidade própria: o `MaxVelocity` é só o tecto da velocidade (B13.9; era também um factor de `τ_y`).
#[inline]
#[must_use]
pub(crate) fn tensao(t0: f64, m: f64, massa: f64, f: f64) -> f64 {
    if t0 <= 0.0 || m <= 0.0 || massa <= 0.0 {
        return 0.0;
    }
    let phi = massa / (massa + crate::painter::receita::K_W * f.max(0.0));
    let pc = medida(&BINGHAM_PHI_C, PHI_C);
    let x = ((phi - pc) / (1.0 - pc)).max(0.0);
    t0 * m.min(1.0) * x.det_powi(EXPOENTE)
}

/// ⭐ **O CORPO do pincel** (E10, dono Q9): `τ_y = τ_meio(m, φ) + TAU_CORPO · corpo` — uma tensão de
/// cedência que NÃO precisa de `φ > φ_c` (a tinta de água tem `φ` baixo) e substitui a parede binária do
/// `levelingOwnStroke` (Bug #56: empilhava 12–39 µm na borda). Só onde há pigmento (`massa > 0`). `corpo = 0` ⇒ a
/// [`tensao`] do meio, ao bit. Faixa MEDIDA: `docs/Painter/Arquitetura/E10a2_meio_e_corpo.md`.
pub const TAU_CORPO: f64 = 0.25;

/// Ferramenta de MEDIÇÃO: os bits de um `f32` — `0` = [`TAU_CORPO`] (o produto), senão outro.
#[doc(hidden)]
pub static PAPEL_CORPO: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// A tensão da célula com o corpo: a do meio ([`tensao`], sobre a massa que ESCOA, [`massa_movel`]) + a do pincel,
/// que vale onde há tinta dele — `toda` = `susp + sett`: o corpo é da TINTA pousada, e a tinta molhada do Wet Paint
/// pousa quase toda assentada (a faixa molhada da cena `sobre` não tinha `susp` e escoava sem cedência).
#[inline]
#[must_use]
pub(crate) fn tensao_com_corpo(
    t0: f64,
    m: f64,
    corpo: f64,
    (movel, toda): (f64, f64),
    f: f64,
) -> f64 {
    let base = tensao(t0, m, movel, f);
    if corpo <= 0.0 || toda <= 0.0 {
        return base;
    }
    base + medida(&PAPEL_CORPO, TAU_CORPO) * corpo
}

/// ⭐ **A CEDÊNCIA da célula fina `fi`** (meio + corpo, [`tensao_com_corpo`] → [`bingham`]) sobre a velocidade `(ex, ey)`:
/// a porta ÚNICA da lei «a cedência decide se a tinta flui», onde o fluxo se CONSTRÓI (Jacobi e serial). O alisamento
/// dos outros três quadros não a aplica: tirar τ quatro vezes por ciclo empilha o óleo (Thick Impasto p99 39,7 → 105,7,
/// `cura-molhada.md`). Campo vazio ⇒ `(ex, ey)`, ao bit.
#[inline]
#[must_use]
pub(crate) fn cede(
    oleo: &crate::grid::oleo_celula::CampoOleo,
    (susp, sett): (&[f32], &[f32]),
    fi: usize,
    f: f64,
    (ex, ey): (f64, f64),
) -> (f64, f64) {
    if oleo.is_empty() {
        return (ex, ey);
    }
    let cel = crate::grid::oleo_celula::da_celula(oleo, fi);
    let massa = (
        massa_movel(susp[fi], sett[fi]),
        f64::from(susp[fi]) + f64::from(sett[fi]),
    );
    let tau = tensao_com_corpo(tau0(), cel.meio.into(), cel.corpo.into(), massa, f);
    bingham(ex, ey, tau)
}

/// Aplica `τ_y` ao incremento de velocidade `(ex, ey)`: abaixo dela nada escoa, acima escoa o excesso.
#[inline]
pub(crate) fn bingham(ex: f64, ey: f64, tau: f64) -> (f64, f64) {
    if tau <= 0.0 {
        return (ex, ey);
    }
    let v = (ex * ex + ey * ey).sqrt();
    if v <= tau {
        return (0.0, 0.0);
    }
    let k = (v - tau) / v;
    (ex * k, ey * k)
}

fn rota() -> u8 {
    match LADEIRA_ROTA.load(std::sync::atomic::Ordering::Relaxed) {
        0 => 7,
        r => r,
    }
}

/// A altura de água (`film`) que uma unidade de massa de óleo assentado ergue. MEDIDO (`diag_a_agua_sobe_a_ladeira_do_oleo`):
/// o miolo de um Dabber tem `~3000` de massa e a água de um Wet Flat `film` `1–4`; a parede tem de ganhar da água.
pub(crate) const FILME_POR_MASSA: f64 = 1.0 / 1000.0;

/// A altura do óleo assentado na célula fina `i`, em unidades de `film`.
pub(crate) fn altura(g: &Grid, i: usize, inteira: bool) -> f32 {
    let m = f64::from(g.sett[i]) + if inteira { f64::from(g.susp[i]) } else { 0.0 };
    if m <= 0.0 {
        return 0.0;
    }
    let k = crate::grid::oleo_celula::ganho(&g.oleo, i);
    if k <= 0.0 {
        return 0.0;
    }
    (m * crate::grid::espessura::factor(g, i) * k * FILME_POR_MASSA) as f32
}

/// A altura do óleo nas linhas FINAS `r0..=r1` (as que o passe de fluxo amostra), em unidades de `film`.
pub(crate) struct LadeiraDoOleo {
    r0: usize,
    s: usize,
    h: Vec<f32>,
    /// A fracção ASSENTADA da massa de cada célula, `sett ÷ (sett + susp)` (`1` sem tinta): o peso da ladeira.
    assente: Vec<f32>,
    /// A ladeira do óleo de ANTES com o óleo na mão: a que segura fica presa a `|dfilm|` ([`LadeiraDoOleo::termo`]).
    presa: bool,
    /// Alguma célula das janelas tem altura (sem nenhuma, o passe corre sem a ladeira, ao bit).
    alguma: bool,
    /// Quantas células a construção visitou (o gate `a_ladeira_so_visita_a_faixa_viva`).
    #[cfg(test)]
    pub(crate) visitadas: usize,
}

/// O `film` do Wet Flat CHEIO sobre a crista do óleo (água ×1), MEDIDO pela sonda J11 (`diag_a_queda_pela_diluicao`):
/// `0,983`. A ladeira que EMPURRA a água ladeira abaixo pesa `min(1, film ÷ FILM_CHEIO)` — o pincel quase seco não
/// escorre, o diluído sim (dono 30/10 escolheu a V3 da prancha `docs/Painter/Pinceis/pranchas/J11_queda_diluicao.png`).
/// A ladeira que SEGURA (contra a diferença de água, a parede ao pé do óleo) fica inteira: pesá-la deixava a água fina
/// subir o óleo (`a_agua_nao_sobe_a_ladeira_do_oleo`: 106 213 de massa no miolo, vermelho).
pub(crate) const FILM_CHEIO: f64 = 0.983;

impl LadeiraDoOleo {
    /// `None` sem altura nenhuma nas janelas `jan` (uma por linha fina `r0..`, `lo > hi` = vazia); `antes` = a altura
    /// fotografada no início do traço ([`Grid::oleo_antes`]) em vez da de agora. Fora das janelas os planos guardam
    /// lixo de outro passe: o passe de fluxo só lê dentro delas ([`do_passe`]).
    fn de(
        g: &Grid,
        (r0, jan, tiles): (usize, &[(i32, i32)], Option<(&[u64], usize)>),
        (mut h, mut assente): (Vec<f32>, Vec<f32>),
        antes: bool,
        mode: crate::par::Rows,
    ) -> Self {
        let s = g.s;
        let n = jan.len() * s;
        if h.len() < n {
            h.resize(n, 0.0);
            assente.resize(n, 1.0);
        }
        #[cfg(test)]
        {
            h.fill(f32::NAN);
            assente.fill(f32::NAN);
        }
        let rt = rota();
        let alguma = std::sync::atomic::AtomicBool::new(false);
        #[cfg(test)]
        let visitadas = std::sync::atomic::AtomicUsize::new(0);
        crate::par::walk_rows2(mode, &mut h[..n], &mut assente[..n], s, |ri, hr, ar| {
            let (lo, hi) = jan[ri];
            if lo > hi {
                return;
            }
            let base = (r0 + ri) * s;
            let words = tiles.map(|(t, tw)| &t[ri * tw..(ri + 1) * tw]);
            for (a, b) in span_runs(words, lo, hi) {
                #[cfg(test)]
                visitadas.fetch_add((b - a + 1) as usize, std::sync::atomic::Ordering::Relaxed);
                for x in a as usize..=b as usize {
                    let i = base + x;
                    ar[x] = match rt {
                        4 => fraccao_assente(g, i),
                        5..=7 if antes => g.oleo_antes_assente.get(i).copied().unwrap_or(1.0),
                        _ => 1.0,
                    };
                    let v = if antes {
                        g.oleo_antes[i]
                    } else {
                        altura(g, i, false)
                    };
                    hr[x] = if v > 0.0 {
                        alguma.store(true, std::sync::atomic::Ordering::Relaxed);
                        v
                    } else {
                        0.0
                    };
                }
            }
        });
        Self {
            r0,
            s,
            h,
            assente,
            presa: antes && rt == 7,
            alguma: alguma.into_inner(),
            #[cfg(test)]
            visitadas: visitadas.into_inner(),
        }
    }

    /// Devolve os planos ao rascunho do solver (o passo seguinte não aloca).
    pub(crate) fn devolve(self, g: &mut Grid) {
        g.scratch.ladeira_h = self.h;
        g.scratch.ladeira_assente = self.assente;
    }

    /// O termo de nivelamento de um eixo, já clampado: `dfilm` = a diferença de `film` entre os vizinhos, `o` = a da
    /// ladeira — pesada pela água desta célula (`f`) quando não se opõe a `dfilm` (ver [`FILM_CHEIO`]).
    /// ⛔ **Óleo sobre óleo** (`presa`, dono 31/10, `smoke_31_10_oleo/dono_4.png`): a que segura pára a água, nunca a
    /// empurra ao contrário — inteira, o relevo da fotografia movia a tinta grossa (film ≈ 0, o sinal de `dfilm` é
    /// ruído) e cada passada cavava e empilhava: caroços, p99 da altura 19,7 → 106,5 em 8 passadas do Thick Impasto
    /// (`sonda_oleo_furos`; sem a ladeira 30,0, presa 31,1; o excesso pesado pela água 54,7 — recusado). Sem o óleo na
    /// mão (a aguada ao pé do óleo seco, `a_agua_nao_sobe_a_ladeira_do_oleo`) fica inteira.
    #[inline]
    pub(crate) fn termo(
        &self,
        dfilm: f64,
        o: f64,
        f: f64,
        level_k: f64,
        dif: f64,
        clamp: f64,
    ) -> f64 {
        let o = if o * dfilm < 0.0 {
            if self.presa {
                o.clamp(-dfilm.abs(), dfilm.abs())
            } else {
                o
            }
        } else {
            o * (f / FILM_CHEIO).clamp(0.0, 1.0)
        };
        ((dfilm + o) * level_k * dif).clamp(-clamp, clamp)
    }

    /// O peso da ladeira na célula fina `fi`: a fracção assentada da tinta de lá (`1` fora das linhas).
    /// ⛔ **A tinta molhada não escorre com a água** (queixa do dono 31/10, `sonda_oleo_derrete`): sobre óleo ainda
    /// em suspensão a ladeira move a água e a advecção leva a massa com ela — o traço inteiro derretia (altura longe do
    /// cruzamento 20,0 → 13,5, as estrias somem, buracos escuros). Pesar só o empurrão não chega (13,9); a ladeira
    /// inteira pesa a tinta ASSENTADA.
    #[inline]
    pub(crate) fn assente(&self, fi: usize) -> f64 {
        fi.checked_sub(self.r0 * self.s)
            .and_then(|k| self.assente.get(k))
            .map_or(1.0, |v| f64::from(*v))
    }

    /// A altura na célula fina `fi` (`0` fora das linhas).
    #[inline]
    pub(crate) fn em(&self, fi: usize) -> f64 {
        fi.checked_sub(self.r0 * self.s)
            .and_then(|k| self.h.get(k))
            .map_or(0.0, |v| f64::from(*v))
    }
}

/// A ladeira das linhas finas que o passe de fluxo das linhas de FLUXO `by0..=by1` (bbox `gbx0..=gbx1`, faixa viva
/// `frow_lo/hi` já refrescada) amostra — só nas janelas que ele LÊ: por linha de fluxo `y`, as colunas
/// `probe(bx0−1)..=probe(bx1+1)` das linhas finas `probe(y−1)`, `probe(y)`, `probe(y+1)` (B13.A1: era a folha inteira).
/// Sem o Real Oil na mão, a altura de agora; com ele (`oleo_na_mao`), só a do óleo de ANTES do traço — o óleo que se
/// pinta corre como sempre (MEDIDO: com a ladeira também nele, as 10 miniaturas Oil/* mudavam 82–230 níveis; a
/// miniatura é um traço só, a fotografia dela é vazia ⇒ `None`, ao bit). Devolva-a com [`LadeiraDoOleo::devolve`].
pub(crate) fn do_passe(
    g: &mut Grid,
    (gbx0, by0, gbx1, by1): (i32, i32, i32, i32),
    oleo_na_mao: bool,
    mode: crate::par::Rows,
) -> Option<LadeiraDoOleo> {
    if g.oleo.is_empty() || rota() == 2 {
        return None;
    }
    let (gw, gh, rf) = (g.w as i32, g.h as i32, g.flow.rf);
    let p = |c: i32, w: i32| crate::flow::flow_probe(c, w, rf).max(0) as usize;
    let r0 = p(by0 - 1, gh);
    let r1 = p(by1 + 1, gh).min(g.cells / g.s - 1);
    if r1 < r0 || (oleo_na_mao && g.oleo_antes.len() != g.cells) {
        return None;
    }
    let mut jan = std::mem::take(&mut g.scratch.ladeira_jan);
    jan.clear();
    jan.resize(r1 + 1 - r0, (i32::MAX, i32::MIN));
    let tiles_on = g.spans_enabled && g.tiles_enabled;
    let tw = tile_words_for(g.s);
    let mut jt = std::mem::take(&mut g.scratch.ladeira_jan_tiles);
    jt.clear();
    if tiles_on {
        jt.resize(jan.len() * tw, 0);
    }
    for y in by0..=by1 {
        let (bx0, bx1) =
            crate::grid::span_x_of(&g.frow_lo, &g.frow_hi, g.spans_enabled, gbx0, gbx1, y);
        if bx0 > bx1 {
            continue;
        }
        let rows = [p(y - 1, gh), p(y, gh), p(y + 1, gh)];
        for (a, b) in span_runs(g.ftiles_row(y), bx0, bx1) {
            let (lo, hi) = (p(a - 1, gw) as i32, p(b + 1, gw) as i32);
            for r in rows {
                let Some(k) = r.checked_sub(r0).filter(|&k| k < jan.len()) else {
                    continue;
                };
                let j = &mut jan[k];
                *j = (j.0.min(lo), j.1.max(hi));
                if tiles_on {
                    let row = &mut jt[k * tw..(k + 1) * tw];
                    for t in (lo / TILE_FAIXA) as usize..=(hi / TILE_FAIXA) as usize {
                        row[t >> 6] |= 1 << (t & 63);
                    }
                }
            }
        }
    }
    let planos = (
        std::mem::take(&mut g.scratch.ladeira_h),
        std::mem::take(&mut g.scratch.ladeira_assente),
    );
    let tiles = tiles_on.then_some((&jt[..], tw));
    let l = LadeiraDoOleo::de(g, (r0, &jan, tiles), planos, oleo_na_mao, mode);
    g.scratch.ladeira_jan = jan;
    g.scratch.ladeira_jan_tiles = jt;
    #[cfg(test)]
    tests::observa(g, r0, r1, oleo_na_mao, &l);
    if l.alguma {
        Some(l)
    } else {
        l.devolve(g);
        None
    }
}

/// No início de um traço: com o Real Oil na mão e óleo na grade, fotografa a altura do óleo dos traços anteriores; senão
/// esvazia a fotografia. Custa uma passagem pela grade só quando se pinta óleo sobre óleo.
pub(crate) fn fotografa(g: &mut Grid, oleo_na_mao: bool) {
    if !oleo_na_mao || g.oleo.is_empty() {
        g.oleo_antes = Vec::new();
        g.oleo_antes_assente = Vec::new();
        return;
    }
    let inteira = matches!(rota(), 3 | 6);
    use rayon::prelude::*;
    let h: Vec<f32> = (0..g.cells)
        .into_par_iter()
        .map(|i| altura(g, i, inteira))
        .collect();
    g.oleo_antes = h;
    g.oleo_antes_assente = (0..g.cells)
        .into_par_iter()
        .map(|i| fraccao_assente(g, i))
        .collect();
}

/// `sett ÷ (sett + susp)` da célula `i` (`1` sem tinta em suspensão).
pub(crate) fn fraccao_assente(g: &Grid, i: usize) -> f32 {
    let (se, su) = (g.sett[i].max(0.0), g.susp[i]);
    if su > 0.0 { se / (se + su) } else { 1.0 }
}

#[cfg(test)]
#[path = "ladeira_do_oleo_tests.rs"]
mod tests;
