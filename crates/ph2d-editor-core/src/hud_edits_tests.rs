//! Os gates do vocabulário da secção HUD.

use super::{HUD_NUMBERS, HUD_TEXTS, HudNumber as N, HudText as T, InspectorHudInfo};

fn cheio() -> InspectorHudInfo {
    InspectorHudInfo {
        entity_bits: 7,
        has_canvas: true,
        // ⚠️ A raiz mostra as próprias linhas ⇒ nenhuma nota a apontar para outro sítio.
        canvas_parent: None,
        ref_w: 32.0,
        ref_h: 18.0,
        fit: 0,
        tem_camera: true,
        has_label: true,
        source: 1,
        source_name: "pontos".into(),
        prefix: "Pontos: ".into(),
        suffix: " !".into(),
        vivo: "Pontos: 9 !".into(),
        has_button: true,
        signal: "bonus".into(),
        disabled: false,
        has_counter: true,
        counter_name: "pontos".into(),
        counter_start: 0.0,
        counter_value: 9,
    }
}

/// A leitura é pela TABELA, e ela cobre todos os campos — uma linha nova entra num sítio só.
#[test]
fn cada_numero_e_cada_texto_tem_leitura() {
    let i = cheio();
    assert_eq!(HUD_NUMBERS.len(), 3);
    assert_eq!(HUD_TEXTS.len(), 5);
    assert!((i.number(N::RefWidth) - 32.0).abs() <= f32::EPSILON);
    assert!((i.number(N::RefHeight) - 18.0).abs() <= f32::EPSILON);
    assert_eq!(i.text(T::SourceName), "pontos");
    assert_eq!(i.text(T::Signal), "bonus");
    assert_eq!(i.text(T::CounterName), "pontos");
}

/// ⛔ **Um bloco ausente não pinta nada** — mostrar sempre os doze campos entregaria nove controlos
/// mortos, que é a espécie que o `CLAUDE.md` §5.0 nomeia.
#[test]
fn um_bloco_ausente_nao_pinta_um_unico_campo() {
    let vazio = InspectorHudInfo::default();
    for n in HUD_NUMBERS {
        assert!(!vazio.mostra_numero(n), "{n:?} não tem dono neste objecto");
    }
    for t in HUD_TEXTS {
        assert!(!vazio.mostra_texto(t), "{t:?} não tem dono neste objecto");
    }
    // O CONTROLO: com os três componentes, tudo aparece.
    let i = cheio();
    assert!(HUD_NUMBERS.into_iter().all(|n| i.mostra_numero(n)));
    assert!(HUD_TEXTS.into_iter().all(|t| i.mostra_texto(t)));
}

/// ⚠️ O nome da fonte some com a fonte AUTORADA — ali não há nome nenhum a dar.
#[test]
fn o_nome_da_fonte_some_quando_o_rotulo_nao_deriva_nada() {
    let mut i = cheio();
    i.source = 0;
    assert!(!i.mostra_texto(T::SourceName));
    assert!(
        i.mostra_texto(T::Prefix) && i.mostra_texto(T::Suffix),
        "o CONTROLO: os irmãos do mesmo bloco ficam"
    );
}

/// Só o dono de cada número o mostra — um canvas sem contador não pinta o `Start`.
#[test]
fn cada_numero_pertence_ao_bloco_dele() {
    let mut i = cheio();
    i.has_counter = false;
    assert!(!i.mostra_numero(N::CounterStart));
    assert!(i.mostra_numero(N::RefWidth), "o canvas continua lá");
    let mut i = cheio();
    i.has_canvas = false;
    assert!(!i.mostra_numero(N::RefWidth) && !i.mostra_numero(N::RefHeight));
    assert!(i.mostra_numero(N::CounterStart));
}
