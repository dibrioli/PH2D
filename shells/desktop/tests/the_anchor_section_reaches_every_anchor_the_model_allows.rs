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
