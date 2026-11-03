#![forbid(unsafe_code)]
//! ⭐ **T44 — a corda do alfinete** (`docs/CartoesUI/01_decisoes_do_dono.md`): o cartão solto preso
//! COM corda liga-se ao seu sujeito no canvas por uma corda com física — a lei do nó
//! `motion.verlet-rope` ([`ph2d_node_motion_verlet_rope::corda`]). Cada quadro: o sujeito em ecrã
//! (quem compõe diz onde), a borda do cartão mais perto dele, um passo; os pontos vão ao store e o
//! painel pinta-os.

pub mod chave;

use ph2d_card_stack::metrics as m;
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{NodeId, WidgetStore};
use ph2d_node_motion_verlet_rope::corda::{Corda, Feitio};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::time::Instant;

const FEITIO: Feitio = Feitio {
    pontos: m::LOOSE_ROPE_POINTS,
    gravidade: m::LOOSE_ROPE_GRAVITY,
    iteracoes: m::LOOSE_ROPE_ITERATIONS,
    amortecimento: m::LOOSE_ROPE_DAMPING,
    flexao: m::LOOSE_ROPE_BEND,
};

#[derive(Default)]
struct Cordas {
    vivas: BTreeMap<(NodeId, NodeId), Corda>,
    /// T41: as que se soltaram — a corda, a ponta do cartão de quando se soltou, o objecto e `0..1`.
    recolhem: BTreeMap<(NodeId, NodeId), (Corda, (f32, f32), (f32, f32), f32)>,
    /// T52: o relógio POR PAINEL — a shell anda o Bones e o Inspector no mesmo quadro.
    antes: BTreeMap<NodeId, Instant>,
    smoke_feito: bool,
}

thread_local! {
    static CORDAS: RefCell<Cordas> = RefCell::default();
}

/// O quadro: anda a corda de cada solto de `panel` preso com corda. `alvo(bits)` = o sujeito preso
/// em ECRÃ (`None` = já não existe).
pub fn anda(store: &mut WidgetStore, panel: NodeId, alvo: impl Fn(u64) -> Option<(f32, f32)>) {
    let agora = Instant::now();
    let dt = CORDAS
        .with(|c| c.borrow_mut().antes.insert(panel, agora))
        .map_or(0.0, |t| agora.duration_since(t).as_secs_f32());
    if std::env::var_os("PH2D_CARTAO_CORDA_SMOKE").is_some() {
        smoke(store, panel);
    }
    anda_com(store, panel, alvo, dt);
}

/// [`anda`] com o passo dado (s) — a porta dos testes.
pub fn anda_com(
    store: &mut WidgetStore,
    panel: NodeId,
    alvo: impl Fn(u64) -> Option<(f32, f32)>,
    dt: f32,
) {
    let dt = dt.clamp(0.0, m::LOOSE_ROPE_DT_MAX);
    let presos = store.loose_ropes_of(panel);
    CORDAS.with(|c| {
        let Cordas {
            vivas, recolhem, ..
        } = &mut *c.borrow_mut();
        // T41: a corda que se solta não some — recolhe-se até ao objecto.
        let saem: Vec<_> = vivas
            .keys()
            .filter(|(p, k)| *p == panel && !presos.iter().any(|(s, _)| s == k))
            .copied()
            .collect();
        for chave in saem {
            if let Some(corda) = vivas.remove(&chave)
                && let (Some(a), Some(o)) = (corda.pontos().first(), corda.pontos().last())
            {
                let (a, o) = ((a[0], -a[1]), (o[0], -o[1]));
                recolhem.insert(chave, (corda, a, o, 0.0));
            }
        }
        for (k, bits) in presos {
            let Some((o, r)) = alvo(bits).zip(rect_de_agora(store, k)) else {
                store.set_loose_rope(k, Vec::new());
                continue;
            };
            let a = borda(r, o);
            // Volta a prender a meio do recolher: a mesma corda (sem salto).
            let corda = vivas.entry((panel, k)).or_insert_with(|| {
                recolhem
                    .remove(&(panel, k))
                    .map(|r| r.0)
                    .unwrap_or_default()
            });
            store.set_reeling_rope(k, panel, Vec::new());
            let pontos = passo(corda, a, o, dt);
            store.set_loose_rope(k, pontos);
        }
        let fim: Vec<_> = recolhem
            .keys()
            .filter(|(p, _)| *p == panel)
            .copied()
            .collect();
        for chave in fim {
            let Some((corda, a0, o, t)) = recolhem.get_mut(&chave) else {
                continue;
            };
            *t += dt / m::LOOSE_ROPE_REEL_S;
            if *t >= 1.0 {
                recolhem.remove(&chave);
                store.set_reeling_rope(chave.1, panel, Vec::new());
                continue;
            }
            let e = *t * *t * (3.0 - 2.0 * *t);
            let a = (a0.0 + (o.0 - a0.0) * e, a0.1 + (o.1 - a0.1) * e);
            let pontos = passo(corda, a, *o, dt);
            store.set_reeling_rope(chave.1, panel, pontos);
        }
    });
}

/// Um passo da corda de `a` (cartão) a `o` (objecto), em ecrã. Passa por cima dos outros soltos
/// (T50, revoga a T48): quem pinta muda o tom do troço que os cruza.
fn passo(corda: &mut Corda, a: (f32, f32), o: (f32, f32), dt: f32) -> Vec<[f32; 2]> {
    // O nó tem o `y` para cima; o ecrã, para baixo.
    corda.passo([a.0, -a.1], [o.0, -o.1], comprimento(a, o), dt, &FEITIO);
    corda.pontos().iter().map(|p| [p[0], -p[1]]).collect()
}

/// `PH2D_CARTAO_CORDA_SMOKE=1` (a foto do T44): o 1.º solto de `panel` com sujeito prende-se COM
/// corda, uma vez — depois o alfinete é do artista.
fn smoke(store: &mut WidgetStore, panel: NodeId) {
    if CORDAS.with(|c| c.borrow().smoke_feito) || store.card_subject(panel).is_none() {
        return;
    }
    let livre = |s: &WidgetStore, k: NodeId| {
        s.is_card_pinnable(k)
            && s.loose_card(k)
                .is_some_and(|c| c.panel == panel && c.pin.is_none())
    };
    if let Some(k) = store
        .loose_section_ids()
        .into_iter()
        .find(|k| livre(store, *k))
    {
        store.toggle_loose_pin(k);
        store.toggle_loose_pin(k);
        CORDAS.with(|c| c.borrow_mut().smoke_feito = true);
    }
}

/// O comprimento da corda de `a` a `o`: a distância mais a folga do vão HORIZONTAL — de lado pende,
/// na vertical é um fio de prumo ESTICADO (a folga não tem para onde ir e o peso encurvava-a num
/// cotovelo, T44c).
fn comprimento(a: (f32, f32), o: (f32, f32)) -> f32 {
    let (dx, dist) = ((o.0 - a.0).abs(), (o.0 - a.0).hypot(o.1 - a.1));
    let lado = if dist > 0.0 { dx / dist } else { 0.0 };
    dist * (1.0 - m::LOOSE_ROPE_PLUMB_TENSION * (1.0 - lado))
        + dx * (m::LOOSE_ROPE_SLACK - 1.0)
        + m::LOOSE_ROPE_SLACK_MIN * lado
}

/// O rect do cartão NESTE quadro ([`WidgetStore::loose_card_rect_now`]: a mão já andou, a aba
/// atrás não o apaga, o suspenso T53 está sob a mão).
fn rect_de_agora(store: &WidgetStore, k: NodeId) -> Option<Rect> {
    store.loose_card_rect_now(k)
}

/// O ponto da borda de `r` mais perto de `p` (dentro do cartão, o próprio `p`).
fn borda(r: Rect, p: (f32, f32)) -> (f32, f32) {
    (p.0.clamp(r.x, r.x + r.w), p.1.clamp(r.y, r.y + r.h))
}

#[cfg(test)]
mod tests;
