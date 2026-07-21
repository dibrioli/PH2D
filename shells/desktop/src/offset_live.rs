//! **O Offset VIVO na shell** — o cozimento e a materialização do [`ph2d_ecs::VecOffset`].
//!
//! Espelho do [`crate::blend_live`] / [`crate::connector_live`]: o componente guarda a relação
//! (quanto, com que quina, de que lado) e a aparência é uma **função pura** dela, re-cozida aqui
//! e desenhada por [`ph2d_vec_render::dispatch`] **no z da forma**.
//!
//! # A promessa que este módulo existe para cumprir
//!
//! *"os botões Miter, Round e Bevel são previsualizações em tempo real dos efeitos, mas para
//! consolidar a curva deve-se apertar Apply Offset ou Convert to Curves"* (Enio, 2026-07-21).
//!
//! Antes disto o preview **substituía** a forma no documento: o resultado era um objeto com id
//! próprio, e o modo Node mostrava as ~238 âncoras dos arcos do Round. O artista lia, com razão,
//! *"a curva já tem todos os vertex novos criados antes de apertar apply"*. Agora o documento
//! **nunca é tocado** enquanto o offset é preview: o `VecPath` continua sendo a curva autorada, e
//! o resultado é DESENHO.
//!
//! # A ordem do cozimento: quina → pilha de efeitos → offset
//!
//! A fonte que entra aqui é o [`ph2d_vec_scene::VecPath::cooked`] — ou seja, o raio de quina
//! (estágio zero, ADR-0121) e a pilha de Live Path Effects (ADR-0132) correm ANTES. É a única
//! ordem que se sustenta: o offset precisa de uma REGIÃO regularizada, e tanto a quina como os
//! efeitos produzem geometria plana que ele pode consumir. O inverso (offsetar e depois ondular)
//! continua alcançável pelo caminho destrutivo — Apply Offset e então o efeito.
//!
//! # O memo não é otimização prematura: é MEDIDO
//!
//! `offset_path` custa, em release, **0,40–0,86 ms** no donut e **0,55–1,07 ms** na estrela com
//! quina Round (Miter/Bevel ficam em 0,1–0,35). O cozimento roda por frame, então sem memo uma
//! cena parada pagaria isso para sempre. A chave é o que de facto o determina — a geometria de
//! MUNDO que entra e os três parâmetros — e não um contador de versão que alguém esqueceria de
//! bumpar.

use std::collections::BTreeMap;

use ph2d_ecs::{Entity, SimWorld, VecOffset};
use ph2d_vec_render::LiveGeometry;
use ph2d_vec_scene::{VecPath, VecPathId, VecScene, VecXforms, bake_xform, xform_of};

use crate::vec_entities::VecEntityMap;

/// Uma entrada do memo: o que ENTROU (a geometria de mundo + os parâmetros) e o que SAIU.
struct Memo {
    spec: VecOffset,
    world: VecPath,
    out: Vec<VecPath>,
}

/// O cozimento vivo de todos os offsets da cena, com memo por caminho.
#[derive(Default)]
pub(crate) struct OffsetLive {
    memo: BTreeMap<VecPathId, Memo>,
    live: LiveGeometry,
}

impl OffsetLive {
    /// A geometria derivada deste frame — o que o [`ph2d_vec_render::dispatch`] desenha no lugar
    /// da fonte. Vazia = nenhum offset vivo na cena, e o desenho é o de sempre.
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
            // A pose viaja DENTRO da geometria: o resultado do offset é de mundo, e o
            // `dispatch` o sobe pela câmera. Assar aqui é o que permite ao offset atravessar
            // uma pose com rotação/escala sem que ninguém a aplique duas vezes.
            let mut world = path.cooked().into_owned();
            bake_xform(&mut world, &xform_of(xforms, path.id));
            let hit = self
                .memo
                .get(&path.id)
                .is_some_and(|m| m.spec == spec && m.world == world);
            if !hit {
                let out = ph2d_vec_boolean::offset_path(
                    &world,
                    spec.d,
                    crate::vec_expand::join_of_code(spec.join),
                    crate::vec_expand::side_of_code(spec.side),
                );
                self.memo.insert(
                    path.id,
                    Memo {
                        spec,
                        world: world.clone(),
                        out,
                    },
                );
            }
            // ⚠️ A entrada é criada MESMO vazia: vazio é a ANIQUILAÇÃO (o offset comeu a
            // forma), e desenhar nada é a resposta certa. Ausente significaria "desenhe a
            // fonte", e a forma reapareceria inteira no extremo do slider.
            if let Some(m) = self.memo.get(&path.id) {
                self.live.insert(path.id, m.out.clone());
            }
        }
        // O memo não pode sobreviver ao componente: uma forma que perdeu o offset (Apply,
        // Ctrl+Z) manteria a resposta velha e re-cozinharia o mundo errado se ele voltasse.
        self.memo.retain(|id, _| self.live.contains_key(id));
    }

    /// Esquece tudo — o load de projeto e o restore de undo trocam a cena inteira debaixo do
    /// memo, e os `VecPathId` são reciclados entre documentos.
    pub(crate) fn forget(&mut self) {
        self.memo.clear();
        self.live.clear();
    }
}

/// O offset vivo de `id`, se houver. Porta única: o cozimento, o `Apply`, o `Convert to Curves`
/// e o publish para o painel perguntam AQUI.
pub(crate) fn spec_of(sim: &SimWorld, map: &VecEntityMap, id: VecPathId) -> Option<VecOffset> {
    let &bits = map.get(&id)?;
    sim.world()
        .get::<VecOffset>(Entity::from_bits(bits))
        .copied()
}

/// **Arma** (ou desarma) o offset vivo de cada caminho de `ids`.
///
/// `d` abaixo de [`ph2d_vec_boolean::MIN_OFFSET`] **REMOVE** o componente em vez de guardar um
/// offset inerte — um documento não deve acumular relações que não desenham nada, e é isso que
/// faz "arrastar o slider de volta ao zero" devolver a forma ao estado limpo em vez de deixá-la
/// com um efeito invisível pendurado.
///
/// Devolve quantas entidades mudaram.
pub(crate) fn arm(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    ids: &[VecPathId],
    d: f64,
    join: u8,
    side: u8,
) -> usize {
    let want = (d.abs() >= ph2d_vec_boolean::MIN_OFFSET && d.is_finite())
        .then(|| VecOffset::new(d, join, side));
    let mut n = 0;
    for id in ids {
        let Some(&bits) = map.get(id) else { continue };
        let e = Entity::from_bits(bits);
        let cur = sim.world().get::<VecOffset>(e).copied();
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
                em.remove::<VecOffset>();
            }
        }
        n += 1;
    }
    n
}

/// Muda só a QUINA e o LADO dos offsets vivos de `ids` — o clique num chip de Corner/Side.
///
/// Não arma nada: um caminho sem offset vivo continua sem. É o que faz clicar "Round" sem ter
/// arrastado o slider não inventar um offset de lugar nenhum.
pub(crate) fn retune(sim: &mut SimWorld, map: &VecEntityMap, ids: &[VecPathId], knobs: (u8, u8)) {
    for id in ids {
        let Some(&bits) = map.get(id) else { continue };
        let e = Entity::from_bits(bits);
        let Some(cur) = sim.world().get::<VecOffset>(e).copied() else {
            continue;
        };
        if (cur.join, cur.side) == knobs {
            continue;
        }
        if let Ok(mut em) = sim.world_mut().get_entity_mut(e) {
            em.insert(VecOffset {
                join: knobs.0,
                side: knobs.1,
                ..cur
            });
        }
    }
}

/// **MATERIALIZA** os offsets vivos da seleção — o `Apply Offset` e o `Convert to Curves`.
///
/// É o único momento em que os vértices do offset passam a existir no documento. Cada forma é
/// assada com o `VecOffset` DELA (não com o slider), pela porta única
/// [`crate::vec_expand::expand_selection`] — a mesma por onde entra o caminho numérico.
///
/// O componente não precisa ser removido: a forma-fonte SAI da cena (remove+insere), e o
/// `vec_entities::sync` do frame seguinte despawna a entidade dela com o componente dentro. A
/// exceção é a forma cujo offset a ANIQUILA (nada a materializar): ela fica, e o offset fica
/// vivo com ela — apagar a arte do artista num clique de "Apply" seria a pior resposta possível
/// a *"não há nada aqui"*.
///
/// `false` = não havia offset vivo nenhum na seleção (e quem chamou segue pelo caminho de
/// sempre). **UM passo de undo** para o gesto inteiro.
pub(crate) fn materialise(
    scene: &mut VecScene,
    sim: &SimWorld,
    pen: &mut ph2d_vec_edit::PenTool,
    history: &mut ph2d_vec_edit::History,
    map: &VecEntityMap,
    xforms: &VecXforms,
    ids: &[VecPathId],
) -> bool {
    let live: Vec<(VecPathId, VecOffset)> = ids
        .iter()
        .filter_map(|id| spec_of(sim, map, *id).map(|spec| (*id, spec)))
        .collect();
    if live.is_empty() {
        return false;
    }
    let pre = scene.clone();
    let touched = crate::vec_expand::expand_selection(scene, pen, xforms, ids, |id| {
        live.iter().find(|(i, _)| *i == id).map(|(_, spec)| {
            (
                crate::vec_expand::Expand::Offset {
                    join: crate::vec_expand::join_of_code(spec.join),
                    side: crate::vec_expand::side_of_code(spec.side),
                },
                spec.d,
            )
        })
    });
    if touched {
        history.push_undo(pre);
    }
    eprintln!("[ph2d-vec] offset materializado: {} forma(s)", live.len());
    true
}

#[cfg(test)]
#[path = "offset_live_tests.rs"]
mod tests;
