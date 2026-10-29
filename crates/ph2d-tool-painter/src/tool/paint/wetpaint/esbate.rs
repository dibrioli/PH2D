//! ⭐ **As pontas que ESBATEM ao comprido** (J6t, [`ph2d_painter_brush::BrushSpec::pontas_esbate`]; doc
//! `docs/Painter/Pinceis/J6t_pontas_esbatem.md`). O traço largo do Rebelle esbate-se ao longo do comprimento nas
//! duas pontas, ao longo de UMA pegada (escala com o diâmetro); o nosso ia de nada a cheio em menos de 15 px.
//!
//! A lei: a `a` raios PARA DENTRO do centro da ponta o traço pousa [`alvo`] `= 1 − e·((1 − a)/2)³` do corpo (`a < 0`
//! = além do centro, na tampa; cheio a partir de um raio para dentro). Pesa DEPOIS do portão do papel (a porta do
//! padrão, onde o depósito por célula é linear — e o alvo do filme também).
//! - O INÍCIO é conhecido: cada célula pesa `alvo` da sua posição ao longo do 1.º rumo desde o 1.º centro — exacto na
//!   soma e no filme (o máximo de `alvo · x` é `alvo · máximo`). O 1.º dab, sem rumo, não pousa.
//! - O FIM só se conhece ao soltar. Sem as pontas rectas, cada dab pesa a sua pegada pelo [`alvo`] da posição ao longo
//!   do SEU rumo (cheio atrás, a cair para a frente): no FILME (o máximo) a célula fica com o dab mais à frente que a
//!   cobre — o último, na ponta; no corpo há sempre um dab um raio à frente (cheio). Na SOMA pesa a parte ÍMPAR do
//!   perfil, `1 + (alvo(−û) − alvo(û))/2` (a frente mais leve, a de trás mais carregada): a soma de qualquer pegada
//!   simétrica dá o corpo de sempre, sem renormalizar (⛔ medido: `2 ÷ ∫ alvo` supõe a pegada chata — a redonda macia
//!   pousava `1,14×` no miolo); a frente nunca desce de ½, e a cauda é mais suave que a do filme.
//! - Com as pontas rectas (cada dab pousa a metade de trás, [`super::tampa`]) a cauda viva é a rampa `a` de um raio;
//!   o FECHO continua-a com dabs de CARGA [`carga_do_fecho`] `= 1 − (3e/8)·(d/R)²` a `d` além do fim — a soma dá o
//!   mesmo [`alvo`] ao longo da cauda.
//!
//! As posições medem-se na CORDA local da pegada redonda (`√(R² − v²)` a `v` do eixo): a tampa esbate até à borda
//! redonda, e a soma renormalizada é exacta a toda a largura. `e = 0`: nada disto corre (ao bit). Gate: `esbate_tests`.

use ph2d_painter_brush::Dab;

use super::session::Lane;
use super::tampa::Lote;

/// Até onde (em raios do 1.º dab, pelo caminho, além de um raio) a entrada ainda pesa: um dab de raio `R` a `s` do
/// 1.º centro só toca células a `s − R`; a partir de um raio para dentro a entrada é cheia.
const ALCANCE_DA_ENTRADA: f32 = 1.5;

/// A fracção do corpo que o traço pousa a `a` raios para dentro do centro da ponta.
#[inline]
pub(super) fn alvo(e: f32, a: f32) -> f32 {
    let t = ((1.0 - a) * 0.5).clamp(0.0, 1.0);
    (1.0 - e * t * t * t).max(0.0)
}

/// A carga de um dab do FECHO a `d` px além do último centro (pegada de raio `r`): a soma com a cauda viva da metade
/// de trás dá o [`alvo`].
#[inline]
pub(super) fn carga_do_fecho(e: f32, d: f32, r: f32) -> f32 {
    let q = d / r.max(1e-3);
    (1.0 - 0.375 * e * q * q).max(0.0)
}

fn unitario(v: [f32; 2]) -> Option<[f32; 2]> {
    let n = v[0].hypot(v[1]);
    (n > 1e-6).then(|| [v[0] / n, v[1] / n])
}

/// A posição de `p` ao longo de `n` desde `o`, em meias-cordas da pegada redonda de raio `r` centrada no eixo.
#[inline]
fn na_corda(p: [f32; 2], o: [f32; 2], n: [f32; 2], r: f32) -> f32 {
    let (dx, dy) = (p[0] - o[0], p[1] - o[1]);
    let u = dx * n[0] + dy * n[1];
    let v = dx * n[1] - dy * n[0];
    let h = (r * r - v * v).max(r * r * 1e-4).sqrt();
    u / h
}

/// O peso de UM dab ([`Self::peso`]).
#[derive(Clone, Copy, Debug)]
pub(super) struct Esbate {
    e: f32,
    /// A entrada: o 1.º centro, o 1.º rumo e o raio do 1.º dab — só perto do início.
    inicio: Option<([f32; 2], [f32; 2], f32)>,
    /// O perfil do fim (sem as pontas rectas): o centro, o rumo, o raio e se o depósito é o filme (o máximo).
    perfil: Option<([f32; 2], [f32; 2], f32, bool)>,
    /// A carga do dab (os do fecho das pontas rectas; 1 os outros).
    carga: f32,
}

impl Esbate {
    /// O peso do dab `d` na lane, ou `None` — o dab sem rumo, que não pousa. `rectas` = as pontas rectas ligadas (a
    /// tampa já ensinou a lane); `filme` = o depósito é o máximo (o perfil inteiro; na soma, a parte ímpar). Um peso que vale 1 em toda a
    /// pegada também dá `Some`: [`Self::trivial`] diz se vale a pena.
    pub(super) fn do_dab(
        d: &Dab,
        lane: &mut Lane,
        lote: Lote,
        e: f32,
        rectas: bool,
        filme: bool,
    ) -> Option<Self> {
        let rumo = unitario(d.dir)?;
        if lane.rumo0 == [0.0; 2] {
            lane.rumo0 = rumo;
        }
        let raio = d.radius_px.max(1e-3);
        let raio0 = lane.raio0.max(1e-3);
        let perto =
            lote == Lote::Abertura || d.arc_len - lane.arco0 < raio0 + ALCANCE_DA_ENTRADA * raio;
        let carga = match (lote, lane.ultimo) {
            (Lote::Fecho, Some(u)) => carga_do_fecho(e, d.arc_len - u.arc_len, u.radius_px),
            _ => 1.0,
        };
        Some(Self {
            e,
            inicio: perto.then_some((lane.inicio, lane.rumo0, raio0)),
            perfil: (!rectas).then_some((d.center, rumo, raio, filme)),
            carga,
        })
    }

    /// O peso é 1 em toda a pegada (longe do início, sem perfil, carga cheia): a porta não precisa dele.
    pub(super) fn trivial(&self) -> bool {
        self.inicio.is_none() && self.perfil.is_none() && self.carga == 1.0
    }

    /// O peso do depósito no píxel `(x, y)` (px de tela).
    #[inline]
    pub(super) fn peso(&self, x: f32, y: f32) -> f32 {
        let p = [x, y];
        let mut w = self.carga;
        if let Some((o, n, r)) = self.inicio {
            w *= alvo(self.e, na_corda(p, o, n, r));
        }
        if let Some((c, n, r, filme)) = self.perfil {
            let u = na_corda(p, c, n, r).clamp(-1.0, 1.0);
            w *= if filme {
                alvo(self.e, -u)
            } else {
                1.0 + 0.5 * (alvo(self.e, -u) - alvo(self.e, u))
            };
        }
        w
    }
}
