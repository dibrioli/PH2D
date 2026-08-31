//! A ida-e-volta do ficheiro, e as três coisas que ele tem de recusar.

use super::*;

fn sample() -> Layout {
    Layout {
        open: vec!["audio_mixer".into()],
        slots: vec![
            ("audio_mixer".into(), Slot::LeftTop),
            ("physics".into(), Slot::RightBottom),
        ],
        dock_w_left: Some(280.0),
        dock_w_right: Some(320.5),
        dock_h_bottom: Some(196.0),
    }
}

#[test]
fn a_layout_survives_the_round_trip() {
    let l = sample();
    assert_eq!(parse(&serialize_section(&l)), l);
}

/// ⭐ **O hash muda com CADA campo** — senão a gravação perde uma metade em silêncio.
#[test]
fn every_field_moves_the_hash() {
    let base = sample();
    let h = hash(&base);
    let mut moved = base.clone();
    moved.slots[0].1 = Slot::RightTop;
    assert_ne!(hash(&moved), h, "mudar um encaixe não moveu o hash");
    let mut moved = base.clone();
    moved.slots[0].0 = "audio_editor".into();
    assert_ne!(hash(&moved), h, "mudar de painel não moveu o hash");
    let mut moved = base.clone();
    moved.dock_w_left = Some(281.0);
    assert_ne!(hash(&moved), h, "arrastar a divisória não moveu o hash");
    let mut moved = base.clone();
    moved.dock_w_right = None;
    assert_ne!(hash(&moved), h, "desistir de uma largura não moveu o hash");
}

/// ⚠️ **Uma linha que não se entende é SALTADA, e o resto do ficheiro sobrevive.**
///
/// É esta tolerância que dispensa o número de schema: um build antigo lê o que conhece.
#[test]
fn junk_is_skipped_and_never_poisons_the_rest() {
    let text = "\
# PH2D layout
dock_w_left=280
isto nao tem igual
slot.audio_mixer=left_top
slot.um_painel=um_encaixe_de_2030
dock_w_right=nao_e_um_numero
uma_chave_de_2030=7
slot.physics=bottom
";
    let l = parse(text);
    assert_eq!(l.dock_w_left, Some(280.0));
    assert_eq!(l.dock_w_right, None, "um número ilegível virou uma largura");
    assert_eq!(
        l.slots,
        vec![
            ("audio_mixer".to_string(), Slot::LeftTop),
            ("physics".to_string(), Slot::Bottom),
        ],
        "o encaixe desconhecido não foi saltado, ou levou os vizinhos consigo"
    );
}

/// ⛔ **Um ficheiro vazio é a arrumação de omissão**, nunca um erro.
#[test]
fn an_absent_file_is_the_default_arrangement() {
    assert_eq!(parse(""), Layout::default());
    assert_eq!(parse("# PH2D layout\n"), Layout::default());
}

/// ⚠️ **A ordem é normalizada na leitura** — senão duas gravações do mesmo estado dariam hashes
/// diferentes e o app escreveria o ficheiro a cada quadro.
#[test]
fn the_order_is_normalised_so_the_same_state_hashes_the_same() {
    let a = parse("slot.zz=left_top\nslot.aa=bottom\n");
    let b = parse("slot.aa=bottom\nslot.zz=left_top\n");
    assert_eq!(a, b);
    assert_eq!(hash(&a), hash(&b));
}

/// ⛔⛔ **A primeira observação de uma sessão nunca grava.**
///
/// ⚠️ Este gate nasceu de um comentário meu que dizia o contrário do que o código fazia: a
/// condição era `c.get() != Some(h)` com o espelho a arrancar em `None`, o que é **sempre
/// verdade** ⇒ o ficheiro era reescrito no arranque de toda sessão.
#[test]
fn the_first_observation_of_a_session_never_writes() {
    assert!(
        !should_save(None, 7),
        "a primeira observação gravou — o ficheiro é reescrito no arranque de toda sessão, mesmo \
         numa em que o artista não tocou em nada"
    );
    assert!(!should_save(Some(7), 7), "gravou sem nada ter mudado");
    assert!(should_save(Some(7), 8), "uma mudança real não foi gravada");
}

/// ⭐⭐ **A arrumação gravada VOLTA** — e a que o produto já não permite **não volta**.
///
/// ⚠️ **A segunda metade é a que importa:** o ficheiro é do artista, mas o `ALLOWED_SLOTS` é do
/// produto. Se uma wave estreitar o que um painel aceita, uma arrumação gravada não pode
/// ressuscitar um sítio onde ele deixou de caber — e a validação vive na LEITURA, porque o ficheiro
/// pode ser mais velho que a regra.
#[test]
fn a_saved_arrangement_comes_back_but_a_forbidden_slot_does_not() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut hero = ph2d_editor::HeroScreen::new(ph2d_editor::NodeId(1));

    let node_of = |id: &str| {
        ph2d_editor::panel::with_registry_ref(|reg| {
            reg.panels()
                .iter()
                .find(|p| p.manifest.id == id)
                .map(|p| p.manifest.panel_node_id)
                .unwrap_or_else(|| panic!("{id} não está registado"))
        })
    };

    install(
        &mut hero,
        &Layout {
            open: vec!["audio_mixer".into(), "um_painel_de_2030".into()],
            slots: vec![
                // Legal: uma coluna de propriedades aceita as duas colunas.
                ("audio_mixer".into(), Slot::LeftTop),
                // ⛔ ILEGAL: o Inspector declara `SIDES`; a faixa de baixo não é dele.
                ("inspector".into(), Slot::Bottom),
                // Um painel que não existe nesta build.
                ("um_painel_de_2030".into(), Slot::RightTop),
            ],
            dock_h_bottom: None,
            dock_w_left: Some(281.0),
            dock_w_right: None,
        },
    );

    assert_eq!(
        hero.store.panel_slot(node_of("audio_mixer")),
        Some(Slot::LeftTop),
        "a arrumação gravada não voltou"
    );
    assert_eq!(
        hero.store.panel_slot(node_of("inspector")),
        None,
        "um encaixe que o painel NÃO permite foi instalado a partir do ficheiro"
    );
    assert_eq!(
        hero.store
            .dock_width_choice(ph2d_editor::screens::layout::DockSide::Left),
        Some(281.0),
        "a largura da coluna não voltou"
    );
    assert_eq!(
        hero.store
            .dock_width_choice(ph2d_editor::screens::layout::DockSide::Right),
        None,
        "uma largura ausente do ficheiro virou uma escolha"
    );
}

/// ⭐ **E o que se GRAVA é a projecção do que está instalado** — a volta completa.
#[test]
fn what_is_installed_is_what_gets_written_back() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut hero = ph2d_editor::HeroScreen::new(ph2d_editor::NodeId(2));
    assert_eq!(
        current(&hero),
        Layout::default(),
        "um app que ninguém arrumou já tem excepções a gravar"
    );

    let before = Layout {
        open: vec!["audio_mixer".into()],
        slots: vec![("audio_mixer".into(), Slot::LeftTop)],
        dock_w_left: Some(281.0),
        dock_w_right: Some(333.0),
        dock_h_bottom: Some(210.0),
    };
    install(&mut hero, &before);
    assert_eq!(current(&hero), before, "a volta ao ficheiro perdeu algo");
}

/// ⭐⭐⭐ **QUAIS PAINÉIS ESTAVAM ABERTOS volta também** — a metade que faltava.
///
/// ⛔ Sem ela a arrumação era **indistinguível de nenhuma**: a posição voltava certa e o painel que
/// o artista tinha movido nascia FECHADO, então o ecrã ao reabrir era o de fábrica. Foi exactamente
/// o que o 1.º smoke reportou (*«não funcionou. Voltou ao zero»*) — e o ficheiro estava certo.
#[test]
fn which_panels_were_open_comes_back_too() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut hero = ph2d_editor::HeroScreen::new(ph2d_editor::NodeId(3));
    assert!(
        !hero.is_panel_visible("audio_mixer"),
        "controlo: o mixer já nasce aberto e o gate mediria o default"
    );
    assert!(
        hero.is_panel_visible("inspector"),
        "controlo: o inspector já nasce fechado e a outra metade não seria medida"
    );

    install(
        &mut hero,
        &Layout {
            // ⚠️ A lista guarda a DIFERENÇA: uma entrada INVERTE o que o painel declara.
            open: vec!["audio_mixer".into(), "inspector".into()],
            ..Layout::default()
        },
    );
    assert!(
        hero.is_panel_visible("audio_mixer"),
        "um painel que o artista tinha aberto voltou fechado"
    );
    assert!(
        !hero.is_panel_visible("inspector"),
        "um painel que o artista tinha FECHADO voltou aberto — a lista só sabe abrir"
    );
}

/// ⚠️ **E a projecção grava a mesma diferença** — senão a volta perde-se na escrita.
#[test]
fn the_projection_writes_only_the_difference_from_what_each_panel_declares() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut hero = ph2d_editor::HeroScreen::new(ph2d_editor::NodeId(4));
    assert_eq!(
        current(&hero).open,
        Vec::<String>::new(),
        "um app que ninguém abriu nem fechou já tem painéis a gravar"
    );
    hero.panel_visibility.insert("audio_mixer", true);
    assert_eq!(current(&hero).open, vec!["audio_mixer".to_string()]);
    hero.panel_visibility.insert("audio_mixer", false);
    assert_eq!(
        current(&hero).open,
        Vec::<String>::new(),
        "fechar de volta deixou o painel na lista — o ficheiro cresce e nunca encolhe"
    );
}

/// ⛔ **As duas listas não se confundem no hash.**
#[test]
fn the_open_list_and_the_slot_list_do_not_collide_in_the_hash() {
    let a = Layout {
        open: vec!["x".into()],
        ..Layout::default()
    };
    let b = Layout {
        slots: vec![("x".into(), Slot::LeftTop)],
        ..Layout::default()
    };
    assert_ne!(hash(&a), hash(&b));
    let mut c = a.clone();
    c.open.push("y".into());
    assert_ne!(hash(&c), hash(&a), "abrir um painel não moveu o hash");
}

/// ⭐⭐ **O ficheiro inteiro: o layout activo e uma arrumação POR layout.**
#[test]
fn the_whole_file_survives_the_round_trip_with_one_arrangement_per_layout() {
    use ph2d_editor::screens::task_layout::TaskLayout;
    let mut v = Saved {
        active: Some(TaskLayout::Vector),
        ..Saved::default()
    };
    v.per_layout.insert(
        "vector".into(),
        Layout {
            open: vec!["vector".into()],
            dock_w_left: Some(280.0),
            ..Layout::default()
        },
    );
    v.per_layout.insert(
        "animation".into(),
        Layout {
            slots: vec![("timeline".into(), Slot::Bottom)],
            ..Layout::default()
        },
    );
    assert_eq!(parse_saved(&serialize_saved(&v)), v);
}

/// ⛔⛔ **A arrumação de um layout NÃO vaza para outro.**
#[test]
fn one_layouts_arrangement_never_leaks_into_another() {
    let text = "\
active=vector

[vector]
dock_w_left=280

[animation]
open.timeline=1
";
    let v = parse_saved(text);
    assert_eq!(v.per_layout["vector"].dock_w_left, Some(280.0));
    assert!(
        v.per_layout["animation"].dock_w_left.is_none(),
        "a largura do *Vector* apareceu no *Animate* — as secções fundiram-se"
    );
    assert_eq!(v.per_layout["animation"].open, vec!["timeline".to_string()]);
    assert!(v.per_layout["vector"].open.is_empty());
}

/// ⚠️ **Um layout que o artista nunca mexeu não tem secção** — e é isso que o deixa receber uma
/// mudança futura na tabela de fábrica.
#[test]
fn a_layout_nobody_touched_has_no_section() {
    let v = parse_saved("active=vector\n");
    assert!(v.per_layout.is_empty());
    assert_eq!(serialize_saved(&v).lines().count(), 2);
}

/// ⛔ **Um layout desconhecido cai no de omissão**, e não estraga o resto do ficheiro.
#[test]
fn an_unknown_active_layout_falls_back_without_poisoning_the_file() {
    let v = parse_saved("active=um_layout_de_2030\n\n[vector]\nopen.vector=1\n");
    assert_eq!(v.active, None, "um layout desconhecido virou o activo");
    assert_eq!(v.per_layout["vector"].open, vec!["vector".to_string()]);
}

/// ⛔⛔ **A arrumação dos OUTROS layouts sobrevive a uma gravação.**
///
/// ⚠️ Este gate nasceu de uma **mutação sobrevivente**: apagar o mapa antes de escrever deixava a
/// suíte verde, porque nada exercitava a composição com **dois** layouts. O artista arruma o
/// *Vector*, muda para o *Animate*, e perde o que fez no primeiro.
#[test]
fn composing_keeps_what_the_other_layouts_had() {
    use ph2d_editor::screens::task_layout::TaskLayout;
    let mut saved = Saved::default();
    saved.per_layout.insert(
        "vector".into(),
        Layout {
            open: vec!["vector".into()],
            ..Layout::default()
        },
    );

    let animate = Layout {
        open: vec!["timeline".into()],
        ..Layout::default()
    };
    compose(&mut saved, TaskLayout::Animation, animate.clone(), false);

    assert_eq!(saved.active, Some(TaskLayout::Animation));
    assert_eq!(saved.per_layout["animation"], animate);
    assert_eq!(
        saved.per_layout["vector"].open,
        vec!["vector".to_string()],
        "gravar o *Animate* apagou o que o artista fez no *Vector*"
    );

    // ⭐ E um layout devolvido ao de fábrica PERDE a secção, em vez de ficar com uma vazia.
    compose(&mut saved, TaskLayout::Animation, Layout::default(), true);
    assert!(
        !saved.per_layout.contains_key("animation"),
        "um layout reposto ficou com uma secção vazia — ele deixa de receber a tabela de fábrica"
    );
    assert!(saved.per_layout.contains_key("vector"));
}
