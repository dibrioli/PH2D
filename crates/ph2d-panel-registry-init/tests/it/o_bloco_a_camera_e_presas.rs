//! ⭐ **NovaUI, bloco A** (`docs/CartoesUI/05` §8 e §9, `10_TODO_O_APP_TRACKER`): no hero montado, o
//! cartão Game Camera tem o corpo do cartão e as caixas e listas de um cartão preso mostram e editam o
//! objecto dele.
#![cfg(feature = "panel-inspector")]

use super::cartoes_mao::{centro, clica, mao, pinta};
use super::o_bloco8_player_solto::{hero_com_aberto, solto_de};
use super::o_inspector_armado::{camera_armada, desarma_tudo, script_armado};
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{loose_pin_of, loose_rect_of};
use ph2d_editor_core::screens::hero::{CameraFieldEdit, HeroScreen, InspectorNameInfo};
use ph2d_editor_core::script_edits::{InspectorScriptProp, InspectorScriptValue};
use ph2d_host::PointerKind;
use ph2d_panel_inspector as insp;
use ph2d_panel_inspector::ids as pids;

const CAMERA: NodeId = ids::INSP_LIVE_CAMERA_SECTION;
const SCRIPT: NodeId = ids::INSP_LIVE_SCRIPT_SECTION;
const HEROI: u64 = super::o_inspector_armado::BITS;
const INIMIGO: u64 = 77;

fn objeto(bits: u64, nome: &str) {
    desarma_tudo();
    insp::set_current_inspector_sprite(None);
    insp::set_current_inspector_transform(Some(super::o_bloco5_abertos::objeto(bits, 2.0)));
    insp::set_current_inspector_name(Some(InspectorNameInfo {
        entity_bits: bits,
        name: nome.to_owned(),
    }));
}

fn camera(bits: u64, nome: &str, ativa: bool) {
    objeto(bits, nome);
    let mut c = camera_armada();
    c.entity_bits = bits;
    c.camera.active = ativa;
    insp::set_current_inspector_camera(Some(c));
}

/// A linha do Enum com `opcoes`: o script do `bits`.
fn script(bits: u64, nome: &str, opcoes: &[&str]) {
    objeto(bits, nome);
    let mut s = script_armado();
    s.entity_bits = bits;
    s.orphans.clear();
    s.props = vec![InspectorScriptProp {
        name: "mood".to_owned(),
        value: InspectorScriptValue::Text(opcoes[0].to_owned()),
        own: false,
        options: opcoes.iter().map(|&o| o.to_owned()).collect(),
        min: None,
        max: None,
        step: None,
        canais: Vec::new(),
    }];
    insp::set_current_inspector_script(Some(s));
}

/// O rect de `id`, rolando a lista até ele estar à vista (a janela do teste mede 900 px).
fn rolado_ate(h: &mut HeroScreen, id: NodeId) -> ph2d_editor_core::zones::Rect {
    let painel = ids::INSP_PANEL;
    let mut rolar = 0.0;
    while rolar < 30_000.0 {
        // Inteiro: a borda da janela recorta o rect, e um rect recortado não mede a caixa.
        if let Some(r) = h.hit_index.rect_for(id)
            && r.y + r.h + m::FIELD_H < h.last_layout.expect("houve um quadro").canvas.h
        {
            return r;
        }
        rolar += 150.0;
        h.store.set_panel_scroll(painel, rolar);
        h.store.set_panel_scroll_live(painel, rolar);
        pinta(h, 3);
    }
    panic!("{id:?} não está no cartão");
}

/// Foto `r4_camera`: Frame + Icon Size e Outside + Editor View são irmãos curtos, dividem a linha, e o
/// texto dos botões cabe. *Mutação: `w_par = w` ⇒ vermelho.*
#[test]
fn the_view_rows_of_the_camera_share_the_line() {
    camera(HEROI, "Camera", true);
    let mut h = hero_com_aberto(CAMERA);
    let frame = rolado_ate(&mut h, pids::INSP_CAMERA_FRAME[0]);
    let icone = rolado_ate(&mut h, pids::INSP_CAMERA_ICON_SIZE);
    let fora = rolado_ate(&mut h, pids::INSP_CAMERA_OUTSIDE[0]);
    let vista = rolado_ate(&mut h, pids::INSP_CAMERA_VIEW[0]);
    let centro = |r: ph2d_editor_core::zones::Rect| r.y + r.h * 0.5;
    assert!((centro(frame) - centro(icone)).abs() < 0.5, "{frame:?} × {icone:?}");
    assert!(icone.x > frame.x + frame.w, "{frame:?} × {icone:?}");
    assert!((centro(fora) - centro(vista)).abs() < 0.5, "{fora:?} × {vista:?}");
    assert!(vista.x > fora.x + fora.w, "{fora:?} × {vista:?}");
    let (_, medidos) = ph2d_editor_core::text_elide::elisao::medindo(|| pinta(&mut h, 1));
    for nome in ["Always", "Selected", "Normal", "Dimmed", "Free", "Follows"] {
        assert!(
            medidos.iter().filter(|e| e.texto == nome).all(|e| e.coube()),
            "{nome:?} saiu cortado"
        );
    }
    desarma_tudo();
}

/// ⭐ 05 §8: o corpo do Camera é o do cartão. A lista e o alvo levam a caixa do cartão a toda a
/// largura com o rótulo EM CIMA; o conta-gotas mora dentro da caixa do alvo; o par de caixas não
/// corta o nome e a marca da esquerda fica colada ao nome dela, longe da caixa da direita.
/// *Mutação: `caixa_do_cartao` sem o `KEY_MARK_BOX`, ou a metade inteira à esquerda.*
#[test]
fn the_camera_card_body_has_the_face_of_a_card() {
    camera(HEROI, "Camera", true);
    let mut h = hero_com_aberto(CAMERA);
    let cartao = h
        .hit_index
        .rect_for(ids::grip_de(CAMERA))
        .expect("o título do cartão Camera");
    let formato = rolado_ate(&mut h, pids::INSP_CAMERA_FORMAT_PICK);
    let active = rolado_ate(&mut h, pids::INSP_CAMERA_ACTIVE);
    let look = rolado_ate(&mut h, pids::INSP_CAMERA_PREVIEW);
    let (_, medidos) = ph2d_editor_core::text_elide::elisao::medindo(|| pinta(&mut h, 1));
    let alvo = rolado_ate(&mut h, pids::INSP_CAMERA_TARGET);
    let gotas = rolado_ate(&mut h, pids::INSP_CAMERA_TARGET_PICK);
    for r in [formato, alvo] {
        assert!(
            (r.h - m::FIELD_H).abs() < 0.5,
            "não é a caixa do cartão: {r:?}"
        );
        assert!(
            r.w > cartao.w * 0.75,
            "o rótulo ainda come a coluna da esquerda: {r:?} em {cartao:?}"
        );
    }
    assert!(
        gotas.x >= alvo.x && gotas.x + gotas.w <= alvo.x + alvo.w,
        "o conta-gotas saiu da caixa do alvo: {gotas:?} × {alvo:?}"
    );
    assert!(
        gotas.y >= alvo.y && gotas.y + gotas.h <= alvo.y + alvo.h,
        "o conta-gotas não está na altura da caixa do alvo: {gotas:?} × {alvo:?}"
    );
    assert!(
        (active.y - look.y).abs() < 0.5,
        "o par não partilha a linha"
    );
    assert!(
        (active.w - look.w).abs() < 0.5 && look.x - (active.x + active.w) >= m::FIELD_GAP - 0.5,
        "o par não está em duas colunas iguais (05 §4): {active:?} × {look:?}"
    );
    for nome in ["Active", "Look Through"] {
        assert!(
            medidos.iter().any(|e| e.texto == nome)
                && medidos
                    .iter()
                    .filter(|e| e.texto == nome)
                    .all(|e| e.coube()),
            "{nome:?} saiu cortado: {medidos:?}"
        );
    }
    desarma_tudo();
}

/// O Camera solto preso ao Herói, com o Inimigo escolhido. Devolve o hero e o rect do solto.
fn preso(
    sec: NodeId,
    arma: impl Fn(u64, &str) + Copy,
) -> (HeroScreen, ph2d_editor_core::zones::Rect) {
    arma(HEROI, "Hero");
    let mut h = hero_com_aberto(sec);
    solto_de(&mut h, sec, 300.0);
    if let Some(c) = h.store.loose_card_mut(sec) {
        c.at.1 += 150.0;
    }
    pinta(&mut h, 3);
    clica(&mut h, loose_pin_of(sec));
    assert_eq!(h.store.loose_card(sec).and_then(|c| c.pin), Some(HEROI));
    arma(INIMIGO, "Enemy");
    quadros(&mut h, arma, 5);
    let r = h
        .store
        .panel_rect(loose_rect_of(sec))
        .expect("o preso fica no canvas");
    (h, r)
}

fn quadros(h: &mut HeroScreen, arma: impl Fn(u64, &str) + Copy, n: usize) {
    for _ in 0..n {
        insp::com_cada_preso(&h.store.loose_pins_of(ids::INSP_PANEL), |_| {
            arma(HEROI, "Hero");
        });
        pinta(h, 1);
    }
}

fn edicoes(h: &mut HeroScreen) -> Vec<(u64, CameraFieldEdit)> {
    h.bus
        .drain()
        .filter_map(|a| match a {
            EditorAction::InspectorCameraEdit { entity_bits, edit } => Some((entity_bits, edit)),
            _ => None,
        })
        .collect()
}

/// ⭐ 05 §9: a CAIXA de um cartão preso mostra o valor do objecto dele (o Herói está ligado, o
/// Inimigo escolhido desligado) e o clique edita-o a ele, com o valor oposto ao que está à vista.
/// *Mutação: `com_retrato` com a seleção ⇒ o pedido vai ao Inimigo com o valor errado.*
#[test]
fn a_pinned_card_check_shows_and_edits_its_own_object() {
    let arma = |b, n: &str| camera(b, n, b == HEROI);
    let (mut h, r) = preso(CAMERA, arma);
    let caixa = h
        .hit_index
        .rect_for(pids::INSP_CAMERA_ACTIVE)
        .expect("Active no cartão preso");
    let c = centro(caixa);
    assert!(r.contains(c.0, c.1), "a caixa pintada não é a do preso");
    let _ = h.bus.drain().count();
    mao(&mut h, PointerKind::Down, c);
    assert_eq!(h.store.card_press_subject(), Some(HEROI));
    mao(&mut h, PointerKind::Up, c);
    let feitas = edicoes(&mut h);
    assert!(
        matches!(feitas.as_slice(), [(HEROI, CameraFieldEdit::Active(false))]),
        "a caixa do preso não editou o Herói (ligado ⇒ desligar): {feitas:?}"
    );
    desarma_tudo();
}

/// ⭐ 05 §9: a LISTA de um cartão preso mostra as opções do objecto dele (o script do Herói tem
/// três, o do Inimigo escolhido tem uma) e a opção, MESMO fora do rect do solto, escolhe para ele.
/// *Mutação: `toma_sujeito_da_lista` ⇒ `None` (uma opção só, as do Inimigo); a guarda da lista em
/// `note_card_press` fora ⇒ a opção fora do solto vai à seleção.*
#[test]
fn a_pinned_card_list_shows_and_edits_its_own_object() {
    let arma = |b, n: &str| {
        if b == HEROI {
            script(b, n, &["calm", "angry", "sad"]);
        } else {
            script(b, n, &["only"]);
        }
    };
    let (mut h, r) = preso(SCRIPT, arma);
    let chip = pids::INSP_SCRIPT_ENUM[0];
    let c = centro(h.hit_index.rect_for(chip).expect("o chip no cartão preso"));
    assert!(r.contains(c.0, c.1), "o chip pintado não é o do preso");
    mao(&mut h, PointerKind::Down, c);
    mao(&mut h, PointerKind::Up, c);
    quadros(&mut h, arma, 3);
    let terceira = pids::INSP_SCRIPT_ENUM_OPT[2];
    let opcao = h
        .hit_index
        .rect_for(terceira)
        .expect("a lista do preso não tem as opções do Herói (as do Inimigo são uma só)");
    // Leve a opção para fora do rect do solto: encolhe-o publicando-o acima da lista.
    let p = centro(opcao);
    h.store.set_panel_rect(
        loose_rect_of(SCRIPT),
        ph2d_editor_core::zones::Rect::new(r.x, r.y, r.w, (p.1 - r.y - 40.0).max(1.0)),
    );
    mao(&mut h, PointerKind::Down, p);
    assert_eq!(
        h.store.card_press_subject(),
        Some(HEROI),
        "a opção fora do rect do solto caiu na seleção"
    );
    let _ = h.bus.drain().count();
    mao(&mut h, PointerKind::Up, p);
    let alvos: Vec<u64> = h
        .bus
        .drain()
        .filter_map(|a| match a {
            EditorAction::InspectorComponentEdit { entity_bits, .. } => Some(entity_bits),
            _ => None,
        })
        .collect();
    assert_eq!(alvos, [HEROI], "a opção do preso editou outro objecto");
    desarma_tudo();
}

/// ⭐ T29: toda linha de lista suspensa de um componente, no cartão, tem a caixa do cartão e deixa
/// lugar à linha seguinte: nenhum outro alvo cai dentro da faixa do chip (o rótulo em cima desce o
/// controlo, e quem avançava `y + passo` escrevia a linha seguinte por cima dele).
/// *Mutação: `PropertyRow::fim` a devolver `y + passo` (control.y − rótulo).*
#[test]
fn every_list_row_in_a_card_leaves_room_for_the_next_row() {
    let chips = [
        (
            "audio",
            ids::INSP_LIVE_AUDIO_SECTION,
            pids::INSP_AUDIO_BUS_PICK,
        ),
        ("camera", CAMERA, pids::INSP_CAMERA_FORMAT_PICK),
        (
            "action",
            ids::INSP_LIVE_ACTION_SECTION,
            pids::INSP_ACTION_VERB_PICK,
        ),
        (
            "action_trigger",
            ids::INSP_LIVE_TRIGGER_SECTION,
            pids::INSP_TRIGGER_EDGE_PICK,
        ),
        (
            "counter_watch",
            ids::INSP_LIVE_WATCH_SECTION,
            pids::INSP_WATCH_CMP_PICK,
        ),
        ("sequence", ids::INSP_LIVE_SEQ_SECTION, pids::INSP_SEQ_PICK),
    ];
    for (nome, sec, id) in chips {
        super::o_inspector_armado::arma_tudo();
        for (porta, desarma) in super::o_inspector_armado::PORTAS {
            if !["name", "transform", nome].contains(porta) {
                desarma();
            }
        }
        let mut h = hero_com_aberto(sec);
        let r = rolado_ate(&mut h, id);
        assert!(
            (r.h - m::FIELD_H).abs() < 0.5,
            "{nome}: o chip não tem a altura da caixa do cartão: {r:?}"
        );
        let por_cima: Vec<_> = h
            .hit_index
            .iter_registrations()
            .filter(|(o, q)| {
                *o != id
                    && q.h <= m::FIELD_H + 0.5
                    && q.y > r.y + 0.5
                    && q.y < r.y + r.h - 0.5
                    && q.x < r.x + r.w
                    && q.x + q.w > r.x
            })
            .collect();
        assert!(
            por_cima.is_empty(),
            "{nome}: há alvos sobre o chip {r:?}: {por_cima:?}"
        );
    }
    desarma_tudo();
}

/// ⭐ 05 §5 e §9, um censo: uma secção do Inspector não pede cor ao tema por fora da tinta do cartão
/// (`resolve` lá é a [`card_ink::tinta`]), e `Checkbox::new` só sobrevive no ramo fora do cartão das
/// três portas. *Cresceu? Passe a linha nova pela porta (`rows::caixa_row`, `rows::seg_row_misto`).*
#[test]
fn a_section_paints_through_the_card_ink() {
    let pasta = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../ph2d-panel-inspector/src/sections"
    );
    let mut crus = Vec::new();
    let mut caixas = Vec::new();
    let mut lidos = 0;
    for f in std::fs::read_dir(pasta).expect("a pasta das secções") {
        let p = f.expect("um ficheiro").path();
        let Ok(fonte) = std::fs::read_to_string(&p) else {
            continue;
        };
        lidos += 1;
        let nome = p
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_owned();
        let cortado = fonte.split("#[cfg(test)]").next().unwrap_or(&fonte);
        // A moldura (T59) pede o degrau de subsecção do tema em que o cartão se pinta.
        let tinta_propria = usize::from(nome == "rows_moldura.rs");
        if cortado.matches("paint::resolve(").count() != tinta_propria {
            crus.push(nome.clone());
        }
        if cortado.matches("Checkbox::new(").count() > 0 {
            caixas.push(nome);
        }
    }
    assert!(lidos > 50, "a régua leu {lidos} ficheiros: o caminho mudou");
    assert!(
        crus.is_empty(),
        "cor pelo tema, por fora da tinta do cartão: {crus:?}"
    );
    caixas.sort();
    assert_eq!(
        caixas,
        ["anchor_mount_row.rs", "rows_par.rs", "script_controlo.rs"],
        "Checkbox::new fora das portas (rows::caixa_row)"
    );
}
