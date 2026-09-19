//! Os gates da escada de queixas do SEGUIDOR DE CAMINHO (suplente #23).
//!
//! ⚠️ **A ordem é o que está a ser afirmado**, e não a existência das variantes: uma escada que
//! responda *«não há relógio»* a quem ainda não escreveu o nome manda o artista resolver a metade
//! errada.

use super::*;

/// Um seguidor com tudo em ordem — a partida de cada gate abaixo.
fn pronto() -> InspectorPathFollowInfo {
    InspectorPathFollowInfo {
        caminho: "Trilho".into(),
        nome_existe: true,
        nome_tem_forma: true,
        relogios: 1,
        duracao_us: Some(1_000_000),
        ..InspectorPathFollowInfo::default()
    }
}

#[test]
fn com_tudo_em_ordem_nao_ha_queixa() {
    assert_eq!(pronto().queixa(), None);
}

/// ⭐⭐⭐ **A escada desce do ESPECÍFICO para o GERAL**, e o gate mede-a com TODOS os defeitos
/// presentes ao mesmo tempo: só assim a ordem é observável.
#[test]
fn a_escada_responde_do_especifico_para_o_geral() {
    let mut i = InspectorPathFollowInfo::default(); // sem nome, sem forma, sem relógio
    assert_eq!(i.queixa(), Some(PathFollowQueixa::SemNome));
    i.caminho = "Trilho".into();
    assert_eq!(i.queixa(), Some(PathFollowQueixa::NomeDesconhecido));
    i.nome_existe = true;
    assert_eq!(i.queixa(), Some(PathFollowQueixa::SemForma));
    i.nome_tem_forma = true;
    assert_eq!(i.queixa(), Some(PathFollowQueixa::SemRelogio));
    i.duracao_us = Some(0);
    assert_eq!(i.queixa(), Some(PathFollowQueixa::RelogioSemDuracao));
    i.duracao_us = Some(1);
    assert_eq!(i.queixa(), None);
}

/// ⚠️ **Espaços não são um nome** — um campo com um espaço lê-se vazio, senão a queixa seguinte
/// mandaria o artista procurar uma forma chamada `" "`.
#[test]
fn so_espacos_contam_como_vazio() {
    let i = InspectorPathFollowInfo {
        caminho: "   ".into(),
        ..pronto()
    };
    assert_eq!(i.queixa(), Some(PathFollowQueixa::SemNome));
}

/// ⛔ **O relógio PARADO não é queixa** — ele é o estado normal de uma cena em edição.
#[test]
fn o_relogio_parado_nao_e_queixa() {
    let i = InspectorPathFollowInfo {
        clock_playing: false,
        ..pronto()
    };
    assert_eq!(i.queixa(), None);
}
