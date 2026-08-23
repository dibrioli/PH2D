//! **§11 Animation** (spec Sprite 08) — o snapshot que a seção lê e o commit que ela escreve.
//! Irmão do [`super::inspector_anchor`], pela mesma razão dele.
//!
//! # ⚠️ A conversão da VELOCIDADE mora aqui, nas duas pontas
//!
//! O motor guarda `Q16.16` (`65536` = `1,0×`) porque a multiplicação inteira é bit-idêntica entre
//! sistemas; o artista escreve `1,5`. O snapshot divide na saída e o commit multiplica na entrada,
//! **no mesmo ficheiro** — duplicar a conversão faria o número que ele lê e o que o motor aplica
//! divergirem no dia em que uma metade fosse corrigida. É a mesma lei da §12 com os pixels.

use ph2d_ecs::scene::{ComponentRegistry, EditorCommandQueue};
use ph2d_ecs::{
    AnimDirection, AnimationTag, Entity, SPEED_MAX_Q16, SPEED_ONE_Q16, SimWorld, SpriteAnimations,
    SpriteAnimator, World,
};
use ph2d_editor::{AnimFieldEdit, InspectorAnimInfo, InspectorAnimRow, Toast};

use super::inspector_ordering::queue_set;

const ANIMATIONS: &str = "ph2d::ecs::SpriteAnimations";
const ANIMATOR: &str = "ph2d::ecs::SpriteAnimator";
/// ⚠️ **O único sink do índice de célula** — ver o doc de [`ph2d_ecs::sprite_anim`].
const SPRITE: &str = "ph2d::render::Sprite";

/// A tag que este tocador aponta, se ela existir na biblioteca de hoje.
fn current_tag(world: &World, entity: Entity, name: &str) -> Option<AnimationTag> {
    world
        .get::<SpriteAnimations>(entity)
        .and_then(|l| l.get(name))
        .cloned()
}

/// **REBOBINAR é repor o ciclo E pôr a IMAGEM no princípio** — as duas coisas, sempre.
///
/// ⚠️ Enquanto isto era só o [`SpriteAnimator::rewind`], o botão *Rewind* repunha contadores que
/// ninguém vê e deixava a sprite na célula onde tinha parado: o artista carregava em «rebobinar» e
/// **nada se mexia**. E com um `repeat` finito o gesto era pior que inerte — a imagem ficava na
/// ponta do intervalo, então o primeiro passo de [`ph2d_ecs::advance`] fechava logo o ciclo e
/// parava outra vez.
///
/// ⚠️ **Trocar de animação passa por aqui pela MESMA razão, e ela não é teórica:** as animações
/// desta engine partilham o pool de células (é a tese do modelo), então uma `walk` a correr na
/// célula 2 e uma `idle` de 0-3 **sobrepõem-se** — o `advance` só reposiciona o que cai FORA do
/// intervalo, e por isso escolher a `idle` começava-a a meio.
///
/// ⚠️ Escreve o `Sprite` **só quando o índice muda**: ele é `SimComponent` e o undo regista por
/// diff.
fn rewind_to_start(
    world: &World,
    entity_bits: u64,
    player: &mut SpriteAnimator,
    tag: Option<&AnimationTag>,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
) {
    player.rewind(tag);
    let entity = Entity::from_bits(entity_bits);
    let Some(tag) = tag else { return };
    let Some(mut sprite) = world.get::<ph2d_render::Sprite>(entity).cloned() else {
        return;
    };
    let Some(frame) = ph2d_ecs::entry_frame(player, tag, cells_of(world, entity)) else {
        return;
    };
    if sprite.frame == frame {
        return;
    }
    sprite.frame = frame;
    queue_set(queue, registry, entity_bits, SPRITE, &sprite);
}

/// **Põe a cabeça de leitura numa célula**, fixando-a ao intervalo da animação em curso.
///
/// ⚠️ **O clamp é do COMMIT, e não do painel.** O painel deriva a célula do curso do slider contra
/// o `progress()` do snapshot — que é de um quadro atrás. Entre o gesto e o commit alguém pode ter
/// encolhido a grelha ou estreitado o intervalo, e uma célula fora dele fica **inalcançável** pelo
/// `advance` (que só reposiciona o que já está fora, e à ponta de entrada — desfazendo o arrasto).
fn set_frame(
    world: &World,
    entity_bits: u64,
    player: &SpriteAnimator,
    cell: u32,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
) {
    let entity = Entity::from_bits(entity_bits);
    let Some(mut sprite) = world.get::<ph2d_render::Sprite>(entity).cloned() else {
        return;
    };
    let cells = cells_of(world, entity);
    // Sem tag que resolva, o pool inteiro é o alcance — é o que a spec §8.9 descreve para uma
    // sprite sem animação escolhida.
    let (lo, hi) = current_tag(world, entity, &player.current)
        .and_then(|t| t.resolve(cells))
        .unwrap_or((0, cells.saturating_sub(1)));
    let frame = cell.clamp(lo, hi);
    if sprite.frame == frame {
        return;
    }
    sprite.frame = frame;
    queue_set(queue, registry, entity_bits, SPRITE, &sprite);
}

/// Quantas células a grelha desta sprite tem — **o pool**, lido do `Sprite` e de mais nada.
fn cells_of(world: &World, entity: Entity) -> u32 {
    world
        .get::<ph2d_render::Sprite>(entity)
        .map_or(1, |s| s.hframes.saturating_mul(s.vframes).max(1))
}

/// Constrói o snapshot da §11, ou `None` quando a entidade não é uma sprite.
///
/// ⚠️ **Não há campo `mixed`, e a ausência é MEDIDA (2026-08-23).** Ele existia, era calculado a
/// cada quadro — clonando a biblioteca da primária **e** a de cada entidade selecionada — e
/// **nenhum pintor o lia**. O que a seleção múltipla precisa de dizer não é *«elas discordam»* e
/// sim *«isto pega só na ativa»*, que é sempre verdade aqui e se responde com o `selected_count`.
/// *Um facto caro que ninguém lê é pior que um facto ausente: ele parece cobertura.*
pub(super) fn build_anim_info(
    world: &World,
    entity_bits: u64,
    selected_count: usize,
) -> Option<InspectorAnimInfo> {
    let entity = Entity::from_bits(entity_bits);
    // ⚠️ **Só uma SPRITE tem animação de frames**, porque o pool é a grelha dela. Numa entidade
    // sem `Sprite`, a seção inteira não se pinta — em vez de oferecer knobs sobre um pool vazio.
    let sprite = world.get::<ph2d_render::Sprite>(entity)?;
    let cells = sprite.hframes.saturating_mul(sprite.vframes).max(1);
    let lib = world.get::<SpriteAnimations>(entity);
    let rows: Vec<InspectorAnimRow> = lib
        .map(|l| {
            l.iter()
                .map(|t| InspectorAnimRow {
                    name: t.name.clone(),
                    from: t.from,
                    to: t.to,
                    frame_ms: t.frame_ms,
                    direction_tag: t.direction.tag(),
                    // `None` (repetir para sempre) viaja como `0` — é o que o campo mostra, e o
                    // rótulo dele di-lo.
                    repeat: t.repeat.unwrap_or(0),
                    hold_ms: t.hold_ms,
                    repeat_delay_ms: t.repeat_delay_ms,
                    signal_on_finish: t.signal_on_finish.clone(),
                    signal_on_loop: t.signal_on_loop.clone(),
                })
                .collect()
        })
        .unwrap_or_default();
    let player = world.get::<SpriteAnimator>(entity);
    Some(InspectorAnimInfo {
        entity_bits,
        rows,
        player_present: player.is_some(),
        cells,
        current: player.map(|p| p.current.clone()).unwrap_or_default(),
        playing: player.is_some_and(|p| p.playing),
        autoplay: player.is_some_and(|p| p.autoplay),
        speed: player.map_or(1.0, |p| p.speed_q16 as f32 / SPEED_ONE_Q16 as f32),
        direction_override_tag: player
            .and_then(|p| p.direction_override)
            .map_or(0, |d| d.tag() + 1),
        loop_override_tag: player.and_then(|p| p.loop_override).map_or(
            0,
            |on| {
                if on { 1 } else { 2 }
            },
        ),
        frame: sprite.frame,
        selected_count,
    })
}

/// Aplica uma [`AnimFieldEdit`]. Devolve um aviso quando a edição foi **recusada**.
pub(super) fn apply_anim_edit(
    sim: &SimWorld,
    entity_bits: u64,
    edit: &AnimFieldEdit,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
) -> Option<Toast> {
    let entity = Entity::from_bits(entity_bits);
    let world = sim.world();

    // ── O TOCADOR ────────────────────────────────────────────────────────────────────────────
    //
    // ⚠️ **Ler-modificar-escrever**, como a lista: mexer numa caixa não pode repor as outras.
    if let Some(player_edit) = as_player_edit(edit) {
        let mut p = world
            .get::<SpriteAnimator>(entity)
            .cloned()
            .unwrap_or_default();
        if p.speed_q16 == 0 && !matches!(edit, AnimFieldEdit::Speed(_)) {
            // Um animador acabado de anexar nasce com velocidade zero (o `Default`), e um
            // `Play` sobre ele não moveria nada. A velocidade normal é o estado de partida.
            p.speed_q16 = SPEED_ONE_Q16;
        }
        match player_edit {
            PlayerEdit::Add => {}
            PlayerEdit::SetCurrent(name) => {
                // ⚠️ **A reprodução que ACABOU volta a tocar quando alguém lhe toca — e escolher
                // outra animação é tocar-lhe.** É a MESMA lei do braço `Playing` logo abaixo, pela
                // outra porta: quem parou foi o fim do ciclo, não o artista, e ele nunca pediu
                // silêncio. Sem isto, a sequência que a cena de smoke descreve — *tocar `attack`
                // uma vez, depois clicar em `walk` para voltar ao ciclo* — deixava a sprite parada,
                // e o artista ia procurar a caixa de `Playing` para reparar um estado que ele não
                // criou (Enio, 2026-08-23).
                //
                // ⚠️ **Uma PAUSA explícita não é tocada por isto**: `is_finished` é falso a meio de
                // um ciclo, então quem desmarcou `Playing` continua a poder folhear a lista em
                // silêncio. *Escolher uma animação nunca PÁRA nada, e só retoma o que se esgotou.*
                if !p.playing
                    && current_tag(world, entity, &p.current).is_some_and(|t| p.is_finished(&t))
                {
                    p.playing = true;
                }
                p.current = name.clone();
                let tag = current_tag(world, entity, name);
                // ⚠️ **Trocar de animação REBOBINA.** O `repeat_count` e o ping-pong são do ciclo
                // que estava a correr; carregá-los para a nova faria a primeira volta dela
                // começar a meio, ou nem acontecer.
                rewind_to_start(world, entity_bits, &mut p, tag.as_ref(), queue, registry);
            }
            // ⚠️ **LIGAR uma animação que já ACABOU é pedir para a rever, e sem isto o gesto é
            // MORTO.** «Pausado» e «terminado» leem-se igual no `playing == false`, e são coisas
            // diferentes: numa que já gastou os ciclos, pôr `playing = true` deixa a imagem na
            // ponta do intervalo com o contador cheio, e o primeiro passo de `advance` fecha o
            // ciclo outra vez e volta a parar — no MESMO tique. O artista marca a caixa, ela fica
            // marcada por um quadro, e nada toca.
            //
            // ⇒ Ligar sobre uma reprodução terminada rebobina-a. Quem pausou a meio continua de
            // onde estava, que é a outra metade da promessa.
            PlayerEdit::Playing(on) => {
                p.playing = *on;
                if *on {
                    let tag = current_tag(world, entity, &p.current);
                    if tag.as_ref().is_some_and(|t| p.is_finished(t)) {
                        rewind_to_start(world, entity_bits, &mut p, tag.as_ref(), queue, registry);
                    }
                }
            }
            PlayerEdit::Autoplay(on) => p.autoplay = *on,
            PlayerEdit::Speed(v) => {
                p.speed_q16 = (v * SPEED_ONE_Q16 as f32) as i32;
                p.speed_q16 = p.speed_q16.clamp(-SPEED_MAX_Q16, SPEED_MAX_Q16);
            }
            PlayerEdit::DirectionOverride(tag) => {
                p.direction_override = (*tag > 0).then(|| AnimDirection::from_tag(tag - 1));
            }
            PlayerEdit::LoopOverride(tag) => {
                p.loop_override = match tag {
                    1 => Some(true),
                    2 => Some(false),
                    _ => None,
                };
            }
            PlayerEdit::Rewind => {
                let tag = current_tag(world, entity, &p.current);
                rewind_to_start(world, entity_bits, &mut p, tag.as_ref(), queue, registry);
            }
            // ⚠️ **AGARRAR A BARRA PAUSA**, e a pausa é o gesto, não um efeito colateral: enquanto
            // a reprodução corre, o tique também escreve o `Sprite::frame`, e o dedo e o relógio
            // disputariam o mesmo campo — a imagem piscaria entre a célula arrastada e a que o
            // relógio acabou de pôr. *Quem pega no volante conduz*, e a caixa `Playing` di-lo.
            //
            // ⚠️ **Não rebobina o ciclo.** O `repeat_count` e o ping-pong são de onde a reprodução
            // ia; largar a barra e voltar a `Playing` continua a volta em curso a partir da célula
            // escolhida, que é o que «arrastar até ali e seguir» quer dizer.
            PlayerEdit::SetFrame(cell) => {
                p.playing = false;
                // O acumulador do frame anterior não pertence a este; deixá-lo faria a primeira
                // célula depois do arrasto durar menos do que as outras.
                p.elapsed_ticks = 0;
                set_frame(world, entity_bits, &p, *cell, queue, registry);
            }
        }
        queue_set(queue, registry, entity_bits, ANIMATOR, &p);
        return None;
    }

    // ── A BIBLIOTECA ─────────────────────────────────────────────────────────────────────────
    let mut lib = world
        .get::<SpriteAnimations>(entity)
        .cloned()
        .unwrap_or_default();
    match edit {
        AnimFieldEdit::Add => {
            let name = lib.next_free_name();
            // A animação nova cobre a grelha inteira: é o que o artista vê e o que ele quase
            // sempre quer estreitar depois. Um intervalo de uma célula não mostraria nada.
            let cells = cells_of(world, entity);
            if let Err(e) = lib.insert(AnimationTag::new(name, 0, cells.saturating_sub(1))) {
                return Some(Toast::error(format!(
                    "Animation not added: {}",
                    describe(e)
                )));
            }
        }
        AnimFieldEdit::Remove(i) => {
            let name = lib.iter().nth(usize::from(*i)).map(|t| t.name.clone())?;
            lib.remove(&name);
        }
        AnimFieldEdit::Rename(i, new_name) => {
            let idx = usize::from(*i);
            lib.iter().nth(idx)?;
            if let Err(e) = ph2d_ecs::validate_tag_name(new_name) {
                return Some(Toast::error(format!(
                    "Animation name rejected: {}",
                    describe(e)
                )));
            }
            if lib
                .iter()
                .enumerate()
                .any(|(j, t)| j != idx && t.name == *new_name)
            {
                return Some(Toast::error(format!(
                    "Animation name '{new_name}' is already used on this sprite"
                )));
            }
            let old = lib.0.get(idx).map(|t| t.name.clone());
            if let Some(t) = lib.0.get_mut(idx) {
                t.name = new_name.clone();
            }
            // ⚠️ **Renomear a animação que TOCA leva o tocador junto.** Sem isto, mudar o nome
            // deixaria o `current` a apontar para uma animação que já não existe — o artista
            // corrigiria uma letra e veria a sprite parar, sem ligar as duas coisas.
            if let Some(old) = old
                && let Some(mut p) = world.get::<SpriteAnimator>(entity).cloned()
                && p.current == old
            {
                p.current = new_name.clone();
                queue_set(queue, registry, entity_bits, ANIMATOR, &p);
            }
        }
        other => {
            let idx = match other {
                AnimFieldEdit::From(i, _)
                | AnimFieldEdit::To(i, _)
                | AnimFieldEdit::FrameMs(i, _)
                | AnimFieldEdit::HoldMs(i, _)
                | AnimFieldEdit::DelayMs(i, _)
                | AnimFieldEdit::Repeat(i, _)
                | AnimFieldEdit::Direction(i, _)
                | AnimFieldEdit::SignalOnFinish(i, _)
                | AnimFieldEdit::SignalOnLoop(i, _) => usize::from(*i),
                // Tratados acima; o braço existe para o `match` não precisar de curinga, que
                // engoliria em silêncio a próxima variante.
                _ => return None,
            };
            write_tag(lib.0.get_mut(idx)?, other);
        }
    }
    queue_set(queue, registry, entity_bits, ANIMATIONS, &lib);
    None
}

/// Escreve um campo de UMA animação. Separado para manter o `match` de cima legível.
fn write_tag(t: &mut AnimationTag, edit: &AnimFieldEdit) {
    match edit {
        AnimFieldEdit::From(_, v) => t.from = *v,
        AnimFieldEdit::To(_, v) => t.to = *v,
        // ⚠️ O cap entra na ESCRITA e a lei volta a impô-lo na leitura: a porta cobre o gesto,
        // e a leitura cobre um ficheiro adulterado.
        AnimFieldEdit::FrameMs(_, v) => {
            t.frame_ms = (*v).clamp(ph2d_ecs::FRAME_MS_MIN, ph2d_ecs::FRAME_MS_MAX);
        }
        AnimFieldEdit::HoldMs(_, v) => t.hold_ms = *v,
        AnimFieldEdit::DelayMs(_, v) => t.repeat_delay_ms = *v,
        // `0` no campo = repetir para sempre. O rótulo do campo di-lo.
        AnimFieldEdit::Repeat(_, v) => t.repeat = (*v > 0).then_some(*v),
        AnimFieldEdit::Direction(_, tag) => t.direction = AnimDirection::from_tag(*tag),
        // ⚠️ **APARADO e CAPADO na porta**, e o cap é o mesmo do nome de uma tag: um nome de sinal
        // vem do teclado, viaja num componente registado e entra no ficheiro do projeto. Espaço à
        // volta é invisível no campo e faz o contrato **não casar** do outro lado.
        AnimFieldEdit::SignalOnFinish(_, v) => t.signal_on_finish = clean_signal_name(v),
        AnimFieldEdit::SignalOnLoop(_, v) => t.signal_on_loop = clean_signal_name(v),
        _ => {}
    }
}

/// As edições que falam do TOCADOR — as que não carregam índice de animação.
enum PlayerEdit<'a> {
    Add,
    SetCurrent(&'a String),
    Playing(&'a bool),
    Autoplay(&'a bool),
    Speed(&'a f32),
    DirectionOverride(&'a u8),
    LoopOverride(&'a u8),
    Rewind,
    SetFrame(&'a u32),
}

fn as_player_edit(edit: &AnimFieldEdit) -> Option<PlayerEdit<'_>> {
    Some(match edit {
        AnimFieldEdit::AddPlayer => PlayerEdit::Add,
        AnimFieldEdit::SetCurrent(n) => PlayerEdit::SetCurrent(n),
        AnimFieldEdit::Playing(v) => PlayerEdit::Playing(v),
        AnimFieldEdit::Autoplay(v) => PlayerEdit::Autoplay(v),
        AnimFieldEdit::Speed(v) => PlayerEdit::Speed(v),
        AnimFieldEdit::DirectionOverride(v) => PlayerEdit::DirectionOverride(v),
        AnimFieldEdit::LoopOverride(v) => PlayerEdit::LoopOverride(v),
        AnimFieldEdit::Rewind => PlayerEdit::Rewind,
        AnimFieldEdit::SetFrame(v) => PlayerEdit::SetFrame(v),
        _ => return None,
    })
}

/// A recusa, em palavras que o artista entende — nunca o nome da variante. ⚠️ Cada braço nomeia
/// o SEU teto, pela lição que o `describe` da §12 pagou.
fn describe(e: ph2d_ecs::AnimTagError) -> String {
    match e {
        ph2d_ecs::AnimTagError::Empty => "the name is empty".to_string(),
        ph2d_ecs::AnimTagError::TooLong => {
            format!("the name is over {} bytes", ph2d_ecs::ANIM_NAME_MAX_BYTES)
        }
        ph2d_ecs::AnimTagError::ControlChar => "the name has a control character".to_string(),
        ph2d_ecs::AnimTagError::Duplicate => "that name is already used".to_string(),
        ph2d_ecs::AnimTagError::ListFull => {
            format!(
                "this sprite already has {} animations",
                ph2d_ecs::ANIM_TAGS_MAX
            )
        }
    }
}

#[cfg(test)]
#[path = "inspector_anim_tests.rs"]
mod tests;

/// **O nome de um sinal, como ele fica guardado.** Aparado e capado no mesmo teto do nome de uma
/// tag (`ph2d_ecs::ANIM_NAME_MAX_BYTES`).
///
/// ⚠️ **Aparar não é higiene:** o contrato de um sinal é a **igualdade de string** (ADR-0143), e
/// um espaço à direita é invisível no campo e faz o consumidor não casar — o pior tipo de defeito,
/// porque tudo parece certo dos dois lados.
///
/// ⚠️ **O corte é por BYTES e respeita o caractere**: `String::truncate` entra em pânico a meio de
/// um multibyte, e um nome com acento é normal.
fn clean_signal_name(raw: &str) -> String {
    let t = raw.trim();
    match t.char_indices().nth_back(0) {
        _ if t.len() <= ph2d_ecs::ANIM_NAME_MAX_BYTES => t.to_owned(),
        _ => {
            let mut end = ph2d_ecs::ANIM_NAME_MAX_BYTES;
            while end > 0 && !t.is_char_boundary(end) {
                end -= 1;
            }
            t[..end].to_owned()
        }
    }
}
