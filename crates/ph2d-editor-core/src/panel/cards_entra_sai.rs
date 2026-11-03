//! ⭐ **T41 — a secção que aparece ou some com a seleção não salta** (`docs/CartoesUI/01_decisoes_do_dono.md`):
//! a que ENTRA nasce fechada e abre-se com a mola das alturas; a que SAI fica no seu lugar a
//! fechar-se e a esmaecer, pintada pela FOTO do último quadro em que existiu (o `CardDef` já não
//! vem: a seleção mudou). As alturas vão de `-CARD_GAP` a `-CARD_GAP` — com o vão, o cartão ocupa
//! zero e o vizinho não dá o salto do vão ao chegar.

use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_card_stack::{Item, Stack, metrics as m};
use ph2d_vector::{Affine, Compose, Mix, VectorScene, VelloBlend};
use std::cell::RefCell;
use std::collections::BTreeMap;

/// O cartão fechado: com o vão que o segue, ocupa zero.
const FECHADO: f32 = -m::CARD_GAP;

#[derive(Default)]
struct Painel {
    /// A foto de cada cartão no último quadro: a cena, o `y` onde se pintou e a altura natural.
    fotos: BTreeMap<NodeId, (VectorScene, f32, f32)>,
    /// Os que saem, com o índice onde estavam.
    saindo: Vec<(NodeId, usize)>,
}

thread_local! {
    static PAINEIS: RefCell<BTreeMap<NodeId, Painel>> = RefCell::default();
}

/// As alturas do quadro (as do anterior, na ordem de agora; um cartão novo pede só o cabeçalho e o
/// fantasma de um solto a sua linha), com os que entram a nascer fechados e os que saem a fechar.
pub(super) fn itens(
    panel: NodeId,
    stack: &mut Stack<NodeId>,
    antes: &[Item<NodeId>],
    ordem: &[NodeId],
    soltos: &[NodeId],
) -> Vec<Item<NodeId>> {
    let altura = |k: &NodeId| {
        if soltos.contains(k) {
            m::LOOSE_GHOST_H
        } else {
            antes
                .iter()
                .find(|it| it.key == *k)
                .map_or(m::CARD_HEAD_H, |it| it.h)
        }
    };
    let mut items: Vec<Item<NodeId>> = ordem
        .iter()
        .map(|k| Item {
            key: *k,
            h: altura(k),
        })
        .collect();
    PAINEIS.with(|p| {
        let mut p = p.borrow_mut();
        let e = p.entry(panel).or_default();
        if !antes.is_empty() {
            // Quem volta a meio da saída abre-se de onde está (a mola dele continua).
            for k in ordem {
                if !antes.iter().any(|it| it.key == *k) && !e.saindo.iter().any(|(s, _)| s == k) {
                    stack.enter(*k, FECHADO);
                }
            }
        }
        e.saindo.retain(|(k, _)| !ordem.contains(k));
        if !antes.is_empty() {
            for (i, it) in antes.iter().enumerate() {
                if !ordem.contains(&it.key) {
                    e.saindo.push((it.key, i));
                }
            }
        }
        let Painel { fotos, saindo } = e;
        fotos.retain(|k, _| ordem.contains(k) || saindo.iter().any(|(s, _)| s == k));
        for (k, i) in saindo.iter() {
            items.insert(
                (*i).min(items.len()),
                Item {
                    key: *k,
                    h: FECHADO,
                },
            );
        }
    });
    items
}

/// Pinta o cartão `k` pela `pinta` numa cena à parte (a foto que a saída usará) e cola-a em `scene`.
pub(super) fn fotografa(
    scene: &mut VectorScene,
    panel: NodeId,
    k: NodeId,
    r: Rect,
    pinta: impl FnOnce(&mut VectorScene) -> f32,
) -> f32 {
    PAINEIS.with(|p| {
        let mut p = p.borrow_mut();
        let fotos = &mut p.entry(panel).or_default().fotos;
        let foto = fotos
            .entry(k)
            .or_insert_with(|| (VectorScene::new(), 0.0, 0.0));
        foto.0.reset();
        let h = pinta(&mut foto.0);
        scene.append_transformed(&foto.0, Affine::IDENTITY);
        (foto.1, foto.2) = (r.y, h);
        h
    })
}

/// O cartão `k` a SAIR, na altura vista `r.h`: a foto recortada e esmaecida. `true` = já não sai
/// (fechou, ou não é um que sai).
pub(super) fn pinta_saida(scene: &mut VectorScene, panel: NodeId, k: NodeId, r: Rect) -> bool {
    PAINEIS.with(|p| {
        let mut p = p.borrow_mut();
        let e = p.entry(panel).or_default();
        if !e.saindo.iter().any(|(s, _)| *s == k) {
            return true;
        }
        if r.h <= FECHADO + 0.5 {
            e.saindo.retain(|(s, _)| *s != k);
            e.fotos.remove(&k);
            return true;
        }
        if let Some((foto, y, h0)) = e.fotos.get(&k)
            && r.h > 0.0
        {
            let alfa = ((r.h - FECHADO) / (h0 - FECHADO)).clamp(0.0, 1.0);
            let normal = VelloBlend::new(Mix::Normal, Compose::SrcOver);
            let corte = crate::paint::rect_to_vello(r);
            scene.push_layer_shape(normal, alfa, Affine::IDENTITY, &corte);
            scene.append_transformed(foto, Affine::translate((0.0, f64::from(r.y - y))));
            scene.pop_layer();
        }
        false
    })
}

/// Os que ainda saem em `panel` (os testes).
#[must_use]
pub fn saindo(panel: NodeId) -> Vec<NodeId> {
    PAINEIS.with(|p| {
        p.borrow()
            .get(&panel)
            .map(|e| e.saindo.iter().map(|(k, _)| *k).collect())
            .unwrap_or_default()
    })
}
