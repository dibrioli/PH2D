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

/// A folga (em múltiplos do raio da alça) com que a pressão pega o CORPO da linha para fincar um
/// waypoint. Mais generosa que a alça: agarrar a linha para dobrá-la é um gesto grosso, e errá-lo
/// só custa um clique — enquanto uma alça agarrada por engano move uma ponta.
const BODY_GRAB_K: f64 = 1.4;

/// Amostras por segmento na busca do ponto mais próximo do corpo da linha.
const BODY_SAMPLES: u32 = 24;

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

/// O que a pressão agarrou.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum Grab {
    /// Uma das duas pontas (o círculo).
    End(EndSide),
    /// Um ponto de passagem (o quadradinho), pelo índice.
    Waypoint(usize),
}

/// O arrasto de uma alça (Down..Up).
#[derive(Clone, Debug)]
pub(crate) struct HandleDrag {
    pub(crate) path: VecPathId,
    grab: Grab,
    /// A ponta como ela **estava** (só para [`Grab::End`]). Durante o arrasto a ponta vira
    /// `Free` (para a linha seguir o cursor ao vivo, que é o preview), e o largar resolve a
    /// partir DAQUI — senão o vínculo original se perderia no meio do caminho e "afastar sem
    /// soltar" viraria "soltar".
    original: ConnectorEnd,
    /// O waypoint acabou de NASCER neste gesto (a pressão caiu no corpo da linha). Se o usuário
    /// soltar sem arrastar, ele é desfeito: um clique perdido não pode fincar um ponto invisível
    /// exatamente em cima da linha.
    born_now: bool,
}

/// A menor distância do ponto `p` ao segmento `a`–`b`.
fn dist_to_segment(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let ab = [b[0] - a[0], b[1] - a[1]];
    let len2 = ab[0] * ab[0] + ab[1] * ab[1];
    let t = if len2 < 1e-12 {
        0.0
    } else {
        (((p[0] - a[0]) * ab[0] + (p[1] - a[1]) * ab[1]) / len2).clamp(0.0, 1.0)
    };
    let q = [a[0] + ab[0] * t, a[1] + ab[1] * t];
    (p[0] - q[0]).hypot(p[1] - q[1])
}

/// **Onde enfiar um waypoint NOVO na lista.**
///
/// A ordem dos waypoints é a ordem em que a linha passa por eles, então inserir no fim seria um
/// bug: fincar um ponto perto da origem faria a linha ir até o fim, voltar, e seguir. O índice
/// certo é o que **menos alonga** a rota — o critério clássico de inserção (o desvio que o ponto
/// novo acrescenta ao trecho que ele parte).
///
/// `stations` são os pontos das estações **em ordem**: a saída, os waypoints, a chegada.
pub(crate) fn insert_index(stations: &[[f64; 2]], p: [f64; 2]) -> usize {
    let mut best = (f64::INFINITY, 0usize);
    for i in 0..stations.len().saturating_sub(1) {
        let (a, b) = (stations[i], stations[i + 1]);
        let detour = (p[0] - a[0]).hypot(p[1] - a[1]) + (b[0] - p[0]).hypot(b[1] - p[1])
            - (b[0] - a[0]).hypot(b[1] - a[1]);
        if detour < best.0 {
            best = (detour, i);
        }
    }
    best.1
}

/// Um waypoint largado **em cima da reta entre os vizinhos dele** não diz nada — ele nao dobra a
/// rota. Removê-lo ali é o gesto natural de desfazer um ponto de passagem (arraste-o de volta
/// para a linha), e de quebra apaga o que um clique perdido criaria.
///
/// `tol` em unidades de MUNDO (o chamador converte dos pixels da alça).
pub(crate) fn is_redundant(stations: &[[f64; 2]], i: usize, tol: f64) -> bool {
    // `i` é o índice do waypoint; nas estações ele está em `i + 1` (a saída vem antes).
    let k = i + 1;
    if k == 0 || k + 1 >= stations.len() {
        return false;
    }
    dist_to_segment(stations[k], stations[k - 1], stations[k + 1]) <= tol
}

impl App {
    /// O componente do conector `path`, mutável.
    fn conn_mut(&mut self, path: VecPathId) -> Option<&mut VecConnector> {
        let &bits = self.vec_entities.get(&path)?;
        let gfx = self.gfx.as_mut()?;
        Some(
            gfx.sim
                .world_mut()
                .get_mut::<VecConnector>(Entity::from_bits(bits))?
                .into_inner(),
        )
    }

    /// Escreve o ponto de mundo `at` no que o gesto agarrou: a ponta vira SOLTA ali (o preview
    /// é o conector), ou o waypoint se muda para lá.
    fn conn_grab_set(&mut self, path: VecPathId, grab: Grab, at: [f64; 2]) {
        let Some(c) = self.conn_mut(path) else { return };
        match grab {
            Grab::End(EndSide::Start) => c.start = ConnectorEnd::Free { at },
            Grab::End(EndSide::End) => c.end = ConnectorEnd::Free { at },
            Grab::Waypoint(i) => {
                if let Some(w) = c.waypoints.get_mut(i) {
                    *w = at;
                }
            }
        }
    }

    /// Os pontos das ESTAÇÕES em ordem (saída, waypoints, chegada) — a régua em que um waypoint
    /// novo acha o seu lugar e um redundante é reconhecido.
    fn conn_stations(&self, path: VecPathId) -> Option<Vec<[f64; 2]>> {
        let gfx = self.gfx.as_ref()?;
        let &bits = self.vec_entities.get(&path)?;
        let c = gfx
            .sim
            .world()
            .get::<VecConnector>(Entity::from_bits(bits))?;
        let (a, b) = handle_points(&gfx.vec_scene, path)?;
        let mut out = Vec::with_capacity(c.waypoints.len() + 2);
        out.push(a);
        out.extend(c.waypoints.iter().copied());
        out.push(b);
        Some(out)
    }

    /// **Down.** A pressão caiu numa alça do conector selecionado? Abre o arrasto.
    ///
    /// Três alvos, nesta ordem: a **ponta** (o círculo), um **waypoint** (o quadradinho) e — o
    /// gesto que cria — o **corpo da linha**. Pressionar o corpo de um conector JÁ SELECIONADO
    /// finca um ponto de passagem ali e passa a arrastá-lo, que é o gesto do draw.io: o ponto
    /// nasce debaixo do dedo, e o usuário o leva para onde quer.
    ///
    /// `false` = não era nenhum dos três, e o clique segue o caminho de sempre (picking, gizmo).
    /// É o contrato que mantém o resto do editor intacto: a alça só rouba o clique quando ela
    /// realmente está ali.
    pub(crate) fn conn_handle_down(&mut self, world: [f64; 2]) -> bool {
        // O MESMO raio em que o círculo é desenhado (`ph2d-vec-render`). Duas constantes
        // fariam o usuário clicar no meio da bolinha e não pegar nada — e a tela estaria
        // certa, que é o sintoma mais enlouquecedor que existe.
        let px = self.vec_px_to_world();
        let r = px * ph2d_vec_render::HANDLE_R_PX;
        let body_r = px * ph2d_vec_render::HANDLE_R_PX * BODY_GRAB_K;
        let Some(gfx) = self.gfx.as_ref() else {
            return false;
        };

        // Só os conectores da SELEÇÃO têm alça — senão toda linha da tela seria um campo minado
        // de pontos agarráveis.
        let mut found: Option<(HandleDrag, [f64; 2])> = None;
        for &id in self.vec_pen.selected_paths() {
            let Some(&bits) = self.vec_entities.get(&id) else {
                continue;
            };
            let Some(c) = gfx.sim.world().get::<VecConnector>(Entity::from_bits(bits)) else {
                continue;
            };
            // 1. As pontas.
            if let Some((side, _)) = hit(&gfx.vec_scene, id, world, r) {
                let original = match side {
                    EndSide::Start => c.start,
                    EndSide::End => c.end,
                };
                found = Some((
                    HandleDrag {
                        path: id,
                        grab: Grab::End(side),
                        original,
                        born_now: false,
                    },
                    world,
                ));
                break;
            }
            // 2. Um waypoint já existente.
            if let Some(i) = c
                .waypoints
                .iter()
                .position(|w| (w[0] - world[0]).hypot(w[1] - world[1]) <= r)
            {
                found = Some((
                    HandleDrag {
                        path: id,
                        grab: Grab::Waypoint(i),
                        original: c.start,
                        born_now: false,
                    },
                    world,
                ));
                break;
            }
            // 3. O CORPO da linha: finca um waypoint novo ali.
            let Some(p) = gfx.vec_scene.paths().iter().find(|p| p.id == id) else {
                continue;
            };
            let near = ph2d_vec_scene::nearest_point_on_path(p, world, BODY_SAMPLES)
                .is_some_and(|(_, _, d2)| d2.sqrt() <= body_r);
            if near {
                let mut stations = vec![handle_points(&gfx.vec_scene, id).map(|(a, _)| a)];
                stations.extend(c.waypoints.iter().map(|&w| Some(w)));
                stations.push(handle_points(&gfx.vec_scene, id).map(|(_, b)| b));
                let stations: Vec<[f64; 2]> = stations.into_iter().flatten().collect();
                let i = insert_index(&stations, world);
                found = Some((
                    HandleDrag {
                        path: id,
                        grab: Grab::Waypoint(i),
                        original: c.start,
                        born_now: true,
                    },
                    world,
                ));
                break;
            }
        }

        let Some((drag, at)) = found else {
            return false;
        };
        if drag.born_now
            && let Grab::Waypoint(i) = drag.grab
        {
            if let Some(c) = self.conn_mut(drag.path) {
                c.waypoints.insert(i, at);
            }
        } else {
            // A ponta vira SOLTA no cursor: o re-cook do frame já desenha a linha seguindo a mão.
            // Não há caminho de preview separado — o preview é o conector.
            self.conn_grab_set(drag.path, drag.grab, at);
        }
        self.vec_conn_handle = Some(drag);
        true
    }

    /// **Move.** O que foi agarrado acompanha o cursor. `false` sem arrasto armado.
    pub(crate) fn conn_handle_move(&mut self, x: f32, y: f32) -> bool {
        let Some(drag) = self.vec_conn_handle.clone() else {
            return false;
        };
        let Some(world) = self.vec_world_at((x, y)) else {
            return false;
        };
        self.conn_grab_set(drag.path, drag.grab, world);
        true
    }

    /// **Up.** Resolve o largar e fecha o arrasto.
    ///
    /// Para uma PONTA, o [`resolve_drop`] decide (religar, fixar, afastar sem soltar). Para um
    /// WAYPOINT, o que decide é ele ter virado **redundante**: largado em cima da reta entre os
    /// vizinhos, ele não dobra a rota, e some. É o mesmo gesto que desfaz um ponto de passagem
    /// (arraste-o de volta para a linha) e que apaga o que um clique perdido criaria — um ponto
    /// invisível, exatamente sobre a linha, que o usuário nunca saberia que existe.
    pub(crate) fn conn_handle_up(&mut self, world: [f64; 2]) -> bool {
        let Some(drag) = self.vec_conn_handle.take() else {
            return false;
        };
        if let Grab::Waypoint(i) = drag.grab {
            let tol = self.vec_px_to_world() * ph2d_vec_render::HANDLE_R_PX;
            self.conn_grab_set(drag.path, drag.grab, world);
            let dead = self
                .conn_stations(drag.path)
                .is_some_and(|st| is_redundant(&st, i, tol));
            if dead
                && let Some(c) = self.conn_mut(drag.path)
                && i < c.waypoints.len()
            {
                c.waypoints.remove(i);
            }
            return true;
        }

        let Grab::End(_) = drag.grab else {
            return true;
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
        if let Some(c) = self.conn_mut(drag.path) {
            match drag.grab {
                Grab::End(EndSide::Start) => c.start = resolved,
                Grab::End(EndSide::End) => c.end = resolved,
                Grab::Waypoint(_) => {}
            }
        }
        true
    }

    /// **Cancela** (Esc, botão direito): tudo volta ao que era. Um waypoint que nasceu neste
    /// gesto é desfeito — desistir não pode deixar um ponto para trás.
    pub(crate) fn conn_handle_cancel(&mut self) -> bool {
        let Some(drag) = self.vec_conn_handle.take() else {
            return false;
        };
        match drag.grab {
            Grab::Waypoint(i) if drag.born_now => {
                if let Some(c) = self.conn_mut(drag.path)
                    && i < c.waypoints.len()
                {
                    c.waypoints.remove(i);
                }
            }
            Grab::End(side) => {
                if let Some(c) = self.conn_mut(drag.path) {
                    match side {
                        EndSide::Start => c.start = drag.original,
                        EndSide::End => c.end = drag.original,
                    }
                }
            }
            Grab::Waypoint(_) => {}
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

/// Os WAYPOINTS a desenhar neste frame (em MUNDO). Vazio quando nenhum conector está
/// selecionado — como as alças de ponta, eles são controles da seleção, não enfeite da tela.
pub(crate) fn waypoint_view(
    sim: &ph2d_ecs::SimWorld,
    map: &crate::vec_entities::VecEntityMap,
    selected: &[VecPathId],
) -> Vec<[f64; 2]> {
    let mut out = Vec::new();
    for &id in selected {
        let Some(&bits) = map.get(&id) else { continue };
        if let Some(c) = sim.world().get::<VecConnector>(Entity::from_bits(bits)) {
            out.extend(c.waypoints.iter().copied());
        }
    }
    out
}
