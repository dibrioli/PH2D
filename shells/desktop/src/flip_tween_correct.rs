//! ADR-0114 **Tween v2 — a correção de pares** (`docs/Flip/11 §6`).
//!
//! O matcher automático (`ph2d_flip::TweenPlan`) acerta a maioria, mas a pesquisa foi
//! categórica: TODO produto de correspondência (CACAni, GSAP, Corel) dá um escape MANUAL,
//! porque nenhum matcher acerta TODO par. Esta é a UI desse escape.
//!
//! **O fluxo:** parado entre duas chaves, ligue **Pairs** na barra da tira → o overlay
//! mostra os dois desenhos (A frio, B quente) com uma linha ligando cada par, pintada pela
//! CONFIANÇA (verde = casou certo · vermelho = duvidoso · âmbar = corrigido à mão). Clique
//! um traço, depois o traço do OUTRO lado com que ele deve casar → o par é forçado. Clique o
//! MESMO traço de novo → ele vira órfão (some/aparece esmaecendo, se Fade). **Add** commita
//! com o plano corrigido; desligar Pairs descarta.
//!
//! **A sessão é ESTADO DE AUTORIA, não documento** (vive na [`crate::flip_strip::FlipStrip`],
//! como os toggles): corrigir um par não muda o desenho até o Add. Ela é PINADA a um
//! intervalo `(camada, A, B)` pela [`crate::flip_strip::current_tween_interval`] — a MESMA
//! porta que o Add usa, senão a sessão descreveria um intervalo e o commit outro.
//!
//! **Coordenadas:** o overlay e o pick vivem em px de TELA (como todo o chrome do Flip —
//! `flip_selection_overlay`), mapeando a arte pela cadeia `câmera ∘ objeto ∘ pose_da_chave`
//! ([`screen_affine`]). A e B carregam poses de chave DIFERENTES, então cada lado tem seu
//! afim, e é por isso que o pick é feito em tela (um espaço só) em vez de inverter duas poses.

use ph2d_core::Vec2;
use ph2d_flip::{FlipDoc, FlipDrawing, FlipObjectId, FlipStroke, Frame, LayerId, Pose, TweenPlan};
use ph2d_vec_scene::Xform;
use ph2d_vector::{Affine, Point};

/// Raio de pick de um traço, em px de TELA. Generoso: um traço fino tem de ser clicável sem
/// mira de pixel, e aqui o alvo é o traço INTEIRO (não uma âncora), então a folga do
/// `flip_select_pick` (5 px) é apertada demais.
const PICK_PX: f64 = 10.0; // LITERAL-PX-OK: folga de pick de tela, não métrica de design

/// Qual dos dois desenhos-chave um traço pertence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Side {
    A,
    B,
}

/// Um traço apontado: o lado + o índice na lista de traços daquele desenho.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PairSel {
    pub(crate) side: Side,
    pub(crate) idx: usize,
}

/// **A sessão de correção de pares** — vive na tira (estado de autoria). Os dois desenhos são
/// CLONES: o overlay e o pick leem a geometria deles sem tocar o documento, e o Add reamostra
/// as MESMAS chaves (a guarda de dimensões do motor recusa um plano que não as descreve).
pub(crate) struct TweenCorrect {
    /// O objeto (para achar a entidade → afim mundo do overlay/pick).
    pub(crate) oid: FlipObjectId,
    /// O intervalo a que a sessão está pinada — a comparação que o Add faz.
    pub(crate) layer: LayerId,
    pub(crate) from: Frame,
    pub(crate) to: Frame,
    /// Clones dos desenhos-chave (a geometria que o overlay desenha e o pick testa).
    pub(crate) a: FlipDrawing,
    pub(crate) b: FlipDrawing,
    /// Poses das duas chaves (A e B podem estar em poses diferentes — cada uma tem seu afim).
    pub(crate) pose_a: Pose,
    pub(crate) pose_b: Pose,
    /// A correspondência EFETIVA (automática, já com as correções aplicadas).
    pub(crate) plan: TweenPlan,
    /// O 1º traço de um re-par em curso (aguardando o 2º clique). `None` = nada marcado.
    pub(crate) pending: Option<PairSel>,
}

/// **Constrói a sessão para o intervalo de tween atual** — `None` se não há dois keyframes
/// para interpolar entre, ou se as chaves não têm desenho.
///
/// A porta única [`crate::flip_strip::current_tween_interval`] resolve o intervalo (o MESMO
/// que o Add commita); daqui em diante só se clonam os desenhos e se constrói o plano.
#[must_use]
pub(crate) fn build(
    flip: &FlipDoc,
    active_layer: Option<LayerId>,
    playhead: &ph2d_core::Playhead,
) -> Option<TweenCorrect> {
    let (oid, lid, from, to) =
        crate::flip_strip::current_tween_interval(flip, active_layer, playhead)?;
    let obj = flip.object(oid)?;
    let layer = obj.layer(lid)?;
    let da = layer.frames().get(&from).and_then(|f| f.drawing)?;
    let db = layer.frames().get(&to).and_then(|f| f.drawing)?;
    let a = obj.drawing(da)?.clone();
    let b = obj.drawing(db)?.clone();
    let plan = TweenPlan::build(&a, &b);
    Some(TweenCorrect {
        oid,
        layer: lid,
        from,
        to,
        a,
        b,
        pose_a: obj.frame_pose(lid, from),
        pose_b: obj.frame_pose(lid, to),
        plan,
        pending: None,
    })
}

/// **ARTE → TELA**, a cadeia inteira: `câmera ∘ objeto ∘ pose_da_chave`.
///
/// A MESMA composição que o render dobra (e que o `flip_selection_overlay` usa para o
/// realce). Pura — o overlay a chama para os DOIS afins (A com `pose_a`, B com `pose_b`) e o
/// pick reusa exatamente o mesmo mapeamento, senão o clique cairia num lugar e o desenho
/// noutro.
#[must_use]
pub(crate) fn screen_affine(l2w: &Xform, pose: Pose, cam: Affine) -> Affine {
    let [a, b, c, d, e, f] = crate::flip_transform::art_to_world(l2w, pose).0;
    cam * Affine::new([a, b, c, d, e, f])
}

/// O centróide (média das posições) de um traço, em coords de ARTE — a âncora da linha de
/// par no overlay. Média basta: a linha só liga *onde A está* a *onde B está*, não é a
/// feature de correspondência (essa é a integral de arco do motor).
#[must_use]
pub(crate) fn stroke_centroid(s: &FlipStroke) -> Vec2 {
    let pos = s.positions();
    if pos.is_empty() {
        return Vec2::ZERO;
    }
    pos.iter().fold(Vec2::ZERO, |a, &p| a + p) / pos.len() as f32
}

/// Distância² (px de tela) do ponto `(x,y)` ao traço `s`, mapeado por `aff`. Cobre a
/// COSTURA de um traço fechado (via `segments()`) e o caso de um ponto só.
fn stroke_screen_dist2(s: &FlipStroke, aff: Affine, x: f64, y: f64) -> f64 {
    let p = Point::new(x, y);
    let mut best = f64::INFINITY;
    for (_, a, b) in s.segments() {
        let pa = aff * Point::new(f64::from(a.x), f64::from(a.y));
        let pb = aff * Point::new(f64::from(b.x), f64::from(b.y));
        best = best.min(seg_dist2(p, pa, pb));
    }
    if s.len() == 1
        && let Some(p0) = s.positions().first()
    {
        let sp = aff * Point::new(f64::from(p0.x), f64::from(p0.y));
        best = best.min((p - sp).hypot2());
    }
    best
}

/// Distância² de `p` ao segmento `a`→`b`, em px de tela.
fn seg_dist2(p: Point, a: Point, b: Point) -> f64 {
    let ab = b - a;
    let len2 = ab.hypot2();
    let t = if len2 > 0.0 {
        ((p - a).dot(ab) / len2).clamp(0.0, 1.0)
    } else {
        0.0
    };
    (p - (a + ab * t)).hypot2()
}

/// **O traço mais próximo do clique** (em tela), de qualquer um dos dois lados, ao alcance
/// de [`PICK_PX`] — ou `None` se o clique caiu no vazio.
///
/// ⚠️ **A ambiguidade de sobreposição é conhecida e aceita:** onde um traço de A e um de B
/// caem no MESMO lugar de tela (um traço que não se move — já casado certo), o empate estrito
/// prefere A. Isso só atrapalha um par que já está correto, que é justamente o que ninguém
/// re-pareia; os traços que o artista quer corrigir são os que se movem, e esses estão
/// separados na tela.
#[must_use]
pub(crate) fn nearest_stroke(
    a: &FlipDrawing,
    aff_a: Affine,
    b: &FlipDrawing,
    aff_b: Affine,
    x: f64,
    y: f64,
) -> Option<PairSel> {
    let thresh2 = PICK_PX * PICK_PX;
    let mut best: Option<(f64, PairSel)> = None;
    let mut consider = |d2: f64, sel: PairSel| {
        if d2 <= thresh2 && best.is_none_or(|(bd, _)| d2 < bd) {
            best = Some((d2, sel));
        }
    };
    for (i, s) in a.strokes.iter().enumerate() {
        consider(
            stroke_screen_dist2(s, aff_a, x, y),
            PairSel {
                side: Side::A,
                idx: i,
            },
        );
    }
    for (j, s) in b.strokes.iter().enumerate() {
        consider(
            stroke_screen_dist2(s, aff_b, x, y),
            PairSel {
                side: Side::B,
                idx: j,
            },
        );
    }
    best.map(|(_, sel)| sel)
}

/// **O gesto de re-par, PURO** — dado o traço que o clique pegou (`hit`, ou `None` no vazio)
/// e o que estava marcado (`pending`), atualiza o plano e devolve a marca nova.
///
/// - vazio ⇒ desmarca;
/// - nada marcado ⇒ marca o traço clicado;
/// - marcado + MESMO traço ⇒ **orfana** (corta o par) e desmarca — o *"click me de novo para
///   soltar"*;
/// - marcado + OUTRO lado ⇒ **força o par** (A↔B) e desmarca;
/// - marcado + outro traço do MESMO lado ⇒ move a marca para ele.
#[must_use]
pub(crate) fn apply_click(
    plan: &mut TweenPlan,
    pending: Option<PairSel>,
    hit: Option<PairSel>,
) -> Option<PairSel> {
    match (pending, hit) {
        (_, None) => None,
        (None, Some(h)) => Some(h),
        (Some(p), Some(h)) if p == h => {
            match h.side {
                Side::A => plan.unpair_a(h.idx),
                Side::B => plan.unpair_b(h.idx),
            };
            None
        }
        (Some(p), Some(h)) if p.side != h.side => {
            let (a, b) = if matches!(p.side, Side::A) {
                (p.idx, h.idx)
            } else {
                (h.idx, p.idx)
            };
            plan.repair(a, b);
            None
        }
        (Some(_), Some(h)) => Some(h),
    }
}

impl crate::App {
    /// A tool Flip quer o canvas para RE-PAREAR agora? (ativa + sessão de pares aberta.)
    #[must_use]
    pub(crate) fn flip_wants_tween_pairs(&self) -> bool {
        self.flip_active && self.flip_strip.tween_correct.is_some()
    }

    /// **Re-pina a sessão ao intervalo atual quando o artista navega.** A sessão SEGUE o
    /// artista: outro intervalo é outra correspondência, e não há correção a preservar entre
    /// eles. No-op se o intervalo é o mesmo — as correções ficam. Sem intervalo agora, a
    /// sessão segue pinada ao último (o overlay mostra aquele até o artista voltar a um
    /// intervalo diferente). Rodado por frame quando ativa (barato: só compara, reconstrói na
    /// troca).
    pub(crate) fn flip_tween_pairs_upkeep(&mut self) {
        if self.flip_strip.tween_correct.is_none() {
            return;
        }
        let active_layer = self.flip_active_layer;
        let playhead = self.playhead;
        let Some(gfx) = self.gfx.as_ref() else {
            return;
        };
        let session = self
            .flip_strip
            .tween_correct
            .as_ref()
            .map(|tc| (tc.layer, tc.from, tc.to));
        let cur = crate::flip_strip::current_tween_interval(&gfx.flip, active_layer, &playhead);
        let rebuild = matches!(
            (session, cur),
            (Some(s), Some((_, lid, from, to))) if s != (lid, from, to)
        );
        if rebuild {
            let rebuilt = build(&gfx.flip, active_layer, &playhead);
            self.flip_strip.tween_correct = rebuilt;
        }
    }

    /// **Pen-DOWN no modo Pairs** — o clique re-pareia. Sempre consome (enquanto Pairs está
    /// ativo o canvas é da correção, não do Draw/Erase): um clique perdido que virasse traço
    /// seria pior que um clique que não faz nada.
    pub(crate) fn flip_tween_pairs_canvas_down(&mut self, x: f32, y: f32) -> bool {
        if !self.flip_wants_tween_pairs() {
            return false;
        }
        let hit = {
            let Some(gfx) = self.gfx.as_ref() else {
                return false;
            };
            let Some(tc) = self.flip_strip.tween_correct.as_ref() else {
                return false;
            };
            let win = gfx.surface.size();
            let l2w = self
                .flip_entities
                .get(&tc.oid)
                .copied()
                .map(ph2d_ecs::Entity::from_bits)
                .filter(|e| gfx.sim.world().get_entity(*e).is_ok())
                .map_or(Xform::IDENTITY, |e| {
                    crate::flip_transform::object_xform(&gfx.sim, e)
                });
            let cam = gfx.camera.world_to_screen_affine(win);
            let aff_a = screen_affine(&l2w, tc.pose_a, cam);
            let aff_b = screen_affine(&l2w, tc.pose_b, cam);
            nearest_stroke(&tc.a, aff_a, &tc.b, aff_b, f64::from(x), f64::from(y))
        };
        if let Some(tc) = self.flip_strip.tween_correct.as_mut() {
            tc.pending = apply_click(&mut tc.plan, tc.pending, hit);
        }
        true
    }
}

#[cfg(test)]
#[path = "flip_tween_correct_tests.rs"]
mod tests;
