use super::sheet_authorship;
use ph2d_editor_core::InspectorSpriteSource as S;

/// ⚠️ **O caso que o Enio relatou.** A peça está na folha e ainda não foi assada: o
/// armazenamento é mesmo `Individual`, mas a AUTORIA já é da folha — e é a autoria que a linha
/// `Strategy` responde (a nota que já estava no `snapshots.rs` di-lo desde sempre; o código é
/// que não a seguia).
#[test]
fn a_piece_dropped_into_a_sheet_reads_as_hand_packed() {
    let (kind, label) = sheet_authorship(S::Individual { texture_id: 7 }, Some("Fruits"), None);
    assert!(matches!(kind, S::HandPacked { .. }));
    assert_eq!(label.as_deref(), Some("Fruits \u{00b7} not baked yet"));
}

/// O mesmo para uma peça que ainda vive no atlas — pô-la na folha é o mesmo gesto.
#[test]
fn an_atlas_piece_in_a_sheet_reads_the_same() {
    let (kind, _) = sheet_authorship(S::Atlas { key: 3 }, Some("Fruits"), None);
    assert!(matches!(kind, S::HandPacked { .. }));
}

/// **Assado ganha do arranjado**, e não o contrário: quando há região nomeada, é ela que o
/// artista quer ler (é por ela que ele reencontra o desenho no Aseprite).
#[test]
fn a_baked_piece_keeps_its_region_name() {
    let (kind, label) = sheet_authorship(
        S::HandPacked {
            sheet: 4,
            region: 2,
        },
        None,
        Some("hero \u{00b7} idle_0".into()),
    );
    assert!(matches!(
        kind,
        S::HandPacked {
            sheet: 4,
            region: 2
        }
    ));
    assert_eq!(label.as_deref(), Some("hero \u{00b7} idle_0"));
}

/// **Controle positivo:** fora de uma folha nada muda. Sem isto, a regra podia estar a
/// devolver `HandPacked` para toda a gente e os testes acima passariam na mesma.
#[test]
fn a_sprite_outside_any_sheet_is_untouched() {
    let (kind, label) = sheet_authorship(S::Individual { texture_id: 7 }, None, None);
    assert!(matches!(kind, S::Individual { texture_id: 7 }));
    assert!(label.is_none());
    let (kind, _) = sheet_authorship(S::Atlas { key: 1 }, None, None);
    assert!(matches!(kind, S::Atlas { key: 1 }));
}
