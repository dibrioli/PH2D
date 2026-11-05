//! Os testes da tabela da `lib.rs` — irmão por tecto de LOC.

use super::*;

#[test]
fn known_keys_round_trip_to_english() {
    // Image-tool labels were abbreviated 2026-05-25 to fit the
    // 44-px chip column; tooltips keep the long English form.
    assert_eq!(tr("tool.trim_transparency.label"), "TRIM");
    assert_eq!(
        tr("tool.trim_transparency.tooltip"),
        "Trim · cut away the transparent edges"
    );
    assert_eq!(tr("tool.make_square.label"), "SQUAR");
    assert_eq!(tr("edit.undo.label"), "Undo");
    // Timeline panel chrome (W2.E9).
    assert_eq!(tr("panel.timeline.title"), "Timeline");
    assert_eq!(tr("panel.timeline.loop"), "Loop");
    assert_eq!(tr("panel.timeline.prop.translate_x"), "Translate X");
    // Vector panel chrome — section headers + the shape-catalogue words.
    assert_eq!(tr("panel.vector.section.tool"), "Tool");
    assert_eq!(tr("panel.vector.mode.shape"), "Shape");
    assert_eq!(tr("panel.vector.category"), "Category");
    assert_eq!(tr("panel.vector.group.iso"), "3D");
}

#[test]
fn a_text_key_says_what_its_key_says() {
    const TITLE: TextKey = TextKey::new("panel.timeline.title");
    assert_eq!(TITLE.tr(), tr("panel.timeline.title"));
    assert_eq!(TITLE.tr(), "Timeline");
    assert_eq!(
        TextKey::new("tool.nonexistent.bar").tr(),
        "tool.nonexistent.bar"
    );
}

#[test]
fn unknown_key_returns_the_key_itself() {
    let leaked = tr("tool.nonexistent.foo");
    assert_eq!(leaked, "tool.nonexistent.foo");
}

/// A memória por fio responde pela chave INTEIRA: os prefixos e os prolongamentos de uma chave da tabela, lidos do
/// mesmo buffer reusado, nas duas ordens, dizem o que a tabela diz.
#[test]
fn a_memoria_por_fio_responde_o_que_a_tabela_diz() {
    // Montadas em execução: o censo de cada painel lê literais e tomaria a chave truncada por chave usada.
    let inteira = "panel.timeline.title";
    let chaves = [
        inteira.to_string(),
        inteira[..inteira.len() - 1].to_string(),
        format!("{inteira}.x"),
        "panel.timeline.loop".to_string(),
        "panel.timeline".to_string(),
        "tool.nonexistent.foo".to_string(),
        String::new(),
    ];
    std::thread::spawn(move || {
        let mut buffer = String::new();
        for k in chaves.iter().chain(chaves.iter().rev()) {
            buffer.clear();
            buffer.push_str(k);
            assert_eq!(tr_ingles(&buffer), tabela(k), "{k:?}");
        }
    })
    .join()
    .expect("as respostas casam");
}

/// O `tr` responde no Drop de um `thread_local` registado antes da memória (os destrutores correm na ordem inversa): a
/// memória já morreu, e um pânico ali aborta o processo.
#[test]
fn o_tr_responde_depois_de_a_memoria_do_fio_morrer() {
    struct NoFim;
    impl Drop for NoFim {
        fn drop(&mut self) {
            assert_eq!(tr("panel.timeline.title"), "Timeline");
        }
    }
    thread_local! {
        static FIM: NoFim = const { NoFim };
    }
    std::thread::spawn(|| {
        FIM.with(|_| {});
        assert_eq!(tr("edit.undo.label"), "Undo");
    })
    .join()
    .expect("o fio acabou");
}
