//! **A SIMETRIA VIVA na shell** — o cozimento das cópias do [`ph2d_ecs::VecSymmetry`].
//!
//! O porquê e a história: `docs/Vector Module/50_cabecalhos_do_app_vec.md` § `symmetry_live.rs`.

use std::collections::BTreeMap;

use ph2d_ecs::{Entity, SimWorld, VecSymmetry};
use ph2d_vec_render::LiveGeometry;
use ph2d_vec_scene::symmetry::{SymmetrySpec, symmetry_paths};
use ph2d_vec_scene::{VecPath, VecPathId, VecScene, VecXforms, Xform, bake_xform, xform_of};

use ph2d_vec_entities::entities::VecEntityMap;

/// Uma entrada do memo: o que ENTROU (a geometria local + a pose + a spec) e o que SAIU.
struct Memo {
    spec: SymmetrySpec,
    xform: Xform,
    local: VecPath,
    out: Vec<VecPath>,
}

/// A direcção do eixo de SESSÃO, em mundo, antes de o `kind` opinar.
///
/// Hoje é sempre esta: não existe gesto para o artista desenhar a linha (item aberto, nomeado no
/// handoff), então `Custom` cai na vertical — o que a [`SymmetrySpec::mirror_dir`] já faz para uma
/// direcção degenerada. Os espelhos X/Y ignoram-na por construção.
const SESSION_DIR: [f64; 2] = [0.0, 1.0];

/// O cozimento vivo de todas as simetrias da cena, com memo por caminho.
#[derive(Default)]
pub struct SymmetryLive {
    memo: BTreeMap<VecPathId, Memo>,
    live: LiveGeometry,
    /// Quem estava em gesto no frame ANTERIOR — o frame em que o pivô assenta. Ver o cabeçalho:
    /// sem isto a captura fica deslocada pela translação do `settle_origins`, em silêncio.
    prev_drawing: Vec<VecPathId>,
}

impl SymmetryLive {
    /// A geometria derivada deste frame — o que o [`ph2d_vec_render::dispatch`] desenha no lugar
    /// da fonte. Vazia = nenhuma simetria viva na cena, e o desenho é o de sempre.
    pub fn live(&self) -> &LiveGeometry {
        &self.live
    }

    /// **Adopta** o que está a ser desenhado e re-afina o ESTILO do que já está armado.
    ///
    /// `origin` é o eixo de sessão em MUNDO. Duas metades, e a divisão é a lei do modelo:
    ///
    /// - o que está **em gesto** (e o que acabou de sair dele) tem o eixo re-derivado do de
    ///   sessão, porque a captura ainda não selou (ver o cabeçalho: o pivô assenta um frame
    ///   depois);
    /// - o que **já está armado** mantém o LUGAR dele e recebe só o estilo — é isto que faz
    ///   arrastar *Segments* actualizar a sessão inteira sem teleportar eixo nenhum.
    ///
    /// ⚠️ Uma forma que não tem o componente e não está em gesto **nunca** é tocada: é essa
    /// ausência que cumpre *"não deve fazer simetria de formas que já existem previamente"*.
    ///
    /// Devolve quantas formas estão armadas — o que o painel usa para oferecer o *Apply*.
    // Quatro dos oito argumentos são o MUNDO (`sim`/`map`/`scene`/`xforms`), a mesma forma
    // que o `recook` irmão já tem; agrupá-los num tipo só para calar a lint criaria uma
    // struct cujo único trabalho é existir. Precedente: `physics::body_desc`.
    #[allow(clippy::too_many_arguments)]
    pub fn adopt(
        &mut self,
        sim: &mut SimWorld,
        map: &VecEntityMap,
        scene: &VecScene,
        xforms: &VecXforms,
        style: ph2d_vec_scene::symmetry::SymmetryStyle,
        origin: [f64; 2],
        drawing: &[VecPathId],
    ) -> usize {
        for id in drawing.iter().chain(self.prev_drawing.iter()) {
            // O eixo de sessão é de MUNDO; o componente guarda-o no LOCAL da forma. Uma pose
            // não-invertível (escala zero) não tem local nenhum — fica com o mundo, que é o que
            // ela desenha de qualquer maneira.
            let (center, dir) = match ph2d_vec_scene::xform_of(xforms, *id).inverse() {
                Some(inv) => (inv.apply(origin), inv.apply_vec(SESSION_DIR)),
                None => (origin, SESSION_DIR),
            };
            let spec = SymmetrySpec::from_style(style, center, dir);
            arm(sim, map, std::slice::from_ref(id), Some(spec));
        }
        self.prev_drawing.clear();
        self.prev_drawing.extend_from_slice(drawing);

        let mut live = 0;
        for path in scene.paths() {
            let Some(cur) = spec_of(sim, map, path.id) else {
                continue;
            };
            live += 1;
            let want = SymmetrySpec::from_style(style, cur.center, cur.dir);
            arm(sim, map, std::slice::from_ref(&path.id), Some(want));
        }
        live
    }

    /// Re-coze o que mudou. Chamado uma vez por frame, DEPOIS do `sync` (senão uma forma
    /// recém-criada ainda não tem entidade e o componente dela não seria encontrado).
    ///
    /// ⚠️ `on` gateia o COZIMENTO e não o componente — é o que faz desarmar esconder as cópias
    /// sem as destruir, e religar trazê-las de volta inteiras. O memo sobrevive de propósito: ele
    /// é chaveado por tudo o que determina a resposta, então religar não paga nada.
    pub fn recook(
        &mut self,
        scene: &VecScene,
        sim: &SimWorld,
        map: &VecEntityMap,
        xforms: &VecXforms,
        on: bool,
    ) {
        self.live.clear();
        if !on {
            return;
        }
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
    pub fn forget(&mut self) {
        self.memo.clear();
        self.live.clear();
        self.prev_drawing.clear();
    }
}

/// **Toda forma armada da cena** — o alvo do *Apply*.
///
/// ⚠️ A selecção não entra nisto, e a razão é o modelo: a simetria é um MODO, então *"consolidar a
/// forma e desativar a simetria"* vale para o que o modo produziu, não para o que estiver
/// seleccionado no momento do clique. Consolidar só o seleccionado deixaria metade das cópias na
/// tela com o modo já desligado — cópias que o cozimento gateado deixaria de desenhar, e o artista
/// veria o trabalho evaporar.
pub fn armed_paths(sim: &SimWorld, map: &VecEntityMap, scene: &VecScene) -> Vec<VecPathId> {
    scene
        .paths()
        .iter()
        .filter(|p| spec_of(sim, map, p.id).is_some())
        .map(|p| p.id)
        .collect()
}

/// **A linha de SESSÃO** — a que aparece no instante em que o botão liga, com a cena vazia.
///
/// ⚠️ A direcção sai da MESMA [`SymmetrySpec::mirror_dir`] que o kernel usa para reflectir: a
/// linha de sessão e o eixo que a próxima forma vai capturar são o mesmo fato, e derivá-lo duas
/// vezes desenharia a promessa num sítio e cumpri-la-ia noutro.
pub fn session_axis(
    style: ph2d_vec_scene::symmetry::SymmetryStyle,
    origin: [f64; 2],
) -> ph2d_vec_render::SymmetryAxis {
    use ph2d_vec_scene::symmetry::SymmetryKind;
    let spec = SymmetrySpec::from_style(style, origin, SESSION_DIR);
    ph2d_vec_render::SymmetryAxis {
        at: origin,
        dir: spec.mirror_dir(),
        segments: (spec.kind == SymmetryKind::Radial).then(|| spec.segments()),
    }
}

/// A simetria viva de `id`, se houver. Porta única: o cozimento, o **Apply**, o overlay das linhas
/// e o publish para o painel perguntam AQUI.
pub fn spec_of(sim: &SimWorld, map: &VecEntityMap, id: VecPathId) -> Option<SymmetrySpec> {
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
pub fn arm(
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
/// `false` = não havia simetria viva na seleção. O gesto inteiro é UMA chamada, e por isso UM passo
/// da fila global de undo (que regista o quadro por diff).
pub fn materialise(
    scene: &mut VecScene,
    sim: &SimWorld,
    pen: &mut ph2d_vec_edit::PenTool,
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
    crate::expand::materialise_selection(scene, pen, xforms, ids, |id, local, xf| {
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
pub fn live_axes(
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
