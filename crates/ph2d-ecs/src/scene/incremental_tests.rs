//! Os gates da captura incremental (ADR-0164 §2.7 / plano F2) — **um por condição**, e cada um
//! com a mutação que o mata escrita no doc.
//!
//! ⚠️ O gate que manda é o da **EQUIVALÊNCIA**: a captura incremental tem de dar, byte a byte, o
//! mesmo que um rebuild completo. Os outros medem *porquê* ela dá — e sem eles um rebuild
//! disfarçado de incremental passaria em todos.

use super::*;
use crate::scene::registry::register_ecs_components;
use crate::scene::save::world_to_snapshot;
use crate::{Name, RootOrder, Transform, TransformPropagationState, Visibility, WorklistBuf};
use bevy_ecs::hierarchy::ChildOf;

fn registry() -> ComponentRegistry {
    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);
    reg
}

/// Um mundo com `n` objectos-raiz nomeados.
fn world_with(n: u32) -> (World, Vec<Entity>) {
    let mut w = World::new();
    let es = (0..n)
        .map(|i| {
            w.spawn((
                Transform::IDENTITY,
                Name::new(format!("obj{i}")),
                RootOrder(i),
            ))
            .id()
        })
        .collect();
    (w, es)
}

fn capture(
    w: &mut World,
    c: &mut CaptureCache,
    reg: &ComponentRegistry,
) -> (CaptureReport, WorldSnapshot) {
    let mut out = WorldSnapshot::new();
    let r = capture_incremental(w, c, reg, &mut out).expect("captura");
    (r, out)
}

/// **O rebuild completo, para comparar** — a mesma função que o save usa.
fn full(w: &mut World, reg: &ComponentRegistry) -> WorldSnapshot {
    let mut st = TransformPropagationState::new(w);
    let mut wl = WorklistBuf::default();
    let mut out = WorldSnapshot::new();
    world_to_snapshot(w, &mut st, &mut wl, reg, &mut out).expect("rebuild");
    out
}

/// ⭐⭐ **A EQUIVALÊNCIA — o gate que manda.** Sob spawn + despawn + reparent + remoção **no
/// mesmo quadro**, a captura incremental tem de dar o mesmo snapshot que um rebuild completo,
/// byte a byte.
///
/// (Mutação: tirar o `c.archetype != archetype_id` do critério de sujidade ⇒ a remoção não é
/// vista e as duas divergem — RED. Tirar a comparação de bytes ⇒ ainda passa, e é por isso que
/// existe o gate do falso positivo ao lado.)
#[test]
fn the_incremental_capture_equals_a_full_rebuild() {
    let reg = registry();
    let (mut w, es) = world_with(6);
    let mut cache = CaptureCache::new();
    let (_, inc0) = capture(&mut w, &mut cache, &reg);
    assert_eq!(
        inc0,
        full(&mut w, &reg),
        "a primeira captura ja' tem de bater"
    );

    // As quatro coisas, no MESMO quadro.
    w.spawn((Transform::IDENTITY, Name::new("novo"), RootOrder(99)));
    w.entity_mut(es[0]).despawn();
    w.entity_mut(es[1]).insert(ChildOf(es[2]));
    w.entity_mut(es[3]).insert(Visibility::hidden());
    w.entity_mut(es[3]).remove::<Visibility>();
    w.entity_mut(es[4]).insert(Name::new("renomeado"));

    let (_, inc1) = capture(&mut w, &mut cache, &reg);
    assert_eq!(
        inc1,
        full(&mut w, &reg),
        "spawn+despawn+reparent+remove no mesmo quadro divergiram do rebuild"
    );
}

/// **Condição 5 — PONTO FIXO:** `capture → quadro sem input → capture` não produz passo nenhum.
///
/// ⚠️ É a memória *"o undo só faz uma etapa"*: se o `clear_trackers` estiver no sítio errado, ou
/// não correr, todo quadro parece sujo e a pilha enche de passos idênticos.
///
/// (Mutação: tirar o `world.clear_trackers()` do fim da captura ⇒ a 2.ª captura vê tudo sujo — RED.)
#[test]
fn a_frame_without_input_produces_no_step() {
    let reg = registry();
    let (mut w, _) = world_with(20);
    let mut cache = CaptureCache::new();
    let (first, a) = capture(&mut w, &mut cache, &reg);
    assert_eq!(first.spawned, 20);

    let (second, b) = capture(&mut w, &mut cache, &reg);
    assert_eq!(second.dirty, 0, "nada mudou, nada pode estar sujo");
    assert_eq!(second.reserialized, 0);
    assert_eq!(second.rows, 20);
    assert_eq!(a, b, "o snapshot tem de ser o MESMO");
}

/// ⭐ **Condição 2 — REMOVER um componente é visto**, e nenhum tick o diria.
///
/// ⚠️ Medido na refutação (R2): `remove::<Sprite>` em 1 % das entidades dava **zero** linhas
/// re-serializadas com o critério só-por-tick. A cura é o `ArchetypeId`, porque **toda remoção
/// muda o archetype**.
///
/// (Mutação: `c.archetype != archetype_id` → `false` ⇒ a remoção fica invisível — RED.)
#[test]
fn removing_a_component_is_seen_even_though_no_tick_says_so() {
    let reg = registry();
    let (mut w, es) = world_with(4);
    w.entity_mut(es[0]).insert(Visibility::hidden());
    let mut cache = CaptureCache::new();
    let (_, before) = capture(&mut w, &mut cache, &reg);

    w.entity_mut(es[0]).remove::<Visibility>();
    let (r, after) = capture(&mut w, &mut cache, &reg);

    assert_eq!(
        r.reserialized, 1,
        "a remocao TEM de produzir uma linha nova"
    );
    assert_ne!(before, after, "…e o snapshot tem de mudar");
    assert_eq!(after, full(&mut w, &reg));
}

/// ⭐⭐ **Condição 5 — O GESTO SEGURADO: um arrasto de N quadros é UM passo com TODAS as
/// mutações.**
///
/// ⚠️ O `is_newer_than` é **estrito**. Se o `clear_trackers` corresse por QUADRO em vez de por
/// CAPTURA, só a mutação do último quadro entraria no passo — o artista arrastaria 10 quadros e
/// o undo devolveria 9/10 do caminho. É a refutação R3, e é por isso que o clear vive **dentro**
/// da captura e não no laço de quadro.
///
/// (Mutação: chamar `world.clear_trackers()` no laço abaixo ⇒ só a última escrita entra — RED.)
#[test]
fn a_held_gesture_of_many_frames_is_one_step_with_every_mutation() {
    let reg = registry();
    let (mut w, es) = world_with(3);
    let mut cache = CaptureCache::new();
    capture(&mut w, &mut cache, &reg);

    // Dez quadros de arrasto: cada um mexe num objecto DIFERENTE, e nenhum captura.
    for i in 0..10u32 {
        let e = es[(i % 3) as usize];
        w.entity_mut(e).insert(RootOrder(100 + i));
        // ⚠️ **NENHUM `clear_trackers` aqui** — é exactamente o que a R3 proíbe.
    }

    let (r, snap) = capture(&mut w, &mut cache, &reg);
    assert_eq!(
        r.reserialized, 3,
        "os TRES objectos tocados entram no passo"
    );
    assert_eq!(
        snap,
        full(&mut w, &reg),
        "o passo tem de conter o estado final de todos os tocados"
    );
}

/// **Condição 3 — o tick é PRÉ-FILTRO, os bytes são a verdade.** Uma escrita que repõe o MESMO
/// valor carimba mudança; a linha é re-serializada, comparada, e **não** emite delta.
///
/// ⚠️ É a segunda armadilha medida da change detection (R2): 1 000 linhas re-serializadas com
/// bytes idênticos. Sem a comparação, um sistema que reescreve o que já lá estava sujaria o
/// mundo todo — e isso é o caso comum, não o exótico (um painel que aplica os seus valores a
/// cada quadro reescreve sempre).
///
/// ⭐⭐ **E este gate corrigiu a minha ideia do mecanismo, com o vermelho a dizê-lo:** a 1.ª
/// versão só ADQUIRIA o `Mut<T>` (`let _ = ...get_mut()`) e o pré-filtro **não acusou nada** —
/// `dirty = 0`. O bevy carimba no **`DerefMut`**, não ao entregar o `Mut`. ⇒ *"chamar `get_mut`
/// suja"* é **falso**; o que suja é **tocar** no valor. A armadilha continua a existir e é a
/// mesma, mas o gatilho é mais estreito do que a refutação fazia parecer, e um teste que só
/// pegasse no `Mut` mediria zero e passaria a acreditar que o pré-filtro é perfeito.
///
/// (Mutação: emitir sempre em vez de comparar ⇒ `reserialized` vira 1 — RED.)
#[test]
fn writing_the_same_value_back_is_absorbed_by_the_byte_compare() {
    let reg = registry();
    let (mut w, es) = world_with(4);
    let mut cache = CaptureCache::new();
    let (_, before) = capture(&mut w, &mut cache, &reg);

    // Escreve o valor que já lá está — `DerefMut` carimba, os bytes não mudam.
    let mut ent = w.entity_mut(es[0]);
    let mut t = ent.get_mut::<Transform>().expect("tem");
    *t = Transform::IDENTITY;

    let (r, after) = capture(&mut w, &mut cache, &reg);
    assert_eq!(
        r.dirty, 1,
        "o pre-filtro TEM de acusar (senao nao ha' o que absorver)"
    );
    assert_eq!(
        r.reserialized, 0,
        "…e a comparacao de bytes TEM de o absorver"
    );
    assert_eq!(before, after, "o snapshot nao pode mudar");
}

/// **Condição 4 — despawn por carimbo.** Uma entidade que sai do mundo sai do snapshot **e da
/// cache**, senão a memória cresce para sempre e um id reciclado leria a linha do morto.
#[test]
fn a_despawned_entity_leaves_the_cache_too() {
    let reg = registry();
    let (mut w, es) = world_with(5);
    let mut cache = CaptureCache::new();
    capture(&mut w, &mut cache, &reg);
    assert_eq!(cache.len(), 5);

    w.entity_mut(es[2]).despawn();
    let (r, snap) = capture(&mut w, &mut cache, &reg);
    assert_eq!(r.despawned, 1);
    assert_eq!(snap.entities.len(), 4);
    assert_eq!(cache.len(), 4, "a cache tem de encolher com o mundo");
}

/// ⭐ **A PARTILHA existe** — as linhas que não mudaram são o MESMO `Arc` entre capturas. É a
/// propriedade inteira sobre a qual o custo da pilha de undo cai (~614 MB → ~12,5 MB medido).
///
/// (Mutação: `Arc::new(row.clone())` para toda linha ⇒ os ponteiros diferem — RED.)
#[test]
fn unchanged_rows_are_the_same_allocation_between_captures() {
    let reg = registry();
    let (mut w, es) = world_with(4);
    let mut cache = CaptureCache::new();
    let (_, a) = capture(&mut w, &mut cache, &reg);

    w.entity_mut(es[0]).insert(Name::new("so' esta mudou"));
    let (_, b) = capture(&mut w, &mut cache, &reg);

    let shared = a
        .entities
        .iter()
        .filter(|ra| b.entities.iter().any(|rb| Arc::ptr_eq(ra, rb)))
        .count();
    assert_eq!(
        shared, 3,
        "as tres linhas intactas tinham de ser o MESMO Arc — sem isso a pilha volta a custar o \
         tamanho do mundo por passo"
    );
}

/// **O restore reinicia a cache, e dá o próprio clear** (condição 5, 2.ª metade). Sem isto a
/// captura seguinte compararia contra linhas de um mundo que já não existe.
#[test]
fn a_reset_forgets_everything_and_the_next_capture_rebuilds() {
    let reg = registry();
    let (mut w, _) = world_with(3);
    let mut cache = CaptureCache::new();
    capture(&mut w, &mut cache, &reg);
    assert_eq!(cache.len(), 3);

    cache.reset(&mut w);
    assert!(cache.is_empty(), "o reset esquece as linhas");

    let (r, snap) = capture(&mut w, &mut cache, &reg);
    assert_eq!(r.spawned, 3, "tudo volta a ser novo depois de um reset");
    assert_eq!(snap, full(&mut w, &reg));
}

/// ⭐⭐ **UM RESPAWN COM A MESMA IDENTIDADE E BITS NOVOS** — o caso que dois gates da SHELL
/// apanharam e que este ficheiro não tinha.
///
/// ⚠️ É o caminho mais comum que existe: o restore do undo despawna tudo e respawna, e o sync do
/// vetor faz o mesmo por quadro. O objeto volta com o MESMO `StableId` e **outros bits**.
///
/// A 1.ª versão da varredura em duas passagens perdia-o: a passagem A ia aos bits cacheados (que
/// já não resolvem) e saltava a linha; a passagem B perguntava *"está na cache?"* e a resposta era
/// **sim** — a linha velha ainda lá estava —, então ninguém a reconstruía e o `retain` apagava-a.
/// **O objeto desaparecia do snapshot.** A cura é a passagem B perguntar pela GERAÇÃO.
///
/// (Mutação: `rows.get(s).map_or(true, |c| c.seen != generation)` → `!rows.contains_key(s)` ⇒ o
/// objeto some — RED.)
#[test]
fn an_entity_respawned_with_the_same_id_and_new_bits_survives() {
    let reg = registry();
    let (mut w, es) = world_with(3);
    let mut cache = CaptureCache::new();
    let (_, before) = capture(&mut w, &mut cache, &reg);
    let id = crate::stable_id_of(&w, es[1]).expect("tem id");

    // O respawn: mesma identidade, bits novos, mesmo conteúdo — é o que o restore faz.
    let name = w.get::<Name>(es[1]).cloned().expect("tem nome");
    let order = *w.get::<RootOrder>(es[1]).expect("tem ordem");
    w.entity_mut(es[1]).despawn();
    let reborn = w.spawn((Transform::IDENTITY, name, order, id)).id();
    assert_ne!(reborn, es[1], "um respawn da' bits novos");

    let (r, after) = capture(&mut w, &mut cache, &reg);
    assert_eq!(
        after.entities.len(),
        3,
        "o objeto renascido TEM de continuar no snapshot"
    );
    assert_eq!(r.despawned, 0, "ele nao morreu — mudou de bits");
    assert_eq!(r.spawned, 0, "…e nao nasceu: a identidade dele ja' existia");
    assert_eq!(
        before, after,
        "o conteudo e' o MESMO, entao a captura tem de ser um PONTO FIXO — um respawn que \
         devolve o mesmo estado nao pode produzir um passo de undo"
    );
    assert_eq!(after, full(&mut w, &reg));
}

/// ⚠️ **E os bits RECICLADOS não podem fazer uma linha ler outro objeto.** O bevy reusa os bits
/// de quem morreu; sem a conferência do `StableId` na passagem A, a linha do morto leria o vivo
/// que herdou os bits — em silêncio.
#[test]
fn recycled_bits_never_make_a_row_read_the_wrong_object() {
    let reg = registry();
    let (mut w, es) = world_with(2);
    let mut cache = CaptureCache::new();
    capture(&mut w, &mut cache, &reg);
    let dead_id = crate::stable_id_of(&w, es[0]).expect("tem id");

    w.entity_mut(es[0]).despawn();
    // O bevy tende a reciclar os bits do ultimo morto — quem nascer agora pode herda'-los.
    let newborn = w
        .spawn((Transform::IDENTITY, Name::new("intruso"), RootOrder(9)))
        .id();

    let (_, snap) = capture(&mut w, &mut cache, &reg);
    let ids: Vec<_> = snap.entities.iter().map(|r| r.id).collect();
    assert!(
        !ids.contains(&dead_id),
        "a identidade do objeto MORTO sobreviveu no snapshot: {ids:?}"
    );
    let born_id = crate::stable_id_of(&w, newborn).expect("o novo tem id");
    assert!(ids.contains(&born_id), "o objeto novo tem de estar la'");
    assert_eq!(snap, full(&mut w, &reg));
}

/// ⭐⭐⭐ **UM TIPO QUE NASCE DEPOIS DA PRIMEIRA CAPTURA CONTINUA A SER VIGIADO** — o defeito por
/// trás de *«o undo/redo está completamente destruído»* (Enio, 2026-09-04, o 5.º report).
///
/// # ⛔⛔⛔ O mecanismo, medido com o app aberto pelo PILL
///
/// A lista de colunas vigiadas resolvia-se **uma vez**, na primeira captura (`primed`), a partir de
/// `world.component_id::<T>()` — que é `None` para todo tipo registado que ainda não existe no
/// mundo. Pelo pill a primeira captura vê a cena **vazia**: nenhum nó do modelador existe, logo
/// `FieldPose`/`FieldNode`/`FieldMods` ficam de fora da lista **para sempre**. Tudo o que nasce
/// depois tem essas colunas, e o pré-filtro nunca as olha ⇒ uma escrita **no lugar** (mover com o
/// gizmo, arrastar um slider, digitar um número) é invisível — só um spawn, um despawn ou uma troca
/// de archetype (pôr um modificador) chegam a ser passo. Medido pela sonda
/// (`PH2D_FIELD_UNDO_PROBE=1`, sem `PH2D_FIELD_SMOKE`):
///
/// ```text
/// f=30 criar pela paleta        undo=0→1            (spawn: visto)
/// f=40..47 arrastar a seta      x=0→0,209  undo=1   ⛔ nenhum passo — e nenhuma supressão
/// f=80 Ctrl+Z                   nos=5→0             ⛔ um Ctrl+Z apaga TUDO
/// ```
///
/// ⚠️ **Com `PH2D_FIELD_SMOKE=1` o mesmo arrasto registava** — a cena de demo nasce ANTES da
/// primeira captura, então as colunas já existiam quando a lista foi resolvida. *Foi por isso que
/// quatro jornadas de sondas passaram verdes sobre um produto partido: a sonda armava o módulo pela
/// variável de ambiente, e o dono arma-o pelo pill.*
///
/// A fixtura reproduz o pill: prime com um mundo em que `Visibility` **ainda não existe**, faz
/// nascer uma entidade com ela, e depois escreve-a **no lugar**.
///
/// (Mutação: voltar a resolver a lista só quando `!primed` ⇒ a escrita no lugar não é vista,
/// a incremental diverge do rebuild — RED, com a mensagem a nomear o quadro.)
#[test]
fn a_component_type_born_after_the_first_capture_is_still_watched() {
    let reg = registry();
    let (mut w, es) = world_with(1);
    let mut c = CaptureCache::new();
    // 1. A primeira captura, com a cena «vazia» daquele tipo: `Visibility` não existe no mundo.
    assert!(
        w.component_id::<Visibility>().is_none(),
        "a fixtura precisa de um tipo que ainda não tenha id no mundo"
    );
    let _ = capture(&mut w, &mut c, &reg);
    // 2. Nasce uma entidade COM o tipo — o «criar pela paleta».
    let novo = w
        .spawn((
            Transform::IDENTITY,
            Name::new("peca"),
            RootOrder(7),
            Visibility::default(),
        ))
        .id();
    let (r, s1) = capture(&mut w, &mut c, &reg);
    assert_eq!(r.spawned, 1, "o nascimento é visto (é um spawn)");
    assert_eq!(s1, full(&mut w, &reg), "depois do nascimento");
    // 3. A escrita NO LUGAR — o «arrastar a seta».
    w.get_mut::<Visibility>(novo).expect("existe").hidden ^= true;
    let (r, s2) = capture(&mut w, &mut c, &reg);
    assert_eq!(
        s2,
        full(&mut w, &reg),
        "⛔ a escrita no lugar de um tipo nascido DEPOIS da primeira captura ficou invisível — é \
         o arrasto do gizmo que nunca vira passo, e o Ctrl+Z que apaga tudo (sujas={}, \
         reserializadas={})",
        r.dirty,
        r.reserialized
    );
    assert!(
        r.reserialized >= 1,
        "a linha tem de ter sido re-serializada"
    );
    let _ = es;
}
