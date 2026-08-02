//! **UM CTRL+Z REPINTA SÓ O QUE ELE MUDOU** — os gates do confinamento (doc 28 §5.63).
//!
//! # O que pode dar errado, e qual gate pega
//!
//! O confinamento tem **dois** modos de falha e eles não têm o mesmo preço:
//!
//! * **reivindicar de menos** (`None` onde cabia um retângulo) — custa um repaint. Pego pelo gate de
//!   COMPORTAMENTO, que exige a pista parcial.
//! * **reivindicar de mais** (um retângulo que não cobre tudo o que mudou) — deixa na tela a figura
//!   ANTERIOR fora dele. **Nenhum gate de conteúdo pega isso**, porque dentro do retângulo está tudo
//!   certo. Pego pelo **ORÁCULO**, que compara a tela contra a rota de repaint inteiro.
//!
//! É por isso que o oráculo é o gate central desta wave, e por isso a ablação
//! ([`crate::undo::UndoController::confine`]) existe: sem uma segunda rota não há contra quem comparar.

use super::measure_stroke_owners::{armed, stroke};
use super::*;

/// Pinta `n` traços e devolve o tool.
fn painted(side: u32, n: u8, confine: bool) -> PainterTool {
    let mut t = armed(side);
    t.undo.confine = confine;
    for k in 0..n {
        stroke(&mut t, 200.0 + f32::from(k) * 40.0);
    }
    let _ = t.take_preview_arc();
    t
}

/// **O gate de COMPORTAMENTO: desfazer um traço não repinta a tela inteira.**
///
/// ⚠️ O oráculo é a pista que a drenagem tomou **mais** o retângulo publicado, e as duas metades são
/// necessárias: a pista sozinha ficaria verde com um retângulo do tamanho do canvas (que é repaint
/// inteiro vestindo o nome de parcial), e o retângulo sozinho ficaria verde com a drenagem o ignorando
/// — que é exatamente o estado em que esta wave passou uma hora (o confinamento disparava, o
/// `restore_selection` derrubava o cache, e o quadro seguia em 381 ms).
#[test]
fn an_undone_stroke_repaints_only_the_window_it_touched() {
    let mut t = painted(1024, 2, true);
    // ⚠️ O diagnóstico é capturado ANTES do passo, porque depois dele a entrada já saiu da pilha — e
    // ele entra nas mensagens de falha em vez de viver como helper sem chamador. Um `None` mudo manda a
    // próxima pessoa reconstruir o instrumento que esta wave já pagou (doc 28 §5.63).
    let why = t.undo.peek_confine_diagnosis(false).unwrap_or_default();
    assert!(t.undo_last());
    let rect = t
        .dirty_rect_now()
        .unwrap_or_else(|| panic!("o undo tem de publicar a janela que reescreveu — {why}"));
    assert!(
        t.composited_is_some(),
        "o cache de composite tem de SOBREVIVER a um undo confinado (quem o derruba manda a tela \
         inteira ser refeita, e o retangulo publicado vira decoracao) — {why}"
    );
    t.paint_tick(1.0 / 60.0);
    let _ = t.take_preview_arc();
    assert_eq!(
        t.last_drain_branch,
        crate::tool::DrainBranch::PartialComposite,
        "a drenagem tem de tomar a pista PARCIAL depois de um undo confinado — {why}"
    );
    // ⚠️ A conta é em PIXELS dos dois lados. A 1ª versão deste gate multiplicava a janela por 4
    // (elementos de um plano RGBA) e a comparava com a tela em pixels — reprovando um retângulo de
    // 72.922 px contra uma tela de 262.144. *Um gate que mistura unidades falha pelo motivo errado.*
    let (w, h) = t.source_size;
    let (win, screen) = (
        u64::from(rect.w) * u64::from(rect.h),
        u64::from(w) * u64::from(h),
    );
    assert!(
        win * 4 < screen,
        "a janela ({}x{} = {win} px) tem de ser uma FRAÇÃO da tela ({w}x{h} = {screen} px) — um \
         retângulo do tamanho do canvas é repaint inteiro vestindo o nome de parcial",
        rect.w,
        rect.h
    );
}

/// **O ORÁCULO: a pista parcial mostra EXATAMENTE o que o repaint inteiro mostra.**
///
/// Duas cópias do mesmo tool, dirigidas pelo mesmo roteiro, uma com o confinamento ablacionado. Os
/// bytes que a tela recebe têm de ser **idênticos** — no undo E no redo, porque as duas direções
/// escrevem regiões diferentes e uma delas pode reivindicar de mais sozinha.
///
/// ⚠️ **Este é o único gate que pode ver "reivindicou de mais"**, e ele não sabe nada sobre janelas: ele
/// compara duas TELAS. Um gate que afirmasse propriedades do retângulo estaria espelhando a regra que
/// julga.
#[test]
fn a_confined_undo_shows_exactly_what_a_full_repaint_shows() {
    let (mut fast, mut slow) = (painted(256, 3, true), painted(256, 3, false));
    let screen = |t: &mut PainterTool| {
        t.paint_tick(1.0 / 60.0);
        t.take_preview_arc().map(|(px, _, _)| px)
    };
    for step in 0..3u8 {
        assert!(
            fast.undo_last() && slow.undo_last(),
            "a fila tem de ter passos"
        );
        let (a, b) = (screen(&mut fast), screen(&mut slow));
        assert_eq!(
            a, b,
            "undo {step}: a pista parcial divergiu do repaint inteiro"
        );
    }
    for step in 0..3u8 {
        assert!(
            fast.redo_last() && slow.redo_last(),
            "a fila de redo tem de ter passos"
        );
        let (a, b) = (screen(&mut fast), screen(&mut slow));
        assert_eq!(
            a, b,
            "redo {step}: a pista parcial divergiu do repaint inteiro"
        );
    }
    // ⚠️ Controle: sem isto o gate ficaria verde comparando duas telas que a ablação nunca separou.
    assert!(
        fast.undo.confine && !slow.undo.confine,
        "a ablacao nao esta armada — os dois bracos rodam o MESMO caminho e o gate e' verde por vacuo"
    );
}

/// **Uma edição ESTRUTURAL não é confinada** — a cerca de Chesterton do `invalidate_composite`, que
/// esta wave estreita em vez de derrubar. Mudar a opacidade de uma camada muda a figura em TODA parte.
///
/// ⚠️ **O passo tem de tocar pixels TAMBÉM, e a 1ª versão deste gate não tocava.** Sem escrita de
/// canvas os dezenove planos saem `Untouched`, o acumulador termina sem região e o veredito já é `None`
/// **pela metade dos PLANOS** — então a mutação *"os metadados são sempre confinados"* passava, e o
/// gate era verde por vácuo sobre a metade que ele diz julgar. Com a escrita, os planos dizem
/// *confinado* e só a metade dos metadados pode recusar.
#[test]
fn a_structural_edit_is_not_confined() {
    let mut t = painted(256, 1, true);
    let id = t.layers.active().expect("camada ativa");
    let before = t.snapshot_model();
    t.set_layer_opacity(id, 0.5);
    // Uma escrita de canvas pequena, para os planos declararem uma janela e a prova cair inteira na
    // metade dos metadados.
    {
        let buf = std::sync::Arc::make_mut(&mut t.canvas_rgba);
        for px in buf.chunks_exact_mut(4).skip(600).take(40) {
            px[0] = px[0].wrapping_add(37);
        }
    }
    t.commit_structural_edit(before);
    assert!(
        t.undo.peek_confined_region(false).is_none(),
        "uma mudanca de opacidade muda o composite fora de retangulo nenhum"
    );
}

/// **Dois planos VAZIOS não azedam o veredito** — a lição que custou a wave (doc 28 §5.63).
///
/// O `split` manda para `Whole` tudo o que não sabe medir, e comprimento zero não é medível; as seis
/// superfícies da sessão de Sculpt são vazias num traço de pigmento comum. Lidas como `Whole`, elas
/// tornavam **todo** passo do produto não-confinado.
#[test]
fn two_empty_planes_do_not_sour_the_confinement() {
    use crate::undo_delta::StoredPlane;
    use crate::undo_delta::confine::PlaneReach;
    let empty: StoredPlane<f32> = StoredPlane::Whole {
        before: std::sync::Arc::new(Vec::new()),
        after: std::sync::Arc::new(Vec::new()),
    };
    assert_eq!(empty.reach(), PlaneReach::Untouched);
    let full: StoredPlane<f32> = StoredPlane::Whole {
        before: std::sync::Arc::new(vec![0.0; 4]),
        after: std::sync::Arc::new(vec![1.0; 4]),
    };
    assert_eq!(
        full.reach(),
        PlaneReach::Whole,
        "um plano com conteudo segue Whole"
    );
}
