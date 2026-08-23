//! `PH2D_MOUNT_SMOKE` — **a âncora deixa de ser autoria e passa a mover coisas** ([ADR-0072] §2.6).
//!
//! ```text
//! cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Sprite \
//!   && env PH2D_MOUNT_SMOKE=1 cargo run -p ph2d-host-desktop --release
//! ```
//!
//! O [`crate::socket_smoke`] mostra o que uma âncora **é**. Este mostra para que ela **serve**: um
//! boneco com duas âncoras, e três objetos pendurados nele.
//!
//! | objeto | monta em | o que prova |
//! |---|---|---|
//! | espada (vermelha) | `hand_r` | segue a mão |
//! | chapéu (amarelo) | `head` | segue a cabeça |
//! | **mancha cinzenta** | *nada* | **o controlo** — fica na origem do boneco |
//! | quadrado verde | `head`, **deslocado** | «Reset to Anchor» tem o que fazer, e a `head` mostra `2 riding` |
//!
//! # ⚠️ A mancha cinzenta é metade da cena, e é a metade que se esquece
//!
//! Ela é filha do boneco tal como as outras duas e **não monta em âncora nenhuma**. Sem ela, uma
//! cena em que a montagem estivesse a ser ignorada por completo pareceria exatamente igual — três
//! filhos a acompanhar o pai é o que filhos já faziam antes deste trabalho existir. *Uma fixtura
//! sem o controlo mede silêncio*; o que se vê aqui é a **diferença** entre os dois.
//!
//! # O gesto
//!
//! O boneco nasce selecionado. Com a seção **Sockets / Anchors** aberta:
//!
//! 1. Escolher `hand_r` na lista → aparecem as alças no canvas.
//! 2. **Arrastar a cruz** → a espada vai junto, ao vivo. A mancha cinzenta não se mexe.
//! 3. **Arrastar o braço** (roda a âncora) → a espada gira em torno da mão.
//! 4. Selecionar a espada: a §12 mostra **«Rides Parent Anchor: hand_r»**. Pôr «—» larga-a na
//!    origem do boneco, em cima da mancha cinzenta — que é a prova de que era a âncora a movê-la.
//!    Voltar a escolher `hand_r` **pousa-a na mão outra vez**, e não onde ela tinha ficado.
//! 5. Selecionar o **verde**: ele monta na `head` e está fora dela. A §12 diz *«Off anchor by
//!    35, 30 px»* e oferece **«Reset to Anchor»** — que o põe em cima do chapéu, e desaparece.
//! 6. Selecionar o **boneco** e marcar **«Always show anchors»**: as cruzes ficam mesmo depois de
//!    selecionar outra coisa, ou de fechar a seção.
//!
//! ⚠️ **A segunda caixa, «Show anchors at runtime», grava e ainda não tem quem a leia** — não há
//! modo de jogo neste app (`shells/game`, Runtime R1, adiado). Ver
//! `ph2d_ecs::AnchorVisibility::at_runtime`.
//!
//! [ADR-0072]: ../../../docs/architecture/decisions/0072-named-anchor-unification.md

use ph2d_core::Vec2;
use ph2d_ecs::{AnchorMount, ChildOf, NamedAnchor, NamedAnchorList, Transform};

/// Lado do boneco, em pixels da fonte.
const BODY_PX: u32 = 128;
/// Lado de cada objeto pendurado. Iguais entre si: o que os distingue é a COR e onde estão,
/// e um tamanho diferente convidaria a ler a distância como perspetiva.
const RIDER_PX: u32 = 48;

/// A mão direita, em metros relativos ao boneco.
///
/// ⚠️ **Estes dois números vieram do gate abaixo, não do olho.** A primeira versão punha a mão a
/// `0,46 m` da origem com objetos de `0,48 m` — a espada tapava a mancha de controlo, e a cena
/// perdia exatamente a comparação que existe para mostrar.
const HAND_R: Vec2 = Vec2::new(0.72, -0.15);
/// A cabeça, idem.
const HEAD: Vec2 = Vec2::new(0.0, 0.80);

// ⚠️ Cores de CENA, não de chrome: o HR-15 fala da UI, e estas são o conteúdo que o Enio olha.
// Sem cores diferentes os três objetos são o mesmo quadrado branco e a cena não ensina nada.
const SWORD_TINT: [f32; 4] = [0.90, 0.20, 0.20, 1.0]; // LITERAL-COLOR-OK: conteúdo da cena
const HAT_TINT: [f32; 4] = [0.95, 0.80, 0.15, 1.0]; // LITERAL-COLOR-OK: conteúdo da cena
const CONTROL_TINT: [f32; 4] = [0.45, 0.45, 0.48, 1.0]; // LITERAL-COLOR-OK: conteúdo da cena
const LOOSE_TINT: [f32; 4] = [0.30, 0.80, 0.40, 1.0]; // LITERAL-COLOR-OK: conteúdo da cena

/// **O quarto objeto nasce FORA da âncora**, de propósito (Enio, 2026-08-23).
///
/// ⚠️ Ele é o que torna «Reset to Anchor» **visível ao abrir**: o botão só se pinta quando há
/// deslocamento, e sem uma peça já deslocada o artista teria de arrastar alguma coisa antes de
/// descobrir que o botão existe. Ele também põe **dois** passageiros na `head`, que é o que faz
/// a lista mostrar `Socket · 2 riding`.
const LOOSE_OFFSET: Vec2 = Vec2::new(0.35, 0.30);

pub(crate) fn enabled() -> bool {
    std::env::var_os("PH2D_MOUNT_SMOKE").is_some()
}

/// Monta a cena. Devolve os bits do BONECO — é ele que fica selecionado, porque é dele que as
/// âncoras são e é a lista dele que o gesto abre.
#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_if_enabled(
    sim: &mut ph2d_ecs::SimWorld,
    renderer: &mut ph2d_render::SpriteRenderer,
    asset_db: &ph2d_asset::AssetDb,
    next_cell: &mut u32,
    pixels_per_meter: f32,
    atlas_asset_map: &mut std::collections::BTreeMap<u32, ph2d_asset::AssetId>,
) -> Option<u64> {
    let mut white = |size_px: u32,
                     at: Vec2,
                     sim: &mut ph2d_ecs::SimWorld,
                     renderer: &mut ph2d_render::SpriteRenderer|
     -> Option<u64> {
        let cell = *next_cell;
        let (_, bits) = crate::image_import::spawn_blank_canvas(
            sim,
            renderer,
            asset_db,
            cell,
            size_px,
            2, // branco opaco: a tinta por cima dele dá a cor de cada objeto
            at,
            pixels_per_meter,
            atlas_asset_map,
        )
        .ok()?;
        *next_cell += 1;
        Some(bits)
    };

    let body_bits = white(BODY_PX, Vec2::ZERO, sim, renderer)?;
    let body = ph2d_ecs::Entity::from_bits(body_bits);

    // As duas âncoras do boneco, pela porta que impõe os caps — nunca `list.0.push`.
    let mut list = NamedAnchorList::new();
    let mut hand = NamedAnchor::socket("hand_r");
    hand.transform.translation = HAND_R;
    list.insert(hand).ok()?;
    let mut head = NamedAnchor::socket("head");
    head.transform.translation = HEAD;
    list.insert(head).ok()?;
    sim.world_mut().get_entity_mut(body).ok()?.insert(list);

    // Os quatro filhos. ⚠️ **Os TRÊS primeiros nascem com pose local zero**: o que os separa é
    // **só** a montagem, e é isso que torna a comparação legível. Se a espada trouxesse um
    // deslocamento próprio, vê-la longe da mancha não provaria nada.
    //
    // ⚠️ O quarto é a exceção, e é deliberada — ele nasce FORA da âncora para que «Reset to
    // Anchor» exista ao abrir a cena (ver `LOOSE_OFFSET`).
    for (tint, mount, local) in [
        (SWORD_TINT, Some("hand_r"), Vec2::ZERO),
        (HAT_TINT, Some("head"), Vec2::ZERO),
        (CONTROL_TINT, None, Vec2::ZERO),
        // O quarto: montado na cabeça e **deslocado** — ver `LOOSE_OFFSET`.
        (LOOSE_TINT, Some("head"), LOOSE_OFFSET),
    ] {
        let Some(bits) = white(RIDER_PX, Vec2::ZERO, sim, renderer) else {
            continue;
        };
        let e = ph2d_ecs::Entity::from_bits(bits);
        if let Some(mut s) = sim.world_mut().get_mut::<ph2d_render::Sprite>(e) {
            s.tint = tint;
        }
        if let Ok(mut ent) = sim.world_mut().get_entity_mut(e) {
            ent.insert((
                Transform {
                    translation: local,
                    ..Transform::default()
                },
                ChildOf(body),
            ));
            if let Some(name) = mount {
                ent.insert(AnchorMount::new(name));
            }
        }
    }
    Some(body_bits)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_ecs::{MountState, World, mount_state_of, world_transform};

    /// Reconstrói a cena **sem** o renderer — só a parte que a montagem lê.
    fn scene() -> (World, [ph2d_ecs::Entity; 4]) {
        let mut w = World::new();
        let mut list = NamedAnchorList::new();
        let mut hand = NamedAnchor::socket("hand_r");
        hand.transform.translation = HAND_R;
        list.insert(hand).unwrap();
        let mut head = NamedAnchor::socket("head");
        head.transform.translation = HEAD;
        list.insert(head).unwrap();
        let body = w.spawn((Transform::IDENTITY, list)).id();
        let mk = |w: &mut World, mount: Option<&str>| {
            let mut e = w.spawn((Transform::IDENTITY, ChildOf(body)));
            if let Some(m) = mount {
                e.insert(AnchorMount::new(m));
            }
            e.id()
        };
        let sword = mk(&mut w, Some("hand_r"));
        let hat = mk(&mut w, Some("head"));
        let control = mk(&mut w, None);
        let loose = mk(&mut w, Some("head"));
        w.entity_mut(loose).insert(Transform {
            translation: LOOSE_OFFSET,
            ..Transform::default()
        });
        (w, [sword, hat, control, loose])
    }

    /// **A cena prova o que diz que prova:** os dois montados estão NA âncora, o controlo na
    /// origem do boneco — e os três têm o mesmo `Transform` local.
    ///
    /// ⚠️ Este é o gate que impede a cena degradar-se sem ninguém ver. Se a montagem parasse de
    /// funcionar, o smoke continuaria a desenhar três quadrados coloridos — só que todos no mesmo
    /// sítio, e nada no ecrã diria que isso está errado.
    #[test]
    fn the_riders_sit_on_their_anchors_and_the_control_does_not() {
        let (w, [sword, hat, control, loose]) = scene();
        assert_eq!(world_transform(&w, sword).unwrap().translation, HAND_R);
        assert_eq!(world_transform(&w, hat).unwrap().translation, HEAD);
        assert_eq!(
            world_transform(&w, control).unwrap().translation,
            Vec2::ZERO,
            "o controlo tem de ficar na origem — e' ele que torna a diferenca visivel"
        );
        assert!(matches!(mount_state_of(&w, sword), MountState::Mounted(_)));
        assert_eq!(mount_state_of(&w, control), MountState::Free);
        // ⚠️ **O quarto tem de estar FORA da âncora** — é isso que faz «Reset to Anchor» existir
        // ao abrir a cena. Um deslocamento a zero aqui devolveria o smoke a um botão invisível.
        assert_eq!(
            world_transform(&w, loose).unwrap().translation,
            HEAD + LOOSE_OFFSET,
            "o quarto objeto tem de nascer deslocado da ancora"
        );
    }

    /// **As duas âncoras estão longe uma da outra e do centro.** Uma cena em que a mão e a cabeça
    /// caíssem quase no mesmo ponto mostraria três quadrados sobrepostos e não ensinaria nada.
    #[test]
    fn the_two_anchors_are_far_enough_apart_to_read() {
        let rider_m = RIDER_PX as f32 / 100.0; // ~a escala de projeto default
        for (a, b) in [(HAND_R, HEAD), (HAND_R, Vec2::ZERO), (HEAD, Vec2::ZERO)] {
            let d = (a - b).length();
            assert!(
                d > rider_m,
                "duas marcas a {d:.2} m sobrepoem-se com objetos de {rider_m:.2} m"
            );
        }
    }
}
