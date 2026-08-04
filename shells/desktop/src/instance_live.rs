//! **A INSTÂNCIA de componente, viva** — o mestre desenhado na pose da cópia (plano UI/UX W5).
//!
//! O **9º produtor** de [`LiveGeometry`], e um dos que **ESTENDEM** o mapa: uma instância é uma
//! forma, e a geometria dela é derivada como a de um offset ou de um contour. Ela não é membro da
//! família que TRANSFORMA (alinhamento, booleana, layout) — esses são componentes do PAI.
//!
//! # A promessa, e por que ela é verdade por CONSTRUÇÃO
//!
//! > **Editar o mestre muda todas as instâncias.**
//!
//! Não há passe de propagação. A instância não guarda cópia nenhuma do mestre: ela guarda o
//! **id** dele, e o desenho é recomposto a cada frame a partir do que o mestre é AGORA. Um passe
//! de propagação seria a segunda resposta a *"o que esta instância mostra?"*, e divergiria no
//! primeiro sítio que editasse o mestre sem o chamar.
//!
//! # O mestre é a sub-árvore, e é isso que faz uma MOLDURA ser um componente
//!
//! O conteúdo do mestre é o caminho-raiz **mais toda a sub-árvore ECS dele**, na ordem de z do
//! documento. Uma moldura (W0) com filhos vira um componente com peças sem uma linha a mais — que
//! é exatamente o que um botão é (uma caixa e um rótulo).
//!
//! # O afim: a cópia herda a FORMA do mestre, não o LUGAR dele
//!
//! Cada peça entra em MUNDO (assada com a pose dela) e é levada para a pose da instância por um
//! **delta** que remove apenas a TRANSLAÇÃO do mestre ([`place_delta`], onde está o porquê e o
//! defeito que isso corrigiu). É isso que preserva a disposição interna — as peças mantêm as
//! posições RELATIVAS que o artista deu ao mestre — **e** faz uma edição na raiz do mestre
//! (redimensionar, girar) alcançar as cópias, que é a promessa da wave.
//!
//! # O mestre que SUMIU não some em silêncio
//!
//! Uma instância cujo mestre foi apagado **não recebe entrada no mapa** — e ausência, aqui,
//! significa *"desenhe a fonte"*: o retângulo-suporte que a instância guarda. O artista vê a
//! moldura vazia da cópia, com o painel a dizer *"main missing"* — o mesmo desenho da binding
//! órfã da timeline, e o oposto de a cópia evaporar sem nada na tela dizer porquê.
//!
//! ⚠️ Isto é a razão de a instância guardar geometria: o suporte é o que a torna **agarrável**
//! (a caixa do gizmo lê a bbox GUARDADA, `vec_gizmo_view::anchor_half`) e o que sobra quando o
//! mestre morre. Uma instância de geometria vazia seria invisível e insetecionável no dia em que
//! o vínculo quebrasse.

use ph2d_ecs::{Entity, OverrideSlot, SimWorld, VecInstance};
use ph2d_vec_render::LiveGeometry;
use ph2d_vec_scene::{Paint, Rgba8, VecPath, VecPathId, VecScene, VecXforms, bake_xform, xform_of};

use crate::vec_entities::VecEntityMap;

/// Teto da caminhada da sub-árvore do mestre — defesa contra hierarquia ciclada, não limite de
/// produto (o mesmo número, e pelo mesmo motivo, que o `layout_live::MAX_DEPTH`).
const MAX_DEPTH: usize = 64;

/// O cozimento vivo de todas as instâncias da cena.
#[derive(Default)]
pub(crate) struct InstanceLive {
    live: LiveGeometry,
    /// Os ids das instâncias cujo mestre não resolve neste frame — o que o painel publica como
    /// *"main missing"*. Ordenado (é uma varredura da cena, que já é ordenada por z).
    orphans: Vec<VecPathId>,
    /// Por instância, as peças do mestre que produziram cada item de [`Self::live`], **na mesma
    /// ordem**. É o que permite ao *Detach* saber QUAL peça é a raiz sem depender da ordem de z.
    pieces: std::collections::BTreeMap<VecPathId, Vec<VecPathId>>,
}

impl InstanceLive {
    /// A geometria derivada deste frame — o que o `dispatch` desenha no lugar do suporte.
    pub(crate) fn live(&self) -> &LiveGeometry {
        &self.live
    }

    /// As instâncias órfãs deste frame (mestre apagado).
    pub(crate) fn orphans(&self) -> &[VecPathId] {
        &self.orphans
    }

    /// As peças do mestre por trás do que esta instância desenhou, na MESMA ordem.
    pub(crate) fn pieces_of(&self, at: VecPathId) -> Option<&[VecPathId]> {
        self.pieces.get(&at).map(Vec::as_slice)
    }

    /// Re-coza todas as instâncias. Chamado uma vez por frame, DEPOIS do `sync` (senão uma forma
    /// recém-criada ainda não teria entidade e o componente dela não seria encontrado).
    pub(crate) fn recook(
        &mut self,
        scene: &VecScene,
        sim: &SimWorld,
        map: &VecEntityMap,
        xforms: &VecXforms,
    ) {
        self.live.clear();
        self.orphans.clear();
        self.pieces.clear();
        // A varredura é sobre a CENA (ordem de z) e não sobre o mundo ECS: é ela que dá a ordem
        // estável de que o `orphans` depende, e é a mesma que o `dispatch` percorre.
        for path in scene.paths() {
            let Some(&bits) = map.get(&path.id) else {
                continue;
            };
            let e = Entity::from_bits(bits);
            let Some(inst) = sim.world().get::<VecInstance>(e).cloned() else {
                continue;
            };
            match cook_one(scene, sim, map, xforms, path.id, &inst) {
                Some((items, pieces)) => {
                    self.live.insert(path.id, items);
                    self.pieces.insert(path.id, pieces);
                }
                None => self.orphans.push(path.id),
            }
        }
    }
}

/// A espécie `Fill` de [`OverrideSlot`] — o mesmo número que `OverrideSlot::kind` devolve.
const FILL_KIND: u8 = 0;
/// A espécie `Hidden`.
const HIDDEN_KIND: u8 = 1;

/// **O que UMA instância desenha** — `None` quando o mestre não resolve (o suporte fica).
///
/// `at` é o caminho da própria instância; quem chama já o tem em mãos, e procurá-lo aqui seria
/// uma busca por conteúdo que confundiria duas instâncias com os mesmos overrides.
pub(crate) fn cook_one(
    scene: &VecScene,
    sim: &SimWorld,
    map: &VecEntityMap,
    xforms: &VecXforms,
    at: VecPathId,
    inst: &VecInstance,
) -> Option<(Vec<VecPath>, Vec<VecPathId>)> {
    let main_id: VecPathId = inst.main;
    // Uma instância de si mesma é um laço; a recusa é aqui, e não numa guarda de profundidade,
    // porque o desenho dela não tem resposta finita nenhuma.
    if main_id == at {
        return None;
    }
    let &main_bits = map.get(&main_id)?;
    let main_e = Entity::from_bits(main_bits);
    // O mestre tem de se declarar mestre: apagar o marcador é a forma de o artista dizer *"isto
    // deixou de ser um componente"*, e uma cópia que o ignorasse estaria a honrar uma decisão que
    // ele revogou.
    sim.world().get::<ph2d_ecs::VecComponentMain>(main_e)?;
    let delta = place_delta(xforms, main_id, at);

    let mut out = Vec::new();
    // ⚠️ As duas listas saem da MESMA travessia: um segundo `visible_pieces` daria a mesma
    // resposta hoje e seria a porta por onde elas passariam a divergir amanhã.
    let mut kept = Vec::new();
    for piece in visible_pieces(scene, sim, map, main_e, inst) {
        let Some(src) = scene.paths().iter().find(|p| p.id == piece) else {
            continue;
        };
        let mut world = src.cooked().into_owned();
        bake_xform(&mut world, &xform_of(xforms, piece));
        bake_xform(&mut world, &delta);
        if let Some(OverrideSlot::Fill(rgba)) = inst.get(piece, FILL_KIND) {
            world.fill = Some(Paint::Solid(Rgba8 {
                r: rgba[0],
                g: rgba[1],
                b: rgba[2],
                a: rgba[3],
            }));
        }
        out.push(world);
        kept.push(piece);
    }
    Some((out, kept))
}

/// **O afim que leva a arte do mestre para a instância: TIRA o LUGAR dele, mantém a FORMA.**
///
/// # Isto foi um defeito reportado, e a diferença entre as duas versões é o que a wave promete
///
/// A v1 compunha `pose_da_cópia ∘ pose_do_mestre⁻¹`, que cancela a pose do mestre **inteira** — e
/// como cada peça é assada com a própria pose de MUNDO (que já contém a do mestre), o resultado é
/// que **nenhuma edição na RAIZ do mestre alcançava as cópias**. Medido: escalar o mestre 3× deixa
/// as duas peças de uma instância byte a byte onde estavam. Era o relato do Enio — *"redimensionei
/// o mestre e só os filhos mudaram"*: uma edição num FILHO mexe no local dele e propaga; uma na
/// raiz mexe na pose dela e evaporava aqui.
///
/// A pergunta certa é *o que, na pose do mestre, é CONTEÚDO?* — e a resposta é: a parte linear
/// (escala, rotação, cisalhamento) descreve a FORMA que o componente tem; a translação descreve
/// **onde ele está na tela**, que é justamente o que a cópia não deve herdar (senão toda instância
/// nasceria em cima do mestre). Então só a translação é removida.
///
/// É o modelo do Figma — redimensionar o *main component* redimensiona as instâncias, movê-lo não
/// as move — e é ele que faz a promessa da wave ser verdadeira para a raiz e não só para os filhos.
///
/// ⚠️ E some com o `inverse()`: um mestre degenerado (escala 0) deixa de ser reportado como
/// **órfão**, o que é mais honesto — ele existe, apenas não desenha nada.
fn place_delta(xforms: &VecXforms, main_id: VecPathId, at: VecPathId) -> ph2d_vec_scene::Xform {
    let m = xform_of(xforms, main_id).0;
    let un_place = ph2d_vec_scene::Xform([1.0, 0.0, 0.0, 1.0, -m[4], -m[5]]);
    un_place.then(&xform_of(xforms, at))
}

/// **As peças que ESTA instância mostra**, na ordem de z do documento — porta única.
///
/// ⚠️ Perguntada por quem DESENHA ([`cook_one`]) e por quem MATERIALIZA (o *Detach*). Uma segunda
/// lista faria o destacar produzir uma árvore que não corresponde ao que estava na tela, e o
/// sintoma seria geometria a mais ou a menos **sem erro nenhum**.
pub(crate) fn visible_pieces(
    scene: &VecScene,
    sim: &SimWorld,
    map: &VecEntityMap,
    main_e: Entity,
    inst: &VecInstance,
) -> Vec<VecPathId> {
    subtree_paths(scene, sim, map, main_e)
        .into_iter()
        // `Hidden`: a peça não aparece nesta instância. Sair FORA do laço de geometria (e não
        // desenhar transparente) é o que faz o override custar zero no desenho.
        .filter(|p| inst.get(*p, HIDDEN_KIND) != Some(OverrideSlot::Hidden))
        .collect()
}

/// Os caminhos do mestre — a raiz e a sub-árvore ECS dela, **na ordem de z do documento**.
///
/// A ordem sai da cena e não da árvore: é ela que decide o que fica por cima, e re-derivá-la de um
/// DFS daria uma segunda resposta a uma pergunta que o documento já responde.
fn subtree_paths(
    scene: &VecScene,
    sim: &SimWorld,
    map: &VecEntityMap,
    main_e: Entity,
) -> Vec<VecPathId> {
    let mut out = Vec::new();
    for p in scene.paths() {
        let Some(&bits) = map.get(&p.id) else {
            continue;
        };
        let e = Entity::from_bits(bits);
        if e == main_e || is_descendant_of(sim, e, main_e) {
            out.push(p.id);
        }
    }
    out
}

/// `e` está sob `root` na hierarquia ECS?
fn is_descendant_of(sim: &SimWorld, e: Entity, root: Entity) -> bool {
    let mut cur = e;
    for _ in 0..MAX_DEPTH {
        let Some(c) = sim.world().get::<ph2d_ecs::ChildOf>(cur) else {
            return false;
        };
        if c.parent() == root {
            return true;
        }
        cur = c.parent();
    }
    false
}

#[cfg(test)]
#[path = "instance_live_tests.rs"]
mod tests;
