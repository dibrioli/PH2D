//! Os gates do vocabulário do abanão. ⚠️ Eles medem as **perguntas derivadas** — as que o painel
//! faz e que, se respondessem errado, pintariam um aviso sobre um produto correcto (ou calariam um
//! sobre um partido).

use super::*;

fn fonte(on: &str, dentro: f32, fora: f32) -> InspectorEmitterRow {
    InspectorEmitterRow {
        on: on.to_owned(),
        de: 0,
        forca: 0.6,
        dentro,
        fora,
    }
}

/// ⚠️ **`fora <= dentro` é LEGAL e quase nunca é intencional** — ver o doc do predicado.
#[test]
fn os_raios_trocados_sao_reconhecidos_e_os_certos_nao() {
    assert!(fonte("boom", 5.0, 2.0).raios_trocados(), "invertidos");
    assert!(
        fonte("boom", 5.0, 5.0).raios_trocados(),
        "iguais: corte duro"
    );
    assert!(
        !fonte("boom", 2.0, 5.0).raios_trocados(),
        "a ordem normal NÃO pode acusar"
    );
}

/// ⚠️ **«calada» e «raios trocados» são DOIS factos** — contá-los juntos mandaria o artista
/// arranjar a metade errada, que é a lei que a lista de gatilhos já escreve.
#[test]
fn as_duas_contagens_nao_se_confundem() {
    let info = InspectorEmitterInfo {
        entity_bits: 1,
        rows: vec![
            fonte("", 2.0, 5.0),     // calada, raios bons
            fonte("boom", 5.0, 2.0), // fala, raios trocados
            fonte("", 9.0, 1.0),     // as duas
            fonte("hit", 1.0, 8.0),  // nenhuma
        ],
        ha_camera_que_treme: true,
        clock_playing: true,
        selected_count: 1,
    };
    assert_eq!(info.caladas(), 2);
    assert_eq!(info.trocadas(), 2);
    // ⭐ E o CONTROLO: uma lista sã não acusa nada.
    let sa = InspectorEmitterInfo {
        rows: vec![fonte("boom", 2.0, 5.0)],
        ..info
    };
    assert_eq!((sa.caladas(), sa.trocadas()), (0, 0));
}

/// Um espaço em branco é **vazio** — o `trim` é a lei, e sem ele um nome de um espaço leria-se como
/// configurado e nunca casaria sinal nenhum.
#[test]
fn um_nome_de_espacos_e_calado() {
    let info = InspectorEmitterInfo {
        entity_bits: 1,
        rows: vec![fonte("   ", 2.0, 5.0)],
        ha_camera_que_treme: true,
        clock_playing: true,
        selected_count: 1,
    };
    assert_eq!(info.caladas(), 1);
}
