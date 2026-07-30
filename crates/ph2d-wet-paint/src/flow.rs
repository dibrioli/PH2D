//! **A grade de FLUXO** — a conversão fina ↔ fluxo, numa porta por pergunta.
//!
//! O motor tem duas grades e uma fronteira entre elas (plano 30):
//!
//! * a **FINA** — `film`, `susp`, `sett`, cores, `wet`, `paper`, `active`,
//!   `bloom`. É onde moram *"edges, granulation and fine linework"*, e é a
//!   metade que a foto do Enio cobra;
//! * a **de FLUXO** — `vel_x`, `vel_y`, `flow_x`, `flow_y`, com lado `rf` vezes
//!   menor. *"Fluid motion is inherently smooth"*, então ela pode ser grossa.
//!
//! ⚠️ **A fronteira é AMOSTRAGEM, não média, e isso foi MEDIDO** (a F1 do plano
//! 30, `tests/measure_flow_reduction.rs`): mediar os planos finos é `O(finas)`
//! por construção — **não encolhe com `rf`** — e a 3,69 ms custava mais que os
//! passes que alimentaria; amostrar UMA célula fina por bloco é `O(grossas)`,
//! **0,29 ms**, e encolhe por `rf²`. O preço honesto da amostra é que uma
//! feição de 1 px pode cair ENTRE dois pontos e o fluxo não a ver — pergunta de
//! **aparência**, decidida por render-and-look.
//!
//! # As duas portas TÊM de ser inversas
//!
//! [`fine_to_flow`] responde *"que célula de fluxo contém esta célula fina?"* e
//! [`flow_probe`] responde *"que célula fina esta célula de fluxo amostra?"*.
//! Elas se compõem numa identidade — `fine_to_flow(flow_probe(c)) == c` — e é
//! ela que impede o campo de sair deslocado meia célula: um passe que LÊ o film
//! da célula-amostra de um bloco e ESCREVE o fluxo daquele bloco tem de
//! concordar sobre qual bloco é. É a mesma lei `seed == sample` que as quatro
//! portas do `grid_map` do tool já pinam, no outro eixo.
//!
//! # `rf = 1` reduz LITERALMENTE
//!
//! Toda função aqui colapsa à expressão que o motor shipava quando `rf == 1` —
//! não a um épsilon dela: `(x - 1) / 1 + 1` **é** `x`, e `flow_coord` devolve o
//! próprio `xf`. É isso que torna o fingerprint do ADR-0134 uma rede de
//! segurança para as fases seguintes em vez de uma promessa.

/// Razão mínima: a grade de fluxo **é** a fina (o motor de hoje).
pub const MIN_FLOW_RATIO: usize = 1;

/// Razão máxima oferecida.
///
/// ⚠️ Não é um teto de recurso — é o ponto em que a grade de fluxo deixa de
/// resolver o PINCEL. O custo já é desprezível bem antes (a amostra mede
/// 0,096 ms a `rf = 8` contra 0,288 a 4), então o que aperta é a física: um
/// pincel que cobre menos de ~2 células de fluxo não tem gradiente de film
/// para nivelar, e a água para de correr. O número final é decisão do smoke —
/// o readout da UI mostra o tamanho resultante para que o limite seja VISÍVEL.
pub const MAX_FLOW_RATIO: usize = 16;

/// Aperta uma razão vinda de fora à faixa oferecida.
#[inline]
#[must_use]
pub const fn clamp_ratio(rf: usize) -> usize {
    if rf < MIN_FLOW_RATIO {
        MIN_FLOW_RATIO
    } else if rf > MAX_FLOW_RATIO {
        MAX_FLOW_RATIO
    } else {
        rf
    }
}

/// Quantas células de fluxo por eixo, para uma grade fina `w × h`.
///
/// Arredonda para CIMA: o último bloco pode ser parcial, e perder a coluna que
/// sobra seria perder a água que corre nela.
#[inline]
#[must_use]
pub const fn flow_dims(w: usize, h: usize, rf: usize) -> (usize, usize) {
    let r = clamp_ratio(rf);
    let fw = w.div_ceil(r);
    let fh = h.div_ceil(r);
    (if fw == 0 { 1 } else { fw }, if fh == 0 { 1 } else { fh })
}

/// **Que célula de FLUXO contém esta célula fina?** (por eixo, interior `1..=w`)
///
/// `rf = 1` ⇒ `(x - 1) / 1 + 1` = `x`, literalmente.
#[inline]
#[must_use]
pub const fn fine_to_flow(x: i32, rf: usize) -> i32 {
    if x <= 0 {
        return 0; // o anel de pad de baixo/esquerda
    }
    (x - 1) / clamp_ratio(rf) as i32 + 1
}

/// **Que célula FINA esta célula de fluxo amostra?** (por eixo)
///
/// O ponto do MEIO do bloco, clampado ao interior — o meio é menos enviesado
/// que a quina, e num bloco parcial (o último, quando `w` não é múltiplo de
/// `rf`) ele cairia fora da folha.
///
/// `rf = 1` ⇒ `(c - 1) * 1 + 1 + 0` = `c`, literalmente.
#[inline]
#[must_use]
pub const fn flow_probe(c: i32, w: i32, rf: usize) -> i32 {
    if c <= 0 {
        return 0; // pad ↔ pad: o anel de fluxo amostra o anel fino
    }
    let r = clamp_ratio(rf) as i32;
    let fw = (w + r - 1) / r; // div_ceil
    if c > fw {
        return w + 1; // idem, do outro lado
    }
    let half = (r - 1) / 2;
    let x = (c - 1) * r + 1 + half;
    if x > w { w } else { x }
}

/// A coordenada de FLUXO (fracionária) de uma posição fina — a inversa contínua
/// do [`flow_probe`], que é o que a amostragem bilinear do `advect` percorre.
///
/// `rf = 1` ⇒ `(xf - 1 - 0) / 1 + 1` = `xf`, **em `f64` exato** (divisão por
/// 1.0 e o par −1/+1 se cancelam sem arredondamento).
#[inline]
#[must_use]
pub fn flow_coord(xf: f64, rf: usize) -> f64 {
    let r = clamp_ratio(rf);
    if r == 1 {
        return xf;
    }
    let half = ((r - 1) / 2) as f64;
    (xf - 1.0 - half) / r as f64 + 1.0
}

/// A GEOMETRIA da grade de fluxo — as dimensões derivadas de `(w, h, rf)`,
/// computadas UMA vez no nascimento do grid.
///
/// ⚠️ Em `rf = 1` **todo campo iguala o da grade fina** (`w`, `s`, `rows`,
/// `cells`), então todo índice que o motor já calculava continua exato — é
/// isso que faz do fingerprint do ADR-0134 a rede de segurança desta wave.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlowGeom {
    /// Células FINAS por célula de fluxo.
    pub rf: usize,
    /// `1 / rf` pré-computado.
    ///
    /// ⚠️ Não é micro-otimização: a [`flow_at_point`] roda **por célula fina**
    /// no `advect`, o laço mais quente do motor, e duas divisões de ponto
    /// flutuante ali são visíveis no relógio do produto. A divisão INTEIRA que
    /// morava no mesmo lugar já custou 2,6 ms num passo (medido).
    pub inv_rf: f64,
    pub w: usize,
    pub h: usize,
    /// Stride = `w + 2` (o mesmo anel de pad da grade fina).
    pub s: usize,
    pub rows: usize,
    pub cells: usize,
}

impl FlowGeom {
    #[must_use]
    pub const fn new(fine_w: usize, fine_h: usize, rf: usize) -> Self {
        let rf = clamp_ratio(rf);
        let (w, h) = flow_dims(fine_w, fine_h, rf);
        let s = w + 2;
        let rows = h + 2;
        FlowGeom {
            rf,
            inv_rf: 1.0 / rf as f64,
            w,
            h,
            s,
            rows,
            cells: s * rows,
        }
    }

    /// O índice de fluxo de uma célula de fluxo `(cx, cy)`.
    #[inline]
    #[must_use]
    pub const fn idx(&self, cx: i32, cy: i32) -> usize {
        cx as usize + cy as usize * self.s
    }

    /// A grade de fluxo **é** a fina? (o caminho que o motor sempre rodou)
    #[inline]
    #[must_use]
    pub const fn is_identity(&self) -> bool {
        self.rf == 1
    }
}

/// **O índice FINO que a célula de fluxo `(cx, cy)` amostra** — a forma livre
/// do probe, para os passes que desestruturam o `Grid` (split-borrow) e não
/// podem chamar um método.
///
/// `rf = 1` ⇒ `cx + cy * s`, o índice fino que o motor sempre calculou.
#[inline]
#[must_use]
pub const fn probe_idx(cx: i32, cy: i32, w: i32, h: i32, s: usize, rf: usize) -> usize {
    let x = flow_probe(cx, w, rf);
    let y = flow_probe(cy, h, rf);
    x as usize + y as usize * s
}

/// **O fator de DIFERENÇA FINITA da grade de fluxo.**
///
/// A regra de unidade da wave inteira, em uma linha: a velocidade é sempre
/// medida em **células FINAS por frame** (é o que o `advect` back-traça e o que
/// o `maxVelocity` significa), mas as diferenças finitas de um passe grosso são
/// tomadas sobre vizinhos que estão `rf` células finas de distância — logo
///
/// ```text
///   ∂/∂x_fino  =  (1 / rf) · ∂/∂x_fluxo
/// ```
///
/// ⇒ **toda diferença finita tomada na grade de fluxo é multiplicada por
/// isto**, e nada mais é. Médias (a viscosidade, o `smooth_velocity`) são
/// adimensionais e NÃO levam o fator; velocidades injetadas direto (o push do
/// backrun, o do fingering, a gravidade) já estão na unidade certa.
///
/// ⚠️ Em `rf = 1` o fator é `1.0`, e `x * 1.0` é **exatamente `x`** em
/// IEEE-754 para todo finito — a byte-identidade não se pede a um épsilon.
#[inline]
#[must_use]
pub fn diff_scale(rf: usize) -> f64 {
    1.0 / clamp_ratio(rf) as f64
}

/// **O fluxo numa posição FINA fracionária** — a amostragem bilinear que o
/// `advect` percorre ao back-traçar.
///
/// ⚠️ **Bilinear, nunca nearest.** Com o valor CHATO do bloco, todas as `rf²`
/// células finas de um bloco andariam com exatamente a mesma velocidade, e a
/// frente de tinta ganharia degraus do tamanho do bloco — que é precisamente o
/// artefato que esta wave existe para remover.
///
/// Em `rf = 1` a coordenada é a própria posição fina, e a expressão é a que o
/// `advect` sempre computou, termo a termo, na mesma ordem.
#[inline]
#[must_use]
pub fn flow_at_point(fx: &[f32], fy: &[f32], geom: &FlowGeom, sx: f64, sy: f64) -> (f64, f64) {
    let half = ((geom.rf - 1) / 2) as f64;
    let (u, v) = if geom.is_identity() {
        (sx, sy)
    } else {
        (
            (sx - 1.0 - half) * geom.inv_rf + 1.0,
            (sy - 1.0 - half) * geom.inv_rf + 1.0,
        )
    };
    // Clampa ao interior: em `rf > 1` o meio do primeiro bloco fica ADIANTE da
    // borda, então `u` pode cair abaixo de 1 sem que a posição fina o faça.
    let u = u.clamp(1.0, geom.w as f64);
    let v = v.clamp(1.0, geom.h as f64);
    let x0 = u as i64; // positivo ⇒ trunc == floor
    let y0 = v as i64;
    let a = u - x0 as f64;
    let b = v - y0 as f64;
    let i00 = x0 as usize + y0 as usize * geom.s;
    let i10 = i00 + 1;
    let i01 = i00 + geom.s;
    let i11 = i01 + 1;
    let w00 = (1.0 - a) * (1.0 - b);
    let w10 = a * (1.0 - b);
    let w01 = (1.0 - a) * b;
    let w11 = a * b;
    (
        fx[i00] as f64 * w00 + fx[i10] as f64 * w10 + fx[i01] as f64 * w01 + fx[i11] as f64 * w11,
        fy[i00] as f64 * w00 + fy[i10] as f64 * w10 + fy[i01] as f64 * w01 + fy[i11] as f64 * w11,
    )
}

/// **O fluxo NA célula fina `(x, y)`**, cujo índice fino já é conhecido.
///
/// ⚠️ A rota de `rf = 1` devolve o valor no índice **verbatim** — não a forma
/// bilinear com pesos zerados. As duas dariam o mesmo número, mas a identidade
/// se CONSTRÓI, não se argumenta: `f00 * 1.0 + f10 * 0.0 + …` obriga a
/// raciocinar sobre `-0.0`, e raciocínio é o que o `x * 1.0` do
/// [`diff_scale`] existe para dispensar.
#[inline]
#[must_use]
pub fn flow_at_cell(
    fx: &[f32],
    fy: &[f32],
    geom: &FlowGeom,
    x: i32,
    y: i32,
    fine_i: usize,
) -> (f64, f64) {
    if geom.is_identity() {
        return (fx[fine_i] as f64, fy[fine_i] as f64);
    }
    flow_at_point(fx, fy, geom, f64::from(x), f64::from(y))
}

/// Esta célula FINA é a que a sua célula de fluxo AMOSTRA?
///
/// É quem responde *"quem escreve a velocidade deste bloco?"*: exatamente uma
/// célula fina por bloco, a mesma que os passes de fluxo leem. Em `rf = 1` toda
/// célula é o próprio probe, então a resposta é sempre sim.
#[inline]
#[must_use]
pub const fn is_probe_cell(x: i32, y: i32, w: i32, h: i32, rf: usize) -> bool {
    flow_probe(fine_to_flow(x, rf), w, rf) == x && flow_probe(fine_to_flow(y, rf), h, rf) == y
}

/// A bbox de FLUXO que cobre uma bbox FINA. Vazia continua vazia.
#[inline]
#[must_use]
pub const fn flow_bbox(b: (i32, i32, i32, i32), rf: usize) -> (i32, i32, i32, i32) {
    let (x0, y0, x1, y1) = b;
    if x1 < x0 || y1 < y0 {
        return (0, 0, -1, -1);
    }
    (
        fine_to_flow(x0, rf),
        fine_to_flow(y0, rf),
        fine_to_flow(x1, rf),
        fine_to_flow(y1, rf),
    )
}

/// Projeta a faixa viva FINA (por linha) na grade de fluxo.
///
/// A faixa de uma linha de fluxo é a UNIÃO das faixas finas do bloco — um
/// superconjunto, como a fina é: ela só pode fazer um passe VISITAR uma célula
/// a mais, nunca pular uma viva.
pub fn project_spans(
    row_lo: &[i32],
    row_hi: &[i32],
    fine_h: usize,
    geom: &FlowGeom,
    out_lo: &mut [i32],
    out_hi: &mut [i32],
) {
    out_lo.fill(i32::MAX);
    out_hi.fill(i32::MIN);
    for y in 1..=fine_h as i32 {
        let r = y as usize;
        if row_lo[r] > row_hi[r] {
            continue;
        }
        let fy = fine_to_flow(y, geom.rf) as usize;
        let lo = fine_to_flow(row_lo[r].max(1), geom.rf);
        let hi = fine_to_flow(row_hi[r].min(geom.w as i32 * geom.rf as i32), geom.rf);
        if lo < out_lo[fy] {
            out_lo[fy] = lo;
        }
        if hi > out_hi[fy] {
            out_hi[fy] = hi;
        }
    }
}

#[cfg(test)]
#[path = "flow_tests.rs"]
mod flow_tests;
