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
//! # Quem lê a DERIVADA e quem lê a FONTE
//!
//! Lêem a derivada os que respondem *"o que está na tela aqui?"* — o
//! [`ph2d_vec_render::dispatch`] e, desde 2026-07-21, o **hit-test de canvas** e o **marquee**
//! ([`crate::vec_gizmo_view`]): clicar no que se vê é a definição de apontar, e enquanto o pick
//! apalpava a curva autorada a forma crescida era pintada num lugar e clicável noutro.
//!
//! Lêem a FONTE os que respondem *"o que o artista escreveu?"*: o modo **Node** (as âncoras que
//! ele arrasta são as autoradas — é a mesma lei do raio de quina vivo, ADR-0121) e a **caixa do
//! gizmo** (o porquê está no cabeçalho do `vec_gizmo_view`). Nenhum dos dois é esquecimento.
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
                let out = ph2d_vec_boolean::offset_path(
                    &world,
                    spec.d,
                    crate::expand::join_of_code(spec.join),
                    crate::expand::side_of_code(spec.side),
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
pub fn arm(
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

/// Muda só a QUINA e o LADO dos offsets vivos de `ids`.
///
/// Não arma nada: um caminho sem offset vivo continua sem. O clique num chip NÃO é isto sozinho —
/// é o [`choose_knobs`], que arma as formas nuas com o valor do slider.
pub fn retune(sim: &mut SimWorld, map: &VecEntityMap, ids: &[VecPathId], knobs: (u8, u8)) {
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
    knobs: (u8, u8),
    d: f64,
) -> usize {
    retune(sim, map, ids, knobs);
    let bare: Vec<VecPathId> = ids
        .iter()
        .copied()
        .filter(|id| spec_of(sim, map, *id).is_none())
        .collect();
    arm(sim, map, &bare, d, knobs.0, knobs.1)
}

/// **O slider de Offset diz a verdade da seleção** (smoke de 11/10: *«o slider mostra 25 mas o efeito
/// não aparece na estrela»*). Sem offset vivo em nenhuma forma de `ids` ele tem de ler ZERO —
/// `Some(0.0)` quando o `slider` mostra outra coisa. `None` = nada a corrigir: há offset vivo (quem
/// escreve o número é o espelho da seleção) ou o slider já está no zero.
#[must_use]
pub fn shown_frac(
    sim: &SimWorld,
    map: &VecEntityMap,
    ids: &[VecPathId],
    slider: f64,
) -> Option<f64> {
    let live = ids.iter().any(|id| spec_of(sim, map, *id).is_some());
    (!live && slider != 0.0).then_some(0.0)
}

/// A fração que um chip arma numa forma nua: a do slider, ou — com ele no zero, que é o que uma
/// forma nua mostra ([`shown_frac`]) — a de NASCENÇA. Um chip que não desenhasse era o BUGS #41.
#[must_use]
pub fn chip_frac(slider: f64) -> f64 {
    if slider == 0.0 {
        ph2d_tool_vector::params::OFFSET_DEFAULT_FRAC
    } else {
        slider
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
    sim: &SimWorld,
    pen: &mut ph2d_vec_edit::PenTool,
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
    crate::expand::materialise_selection(scene, pen, xforms, ids, |id, local, xf| {
        let Some((_, spec)) = live.iter().find(|(i, _)| *i == id) else {
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
            },
            spec.d,
        )
    });
    eprintln!("[ph2d-vec] offset materializado: {} forma(s)", live.len());
    true
}

#[cfg(test)]
#[path = "offset_live_tests.rs"]
mod tests;
