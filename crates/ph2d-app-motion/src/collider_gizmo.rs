//! ⭐⭐ **O GIZMO DO COLISOR DA FORMA** — doc 109 §5, report do dono (2026-09-13): *«no mínimo
//! precisamos de colliders circulares e retangulares […] e que tenham ajustes de tamanho com gizmo
//! visível para o usuário»*, e o report seguinte: *«o collider deve aparecer na frente da shape
//! (z-index maior) e visível em TODAS as formas. Coloque um botão no nó shape: Ver collider»*.
//!
//! ## O que se vê
//!
//! Com a tool Motion activa, **toda** forma que tem `Collide` e `Show Collider` ligados pinta o
//! contorno do colisor de cada peça que carimbou — seleccionada ou não, e **por cima da arte** (a
//! fase que desenha o gizmo corre depois da que codifica as formas; há gate de ordem na shell).
//!
//! As **ALÇAS** ficam numa peça só: a mais próxima do cursor, na forma SELECCIONADA. ⚠️ Numa só
//! porque o colisor é UM, da forma — arrastar a alça de uma cópia muda todas, e vinte e cinco
//! conjuntos de alças seriam vinte e cinco alvos para o mesmo número; e na seleccionada porque o
//! arrasto edita os params DELA.
//!
//! ## O que o arrasto escreve
//!
//! Os params do cartão — `Collider Width`/`Collider Height` numa caixa, `Collider Radius` num
//! círculo — pela MESMA porta do slider (`Graph::set_param`), com o arrasto inteiro num passo de
//! undo (`MotionHistory::begin` / `commit_if_changed`, o idioma do gizmo de field). O redimensionar
//! é SIMÉTRICO à volta do centro, porque o centro não é um param: é o meio da forma.
//!
//! ⚠️ **O arrasto parte do valor que o param TINHA e soma o que a mão ANDOU** — nunca «o lado vai
//! para onde está o cursor»: agarrar a alça a meio do raio de agarre faria o colisor saltar por essa
//! folga logo no primeiro pixel.
//!
//! ## Quem é peça de uma forma
//!
//! As linhas do sink dela com o MESMO `geometry_id` e a MESMA declaração local (as colunas do
//! colisor, ao bit) que a saída do nó. ⚠️ Duas formas IGUAIS com o mesmo colisor são
//! indistinguíveis por aqui — as duas acendem, e o arrasto edita a seleccionada.

use crate::motion_bridge::params::param_value;
use crate::motion_shape_gen::collider::ColliderFit;
use crate::motion_state::MotionState;
use ph2d_contact::{Colisor, Forma};
use ph2d_node_motion_shape::param;
use ph2d_nodegraph::attr::{
    COLLIDER_BOX_COLUMN, COLLIDER_COLUMN, COLLIDER_OFFSET_COLUMN, Column, SIZE_IDENTITY, Stream,
};
use ph2d_nodegraph::graph::{Graph, NodeId};

/// **Quantos contornos o gizmo desenha, no máximo** — os MAIS PRÓXIMOS do cursor, somados sobre
/// todas as formas, e a peça com alças nunca cai fora. ⚠️ O recurso é o tempo de CODIFICAR os
/// caminhos no quadro; o número e a tabela estão na sonda `measure_the_outline_paint_cost`.
pub const MAX_CONTORNOS: usize = 2048;

/// **Uma peça**, no sítio onde o sink a entrega.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Peca {
    /// O índice da linha no stream do sink — o que o arrasto segura.
    pub linha: usize,
    pub p: [f32; 2],
    /// O colisor de MUNDO, pela porta do solver.
    pub colisor: Colisor,
    /// O `size` da linha — o que converte um arrasto de mundo num multiplicador do cartão.
    pub size: [f32; 2],
}

/// **As peças de UMA forma** e o que o cartão dela diz.
#[derive(Clone, Debug, PartialEq)]
pub struct Grupo {
    pub node: NodeId,
    /// `Circle` (senão `Box`).
    pub circulo: bool,
    /// A caixa envolvente da geometria, em unidade de geometria — a base dos multiplicadores.
    pub fit: ColliderFit,
    pub pecas: Vec<Peca>,
}

/// **O retrato do gizmo deste quadro** — tudo o que o pintor e o ponteiro precisam, já resolvido.
#[derive(Clone, Debug, PartialEq)]
pub struct ColliderGizmoView {
    /// Uma entrada por forma que mostra o colisor.
    pub grupos: Vec<Grupo>,
    /// O grupo e a peça que têm as ALÇAS — a forma seleccionada, a peça mais próxima do cursor.
    pub ativa: Option<(usize, usize)>,
}

impl ColliderGizmoView {
    /// A peça com alças, se houver.
    pub fn peca_ativa(&self) -> Option<(&Grupo, &Peca)> {
        let (g, p) = self.ativa?;
        let grupo = self.grupos.get(g)?;
        Some((grupo, grupo.pecas.get(p)?))
    }
}

/// **Uma alça**: a direcção dela no referencial do colisor, `x, y ∈ {−1, 0, 1}`. Numa caixa, um
/// canto tem as duas e um lado uma só; num círculo só há as quatro dos eixos.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Alca {
    pub x: i8,
    pub y: i8,
}

/// Uma alça e onde ela está, em mundo.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Handle {
    pub alca: Alca,
    pub world: [f32; 2],
}

/// **As alças de uma peça** — oito numa caixa (cantos e lados, girados com ela), quatro num círculo.
pub fn handles(peca: &Peca) -> Vec<Handle> {
    let c = peca.colisor.centro(peca.p);
    match peca.colisor.forma {
        Forma::Caixa { meia, eixo } => {
            let v = [-eixo[1], eixo[0]];
            [
                (1, 1),
                (0, 1),
                (-1, 1),
                (-1, 0),
                (-1, -1),
                (0, -1),
                (1, -1),
                (1, 0),
            ]
            .into_iter()
            .map(|(x, y): (i8, i8)| {
                let (fx, fy) = (f32::from(x) * meia[0], f32::from(y) * meia[1]);
                Handle {
                    alca: Alca { x, y },
                    world: [
                        c[0] + fx * eixo[0] + fy * v[0],
                        c[1] + fx * eixo[1] + fy * v[1],
                    ],
                }
            })
            .collect()
        }
        Forma::Disco(r) => [(1, 0), (0, 1), (-1, 0), (0, -1)]
            .into_iter()
            .map(|(x, y): (i8, i8)| Handle {
                alca: Alca { x, y },
                world: [c[0] + f32::from(x) * r, c[1] + f32::from(y) * r],
            })
            .collect(),
    }
}

/// **A alça sob o cursor** — a mais próxima dentro do raio de agarre, que é o MESMO do gizmo de
/// warp: *o artista aprende um alcance, não um por gizmo*.
pub fn hit(hs: &[Handle], world: [f32; 2], world_per_px: f32) -> Option<usize> {
    let alcance = crate::warp_gizmo::GRAB_PX * world_per_px;
    hs.iter()
        .enumerate()
        .map(|(i, h)| (i, (h.world[0] - world[0]).hypot(h.world[1] - world[1])))
        .filter(|(_, d)| *d <= alcance)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(i, _)| i)
}

/// **O arrasto em curso** — congelado no `Down`: a peça, a alça, a base e de onde a mão partiu.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Arrasto {
    pub node: NodeId,
    /// A linha do sink agarrada — ela fica com as alças enquanto o arrasto dura.
    pub linha: usize,
    pub alca: Alca,
    pub circulo: bool,
    pub fit: ColliderFit,
    pub size: [f32; 2],
    /// O eixo `x` da caixa no `Down` (o círculo não o usa).
    pub eixo: [f32; 2],
    /// Onde a mão pousou, em mundo.
    pub ancora: [f32; 2],
    /// Os multiplicadores no `Down`: `[largura, altura]` numa caixa, `[raio, raio]` num círculo.
    pub inicio: [f32; 2],
}

/// **O que um arrasto até `world` escreve no cartão** — `(param, valor)`.
///
/// A meia extensão de MUNDO parte da que o param dava no `Down` e anda o que a mão andou ao longo
/// da direcção da alça; o multiplicador é essa meia sobre a base (`fit × |size|`). Nunca negativo:
/// passar o centro dá zero, e não uma caixa do avesso.
pub fn edits(a: &Arrasto, world: [f32; 2]) -> Vec<(&'static str, f32)> {
    let d = [world[0] - a.ancora[0], world[1] - a.ancora[1]];
    if a.circulo {
        let base = a.fit.half[0].max(a.fit.half[1]) * a.size[0].abs().max(a.size[1].abs());
        if base <= 0.0 || !base.is_finite() {
            return Vec::new();
        }
        let andou = d[0] * f32::from(a.alca.x) + d[1] * f32::from(a.alca.y);
        let r = (base * a.inicio[0] + andou).max(0.0);
        return vec![(param::COLLIDER_RADIUS, r / base)];
    }
    let v = [-a.eixo[1], a.eixo[0]];
    let local = [
        d[0] * a.eixo[0] + d[1] * a.eixo[1],
        d[0] * v[0] + d[1] * v[1],
    ];
    let mut out = Vec::with_capacity(2);
    for (k, lado, nome) in [
        (0, a.alca.x, param::COLLIDER_WIDTH),
        (1, a.alca.y, param::COLLIDER_HEIGHT),
    ] {
        let base = a.fit.half[k] * a.size[k].abs();
        if lado == 0 || base <= 0.0 || !base.is_finite() {
            continue;
        }
        let meia = (base * a.inicio[k] + f32::from(lado) * local[k]).max(0.0);
        out.push((nome, meia / base));
    }
    out
}

/// **A forma seleccionada, se tiver o colisor ligado** — quem recebe as alças.
pub fn selected_collider_shape(motion: &MotionState) -> Option<NodeId> {
    let nid = crate::motion_bridge::params::selected_motion_node().map(NodeId)?;
    mostra_colisor(motion, nid).then_some(nid)
}

/// Este nó é uma forma que DECLARA e MOSTRA o colisor?
fn mostra_colisor(motion: &MotionState, nid: NodeId) -> bool {
    motion
        .doc
        .graph
        .node(nid)
        .is_some_and(|n| n.type_name == ph2d_node_motion_shape::MANIFEST.name)
        && param_value(motion, nid, param::COLLIDE) >= 0.5
        && param_value(motion, nid, param::SHOW_COLLIDER) >= 0.5
}

/// **Todas as formas que mostram o colisor** — a população do gizmo (report do dono: *«visível em
/// todas as formas»*), na ordem do grafo.
pub fn shapes_showing_collider(motion: &MotionState) -> Vec<NodeId> {
    motion
        .doc
        .graph
        .nodes()
        .iter()
        .map(|n| n.id)
        .filter(|id| mostra_colisor(motion, *id))
        .collect()
}

/// **O SINK a que a forma chega** — em largura, pelas arestas NÃO atrasadas.
///
/// ⚠️ Não é o `warp_gizmo::sink_of`, que segue a primeira aresta de cada nó: numa simulação a zona
/// tem duas saídas e a primeira é a entrada ATRASADA do laço (`zone → wind`), e aquele passeio dava
/// a volta ao laço até ao limite de passos sem encontrar o sink — medido na `=114`.
pub fn sink_of(graph: &Graph, node: NodeId) -> Option<NodeId> {
    let mut visto: Vec<NodeId> = Vec::new();
    let mut fila = std::collections::VecDeque::from([node]);
    while let Some(cur) = fila.pop_front() {
        if visto.contains(&cur) {
            continue;
        }
        visto.push(cur);
        if graph
            .node(cur)
            .is_some_and(|n| n.type_name == "motion.output")
        {
            return Some(cur);
        }
        for e in graph
            .edges()
            .iter()
            .filter(|e| e.from.0 == cur && !e.delayed)
        {
            fila.push_back(e.to.0);
        }
    }
    None
}

/// **As TOMADAS que este gizmo precisa**: por forma que mostra o colisor, a saída dela (a declaração
/// e a geometria) e o sink (as peças). Unidas às dos sinais e às do warp no `motion_bridge`.
pub fn taps_for(motion: &MotionState) -> Vec<NodeId> {
    let mut out = Vec::new();
    for nid in shapes_showing_collider(motion) {
        if !out.contains(&nid) {
            out.push(nid);
        }
        if let Some(s) = sink_of(&motion.doc.graph, nid)
            && !out.contains(&s)
        {
            out.push(s);
        }
    }
    out
}

fn tap(motion: &MotionState, node: NodeId) -> Option<&Stream> {
    motion
        .pump
        .tap_streams()
        .iter()
        .find(|(n, _)| *n == node)
        .map(|(_, s)| s)
}

/// As colunas da declaração de um stream, hasteadas — um `get` por coluna, não por linha.
struct Declaracao<'a> {
    geometria: Option<&'a [f32]>,
    raio: Option<&'a [f32]>,
    caixa: Option<&'a [[f32; 2]]>,
    desvio: Option<&'a [[f32; 2]]>,
}

impl<'a> Declaracao<'a> {
    fn de(s: &'a Stream) -> Self {
        let esc = |nome: &str| match s.get(nome) {
            Some(Column::Scalar(v)) if v.len() == s.count() => Some(v.as_slice()),
            _ => None,
        };
        let par = |nome: &str| match s.get(nome) {
            Some(Column::Vec2(v)) if v.len() == s.count() => Some(v.as_slice()),
            _ => None,
        };
        Self {
            geometria: esc("geometry_id"),
            raio: esc(COLLIDER_COLUMN),
            caixa: par(COLLIDER_BOX_COLUMN),
            desvio: par(COLLIDER_OFFSET_COLUMN),
        }
    }

    /// A declaração LOCAL da linha `i`, ao bit — uma coluna ausente lê `0`, como a união do
    /// `motion.combine` a preenche.
    fn linha(&self, i: usize) -> [u32; 6] {
        let esc = |c: Option<&[f32]>| c.map_or(0, |v| v[i].to_bits());
        let par =
            |c: Option<&[[f32; 2]]>| c.map_or([0, 0], |v| [v[i][0].to_bits(), v[i][1].to_bits()]);
        let (caixa, desvio) = (par(self.caixa), par(self.desvio));
        [
            esc(self.geometria),
            esc(self.raio),
            caixa[0],
            caixa[1],
            desvio[0],
            desvio[1],
        ]
    }
}

/// As peças de UMA forma, sem cortar pelo tecto — `None` quando a forma não tem gizmo neste quadro.
fn grupo_de(motion: &MotionState, node: NodeId) -> Option<Grupo> {
    let saida = tap(motion, node)?;
    if saida.count() == 0 {
        return None;
    }
    let alvo = Declaracao::de(saida).linha(0);
    let fit = motion
        .shape_store
        .measured_collider_fit(geometria(saida)?)?;
    let s = tap(motion, sink_of(&motion.doc.graph, node)?)?;
    let colisores = ph2d_contact::colisores(s)?;
    let p = match s.get("P") {
        Some(Column::Vec2(v)) if v.len() == s.count() => v,
        _ => return None,
    };
    let size = match s.get("size") {
        Some(Column::Vec2(v)) if v.len() == s.count() => Some(v),
        _ => None,
    };
    let decl = Declaracao::de(s);
    let pecas: Vec<Peca> = (0..s.count())
        .filter(|&i| decl.linha(i) == alvo)
        .filter_map(|i| {
            Some(Peca {
                linha: i,
                p: p[i],
                colisor: colisores[i]?,
                size: size.map_or(SIZE_IDENTITY, |v| v[i]),
            })
        })
        .collect();
    (!pecas.is_empty()).then(|| Grupo {
        node,
        circulo: param_value(motion, node, param::COLLIDER_SHAPE) >= 0.5,
        fit,
        pecas,
    })
}

/// **Resolve o retrato a partir do estado** — a porta única, para o laço de render e o gate lerem a
/// MESMA resposta. `pointer_world` escolhe a peça com alças e o que sobrevive ao tecto.
///
/// `None` quando: a tool não é a Motion · nenhuma forma mostra o colisor · as tomadas ainda não
/// trouxeram os streams · nenhuma peça do sink é de uma dessas formas.
pub fn resolve(
    motion: &MotionState,
    tool_is_motion: bool,
    pointer_world: Option<[f32; 2]>,
) -> Option<ColliderGizmoView> {
    if !tool_is_motion {
        return None;
    }
    let mut grupos: Vec<Grupo> = shapes_showing_collider(motion)
        .into_iter()
        .filter_map(|n| grupo_de(motion, n))
        .collect();
    if grupos.is_empty() {
        return None;
    }
    let dist = |q: &Peca| {
        let c = q.colisor.centro(q.p);
        pointer_world.map_or(0.0, |w| (c[0] - w[0]).hypot(c[1] - w[1]))
    };
    // A peça com ALÇAS: a agarrada, senão a mais próxima do cursor na forma SELECCIONADA.
    let preso = motion.collider_drag;
    let alvo_node = preso
        .map(|a| a.node)
        .or_else(|| selected_collider_shape(motion));
    let mut ativa = alvo_node.and_then(|node| {
        let gi = grupos.iter().position(|g| g.node == node)?;
        let pi = match preso.filter(|a| a.node == node) {
            Some(a) => grupos[gi].pecas.iter().position(|q| q.linha == a.linha)?,
            None => (0..grupos[gi].pecas.len()).min_by(|&x, &y| {
                dist(&grupos[gi].pecas[x]).total_cmp(&dist(&grupos[gi].pecas[y]))
            })?,
        };
        Some((gi, pi))
    });
    // O TECTO, somado sobre as formas: ficam as mais próximas do cursor, e a peça com alças nunca
    // cai fora — ela é o alvo do gesto que está em curso.
    let total: usize = grupos.iter().map(|g| g.pecas.len()).sum();
    if total > MAX_CONTORNOS {
        let mut todas: Vec<(usize, usize, f32)> = grupos
            .iter()
            .enumerate()
            .flat_map(|(gi, g)| {
                g.pecas
                    .iter()
                    .enumerate()
                    .map(move |(pi, q)| (gi, pi, dist(q)))
            })
            .collect();
        todas.select_nth_unstable_by(MAX_CONTORNOS - 1, |a, b| {
            let chave = |t: &(usize, usize, f32)| (Some((t.0, t.1)) != ativa, t.2);
            let (ka, kb) = (chave(a), chave(b));
            ka.0.cmp(&kb.0).then(ka.1.total_cmp(&kb.1))
        });
        todas.truncate(MAX_CONTORNOS);
        let mut fica: Vec<Vec<usize>> = vec![Vec::new(); grupos.len()];
        for (gi, pi, _) in todas {
            fica[gi].push(pi);
        }
        let antiga = ativa;
        ativa = None;
        for (gi, g) in grupos.iter_mut().enumerate() {
            fica[gi].sort_unstable();
            let mut i = 0;
            g.pecas.retain(|_| {
                let fica_esta = fica[gi].binary_search(&i).is_ok();
                if antiga == Some((gi, i)) && fica_esta {
                    ativa = Some((gi, fica[gi].binary_search(&i).unwrap_or(0)));
                }
                i += 1;
                fica_esta
            });
        }
        grupos.retain(|g| !g.pecas.is_empty());
    }
    Some(ColliderGizmoView { grupos, ativa })
}

/// O `geometry_id` da primeira linha, como o handle do store.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "um handle de geometria e' um inteiro >= 1 guardado num f32"
)]
fn geometria(s: &Stream) -> Option<u32> {
    match s.get("geometry_id") {
        Some(Column::Scalar(v)) => v.first().map(|g| g.max(0.0) as u32),
        _ => None,
    }
}

/// [`resolve`] com o cursor em PIXELS, projectado pela janela da CENA — a mesma porta que a tinta e
/// o agarre usam ([`crate::warp_gizmo::scene_window`]).
pub fn resolve_at(
    motion: &MotionState,
    tool_is_motion: bool,
    camera: &ph2d_render::Camera2d,
    center_split: ph2d_editor_core::screens::layout::CenterSplit,
    full_window: ph2d_host::WindowSize,
    pointer_px: (f32, f32),
) -> Option<ColliderGizmoView> {
    let win = crate::warp_gizmo::scene_window(center_split, full_window);
    resolve(
        motion,
        tool_is_motion,
        Some(camera.screen_to_world(pointer_px, win)),
    )
}

static VIEW: std::sync::Mutex<Option<ColliderGizmoView>> = std::sync::Mutex::new(None);

/// Publica (ou limpa) o retrato deste quadro. ⚠️ Publicar de novo SUBSTITUI: largar a selecção limpa
/// as alças em vez de as deixar a pairar.
pub fn publish(v: Option<ColliderGizmoView>) {
    if let Ok(mut slot) = VIEW.lock() {
        *slot = v;
    }
}

/// O retrato deste quadro, se houver.
pub fn view() -> Option<ColliderGizmoView> {
    VIEW.lock().ok().and_then(|s| s.clone())
}

/// **O `Down`**: agarra a alça sob o cursor, na peça que tem as alças. `true` = agarrou, e o clique
/// não segue para mais ninguém.
pub fn pointer_down(motion: &mut MotionState, world: [f32; 2], world_per_px: f32) -> bool {
    let Some(v) = view() else {
        return false;
    };
    let Some((grupo, peca)) = v.peca_ativa() else {
        return false;
    };
    let (node, circulo, fit, peca) = (grupo.node, grupo.circulo, grupo.fit, *peca);
    let hs = handles(&peca);
    let Some(i) = hit(&hs, world, world_per_px) else {
        return false;
    };
    let inicio = if circulo {
        let r = param_value(motion, node, param::COLLIDER_RADIUS);
        [r, r]
    } else {
        [
            param_value(motion, node, param::COLLIDER_WIDTH),
            param_value(motion, node, param::COLLIDER_HEIGHT),
        ]
    };
    let eixo = match peca.colisor.forma {
        Forma::Caixa { eixo, .. } => eixo,
        Forma::Disco(_) => [1.0, 0.0],
    };
    motion.collider_drag = Some(Arrasto {
        node,
        linha: peca.linha,
        alca: hs[i].alca,
        circulo,
        fit,
        size: peca.size,
        eixo,
        ancora: world,
        inicio,
    });
    motion.history.begin(&motion.doc);
    true
}

/// **O `Move`** com uma alça agarrada: escreve os params do cartão. `true` = consumiu.
pub fn pointer_move(motion: &mut MotionState, world: [f32; 2]) -> bool {
    let Some(a) = motion.collider_drag else {
        return false;
    };
    for (nome, valor) in edits(&a, world) {
        motion.doc.graph.set_param(a.node, nome, valor);
    }
    motion.pump.mark_dirty();
    true
}

/// **O `Up`**: larga a alça e fecha o passo de undo. `true` = havia um arrasto.
pub fn pointer_up(motion: &mut MotionState) -> bool {
    if motion.collider_drag.take().is_none() {
        return false;
    }
    motion.history.commit_if_changed(&motion.doc);
    true
}

#[cfg(test)]
#[path = "collider_gizmo_tests.rs"]
mod tests;
