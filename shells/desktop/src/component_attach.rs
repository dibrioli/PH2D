//! ⭐ **As duas pontas do `+` do Inspector** (ADR-0166 / plano F3): abrir a paleta para quem
//! pediu, e anexar o componente que ela escolheu.
//!
//! Irmã por ASSUNTO da [`crate::component_palette`] (que constrói o modelo), como a
//! `motion_bridge_library` é irmã do `motion_bridge`: **o modelo é uma função pura e testável;
//! isto é o que toca no mundo.**

use ph2d_ecs::SimWorld;
use ph2d_ecs::scene::ComponentRegistry;
use ph2d_editor::{HeroScreen, Toast};

/// **Que TIPO de objeto é este** — lido por PRESENÇA de um marcador, nunca por um campo
/// (ADR-0166 / o `ObjectKind` da F0).
///
/// ⚠️ A ordem importa: um objeto pode carregar mais de um marcador em teoria, e o primeiro que
/// casar manda. Hoje eles são exclusivos na prática; a ordem torna a resposta **determinística**
/// mesmo que deixem de ser.
fn kind_of(world: &ph2d_ecs::World, entity: ph2d_ecs::Entity) -> ph2d_component_desc::ObjectKind {
    use ph2d_component_desc::ObjectKind;
    if world.get::<ph2d_render::Sprite>(entity).is_some() {
        ObjectKind::Image
    } else if world.get::<ph2d_ecs::VecPathRef>(entity).is_some() {
        ObjectKind::Vector
    } else if world.get::<ph2d_field_ecs::FieldObject>(entity).is_some() {
        ObjectKind::Model3D
    } else {
        ObjectKind::Empty
    }
}

/// Os nomes canónicos que este objeto **já tem** — o que a paleta não pode voltar a oferecer.
fn present_on(
    world: &ph2d_ecs::World,
    entity: ph2d_ecs::Entity,
    registry: &ComponentRegistry,
) -> Vec<&'static str> {
    registry
        .iter()
        .filter(|e| matches!((e.serialize)(world, entity), Ok(Some(_))))
        .map(|e| e.canonical_name)
        .collect()
}

/// **Abre a paleta** para o objeto que pediu, se algum pediu.
pub(crate) fn open_palette_if_asked(
    hero: &mut HeroScreen,
    sim: &SimWorld,
    registry: &ComponentRegistry,
    asked_for: Option<u64>,
    // ⚠️ **Quem pediu, LEMBRADO** — e não re-derivado da seleção no momento do pick. O modal tem
    // scrim, então hoje a seleção não pode mudar entre abrir e escolher; mas *"não pode mudar"* é
    // um invariante que ninguém impõe, e o precedente da casa (a biblioteca do Motion guarda o
    // `library_open`) é guardar o contexto. Um dia em que o scrim mude, isto continua certo.
    target: &mut Option<u64>,
) {
    let Some(bits) = asked_for else {
        return;
    };
    let Some(entity) = ph2d_ecs::Entity::try_from_bits(bits) else {
        return;
    };
    let world = sim.world();
    if world.get_entity(entity).is_err() {
        return;
    }
    // ⚠️ **A paleta abre com o *Show all* DESLIGADO**, sempre: o inaplicável é contexto, e um
    // artista que a abre para pôr um componente quer ver o que serve. O estado da caixa não
    // sobrevive ao fecho de propósito.
    hero.store
        .open_command_palette(model_for(sim, registry, bits, false));
    *target = Some(bits);
}

/// O modelo da paleta para `bits`, com a caixa no estado `show_all`.
///
/// ⚠️ **Uma porta só**, chamada ao abrir **e** ao virar a caixa: duas construções do modelo seriam
/// duas respostas a *"o que este objeto pode receber?"*, e a que o artista vê depois de clicar na
/// caixa seria a que envelhece.
fn model_for(
    sim: &SimWorld,
    registry: &ComponentRegistry,
    bits: u64,
    show_all: bool,
) -> ph2d_editor::widget::command_palette::PaletteModel {
    let world = sim.world();
    let entity = ph2d_ecs::Entity::from_bits(bits);
    let kind = kind_of(world, entity);
    let present = present_on(world, entity, registry);
    // ⚠️ **O que o registo sabe CONSTRUIR** — sem `insert_default` a paleta não tem valor inicial, e
    // um item que aceita o clique e não anexa nada é o defeito que o `+` existe para não ter. É o
    // registo que responde, não uma lista à mão.
    let can_build = |name: &str| {
        registry
            .get_by_id(ph2d_ecs::scene::stable_type_id(name))
            .is_some_and(|e| e.insert_default.is_some())
    };
    crate::component_palette::build(kind, &present, &can_build, show_all)
}

/// ⭐ **A caixa *Show all* foi virada ⇒ reconstrói o modelo** (ADR-0166 / F3).
///
/// ⚠️ **O widget vira o estado dele e AVISA; quem reconstrói é quem abriu** — o `command_palette` é
/// genérico e não sabe o que *"mostrar tudo"* significa, exatamente como não sabe o que um item
/// significa. É a mesma costura do `command_pick`, um controlo mais acima.
///
/// ⚠️ **A busca sobrevive** (`set_command_palette_model`, e não `open_command_palette`): ligar a
/// caixa depois de escrever «rigid» não pode apagar o que foi escrito.
pub(crate) fn refresh_palette_on_toggle(
    hero: &mut HeroScreen,
    sim: &SimWorld,
    registry: &ComponentRegistry,
    target: Option<u64>,
) {
    if !hero.store.take_command_palette_toggled() {
        return;
    }
    let Some(bits) = target else {
        return;
    };
    let show_all = hero.store.command_palette_toggle_on();
    hero.store
        .set_command_palette_model(model_for(sim, registry, bits, show_all));
}

/// **Drena o pick da paleta** e devolve `(entidade, nome canónico)`.
///
/// ⚠️ **O dreno é CONDICIONAL** (`take_command_pick_if`), e é o que torna a ordem dos drenos
/// irrelevante: este canal já tinha DOIS consumidores (a biblioteca do Motion e o `Ctrl+K`
/// global), e um `take` incondicional engoliria o pick de outro e devolveria `None` a quem o
/// soubesse executar — com o sintoma a ser *«às vezes não faz nada»*.
#[must_use]
pub(crate) fn route_pick(hero: &mut HeroScreen, target: &mut Option<u64>) -> Option<(u64, String)> {
    let bits = (*target)?;
    let id = hero
        .store
        .take_command_pick_if(|id| crate::component_palette::name_of_pick(id).is_some())?;
    let name = crate::component_palette::name_of_pick(id)?;
    *target = None;
    Some((bits, name.to_string()))
}

/// **Anexa o que a paleta escolheu**, no ponto NEUTRO do tipo.
///
/// ⚠️ **Escreve DIRETO no mundo, e não pelo `EditorCommandQueue`** — e a razão é o que o
/// `insert_default` é: um construtor type-erased (`fn(&mut World, Entity)`), não bytes. Pô-lo na
/// fila exigiria serializar o `Default` só para o desserializar do outro lado, e o registo não
/// expõe esses bytes.
///
/// ⚠️ **O desfazer não depende da fila.** Ele nasce do DIFF do fim do quadro
/// (`App::post_frame_undo`), então uma escrita direta é capturada como qualquer outra — e anexar
/// é INERTE por construção (o ponto neutro do tipo), o que torna o passo um `add component` puro.
pub(crate) fn attach_picked(
    picks: Option<&(u64, String)>,
    sim: &mut SimWorld,
    registry: &ComponentRegistry,
    toasts: &mut ph2d_editor::ToastQueue,
) {
    let Some((bits, name)) = picks else {
        return;
    };
    if let Err(msg) = attach_by_name(sim, registry, *bits, name) {
        toasts.push(Toast::error(msg));
    }
}

/// ⭐ **A PORTA de anexar um componente pelo nome canónico** — o ponto neutro do tipo, e depois o
/// seed. Devolve a mensagem de erro quando não dá.
///
/// ⚠️ **É `pub(crate)` de propósito:** os gates que provam *"o gesto inteiro chega a algum lugar"*
/// têm de atravessar **esta** função, e não encenar um `world.insert()` à mão — foi assim que a
/// autoria de física por gestos passou a ser medida depois de a face vazia da §11 morrer.
pub(crate) fn attach_by_name(
    sim: &mut SimWorld,
    registry: &ComponentRegistry,
    entity_bits: u64,
    name: &str,
) -> Result<(), String> {
    // ⭐ **A CASCATA primeiro** (ADR-0166 / F3): o que este componente não funciona sem.
    //
    // ⚠️ **Antes dele, e não depois** — o seed de um dependente pode ler o que a dependência traz
    // (o `PlatformPlayer` mede o `Collider` para saber a que altura pairar). Na ordem inversa ele
    // leria um mundo sem collider e nasceria tangente.
    //
    // ⚠️ **A recursão termina** porque `attach_one` é no-op sobre o que já está lá, e o gate
    // `the_require_graph_has_no_cycles` afirma que o grafo é acíclico — um ciclo no catálogo faria
    // isto recorrer para sempre em vez de falhar alto.
    if let Some(desc) = ph2d_component_desc::desc_for(name) {
        for dep in desc.requires {
            attach_by_name(sim, registry, entity_bits, dep)?;
        }
    }
    attach_one(sim, registry, entity_bits, name)
}

/// Anexa **um** componente (sem a cascata), e é no-op quando ele já está lá.
fn attach_one(
    sim: &mut SimWorld,
    registry: &ComponentRegistry,
    entity_bits: u64,
    name: &str,
) -> Result<(), String> {
    let entity =
        ph2d_ecs::Entity::try_from_bits(entity_bits).ok_or_else(|| "No such entity".to_string())?;
    let type_id = ph2d_ecs::scene::stable_type_id(name);
    let entry = registry
        .get_by_id(type_id)
        .ok_or_else(|| format!("Unknown component: {name}"))?;
    // ⚠️ Inalcançável pela paleta (ela só oferece o que se constrói), e por isso mesmo vale uma
    // mensagem em vez de um `return` mudo: chegar aqui significa que a paleta e o registo
    // discordam, e isso é um defeito de programa.
    let insert = entry
        .insert_default
        .ok_or_else(|| format!("{name} has no default to attach"))?;
    // ⚠️ **Já lá está ⇒ NO-OP.** É o que faz a cascata terminar e o que impede que anexar
    // *Platform Player* num corpo que já existe rebaixe esse corpo ao ponto neutro — o `insert` do
    // bevy substitui, não funde.
    if matches!((entry.serialize)(sim.world(), entity), Ok(Some(_))) {
        return Ok(());
    }
    insert(sim.world_mut(), entity).map_err(|e| format!("Attach failed: {e}"))?;
    // ⭐ **E o SEED, se este componente tiver um** (ADR-0166 / F3 · a emenda medida na F0).
    //
    // ⚠️ **Depois do `insert_default`, nunca em vez dele:** o valor gravado continua a ser *o ponto
    // neutro do tipo, corrigido pelo contexto*. Uma construção alternativa aqui seria a segunda
    // porta que apodrece quando o tipo ganha campos. Ver [`crate::component_seed`].
    crate::component_seed::seed_after_attach(sim, entity_bits, name);
    Ok(())
}

/// ⭐ **A roda sobre a paleta aberta é DELA** (F3 / ADR-0166) — devolve `true` quando consumiu.
///
/// ⚠️ **Consome enquanto ela estiver ABERTA, e não «quando o cursor está sobre o cartão»**: ao
/// contrário da janela do Input Map, esta é um modal de **tela cheia com scrim** — não há «por
/// fora dela». Uma roda que atravessasse o scrim daria zoom no canvas escurecido por baixo, que é
/// o gesto errado com a mão no sítio certo.
///
/// ⚠️ **O sinal é INVERTIDO** (`-dy`), como no Input Map: a roda para cima sobe a lista, isto é
/// DIMINUI o deslocamento.
pub(crate) fn palette_wheel(
    hero: &mut HeroScreen,
    text_system: &mut ph2d_text::TextSystem,
    viewport: ph2d_editor::zones::Rect,
    dy: f32,
) -> bool {
    if !hero.store.command_palette_open() {
        return false;
    }
    let max = ph2d_editor::screens::hero::chrome::command_palette_max_scroll(
        text_system,
        &hero.store,
        viewport,
    );
    hero.store.scroll_command_palette(-dy, max);
    true
}

/// ⭐ A SEQUÊNCIA — ver [`crate::component_attach_tests`].
#[cfg(test)]
#[path = "component_attach_tests.rs"]
mod tests;
