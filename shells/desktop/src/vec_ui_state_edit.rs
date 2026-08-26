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

use ph2d_ecs::{Entity, SimWorld, Transform, VecFilter, VecStrokeProfile};
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
        // **OS FILTROS** (FX raster) — o outro canal que mora num componente, e pela mesma razão
        // aqui. Sem esta linha a pose nasce sempre vazia de filtros e o `install` apagaria o blur
        // do artista no primeiro Show: um produtor que falta não dá erro nenhum, ele só perde o
        // canal em silêncio (é o que aconteceu com a `geometry` por uma wave inteira).
        pose.filters = sim
            .world()
            .get::<VecFilter>(e)
            .map(|f| f.ops.clone())
            .unwrap_or_default();
        // **O VERBO PRÓPRIO** desta forma dentro da booleana viva. ⚠️ A ausência do componente
        // grava-se como `None`, que é o mesmo *"herda o do grupo"* que ele significa no mundo —
        // traduzi-la aqui para o verbo efetivo seria congelar, no arquivo, uma herança que o
        // artista ainda pode mudar no grupo.
        pose.bool_op = sim.world().get::<ph2d_ecs::VecBoolOp>(e).map(|v| v.op);
        // ⭐⭐⭐ **EM QUE FORMA o conjunto de Morph States está** (plano 32 W11c) — o quarto canal
        // que mora num COMPONENTE e não no `VecPath`, e o que torna um conjunto animável pelo
        // sistema de States (Enio, 2026-08-26).
        //
        // ⚠️ **A forma que a cena MOSTRA é `sources[1]`** — o destino do último voo —, e não
        // `sources[0]`: `t = 1` no par `(A, B)` já **é** a forma B, e é essa a leitura que o
        // readout do painel e o motor partilham. Gravar a origem faria o `Hover` capturar a forma
        // de onde a máquina veio.
        //
        // ⛔ **Sem `VecMorphMachine` grava-se `None`**, e não a forma corrente: um morph autorado
        // à mão (dois operandos, `t` keyado pela timeline) não é um conjunto de estados, e dizer
        // que ele *está* numa forma faria o `install` prendê-lo lá — matando o `t` que a timeline
        // conduz.
        pose.morph_shape = sim
            .world()
            .get::<ph2d_ecs::VecMorphMachine>(e)
            .and(sim.world().get::<ph2d_ecs::VecMorph>(e))
            .map(|m| m.sources[1]);
    }
    // **E A OPERAÇÃO DO GRUPO acima dela** — o outro canal, e o que faz a receita inteira mudar
    // entre dois estados (as quatro de conjunto **e** as quatro receitas, que não têm decomposição
    // por forma nenhuma).
    //
    // ⚠️ **Ela é lida pela porta única** (`bool_live::group_above`) e não por uma segunda subida da
    // árvore: uma caminhada própria aqui daria uma segunda resposta a *"a quem esta forma
    // pertence"*, e as duas divergiriam no primeiro documento aninhado.
    pose.bool_group_op = crate::bool_live::group_above(sim, map, id).map(|(_, op)| op);
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
        // **OS FILTROS.** ⚠️ Pilha vazia REMOVE o componente — a lei do `VecOffset` que o próprio
        // `ph2d-fx-op` já enuncia (*"um documento não acumula relações inertes que não desenham
        // nada"*), e é ela que faz um estado sem filtro devolver a forma byte-idêntica em vez de
        // lhe pendurar uma pilha vazia.
        //
        // ⚠️ **Vazio ≠ neutro, e a diferença é load-bearing:** a pose do MEIO de uma transição
        // traz degraus de intensidade zero (não uma pilha vazia), então o componente existe e o
        // device desenha nada — que é o que faz o filtro CRESCER. Tratar o meio como vazio o
        // faria piscar de volta ao original em cada quadro.
        if let Ok(mut em) = sim.world_mut().get_entity_mut(e) {
            if pose.filters.is_empty() {
                em.remove::<VecFilter>();
            } else {
                em.insert(VecFilter {
                    ops: pose.filters.clone(),
                });
            }
            // **O VERBO PRÓPRIO.** ⚠️ `None` REMOVE o componente, e é a mesma lei do filtro e do
            // `VecOffset`: um documento não acumula relações inertes. Aqui ela tem um segundo
            // efeito que é o importante — sem a remoção, um estado que devolve a forma à herança
            // deixaria o override do outro estado colado nela, e o grupo passaria a ter uma forma
            // que não obedece a ninguém.
            match pose.bool_op {
                Some(op) => {
                    em.insert(ph2d_ecs::VecBoolOp { op });
                }
                None => {
                    em.remove::<ph2d_ecs::VecBoolOp>();
                }
            }
            // ⭐⭐⭐ **A FORMA do conjunto de Morph States** (W11c) — a chegada põe o par em
            // `(shape, shape)`, que é a forma exacta.
            //
            // ⛔ **`None` NÃO remove nada**, e a diferença do `bool_op` acima é o que ele
            // significa: ali `None` é *"volta à herança"* (uma decisão); aqui é *"esta pose não se
            // pronuncia"* — e escrever sobre um conjunto por causa disso poria uma pose antiga a
            // mandar num objecto que ela nunca conheceu.
            //
            // ⚠️ **Só se houver MÁQUINA.** Um morph autorado à mão tem o `t` conduzido pela
            // timeline; prendê-lo num par degenerado mataria a curva dela.
            if let Some(shape) = pose.morph_shape
                && em.get::<ph2d_ecs::VecMorphMachine>().is_some()
                && let Some(mut m) = em.get_mut::<ph2d_ecs::VecMorph>()
            {
                m.sources = [shape, shape];
                m.t = 0.0;
            }
        }
    }
    // **A OPERAÇÃO DO GRUPO.**
    //
    // ⚠️ **`None` NÃO desfaz o grupo** — ele é *"esta pose não sabe de grupo nenhum"*, e a escrita
    // simplesmente não acontece. Lê-lo como *"remova o `VecBoolGroup`"* faria uma pose gravada
    // ANTES de o artista criar a booleana **destruir** a booleana no primeiro Show.
    //
    // ⚠️ **E a escrita é condicional ao valor DIFERIR**, o que não é economia: `install` corre uma
    // vez por objeto POR QUADRO durante uma transição, e cada operando do grupo escreve o mesmo
    // número — um `insert` incondicional marcaria o grupo como mudado sessenta vezes por segundo,
    // vezes o número de operandos, para não mudar nada.
    if let Some(op) = pose.bool_group_op
        && let Some((group, current)) = crate::bool_live::group_above(sim, map, pose.id)
        && current != op
    {
        sim.world_mut()
            .entity_mut(group)
            .insert(ph2d_ecs::VecBoolGroup { op });
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
    let h = host_of_selection(sim, scene, map, selected)?;
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

/// **O gesto de tabela que um id de painel endereça**, se ele endereçar algum.
///
/// ⚠️ **Uma porta só para os quatro gestos**, e não quatro varreduras espalhadas pelo roteador da
/// shell: os ids são hashes derivados do índice, então quem os inverte tem de conhecer o mesmo
/// teto que o `populate` regista e o `paint` percorre. Uma segunda varredura escrita noutro
/// arquivo é a que esquece o `MAX` quando ele se mover.
#[must_use]
pub(crate) fn signal_edit_for_id(id: ph2d_editor::NodeId) -> Option<SignalEdit> {
    if id == ph2d_editor::ids::VECTOR_STATE_SIGNAL_ADD {
        return Some(SignalEdit::Add);
    }
    for i in 0..ph2d_editor::ids::MAX_SIGNAL_BINDINGS {
        if id == ph2d_editor::ids::vector_state_signal_remove_id(i) {
            return Some(SignalEdit::Remove(i));
        }
        for (r, role) in StateRole::ALL.iter().enumerate() {
            if id == ph2d_editor::ids::vector_state_signal_role_id(i, r) {
                return Some(SignalEdit::Role(i, *role));
            }
        }
    }
    None
}

/// O que um gesto da tabela sinal → papel pede.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SignalEdit {
    /// Acrescenta uma ligação vazia.
    Add,
    /// Apaga a ligação `i`.
    Remove(usize),
    /// Re-aponta a ligação `i` para outro papel.
    Role(usize, StateRole),
}

/// **Aplica um gesto da tabela** ao hospedeiro selecionado.
///
/// ⚠️ **Ele exige hospedeiro ÚNICO**, a mesma guarda da duração e da curva: a tabela é por
/// hospedeiro, e carimbar a mesma ligação em vários seria um gesto cujo alcance o artista não vê.
pub(crate) fn apply_signal_edit(
    sim: &SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    states: &mut StateSets,
    selected: &[VecPathId],
    edit: SignalEdit,
) -> bool {
    let Some(h) = host_of_selection(sim, scene, map, selected) else {
        return false;
    };
    match edit {
        // ⚠️ **O teto é do PAINEL e a guarda mora aqui também**, e não é redundância: o painel
        // esconde o botão no teto (a metade visível), e a porta o honra (a metade que decide).
        // Sem esta, um clique que chegasse por outra rota cresceria a lista além do que a UI
        // sabe mostrar — e as linhas extra ficariam invisíveis no documento.
        SignalEdit::Add => {
            if states.bindings(h).len() >= ph2d_editor::ids::MAX_SIGNAL_BINDINGS {
                return false;
            }
            states.push_binding(h);
        }
        SignalEdit::Remove(i) => states.remove_binding(h, i),
        SignalEdit::Role(i, role) => states.set_binding_role(h, i, role),
    }
    true
}

/// **O nome commitado num campo da tabela** — o índice da linha, se o id for de uma.
#[must_use]
pub(crate) fn signal_name_row(id: ph2d_editor::NodeId) -> Option<usize> {
    (0..ph2d_editor::ids::MAX_SIGNAL_BINDINGS)
        .find(|&i| ph2d_editor::ids::vector_state_signal_name_id(i) == id)
}

/// **O HOSPEDEIRO e a PROJEÇÃO** — irmão por LOC (HR-18), cortado por responsabilidade.
#[path = "vec_ui_state_host.rs"]
mod host;
pub(crate) use host::{host_of_selection, publish};

#[cfg(test)]
#[path = "vec_ui_state_edit_tests.rs"]
mod tests;

/// Os gates do canal de FILTROS na pose — irmão por LOC (HR-18), com fixture própria.
#[cfg(test)]
#[path = "vec_ui_state_edit_filter_tests.rs"]
mod filter_tests;

#[cfg(test)]
#[path = "vec_ui_state_signal_tests.rs"]
mod signal_tests;

/// Os gates dos dois canais da BOOLEANA VIVA na pose — irmão por LOC (HR-18), com fixture própria
/// (uma cena SEM grupo booleano não teria como afirmar nada sobre eles).
#[cfg(test)]
#[path = "vec_ui_state_edit_bool_tests.rs"]
mod bool_tests;
