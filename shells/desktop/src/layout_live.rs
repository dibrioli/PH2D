//! **O AUTO LAYOUT, vivo** — a moldura empilha os filhos, e as posições são derivadas por frame.
//!
//! O **8º produtor** de [`LiveGeometry`], e o **terceiro que TRANSFORMA o mapa** em vez de o
//! estender (os outros dois são o [`crate::align_live`] e o [`crate::bool_live`]). A razão é a
//! mesma: os cinco produtores que o `render_loop` funde com `extend` são mutuamente exclusivos —
//! uma forma é offset OU pattern OU contour —, e **o layout não é membro dessa família**: ele é um
//! componente do PAI, então cada filho que ele coloca pode ter, ele próprio, o seu offset vivo.
//! Fundido por `extend` ele apagaria esses offsets em silêncio.
//!
//! # A lei (ADR-0153)
//!
//! > **O passe publica ONDE as coisas ficam. Ele não escreve ONDE elas estão.**
//!
//! Nada aqui toca `Transform`. O undo deste editor é por DIFF do mundo ECS, então escrever a pose
//! derivada faria **cada frame de um redimensionamento virar um passo de undo** — e o layout
//! brigaria com o arrasto do artista dentro do mesmo frame.
//!
//! # A conversão de eixo é UMA, e é aqui
//!
//! O motor fala **CSS** (`y` para baixo, origem no canto superior-esquerdo da moldura); o documento
//! é **Y-up**. A troca acontece em [`world_target`] e em mais lugar nenhum — meia troca de eixo
//! espalhada por dois sítios é a forma de a moldura empilhar para cima num dia e para baixo noutro.
//!
//! # Posição e tamanho viajam pelo MESMO canal
//!
//! Um filho colocado ganha um afim `translate ∘ scale` aplicado à geometria de MUNDO dele **e à de
//! toda a sub-árvore dele** — é o que faz um grupo inteiro andar junto com a caixa que o contém.
//! O `scale` só é ≠ 1 quando o `grow`/`shrink` do filho de facto mudou o tamanho, e o neutro
//! (ninguém pediu nada) é a identidade: o mapa fica **intocado**, e o mundo pré-layout é
//! byte-idêntico.

use ph2d_ecs::{Entity, SimWorld, VecLayout, VecLayoutAbsolute, VecLayoutItem, VecLayoutSize};
use ph2d_vec_layout::{ItemStyle, Node, Solved};
use ph2d_vec_render::LiveGeometry;
use ph2d_vec_scene::{VecPath, VecPathId, VecScene, VecXforms, Xform, bake_xform, xform_of};

use crate::vec_entities::VecEntityMap;

/// Teto da caminhada de ancestrais — defesa contra hierarquia ciclada, não limite de produto (o
/// mesmo número, e pelo mesmo motivo, que o `vec_entities::MAX_DEPTH`).
const MAX_DEPTH: usize = 64;

/// **A geometria de MUNDO de um caminho neste frame** — o que o mapa já diz, ou a fonte assada.
///
/// A mesma lei do `align_live`/`bool_live`: o layout coloca *o que se DESENHA*, e não *o que está
/// guardado* — um filho com offset vivo tem de andar com o offset dele junto.
fn world_of(
    scene: &VecScene,
    xforms: &VecXforms,
    live: &LiveGeometry,
    id: VecPathId,
) -> Vec<VecPath> {
    match live.get(&id) {
        Some(items) => items.clone(),
        None => match scene.paths().iter().find(|p| p.id == id) {
            Some(p) => {
                let mut world = p.cooked().into_owned();
                bake_xform(&mut world, &xform_of(xforms, id));
                vec![world]
            }
            None => Vec::new(),
        },
    }
}

/// A caixa de MUNDO de uma lista de caminhos já assados. `None` quando não há geometria.
fn bbox_of(items: &[VecPath]) -> Option<([f64; 2], [f64; 2])> {
    let mut out: Option<([f64; 2], [f64; 2])> = None;
    for p in items {
        let Some((lo, hi)) = ph2d_vec_scene::curve_bbox_in_frame(p, 1.0, 0.0) else {
            continue;
        };
        out = Some(match out {
            None => (lo, hi),
            Some((a, b)) => (
                [a[0].min(lo[0]), a[1].min(lo[1])],
                [b[0].max(hi[0]), b[1].max(hi[1])],
            ),
        });
    }
    out
}

/// Um nó recolhido: o índice na fatia do motor, a entidade, e os caminhos que ele MOVE.
struct Collected {
    /// Todos os caminhos da sub-árvore deste nó — é o conjunto que anda junto com a caixa dele.
    paths: Vec<VecPathId>,
    /// A caixa de mundo que ele ocupa HOJE.
    bbox: ([f64; 2], [f64; 2]),
    /// A entidade deste nó e a moldura que o COLOCA — `None` na raiz, que não é colocada por
    /// ninguém. É o que o gesto de reordenar consulta depois (ver [`FlowSlots`]).
    who: Option<(Entity, Entity)>,
}

/// **O canto superior-esquerdo, em MUNDO, do retângulo que o motor resolveu.**
///
/// ⚠️ Aqui mora a troca de eixo, e ela é a única do passe: o motor mede `y` para BAIXO a partir do
/// topo da moldura, o documento mede para CIMA. `top - y` é a conversão inteira.
fn world_target(frame_bbox: ([f64; 2], [f64; 2]), solved: Solved) -> ([f64; 2], [f64; 2]) {
    let (lo, hi) = frame_bbox;
    let left = lo[0] + solved[0];
    let top = hi[1] - solved[1];
    ([left, top - solved[3]], [left + solved[2], top])
}

/// O afim que leva a caixa `from` à caixa `to` (translada e escala em torno do canto
/// superior-esquerdo). Escala degenerada (caixa de largura ou altura zero) vira `1` — mover uma
/// coisa sem tamanho é legítimo, esticá-la não significa nada.
fn fit(from: ([f64; 2], [f64; 2]), to: ([f64; 2], [f64; 2])) -> Xform {
    let (fw, fh) = (from.1[0] - from.0[0], from.1[1] - from.0[1]);
    let (tw, th) = (to.1[0] - to.0[0], to.1[1] - to.0[1]);
    let sx = if fw.abs() > 1e-9 { tw / fw } else { 1.0 };
    let sy = if fh.abs() > 1e-9 { th / fh } else { 1.0 };
    // Canto superior-esquerdo: x mínimo, y MÁXIMO (o mundo é Y-up).
    let (fx, fy) = (from.0[0], from.1[1]);
    let (tx, ty) = (to.0[0], to.1[1]);
    Xform([sx, 0.0, 0.0, sy, tx - sx * fx, ty - sy * fy])
}

#[path = "layout_live_slots.rs"]
mod slots_mod;
pub(crate) use slots_mod::{Box2, FlowSlots, Reading};

/// **O auto layout de toda a cena.** Roda DEPOIS da booleana (ele coloca *o que os filhos de facto
/// desenham*) e ANTES do alinhamento (que recorta a faixa do traço na largura AUTORADA — escalar
/// depois de a recortar mudaria a espessura dela).
#[derive(Default)]
pub(crate) struct LayoutLive {
    /// Quantos filhos o último passe colocou.
    placed: usize,
    /// Por moldura que flui (bits da entidade), onde os filhos dela ficaram.
    ///
    /// ⚠️ `BTreeMap` e não `HashMap`: a ordem de iteração é parte do que o editor guarda, e o
    /// `HashMap` é banido por lint neste repo justamente por não a ter.
    slots: std::collections::BTreeMap<u64, FlowSlots>,
    /// **A POSE que cada caminho colocado recebeu** — o mesmo afim que foi assado na geometria,
    /// publicado também como número.
    ///
    /// ⚠️ Assar serve a quem DESENHA; quem **aponta** e quem **anota** (as âncoras do modo Node,
    /// a caixa do gizmo, o hit-test) não desenha geometria nenhuma — lê a pose autorada, que não
    /// se mexeu. Sem esta tabela as âncoras ficam no lugar de origem e o clique procura a forma
    /// onde ela já não está.
    ///
    /// ⚠️ Os dois saem do MESMO `x`, uma linha um do outro: é isso que impede a pose publicada de
    /// divergir da geometria assada.
    poses: std::collections::BTreeMap<VecPathId, Xform>,
    /// Quantos filhos ANCORADOS o último passe moveu (o braço do W3).
    anchored: usize,
    /// **Quanto o conteúdo passa da moldura**, por moldura que flui — MEDIDO por este passe, em
    /// unidades do motor (`y` para baixo). Zero (ou negativo, guardado como zero) = cabe.
    ///
    /// ⚠️ Ele é **derivado**, nunca autorado: uma moldura rola porque o conteúdo dela não cabe, e
    /// não porque alguém marcou uma caixa. Um controlo ao lado seria um segundo lugar a discordar
    /// do primeiro no instante em que o artista acrescentasse um filho.
    overflow: std::collections::BTreeMap<u64, [f64; 2]>,
    /// **O deslocamento do conteúdo**, por moldura — o único campo desta struct que SOBREVIVE ao
    /// `recook`, porque ele é o gesto e não a medição.
    ///
    /// ⚠️ **VISTA, e não documento.** Rolar é olhar, não editar — e a razão é a mesma que fez o
    /// modo de preview existir (`render_loop::ui_preview`): *o undo deste editor é por DIFF do
    /// mundo*, então um deslocamento escrito no ECS faria **cada tique de roda virar um passo de
    /// undo**. Corolário honesto: ele não viaja no arquivo, e reabrir um projeto mostra o topo da
    /// lista — que é o que o artista espera de uma posição de rolagem.
    scroll: std::collections::BTreeMap<u64, [f64; 2]>,
    /// **As molduras que a roda pode rolar**, com a caixa em que ficaram — publicadas por este
    /// passe (a lei do `FlowSlots`: quem coloca é quem diz onde) e consumidas pelo gesto, que corre
    /// fora do frame e não tem a geometria viva na mão.
    scrollables: Vec<scroll::ScrollTarget>,
}

impl LayoutLive {
    /// ⚠️ **Hoje o único leitor é o gate**, e por isso o acessor é `cfg(test)`: um `pub` sem
    /// chamador não é código morto silencioso, é uma segunda resposta à espera de alguém a chamar
    /// (a lição do `warp_axis` no Painter). O segundo leitor nasce com a seção do painel, que
    /// precisa dele para decidir se oferece o Apply.
    #[cfg(test)]
    pub(crate) fn placed(&self) -> usize {
        self.placed
    }

    /// Quantos filhos ANCORADOS o último passe moveu — o irmão do [`Self::placed`], e `cfg(test)`
    /// pela mesma razão.
    #[cfg(test)]
    pub(crate) fn anchored(&self) -> usize {
        self.anchored
    }

    /// **A tabela de poses deste frame**, para o `VecViewState` que a shell publica.
    pub(crate) fn poses(&self) -> Vec<(VecPathId, Xform)> {
        self.poses.iter().map(|(id, x)| (*id, *x)).collect()
    }

    /// **A ÚNICA porta que publica uma pose** — e ela COMPÕE, nunca sobrescreve.
    ///
    /// ⚠️ Um caminho pode ser movido DUAS vezes no mesmo frame: o fluxo coloca a moldura interna e
    /// a âncora depois move a moldura externa inteira, ou o inverso. A geometria compõe sozinha (o
    /// segundo braço assa por cima do que o primeiro deixou em `live`), mas a TABELA não: um
    /// `insert` cru guardaria só o último afim, e quem **aponta** — o hit-test, a caixa do gizmo,
    /// as âncoras do modo Node — procuraria a forma a meio caminho de onde ela está.
    fn add_pose(&mut self, id: VecPathId, x: Xform) {
        let composed = match self.poses.get(&id) {
            Some(prev) => prev.then(&x),
            None => x,
        };
        self.poses.insert(id, composed);
    }

    /// Onde o último passe pôs os filhos desta moldura — `None` se ela não flui.
    pub(crate) fn slots_of(&self, frame: Entity) -> Option<&FlowSlots> {
        self.slots.get(&frame.to_bits())
    }

    /// Uma régua montada à mão — **só para os gates do gesto de reordenar**.
    ///
    /// ⚠️ Ela existe para o gate poder afirmar o GESTO sem montar uma cena vetorial inteira; o
    /// produto nunca a chama, e o `cfg(test)` é o que impede um segundo produtor de posições de
    /// nascer ao lado do passe (a lei do ADR-0153: a régua é publicada por quem coloca).
    #[cfg(test)]
    pub(crate) fn with_slots(frame: Entity, reading: Reading, kids: Vec<(Entity, Box2)>) -> Self {
        let mut me = Self::default();
        me.slots
            .insert(frame.to_bits(), FlowSlots { reading, kids });
        me
    }

    /// ⚠️ **O `tok` é o MODO + a RÉGUA** (W4c.4): um vão pode seguir um token de escala, e resolver
    /// um token de comprimento exige a régua px↔mundo do projeto. Ele viaja como par nomeado
    /// (`vec_bindings::TokenCtx`) pela razão que aquele tipo documenta.
    pub(crate) fn recook(
        &mut self,
        scene: &VecScene,
        sim: &SimWorld,
        map: &VecEntityMap,
        xforms: &VecXforms,
        live: &mut LiveGeometry,
        tok: crate::vec_bindings::TokenCtx,
    ) {
        self.placed = 0;
        self.anchored = 0;
        self.slots.clear();
        self.poses.clear();
        // ⚠️ **O excedente limpa; o deslocamento NÃO.** O primeiro é a medição deste passe (uma
        // moldura que deixou de fluir não tem excedente nenhum a reportar); o segundo é o gesto do
        // artista, e limpá-lo faria a lista saltar de volta ao topo a cada frame.
        self.overflow.clear();
        self.scrollables.clear();
        for frame in outermost_flowing_frames(scene, sim, map) {
            self.lay_out(scene, sim, xforms, live, frame, tok);
        }
        // ⚠️ **As âncoras DEPOIS do fluxo, e a ordem é load-bearing pela MEDIÇÃO** (plano UI/UX
        // W3). A âncora lê a caixa da moldura; o fluxo DECIDE essa caixa. Ao contrário, o fluxo
        // mediria a moldura pela sub-árvore que a âncora acabou de colocar — e a âncora colocou-a
        // lendo a caixa que o fluxo ainda ia decidir: um laço.
        //
        // ⚠️ E a razão NÃO é a que parece: para uma colocação que é `translate ∘ scale`, as duas
        // ordens dão o MESMO ponto (`S·(x+d) + t` contra `S·x + t + S·d` — a mesma expressão), e
        // a primeira versão do gate ficou verde sobre a ordem trocada por causa disso. O que
        // discrimina é só o tamanho MEDIDO do nó, e é exactamente isso que o gate
        // `the_anchor_does_not_feed_back_into_the_flows_measurement` afirma.
        self.anchor_all(scene, sim, map, xforms, live);
    }

    /// Uma moldura (e tudo o que flui dentro dela).
    fn lay_out(
        &mut self,
        scene: &VecScene,
        sim: &SimWorld,
        xforms: &VecXforms,
        live: &mut LiveGeometry,
        frame: Entity,
        tok: crate::vec_bindings::TokenCtx,
    ) {
        let w = sim.world();
        let mut nodes: Vec<Node> = Vec::new();
        let mut collected: Vec<Collected> = Vec::new();

        // A raiz: a própria moldura. O tamanho dela é o da caixa que ela DESENHA (o retângulo vivo
        // que a carrega, ADR-0153 W0) — nunca um segundo número guardado ao lado.
        let Some(root_paths) = own_paths(sim, scene, frame) else {
            return;
        };
        let root_world = world_of_all(scene, xforms, live, &root_paths);
        let Some(root_bbox) = bbox_of(&root_world) else {
            return;
        };
        let Some(root_layout) = w.get::<VecLayout>(frame) else {
            return;
        };
        let root_measured = [
            root_bbox.1[0] - root_bbox.0[0],
            root_bbox.1[1] - root_bbox.0[1],
        ];
        let (root_size, root_min, root_max) =
            size_of(w.get::<VecLayoutSize>(frame), true, root_measured);
        nodes.push(Node {
            parent: None,
            frame: Some(frame_style(
                root_layout,
                crate::vec_bindings::bound_gap(sim, frame, tok),
            )),
            item: ItemStyle::default(),
            size: root_size,
            min: root_min,
            max: root_max,
        });
        collected.push(Collected {
            // ⚠️ **A raiz É a régua, e mesmo assim entrega os caminhos dela.** Uma moldura que
            // ABRAÇA muda de tamanho, e o tamanho novo tem de ser DESENHADO — senão o abraço é um
            // número que o motor calcula e que ninguém vê. Quando ela não abraça o alvo é a caixa
            // que ela já ocupa, o afim sai identidade, e o `is_identity` do laço final não paga
            // sequer a cópia: o caminho de quem nunca pediu abraço fica byte-intocado.
            paths: root_paths.clone(),
            bbox: root_bbox,
            who: None,
        });

        // Os filhos, em largura, na ordem da HIERARQUIA — é ela que o artista vê e reordena.
        let mut queue: Vec<(Entity, usize)> = vec![(frame, 0)];
        while let Some((parent, parent_idx)) = queue.pop() {
            let Some(kids) = w.get::<ph2d_ecs::Children>(parent) else {
                continue;
            };
            for &kid in kids.iter() {
                // ⚠️ **O fora-do-fluxo sai da FATIA, e não do motor** — o *Absolute position* do
                // Figma. Um nó que o motor nunca vê fica exactamente com a pose que o artista lhe
                // deu, e continua a andar com o pai e a ser recortado por ele (ele não deixou de
                // ser filho na hierarquia). Dizê-lo ao motor em vez disto pediria um inset — quatro
                // números derivados que ninguém autorou, para reproduzir a posição que já existe.
                if w.get::<VecLayoutAbsolute>(kid).is_some() {
                    continue;
                }
                let flows_here = w.get::<VecLayout>(kid).is_some();
                // ⚠️ **Uma moldura que FLUI mede-se e move-se por SI, nunca pela sub-árvore.**
                //
                // Os filhos dela viram nós próprios, com transformação própria; incluí-los aqui
                // aplicaria a deles DUAS vezes — a do pai a mover a sub-árvore inteira, e a
                // própria — e a cada frame os netos fugiriam mais para fora da moldura. Um nó que
                // NÃO flui é o oposto: nada lá dentro é nó, então ele carrega a sub-árvore toda.
                let paths = if flows_here {
                    own_paths(sim, scene, kid).unwrap_or_default()
                } else {
                    crate::vec_entities::subtree_paths(sim, scene, kid)
                };
                if paths.is_empty() {
                    continue;
                }
                let items = world_of_all(scene, xforms, live, &paths);
                let Some(bbox) = bbox_of(&items) else {
                    continue;
                };
                let measured = [bbox.1[0] - bbox.0[0], bbox.1[1] - bbox.0[1]];
                let (size, min, max) = size_of(w.get::<VecLayoutSize>(kid), flows_here, measured);
                nodes.push(Node {
                    parent: Some(parent_idx),
                    frame: w
                        .get::<VecLayout>(kid)
                        .map(|l| frame_style(l, crate::vec_bindings::bound_gap(sim, kid, tok))),
                    item: item_style(w.get::<VecLayoutItem>(kid)),
                    size,
                    min,
                    max,
                });
                let idx = nodes.len() - 1;
                collected.push(Collected {
                    paths,
                    bbox,
                    who: Some((kid, parent)),
                });
                if flows_here {
                    queue.push((kid, idx));
                }
            }
        }
        if nodes.len() < 2 {
            return; // uma moldura sem filhos não coloca nada
        }

        let Ok(solved) = ph2d_vec_layout::solve(&nodes) else {
            // O motor recusou (uma árvore que este coletor não devia produzir): a arte fica como
            // está, em vez de piscar para posições que ninguém pediu.
            return;
        };

        // ⚠️ **Do ZERO, e o zero é a moldura.** Ela entra no laço porque um `Hug` muda o tamanho
        // dela, e o tamanho novo precisa de ser desenhado como o de qualquer filho que cresce. Sem
        // abraço o alvo dela é a caixa que ela já ocupa ⇒ o afim é a identidade ⇒ o `continue`
        // abaixo salta-a antes de qualquer cópia, e o mundo de quem nunca pediu abraço fica
        // byte-intocado.
        //
        // ⚠️ Os dois blocos que só valem para FILHOS já se guardam sozinhos: a régua de reordenar
        // pergunta `c.who` (a raiz não é colocada por ninguém, logo `None`), e o `placed` conta
        // colocações — redimensionar a própria moldura não é uma.
        // **A ROLAGEM**, em duas passadas sobre a árvore que o motor acabou de resolver — e as duas
        // são `O(nós)` porque a fatia vem com o **pai antes dos filhos** (é o que [`solve`] exige).
        let ents: Vec<Option<Entity>> = collected
            .iter()
            .enumerate()
            .map(|(i, c)| {
                if i == 0 {
                    Some(frame)
                } else {
                    c.who.map(|(k, _)| k)
                }
            })
            .collect();
        // A profundidade na árvore do FLUXO — a régua do *mais interno ganha* do alvo da roda.
        // Ela sai da fatia (pai antes do filho), e não do ECS: é a mesma árvore que o motor
        // resolveu, e uma segunda caminhada podia discordar dela.
        let mut depth = vec![0_usize; nodes.len()];
        for i in 0..nodes.len() {
            if let Some(p) = nodes[i].parent {
                depth[i] = depth[p] + 1;
            }
        }
        self.measure_overflow(&nodes, &solved, &ents);
        let off = self.scroll_offsets(&nodes, &ents);

        for (i, c) in collected.iter().enumerate() {
            // ⚠️ O deslocamento entra no espaço do MOTOR, e não no de mundo: a troca de eixo
            // continua a acontecer **num lugar só** (`world_target`), que é o que impede a
            // rolagem de descobrir sozinha que o mundo é Y-up e errar o sinal.
            let mut s = solved[i];
            s[0] -= off[i][0];
            s[1] -= off[i][1];
            let target = world_target(root_bbox, s);
            // **Publica ONDE este filho ficou**, antes de qualquer early-out: o gesto de reordenar
            // precisa da régua mesmo quando a colocação não moveu nada (uma moldura já arrumada é
            // exactamente onde o artista vai arrastar).
            if let Some((kid, parent)) = c.who
                && let Some(l) = w.get::<VecLayout>(parent)
            {
                // ⚠️ **`RowWrap` e `Grid` leem-se em LINHAS**, e o wrap sempre se leu — ele
                // dispunha em faixas e a régua media só o X desde que nasceu. O defeito ficou
                // invisível ali porque uma faixa de wrap raramente tem duas linhas alinhadas; numa
                // grade ele é visível em toda a primeira linha (medido: soltar na célula (0,0) de
                // uma 3×3 dava o slot 3, o começo da SEGUNDA linha).
                let reading = match l.dir {
                    ph2d_ecs::LayoutDir::Column => Reading::ColumnY,
                    ph2d_ecs::LayoutDir::Row => Reading::RowX,
                    ph2d_ecs::LayoutDir::RowWrap | ph2d_ecs::LayoutDir::Grid => Reading::Rows,
                };
                self.slots
                    .entry(parent.to_bits())
                    .or_insert_with(|| FlowSlots {
                        reading,
                        kids: Vec::new(),
                    })
                    .kids
                    .push((kid, target));
            }
            // **Este nó é alvo da roda?** Publicado aqui porque é aqui que a caixa final dele
            // existe — uma moldura que ABRAÇA acabou de mudar de tamanho, e re-medi-la noutro
            // sítio poria o alvo onde ela estava.
            if let Some(e) = ents[i] {
                let clips = w.get::<ph2d_ecs::VecFrame>(e).is_some_and(|f| f.clip);
                self.offer_scroll_target(e, clips, depth[i], target);
            }
            let x = fit(c.bbox, target);
            if is_identity(&x) {
                continue; // nada a fazer: não paga sequer a cópia
            }
            for &id in &c.paths {
                let mut items = world_of(scene, xforms, live, id);
                for p in &mut items {
                    bake_xform(p, &x);
                }
                live.insert(id, items);
                // O MESMO afim, como número: quem não desenha geometria precisa dele.
                self.add_pose(id, x);
            }
            if c.who.is_some() {
                self.placed += 1;
            }
        }
    }
}

fn is_identity(x: &Xform) -> bool {
    let e = 1e-9;
    (x.0[0] - 1.0).abs() < e
        && x.0[1].abs() < e
        && x.0[2].abs() < e
        && (x.0[3] - 1.0).abs() < e
        && x.0[4].abs() < e
        && x.0[5].abs() < e
}

/// A geometria de mundo de vários caminhos, concatenada.
fn world_of_all(
    scene: &VecScene,
    xforms: &VecXforms,
    live: &LiveGeometry,
    ids: &[VecPathId],
) -> Vec<VecPath> {
    ids.iter()
        .flat_map(|&id| world_of(scene, xforms, live, id))
        .collect()
}

/// Os caminhos da PRÓPRIA entidade (não da sub-árvore) — a moldura mede-se por si.
fn own_paths(sim: &SimWorld, scene: &VecScene, e: Entity) -> Option<Vec<VecPathId>> {
    let id = sim.world().get::<ph2d_ecs::VecPathRef>(e)?.0;
    scene.paths().iter().find(|p| p.id == id)?;
    Some(vec![id])
}

/// **As molduras que fluem e que não estão DENTRO de outra que flui.**
///
/// ⚠️ Uma moldura aninhada não entra nesta lista: ela é resolvida como parte da árvore da
/// ancestral, e é isso que faz o `grow` dela ser honrado antes de ela colocar os próprios filhos.
/// Processá-la também por fora seria colocá-la duas vezes, com a segunda a ler a caixa que a
/// primeira acabou de mover.
fn outermost_flowing_frames(scene: &VecScene, sim: &SimWorld, map: &VecEntityMap) -> Vec<Entity> {
    let w = sim.world();
    let mut found: Vec<Entity> = Vec::new();
    for path in scene.paths() {
        let Some(&bits) = map.get(&path.id) else {
            continue;
        };
        let mut cur = Entity::from_bits(bits);
        for _ in 0..MAX_DEPTH {
            if w.get::<VecLayout>(cur).is_some() && !found.contains(&cur) {
                found.push(cur);
            }
            match w.get::<ph2d_ecs::ChildOf>(cur) {
                Some(c) => cur = c.parent(),
                None => break,
            }
        }
    }
    // Fica só quem NÃO tem ancestral que flui.
    found.retain(|&e| !has_flowing_ancestor(sim, e));
    // Ordem estável entre frames.
    found.sort_unstable();
    found
}

fn has_flowing_ancestor(sim: &SimWorld, e: Entity) -> bool {
    let w = sim.world();
    let mut cur = e;
    for _ in 0..MAX_DEPTH {
        match w.get::<ph2d_ecs::ChildOf>(cur) {
            Some(c) => {
                cur = c.parent();
                if w.get::<VecLayout>(cur).is_some() {
                    return true;
                }
            }
            None => return false,
        }
    }
    false
}

/// O braço das ÂNCORAS — a outra metade do passe (plano UI/UX W3). Módulo FILHO para poder usar
/// os helpers privados daqui sem os tornar públicos, e para o dono da tabela de poses continuar a
/// ser UM.
#[path = "layout_live_anchors.rs"]
pub(crate) mod anchors;

/// O braço da ROLAGEM — a terceira metade do passe (o item 3 do estudo dos contêineres). Módulo
/// FILHO pela MESMA razão do das âncoras: o dono da tabela de poses continua a ser UM.
#[path = "layout_live_scroll.rs"]
pub(crate) mod scroll;

/// A TRADUÇÃO documento → motor. Módulo FILHO pelo teto de LOC, com o corte por assunto.
#[path = "layout_live_style.rs"]
mod style;
use style::{frame_style, item_style, size_of};

#[cfg(test)]
#[path = "layout_live_tests.rs"]
mod tests;
