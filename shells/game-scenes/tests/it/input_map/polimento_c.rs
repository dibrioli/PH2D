//! R3-C (onda 2): a vista C no registo de controlos (plano §13.3) — receitas, Replace/Cancel, caixas, «Opens with»,
//! Forget e o polegar: hover no quadro só no pairado, premido no Down, arrastar para fora cancela, dispara no Up
//! dentro, o Tab chega com o anel, o Enter activa, o nó a11y com papel e nome, e o cursor.

use ph2d_app_input_map::TAB_ID;
use ph2d_app_input_map::View;
use ph2d_app_input_map::view_try::state::TryHit;
use ph2d_editor_core::doc_surface::{DocumentSurface, Key as K, PointerLook};
use ph2d_input::recipes::Recipe;

use super::Mesa;
use super::lista::onze;

fn key(n: &str) -> ph2d_input::Key {
    ph2d_input::Key::named_in_script(n).expect("tecla")[0]
}

fn mesa() -> Mesa {
    let mut m = Mesa::aberta(View::TryIt);
    m.hero.input_map = onze();
    m.frame();
    m
}

fn longe(m: &Mesa, id: ph2d_a11y::NodeId) -> (f32, f32) {
    let r = m.tab().doc.rect_of(id).expect("pintado");
    (r.x + r.w * 0.5, r.y - r.h * 4.0)
}

fn node(m: &Mesa, id: ph2d_a11y::NodeId) -> ph2d_a11y::Node {
    m.tab()
        .a11y(TAB_ID)
        .into_iter()
        .find(|(i, _)| *i == id)
        .map(|(_, n)| n)
        .expect("o nó do controlo")
}

/// Os alvos da C que se carregam, com o papel e o nome lido.
fn kinds(m: &Mesa) -> Vec<(TryHit, ph2d_a11y::Role, String)> {
    let jump = m.hero.input_map.id("jump").expect("jump");
    vec![
        (
            TryHit::Recipe(Recipe::Platformer),
            ph2d_a11y::Role::Button,
            "Start from Platformer".into(),
        ),
        (
            TryHit::PlayersCanChange,
            ph2d_a11y::Role::CheckBox,
            "Players can change keys".into(),
        ),
        (
            TryHit::Shown(jump),
            ph2d_a11y::Role::CheckBox,
            "jump".into(),
        ),
        (
            TryHit::OpenKey,
            ph2d_a11y::Role::Button,
            "Opens with F1".into(),
        ),
    ]
}

#[test]
fn cada_controlo_da_c_acende_so_ele_no_quadro() {
    let mut m = mesa();
    for (h, _, _) in kinds(&m) {
        let at = m.at(h.id());
        m.hover(at);
        assert_eq!(
            m.tab().doc.hover_live(h.id()),
            1.0,
            "{h:?} acende no quadro"
        );
        for (o, _, _) in kinds(&m) {
            if o != h {
                assert_eq!(m.tab().doc.hover_live(o.id()), 0.0, "{o:?} frio sob {h:?}");
            }
        }
        m.hover(longe(&m, h.id()));
        assert_eq!(m.tab().doc.hover_live(h.id()), 0.0, "{h:?} apaga no quadro");
    }
}

#[test]
fn cada_controlo_da_c_prime_no_down_cancela_fora_e_dispara_no_up_dentro() {
    let mut m = mesa();
    for (h, _, _) in kinds(&m) {
        let at = m.at(h.id());
        let before = (
            m.hero.input_map.clone(),
            m.tab().doc.tryit.listening_open_key,
        );
        assert!(m.down(at));
        assert!(m.tab().doc.is_pressed(h.id()), "{h:?} premido");
        m.hover(longe(&m, h.id()));
        assert!(
            !m.tab().doc.is_pressed(h.id()),
            "{h:?}: fora apaga o premido"
        );
        m.up(longe(&m, h.id()));
        let after = (
            m.hero.input_map.clone(),
            m.tab().doc.tryit.listening_open_key,
        );
        assert_eq!(after, before, "{h:?}: o Up fora cancela");
        assert_eq!(m.tab().doc.tryit.confirm, None, "{h:?}: nada pedido");
        m.click(at);
        let fired = m.hero.input_map != before.0
            || m.tab().doc.tryit.listening_open_key != before.1
            || m.tab().doc.tryit.confirm.is_some();
        assert!(fired, "{h:?}: o Up dentro dispara");
        // Desfaz o gesto para o seguinte partir do mesmo sítio.
        m.hero.input_map = before.0;
        m.tab_mut().doc.tryit.listening_open_key = false;
        m.tab_mut().doc.tryit.confirm = None;
        m.frame();
    }
}

#[test]
fn o_tab_chega_a_cada_controlo_da_c_e_o_enter_activa() {
    let mut m = mesa();
    let id = TryHit::Recipe(Recipe::Platformer).id();
    let mut seen = false;
    for _ in 0..64 {
        m.key(K::Tab, None);
        if m.tab().doc.focused() == Some(id) {
            seen = true;
            break;
        }
    }
    assert!(seen, "o Tab chega à receita");
    m.key(K::Enter, None);
    assert_eq!(
        m.tab().doc.tryit.confirm,
        Some(Recipe::Platformer),
        "o Enter activa a receita focada"
    );
}

#[test]
fn as_teclas_ao_vivo_sao_so_leitura_fora_do_tab() {
    let mut m = mesa();
    let w = ph2d_app_input_map::view_keyboard::state::KbHit::Key(key("w")).id();
    assert!(
        m.tab().doc.rect_of(w).is_some(),
        "a tecla W está pintada na C"
    );
    let _ = node(&m, w);
    for _ in 0..200 {
        m.key(K::Tab, None);
        assert_ne!(
            m.tab().doc.focused(),
            Some(w),
            "a tecla W não recebe foco na C"
        );
    }
}

#[test]
fn cada_controlo_da_c_tem_no_a11y_e_cursor() {
    let m = mesa();
    for (h, role, label) in kinds(&m) {
        let n = node(&m, h.id());
        assert_eq!(n.role(), role, "{h:?}");
        assert_eq!(n.label(), Some(label.as_str()), "{h:?}");
        let (x, y) = m.at(h.id());
        assert_eq!(
            m.tab().pointer_look(TAB_ID, x, y),
            Some(PointerLook::Hand),
            "{h:?}"
        );
    }
}

#[test]
fn replace_e_cancel_premem_tem_nos_e_esc_enter_respondem() {
    let mut m = mesa();
    let before = m.hero.input_map.clone();
    m.click(m.at(TryHit::Recipe(Recipe::Shooter).id()));
    for (h, label) in [(TryHit::Replace, "Replace"), (TryHit::Cancel, "Cancel")] {
        let n = node(&m, h.id());
        assert_eq!(n.role(), ph2d_a11y::Role::Button);
        assert_eq!(n.label(), Some(label));
        let at = m.at(h.id());
        m.down(at);
        assert!(m.tab().doc.is_pressed(h.id()), "{h:?} premido");
        m.hover(longe(&m, h.id()));
        m.up(longe(&m, h.id()));
        assert_eq!(
            m.tab().doc.tryit.confirm,
            Some(Recipe::Shooter),
            "{h:?}: fora cancela"
        );
    }
    m.physical(key("escape"));
    assert_eq!(m.tab().doc.tryit.confirm, None, "Esc = Cancel");
    assert_eq!(m.hero.input_map, before);
    m.click(m.at(TryHit::Recipe(Recipe::Shooter).id()));
    m.physical(key("enter"));
    m.frame();
    assert_eq!(m.tab().doc.tryit.confirm, None, "Enter = Replace");
    assert_ne!(m.hero.input_map, before, "o Enter substituiu");
}

#[test]
fn o_forget_e_uma_ligacao_que_sublinha_e_dispara() {
    let mut m = mesa();
    m.tab_mut().doc.tryit.player_changed = 2;
    m.frame();
    let id = TryHit::Forget.id();
    let n = node(&m, id);
    assert_eq!(n.role(), ph2d_a11y::Role::Link);
    assert_eq!(n.label(), Some("Forget them"));
    let (x, y) = m.at(id);
    assert_eq!(m.tab().pointer_look(TAB_ID, x, y), Some(PointerLook::Hand));
    m.hover((x, y));
    assert_eq!(m.tab().doc.hover_live(id), 1.0);
}

#[test]
fn o_anel_da_escuta_pulsa_e_fica_parado_com_reduced_motion() {
    let mut m = mesa();
    m.hero
        .motion
        .set_character(ph2d_editor_core::motion::UiCharacter::Expressive);
    m.click(m.at(TryHit::OpenKey.id()));
    assert!(m.tab().doc.tryit.listening_open_key);
    assert!(m.tab().wants_frame(), "à escuta, o pulso pede quadro");
    m.hero.motion.set_reduced_motion(true);
    m.frame();
    assert!(
        !m.tab().wants_frame(),
        "reduced motion: anel parado, sem quadro"
    );
    assert!(
        !m.tab().doc.census.dashed.is_empty(),
        "o anel tracejado continua lá"
    );
}

#[test]
fn o_polegar_da_c_e_uma_barra_que_desvanece() {
    let mut m = mesa();
    m.paint_in(ph2d_editor_core::zones::Rect::new(0.0, 0.0, 1280.0, 720.0)); // LITERAL-PX-OK: a janela do plano §8
    let id = ph2d_app_input_map::view_try::THUMB;
    let n = node(&m, id);
    assert_eq!(n.role(), ph2d_a11y::Role::ScrollBar);
    assert_eq!(m.tab().doc.thumb_fade(id), 0.0, "frio sem o ponteiro");
}

/// O censo da C nas áreas REAIS da aba (1600×1000 e 1280×720), nos dois modos de texto, com a pergunta aberta,
/// à escuta e com o Forget à vista: zero texto cortado.
#[test]
fn o_censo_da_c_fica_a_zero_nas_areas_reais() {
    use ph2d_editor_core::zones::Rect;
    // LITERAL-PX-OK: as áreas reais da aba (lição 1 da missão)
    let areas = [
        Rect::new(308.0, 28.0, 988.0, 890.0),
        Rect::new(297.0, 28.0, 690.0, 604.0),
    ];
    super::em_cada_modo_de_texto(|| {
        for area in areas {
            let mut m = mesa();
            m.tab_mut().doc.tryit.player_changed = 3;
            m.tab_mut().doc.tryit.confirm = Some(Recipe::Shooter);
            m.tab_mut().doc.tryit.listening_open_key = true;
            m.frame();
            m.paint_in(area);
            let c = &m.tab().doc.census;
            assert_eq!(c.elided(), Vec::<&str>::new(), "{area:?}: texto cortado");
        }
    });
}

// ── R4: a precedência das teclas (o diálogo e a escuta antes do foco) e o polegar que se arrasta ────────────────

/// A pergunta da receita nasce com o foco no Cancel (o seguro); o Tab e o Shift+Tab giram SÓ entre Replace e
/// Cancel; o Esc com o foco lá fecha a pergunta (antes, só largava o foco); o Enter é o Replace mesmo no Cancel.
#[test]
fn o_dialogo_da_receita_prende_o_foco_e_nasce_no_cancel() {
    let mut m = mesa();
    let before = m.hero.input_map.clone();
    let (replace, cancel) = (TryHit::Replace.id(), TryHit::Cancel.id());
    m.click(m.at(TryHit::Recipe(Recipe::Shooter).id()));
    assert_eq!(
        m.tab().doc.focused(),
        Some(cancel),
        "o foco nasce no Cancel"
    );
    for want in [replace, cancel, replace] {
        m.physical(key("tab"));
        assert_eq!(
            m.tab().doc.focused(),
            Some(want),
            "o Tab gira só no diálogo"
        );
    }
    m.key(K::Tab, None);
    assert_eq!(
        m.tab().doc.focused(),
        Some(cancel),
        "pela porta do texto também"
    );
    m.physical(key("left_shift"));
    m.physical(key("tab"));
    assert_eq!(m.tab().doc.focused(), Some(replace), "Shift+Tab volta");
    let _ = m.tab_mut().doc.raw_key(key("left_shift"), false, false);
    m.physical(key("escape"));
    assert_eq!(
        m.tab().doc.tryit.confirm,
        None,
        "o Esc com foco fecha a pergunta"
    );
    assert_eq!(m.tab().doc.focused(), None, "o foco sai com ela");
    assert_eq!(m.hero.input_map, before);
    m.click(m.at(TryHit::Recipe(Recipe::Shooter).id()));
    assert_eq!(m.tab().doc.focused(), Some(cancel));
    m.physical(key("enter"));
    m.frame();
    assert_eq!(m.tab().doc.tryit.confirm, None);
    assert_ne!(
        m.hero.input_map, before,
        "o Enter é o Replace, com o foco no Cancel"
    );
    m.click(m.at(TryHit::Recipe(Recipe::Platformer).id()));
    m.physical(key("space"));
    assert_eq!(
        m.tab().doc.tryit.confirm,
        None,
        "o Space activa o focado: o Cancel"
    );
}

/// À escuta do «Opens with», a escuta vem antes do foco: o Tab é uma tecla (vira a tecla que abre a tela) e
/// o foco não anda.
#[test]
fn a_escuta_do_opens_with_toma_o_tab_antes_do_foco() {
    let mut m = mesa();
    let id = TryHit::OpenKey.id();
    for _ in 0..64 {
        m.key(K::Tab, None);
        if m.tab().doc.focused() == Some(id) {
            break;
        }
    }
    assert_eq!(m.tab().doc.focused(), Some(id), "o Tab chega ao Opens with");
    m.key(K::Enter, None);
    assert!(m.tab().doc.tryit.listening_open_key, "à escuta");
    m.physical(key("escape"));
    assert!(
        !m.tab().doc.tryit.listening_open_key,
        "o Esc cancela a escuta, como em toda a aba"
    );
    assert_eq!(m.tab().doc.focused(), Some(id), "e não larga o foco");
    assert_eq!(m.hero.input_map.controls().open_key, key("f1"));
    m.key(K::Enter, None);
    assert!(m.tab().doc.tryit.listening_open_key, "de novo à escuta");
    m.physical(key("tab"));
    assert!(
        !m.tab().doc.tryit.listening_open_key,
        "a escuta tomou o Tab"
    );
    assert_eq!(m.hero.input_map.controls().open_key, key("tab"));
    assert_eq!(m.tab().doc.focused(), Some(id), "e o foco não andou");
}

/// O polegar da C arrasta como o da A e o da B: na área real a 1280×720 a coluna rola com o ponteiro.
#[test]
fn o_polegar_da_c_arrasta() {
    use ph2d_editor_core::zones::Rect;
    let mut m = mesa();
    let area = Rect::new(297.0, 28.0, 690.0, 604.0); // LITERAL-PX-OK: a área real da aba a 1280×720
    m.paint_in(area);
    let thumb = m
        .tab()
        .doc
        .rect_of(ph2d_app_input_map::view_try::THUMB)
        .expect("a 1280 a coluna rola");
    let (x, y) = (thumb.x + thumb.w * 0.5, thumb.y + thumb.h * 0.5);
    let doc = &mut m.tab_mut().doc;
    assert!(doc.press(x, y));
    assert_eq!(doc.tryit.scroll, 0.0);
    assert!(doc.motion(x, y + thumb.h), "arrastar rola");
    let s = doc.tryit.scroll;
    assert!(s > 0.0, "a coluna desceu: {s}");
    doc.release_at(x, y + thumb.h);
    assert_eq!(doc.tryit.thumb_drag, None, "soltar larga o polegar");
    assert!(
        !doc.motion(x, y + thumb.h * 2.0),
        "sem botão, mover não rola"
    );
    assert_eq!(doc.tryit.scroll, s);
}

// ── F2 (onda 2 W6): o acabamento da C e da tela do jogador ────────────────────────────────────────────────────────

/// As áreas REAIS da aba a 1600×1000 e a 1280×720.
fn areas() -> [ph2d_editor_core::zones::Rect; 2] {
    use ph2d_editor_core::zones::Rect;
    // LITERAL-PX-OK: as áreas reais da aba (lição 1 da missão)
    [
        Rect::new(308.0, 28.0, 988.0, 890.0),
        Rect::new(297.0, 28.0, 690.0, 604.0),
    ]
}

fn text_of(m: &Mesa, s: &str) -> ph2d_editor_core::zones::Rect {
    (m.tab().doc.census.texts.iter())
        .find(|t| t.1 == s)
        .unwrap_or_else(|| panic!("«{s}» pintado"))
        .0
}

/// O Esc à escuta do «Opens with» CANCELA (como em toda a aba): a tecla fica a de antes e nada se recusa; o F5
/// continua recusado com o porquê e a escuta continua.
#[test]
fn o_esc_cancela_a_escuta_do_opens_with_e_o_f5_fica_recusado() {
    let mut m = mesa();
    let before = m.hero.input_map.controls().open_key;
    m.click(m.at(TryHit::OpenKey.id()));
    assert!(m.tab().doc.tryit.listening_open_key);
    m.physical(key("escape"));
    let doc = &m.tab().doc;
    assert!(!doc.tryit.listening_open_key, "o Esc cancela");
    assert_eq!(doc.tryit.refused, None, "e não é uma recusa");
    assert_eq!(m.hero.input_map.controls().open_key, before);
    m.click(m.at(TryHit::OpenKey.id()));
    m.physical(key("f5"));
    assert!(
        m.tab().doc.tryit.listening_open_key,
        "o F5 recusado, à escuta"
    );
    assert!(m.tab().doc.tryit.refused.is_some());
    assert_eq!(m.hero.input_map.controls().open_key, before);
}

/// As receitas são colunas IGUAIS que enchem o cartão (sem a cauda vazia de 270 px a 1600); no estreito partem-se em
/// menos colunas, iguais também.
#[test]
fn as_receitas_sao_colunas_iguais_que_enchem_o_cartao() {
    for area in areas() {
        let mut m = mesa();
        m.paint_in(area);
        let doc = &m.tab().doc;
        let tiles: Vec<_> = (Recipe::ALL.iter())
            .map(|r| doc.rect_of(TryHit::Recipe(*r).id()).expect("o ladrilho"))
            .collect();
        let left = text_of(&m, ph2d_i18n::tr("input_map_try.start_from")).x;
        let stopped = text_of(&m, ph2d_i18n::tr("input_map_try.stopped"));
        let right = stopped.x + stopped.w;
        let cols = tiles
            .iter()
            .filter(|t| (t.y - tiles[0].y).abs() < 0.5)
            .count();
        assert!(cols > 1, "{area:?}: mais de uma coluna");
        for t in &tiles {
            assert!(
                (t.w - tiles[0].w).abs() < 0.5,
                "{area:?}: colunas iguais {t:?}"
            );
        }
        assert!(
            (tiles[0].x - left).abs() < 0.5,
            "{area:?}: encosta à esquerda"
        );
        let last = tiles[cols - 1];
        assert!(
            (last.x + last.w - right).abs() < 0.5,
            "{area:?}: a última coluna acaba no recuo direito ({} × {right})",
            last.x + last.w
        );
    }
}

/// «Shown in the game» numa grelha certa: colunas da MESMA largura, caixas da mesma altura, o vão das métricas nas
/// duas direcções e filas equilibradas (nunca uma caixa órfã sozinha numa fila); o «Opens with F1» no chip de tecla.
#[test]
fn as_caixas_do_shown_sao_uma_grelha_e_o_opens_with_e_um_chip_de_tecla() {
    use ph2d_app_input_map::view_try::{KEY_CHIP_H, SHOWN_GAP};
    const ROW_H: f32 = 28.0; // LITERAL-PX-OK: FIELD_H, a altura da pílula
    let m = super::try_it_censo::cena("C4");
    let doc = &m.tab().doc;
    let cells: Vec<_> = (m.hero.input_map.actions().iter())
        .map(|a| doc.rect_of(TryHit::Shown(a.id).id()).expect("a caixa"))
        .collect();
    assert!(cells.len() > 2);
    let cols = cells
        .iter()
        .filter(|c| (c.y - cells[0].y).abs() < 0.5)
        .count();
    // O alvo da caixa vive dentro da pílula (o recuo dela dos dois lados): o passo entre alvos é o da grelha.
    let (step_x, step_y) = (cells[0].w + SHOWN_GAP, cells[0].h + SHOWN_GAP);
    let inset = (ROW_H - cells[0].h) * 0.5;
    for (i, c) in cells.iter().enumerate() {
        assert!((c.w - cells[0].w).abs() < 0.5, "largura igual: {c:?}");
        assert!((c.h - cells[0].h).abs() < 0.5, "altura igual: {c:?}");
        if i % cols > 0 {
            let step = c.x - cells[i - 1].x;
            let pill = cells[0].w + (step - cells[0].w - SHOWN_GAP);
            assert!(
                step >= step_x - 0.5 && (step - (pill + SHOWN_GAP)).abs() < 0.5,
                "passo {step}"
            );
            assert!(
                (step - (cells[1].x - cells[0].x)).abs() < 0.5,
                "o mesmo passo: {step}"
            );
        }
        if i >= cols {
            let step = c.y - cells[i - cols].y;
            assert!(
                (step - (step_y + inset * 2.0)).abs() < 0.5,
                "passo vertical {step}"
            );
        }
    }
    let tail = cells.len() % cols;
    assert!(
        tail == 0 || tail * 2 >= cols,
        "a última fila não fica órfã: {tail} de {cols}"
    );
    let open = doc.rect_of(TryHit::OpenKey.id()).expect("o chip");
    assert!(
        (open.h - KEY_CHIP_H).abs() < 0.5,
        "o chip de tecla: {open:?}"
    );
}

/// A manche: o ponto ao vivo na proporção da maquete C3 (14 px num círculo de 138), um vão de linha do cartão entre
/// o título e o «Dead zone», e o disco Dead com contraste de token sobre o círculo.
#[test]
fn a_manche_tem_o_ponto_da_maquete_e_o_titulo_respira() {
    use ph2d_app_input_map::view_try::stick::{DEAD_INK, DOT_OF_RADIUS, OUTER_INK};
    use ph2d_tokens::Theme;
    assert!(
        (DOT_OF_RADIUS - 7.0 / 69.0).abs() < 0.01,
        "o ponto da maquete"
    ); // LITERAL-PX-OK: a maquete C3
    let lum = |t: ph2d_tokens::ColorToken| {
        let c = t.resolve(Theme::default());
        let ch = |v: u8| {
            let v = f32::from(v) / 255.0;
            if v <= 0.040_45 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * ch(c.r) + 0.7152 * ch(c.g) + 0.0722 * ch(c.b)
    };
    let ratio = (lum(DEAD_INK) + 0.05) / (lum(OUTER_INK) + 0.05);
    assert!(
        ratio >= 2.2,
        "o disco Dead destaca-se do círculo (F3): {ratio}"
    );
    let m = super::try_it_censo::cena("C3");
    let doc = &m.tab().doc;
    let a = (m.hero.input_map.actions().iter())
        .find(|a| a.is_analog())
        .expect("a manche");
    let title = (doc.census.texts.iter())
        .find(|t| t.1.starts_with(&a.name) && t.1.contains('\u{00B7}'))
        .expect("o título")
        .0;
    let dead = text_of(&m, ph2d_i18n::tr("input_map_list.dead"));
    let gap = dead.y - (title.y + title.h);
    assert!(gap >= 5.5, "o vão título × Dead: {gap}"); // LITERAL-PX-OK: FIELD_ROW_GAP (6) menos meio px
}

/// A hierarquia do texto da C: corpo, chips e dicas no Regular; o SemiBold só nos destaques (o título da receita,
/// «You are pressing …:», a leitura da manche da maquete C3) e nos nomes de acção acesos. Nas duas áreas reais, nos dois modos de texto.
#[test]
fn o_negrito_da_c_e_so_dos_destaques_e_dos_nomes_de_accao() {
    use ph2d_i18n::tr;
    super::em_cada_modo_de_texto(|| {
        for area in areas() {
            let mut m = mesa();
            m.tab_mut().doc.tryit.player_changed = 2;
            m.physical(key("up"));
            m.frame();
            m.paint_in(area);
            let c = &m.tab().doc.census;
            let leads: Vec<&str> = Recipe::ALL.iter().map(|r| tr(r.label_key())).collect();
            let names: Vec<String> = (m.hero.input_map.actions().iter())
                .map(|a| a.name.clone())
                .collect();
            let lead_of_press = tr("input_map_try.pressing").replace("{keys}", "");
            let reading: Vec<&str> = c
                .runs
                .iter()
                .filter(|r| r.2)
                .map(|r| r.0.as_str())
                .collect();
            for s in &c.semibold {
                assert!(
                    leads.contains(&s.as_str())
                        || reading.contains(&s.as_str())
                        || names.contains(s)
                        || s.starts_with(lead_of_press.trim_end_matches(':').trim_end()),
                    "{area:?}: «{s}» em negrito sem ser destaque nem nome de acção"
                );
            }
            for body in [tr("input_map_try.opens_with"), tr("input_map_try.forget")] {
                assert!(
                    c.regular.iter().any(|s| s == body),
                    "{area:?}: «{body}» no Regular"
                );
            }
            assert_eq!(c.elided(), Vec::<&str>::new(), "{area:?}: texto cortado");
        }
    });
}

/// F3 (C2): a grelha dos medidores tem uma calha de verdade entre colunas — o valor de uma coluna fica a pelo menos
/// o vão dos cartões do quadradinho da coluna seguinte. Nas duas áreas reais, nos dois modos de texto.
#[test]
fn os_medidores_tem_uma_calha_entre_colunas() {
    use ph2d_app_input_map::view_try::{METER_GUTTER, METER_NAME_X};
    super::em_cada_modo_de_texto(|| {
        for area in areas() {
            let mut m = mesa();
            m.paint_in(area);
            let c = &m.tab().doc.census;
            let names: Vec<String> = (m.hero.input_map.actions().iter())
                .map(|a| a.name.clone())
                .collect();
            let find = |s: &str| -> Vec<ph2d_editor_core::zones::Rect> {
                (c.texts.iter()).filter(|t| t.1 == s).map(|t| t.0).collect()
            };
            let values = find("0.00");
            assert!(
                values.len() >= names.len(),
                "{area:?}: um valor por acção (mais os 0.00 da manche)"
            );
            let mut seguintes = 0;
            for n in &names {
                for nr in find(n) {
                    let dot = nr.x - METER_NAME_X;
                    let antes = (values.iter())
                        .filter(|v| (v.y - nr.y).abs() < 0.5 && v.x < nr.x)
                        .map(|v| v.x + v.w)
                        .fold(f32::NEG_INFINITY, f32::max);
                    if antes.is_finite() {
                        seguintes += 1;
                        assert!(
                            dot - antes >= METER_GUTTER - 0.5,
                            "{area:?}: «{n}» — o ponto a {} px do valor à esquerda",
                            dot - antes
                        );
                    }
                }
            }
            assert!(seguintes > 0, "{area:?}: há mais de uma coluna");
        }
    });
}
