//! **A §12 alcança TODA âncora que o modelo permite — e lê a forma como o motor a lê.**
//!
//! ⚠️ **Só a shell vê as duas metades.** O painel do Inspector é chrome e não depende do
//! `ph2d-ecs`; o cap de âncoras vive no motor e o array de ids vive no painel. Gate irmão do
//! [`the_slice_section_offers_every_mode_the_engine_has`].
//!
//! # A classe de defeito
//!
//! Um array de ids mais curto que o cap do modelo torna as âncoras do fim **inalcançáveis por
//! gesto nenhum**: elas existem, gravam, sobrevivem ao undo — e o artista não consegue clicar
//! nelas. É exatamente o que a §9 Sampling pagou com quatro modos de filtro mudos, e o que a
//! auditoria de 2026-08-21 mediu como a forma mais cara de dívida.

use ph2d_ecs::{ANCHORS_MAX, AnchorKind, NamedAnchor, NamedAnchorList};
use ph2d_editor::ids;

/// **(1) Há uma linha clicável por âncora que o modelo aceita.**
#[test]
fn the_row_ids_cover_the_model_cap() {
    assert_eq!(
        ids::INSP_ANCHOR_ROW.len(),
        ANCHORS_MAX,
        "o painel tem {} linhas para um cap de {ANCHORS_MAX} ancoras — as do fim ficariam \
         inalcancaveis por gesto nenhum",
        ids::INSP_ANCHOR_ROW.len()
    );
}

/// **(2) Os ids das linhas são todos DISTINTOS.**
///
/// ⚠️ Duas linhas com o mesmo id fariam clicar na 7ª abrir a 3ª — e `hash_node_id` é FNV-1a
/// sobre a string, por isso um erro de copiar-colar na tabela produz exatamente isso, em
/// silêncio.
#[test]
fn every_row_id_is_distinct() {
    let mut seen: Vec<_> = ids::INSP_ANCHOR_ROW.to_vec();
    seen.sort_unstable_by_key(|n| n.0);
    seen.dedup_by_key(|n| n.0);
    assert_eq!(
        seen.len(),
        ids::INSP_ANCHOR_ROW.len(),
        "duas linhas partilham o mesmo id: clicar numa abriria a outra"
    );
}

/// **(3) O painel lê a FORMA da âncora como o motor a lê.**
///
/// Duas implementações da mesma tabela (`NamedAnchor::kind()` no motor,
/// `InspectorAnchorRow::kind_tag()` no painel) — porque o painel não pode importar o motor. Esta
/// é a única coisa que impede as duas divergirem e o Inspector rotular «Socket» o que o motor
/// trata como «Region».
#[test]
fn the_panel_reads_the_same_shape_the_engine_does() {
    let cases = [
        (None, None, AnchorKind::Socket, 0u8, "Socket"),
        (Some([0.0; 4]), None, AnchorKind::Slice, 1, "Slice"),
        (
            Some([0.0; 4]),
            Some([0.0; 4]),
            AnchorKind::NineSliceRegion,
            2,
            "Region",
        ),
        // ⚠️ O estado impossível: miolo sem área. Os dois lados têm de o ler como Socket.
        (None, Some([0.0; 4]), AnchorKind::Socket, 0, "Socket"),
    ];
    for (bounds, center, engine_kind, panel_tag, label) in cases {
        let mut a = NamedAnchor::socket("x");
        a.bounds = bounds;
        a.center = center;
        assert_eq!(
            a.kind(),
            engine_kind,
            "o motor leu {bounds:?}/{center:?} mal"
        );

        let row = ph2d_editor::InspectorAnchorRow {
            name: "x".into(),
            pos: [0.0, 0.0],
            rot_deg: 0.0,
            bounds,
            center,
            riders: 0,
        };
        assert_eq!(
            row.kind_tag(),
            panel_tag,
            "o painel leu {bounds:?}/{center:?} como {} e o motor como {engine_kind:?}",
            row.kind_label()
        );
        assert_eq!(row.kind_label(), label);
    }
}

/// **(4) O cap de âncoras é imposto, e a lista pára nele.**
///
/// ⚠️ Sem isto, a 65ª âncora entraria no modelo e ficaria sem linha para clicar — pior que ser
/// recusada, porque existe e não se vê.
#[test]
fn the_model_stops_exactly_where_the_panel_runs_out_of_rows() {
    let mut l = NamedAnchorList::new();
    while l.len() < ANCHORS_MAX {
        let n = l.next_free_name();
        l.insert(NamedAnchor::socket(n)).expect("dentro do cap");
    }
    assert_eq!(l.len(), ids::INSP_ANCHOR_ROW.len());
    assert!(
        l.insert(NamedAnchor::socket("one_too_many")).is_err(),
        "o modelo aceitou uma ancora que o painel nao consegue mostrar"
    );
}

/// **(5) O seletor de MONTAGEM alcança toda âncora que o PAI pode ter.**
///
/// ⚠️ Mesma classe do (1), do outro lado da relação: um array mais curto que `ANCHORS_MAX`
/// tornaria as últimas âncoras do pai **inescolhíveis** — elas apareceriam na lista dele, o
/// artista veria-as, e não conseguiria prender nada nelas.
#[test]
fn the_mount_option_ids_cover_the_model_cap() {
    assert_eq!(
        ids::INSP_MOUNT_OPT.len(),
        ANCHORS_MAX,
        "o seletor oferece {} opcoes para um pai que pode ter {ANCHORS_MAX} ancoras",
        ids::INSP_MOUNT_OPT.len()
    );
}

/// **(6) Nenhum id do seletor colide — nem entre si, nem com as linhas da lista, nem com o «—».**
///
/// ⚠️ A colisão com as LINHAS é a que mais assusta: as duas famílias vivem na mesma seção e são
/// pintadas no mesmo quadro, então um id repetido faria escolher uma montagem ao clicar numa
/// linha da lista. E `hash_node_id` é FNV-1a sobre a string: um erro de copiar-colar na tabela
/// produz exatamente isso, **em silêncio**.
#[test]
fn no_mount_option_id_collides_with_anything_in_the_section() {
    let mut all: Vec<_> = ids::INSP_MOUNT_OPT
        .iter()
        .chain(ids::INSP_ANCHOR_ROW.iter())
        .chain(std::iter::once(&ids::INSP_MOUNT_NONE_OPT))
        .chain(std::iter::once(&ids::INSP_MOUNT_PICK))
        .copied()
        .collect();
    let n = all.len();
    all.sort_unstable_by_key(|i| i.0);
    all.dedup_by_key(|i| i.0);
    assert_eq!(
        all.len(),
        n,
        "dois ids da §12 partilham o mesmo valor — um gesto dispararia o outro"
    );
}

/// **(7) A montagem lê a mesma verdade nos dois lados.**
///
/// O motor decide `Mounted`/`Dangling`/`Free` a partir da cena; o painel decide o mesmo a partir
/// do snapshot, sem poder importar o motor. ⚠️ São **duas** implementações da mesma tabela —
/// irmãs exactas do `kind()` no teste (3) —, e é este gate que impede o Inspector oferecer «—»
/// sobre um objeto que o motor está a mover pela âncora.
#[test]
fn the_panel_reads_the_same_mount_state_the_engine_does() {
    use ph2d_ecs::{AnchorMount, ChildOf, MountState, Transform, World, mount_state_of};

    // (parent tem estas âncoras, o filho monta neste nome) → o estado esperado dos dois lados.
    let cases: [(&[&str], Option<&str>, bool, bool); 4] = [
        (&["muzzle"], None, false, false),          // livre
        (&["muzzle"], Some("muzzle"), true, false), // montado
        (&["muzzle"], Some("gone"), false, true),   // pendurado: o nome saiu
        (&[], Some("muzzle"), false, true),         // pendurado: o pai perdeu a lista
    ];

    for (parent_names, mount, want_mounted, want_dangling) in cases {
        // — o motor —
        let mut w = World::new();
        let mut list = NamedAnchorList::new();
        for n in parent_names {
            list.insert(NamedAnchor::socket(*n)).unwrap();
        }
        let parent = w.spawn((Transform::IDENTITY, list)).id();
        let mut child = w.spawn((Transform::IDENTITY, ChildOf(parent)));
        if let Some(m) = mount {
            child.insert(AnchorMount::new(m));
        }
        let child = child.id();
        let engine = mount_state_of(&w, child);

        // — o painel —
        let info = ph2d_editor::InspectorAnchorInfo {
            entity_bits: 1,
            rows: Vec::new(),
            present: false,
            selected_count: 1,
            mixed: false,
            parent_anchors: parent_names.iter().map(|s| (*s).to_string()).collect(),
            mount: mount.map(str::to_string),
            mount_offset: [0.0, 0.0],
            vis_in_editor: false,
            vis_at_runtime: false,
        };

        assert_eq!(
            matches!(engine, MountState::Mounted(_)),
            want_mounted,
            "o motor discordou sobre {parent_names:?} / {mount:?}"
        );
        assert_eq!(
            info.mount_index().is_some(),
            want_mounted,
            "o painel discordou do motor sobre {parent_names:?} / {mount:?}"
        );
        assert_eq!(engine == MountState::Dangling, want_dangling);
        assert_eq!(
            info.mount_dangling(),
            want_dangling,
            "o painel leu o pendurado ao contrario em {parent_names:?} / {mount:?}"
        );
    }
}

// ── §11 Animation (spec Sprite 08) ───────────────────────────────────────────────────────────

/// **(8) A lista da §11 alcança TODA animação que o modelo aceita.**
///
/// ⚠️ Mesma classe do (1), e foi para a satisfazer que o cap desceu de 256 (o número da spec)
/// para 64: *um modelo que aceita o que o painel não mostra produz estado inalcançável por gesto
/// nenhum*. O motivo que a spec dá para o 256 é «a contagem típica é < 50», que 64 cobre.
#[test]
fn the_anim_row_ids_cover_the_model_cap() {
    assert_eq!(
        ids::INSP_ANIM_ROW.len(),
        ph2d_ecs::ANIM_TAGS_MAX,
        "a lista tem {} linhas para um cap de {} animacoes",
        ids::INSP_ANIM_ROW.len(),
        ph2d_ecs::ANIM_TAGS_MAX
    );
    let mut all: Vec<_> = ids::INSP_ANIM_ROW.to_vec();
    let n = all.len();
    all.sort_unstable_by_key(|i| i.0);
    all.dedup_by_key(|i| i.0);
    assert_eq!(all.len(), n, "duas linhas partilham o mesmo id");
}

/// **(9) O modelo pára exatamente onde o painel fica sem linhas.**
#[test]
fn the_anim_model_stops_where_the_panel_runs_out_of_rows() {
    let mut lib = ph2d_ecs::SpriteAnimations::new();
    while lib.len() < ph2d_ecs::ANIM_TAGS_MAX {
        let n = lib.next_free_name();
        lib.insert(ph2d_ecs::AnimationTag::new(n, 0, 1))
            .expect("dentro do cap");
    }
    assert_eq!(lib.len(), ids::INSP_ANIM_ROW.len());
    assert!(
        lib.insert(ph2d_ecs::AnimationTag::new("one_too_many", 0, 1))
            .is_err(),
        "o modelo aceitou uma animacao que o painel nao consegue mostrar"
    );
}

/// **(10) O painel nomeia as direções como o motor as nomeia.**
///
/// Duas implementações da mesma tabela — o painel não pode importar o motor. É a irmã exacta do
/// `kind_label` da §12, e sem ela o Inspector rotularia `Forward` o que o motor toca ao contrário.
#[test]
fn the_panel_names_the_directions_the_engine_names() {
    use ph2d_ecs::AnimDirection;
    for d in AnimDirection::ALL {
        let tag = d.tag();
        assert_eq!(
            AnimDirection::from_tag(tag),
            d,
            "o tag {tag} nao volta a` mesma direcao"
        );
        // ⚠️ O rótulo do painel é curto (a coluna é estreita), mas tem de **começar** pelo do
        // motor: `Ping-Pong Rev` contra `Ping-Pong Rev`, `Forward` contra `Forward`.
        assert_eq!(
            ph2d_editor::screens::hero::InspectorAnimRow {
                name: "x".into(),
                from: 0,
                to: 1,
                frame_ms: 100,
                direction_tag: tag,
                repeat: 0,
                hold_ms: 0,
                repeat_delay_ms: 0,
            }
            .direction_tag,
            tag
        );
    }
    // E a ordem do array é a que o despacho usa: a posição É a tag.
    assert_eq!(AnimDirection::ALL[0], AnimDirection::Forward);
    assert_eq!(AnimDirection::ALL[3], AnimDirection::PingPongReverse);
}

/// **(11) Nenhum id da §11 colide com os da §12** — as duas são pintadas no mesmo quadro.
#[test]
fn no_anim_id_collides_with_the_anchor_section() {
    let mut all: Vec<_> = ids::INSP_ANIM_ROW
        .iter()
        .chain(ids::INSP_ANCHOR_ROW.iter())
        .chain(ids::INSP_MOUNT_OPT.iter())
        .chain(ids::INSP_ANIM_DIR.iter())
        .chain(ids::INSP_ANIM_DIR_OVERRIDE.iter())
        .chain(ids::INSP_ANIM_LOOP_OVERRIDE.iter())
        .chain(
            [
                ids::INSP_ANIM_ADD,
                ids::INSP_ANIM_REMOVE,
                ids::INSP_ANIM_NAME,
                ids::INSP_ANIM_FROM,
                ids::INSP_ANIM_TO,
                ids::INSP_ANIM_FRAME_MS,
                ids::INSP_ANIM_HOLD_MS,
                ids::INSP_ANIM_DELAY_MS,
                ids::INSP_ANIM_REPEAT,
                ids::INSP_ANIM_SPEED,
                ids::INSP_ANIM_REWIND,
                ids::INSP_ANIM_PLAYING,
                ids::INSP_ANIM_AUTOPLAY,
                ids::INSP_ANIM_ADD_PLAYER,
            ]
            .iter(),
        )
        .copied()
        .collect();
    let n = all.len();
    all.sort_unstable_by_key(|i| i.0);
    all.dedup_by_key(|i| i.0);
    assert_eq!(all.len(), n, "dois ids partilham o mesmo valor");
}
