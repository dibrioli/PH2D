//! **As duas alças das pontas do conector** — os círculos que reposicionam onde a linha
//! encosta na forma.
//!
//! Selecione um conector e cada ponta ganha um círculo. Arrastá-lo diz **onde** a linha toca a
//! forma; largá-lo sobre outra forma **religa** a linha; e — o pedido que define o módulo —
//! puxá-lo para longe da forma **não solta o vínculo**: a ponta continua presa, só que num
//! ponto afastado, e continua andando quando a forma anda.
//!
//! # A ideia que faz tudo isso caber num só campo
//!
//! A ponta afastada parece exigir um conceito novo (uma "folga", um "offset"). Não exige. O
//! [`Anchor::Port`] já guarda um ponto **normalizado na caixa LOCAL da forma** — e nada obriga
//! `u`/`v` a ficarem dentro de `[0, 1]`. Um `u = 1.8` é simplesmente um ponto além da borda
//! direita, medido na régua da própria forma.
//!
//! Isso não é economia de campo: é o que faz o afastamento **girar e escalar junto com a
//! forma** de graça (ADR-0111). Uma folga em unidades de mundo teria de ser rotacionada à mão,
//! e escalaria errado no dia em que alguém escalasse a caixa. Aqui não há nada a manter — a
//! coordenada já vive no referencial certo.
//!
//! O preço: quem lê um `Port` **não pode clampar** `u`/`v` (e o `port_world` do
//! `connector_live` deixou de fazê-lo).
//!
//! # A zona do centro
//!
//! Largar a alça no MEIO da forma devolve a ponta ao modo **automático** (o lado de saída volta
//! a ser re-escolhido a cada frame conforme a outra forma se move — a âncora "verde" do
//! draw.io). É a única maneira de desfazer um port fixo, e é onde o olho a procura: o centro é
//! o "sem preferência".

use ph2d_ecs::{Anchor, ConnectorEnd};
use ph2d_vec_scene::{VecPathId, VecScene, VecXforms, Xform, xform_of};

/// Qual das duas pontas.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum EndSide {
    Start,
    End,
}

/// Meia-largura da zona do centro, em fração da caixa. Largar dentro dela = voltar ao
/// automático. `0.2` ⇒ os 40% centrais de cada eixo — grande o bastante para ser fácil de
/// acertar, pequeno o bastante para não roubar a borda, que é onde os ports úteis vivem.
const CENTRE_ZONE: f64 = 0.2;

/// A forma sob o cursor no momento de largar.
#[derive(Copy, Clone, Debug)]
pub(crate) struct DropTarget {
    pub id: VecPathId,
    /// A caixa LOCAL da forma (é nela que `u`/`v` são medidos).
    pub lo: [f64; 2],
    pub hi: [f64; 2],
    /// O afim local→mundo da forma.
    pub xform: Xform,
}

/// As coordenadas normalizadas de `world` na caixa local da forma. **Sem clamp** — é o que
/// permite a `u`/`v` saírem de `[0, 1]` e a ponta ficar afastada, ainda vinculada.
///
/// `None` se o afim é singular (forma escalada a zero) ou a caixa é degenerada: aí não há
/// régua em que medir, e o chamador cai no ponto solto.
pub(crate) fn uv_of(t: &DropTarget, world: [f64; 2]) -> Option<(f64, f64)> {
    let local = t.xform.inverse()?.apply(world);
    let (w, h) = (t.hi[0] - t.lo[0], t.hi[1] - t.lo[1]);
    if w.abs() < 1e-9 || h.abs() < 1e-9 {
        return None;
    }
    Some(((local[0] - t.lo[0]) / w, (local[1] - t.lo[1]) / h))
}

/// A âncora que um `(u, v)` descreve: o **centro** devolve a ponta ao automático; qualquer
/// outro ponto a fixa ali.
fn anchor_for(u: f64, v: f64) -> Anchor {
    if (u - 0.5).abs() < CENTRE_ZONE && (v - 0.5).abs() < CENTRE_ZONE {
        return Anchor::Floating;
    }
    #[expect(
        clippy::cast_possible_truncation,
        reason = "o port e uma coordenada de UI; f32 sobra"
    )]
    Anchor::Port {
        u: u as f32,
        v: v as f32,
    }
}

/// **A resolução do largar.** Puro de propósito: é aqui que mora a regra, e é aqui que ela é
/// testada — sem montar mundo, câmera nem ponteiro.
///
/// | onde a alça caiu | o que acontece |
/// |---|---|
/// | sobre uma forma, no **centro** dela | prende nela, em modo **automático** |
/// | sobre uma forma, fora do centro | prende nela, **naquele ponto** |
/// | no vazio, com a ponta já **presa** | **continua presa** — o ponto vira um `u`/`v` fora de `[0,1]`, e a linha se afasta sem soltar |
/// | no vazio, com a ponta **solta** | fica solta, no lugar novo |
///
/// A terceira linha é a que o Enio pediu, e a razão de esta função existir em vez de um `if`
/// espalhado pelo handler do ponteiro.
pub(crate) fn resolve_drop(
    current: &ConnectorEnd,
    drop_world: [f64; 2],
    over: Option<DropTarget>,
    bound_target: Option<DropTarget>,
) -> ConnectorEnd {
    // 1. Largou sobre uma forma: prende nela (religando, se for outra).
    if let Some(t) = over
        && let Some((u, v)) = uv_of(&t, drop_world)
    {
        return ConnectorEnd::Bound {
            target: t.id,
            anchor: anchor_for(u, v),
        };
    }
    // 2. Largou no vazio, mas a ponta JÁ ESTAVA PRESA: ela continua presa. O ponto é medido na
    //    régua da forma — e sai de `[0, 1]`, que é exatamente o que "afastar sem soltar"
    //    significa.
    if let (ConnectorEnd::Bound { .. }, Some(t)) = (current, bound_target)
        && let Some((u, v)) = uv_of(&t, drop_world)
    {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "o port e uma coordenada de UI; f32 sobra"
        )]
        return ConnectorEnd::Bound {
            target: t.id,
            anchor: Anchor::Port {
                u: u as f32,
                v: v as f32,
            },
        };
    }
    // 3. Solta, e continua solta.
    ConnectorEnd::Free { at: drop_world }
}

/// O alvo de uma ponta presa, resolvido na cena (para a regra 2 acima).
pub(crate) fn target_of(
    scene: &VecScene,
    xforms: &VecXforms,
    end: &ConnectorEnd,
) -> Option<DropTarget> {
    let ConnectorEnd::Bound { target, .. } = end else {
        return None;
    };
    let (lo, hi) = scene.path_curve_bbox(*target)?;
    Some(DropTarget {
        id: *target,
        lo,
        hi,
        xform: xform_of(xforms, *target),
    })
}

/// As duas alças, em MUNDO: as pontas da polilinha cozida do conector.
pub(crate) fn handle_points(scene: &VecScene, id: VecPathId) -> Option<([f64; 2], [f64; 2])> {
    let p = scene.paths().iter().find(|p| p.id == id)?;
    Some((p.verts.first()?.anchor, p.verts.last()?.anchor))
}

/// Qual alça o cursor pegou. A ponta do FIM ganha o desempate: é a que tem a seta, e é a que o
/// usuário mira quando as duas coincidem (num conector degenerado, colapsado num ponto).
pub(crate) fn hit(
    scene: &VecScene,
    id: VecPathId,
    world: [f64; 2],
    r: f64,
) -> Option<(EndSide, [f64; 2])> {
    let (a, b) = handle_points(scene, id)?;
    let near = |p: [f64; 2]| (p[0] - world[0]).hypot(p[1] - world[1]) <= r;
    if near(b) {
        return Some((EndSide::End, b));
    }
    if near(a) {
        return Some((EndSide::Start, a));
    }
    None
}

#[cfg(test)]
#[path = "connector_handles_tests.rs"]
mod tests;

// ── O gesto ──────────────────────────────────────────────────────────────────────────────

use crate::app_state::App;
use ph2d_ecs::{Entity, VecConnector};

/// O arrasto de uma alça (Down..Up).
#[derive(Clone, Debug)]
pub(crate) struct HandleDrag {
    pub(crate) path: VecPathId,
    side: EndSide,
    /// A ponta como ela **estava**. Durante o arrasto a ponta vira `Free` (para a linha seguir
    /// o cursor ao vivo, que é o preview), e o largar resolve a partir DAQUI — senão o vínculo
    /// original se perderia no meio do caminho e "afastar sem soltar" viraria "soltar".
    original: ConnectorEnd,
}

impl App {
    /// A ponta `side` do conector `path`, no componente.
    fn conn_end_mut(&mut self, path: VecPathId, side: EndSide) -> Option<&mut ConnectorEnd> {
        let &bits = self.vec_entities.get(&path)?;
        let gfx = self.gfx.as_mut()?;
        let c = gfx
            .sim
            .world_mut()
            .get_mut::<VecConnector>(Entity::from_bits(bits))?
            .into_inner();
        Some(match side {
            EndSide::Start => &mut c.start,
            EndSide::End => &mut c.end,
        })
    }

    /// **Down.** A pressão caiu numa alça do conector selecionado? Abre o arrasto.
    ///
    /// `false` = não era alça, e o clique segue o caminho de sempre (picking, gizmo). É o
    /// contrato que mantém o resto do editor intacto: a alça só rouba o clique quando ela
    /// realmente está ali.
    pub(crate) fn conn_handle_down(&mut self, world: [f64; 2]) -> bool {
        // O MESMO raio em que o círculo é desenhado (`ph2d-vec-render`). Duas constantes
        // fariam o usuário clicar no meio da bolinha e não pegar nada — e a tela estaria
        // certa, que é o sintoma mais enlouquecedor que existe.
        let r = self.vec_px_to_world() * ph2d_vec_render::HANDLE_R_PX;
        let Some(gfx) = self.gfx.as_ref() else {
            return false;
        };
        // Só os conectores da SELEÇÃO têm alça — senão toda linha da tela seria um campo minado
        // de pontos agarráveis.
        let found = self.vec_pen.selected_paths().iter().find_map(|&id| {
            let &bits = self.vec_entities.get(&id)?;
            let c = gfx
                .sim
                .world()
                .get::<VecConnector>(Entity::from_bits(bits))?;
            let (side, _) = hit(&gfx.vec_scene, id, world, r)?;
            let original = match side {
                EndSide::Start => c.start,
                EndSide::End => c.end,
            };
            Some(HandleDrag {
                path: id,
                side,
                original,
            })
        });
        let Some(drag) = found else {
            return false;
        };
        // A ponta vira SOLTA no cursor: o re-cook do frame já desenha a linha seguindo a mão.
        // Não há caminho de preview separado — o preview é o conector.
        if let Some(end) = self.conn_end_mut(drag.path, drag.side) {
            *end = ConnectorEnd::Free { at: world };
        }
        self.vec_conn_handle = Some(drag);
        true
    }

    /// **Move.** A ponta acompanha o cursor. `false` sem arrasto armado.
    pub(crate) fn conn_handle_move(&mut self, x: f32, y: f32) -> bool {
        let Some(drag) = self.vec_conn_handle.clone() else {
            return false;
        };
        let Some(world) = self.vec_world_at((x, y)) else {
            return false;
        };
        if let Some(end) = self.conn_end_mut(drag.path, drag.side) {
            *end = ConnectorEnd::Free { at: world };
        }
        true
    }

    /// **Up.** Resolve o largar ([`resolve_drop`]) e fecha o arrasto.
    pub(crate) fn conn_handle_up(&mut self, world: [f64; 2]) -> bool {
        let Some(drag) = self.vec_conn_handle.take() else {
            return false;
        };
        let over = self.shape_under_cursor(world);
        let Some(gfx) = self.gfx.as_ref() else {
            return false;
        };
        let xforms = crate::vec_transform::build(&gfx.sim, &self.vec_entities);
        let mk = |id: VecPathId| -> Option<DropTarget> {
            let (lo, hi) = gfx.vec_scene.path_curve_bbox(id)?;
            Some(DropTarget {
                id,
                lo,
                hi,
                xform: xform_of(&xforms, id),
            })
        };
        let over = over.and_then(mk);
        let bound = target_of(&gfx.vec_scene, &xforms, &drag.original);
        let resolved = resolve_drop(&drag.original, world, over, bound);
        if let Some(end) = self.conn_end_mut(drag.path, drag.side) {
            *end = resolved;
        }
        true
    }

    /// **Cancela** (Esc, ou o ponteiro sumiu): a ponta volta ao que era.
    pub(crate) fn conn_handle_cancel(&mut self) -> bool {
        let Some(drag) = self.vec_conn_handle.take() else {
            return false;
        };
        if let Some(end) = self.conn_end_mut(drag.path, drag.side) {
            *end = drag.original;
        }
        true
    }
}

/// As alças a DESENHAR neste frame: `(ponto de mundo, está fixada?)`. Vazio quando nenhum
/// conector está selecionado.
///
/// **Função livre, e não um método de `App`**: o passe de render já tem `sim`/`scene`
/// emprestados de dentro do `gfx`, e um `&self` por cima colidiria com esses empréstimos. Os
/// quatro argumentos são exatamente o que ela lê — e o preço de os passar é ela ficar testável
/// sem montar um `App`.
///
/// O `bool` é o que separa a bolinha **automática** da **fixa** (o verde e o azul do draw.io):
/// sem essa distinção o usuário não tem como saber, olhando, se aquela ponta ainda escolhe o
/// lado sozinha ou se ele a pregou ali.
pub(crate) fn view(
    sim: &ph2d_ecs::SimWorld,
    scene: &VecScene,
    map: &crate::vec_entities::VecEntityMap,
    selected: &[VecPathId],
) -> Vec<([f64; 2], bool)> {
    let pinned = |e: &ConnectorEnd| {
        matches!(
            e,
            ConnectorEnd::Bound {
                anchor: Anchor::Port { .. },
                ..
            }
        )
    };
    let mut out = Vec::new();
    for &id in selected {
        let Some(&bits) = map.get(&id) else { continue };
        let Some(c) = sim.world().get::<VecConnector>(Entity::from_bits(bits)) else {
            continue;
        };
        let Some((a, b)) = handle_points(scene, id) else {
            continue;
        };
        out.push((a, pinned(&c.start)));
        out.push((b, pinned(&c.end)));
    }
    out
}
