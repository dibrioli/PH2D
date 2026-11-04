//! ⭐ **R5: o que abre sobre o texto fica por cima do HUD** (`panel/camada_das_listas.rs`): o HUD de baixo
//! pinta-se depois da aba e tapava a caixa junto ao fundo. Não tem alvos, por isso os cliques não mudam.

use crate::{Deco, frame};
use ph2d_editor_core::interaction::HitIndex;
use ph2d_editor_core::panel::camada_das_listas::{abre, fecha};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_code::{CodeView, RenameView};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

/// *Mutação: tirar o `por_cima` de `CodeView::paint` ⇒ o que se pinta depois tapa a caixa.*
#[test]
fn a_caixa_de_renomear_vai_para_a_camada() {
    let (buf, d) = (
        ph2d_code_buffer::Buffer::new("local nome = 1\n"),
        Deco::default(),
    );
    let mut ts = TextSystem::without_system_fonts();
    let mut v = CodeView::default();
    let area = Rect::new(0.0, 0.0, 800.0, 400.0);
    let mut spans = |r| crate::spans_de(&buf, r);
    let mut marcas = crate::marcas(&d);
    let mut f = frame(&buf, &d, &mut spans, &mut marcas);
    f.rename = Some(RenameView {
        word: 6..10,
        field: None,
        hint: "",
        error: Some("no"),
    });
    let mut scene = VectorScene::new();
    scene.record();
    abre(&scene);
    v.paint(f, area, &mut scene, &mut ts, &Theme::default());
    let do_texto = scene.take_record().len();
    let mut hits = HitIndex::new();
    fecha(&mut scene, &mut hits);
    assert!(do_texto > 0, "a aba não pintou");
    assert!(
        !scene.take_record().is_empty(),
        "a caixa ficou na cena da aba"
    );
}
