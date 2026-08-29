//! **A metade da SHELL da row de ficheiro** — a que abre o diálogo e a que sabe que
//! extensões este build lê.
//!
//! A metade do painel (`lib_file_tests` no `ph2d-panel-motion-params`) prova que o botão é
//! pintado, é registado, e que o clique chega ao barramento. Esta prova a outra ponta: que a
//! espécie declarada pelo nó **resolve** para um filtro real, e que o caminho que o artista
//! escolheu chega ao documento pela mesma chave que o cook lê.

// ⚠️ `super` AQUI é o próprio `params_file` (o módulo `#[path]` que declara este ficheiro),
// e `super::super` é o `params`. O mesmo salto de um nível a mais já custou uma compilação
// nesta linha.
use super::{file_filter, kind_of};
use crate::motion_state::MotionState;
use ph2d_node_registry::FileKind;

/// **Toda espécie declarada resolve para um filtro utilizável.**
///
/// ⚠️ O `match` exaustivo do `file_filter` já garante que existe um BRAÇO por espécie; o que
/// ele não garante é que o braço devolve alguma coisa. Um filtro vazio abre um diálogo que
/// **não mostra nenhum ficheiro** — e da cadeira isso lê-se como *"este programa não abre o
/// meu ficheiro"*, sem nada vermelho em lado nenhum.
///
/// ⚠️ **E o controlo é a CONTAGEM**: um laço sobre uma lista vazia passa, e *um zero de «não
/// medido» e um de «tudo certo» são o mesmo byte*.
#[test]
fn every_file_kind_the_registry_declares_has_a_filter() {
    assert!(!FileKind::ALL.is_empty(), "a lista de especies esta vazia");
    for kind in FileKind::ALL {
        let (label, exts) = file_filter(*kind);
        assert!(!label.is_empty(), "{kind:?} nao tem rotulo de filtro");
        assert!(!exts.is_empty(), "{kind:?} abriria um dialogo vazio");
        for e in exts {
            assert!(
                !e.is_empty() && !e.starts_with('.'),
                "{kind:?}: extensao {e:?} — o rfd quer `wav`, nunca `.wav`"
            );
        }
    }
}

/// **O filtro de áudio é a lista CANÓNICA do app, não uma cópia.**
///
/// ⚠️ É a única afirmação que impede o defeito que o §5 do `CLAUDE.md` regista pelo nome no
/// importador de imagens: *uma lista escrita à mão ao lado de um predicado é duas respostas à
/// mesma pergunta, e a que o artista vê é a que envelhece*. Aqui, «a que o artista vê» seria
/// este diálogo, e ela envelheceria no dia em que um codec entrasse.
#[test]
fn the_audio_filter_is_the_apps_own_import_list() {
    let (_, exts) = file_filter(FileKind::Audio);
    assert_eq!(exts, crate::audio::decode_any::AUDIO_IMPORT_EXTS);
}

/// **O `audio.bands` pede um ficheiro, e a shell sabe qual** — a costura completa entre o
/// que o nó declara e o que o diálogo vai oferecer.
///
/// ⚠️ Sobre o nó REAL do registry, nunca sobre uma fixture: o hint dele já foi um
/// `ParamWidget::Text` (o artista digitava o caminho à mão), e uma fixture continuaria verde
/// no dia em que alguém o revertesse.
#[test]
fn the_audio_node_asks_for_a_file_and_the_shell_knows_which() {
    let mut state = MotionState::new();
    let n = state.doc.graph.add_node("audio.bands");
    assert_eq!(
        kind_of(&state, n, ph2d_node_audio_bands::FILE_KEY),
        Some(FileKind::Audio),
        "o param `file` do audio.bands tem de ser um ficheiro de SOM"
    );
}

/// **E o controlo: um param que não é ficheiro não resolve para nenhum.**
///
/// Sem ele, um `kind_of` que devolvesse `Some(Audio)` para tudo passaria o gate acima — e
/// clicar num knob qualquer abriria um selector de música.
#[test]
fn a_param_that_is_not_a_file_resolves_to_nothing() {
    let mut state = MotionState::new();
    let n = state.doc.graph.add_node("audio.bands");
    assert_eq!(
        kind_of(&state, n, ph2d_node_audio_bands::param::COUNT),
        None,
        "`count` e' um numero, nao um caminho"
    );
    assert_eq!(kind_of(&state, n, "nao_existe"), None);
}

/// **A marca de *missing footage* diz a verdade sobre os TRÊS casos** — e são três, não dois.
///
/// | caminho | `missing` | o que o artista lê |
/// |---|---|---|
/// | vazio | `false` | *ainda não escolhi* |
/// | existe | `false` | *está bom* |
/// | não existe | **`true`** | *isto está partido, e é aqui que se corrige* |
///
/// ⚠️ Sem o primeiro caso, um nó acabado de criar nasceria com um aviso vermelho a dizer que
/// falta um ficheiro que ninguém pediu — que é ruído a treinar o artista a ignorar o aviso.
#[test]
fn the_missing_mark_tells_the_truth_about_all_three_cases() {
    let dir = std::env::temp_dir().join("ph2d_file_row_gate");
    std::fs::create_dir_all(&dir).ok();
    let real = dir.join("here.wav");
    std::fs::write(&real, b"not really a wav, but it EXISTS").expect("escreve");

    let mut state = MotionState::new();
    let n = state.doc.graph.add_node("audio.bands");
    ph2d_panel_motion_graph::set_graph_selection(vec![n.0]);

    let missing_of = |state: &MotionState| {
        let snap =
            super::super::build_params_snapshot(state, ph2d_editor::ProjectSettings::default())
                .expect("o no selecionado tem params");
        snap.rows
            .iter()
            .find_map(|r| match r {
                ph2d_panel_motion_params::ParamRow::File(f) => Some(f.missing),
                _ => None,
            })
            .expect("o audio.bands tem uma row de ficheiro")
    };

    assert!(!missing_of(&state), "sem caminho nao ha nada em falta");

    state.doc.graph.set_text_param(
        n,
        ph2d_node_audio_bands::FILE_KEY,
        real.to_string_lossy().as_ref(),
    );
    assert!(!missing_of(&state), "o ficheiro existe: {real:?}");

    state.doc.graph.set_text_param(
        n,
        ph2d_node_audio_bands::FILE_KEY,
        dir.join("gone.wav").to_string_lossy().as_ref(),
    );
    assert!(missing_of(&state), "o ficheiro nao existe — tem de acusar");

    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
}
