//! **O Offset VIVO na shell** — o cozimento e a materialização do [`ph2d_ecs::VecOffset`].
//!
//! O porquê e a história: `docs/Vector Module/50_cabecalhos_do_app_vec.md` § `offset_live.rs`.

use std::collections::BTreeMap;

use ph2d_ecs::{Entity, SimWorld, VecOffset};
use ph2d_vec_render::LiveGeometry;
use ph2d_vec_scene::{VecPath, VecPathId, VecScene, VecXforms, bake_xform, xform_of};

use ph2d_vec_entities::entities::VecEntityMap;

/// Uma entrada do memo: o que ENTROU (a geometria de mundo + os parâmetros) e o que SAIU.
struct Memo {
    spec: VecOffset,
    world: VecPath,
    out: Vec<VecPath>,
}

/// O cozimento vivo de todos os offsets da cena, com memo por caminho.
#[derive(Default)]
pub struct OffsetLive {
    memo: BTreeMap<VecPathId, Memo>,
    live: LiveGeometry,
}

impl OffsetLive {
    /// A geometria derivada deste frame — o que o [`ph2d_vec_render::dispatch`] desenha no lugar
    /// da fonte. Vazia = nenhum offset vivo na cena, e o desenho é o de sempre.
    pub fn live(&self) -> &LiveGeometry {
        &self.live
    }

    /// Re-coze o que mudou. Chamado uma vez por frame, DEPOIS do `sync` (senão uma forma
    /// recém-criada ainda não tem entidade e o componente dela não seria encontrado).
    pub fn recook(
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
                let mut out = ph2d_vec_boolean::offset_path_with(
                    &world,
                    spec.d,
                    crate::expand::join_of_code(spec.join),
                    crate::expand::side_of_code(spec.side),
                    spec.miter,
                );
                // ⭐ `Result: New shape behind` — a fonte desenha-se POR CIMA do offset, no z dela.
                if spec.behind && !out.is_empty() {
                    out.push(world.clone());
                }
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
    pub fn forget(&mut self) {
        self.memo.clear();
        self.live.clear();
    }
}

/// O offset vivo de `id`, se houver. Porta única: o cozimento, o `Apply`, o `Convert to Curves`
/// e o publish para o painel perguntam AQUI.
pub fn spec_of(sim: &SimWorld, map: &VecEntityMap, id: VecPathId) -> Option<VecOffset> {
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
pub fn arm(sim: &mut SimWorld, map: &VecEntityMap, ids: &[VecPathId], spec: VecOffset) -> usize {
    let d = spec.d;
    let want = (d.abs() >= ph2d_vec_boolean::MIN_OFFSET && d.is_finite()).then_some(spec);
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

/// Muda os KNOBS (tudo menos a distância) dos offsets vivos de `ids`.
///
/// Não arma nada: um caminho sem offset vivo continua sem. O clique num chip NÃO é isto sozinho —
/// é o [`choose_knobs`], que arma as formas nuas com a distância do campo.
pub fn retune(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    ids: &[VecPathId],
    knobs: ph2d_panel_vector::OffsetKnobs,
) {
    for id in ids {
        let Some(&bits) = map.get(id) else { continue };
        let e = Entity::from_bits(bits);
        let Some(cur) = sim.world().get::<VecOffset>(e).copied() else {
            continue;
        };
        let want = crate::expand::spec_with(knobs, cur.d);
        if cur == want {
            continue;
        }
        if let Ok(mut em) = sim.world_mut().get_entity_mut(e) {
            em.insert(want);
        }
    }
}

/// **O clique num chip de Corner/Side** — uma PRÉ-VISUALIZAÇÃO, nunca uma preparação muda
/// (*«os botões Miter, Round e Bevel são previsualizações em tempo real»*, Enio, 2026-07-21).
///
/// Uma forma com offset vivo troca só a quina e o lado ([`retune`]: o `d` é DELA). Uma forma sem
/// offset é armada com `d`, o valor que o slider MOSTRA — o painel já o exibe, e um clique que não
/// o desenhasse era um controlo morto (smoke de 11/10). Slider no zero ⇒ nada (a lei do [`arm`]).
///
/// Devolve quantas formas foram ARMADAS.
pub fn choose_knobs(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    ids: &[VecPathId],
    knobs: ph2d_panel_vector::OffsetKnobs,
    d: f64,
) -> usize {
    retune(sim, map, ids, knobs);
    let bare: Vec<VecPathId> = ids
        .iter()
        .copied()
        .filter(|id| spec_of(sim, map, *id).is_none())
        .collect();
    arm(sim, map, &bare, crate::expand::spec_with(knobs, d))
}

/// **O `Distance (px)` diz a verdade da seleção** (smoke de 11/10: *«o slider mostra 25 mas o efeito
/// não aparece na estrela»*): a distância do primeiro offset vivo de `ids`, ou ZERO sem nenhum.
#[must_use]
pub fn shown_d(sim: &SimWorld, map: &VecEntityMap, ids: &[VecPathId]) -> f64 {
    ids.iter()
        .find_map(|id| spec_of(sim, map, *id))
        .map_or(0.0, |o| o.d)
}

/// A distância que um chip arma numa forma nua: a do campo, ou — com ele no zero, que é o que uma
/// forma nua mostra ([`shown_d`]) — a de NASCENÇA, um quarto da meia-dimensão da seleção (`scale`,
/// a de [`crate::expand::offset_scale`]). Um chip que não desenhasse era o BUGS #41.
#[must_use]
pub fn chip_d(field: f64, scale: f64) -> f64 {
    if field == 0.0 {
        ph2d_tool_vector::params::OFFSET_DEFAULT_FRAC * scale
    } else {
        field
    }
}

/// **MATERIALIZA** os offsets vivos da seleção — o `Apply Offset` e o `Convert to Curves`.
///
/// É o único momento em que os vértices do offset passam a existir no documento. Cada forma é
/// assada com o `VecOffset` DELA (não com o slider), pela porta única
/// [`crate::expand::expand_selection`] — a mesma por onde entra o caminho numérico.
///
/// O componente não precisa ser removido: a forma-fonte SAI da cena (remove+insere), e o
/// `vec_entities::sync` do frame seguinte despawna a entidade dela com o componente dentro. A
/// exceção é a forma cujo offset a ANIQUILA (nada a materializar): ela fica, e o offset fica
/// vivo com ela — apagar a arte do artista num clique de "Apply" seria a pior resposta possível
/// a *"não há nada aqui"*.
///
/// `false` = não havia offset vivo nenhum na seleção (e quem chamou segue pelo caminho de
/// sempre). O gesto inteiro é UMA chamada, e por isso UM passo da fila global de undo (que regista
/// o quadro por diff).
pub fn materialise(
    scene: &mut VecScene,
    sim: &mut SimWorld,
    pen: &mut ph2d_vec_edit::PenTool,
    map: &VecEntityMap,
    xforms: &VecXforms,
    ids: &[VecPathId],
    restack: &mut Vec<Vec<VecPathId>>,
) -> bool {
    let live: Vec<(VecPathId, VecOffset)> = ids
        .iter()
        .filter_map(|id| spec_of(sim, map, *id).map(|spec| (*id, spec)))
        .collect();
    if live.is_empty() {
        return false;
    }
    crate::expand::materialise_selection(scene, pen, xforms, ids, |id, local, xf| {
        let Some((_, spec)) = live.iter().find(|(i, _)| *i == id && !o_behind(&live, *i)) else {
            return Vec::new(); // fora do comando: fica onde está, e segue selecionado
        };
        // A distância do offset é de MUNDO, então a pose entra ANTES do motor.
        let mut world = local;
        ph2d_vec_scene::bake_xform(&mut world, xf);
        crate::expand::expand_layers(
            &world,
            crate::expand::Expand::Offset {
                join: crate::expand::join_of_code(spec.join),
                side: crate::expand::side_of_code(spec.side),
                miter: spec.miter,
            },
            spec.d,
        )
    });
    // ⭐ `New shape behind`: a fonte FICA (o mesmo id, a mesma pose, os mesmos componentes) e o
    // offset entra logo ATRÁS dela; a fonte perde o offset vivo, e a forma nova fica selecionada.
    let mut novas = Vec::new();
    for (id, spec) in live.iter().filter(|(_, o)| o.behind) {
        let Some(z) = scene.paths().iter().position(|p| p.id == *id) else {
            continue;
        };
        let src = &scene.paths()[z];
        let mut world = src.cooked().into_owned();
        ph2d_vec_scene::bake_xform(&mut world, &xform_of(xforms, *id));
        let pecas = crate::expand::expand_layers(
            &world,
            crate::expand::Expand::Offset {
                join: crate::expand::join_of_code(spec.join),
                side: crate::expand::side_of_code(spec.side),
                miter: spec.miter,
            },
            spec.d,
        );
        if pecas.is_empty() {
            continue; // aniquilou: nada a pôr atrás, e o offset fica vivo (a lei de cima)
        }
        let antes = novas.len();
        for (at, p) in (z..).zip(pecas) {
            novas.push(scene.insert_path(at, p));
        }
        // ⚠️ A ordem de desenho é a da HIERARCHY, e a forma nova só ganha entidade no `sync` do
        // quadro seguinte (entraria no TOPO): o `restack` põe-na logo atrás da fonte, lá.
        restack.push(novas[antes..].iter().copied().chain([*id]).collect());
        arm(sim, map, &[*id], VecOffset { d: 0.0, ..*spec });
    }
    if !novas.is_empty() {
        pen.select_many(&novas);
    }
    eprintln!("[ph2d-vec] offset materializado: {} forma(s)", live.len());
    true
}

fn o_behind(live: &[(VecPathId, VecOffset)], id: VecPathId) -> bool {
    live.iter().any(|(i, o)| *i == id && o.behind)
}

#[cfg(test)]
#[path = "offset_live_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "offset_live_behind_tests.rs"]
mod behind_tests;
