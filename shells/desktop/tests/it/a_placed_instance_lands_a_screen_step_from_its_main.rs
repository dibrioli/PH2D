//! **Arch-gate: a cópia nasce a um degrau de TELA do mestre, e a folga é a do paste.**
//!
//! O relato do Enio no smoke da cena `=53` foram duas frases sobre o mesmo botão: *"as cópias
//! foram criadas a uma distância muito grande e nem saberia que existiam"* e *"as cópias foram
//! criadas uma em cima da outra"*.
//!
//! A causa da primeira estava escrita no código, com o doc-comment a afirmar o contrário: o
//! `Place` tinha um `PLACE_OFFSET: f32 = 24.0` em unidades de **MUNDO** e o comentário dizia ser
//! *"a mesma folga do Duplicate do Arrange"* — que são **12 px de TELA**, convertidos pelo zoom.
//! Medido na própria cena (~29 px por unidade), a cópia nascia a **~700 px do mestre**: 58× um
//! paste, e sete larguras do botão que ela copiava.
//!
//! # Porque é um arch-gate, e não um gate de unidade
//!
//! O que os gates de `vec_component_edit::tests` provam é a **cascata** — que a n-ésima cópia está
//! a n degraus —, e eles recebem o degrau como argumento. Nenhum deles pode ver de ONDE o produto
//! tira esse degrau, e é aí que o defeito viveu: uma constante de mundo passa por toda asserção
//! sobre múltiplos sem piscar. Só a fiação responde *"o degrau é de tela?"*, e ela precisa de
//! janela e câmara — nenhum teste de unidade a alcança.
//! [[feedback_geometry_over_mixed_units_needs_the_consumers_conversion]]

use std::fs;

fn src(name: &str) -> String {
    fs::read_to_string(format!("{}/src/{name}", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|e| panic!("{name}: {e}"))
}

/// O sítio que despacha o verbo deriva o degrau da CÂMARA, pela porta do paste.
///
/// ⛔⛔ **RE-ANCORADO em 2026-09-07 (F4.6c), e a razão é a que este ficheiro já escrevia.** A
/// âncora era `arm_instance_of` — a variável do motor `VecInstance`, que saiu com ele —, e o gate
/// reprovou sobre uma lei que continua **viva e correcta**. *Um gate ancorado no que ele NÃO julga
/// é um proxy que expira*, e desta vez o que expirou foi o motor inteiro por baixo da âncora.
///
/// ⚠️ **A lei não mudou de conteúdo, mudou de dono:** quem conta as cópias já existentes deixou de
/// ser o `instance_count` do vetor e passou a ser o `cascade` do dreno geral (`instance_verbs`),
/// que lê `instances_of(master)`. O que a shell continua a dever é a **origem do degrau** — e é só
/// isso que um censo de fonte pode julgar aqui.
///
/// ⚠️ **A metade da CONTAGEM saiu deste ficheiro de propósito**: ela deixou de ser observável no
/// fonte da shell (vive dentro do dreno) e passou a ter régua de COMPORTAMENTO, em
/// `instance_verbs_tests::no_two_copies_of_a_recipe_ever_land_on_each_other`. *Um censo textual
/// que persegue uma lei para dentro de outra crate mede o nome dela, não o efeito.*
///
/// ⚠️ O sítio que despacha é o QUADRO pela ordem em que corre (`frame_text::render_frame`): desde a OBRA 2 da
/// `line/render-loop` (2026-09-13) o bloco do verbo de prefab mora numa fase, e a janela abre-se onde ele está.
#[test]
fn the_place_step_comes_from_the_camera_not_from_a_constant() {
    let s = crate::frame_text::render_frame();
    let Some(at) = s.find("let subject = crate::vec_component_general::subject_of(") else {
        panic!("o sítio que despacha o verbo de prefab mudou de forma — reancore este gate");
    };
    // ⚠️ A janela acaba na PRÓPRIA chamada que ela julga, e não numa linha vizinha: a 1ª versão
    // ancorava no bloco do Detach logo abaixo, e a wave seguinte — que mexeu no Detach e não no
    // Place — derrubou este gate sobre código correto.
    let block = &s[at..];
    let end = block
        .find("crate::vec_component_general::dispatch(")
        .expect("o bloco do verbo deixou de chamar o dreno do modo");
    let block = &block[..end];
    assert!(
        block.contains("screen_offset_world") && block.contains("PASTE_OFFSET_PX"),
        "o Place deixou de derivar o degrau da câmara. Se ele voltou a um número próprio, a \
         cópia nasce a uma distância que muda com o zoom — e foi assim que ela nasceu a ~700 px \
         do mestre, longe demais para o artista sequer saber que existia.\n{block}"
    );
}

/// A folga do paste é uma só, e o Place não tem a sua.
///
/// ⚠️ A metade que o gate acima não cobre: alguém pode chamar `screen_offset_world` **com um
/// número próprio** em vez do `PASTE_OFFSET_PX`, e a unidade ficaria certa com a folga a
/// discordar do Ctrl+D — duas respostas a *"onde nasce uma cópia?"*, que é o que este ficheiro
/// existe para impedir.
#[test]
fn there_is_exactly_one_paste_gap_in_the_shell() {
    let s = src("input_dispatch.rs");
    assert_eq!(
        s.matches("const PASTE_OFFSET_PX").count(),
        1,
        "nasceu uma segunda constante de folga de paste"
    );
    // ⚠️ A âncora é a DECLARAÇÃO, não o nome nu: o doc-comment do `cascade_offset` cita
    // `PLACE_OFFSET` para contar como a constante morreu, e um gate ancorado no token nu falha
    // sobre a própria prosa que o justifica (foi o que ele fez na 1ª corrida).
    // ⚠️ **A varredura passou a ser a SHELL inteira** (F4.6c): a constante vivia no
    // `vec_component_edit.rs`, que encolheu para a tabela `id → verbo`, e um censo apontado a um
    // ficheiro que já não é o dono da lei mede o sítio errado. *Quando o dono de uma lei muda de
    // casa, o censo que a guarda muda de janela — ou fica verde sobre o quarto vazio.*
    let mut culpados = Vec::new();
    let raiz = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut pilha = vec![raiz.clone()];
    while let Some(dir) = pilha.pop() {
        for e in fs::read_dir(&dir).expect("ler o src") {
            let c = e.expect("entrada").path();
            if c.is_dir() {
                pilha.push(c);
            } else if c.extension().is_some_and(|x| x == "rs")
                && fs::read_to_string(&c)
                    .expect("ler")
                    .contains("const PLACE_OFFSET")
            {
                culpados.push(c.strip_prefix(&raiz).unwrap_or(&c).to_path_buf());
            }
        }
    }
    assert!(
        culpados.is_empty(),
        "o `PLACE_OFFSET` em unidades de mundo voltou ({culpados:?}) — é ele que punha a cópia a \
         ~700 px do mestre"
    );
}
