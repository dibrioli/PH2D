//! A ida-e-volta do ficheiro, e as três coisas que ele tem de recusar.

use super::*;

fn sample() -> Layout {
    Layout {
        slots: vec![
            ("audio_mixer".into(), Slot::LeftTop),
            ("physics".into(), Slot::RightBottom),
        ],
        dock_w_left: Some(280.0),
        dock_w_right: Some(320.5),
    }
}

#[test]
fn a_layout_survives_the_round_trip() {
    let l = sample();
    assert_eq!(parse(&serialize(&l)), l);
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
            slots: vec![
                // Legal: uma coluna de propriedades aceita as duas colunas.
                ("audio_mixer".into(), Slot::LeftTop),
                // ⛔ ILEGAL: o Inspector declara `SIDES`; a faixa de baixo não é dele.
                ("inspector".into(), Slot::Bottom),
                // Um painel que não existe nesta build.
                ("um_painel_de_2030".into(), Slot::RightTop),
            ],
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
        slots: vec![("audio_mixer".into(), Slot::LeftTop)],
        dock_w_left: Some(281.0),
        dock_w_right: Some(333.0),
    };
    install(&mut hero, &before);
    assert_eq!(current(&hero), before, "a volta ao ficheiro perdeu algo");
}
