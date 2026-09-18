//! **Fase do quadro: OS INSTANTÂNEOS QUE O `publish` NÃO PODE CALCULAR** (SCRIPT · PARTICLES ·
//! HUD) —
//! fase-filha do [`super`], num ficheiro irmão.
//!
//! ⚠️ **O corte foi imposto pelo tecto de FUNÇÃO** (a `fase_snapshots_publish` chegou a `208` contra
//! `200` ao ganhar o emissor) **e é o certo por responsabilidade**: as duas secções leem coisas que
//! **não estão no mundo** — a VM dos scripts, e o relógio mais as partículas vivas da corrida —, e é
//! exactamente por isso que nenhuma delas cabe na lista de argumentos do `publish`.
//!
//! ⚠️ **Depois do passo dos motores** (`fase_signal_outbox`): publicar antes mostraria a contagem de
//! partículas do quadro ANTERIOR.
//!
//! ⛔ **O corpo mora na família** (`ph2d_app_components::{script_inspector, particles_inspector}`);
//! daqui sai só a escolha de quem é o sujeito — o objecto activo.

use ph2d_app_components::particles_bridge::ParticlesState;
use ph2d_ecs::SimWorld;
use ph2d_script::ScriptHost;

/// Publica os dois instantâneos do objecto ACTIVO. `None` = ninguém escolhido ⇒ nenhuma das duas
/// secções existe, que é a lei do ADR-0166.
///
/// ⚠️ **O `sim` é `&mut` por causa do HUD**, e a razão é a lei dele: o que um rótulo MOSTRA é
/// derivado do mundo pela MESMA porta que o desenho usa (`ph2d_ecs::hud::texto`), e uma consulta
/// do `bevy` pede `&mut World`. *Uma segunda conta aqui seria a segunda resposta a «o que este
/// rótulo diz?», e as duas divergiriam no dia em que uma fonte nova entrasse.*
/// **O que o EDITOR sabe e o mundo não** — o sujeito escolhido, o relógio e o mapa de acções.
///
/// ⚠️ **Ela existe porque a lista de argumentos cresceu para OITO** ao ganhar o mapa (o gatilho
/// precisa de saber se a acção que uma linha nomeia existe), e o corte não é de estilo: os quatro
/// primeiros argumentos do [`publica`] são o MUNDO e estes quatro são a SESSÃO. *Duas naturezas,
/// dois grupos.* ⛔ Um `#[allow]` teria calado o diagnóstico sem responder à pergunta dele.
pub(super) struct Sessao<'a> {
    /// Quem está escolhido (`None` = ninguém ⇒ nenhuma das duas secções existe, ADR-0166).
    pub(super) escolhido: Option<u64>,
    /// Quantos estão escolhidos — a BulkSelect, que o painel NOMEIA e não espalha.
    pub(super) quantos: usize,
    /// A corrida está a andar? (É a cerca que faz as teclas do jogo não serem as do editor.)
    pub(super) a_correr: bool,
    /// O mapa de acções, para a coluna *«esta acção existe?»* da secção do gatilho.
    pub(super) mapa: &'a ph2d_input::InputMap,
}

pub(super) fn publica(
    sim: &mut SimWorld,
    tags: &ph2d_tags::TagTree,
    script: Option<&ScriptHost>,
    particles: &ParticlesState,
    s: &Sessao<'_>,
) {
    let Sessao {
        escolhido,
        quantos,
        a_correr,
        mapa,
    } = *s;
    // ⚠️ **A pergunta é «há CÂMERA DE JOGO na cena?»**, e não «a pré-visualização está ligada»: o
    // canvas cola-se à vista dela em qualquer dos casos, e é a AUSÊNCIA da câmera que deixa o HUD
    // onde o artista o pôs. ⭐ Ela mora AQUI e não na fase-mãe porque é uma pergunta ao MUNDO —
    // que é o que esta fase-filha tem em mãos — e porque o tecto daquela função é para o que ela
    // COMPÕE, não para o que ela calcula.
    let tem_camera = {
        let world = sim.world_mut();
        world
            .query::<&ph2d_ecs::GameCamera>()
            .iter(world)
            .next()
            .is_some()
    };
    ph2d_panel_inspector::set_current_inspector_script(escolhido.and_then(|b| {
        ph2d_app_components::script_inspector::build_info(sim, script, b, quantos, a_correr)
    }));
    ph2d_panel_inspector::set_current_inspector_particles(escolhido.and_then(|b| {
        ph2d_app_components::particles_inspector::build_info(
            sim,
            b,
            quantos,
            a_correr,
            particles.alive_of(b),
        )
    }));
    // ⭐⭐⭐ **O HUD** (TOP-20 #20) — e ele traz DOIS factos que não são campos: se há câmera de
    // jogo (senão o canvas não se cola a nada) e o que o rótulo mostra AGORA.
    ph2d_panel_inspector::set_current_inspector_hud(
        escolhido
            .and_then(|b| ph2d_app_components::hud_inspector::build_info(sim, tags, b, tem_camera)),
    );
    // ⭐⭐⭐ **A VIGIA DO CONTADOR** — ela cabe aqui e não na fase-mãe pela mesma razão das irmãs:
    // a coluna *«existe um contador com este nome?»* é uma **varredura do MUNDO**, e o painel não
    // o vê. ⚠️ Sem ela, uma regra com um `d` a mais no nome lê-se exactamente como uma que funciona.
    ph2d_panel_inspector::set_current_inspector_counter_watch(escolhido.and_then(|b| {
        ph2d_app_components::counter_watch_inspector::build_info(sim, b, a_correr, quantos)
    }));
    // ⭐⭐⭐ **O GATILHO** (suplente #24) — e ele cabe aqui pela MESMA razão das irmãs: a coluna
    // *«existe uma acção com este nome?»* está no **Input Map**, que não é o mundo. ⚠️ Sem ela,
    // uma linha com `fier` em vez de `fire` lê-se exactamente como uma que funciona — e o silêncio
    // dela é DUPLO, porque a lei do motor também cala uma acção que o mapa não conhece.
    ph2d_panel_inspector::set_current_inspector_action_trigger(escolhido.and_then(|b| {
        ph2d_app_components::action_trigger_inspector::build_info(sim, b, a_correr, quantos, &|n| {
            mapa.id(n).is_some()
        })
    }));
}
