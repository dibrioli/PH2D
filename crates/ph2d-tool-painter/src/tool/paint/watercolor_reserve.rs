//! **A RESERVA de pigmento do traço é um NÍVEL que o carimbo DISPUTA — e um campo que se LÊ macio**
//! (doc 40/41, report do Enio 2026-09-20: *«ao traçar voltando sobre o próprio traço o pincel deixa
//! por baixo uma borda plana dura pixelada, e nem Rewet nem Smudge no máximo a desfazem»*).
//!
//! ## O defeito, medido
//!
//! O mapa da reserva (MIX-1) era `max_d(v_d · rampa_d)`, com a rampa a cair nos últimos `15 %` do
//! raio. Onde a perna de VOLTA (pálida, `v₂`) cobre a beira da perna de IDA (escura, `v₁`), o `max`
//! entrega um degrau de largura `0,15·r·(1 − v₂/v₁)` — `~4 px` a `r = 45`, `1 px` a `r = 8` — e o
//! composite lia-o por **vizinho-mais-próximo em coordenadas DEFORMADAS** pelo Ragged Edge: escada
//! de pixel. A régua (`tests/watercolor_selfseam.rs`) lia a costura tão dura como a borda EXTERNA
//! do traço, que é nítida de propósito (`0,41` do contraste num pixel a `r = 32`).
//!
//! ## A lei, e de onde ela vem
//!
//! O oráculo (libmypaint 1.6.1, ISC — `docs/Painter/ferramentas/oraculo_costura/`) mede que num
//! motor de dab macio a costura interna tem a largura do **perfil do próprio dab** (razão `1,00`
//! contra a borda do dab). Lá isso cai de graça porque cada dab mistura com o peso do seu perfil.
//! Aqui a regra de domínio é outra e FICA — o `max` (*re-entintar um rasto pálido restaura-o; um
//! pincel esgotado não clareia tinta escura*) —, logo a lei transpõe-se assim:
//!
//! 1. **Cada dab DISPUTA o nível com o peso do que DEPOSITA ali** — `q = feather(dn)`, o perfil do
//!    depósito, e não uma rampa inventada. O nível de um pixel é `L = max_d(v_d·q_d) / max_d(q_d)`.
//!    Numa passada só isso é `v` (o mesmo dab ganha em cima e em baixo); numa sobreposição o nível
//!    escuro cede ao pálido ao longo do PRÓPRIO feather da perna escura — contínuo, e à escala do
//!    pincel (`~0,33·r` de 10 a 90 %), que é o que o oráculo mede.
//! 2. **O afilamento da beira é geometria e vive à parte** — `T = rampa(proximidade)`. A reserva que
//!    o composite multiplica é `T · S`. Numa passada só `T · v` é o mapa de sempre: a anatomia da
//!    borda externa, que o dono aprovou, não se mexe.
//! 3. **O campo lê-se MACIO**: `S` = média de `L` pesada por `q` numa caixa de raio `R` que **não
//!    atravessa papel seco** (as somas correm por TROÇOS contíguos da lavagem — difusão sem fluxo na
//!    silhueta; dois washes que não se tocam não se influenciam, qualquer que seja `R`), amostrada
//!    por **bilinear** nas coordenadas deformadas. O `R` é do DONO do pixel (estilo capturado no
//!    pen-down), arredonda o joelho do `max`, e **cresce com o Rewet** (`Rewet × Bleed`): a água
//!    re-molha e o pigmento difunde mais — é por aqui que o Rewet passa a alcançar a costura.
//!
//! ⚠️ **As somas são INTEIRAS.** O `box_blur` da casa soma `f32` desde a origem da JANELA, logo o
//! arredondamento depende de onde a janela começa; aqui `Σ L·q` e `Σ q` são `u64` exactos e a
//! divisão é uma só ⇒ o campo é função do MAPA e não da janela (`incremental ≡ full` por
//! construção, não por tolerância).
//!
//! ⚠️ **Charge = 1 nunca chega aqui**: sem mixer o mapa não existe e o composite fica byte-idêntico.

use rayon::prelude::*;

use super::watercolor_field::{WetStrokeStyle, sample_bilinear};

/// A casca externa do dab em que a reserva afila até zero (`dn ∈ [1−RAMP, 1]`) — a anatomia da borda
/// de um traço com Charge < 1 (MIX-1, 2026-07-08). Era aplicada DENTRO do `max`; hoje é o factor `T`.
pub(super) const RESERVE_RIM_RAMP: f32 = 0.15;

/// O raio de base do alisamento, como fracção do raio do pincel — arredonda o joelho que o `max`
/// deixa onde o nível escuro encontra o pálido. ⚠️ Medido, não escolhido: ver a tabela em
/// `tests/watercolor_selfseam.rs::measure_the_self_seam` (doc 41 §5).
pub(super) const RESERVE_ROUND_FRAC: f32 = 0.10;

// O CONTROLO dos gates: a lei de ANTES (`max(v·rampa)` lido por vizinho-mais-próximo), ligável por
// thread e só em teste. Sem ele a régua do produto não tinha como provar que a fixtura CONTÉM o
// fenómeno — e um gate cuja fixtura deixou de o conter fica verde a afirmar nada.
#[cfg(test)]
thread_local! {
    pub(in crate::tool::paint) static LEI_ANTIGA: std::cell::Cell<bool> =
        const { std::cell::Cell::new(false) };
}

#[inline]
fn lei_antiga() -> bool {
    #[cfg(test)]
    return LEI_ANTIGA.with(std::cell::Cell::get);
    #[cfg(not(test))]
    false
}

/// Proximidade quantizada de um dab a um pixel: `255` no centro, `1` na beira, `0` = fora. É a
/// única grandeza geométrica guardada — o peso da disputa e o afilamento derivam dela por tabela.
#[inline]
pub(super) fn prox8(dn: f32) -> u8 {
    (((1.0 - dn) * 255.0).round() as i32).clamp(1, 255) as u8
}

/// A CAUDA da disputa a seco: a do próprio depósito. O `feather` é um planalto (`1,00 → 0,92` até
/// `dn = 0,62`) e uma cauda linear até zero; a disputa normaliza pelo planalto, logo dentro dele o
/// peso é `1` CHEIO — **o miolo de uma perna escura guarda o nível dela ao bit** sob qualquer
/// passada pálida (a regra de domínio do `max`) — e quem cede é só a cauda, onde o depósito de
/// facto esmorece. Não é um número escolhido: é `1 − 0,62`, lido do `feather` (gate
/// `the_dry_claim_is_the_deposits_own_feather`).
pub(super) const CLAIM_TAIL_DRY: f32 = 0.38;
fn claim_table(tail: f32) -> [u32; 256] {
    let mut t = [0u32; 256];
    for (p, q) in t.iter_mut().enumerate().skip(1) {
        let claim = ((p as f32 / 255.0) / tail).min(1.0);
        *q = ((claim * 255.0).round() as u32).clamp(1, 255);
    }
    t
}

/// O peso da disputa por proximidade, `q8[p] ∈ 1..=255` (nunca `0` onde há lavagem: é o
/// denominador) — `min(1, (1−dn)/cauda)`. É também o PESO da média do campo.
///
/// ⛔ **RECUSA MEDIDA — o Rewet NÃO alarga esta cauda** (doc 41 §6). Duas redacções o tentaram: as
/// duas caudas largas (`9,3 → 10,8 px` a `r = 32`, quando a conta pedia o dobro) e só a de quem já
/// está no papel (`7 → 0 px` na lei nua). Nas duas o dab pálido deixa de estar SATURADO sobre a
/// cauda da escura, a transição passa a ser a razão `q₁/q₂` — que muda depressa — e a costura
/// DESLOCA-SE em vez de abrir. *A disputa é local e a sua largura é a da sobreposição; quem alarga
/// para lá dela é a DIFUSÃO, que lê vizinhança* — é por aí que o Rewet entra ([`reserve_radius`]).
fn claim_lut() -> &'static [u32; 256] {
    static LUT: std::sync::OnceLock<[u32; 256]> = std::sync::OnceLock::new();
    LUT.get_or_init(|| claim_table(CLAIM_TAIL_DRY))
}

/// `T[p]` — o afilamento da beira na proximidade `p`.
fn taper_lut() -> &'static [f32; 256] {
    static LUT: std::sync::OnceLock<[f32; 256]> = std::sync::OnceLock::new();
    LUT.get_or_init(|| {
        let mut t = [0.0f32; 256];
        for (p, v) in t.iter_mut().enumerate() {
            *v = ((p as f32 / 255.0) / RESERVE_RIM_RAMP).min(1.0);
        }
        t
    })
}

/// **A disputa** — um dab de reserva `v` chega a um pixel à distância normalizada `dn`.
/// `N = L·q` é o que cada lado reclama; ganha o maior, e o nível é `N / max(q)`. Aritmética
/// inteira ⇒ determinística, e IDEMPOTENTE quando o dab não ganha nada (`L` volta a si mesmo).
#[inline]
pub(super) fn splat_level(level: &mut u8, prox: &mut u8, dn: f32, v: f32) {
    if lei_antiga() {
        let old = (v * ((1.0 - dn) / RESERVE_RIM_RAMP).min(1.0) * 255.0) as u8;
        (*level, *prox) = ((*level).max(old), 255);
        return;
    }
    let lut = claim_lut();
    let p_new = prox8(dn);
    let (q_new, q_old) = (lut[p_new as usize], lut[*prox as usize]);
    let n_new = ((v.clamp(0.0, 1.0) * 255.0).round() as u32) * q_new;
    let n_old = u32::from(*level) * q_old;
    if n_new <= n_old && p_new <= *prox {
        return;
    }
    let q = q_old.max(q_new);
    let n = n_old.max(n_new);
    *level = ((n + q / 2) / q).min(255) as u8;
    *prox = (*prox).max(p_new);
}

/// **Smudge sobre o traço VIVO** — o gémeo escalar do `smear_dab`: arrasta o NÍVEL de `from` para
/// `to` dentro da pegada do dab (`dest = lerp(dest, nível[dest − passo], peso)`), só entre pixels que
/// a lavagem já tocou. O `smear_wet_base` arrasta a tinta ASSADA por baixo; a costura do próprio
/// traço vive nestes planos, que ele nunca via — medido: Smudge `1,0` deixava o traço byte-idêntico
/// ao de Smudge `0`. Corre POR DAB, antes do depósito desse dab ⇒ função do caminho, não do polling.
#[allow(clippy::too_many_arguments)] // a costura do kernel: cada entrada é um facto distinto do dab
pub(super) fn smear_level(
    level: &mut [u8],
    prox: &[u8],
    (fw, fh): (usize, usize),
    (from, to): ([f32; 2], [f32; 2]),
    radius: f32,
    weight: impl Fn(f32) -> f32,
    wrap: [bool; 2],
    lifted: &mut Vec<u8>,
) {
    let step = [
        (to[0].round() as i64) - (from[0].round() as i64),
        (to[1].round() as i64) - (from[1].round() as i64),
    ];
    if radius <= 0.0 || (step[0] == 0 && step[1] == 0) {
        return;
    }
    let (w, h) = (fw as i64, fh as i64);
    let x0 = (to[0] - radius).floor().max(0.0) as i64;
    let y0 = (to[1] - radius).floor().max(0.0) as i64;
    let x1 = ((to[0] + radius).ceil() as i64).min(w);
    let y1 = ((to[1] + radius).ceil() as i64).min(h);
    if x1 <= x0 || y1 <= y0 {
        return;
    }
    let bw = (x1 - x0) as usize;
    // Levanta PRIMEIRO (origem e destino sobrepõem-se): `0` = a origem é papel seco, nada a trazer.
    lifted.clear();
    lifted.resize(bw * (y1 - y0) as usize, 0);
    let src = |v: i64, n: i64, wrap: bool| -> Option<i64> {
        if wrap {
            Some(v.rem_euclid(n))
        } else {
            (0..n).contains(&v).then_some(v)
        }
    };
    for y in y0..y1 {
        let Some(sy) = src(y - step[1], h, wrap[1]) else {
            continue;
        };
        for x in x0..x1 {
            if let Some(sx) = src(x - step[0], w, wrap[0]) {
                let si = (sy * w + sx) as usize;
                if prox[si] > 0 {
                    // `0` quer dizer «seco»: um nível esgotado (0) sobe a 1 para não se confundir.
                    lifted[(y - y0) as usize * bw + (x - x0) as usize] = level[si].max(1);
                }
            }
        }
    }
    let inv_r = 1.0 / radius;
    for y in y0..y1 {
        let dy = y as f32 + 0.5 - to[1];
        for x in x0..x1 {
            let di = (y * w + x) as usize;
            let l = lifted[(y - y0) as usize * bw + (x - x0) as usize];
            if l == 0 || prox[di] == 0 {
                continue;
            }
            let dx = x as f32 + 0.5 - to[0];
            let t = (dx * dx + dy * dy).sqrt() * inv_r;
            if t >= 1.0 {
                continue;
            }
            let k = weight(t);
            if k > 0.0 {
                let d = f32::from(level[di]);
                level[di] = (d + (f32::from(l) - d) * k).round().clamp(0.0, 255.0) as u8;
            }
        }
    }
}

/// Quanto do RAIO DO PINCEL o Rewet cheio difunde. ⚠️ Em pixels do Bleed (a 1.ª redacção) o Rewet
/// não se via num pincel grande: `22,0 → 22,1 px` de costura a `r = 96`. A difusão tem de ter a
/// escala do pincel, como a costura que ela abre. Medido (doc 41 §5): com `0,25` o Rewet cheio leva
/// a costura de `~0,25·r` para `~0,40·r` em todos os raios.
pub(super) const REWET_DIFFUSE_FRAC: f32 = 0.25;

/// O raio do alisamento de UM dono: a base arredonda o joelho da disputa (limitada ao `core_r`); o
/// **Rewet** difunde — `Rewet × max(Bleed, 0,25·r)`. ⚠️ Acima do `core_r`/Bleed a janela do composite
/// tem de o saber: [`WetSessionStyles::reserve_reach`] entra no `reach` dela enquanto o mapa vive.
pub(in crate::tool::paint) fn reserve_radius(
    radius_px: f32,
    core_r: usize,
    spread_px: usize,
    wet: f32,
) -> u16 {
    let base = ((radius_px * RESERVE_ROUND_FRAC).round() as usize).clamp(1, core_r.max(1));
    let reach = (spread_px as f32).max(radius_px * REWET_DIFFUSE_FRAC);
    let rewet = (wet.clamp(0.0, 1.0) * reach).round() as usize;
    base.max(rewet).min(RESERVE_R_MAX) as u16
}

/// O tecto do raio, e o recurso é a JANELA do composite: ela cresce `R` para cada lado (duas vezes —
/// saída e leitura), e a área dela é o custo do quadro. `256` é `0,25 ×` o maior raio que o painel
/// oferece (`1024 px`); acima disso o Rewet satura em vez de estourar o quadro.
pub(super) const RESERVE_R_MAX: usize = 256;

/// A janela de leitura do composite: `(largura do canvas, n.º de pixels, origem, extensão)`.
pub(super) type ReserveWindow = (usize, usize, (usize, usize), (usize, usize));

impl super::PainterTool {
    /// A porta do composite: os dois planos do traço + a tabela de estilos da sessão.
    pub(super) fn reserve_fields(
        &self,
        win: ReserveWindow,
        cur: &WetStrokeStyle,
    ) -> Option<ReserveFields> {
        let planes = (
            &self.paint.stroke_deplete[..],
            &self.paint.stroke_deplete_prox[..],
        );
        ReserveFields::build(planes, win, &self.paint.wet_styles.table, cur)
    }
}

/// O campo `T·S` na janela de leitura, um por raio distinto entre os donos da sessão.
pub(super) struct ReserveFields {
    by_r: Vec<(u16, Vec<f32>)>,
    rw: usize,
    rh: usize,
    /// Só o CONTROLO de teste o liga: leitura por vizinho-mais-próximo, como era.
    nearest: bool,
}

impl ReserveFields {
    /// `level`/`prox` são os planos do canvas inteiro (`fw` de largura); a janela é
    /// `(rx0, ry0, rw, rh)`. `None` quando os planos não existem (mixer nunca ligou).
    pub(super) fn build(
        (level, prox): (&[u8], &[u8]),
        (fw, n, (rx0, ry0), (rw, rh)): ReserveWindow,
        table: &[WetStrokeStyle],
        cur: &WetStrokeStyle,
    ) -> Option<Self> {
        if level.len() != n || prox.len() != n || n == 0 || rw == 0 || rh == 0 {
            return None;
        }
        if lei_antiga() {
            let mut raw = vec![0.0f32; rw * rh];
            for (wy, row) in raw.chunks_mut(rw).enumerate() {
                for (wx, o) in row.iter_mut().enumerate() {
                    *o = f32::from(level[(ry0 + wy) * fw + rx0 + wx]) / 255.0;
                }
            }
            let by_r = vec![(cur.reserve_r, raw)];
            return Some(Self {
                by_r,
                rw,
                rh,
                nearest: true,
            });
        }
        let lut = claim_lut();
        let n = rw * rh;
        // Numerador e denominador na janela, JUNTOS: `[L·q, q]` (0 fora da lavagem). O `q` é também a
        // MÁSCARA das duas caixas — os dois campos correm os mesmos troços, logo uma passagem soma os
        // dois. ⚠️ Era um campo de cada vez, com a máscara transposta de novo a cada caixa.
        let mut ab = vec![[0u64; 2]; n];
        let mut b = vec![0u64; n];
        ab.par_chunks_mut(rw)
            .zip(b.par_chunks_mut(rw))
            .enumerate()
            .for_each(|(wy, (abr, br))| {
                let s = (ry0 + wy) * fw + rx0;
                for wx in 0..rw {
                    let q = u64::from(lut[prox[s + wx] as usize]);
                    abr[wx] = [u64::from(level[s + wx]) * q, q];
                    br[wx] = q;
                }
            });
        let mut radii: Vec<u16> = table.iter().map(|s| s.reserve_r).collect();
        radii.push(cur.reserve_r);
        radii.sort_unstable();
        radii.dedup();
        let taper = taper_lut();
        let mut caixa = Caixa::nova(&b, rw, rh);
        let by_r = radii
            .into_iter()
            .map(|r| {
                // Duas caixas (`r₁ + r₂ = R`) ⇒ núcleo triangular: a transição sai C¹ e o suporte
                // total continua a ser `R`, que é o que a janela garante.
                let (r1, r2) = ((r / 2) as usize, (r - r / 2) as usize);
                let mut s = ab.clone();
                for rr in [r1, r2] {
                    if rr > 0 {
                        caixa.aplica(&mut s, rr);
                    }
                }
                let mut out = vec![0.0f32; n];
                out.par_chunks_mut(rw).enumerate().for_each(|(wy, orow)| {
                    let base = (ry0 + wy) * fw + rx0;
                    for (wx, o) in orow.iter_mut().enumerate() {
                        let [sa, sb] = s[wy * rw + wx];
                        if sb > 0 {
                            let lvl = (sa as f64 / (sb as f64 * 255.0)) as f32;
                            *o = taper[prox[base + wx] as usize] * lvl.min(1.0);
                        }
                    }
                });
                (r, out)
            })
            .collect();
        Some(Self {
            by_r,
            rw,
            rh,
            nearest: false,
        })
    }

    /// A reserva em `(sx, sy)` (coordenadas da janela, já deformadas) para um dono de raio `r`.
    #[inline]
    pub(super) fn sample(&self, r: u16, sx: f32, sy: f32) -> f32 {
        let f = self
            .by_r
            .iter()
            .find(|(br, _)| *br == r)
            .map_or(&self.by_r[0].1, |(_, f)| f);
        if self.nearest {
            let x = (sx.max(0.0) as usize).min(self.rw - 1);
            return f[(sy.max(0.0) as usize).min(self.rh - 1) * self.rw + x];
        }
        sample_bilinear(f, self.rw, self.rh, sx, sy)
    }
}

/// Soma em caixa separável **restrita aos TROÇOS contíguos da máscara** (`mask > 0`): a caixa de
/// cada pixel é cortada nas pontas do troço a que ele pertence, na horizontal e depois na vertical.
/// Nenhuma soma atravessa um pixel seco ⇒ difusão sem fluxo na silhueta. `O(n)` por somas de
/// prefixo por troço; inteiros ⇒ exacta e independente da origem da janela.
///
/// ⭐ **As somas são INTEIRAS, logo a ORDEM em que se fazem não muda um bit** — é o que deixa esta
/// casa escolher a arrumação pela memória e não pela aritmética (ao contrário do `box_blur` em
/// `f32`, onde uma soma reordenada é outro número). O que ela guarda entre caixas:
/// - a máscara TRANSPOSTA, calculada UMA vez por janela (era transposta de novo a cada caixa, dos
///   dois campos: quatro vezes por raio);
/// - três planos de rascunho reusados por todas as caixas e todos os raios (eram cinco planos
///   alocados e ZERADOS por caixa — a medição do produto via `alloc_zeroed` como a maior fatia da
///   thread principal). Cada passagem escreve TODOS os píxeis, os de fora da lavagem a zero, logo o
///   rascunho nunca precisa de nascer limpo.
struct Caixa<'m> {
    mask: &'m [u64],
    mask_t: Vec<u64>,
    w: usize,
    h: usize,
    linhas: Vec<[u64; 2]>,
    colunas: Vec<[u64; 2]>,
    colunas_out: Vec<[u64; 2]>,
}

impl<'m> Caixa<'m> {
    fn nova(mask: &'m [u64], w: usize, h: usize) -> Self {
        let mut mask_t = vec![0u64; w * h];
        transpoe(mask, w, h, &mut mask_t);
        let n = w * h;
        Self {
            mask,
            mask_t,
            w,
            h,
            linhas: vec![[0; 2]; n],
            colunas: vec![[0; 2]; n],
            colunas_out: vec![[0; 2]; n],
        }
    }

    /// Uma caixa de raio `r` sobre os dois campos de `s`, no lugar.
    fn aplica(&mut self, s: &mut [[u64; 2]], r: usize) {
        let (w, h) = (self.w, self.h);
        passagem(s, self.mask, w, r, &mut self.linhas);
        // Vertical = a mesma passagem sobre o TRANSPOSTO (linhas contíguas para o rayon), e de volta.
        transpoe(&self.linhas, w, h, &mut self.colunas);
        passagem(&self.colunas, &self.mask_t, h, r, &mut self.colunas_out);
        transpoe(&self.colunas_out, h, w, s);
    }
}

/// Uma passagem por LINHAS: cada linha de `out` é a soma em caixa de raio `r` da linha de `src`,
/// cortada nos troços da máscara. Escreve a linha inteira (zero fora dos troços).
fn passagem(src: &[[u64; 2]], mask: &[u64], w: usize, r: usize, out: &mut [[u64; 2]]) {
    out.par_chunks_mut(w)
        .zip(src.par_chunks(w).zip(mask.par_chunks(w)))
        .for_each_init(
            || vec![[0u64; 2]; w + 1],
            |pref, (orow, (srow, mrow))| {
                let mut x = 0;
                while x < w {
                    if mrow[x] == 0 {
                        orow[x] = [0, 0];
                        x += 1;
                        continue;
                    }
                    let lo = x;
                    while x < w && mrow[x] > 0 {
                        x += 1;
                    }
                    let hi = x; // troço [lo, hi)
                    pref[lo] = [0, 0];
                    for i in lo..hi {
                        pref[i + 1] = [pref[i][0] + srow[i][0], pref[i][1] + srow[i][1]];
                    }
                    for (i, o) in orow.iter_mut().enumerate().take(hi).skip(lo) {
                        let a = i.saturating_sub(r).max(lo);
                        let b = (i + r + 1).min(hi);
                        *o = [pref[b][0] - pref[a][0], pref[b][1] - pref[a][1]];
                    }
                }
            },
        );
}

/// `out[x·h + y] = src[y·w + x]`, colunas de `out` em paralelo.
fn transpoe<T: Copy + Send + Sync>(src: &[T], w: usize, h: usize, out: &mut [T]) {
    out.par_chunks_mut(h).enumerate().for_each(|(x, col)| {
        for (y, c) in col.iter_mut().enumerate() {
            *c = src[y * w + x];
        }
    });
}

#[cfg(test)]
#[path = "watercolor_reserve_tests.rs"]
mod tests;
