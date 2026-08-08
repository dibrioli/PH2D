//! **Os verbos dos ESTADOS de UI** (plano UI/UX W7) — irmão do [`crate::vec_widget_edit`], mesma
//! divisão: o painel PEDE, a shell FAZ, e o que muda é o documento.
//!
//! # O que um estado GRAVA, e por que a sub-árvore inteira
//!
//! Um botão não é uma forma: é um retângulo, um rótulo e talvez um ícone. Gravar só o hospedeiro
//! deixaria de fora justamente o que se move num hover. Então um estado captura o hospedeiro **e
//! cada descendente que é uma forma**, que é o mesmo conjunto que a booleana, o gizmo e o z-order
//! já chamam de *a sub-árvore* — [`crate::vec_entities::subtree_paths`], nunca uma travessia
//! própria.
//!
//! # O `Transform` LOCAL, nunca o de mundo
//!
//! Uma pose é relativa ao hospedeiro: arrastar o botão inteiro para outro canto da tela **não pode
//! invalidar os estados dele**. O local é exatamente essa relação, e é o que a hierarquia já
//! guarda — pedir mundo aqui obrigaria a re-derivar contra a pose do pai a cada gravação, e a
//! primeira vez que alguém movesse o pai os dois números discordariam.
//!
//! ⚠️ **`skew` fica de fora, e a ausência é decisão:** a pose interpola por T/R/S decompostos
//! ([`ph2d_ui_state::ObjectPose`]) justamente para não lerpar matriz; um skew autorado sobrevive
//! porque nada aqui o escreve, mas ele **não anima** entre dois estados. Nomeado em vez de
//! descoberto.

use ph2d_ecs::{Entity, SimWorld, Transform, VecStrokeProfile};
use ph2d_ui_state::{ObjectPose, StateRole, StateSets, UiState};
use ph2d_vec_scene::WidthStops;
use ph2d_vec_scene::{VecPathId, VecScene};

use crate::vec_entities::VecEntityMap;

/// O que um clique na seção STATES pede.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum UiStateEdit {
    /// **Record / Update** — grava a pose ATUAL da sub-árvore neste papel.
    Record(StateRole),
    /// **Clear** — esquece a pose deste papel.
    Clear(StateRole),
    /// **Apply** — põe a cena na pose deste papel, para o artista a editar.
    Apply(StateRole),
}

/// Este id é um verbo de estado? **Porta única** do roteador.
#[must_use]
pub(crate) fn ui_state_edit_for_id(id: ph2d_editor::NodeId) -> Option<UiStateEdit> {
    for (i, &role) in StateRole::ALL.iter().enumerate() {
        if id == ph2d_editor::ids::vector_state_record_id(i) {
            return Some(UiStateEdit::Record(role));
        }
        if id == ph2d_editor::ids::vector_state_clear_id(i) {
            return Some(UiStateEdit::Clear(role));
        }
        if id == ph2d_editor::ids::vector_state_apply_id(i) {
            return Some(UiStateEdit::Apply(role));
        }
    }
    None
}

/// Que metade da CURVA um clique escolheu.
///
/// ⚠️ **Duas variantes e não uma `Easing` inteira**, porque o artista escolhe uma metade de cada
/// vez: clicar numa família tem de preservar a direção que ele já escolheu, e vice-versa. Um pick
/// que carregasse a curva completa obrigaria o painel a reconstruir a outra metade — e o painel
/// pinta a partir do que a shell publica, então ele estaria a adivinhar o que o documento tem.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EasingPick {
    Family(ph2d_anim::EasingFamily),
    Mode(ph2d_anim::EasingMode),
}

/// Este id é um chip do seletor de curva? **Porta única** do roteador — a irmã exata do
/// [`ui_state_edit_for_id`], e percorrida pelo mesmo `ALL` que os pinta.
#[must_use]
pub(crate) fn easing_pick_for_id(id: ph2d_editor::NodeId) -> Option<EasingPick> {
    for (i, &f) in ph2d_anim::EasingFamily::ALL.iter().enumerate() {
        if id == ph2d_editor::ids::vector_easing_family_id(i) {
            return Some(EasingPick::Family(f));
        }
    }
    for (i, &m) in ph2d_anim::EasingMode::ALL.iter().enumerate() {
        if id == ph2d_editor::ids::vector_easing_mode_id(i) {
            return Some(EasingPick::Mode(m));
        }
    }
    None
}

/// Aplica um pick à curva que o hospedeiro tem hoje.
///
/// ⚠️ **Escolher `Linear` NÃO normaliza o modo guardado**, e é deliberado: `Linear` ignora-o, mas
/// o artista que passe por `Linear` e volte a `Quad` espera reencontrar a direção que escolheu.
/// Zerá-la aqui seria perder uma decisão dele para arrumar um byte que ninguém lê.
#[must_use]
pub(crate) fn easing_with(cur: ph2d_anim::Easing, pick: EasingPick) -> ph2d_anim::Easing {
    match pick {
        EasingPick::Family(family) => ph2d_anim::Easing { family, ..cur },
        EasingPick::Mode(mode) => ph2d_anim::Easing { mode, ..cur },
    }
}

/// O hospedeiro: a forma ÚNICA selecionada.
///
/// ⚠️ Seleção múltipla não tem hospedeiro — *"gravar o estado destas três formas"* teria de
/// escolher qual delas é o assunto, e escolher em silêncio é como um estado nasce pendurado no
/// objeto errado.
fn host(selected: &[VecPathId]) -> Option<VecPathId> {
    match selected {
        [only] => Some(*only),
        _ => None,
    }
}

fn entity_of(map: &VecEntityMap, id: VecPathId) -> Option<Entity> {
    map.get(&id).map(|&bits| Entity::from_bits(bits))
}

/// Os caminhos que este hospedeiro governa: ele próprio e cada descendente que é uma forma.
///
/// ⚠️ `pub(crate)` desde o modo de PREVIEW (W7r): ele pergunta o INVERSO — *"o que o rato tocou
/// pertence a algum hospedeiro?"* — e uma segunda travessia da árvore ao lado daria duas respostas
/// a *"quem este botão governa"*, com o hover a morrer sobre a metade que uma delas esquecesse.
pub(crate) fn members(
    sim: &SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    host: VecPathId,
) -> Vec<VecPathId> {
    let Some(e) = entity_of(map, host) else {
        return Vec::new();
    };
    let mut v = crate::vec_entities::subtree_paths(sim, scene, e);
    // O hospedeiro pode ser um GRUPO puro, que não tem forma própria — nesse caso ele não aparece
    // na lista, e é isso que queremos: um grupo não tem tinta para gravar.
    if !v.contains(&host) && scene.paths().iter().any(|p| p.id == host) {
        v.push(host);
    }
    v
}

/// **A pose de AGORA**, lida do mundo e do documento.
///
/// ⚠️ `pub(crate)` desde o modo de PREVIEW (W7r): ele captura, ao ENTRAR, exactamente a mesma
/// coisa que o **Rec** captura — *o que a cena mostra agora* —, e uma segunda leitura ao lado
/// seria a que esquece um canal no dia em que a pose ganhar um. Uma porta, dois consumidores.
#[must_use]
pub(crate) fn capture(
    sim: &SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    id: VecPathId,
) -> ObjectPose {
    let mut pose = ObjectPose::new(id);
    if let Some(e) = entity_of(map, id) {
        if let Some(t) = sim.world().get::<Transform>(e) {
            pose.translation = [f64::from(t.translation.x), f64::from(t.translation.y)];
            pose.rotation = f64::from(t.rotation);
            pose.scale = [f64::from(t.scale.x), f64::from(t.scale.y)];
        }
        // A LARGURA VIVA: o único canal de forma que mora num componente, e não no `VecPath`.
        pose.width = sim
            .world()
            .get::<VecStrokeProfile>(e)
            .map(|w| w.stops.clone());
    }
    if let Some(p) = scene.paths().iter().find(|p| p.id == id) {
        pose.fill.clone_from(&p.fill);
        pose.stroke = p.stroke;
        // **A FORMA, sempre.** Um estado que não a gravasse não teria como animar uma edição de
        // nó, um Fillet ou um Chamfer — e o campo existia, com o motor pronto do outro lado,
        // esperando um produtor que nunca chegou.
        //
        // ⚠️ **A fonte AUTORADA, não a cozida.** É ela que o modo Node edita e é ela que o
        // `install` devolve na chegada; gravar a cozida assaria o raio de quina e a pilha de
        // efeitos no documento, e o artista perderia as alças no primeiro Show. Quem coze é a
        // [`ph2d_ui_state::Transition`], para o CAMINHO — a costura fonte≠cozido do ADR-0121,
        // no nível do estado.
        //
        // ⚠️ **E a tinta sai daqui.** Ela é campo de primeira classe da pose; deixá-la também
        // dentro da geometria daria dois lugares para o mesmo fato dentro do mesmo arquivo.
        let mut g = p.clone();
        g.fill = None;
        g.stroke = None;
        pose.geometry = Some(g);
    }
    pose
}

/// **Escreve uma pose de volta** no mundo e no documento.
///
/// ⚠️ **A forma escreve TUDO o que a forma é** — verts, fechamento, contornos, regra de
/// preenchimento e a pilha de efeitos. Escrever metade deixaria a outra metade a descrever a
/// forma anterior: a pilha do estado antigo re-aplicada sobre a geometria do novo é uma dobra
/// que ninguém autorou.
///
/// ⚠️ **E é por isso que a pose do MEIO chega aqui já cozida e com a pilha VAZIA**
/// ([`ph2d_ui_state::Transition`]): a geometria intermédia já tem o raio e os efeitos
/// realizados, então re-cozinhá-la seria aplicá-los duas vezes. Na CHEGADA volta a autorada, com
/// as alças de quina e a pilha intactas — a passagem pelo documento é transitória e cura-se
/// sozinha, e é o preço de o Show ter de deixar a cena *editável no estado que mostra*.
pub(crate) fn install(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    map: &VecEntityMap,
    pose: &ObjectPose,
) {
    if let Some(e) = entity_of(map, pose.id) {
        if let Some(mut t) = sim.world_mut().get_mut::<Transform>(e) {
            #[allow(clippy::cast_possible_truncation)]
            {
                t.translation.x = pose.translation[0] as f32;
                t.translation.y = pose.translation[1] as f32;
                t.rotation = pose.rotation as f32;
                t.scale.x = pose.scale[0] as f32;
                t.scale.y = pose.scale[1] as f32;
            }
        }
        // ⚠️ **Pela porta do Width Tool**, e não por uma escrita própria: `profile_live::arm` já
        // é quem sabe que *uniforme é a AUSÊNCIA do componente* (a lei do `VecOffset` com
        // `d = 0`). Uma segunda escrita aqui seria uma segunda resposta a *"como um perfil chega
        // ao mundo?"*, e as duas divergiriam no dia em que a lei mudasse de um lado só.
        crate::profile_live::arm(
            sim,
            map,
            &[pose.id],
            pose.width.as_ref().unwrap_or(&WidthStops::default()),
        );
    }
    if let Some(p) = scene.paths_mut().iter_mut().find(|p| p.id == pose.id) {
        p.fill.clone_from(&pose.fill);
        p.stroke = pose.stroke;
        if let Some(g) = &pose.geometry {
            p.verts.clone_from(&g.verts);
            p.closed = g.closed;
            p.subpaths.clone_from(&g.subpaths);
            p.fill_rule = g.fill_rule;
            p.effects.clone_from(&g.effects);
        }
    }
}

/// Aplica o verbo. Sem hospedeiro único, não faz nada — a seção nem é oferecida nesse caso.
///
/// Devolve o `(hospedeiro, papel)` que o artista pediu para **VER**, se foi esse o verbo.
///
/// ⚠️ **O Show não escreve pose aqui**, e a fronteira é o desenho inteiro: uma escrita direta
/// seria uma SEGUNDA porta para *"pôr a cena nesta pose"*, ao lado da máquina — e a diferença
/// entre as duas é justamente o tween que o artista autorou. Quem mostra é a
/// [`crate::render_loop::ui_state_bridge`]; aqui só se decodifica o pedido.
pub(crate) fn apply(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    map: &VecEntityMap,
    selected: &[VecPathId],
    states: &mut StateSets,
    verb: UiStateEdit,
) -> Option<(VecPathId, StateRole)> {
    let h = host(selected)?;
    match verb {
        UiStateEdit::Record(role) => {
            let mut s = UiState::new(role);
            s.objects = members(sim, scene, map, h)
                .into_iter()
                .map(|id| capture(sim, scene, map, id))
                .collect();
            states.set(h, s);
            None
        }
        UiStateEdit::Clear(role) => {
            states.clear(h, role);
            None
        }
        UiStateEdit::Apply(role) => Some((h, role)),
    }
}

/// O que o painel mostra para a seleção — `None` = não oferecer a seção.
///
/// ⚠️ Ela é oferecida para **qualquer forma única**, com estados ou sem — uma seção que só
/// existisse onde já há estados tornaria a feature alcançável apenas onde ela já foi usada, ou
/// seja em lugar nenhum. É a mesma lei da seção de física, cuja face VAZIA é a importante.
/// **Move o widget inteiro, carregando TODOS os estados** (Enio, 2026-08-07).
///
/// Desloca a pose do **HOSPEDEIRO** por `delta` em cada estado gravado dele. Devolve `true` se
/// alguma pose se moveu.
///
/// # ⚠️ Só o HOSPEDEIRO, e é isso que torna a operação correta
///
/// As poses dos filhos são **LOCAIS ao hospedeiro** ([`capture`]), então mover o `Transform` dele
/// já os leva junto na tela. Deslocá-los também moveria tudo **duas vezes** — e destruiria
/// exactamente o que o artista quer preservar: *a coreografia interna do widget*.
///
/// # Por que ela precisa de existir
///
/// Um estado grava a sub-árvore, e o hospedeiro está nela sempre que ele próprio é uma forma
/// desenhada. Então a translação ABSOLUTA dele fica congelada em cada estado, e relocar o widget
/// deixa de funcionar: mostrar um estado **devolve a forma ao lugar antigo**. ⚠️ Um hospedeiro que
/// seja um GRUPO puro nunca teve o problema (o `members` não o inclui — ele não tem forma), e é
/// por isso que o defeito só aparece depois de o artista gravar um estado que move a própria
/// forma-hospedeiro.
pub(crate) fn shift_host_in_all_states(
    states: &mut StateSets,
    host: VecPathId,
    delta: [f64; 2],
) -> bool {
    if delta == [0.0, 0.0] {
        return false;
    }
    let mut moved = false;
    for role in StateRole::ALL {
        let Some(mut st) = states.role(host, role).cloned() else {
            continue;
        };
        // ⚠️ **A flag é POR ESTADO**, e não do laço: com uma flag acumulada, o primeiro estado
        // que se move faz TODOS os seguintes serem re-escritos, inclusive os que não contêm o
        // hospedeiro. É inócuo hoje (re-escrever o mesmo valor), e é a forma exacta de um defeito
        // que só aparece no dia em que o `set` ganhar um efeito colateral.
        let mut here = false;
        for pose in &mut st.objects {
            if pose.id == host {
                pose.translation[0] += delta[0];
                pose.translation[1] += delta[1];
                here = true;
            }
        }
        if here {
            states.set(host, st);
            moved = true;
        }
    }
    moved
}

#[must_use]
pub(crate) fn publish(
    selected: &[VecPathId],
    states: &StateSets,
    live: Option<usize>,
    preview_on: bool,
    move_all: bool,
) -> Option<ph2d_panel_vector::state::UiStatesState> {
    let h = host(selected)?;
    let (duration, easing) = states.timing(h);
    Some(ph2d_panel_vector::state::UiStatesState {
        recorded: StateRole::ALL.map(|r| states.role(h, r).is_some()),
        // ⚠️ Os rótulos saem do CATÁLOGO, não de uma lista no painel: um papel novo aparece
        // nomeado sem ninguém tocar na UI, e nenhuma segunda lista pode envelhecer ao lado
        // desta.
        role_labels: StateRole::ALL.map(|r| ph2d_i18n::tr(r.i18n_key()).to_string()),
        live,
        #[allow(clippy::cast_possible_truncation)]
        duration_s: duration as f32,
        // **A MOLA** — pela MESMA porta que o motor pergunta (`StateSets::spring`), e não por uma
        // leitura ao lado: se o painel derivasse a resposta noutro lugar, ele pintaria as linhas
        // de mola sobre uma cena a andar por curva.
        #[allow(clippy::cast_possible_truncation)]
        spring: states
            .spring(h)
            .map(|s| (s.stiffness as f32, s.damping as f32)),
        // **O interruptor da PREVIEW** só é oferecido quando existe pose autorada em ALGUM
        // hospedeiro — que é exatamente a condição em que [`UiPreview::enter`] liga.
        //
        // ⚠️ **A pergunta é a MESMA que o modelo faz**, e não uma segunda cópia da regra: um
        // botão pintado sobre uma cena sem poses seria um clique que não faz nada, e o artista
        // não teria como saber que o que falta é gravar um estado.
        preview: (!states.is_empty()).then_some(preview_on),
        // **Mover carregando todos os estados** só faz sentido onde ESTE hospedeiro tem pose
        // gravada — a pergunta é sobre o widget que se vai arrastar, não sobre a cena (é o
        // oposto do `preview`, que entrega o rato a todos).
        move_all: StateRole::ALL
            .iter()
            .any(|&r| states.role(h, r).is_some())
            .then_some(move_all),
        // **A CURVA** — publicada SEMPRE, e não só onde há pose gravada.
        //
        // ⚠️ É a mesma regra da duração, que está uma linha acima: as duas descrevem *como este
        // hospedeiro transita*, e afiná-las antes de gravar o primeiro estado é a ordem natural
        // (escolho o feel, depois poso). Escondê-las até haver pose seria uma seção que muda de
        // tamanho enquanto o artista trabalha, pelo motivo que ele não vê.
        easing,
    })
}

#[cfg(test)]
#[path = "vec_ui_state_edit_tests.rs"]
mod tests;
