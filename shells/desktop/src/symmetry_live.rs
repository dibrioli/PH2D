//! **A SIMETRIA VIVA na shell** — o cozimento das cópias do [`ph2d_ecs::VecSymmetry`].
//!
//! Espelho do [`crate::offset_live`] / [`crate::contour_live`]: o componente guarda a relação (que
//! espelho, onde, quantas cópias) e a aparência é uma **função pura** dela, re-cozida aqui e
//! desenhada por [`ph2d_vec_render::dispatch`] **no z da forma**.
//!
//! # A promessa que este módulo existe para cumprir
//!
//! *"Se a simetria for desmarcada antes do apply, as cópias somem mas não são destruídas"* (Enio,
//! 2026-08-01). Aqui isso não é uma regra a honrar — é o que **acontece**: as cópias nunca
//! estiveram no documento. Desarmar remove o componente, a `LiveGeometry` deixa de ter a entrada,
//! e o `dispatch` volta a desenhar a fonte. Não há estado a limpar porque não houve estado.
//!
//! # O cozimento é LOCAL, e a ORDEM é a espinha
//!
//! O eixo vive no espaço da geometria autorada (ver o cabeçalho do componente), então a reflexão
//! corre **antes** de a pose entrar: `reflectir → assar a pose`. A ordem inversa só daria o mesmo
//! resultado se a pose fosse uma semelhança — sob escala não-uniforme ou skew as duas divergem, e
//! a que corresponde ao que o artista vê é esta (o espelho é uma propriedade da FORMA, e a pose
//! leva a forma inteira, cópias incluídas).
//!
//! ⚠️ É por isso que este módulo **não** pode copiar o `offset_live` verbatim: lá a distância é de
//! MUNDO, então ele assa primeiro e offseta depois. Aqui é o contrário, e a chave do memo tem de
//! carregar a pose junto (o resultado depende dela).
//!
//! # Quem lê a DERIVADA e quem lê a FONTE
//!
//! Lêem a derivada os que respondem *"o que está na tela aqui?"* — o [`ph2d_vec_render::dispatch`]
//! e o hit-test de canvas. Lêem a FONTE os que respondem *"o que o artista escreveu?"*: o modo
//! **Node** (as âncoras que ele arrasta são as autoradas — a mesma lei do raio de quina vivo,
//! ADR-0121) e a **caixa do gizmo**. Nenhum dos dois é esquecimento.
//!
//! # O memo
//!
//! O cozimento roda por frame, então sem memo uma cena parada pagaria a reflexão para sempre. A
//! chave é o que de facto determina o resultado — a geometria LOCAL que entra, a pose, e a spec —
//! e não um contador de versão que alguém esqueceria de bumpar.

use std::collections::BTreeMap;

use ph2d_ecs::{Entity, SimWorld, VecSymmetry};
use ph2d_vec_render::LiveGeometry;
use ph2d_vec_scene::symmetry::{SymmetrySpec, symmetry_paths};
use ph2d_vec_scene::{VecPath, VecPathId, VecScene, VecXforms, Xform, bake_xform, xform_of};

use crate::vec_entities::VecEntityMap;

/// Uma entrada do memo: o que ENTROU (a geometria local + a pose + a spec) e o que SAIU.
struct Memo {
    spec: SymmetrySpec,
    xform: Xform,
    local: VecPath,
    out: Vec<VecPath>,
}

/// O cozimento vivo de todas as simetrias da cena, com memo por caminho.
#[derive(Default)]
pub(crate) struct SymmetryLive {
    memo: BTreeMap<VecPathId, Memo>,
    live: LiveGeometry,
}

impl SymmetryLive {
    /// A geometria derivada deste frame — o que o [`ph2d_vec_render::dispatch`] desenha no lugar
    /// da fonte. Vazia = nenhuma simetria viva na cena, e o desenho é o de sempre.
    pub(crate) fn live(&self) -> &LiveGeometry {
        &self.live
    }

    /// Re-coze o que mudou. Chamado uma vez por frame, DEPOIS do `sync` (senão uma forma
    /// recém-criada ainda não tem entidade e o componente dela não seria encontrado).
    pub(crate) fn recook(
        &mut self,
        scene: &VecScene,
        sim: &SimWorld,
        map: &VecEntityMap,
        xforms: &VecXforms,
    ) {
        self.live.clear();
        for path in scene.paths() {
            let Some(spec) = spec_of(sim, map, path.id) else {
                continue;
            };
            // A fonte é a curva COZIDA (raio de quina + pilha de efeitos já correram) mas ainda em
            // coordenadas LOCAIS — é nelas que o eixo autorado vive.
            let local = path.cooked().into_owned();
            let xform = xform_of(xforms, path.id);
            let hit = self
                .memo
                .get(&path.id)
                .is_some_and(|m| m.spec == spec && m.xform == xform && m.local == local);
            if !hit {
                let mut out = symmetry_paths(&local, &spec);
                for p in &mut out {
                    bake_xform(p, &xform);
                }
                self.memo.insert(
                    path.id,
                    Memo {
                        spec,
                        xform,
                        local: local.clone(),
                        out,
                    },
                );
            }
            if let Some(m) = self.memo.get(&path.id) {
                self.live.insert(path.id, m.out.clone());
            }
        }
        // ⚠️ Isto é HIGIENE, e não correcção — a distinção importa para quem vier depois. A chave
        // carrega tudo o que determina a resposta (spec + pose + geometria local), então uma
        // entrada velha só acerta quando as três coincidem, e aí ela está CERTA. O que o `retain`
        // impede é o memo crescer com formas que já não têm simetria. Não há gate porque não há
        // falha a produzir; se a chave um dia deixar de ser completa, esta linha passa a ser
        // correcção e o gate nasce com ela.
        self.memo.retain(|id, _| self.live.contains_key(id));
    }

    /// Esquece tudo — o load de projeto e o restore de undo trocam a cena inteira debaixo do memo,
    /// e os `VecPathId` são reciclados entre documentos.
    pub(crate) fn forget(&mut self) {
        self.memo.clear();
        self.live.clear();
    }
}

/// A simetria viva de `id`, se houver. Porta única: o cozimento, o **Apply**, o overlay das linhas
/// e o publish para o painel perguntam AQUI.
pub(crate) fn spec_of(sim: &SimWorld, map: &VecEntityMap, id: VecPathId) -> Option<SymmetrySpec> {
    let &bits = map.get(&id)?;
    sim.world()
        .get::<VecSymmetry>(Entity::from_bits(bits))
        .map(|s| s.spec)
}

/// **Arma** (ou desarma) a simetria viva de cada caminho de `ids`.
///
/// `spec = None` **REMOVE** o componente em vez de guardar um espelho inerte — é a lei do
/// `VecOffset` com `d = 0`, e é ela que faz *"desmarcar antes do Apply"* devolver a forma ao
/// estado limpo em vez de a deixar com uma relação invisível pendurada.
///
/// Devolve quantas entidades mudaram.
pub(crate) fn arm(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    ids: &[VecPathId],
    spec: Option<SymmetrySpec>,
) -> usize {
    let want = spec.map(VecSymmetry::new);
    let mut n = 0;
    for id in ids {
        let Some(&bits) = map.get(id) else { continue };
        let e = Entity::from_bits(bits);
        let cur = sim.world().get::<VecSymmetry>(e).copied();
        if cur == want {
            continue;
        }
        let Ok(mut em) = sim.world_mut().get_entity_mut(e) else {
            continue;
        };
        match want {
            Some(v) => {
                em.insert(v);
            }
            None => {
                em.remove::<VecSymmetry>();
            }
        }
        n += 1;
    }
    n
}

/// **MATERIALIZA** as cópias da seleção — o botão *Apply*.
///
/// *"Botão apply para consolidar a forma e desativar a simetria"* (Enio, 2026-08-01). É o único
/// momento em que as cópias passam a existir no documento; até aqui elas eram desenho, e é isso
/// que faz o desarmar não destruir nada.
///
/// ⚠️ **Uma porta, dois consumidores:** o `layers_for` daqui produz a MESMA lista que o
/// [`SymmetryLive::recook`] cozinha — `symmetry_paths` sobre a curva local, e só então a pose. Se
/// o Apply tivesse a sua própria produção, a forma SALTARIA no clique, que é o defeito que o
/// ADR-0128 pagou cinco vezes.
///
/// ⚠️ **O componente não precisa de ser removido:** a forma-fonte SAI da cena (remove+insere) e o
/// `vec_entities::sync` do frame seguinte despawna a entidade dela com o componente dentro. Quem
/// não produziu nada fica — e fica COM a simetria, porque apagar a arte do artista num clique de
/// *Apply* seria a pior resposta possível a *"não há nada aqui"*.
///
/// `false` = não havia simetria viva na seleção. **UM passo de undo** para o gesto inteiro.
pub(crate) fn materialise(
    scene: &mut VecScene,
    sim: &SimWorld,
    pen: &mut ph2d_vec_edit::PenTool,
    history: &mut ph2d_vec_edit::History,
    map: &VecEntityMap,
    xforms: &VecXforms,
    ids: &[VecPathId],
) -> bool {
    let live: Vec<(VecPathId, SymmetrySpec)> = ids
        .iter()
        .filter_map(|id| spec_of(sim, map, *id).map(|s| (*id, s)))
        .collect();
    if live.is_empty() {
        return false;
    }
    let pre = scene.clone();
    let touched =
        crate::vec_expand::materialise_selection(scene, pen, xforms, ids, |id, local, xf| {
            let Some((_, spec)) = live.iter().find(|(i, _)| *i == id) else {
                return Vec::new();
            };
            // A MESMA ordem do cozimento: reflectir no espaço da forma, e só então assar a pose.
            let mut out = symmetry_paths(&local, spec);
            for p in &mut out {
                bake_xform(p, xf);
            }
            out
        });
    if touched {
        history.push_undo(pre);
    }
    eprintln!("[ph2d-vec] simetria materializada: {} forma(s)", live.len());
    true
}

/// **Os eixos vivos da cena, em MUNDO** — o que o overlay desenha.
///
/// ⚠️ A direcção sai de [`SymmetrySpec::mirror_dir`], a MESMA porta que o kernel usa para
/// reflectir, e só depois sobe pela pose (`apply_vec`, não `apply`: levar uma direção como ponto a
/// transladaria). Uma segunda derivação aqui desenharia um eixo onde a geometria não espelha — e
/// ninguém lê um número numa screenshot, então a divergência apareceria como *"a linha está
/// torta"*, meses depois.
pub(crate) fn live_axes(
    scene: &VecScene,
    sim: &SimWorld,
    map: &VecEntityMap,
    xforms: &VecXforms,
) -> Vec<ph2d_vec_render::SymmetryAxis> {
    use ph2d_vec_scene::symmetry::SymmetryKind;
    scene
        .paths()
        .iter()
        .filter_map(|p| spec_of(sim, map, p.id).map(|s| (p.id, s)))
        .map(|(id, spec)| {
            let xf = xform_of(xforms, id);
            ph2d_vec_render::SymmetryAxis {
                at: xf.apply(spec.center),
                dir: xf.apply_vec(spec.mirror_dir()),
                segments: (spec.kind == SymmetryKind::Radial).then(|| spec.segments()),
            }
        })
        .collect()
}

#[cfg(test)]
#[path = "symmetry_live_tests.rs"]
mod tests;
