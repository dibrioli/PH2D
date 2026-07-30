//! **A LARGURA VIVA na shell** — o cozimento e a materialização do
//! [`ph2d_ecs::VecStrokeProfile`] (ADR-0145).
//!
//! Espelho exato do [`crate::offset_live`]: o componente guarda a **relação** (que largura, onde)
//! e a aparência é uma **função pura** dela, re-cozida aqui e desenhada por
//! [`ph2d_vec_render::dispatch`] **no z da forma**. O documento **nunca é tocado** enquanto o
//! perfil é preview — o `VecPath` continua sendo a curva autorada (é ela que o modo Node edita), e
//! a fita de largura variável é DESENHO.
//!
//! # A promessa que este módulo cumpre
//!
//! Antes disto os quatro sliders `W Start/Mid/End/Pos` eram **parâmetros de um comando**: nada
//! acontecia até o clique em *Power Stroke*, e o clique assava. O artista escolhia quatro números
//! às cegas e via o resultado depois de a curva já ter sido consumida. Agora eles autoram um
//! perfil vivo — o traço engrossa e afina **enquanto o slider anda** —, e o botão MATERIALIZA, o
//! mesmo par que o Offset já tinha (*"previsualizações em tempo real … para consolidar a curva
//! deve-se apertar Apply"*, Enio 2026-07-21).
//!
//! # Uma porta produz o desenho, e é a MESMA que assa
//!
//! [`crate::vec_expand::power_stroke_layers`] — o preview desenha o que ela devolve e o Apply
//! insere o que ela devolve. Uma segunda rota ("um aproximador só para o preview") faria a forma
//! **SALTAR** no instante do Apply, que é o defeito que o ADR-0128 pagou cinco vezes. Há gate
//! comparando as duas saídas byte a byte.
//!
//! # O neutro é a AUSÊNCIA
//!
//! Perfil uniforme ⇒ o componente é **removido**, não guardado com todos os multiplicadores em
//! `1.0`. Um documento não acumula relações invisíveis que não desenham nada, e é isso que faz
//! *"arrastar os sliders de volta ao neutro"* devolver a forma ao estado limpo — a mesma lei do
//! `VecOffset` com `d = 0`.
//!
//! # O memo é medido, como o do irmão
//!
//! `power_stroke` custa **0,26–0,55 ms** por forma (`measure_power_stroke`, release). O cozimento
//! roda por frame, então sem memo uma cena parada pagaria isso para sempre. A chave é o que de
//! facto o determina — a geometria de MUNDO que entra (que já carrega a largura do traço) e as
//! paradas —, e não um contador de versão que alguém esqueceria de bumpar.

use std::collections::BTreeMap;

use ph2d_ecs::{Entity, SimWorld, VecStrokeProfile};
use ph2d_editor::WidgetStore;
use ph2d_tool_vector::params;
use ph2d_vec_render::LiveGeometry;
use ph2d_vec_scene::{
    VecPath, VecPathId, VecScene, VecXforms, WidthProfile, WidthStops, bake_xform, xform_of,
};

use crate::vec_entities::VecEntityMap;

/// Uma entrada do memo: o que ENTROU (a geometria de mundo + as paradas) e o que SAIU.
struct Memo {
    stops: WidthStops,
    world: VecPath,
    out: Vec<VecPath>,
}

/// O cozimento vivo de todos os perfis de largura da cena, com memo por caminho.
#[derive(Default)]
pub(crate) struct ProfileLive {
    memo: BTreeMap<VecPathId, Memo>,
    live: LiveGeometry,
}

impl ProfileLive {
    /// A geometria derivada deste frame — o que o [`ph2d_vec_render::dispatch`] desenha no lugar
    /// da fonte. Vazia = nenhum perfil vivo na cena, e o desenho é o de sempre.
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
            let Some(stops) = spec_of(sim, map, path.id) else {
                continue;
            };
            // A pose viaja DENTRO da geometria: o resultado é de mundo e o `dispatch` o sobe pela
            // câmera. Assar aqui é o que permite ao perfil atravessar uma pose com rotação/escala
            // sem que ninguém a aplique duas vezes.
            let mut world = path.cooked().into_owned();
            bake_xform(&mut world, &xform_of(xforms, path.id));
            let hit = self
                .memo
                .get(&path.id)
                .is_some_and(|m| m.stops == stops && m.world == world);
            if !hit {
                let out = crate::vec_expand::power_stroke_layers(&world, &stops);
                self.memo.insert(
                    path.id,
                    Memo {
                        stops,
                        world: world.clone(),
                        out,
                    },
                );
            }
            // ⚠️ **Vazio OMITE a entrada — o oposto do irmão do offset, e por um motivo que não
            // se transfere.** Lá, vazio é a ANIQUILAÇÃO (o offset comeu a forma) e desenhar nada
            // é a resposta certa. Aqui, `power_stroke` devolve vazio quando a forma **não tem
            // traço**: uma forma só-preenchida com um perfil armado não tem fita nenhuma, e
            // substituí-la por nada a faria DESAPARECER da tela. Sem entrada, o `dispatch`
            // desenha a fonte — que é exatamente o que ela é.
            if let Some(m) = self.memo.get(&path.id).filter(|m| !m.out.is_empty()) {
                self.live.insert(path.id, m.out.clone());
            }
        }
        // O memo não pode sobreviver ao componente: uma forma que perdeu o perfil (Apply, Ctrl+Z)
        // manteria a resposta velha e re-cozinharia o mundo errado se ele voltasse.
        //
        // ⚠️ A pergunta é ao COMPONENTE, e não a `self.live`: uma forma sem traço tem perfil
        // armado e saída vazia (ver acima), então varrer pela geometria derivada jogaria o memo
        // dela fora a cada frame — e re-cozinhar de graça é justamente o que o memo existe para
        // não fazer.
        self.memo.retain(|id, _| spec_of(sim, map, *id).is_some());
    }

    /// Esquece tudo — o load de projeto e o restore de undo trocam a cena inteira debaixo do
    /// memo, e os `VecPathId` são reciclados entre documentos.
    pub(crate) fn forget(&mut self) {
        self.memo.clear();
        self.live.clear();
    }
}

/// O perfil vivo de `id`, se houver. Porta única: o cozimento, o `Apply` e o publish para o
/// painel perguntam AQUI.
pub(crate) fn spec_of(sim: &SimWorld, map: &VecEntityMap, id: VecPathId) -> Option<WidthStops> {
    let &bits = map.get(&id)?;
    sim.world()
        .get::<VecStrokeProfile>(Entity::from_bits(bits))
        .map(|p| p.stops.clone())
}

/// **Arma** (ou desarma) o perfil vivo de cada caminho de `ids`.
///
/// Um perfil UNIFORME **REMOVE** o componente em vez de guardar um perfil inerte — ver o
/// cabeçalho. Devolve quantas entidades mudaram.
pub(crate) fn arm(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    ids: &[VecPathId],
    stops: &WidthStops,
) -> usize {
    let want = (!stops.is_uniform()).then(|| VecStrokeProfile {
        stops: stops.clone(),
    });
    let mut n = 0;
    for id in ids {
        let Some(&bits) = map.get(id) else { continue };
        let e = Entity::from_bits(bits);
        let cur = sim.world().get::<VecStrokeProfile>(e).cloned();
        if cur == want {
            continue;
        }
        let Ok(mut em) = sim.world_mut().get_entity_mut(e) else {
            continue;
        };
        match &want {
            Some(v) => {
                em.insert(v.clone());
            }
            None => {
                em.remove::<VecStrokeProfile>();
            }
        }
        n += 1;
    }
    n
}

/// **MATERIALIZA** os perfis vivos da seleção — o botão *Power Stroke*.
///
/// É o único momento em que os vértices da fita passam a existir no documento. Cada forma é
/// assada com o perfil DELA (não com os sliders), pela porta única
/// [`crate::vec_expand::expand_selection`] — a mesma por onde entra o caminho numérico.
///
/// `false` = não havia perfil vivo nenhum na seleção (e quem chamou segue pelo caminho de
/// sempre, lendo os sliders). **UM passo de undo** para o gesto inteiro.
pub(crate) fn materialise(
    scene: &mut VecScene,
    sim: &SimWorld,
    pen: &mut ph2d_vec_edit::PenTool,
    history: &mut ph2d_vec_edit::History,
    map: &VecEntityMap,
    xforms: &VecXforms,
    ids: &[VecPathId],
) -> bool {
    let live: Vec<(VecPathId, WidthStops)> = ids
        .iter()
        .filter_map(|id| spec_of(sim, map, *id).map(|s| (*id, s)))
        .collect();
    if live.is_empty() {
        return false;
    }
    let pre = scene.clone();
    let touched = crate::vec_expand::expand_selection(scene, pen, xforms, ids, |id| {
        live.iter().find(|(i, _)| *i == id).map(|(_, stops)| {
            (
                crate::vec_expand::Expand::PowerStroke {
                    stops: stops.clone(),
                },
                0.0,
            )
        })
    });
    if touched {
        history.push_undo(pre);
    }
    eprintln!(
        "[ph2d-vec] power stroke materializado: {} forma(s)",
        live.len()
    );
    true
}

/// **Os quatro sliders → o preset.** Porta única: o armamento do perfil vivo e o caminho
/// numérico (clicar o botão sem nada armado) leem daqui, e os defaults são os do
/// [`ph2d_tool_vector::params`] — antes eram literais repetidos no `render_loop`, uma segunda
/// cópia de quatro números que o painel já publica.
#[must_use]
pub(crate) fn preset_from_store(store: &WidgetStore) -> WidthProfile {
    let mult = |id, default: f64| {
        store
            .slider(id)
            .map_or(default, |(_, v)| params::slider_to_wprofile(v))
    };
    WidthProfile {
        start: mult(
            ph2d_editor::ids::VECTOR_EXPAND_W_START,
            params::WPROFILE_DEFAULT_START,
        ),
        mid: mult(
            ph2d_editor::ids::VECTOR_EXPAND_W_MID,
            params::WPROFILE_DEFAULT_MID,
        ),
        end: mult(
            ph2d_editor::ids::VECTOR_EXPAND_W_END,
            params::WPROFILE_DEFAULT_END,
        ),
        // A posição é a fração crua do trilho — o meio senta onde o slider está, e não há
        // faixa a remapear (o domínio dela JÁ é `[0,1]`).
        position: store
            .slider(ph2d_editor::ids::VECTOR_EXPAND_W_POS)
            .map_or(params::WPROFILE_DEFAULT_POS, |(_, v)| f64::from(v)),
    }
}

/// **As paradas → o preset**, quando elas SÃO um preset (três paradas, nas pontas e no meio).
///
/// `None` para qualquer outra lista: um perfil de alças arbitrárias (W2) não tem quatro números
/// que o descrevam, e inventar quatro faria os sliders mentirem sobre a forma. O espelho da
/// seleção então deixa os knobs onde estão — que é a resposta honesta a *"isto não cabe aqui"*.
#[must_use]
pub(crate) fn preset_of(stops: &WidthStops) -> Option<WidthProfile> {
    let s = stops.as_slice();
    let [a, m, b] = s else { return None };
    (a.pos == 0.0 && b.pos == 1.0).then_some(WidthProfile {
        start: a.mult,
        mid: m.mult,
        end: b.mult,
        position: m.pos,
    })
}

/// **O preset → os quatro sliders**, o espelho da seleção: escolher uma forma que já tem perfil
/// mostra o perfil DELA nos knobs, em vez de deixar o painel mentindo sobre o que está na tela.
///
/// ⚠️ Só faz sentido para um perfil que É um preset (três paradas). Um perfil de alças arbitrárias
/// (W2) não tem quatro números que o descrevam, e é o painel de então que decide o que mostrar —
/// por isso a conversão de volta vive aqui, no lado da UI, e não no tipo.
pub(crate) fn write_preset_to_store(store: &mut WidgetStore, p: &WidthProfile) {
    for (id, v) in [
        (
            ph2d_editor::ids::VECTOR_EXPAND_W_START,
            params::wprofile_to_slider(p.start),
        ),
        (
            ph2d_editor::ids::VECTOR_EXPAND_W_MID,
            params::wprofile_to_slider(p.mid),
        ),
        (
            ph2d_editor::ids::VECTOR_EXPAND_W_END,
            params::wprofile_to_slider(p.end),
        ),
        #[allow(clippy::cast_possible_truncation)]
        (
            ph2d_editor::ids::VECTOR_EXPAND_W_POS,
            p.position.clamp(0.0, 1.0) as f32,
        ),
    ] {
        store.set_slider_value(id, v);
    }
}

#[cfg(test)]
#[path = "profile_live_tests.rs"]
mod tests;
