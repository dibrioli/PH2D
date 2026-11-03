//! ⭐⭐ **Os cartões de casos reais** (doc `CartoesUI/08` §1.3): os gestos reais (ponteiro → despacho →
//! `apply_event` → pintura seguinte), pelo mesmo [`Galeria`] do irmão.

use super::cartoes::Galeria;
use bumpalo::Bump;
use ph2d_editor_core::interaction::dispatch_text_input;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::widget::card_field::painted_letter;
use ph2d_host::PointerKind;
use ph2d_panel_widget_gallery::cards_reais_mundo::estado_do_alvo;
use ph2d_panel_widget_gallery::cards_rive;
use ph2d_panel_widget_gallery::controlos::{controls, lit};
use ph2d_panel_widget_gallery::ids as g;
use ph2d_panel_widget_gallery::state::{Flag, Pick};

/// Decisão do dono (26/10): o «Use» troca as linhas — Repeated mostra Width/Height, Animated os
/// campos da folha. *Mutação: `Kind::Pick` só consumido; o `match` do `image` sempre no 0.*
#[test]
fn o_use_troca_as_linhas_da_imagem() {
    let mut gal = Galeria::aberta();
    assert!(
        gal.rect(g::IMG_FRAME).is_some(),
        "fixtura: Still mostra Frame"
    );
    assert!(gal.rect(g::IMG_W).is_none(), "fixtura: Still sem Width");
    gal.clica(g::USE_REPEATED);
    assert!(gal.rect(g::IMG_W).is_some(), "Repeated não mostrou Width");
    assert!(gal.rect(g::IMG_FRAME).is_none(), "Repeated manteve o Frame");
    gal.clica(g::USE_ANIMATED);
    assert!(
        gal.rect(g::IMG_FPS).is_some(),
        "Animated não mostrou Frames/s"
    );
    assert!(gal.rect(g::IMG_W).is_none(), "Animated manteve o Width");
}

/// Decisão do dono (02/11): o alvo escreve-se. Clicar no campo, teclar o nome: o estado diz se o
/// objeto existe. *Mutação: o `TextInput` do alvo fora da tabela `TEXTS`.*
#[test]
fn o_alvo_da_camera_escreve_se() {
    let mut gal = Galeria::aberta();
    let texto = |gal: &Galeria| {
        gal.host
            .store()
            .text(g::CAM_TARGET)
            .unwrap_or_default()
            .to_owned()
    };
    assert_eq!(texto(&gal), "Player A", "fixtura: o alvo de fábrica");
    gal.clica(g::CAM_TARGET);
    assert_eq!(
        gal.host.store().focus_id(),
        Some(g::CAM_TARGET),
        "o clique não deu foco ao alvo"
    );
    let arena = Bump::new();
    for ch in "x".chars() {
        let _ = dispatch_text_input(gal.host.store_mut(), ch, &arena);
    }
    gal.repinta();
    let alvo = texto(&gal);
    assert!(alvo.contains('x'), "a tecla não chegou ao alvo: {alvo}");
    assert!(
        estado_do_alvo(false, &alvo).contains('\u{201c}'),
        "um nome que não existe não avisou: {}",
        estado_do_alvo(false, &alvo)
    );
    assert!(
        estado_do_alvo(false, "Enemy").starts_with("Follows"),
        "um nome que existe não segue"
    );
}

/// O conta-gotas arma e o 2.º clique escreve o nome: nenhuma lista de objetos. *Mutação:
/// `Act::PickTarget` só consumido.*
#[test]
fn o_conta_gotas_da_camera_arma_e_escreve_o_nome() {
    let mut gal = Galeria::aberta();
    gal.clica(g::CAM_PICK);
    assert!(gal.st.choices.on(Flag::CamPicking), "não armou");
    gal.clica(g::CAM_PICK);
    assert!(!gal.st.choices.on(Flag::CamPicking), "não desarmou");
    assert_eq!(
        gal.host.store().text(g::CAM_TARGET),
        Some("Coin"),
        "não escreveu"
    );
}

/// Decisão do dono (25/10): UM cartão, com a lista de regras: «+» acrescenta, «−» tira a dela.
/// *Mutação: `Kind::Add` / `Kind::Remove` só consumidos.*
#[test]
fn as_regras_da_animacao_acrescentam_e_tiram() {
    let mut gal = Galeria::aberta();
    let (a, b, c) = (g::RULES[0], g::RULES[1], g::RULES[2]);
    assert!(gal.rect(b.when).is_some(), "fixtura: duas regras");
    assert!(gal.rect(c.when).is_none(), "fixtura: a terceira livre");
    gal.clica(g::RULE_ADD);
    assert!(gal.rect(c.when).is_some(), "o «+» não acrescentou a regra");
    assert!(
        gal.rect(g::RULE_ADD).is_none(),
        "a lista cheia ainda tem «+»"
    );
    gal.clica(b.remove);
    assert!(gal.rect(b.when).is_none(), "o «−» não tirou a regra dele");
    assert!(gal.rect(a.when).is_some(), "o «−» tirou a regra alheia");
    gal.clica(a.when);
    gal.clica(a.whens[3]);
    assert_eq!(
        gal.st.choices.pick(Pick::RuleWhen0),
        3,
        "o «quando» não mudou"
    );
    gal.clica(g::ANIM_ME);
    assert!(gal.rect(a.when).is_none(), "«Me» manteve as regras");
    assert!(gal.rect(g::RULE_ADD).is_none(), "«Me» manteve o «+»");
}

/// Decisão do dono (01/11): Paint type é seletor Water · Oil, e o que cada um pede aparece.
/// *Mutação: `BR_OIL` fora do `Kind::Pick`.*
#[test]
fn o_paint_type_escolhe_e_troca_a_linha_propria() {
    let mut gal = Galeria::aberta();
    assert!(
        gal.rect(g::BR_REWET).is_some(),
        "fixtura: Water mostra Re-wet"
    );
    gal.clica(g::BR_OIL);
    assert_eq!(gal.st.choices.pick(Pick::PaintType), 1, "Oil não escolheu");
    assert!(gal.rect(g::BR_BODY).is_some(), "Oil não mostrou Body");
    assert!(gal.rect(g::BR_REWET).is_none(), "Oil manteve o Re-wet");
    assert!(gal.rect(g::BR_DILUTION).is_some(), "Dilution é dos dois");
}

/// O rack do Audio segue o Effects (F43): «+» com tipos, lista, propriedades ao lado.
#[test]
fn o_rack_do_audio_segue_o_effects() {
    let mut gal = Galeria::aberta();
    let (a, b) = (g::RACK[0], g::RACK[1]);
    assert!(gal.rect(b.open).is_none(), "fixtura: uma linha");
    gal.clica(g::AU_ADD);
    gal.clica(g::AU_ADD_2);
    assert!(gal.rect(b.open).is_some(), "o «+» não acrescentou");
    assert_eq!(
        gal.st.choices.pick(Pick::Rack1),
        2,
        "o tipo não veio da opção"
    );
    assert!(gal.rect(a.mix).is_none(), "fixtura: fechado");
    gal.clica(a.open);
    let mix = gal.rect(a.mix).expect("a linha não abriu as propriedades");
    let linha = gal.rect(a.open).expect("a linha sumiu");
    assert!(
        mix.x + mix.w <= linha.x || mix.x >= linha.x + linha.w,
        "as propriedades {mix:?} abriram por cima da coluna {linha:?}"
    );
}

/// A cor da paleta vai para o alvo; o alvo escolhe-se clicando na caixa; a troca inverte.
/// *Mutação: `Kind::Palette` só consumido.*
#[test]
fn a_paleta_pinta_o_alvo_e_a_troca_inverte() {
    let mut gal = Galeria::aberta();
    gal.clica(g::PALETTE[3]);
    assert_eq!(
        gal.st.choices.pick(Pick::PixelFg),
        3,
        "o Foreground não mudou"
    );
    gal.clica(g::PX_BG);
    gal.clica(g::PALETTE[5]);
    assert_eq!(
        gal.st.choices.pick(Pick::PixelBg),
        5,
        "o Background não mudou"
    );
    assert_eq!(
        gal.st.choices.pick(Pick::PixelFg),
        3,
        "o Foreground mudou junto"
    );
    gal.clica(g::PX_SWAP);
    assert_eq!(
        gal.st.choices.pick(Pick::PixelFg),
        5,
        "a troca não inverteu"
    );
}

/// Sem Overheal, o Start não passa do Max. *Mutação: `sync_health` desligado.*
#[test]
fn sem_overheal_o_start_nao_passa_do_max() {
    let mut gal = Galeria::aberta();
    gal.host.store_mut().set_number_value(g::HP_MAX, 40.0);
    gal.repinta();
    assert_eq!(gal.valor(g::HP_START), 40.0, "o Start passou do Max");
}

/// Rive: o Trigger dispara e pisca; «+» traz o tipo pedido. *Mutação: `Act::Fire` só consumido.*
#[test]
fn o_trigger_dispara_e_pisca() {
    let mut gal = Galeria::aberta();
    let t = g::INPUTS[2];
    assert_eq!(gal.st.fires[2], 0, "fixtura: nunca disparou");
    gal.clica(t.fire);
    assert_eq!(gal.st.fires[2], 1, "o Fire não disparou");
    // A pintura assentar leva mais que o piscar: o relógio do teste é o carimbo, nunca a parede.
    assert!(gal.st.fired_at[2].is_some(), "o Fire não carimbou a hora");
    assert!(gal.st.fired_at[1].is_none(), "carimbou a entrada alheia");
    gal.st.fired_at[2] = Some(std::time::Instant::now());
    assert!(cards_rive::a_piscar(&gal.st, 2), "o Fire não acende");
    gal.st.fired_at[2] = std::time::Instant::now().checked_sub(std::time::Duration::from_secs(5));
    assert!(!cards_rive::a_piscar(&gal.st, 2), "o Fire não apaga");
    gal.clica(t.remove);
    assert!(gal.rect(t.fire).is_none(), "o «−» não tirou a entrada");
    gal.clica(g::SM_ADD);
    gal.clica(g::SM_ADD_BOOLEAN);
    assert!(gal.rect(t.check).is_some(), "o «+» não trouxe o Boolean");
    gal.clica(t.check);
    assert!(gal.st.choices.on(Flag::InputBool2), "a caixa não marcou");
}

/// Rive: no Solo só um filho vive; clicar outro apaga o primeiro. *Mutação: `Kind::Pick` só consumido.*
#[test]
fn o_solo_e_exclusivo() {
    let mut gal = Galeria::aberta();
    assert_eq!(gal.st.choices.pick(Pick::Solo), 0, "fixtura: o primeiro");
    gal.clica(g::SOLO[2]);
    assert_eq!(
        gal.st.choices.pick(Pick::Solo),
        2,
        "o clique não mudou o ativo"
    );
    let vivos = g::SOLO
        .iter()
        .filter(|id| {
            let c = controls().find(|c| c.id == **id).expect("na tabela");
            lit(c, &gal.st.choices, gal.host.store())
        })
        .count();
    assert_eq!(vivos, 1, "o Solo não é exclusivo");
}

/// Rive: o campo do Cubic muda o que a pré-visualização pinta; a pega também. *Mutação: o quadro a
/// publicar os valores de fábrica; `sync_pegas` desligado.*
#[test]
fn o_campo_e_a_pega_do_cubic_mudam_a_curva_pintada() {
    let mut gal = Galeria::aberta();
    let (tipo, antes, _) = cards_rive::curva_publicada().expect("o quadro não publicou");
    assert_eq!(tipo, 1, "fixtura: Cubic");
    gal.arrasta_o_icone(g::CU_X1, 40.0);
    let (_, depois, _) = cards_rive::curva_publicada().expect("o quadro não publicou");
    assert_ne!(antes[0], depois[0], "o X1 não mudou a curva pintada");
    assert_eq!(
        depois[0] as f64,
        gal.valor(g::CU_X1) as f32 as f64,
        "a curva não é o campo"
    );
    let pega = gal.rect(g::CU_H2).expect("a pega não foi pintada");
    let (x, y) = (pega.x + pega.w * 0.5, pega.y + pega.h * 0.5);
    gal.ponteiro(PointerKind::Down, x, y);
    gal.ponteiro(PointerKind::Move, x - 30.0, y + 12.0);
    gal.ponteiro(PointerKind::Up, x - 30.0, y + 12.0);
    gal.repinta();
    let (_, arrastada, _) = cards_rive::curva_publicada().expect("o quadro não publicou");
    assert!(arrastada[2] < depois[2], "a pega não moveu o X2");
    assert!(arrastada[3] < depois[3], "a pega não moveu o Y2");
    gal.clica(g::IN_HOLD);
    assert!(
        gal.rect(g::CU_X1).is_none(),
        "o Hold manteve os campos do Cubic"
    );
    assert_eq!(
        cards_rive::curva_publicada().expect("publica").0,
        2,
        "o tipo não mudou"
    );
}

/// Rive: o gradiente: a pega escolhe a paragem, o «+» acrescenta, com duas o «−» some.
#[test]
fn a_paragem_do_gradiente_escolhe_e_o_mais_acrescenta() {
    let mut gal = Galeria::aberta();
    let (a, b, c) = (g::STOPS[0], g::STOPS[1], g::STOPS[2]);
    assert!(
        gal.rect(a.position).is_some(),
        "fixtura: a primeira escolhida"
    );
    gal.clica(b.handle);
    assert!(
        gal.rect(b.position).is_some(),
        "a pega não escolheu a paragem"
    );
    assert!(gal.rect(a.position).is_none(), "a escolha não é exclusiva");
    assert!(gal.rect(b.remove).is_none(), "com duas paragens não há «−»");
    gal.clica(g::GR_ADD);
    assert!(
        gal.rect(c.handle).is_some(),
        "o «+» não acrescentou a paragem"
    );
    assert!(gal.rect(b.remove).is_some(), "com três falta o «−»");
    gal.clica(g::GR_SOLID);
    assert!(gal.rect(a.handle).is_none(), "Solid manteve a barra");
}

fn fabrica(id: ph2d_a11y::NodeId) -> f64 {
    g::FIELDS
        .iter()
        .find(|f| f.0 == id)
        .map(|f| f.1)
        .expect("campo da tabela")
}

/// Os números de partida são os do código real, nunca inventados: quem copia o cartão copia a verdade.
/// *Mutação: `PL_SPEED` a 8 ⇒ vermelho.*
#[test]
fn os_valores_de_partida_sao_os_de_fabrica_do_codigo_real() {
    let p = ph2d_platformer::PlayerConfig::STARTING_POINT;
    let h = ph2d_health::Config::default();
    for (id, real, nome) in [
        (g::PL_SPEED, f64::from(p.walk.speed), "Speed"),
        (g::PL_ACCEL, f64::from(p.walk.acceleration), "Acceleration"),
        (g::PL_AIR, f64::from(p.walk.air_acceleration), "Air"),
        (g::PL_JUMP, f64::from(p.jump.jump_height), "Jump Height"),
        (g::PL_AIR_JUMPS, f64::from(p.jump.air_jumps), "Air Jumps"),
        (g::PL_COYOTE, f64::from(p.jump.coyote_time), "Coyote"),
        (g::PL_BUFFER, f64::from(p.jump.jump_buffer), "Buffer"),
        (g::HP_MAX, h.maximo, "Max"),
        (g::HP_START, h.maximo, "Start"),
        (g::HP_REGEN, h.regen, "Regen"),
        (g::HP_DELAY, h.regen_atraso_s, "Regen Delay"),
    ] {
        assert!(
            (fabrica(id) - real).abs() < 1e-6,
            "{nome}: a galeria parte de {}, o código real de {real}",
            fabrica(id)
        );
    }
}

/// ⭐ T56 no cartão: a pega mostra a LETRA da grandeza — os pares leem-se X/Y, W/H, M/S. A prova é a
/// letra que a pintura registou, não a tabela. *Mutação: `letra` a devolver a 1.ª palavra ⇒ vermelho.*
#[test]
fn cada_campo_mostra_a_letra_da_sua_grandeza() {
    let mut gal = Galeria::aberta();
    let _ = gal.rect(g::COL_OX);
    for (id, letra) in [
        (g::COL_W, "W"),
        (g::COL_H, "H"),
        (g::COL_OX, "X"),
        (g::COL_OY, "Y"),
        (g::COL_FRICTION, "F"),
        (g::COL_BOUNCE, "B"),
        (g::HP_MAX, "M"),
        (g::HP_START, "S"),
        (g::HP_REGEN, "R"),
        (g::HP_DELAY, "D"),
        (g::PL_SPEED, "S"),
        (g::PL_JUMP, "H"),
        (g::PL_COYOTE, "T"),
        (g::PL_BUFFER, "B"),
        (g::TR_START, "S"),
        (g::TR_END, "E"),
        (g::SC_SPEED, "S"),
        (g::SC_JUMP, "P"),
    ] {
        assert_eq!(painted_letter(id).as_deref(), Some(letra), "campo {id:?}");
    }
    gal.clica(g::COL_CIRCLE);
    assert!(
        gal.rect(g::COL_RADIUS).is_some(),
        "Circle não mostrou o raio"
    );
    assert_eq!(
        painted_letter(g::COL_RADIUS),
        None,
        "o raio tem ícone que o quer dizer"
    );
}

/// Censo: nas pegas de número só entram os ícones que QUEREM DIZER a grandeza. Um ícone emprestado
/// é um desenho sem significado ao lado de um número. *Mutação: `Glyph::Icon(IconId::CardFlow)`.*
#[test]
fn nenhum_campo_usa_um_icone_emprestado() {
    const QUEREM_DIZER: &[&str] = &[
        "CardAngle",   // ângulo
        "CardOpacity", // opacidade
        "CardWeight",  // peso do traço
        "CardRadius",  // raio
        "CardMix",     // mistura
        "CardSoft",    // desfoque
        "CardChain",   // corrente de ossos
        "FxWave",      // a linha do rack: o tipo, não um número
    ];
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut vistos = 0;
    for ficheiro in std::fs::read_dir(src).expect("src") {
        let p = ficheiro.expect("entrada").path();
        if !p
            .file_name()
            .is_some_and(|n| n.to_string_lossy().starts_with("cards"))
        {
            continue;
        }
        let t = std::fs::read_to_string(&p).expect("fonte");
        let mut sitios: Vec<&str> = t.split("Glyph::Icon(").skip(1).collect();
        // `num!(id, IconId::X, rótulo)`: o ícone vem antes do rótulo.
        sitios.extend(t.split("num!(").skip(1).filter_map(|r| {
            let antes = r.split("tr(").next().unwrap_or_default();
            antes.split_once("IconId::").map(|(_, d)| d)
        }));
        for s in sitios {
            let s = s.trim_start();
            let s = s.strip_prefix("IconId::").unwrap_or(s);
            let nome: String = s.chars().take_while(|c| c.is_alphanumeric()).collect();
            // `Glyph::Icon(if …)`, `Glyph::Icon(fx_icon(kind))`: a expressão fica fora do censo.
            if !nome.starts_with(char::is_uppercase) {
                continue;
            }
            vistos += 1;
            assert!(
                QUEREM_DIZER.contains(&nome.as_str()),
                "{}: `{nome}` numa pega de número não diz a grandeza; a cura é `num!(id, rótulo)` \
                 (a letra, T56)",
                p.display()
            );
        }
    }
    assert!(vistos >= 8, "o censo não leu o sítio certo ({vistos})");
}
