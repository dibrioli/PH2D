//! **O espaço de autoria do catálogo** — a caixa unitária, com `v` para BAIXO.
//!
//! ## Por que este módulo existe
//!
//! O mundo do PH2D é **Y-para-cima**: a câmera leva mundo→tela com `scale(k, −k)`
//! (vide `vec_glyph`), então `y` maior é mais ALTO na tela. Toda referência de onde as
//! formas vêm — SVG, a geometria pré-definida do OOXML, os stencils do draw.io — é
//! **Y-para-baixo**. Escrever a fórmula de uma forma "como está na referência" e assiná-la
//! direto em coordenadas de mundo é, portanto, um espelhamento vertical silencioso: o cone
//! nasce de ponta-cabeça, o coração com o bico pra cima, o rabo do balão apontando pro céu.
//!
//! Foi exatamente o que aconteceu (2026-07-12): `symbols`, `flow`, `arrows` e as
//! isométricas foram todas escritas tratando `cy − hh` como "o topo". As formas simétricas
//! em Y (retângulo, elipse, estrela) esconderam o defeito; as assimétricas saíram todas
//! invertidas. **Uma causa, não vinte bugs.**
//!
//! A correção não é acertar o sinal trinta vezes — é **não ter sinal para errar**. Aqui a
//! forma é autorada em `(u, v) ∈ [0,1]²`, `v = 0` no TOPO (a convenção da referência), e
//! [`Unit::p`] é o ÚNICO lugar do catálogo que sabe para que lado o mundo aponta. Uma
//! fórmula copiada de uma referência entra sem tradução mental, e o teste que prova a
//! orientação (`the_unit_box_has_v_zero_at_the_top`) prova todas as formas de uma vez.
//!
//! ## O mapa é afim, então os handles vêm de graça
//!
//! [`Unit::p`] é afim (translação + escala não-uniforme). Curvas de Bézier são invariantes
//! por afim: mapear as âncoras E os handles ponto a ponto dá **exatamente** a mesma curva
//! que mapear a curva. É por isso que os arcos podem ser construídos em espaço de autoria
//! e mapeados no fim — sem reconstruir tangente nenhuma no mundo.

use crate::{Contour, FillRule, VecPath, VecVertex};

/// Um par `(u, v)` em espaço de autoria: `u` 0→1 da esquerda para a direita, `v` 0→1 do
/// **topo para a base** (convenção SVG/OOXML).
pub(crate) type Uv = (f64, f64);

/// A caixa do gesto vista como uma `viewBox` de SVG.
#[derive(Copy, Clone, Debug)]
pub(crate) struct Unit {
    cx: f64,
    cy: f64,
    hw: f64,
    hh: f64,
}

impl Unit {
    /// A caixa definida pelos dois cantos do gesto (em qualquer ordem).
    #[must_use]
    pub(crate) fn of(a: [f64; 2], b: [f64; 2]) -> Self {
        Self {
            cx: (a[0] + b[0]) * 0.5,
            cy: (a[1] + b[1]) * 0.5,
            hw: (b[0] - a[0]).abs() * 0.5,
            hh: (b[1] - a[1]).abs() * 0.5,
        }
    }

    /// **A fronteira.** `(u, v)` de autoria → ponto no MUNDO. O `1 − 2v` é a inversão do
    /// eixo — a única do catálogo inteiro.
    #[must_use]
    pub(crate) fn p(&self, (u, v): Uv) -> [f64; 2] {
        [
            self.cx + self.hw * (2.0 * u - 1.0),
            self.cy + self.hh * (1.0 - 2.0 * v),
        ]
    }

    /// Vértice de quina (sem curvatura) em `(u, v)`.
    #[must_use]
    pub(crate) fn corner(&self, at: Uv) -> VecVertex {
        VecVertex::corner(self.p(at))
    }

    /// Vértice suave: âncora + handle de entrada + handle de saída, os três em `(u, v)`.
    #[must_use]
    pub(crate) fn smooth(&self, anchor: Uv, in_h: Uv, out_h: Uv) -> VecVertex {
        VecVertex::smooth(self.p(anchor), self.p(in_h), self.p(out_h))
    }

    /// Arco elíptico **de verdade** (cúbicas exatas, não polígono): centro `c`, raios
    /// `(ru, rv)` em fração da caixa CHEIA, de `start_deg` por `sweep_deg` graus.
    ///
    /// O ângulo é medido do eixo `+u` (3 horas) em direção ao `+v` — que aponta para
    /// BAIXO —, então `90°` é a base da forma. É a convenção da referência: quem lê
    /// "o bico fica em 90°" vê o bico embaixo.
    ///
    /// `sweep_deg` é **com sinal** (negativo percorre ao contrário — é o que faz o
    /// contorno interno de uma rosquinha correr no sentido oposto ao externo, para o furo
    /// ser furo). Fatia em pedaços de ≤90° e usa o comprimento de handle exato de cada um,
    /// `(4/3)·tan(θ/4)·r` — a generalização do `KAPPA`, que é esse valor em 90°.
    #[must_use]
    pub(crate) fn arc(
        &self,
        c: Uv,
        ru: f64,
        rv: f64,
        start_deg: f64,
        sweep_deg: f64,
    ) -> Vec<VecVertex> {
        let start = start_deg.to_radians();
        let total = sweep_deg.to_radians();
        let n = (total.abs() / std::f64::consts::FRAC_PI_2).ceil().max(1.0);
        let seg = total / n;
        // O sinal do sweep viaja no `h`: com sweep negativo os handles invertem sozinhos,
        // e o arco sai percorrido ao contrário sem um `reverse` para esquecer de passar.
        let h = (4.0 / 3.0) * (seg / 4.0).tan();
        #[expect(clippy::cast_sign_loss, reason = "n = ceil(|x|).max(1) é positivo")]
        let n_seg = n as usize;
        (0..=n_seg)
            .map(|i| {
                let a = start + seg * i as f64;
                let (s, co) = a.sin_cos();
                let anchor = (c.0 + ru * co, c.1 + rv * s);
                // Tangente ao arco em `a`, escalada pelo comprimento de handle.
                let (tu, tv) = (-ru * s * h, rv * co * h);
                self.smooth(
                    anchor,
                    (anchor.0 - tu, anchor.1 - tv),
                    (anchor.0 + tu, anchor.1 + tv),
                )
            })
            .collect()
    }

    /// Uma volta inteira: como [`Unit::arc`], mas sem o vértice repetido do fecho.
    #[must_use]
    pub(crate) fn ellipse(&self, c: Uv, ru: f64, rv: f64) -> Vec<VecVertex> {
        let mut v = self.arc(c, ru, rv, 0.0, 360.0);
        v.pop(); // o último coincide com o primeiro
        v
    }

    /// A razão altura/largura da caixa. Necessária para a forma que precisa ser
    /// **circular no MUNDO** (a tampa do "delay", o cantinho da pílula) e não elíptica:
    /// um raio em unidades de `u` vale `2·hw` no mundo, e em `v` vale `2·hh`, então um
    /// círculo verdadeiro tem `ru = rv · aspect`.
    #[must_use]
    pub(crate) fn aspect(&self) -> f64 {
        if self.hw.abs() < f64::EPSILON {
            1.0
        } else {
            self.hh / self.hw
        }
    }
}

/// Achata a junção entre `verts[i]` e `verts[i+1]` numa RETA: sem isto, um lado reto que
/// nasce ao lado de um arco herda a tangente do arco nos handles — a reta continua reta
/// (os controles ficam colineares), mas o vértice fica com handles fantasmas que o usuário
/// vê e arrasta ao editar. Aqui a quina é quina.
pub(crate) fn straighten(verts: &mut [VecVertex], i: usize) {
    if let Some(v) = verts.get_mut(i) {
        v.out_handle = v.anchor;
    }
    if let Some(v) = verts.get_mut(i + 1) {
        v.in_handle = v.anchor;
    }
}

/// As raízes em `(0,1)` da derivada de uma cúbica escalar — onde a curva tem extremo
/// INTERNO. Sem elas a bbox é a das âncoras, que **subestima** a forma: uma Bézier passeia
/// fora do casco das suas âncoras (o bico do coração e a barriga da gota vivem justamente
/// aí). É a diferença entre a caixa que o gizmo desenha e a forma que o usuário vê.
fn extrema_1d(p0: f64, p1: f64, p2: f64, p3: f64) -> impl Iterator<Item = f64> {
    let a = 3.0 * (-p0 + 3.0 * p1 - 3.0 * p2 + p3);
    let b = 6.0 * (p0 - 2.0 * p1 + p2);
    let c = 3.0 * (p1 - p0);
    let mut out = [f64::NAN; 2];
    if a.abs() < 1e-12 {
        if b.abs() > 1e-12 {
            out[0] = -c / b; // degenerou em reta: um extremo só
        }
    } else {
        let disc = b * b - 4.0 * a * c;
        if disc >= 0.0 {
            let s = disc.sqrt();
            out[0] = (-b + s) / (2.0 * a);
            out[1] = (-b - s) / (2.0 * a);
        }
    }
    out.into_iter()
        .filter(|t| t.is_finite() && *t > 0.0 && *t < 1.0)
}

/// Acumula um contorno na bbox: as âncoras **e** os extremos internos de cada cúbica.
fn eat_contour(verts: &[VecVertex], is_closed: bool, lo: &mut [f64; 2], hi: &mut [f64; 2]) {
    for v in verts {
        for k in 0..2 {
            lo[k] = lo[k].min(v.anchor[k]);
            hi[k] = hi[k].max(v.anchor[k]);
        }
    }
    let n = verts.len();
    let last = if is_closed { n } else { n.saturating_sub(1) };
    for i in 0..last {
        let (a, b) = (&verts[i], &verts[(i + 1) % n]);
        for k in 0..2 {
            let (p0, p1, p2, p3) = (a.anchor[k], a.out_handle[k], b.in_handle[k], b.anchor[k]);
            for t in extrema_1d(p0, p1, p2, p3) {
                let u = 1.0 - t;
                let val =
                    u * u * u * p0 + 3.0 * u * u * t * p1 + 3.0 * u * t * t * p2 + t * t * t * p3;
                lo[k] = lo[k].min(val);
                hi[k] = hi[k].max(val);
            }
        }
    }
}

/// A bbox VERDADEIRA da curva (âncoras **+ extremos internos das cúbicas**), em mundo.
fn curve_bbox(path: &VecPath) -> ([f64; 2], [f64; 2]) {
    let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
    eat_contour(&path.verts, path.closed, &mut lo, &mut hi);
    for c in &path.subpaths {
        eat_contour(&c.verts, c.closed, &mut lo, &mut hi);
    }
    (lo, hi)
}

/// **Ajusta a forma na caixa do gesto, exatamente.** Mede a bbox real da CURVA e mapeia-a
/// (por um afim) na caixa — âncoras e handles juntos, então as Béziers ficam intactas.
///
/// É a doutrina que o próprio OOXML segue: a `star5` do ECMA-376 carrega duas constantes
/// mágicas, `hf = 105146` e `vf = 110557`, que são **exatamente** `1/cos 18°` e
/// `2/(1 + cos 36°)` — os fatores de ajuste de um pentagrama, pré-calculados à mão. Aqui a
/// conta é feita em tempo de cozimento, para qualquer parâmetro.
///
/// O ganho: **nenhuma forma precisa de clamp anti-transbordo**. Ela é autorada no espaço
/// que for natural (o coração da especificação estoura a caixa em 0,36%; a lua astronômica
/// vive num disco de raio 1) e o ajuste garante que a bbox É a caixa. Sem isso, ou a forma
/// vaza (e o gizmo desalinha do desenho) ou nasce cheia de clamps que deformam.
///
/// Eixo degenerado (um parâmetro achatou a forma) não escala — só centra.
pub(crate) fn fit(u: &Unit, path: &mut VecPath) {
    let (lo, hi) = curve_bbox(path);
    let target_lo = [u.cx - u.hw, u.cy - u.hh];
    let target_hi = [u.cx + u.hw, u.cy + u.hh];
    let mut scale = [1.0_f64; 2];
    let mut shift = [0.0_f64; 2];
    for k in 0..2 {
        let span = hi[k] - lo[k];
        if span > 1e-9 {
            scale[k] = (target_hi[k] - target_lo[k]) / span;
            shift[k] = target_lo[k] - lo[k] * scale[k];
        } else {
            shift[k] = (target_lo[k] + target_hi[k]) * 0.5 - lo[k];
        }
    }
    let map = |p: &mut [f64; 2]| {
        for k in 0..2 {
            p[k] = p[k] * scale[k] + shift[k];
        }
    };
    for v in &mut path.verts {
        map(&mut v.anchor);
        map(&mut v.in_handle);
        map(&mut v.out_handle);
    }
    for c in &mut path.subpaths {
        for v in &mut c.verts {
            map(&mut v.anchor);
            map(&mut v.in_handle);
            map(&mut v.out_handle);
        }
    }
}

/// Polilinha fechada de quinas retas.
#[must_use]
pub(crate) fn poly(u: &Unit, pts: &[Uv]) -> VecPath {
    closed(pts.iter().map(|&at| u.corner(at)).collect())
}

/// Contorno fechado a partir de vértices já construídos.
#[must_use]
pub(crate) fn closed(verts: Vec<VecVertex>) -> VecPath {
    VecPath {
        verts,
        closed: true,
        ..VecPath::default()
    }
}

/// Contorno ABERTO (um traço, sem interior).
#[must_use]
pub(crate) fn open(verts: Vec<VecVertex>) -> VecPath {
    VecPath {
        verts,
        closed: false,
        ..VecPath::default()
    }
}

/// Anexa um sub-contorno a um path (furo, aresta interna ou face separada).
pub(crate) fn add_sub(path: &mut VecPath, verts: Vec<VecVertex>, closed: bool) {
    path.subpaths.push(Contour { verts, closed });
}

/// Marca o path como composto com furo: `EvenOdd` faz o sub-contorno interno virar FURO
/// (com `NonZero` e os dois no mesmo sentido, o furo sumiria).
pub(crate) fn punch(path: &mut VecPath) {
    path.fill_rule = FillRule::EvenOdd;
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: [f64; 2] = [-2.0, -1.0];
    const B: [f64; 2] = [2.0, 1.0];

    /// **O teste que vale por trinta.** É a definição executável da convenção: `v = 0` é o
    /// topo VISUAL, e o mundo é Y-para-cima. Enquanto isto valer, uma fórmula copiada de
    /// uma referência SVG/OOXML entra sem espelhar. Se alguém inverter o mapa, TODA forma
    /// assimétrica do catálogo quebra junto — e este teste cai primeiro, apontando a causa.
    #[test]
    fn the_unit_box_has_v_zero_at_the_top() {
        let u = Unit::of(A, B);
        let top = u.p((0.5, 0.0));
        let bottom = u.p((0.5, 1.0));
        assert!(
            top[1] > bottom[1],
            "v=0 tem de ser o TOPO no mundo Y-para-cima: topo {top:?} nao esta acima de {bottom:?}"
        );
        // E os cantos são os cantos da caixa do gesto, sem sobra.
        assert_eq!(u.p((0.0, 0.0)), [-2.0, 1.0], "canto superior esquerdo");
        assert_eq!(u.p((1.0, 1.0)), [2.0, -1.0], "canto inferior direito");
        assert_eq!(u.p((0.5, 0.5)), [0.0, 0.0], "o centro e o centro");
    }

    /// O ângulo cresce do `+u` em direção ao `+v` (para BAIXO): `90°` é a base. É o que
    /// permite ler a fórmula da referência sem traduzir ("o bico fica em 90°" = embaixo).
    #[test]
    fn ninety_degrees_is_the_bottom_of_the_shape() {
        let u = Unit::of(A, B);
        let quarter = u.arc((0.5, 0.5), 0.5, 0.5, 0.0, 90.0);
        let end = quarter.last().expect("arco tem vertices").anchor;
        let start = quarter[0].anchor;
        assert!(start[0] > 0.0 && start[1].abs() < 1e-9, "0 graus e 3 horas");
        assert!(
            end[1] < start[1],
            "90 graus tem de estar ABAIXO do inicio (mundo): {end:?} vs {start:?}"
        );
    }

    /// O arco é uma cúbica EXATA, não um polígono: uma volta inteira tem 4 vértices e o
    /// handle de 90° é o `KAPPA` clássico. (A seta curvada nascia poligonizada — visível
    /// a olho nu; este teste é a rede.)
    #[test]
    fn a_full_turn_is_four_exact_cubics_not_a_polygon() {
        let u = Unit::of([-1.0, -1.0], [1.0, 1.0]);
        let e = u.ellipse((0.5, 0.5), 0.5, 0.5);
        assert_eq!(e.len(), 4, "uma volta = 4 cubicas de 90 graus");
        // Raio 0.5 em unidades de caixa = 1.0 no mundo (a caixa tem 2 de largura).
        let handle = (e[0].out_handle[1] - e[0].anchor[1]).abs();
        assert!(
            (handle - crate::corners::KAPPA).abs() < 1e-9,
            "handle de 90 graus e o KAPPA: {handle}"
        );
    }

    /// Sweep negativo percorre ao contrário — e os handles acompanham (sem isso o contorno
    /// interno de uma rosquinha sairia com laços).
    #[test]
    fn a_negative_sweep_runs_backwards_with_its_handles() {
        let u = Unit::of(A, B);
        let fwd = u.arc((0.5, 0.5), 0.4, 0.4, 0.0, 90.0);
        let back = u.arc((0.5, 0.5), 0.4, 0.4, 0.0, -90.0);
        assert_eq!(fwd[0].anchor, back[0].anchor, "partem do mesmo ponto");
        assert!(
            (fwd[0].out_handle[1] - fwd[0].anchor[1]).signum()
                != (back[0].out_handle[1] - back[0].anchor[1]).signum(),
            "os handles tem de apontar para lados opostos"
        );
    }
}
