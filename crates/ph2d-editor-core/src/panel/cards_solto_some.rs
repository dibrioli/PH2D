//! ⭐ **T41 — o solto que some ou volta pela T42 esmaece** (`docs/CartoesUI/01_decisoes_do_dono.md`):
//! cada solto pinta-se numa cena à parte (a FOTO) que entra na tela com a opacidade dele; escolher
//! outra classe deixa de dar o `CardDef`, e o que fica a esmaecer é a foto do último quadro. Sem
//! clique enquanto esmaece (nada se regista). E a corda que se soltou recolhe-se (`ph2d-card-rope`).

use crate::interaction::{WidgetStore, loose_rect_of};
use crate::widget::card_ink::CardInk;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::{Affine, Compose, Mix, Shape, VectorScene, VelloBlend};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::time::Instant;

#[derive(Default)]
struct Foto {
    cena: VectorScene,
    alfa: f32,
}

#[derive(Default)]
struct Fotos {
    por_solto: BTreeMap<NodeId, Foto>,
    antes: Option<Instant>,
}

thread_local! {
    static FOTOS: RefCell<Fotos> = RefCell::default();
}

/// O passo de opacidade deste quadro (uma vez por quadro, à entrada da pintura dos soltos).
pub(super) fn passo(store: &WidgetStore) -> f32 {
    FOTOS.with(|f| {
        let mut f = f.borrow_mut();
        f.por_solto.retain(|k, _| store.loose_card(*k).is_some());
        let agora = Instant::now();
        let dt = f
            .antes
            .replace(agora)
            .map_or(0.0, |t| agora.duration_since(t).as_secs_f64().min(0.1));
        #[allow(clippy::cast_possible_truncation)]
        let dt = crate::aquecimento::passo_do_quadro(dt) as f32;
        dt / m::LOOSE_FADE_S
    })
}

/// A cena onde se pinta o solto `k` neste quadro (a foto, limpa).
pub(super) fn toma(k: NodeId) -> VectorScene {
    FOTOS.with(|f| {
        let mut f = f.borrow_mut();
        let foto = f.por_solto.entry(k).or_insert_with(|| Foto {
            cena: VectorScene::new(),
            alfa: 1.0,
        });
        let mut cena = std::mem::take(&mut foto.cena);
        cena.reset();
        cena
    })
}

/// Cola o solto `k` pintado em `cena` na `tela`, a subir a opacidade (`d` = o passo do quadro).
pub(super) fn cola(tela: &mut VectorScene, k: NodeId, cena: VectorScene, d: f32) {
    FOTOS.with(|f| {
        let mut f = f.borrow_mut();
        let foto = f.por_solto.entry(k).or_default();
        foto.alfa = (foto.alfa + d).min(1.0);
        foto.cena = cena;
        com_alfa(tela, &foto.cena, foto.alfa);
    });
}

/// O solto `k` que a seleção escondeu: a foto a descer a opacidade, até sumir.
pub(super) fn apaga(tela: &mut VectorScene, k: NodeId, d: f32) {
    FOTOS.with(|f| {
        if let Some(foto) = f.borrow_mut().por_solto.get_mut(&k) {
            foto.alfa = (foto.alfa - d).max(0.0);
            if foto.alfa > 0.0 {
                com_alfa(tela, &foto.cena, foto.alfa);
            }
        }
    });
}

/// A opacidade de agora do solto `k` (os testes).
#[must_use]
pub fn alfa(k: NodeId) -> Option<f32> {
    FOTOS.with(|f| f.borrow().por_solto.get(&k).map(|x| x.alfa))
}

fn com_alfa(tela: &mut VectorScene, cena: &VectorScene, alfa: f32) {
    if alfa >= 1.0 {
        tela.append_transformed(cena, Affine::IDENTITY);
        return;
    }
    let tudo = ph2d_vector::Rect::new(-1e6, -1e6, 1e6, 1e6);
    let normal = VelloBlend::new(Mix::Normal, Compose::SrcOver);
    tela.push_layer_shape(normal, alfa, Affine::IDENTITY, &tudo);
    tela.append_transformed(cena, Affine::IDENTITY);
    tela.pop_layer();
}

thread_local! {
    /// Os painéis que pintaram soltos neste quadro, à espera do fim da caminhada.
    static CORDAS_PEDIDAS: std::cell::RefCell<Vec<(NodeId, Theme)>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// O painel pintou os soltos dele: as cordas dele pintam-se no fim, por cima dos de todos.
pub(super) fn pede_as_cordas(panel: NodeId, theme: Theme) {
    CORDAS_PEDIDAS.with(|c| {
        let mut c = c.borrow_mut();
        if !c.iter().any(|(p, _)| *p == panel) {
            c.push((panel, theme));
        }
    });
}

/// As cordas dos painéis que pintaram soltos neste quadro, depois de todos os painéis.
pub fn pinta_as_cordas(tela: &mut VectorScene, store: &WidgetStore) {
    for (panel, theme) in CORDAS_PEDIDAS.with(std::cell::RefCell::take) {
        cordas(tela, store, panel, theme);
    }
}

/// ⭐ T50: as cordas de `panel` POR CIMA dos soltos dele (depois de publicados os rects deste
/// quadro) — a viva (com o nó no objecto e a solda no cartão; o suspenso T53 sob a mão) e as que se
/// recolhem (T41). O troço que cruza OUTRO solto muda de tom (`CardField`): a corda tem a cor do
/// cartão e sumiria nele.
fn cordas(tela: &mut VectorScene, store: &WidgetStore, panel: NodeId, theme: Theme) {
    let outros = |k: NodeId| -> Vec<Rect> {
        store
            .loose_section_ids()
            .into_iter()
            .filter(|o| *o != k)
            .filter_map(|o| store.panel_rect(loose_rect_of(o)))
            .collect()
    };
    let suspenso = store.suspended_card().filter(|(_, c)| c.panel == panel);
    let vivas = store
        .loose_cards_of(panel)
        .into_iter()
        .chain(suspenso)
        .filter(|(_, c)| c.rope);
    for (k, _) in vivas {
        let pontos = store.loose_rope(k);
        let pintado =
            store.panel_rect(loose_rect_of(k)).is_some() || suspenso.is_some_and(|(s, _)| s == k);
        let (Some(r), true) = (store.loose_card_rect_now(k), pintado) else {
            continue;
        };
        let mut linha: Vec<(f32, f32)> = pontos.iter().map(|p| (p[0], p[1])).collect();
        let borda = super::solto::na_borda(pontos, r);
        if let (Some(a), Some(p)) = (linha.first_mut(), borda) {
            *a = p;
        }
        let ink = CardInk::of_card(
            store.section_theme(crate::interaction::loose_face_of(k)),
            theme,
        );
        pinta_corda(tela, &linha, &outros(k), ink);
        if let Some((x, y)) = borda {
            let cor = ink.color(ColorToken::KanbanCard);
            crate::paint::fill_circle(tela, x, y, m::LOOSE_ROPE_WELD_R, cor);
        }
    }
    for (k, pontos) in store.reeling_ropes_of(panel) {
        let linha: Vec<(f32, f32)> = pontos.iter().map(|p| (p[0], p[1])).collect();
        pinta_corda(
            tela,
            &linha,
            &outros(k),
            CardInk::of_card(
                store.section_theme(crate::interaction::loose_face_of(k)),
                theme,
            ),
        );
    }
}

/// A corda e o nó na ponta do objecto; o troço sobre outro solto no tom do campo do cartão.
fn pinta_corda(tela: &mut VectorScene, linha: &[(f32, f32)], outros: &[Rect], ink: CardInk) {
    for (troco, sobre) in trocos(linha, outros) {
        let token = if sobre {
            ColorToken::CardField
        } else {
            ColorToken::KanbanCard
        };
        crate::paint::stroke_polyline(tela, &troco, m::LOOSE_ROPE_W, ink.color(token));
    }
    if let Some(&(x, y)) = linha.last() {
        let cor = ink.color(ColorToken::KanbanCard);
        // Pelo `fill_path`, que a gravação vê: é o nó que prova em que ponto do quadro a corda entra.
        const TOLERANCIA: f64 = 0.1; // LITERAL-PX-OK: tolerância da curva, a mesma do Vello
        let no = ph2d_vector::Circle::new(
            (f64::from(x), f64::from(y)),
            f64::from(m::LOOSE_ROPE_KNOT_R),
        );
        tela.fill_path(
            &no.to_path(TOLERANCIA),
            &ph2d_vector::Brush::Solid(cor),
            Affine::IDENTITY,
        );
    }
}

/// T50: a linha em troços seguidos — `true` = o troço cujo MEIO cai dentro de um de `outros`.
fn trocos(linha: &[(f32, f32)], outros: &[Rect]) -> Vec<(Vec<(f32, f32)>, bool)> {
    let mut out: Vec<(Vec<(f32, f32)>, bool)> = Vec::new();
    for s in linha.windows(2) {
        let meio = ((s[0].0 + s[1].0) * 0.5, (s[0].1 + s[1].1) * 0.5);
        let sobre = outros.iter().any(|r| r.contains(meio.0, meio.1));
        match out.last_mut() {
            Some((t, f)) if *f == sobre => t.push(s[1]),
            _ => out.push((vec![s[0], s[1]], sobre)),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{Rect, trocos};

    /// T50: a corda que cruza outro solto parte-se em três troços — fora · sobre · fora — e os
    /// troços partilham as pontas (sem buracos).
    #[test]
    fn a_rope_segment_over_another_loose_card_gets_its_own_tone() {
        let linha: Vec<(f32, f32)> = (0..=10).map(|i| (i as f32 * 10.0, 0.0)).collect();
        let t = trocos(&linha, &[Rect::new(30.0, -5.0, 30.0, 10.0)]);
        let f: Vec<bool> = t.iter().map(|x| x.1).collect();
        assert_eq!(f, [false, true, false]);
        assert_eq!(t[1].0, [(30.0, 0.0), (40.0, 0.0), (50.0, 0.0), (60.0, 0.0)]);
        assert_eq!(t[0].0.last(), t[1].0.first());
        assert_eq!(trocos(&linha, &[]).len(), 1);
    }
}
