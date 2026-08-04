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

/// O sítio que arma a instância deriva o degrau da CÂMARA, pela porta do paste.
#[test]
fn the_place_step_comes_from_the_camera_not_from_a_constant() {
    let s = src("render_loop/mod.rs");
    let Some(at) = s.find("if let Some((new_id, main)) = arm_instance_of") else {
        panic!("o sítio que arma a instância mudou de forma — reancore este gate");
    };
    // ⚠️ A janela acaba na PRÓPRIA chamada que ela julga, e não numa linha vizinha: a 1ª versão
    // ancorava no bloco do Detach logo abaixo, e a wave seguinte — que mexeu no Detach e não no
    // Place — derrubou este gate sobre código correto. Um gate ancorado no que ele NÃO julga é um
    // proxy que expira.
    let block = &s[at..];
    let end = block
        .find("crate::vec_component_edit::arm_instance(")
        .expect("o bloco do Place deixou de armar a instância");
    let block = &block[..end];
    assert!(
        block.contains("screen_offset_world") && block.contains("PASTE_OFFSET_PX"),
        "o Place deixou de derivar o degrau da câmara. Se ele voltou a um número próprio, a \
         cópia nasce a uma distância que muda com o zoom — e foi assim que ela nasceu a ~700 px \
         do mestre, longe demais para o artista sequer saber que existia.\n{block}"
    );
    assert!(
        block.contains("cascade_offset") && block.contains("instance_count"),
        "o Place deixou de contar as cópias que já existem. Sem a contagem o degrau é o mesmo em \
         todo clique, e três cliques põem três cópias NO MESMO SÍTIO.\n{block}"
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
    assert!(
        !src("vec_component_edit.rs").contains("const PLACE_OFFSET"),
        "o `PLACE_OFFSET` em unidades de mundo voltou — é ele que punha a cópia a ~700 px"
    );
}
