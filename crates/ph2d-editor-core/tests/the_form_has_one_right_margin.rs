//! ⭐⭐⭐ **A marca da caixa de verificação está ANCORADA À DIREITA — na coluna do número.**
//!
//! O widget mais usado do app (**81** sítios fora do `ph2d-editor-core`, contra 58 do slider —
//! pesquisa `07` §15.1) desenhava a marca em `rect.x`, à esquerda, com o rótulo a seguir. A linha
//! de propriedade põe o **valor à direita**. ⇒ um formulário tinha **duas margens direitas** e duas
//! ordens de leitura, alternando linha sim linha não.
//!
//! Depois do redesenho as duas partilham a `property_box::value_column`, e por isso a marca e o
//! número acabam no **mesmo `x`**.
//!
//! # ⚠️ Porque é que este gate mede TINTA e não geometria
//!
//! Comparar `value_column(rect, 18.0, false)` com o que o pintor usa seria a mesma expressão dos
//! dois lados — vácua. Aqui pinta-se a caixa **sem rótulo** em duas larguras e exige-se que a tinta
//! **mude**: uma marca ancorada à esquerda é byte a byte a mesma nas duas, porque nada mais é
//! desenhado. O controlo ao lado prova que a medição distingue as duas âncoras.

use ph2d_a11y::NodeId;
use ph2d_editor_core::widget::{
    CHECKBOX_BOX_PX, Checkbox, CheckboxState, CheckboxValue, paint_checkbox,
};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

const ID: NodeId = NodeId(1);

/// A tinta de uma caixa **sem rótulo** numa linha de largura `w`.
///
/// ⚠️ Sem rótulo de propósito: com texto, a tinta mudaria com a largura por causa da truncagem, e
/// o gate ficaria verde sem nunca olhar para a âncora da marca.
fn mark_ink(w: f32, value: CheckboxValue) -> (Vec<u32>, Vec<u32>) {
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    let cb = Checkbox::new(ID, "")
        .state(CheckboxState::Normal)
        .value(value);
    paint_checkbox(
        &cb,
        Rect::new(0.0, 0.0, w, 22.0),
        &mut scene,
        &mut text,
        Theme::Forge,
    );
    let e = scene.inner().encoding();
    (e.path_data.clone(), e.draw_data.clone())
}

/// **Alargar a linha MOVE a marca** — logo ela segue a borda direita.
///
/// **Mutação que deve sangrar:** voltar a `Rect::new(rect.x, box_y, box_size, box_size)` no
/// `paint_checkbox` — as duas larguras passam a dar a mesma tinta e as 81 linhas do app voltam a
/// ter a marca na margem oposta à do número.
#[test]
fn the_mark_is_anchored_to_the_right_edge() {
    assert_ne!(
        mark_ink(200.0, CheckboxValue::Unchecked),
        mark_ink(300.0, CheckboxValue::Unchecked),
        "a marca nao se moveu ao alargar a linha: ela esta' ancorada a ESQUERDA, \
         e um formulario com numeros a' direita fica com duas margens"
    );
    // E com a marca desenhada — o glifo tem de viajar com a caixa, não ficar para trás.
    assert_ne!(
        mark_ink(200.0, CheckboxValue::Checked),
        mark_ink(300.0, CheckboxValue::Checked),
        "o VISTO nao acompanhou a caixa"
    );
}

/// **O CONTROLO: uma marca ancorada à esquerda daria tinta IDÊNTICA nas duas larguras.**
///
/// ⚠️ Sem isto, o gate acima poderia estar a detectar qualquer diferença — a largura da própria
/// linha, por exemplo — e não a âncora. Aqui reproduz-se a lei antiga à mão e mostra-se que ela é
/// **cega** à largura: é isso que o gate de cima deixou de ser.
#[test]
fn a_left_anchored_mark_would_be_blind_to_the_width() {
    use ph2d_editor_core::paint::{fill_rounded_rect, resolve};
    let ink = |w: f32| {
        let mut scene = VectorScene::new();
        let r = Rect::new(0.0, 0.0, w, 22.0);
        // A lei antiga: a caixa em `rect.x`, sem olhar para `rect.w`.
        let box_rect = Rect::new(
            r.x,
            r.y + (r.h - CHECKBOX_BOX_PX) * 0.5,
            CHECKBOX_BOX_PX,
            CHECKBOX_BOX_PX,
        );
        fill_rounded_rect(
            &mut scene,
            box_rect,
            4.0,
            resolve(ph2d_tokens::ColorToken::Bg1, Theme::Forge),
        );
        let e = scene.inner().encoding();
        (e.path_data.clone(), e.draw_data.clone())
    };
    assert_eq!(
        ink(200.0),
        ink(300.0),
        "o controlo nao reproduz a lei antiga: se a ancora esquerda ja' respondesse a' largura, \
         o gate irmao nao estaria a medir a ancora"
    );
}

/// ⛔ **PONTO CEGO NOMEADO: a truncagem do rótulo NÃO é observável neste arnês.**
///
/// O `TextSystem::without_system_fonts()` — o que todo gate de widget desta casa usa — não tem
/// glifos, então `paint_text` **não produz tinta nenhuma**: medido em 2026-09-03, `"On"` e um
/// rótulo de 40 caracteres dão `path_data` byte a byte igual (só a caixa). ⇒ um gate de tinta
/// sobre o corte do rótulo ficaria **verde por vacuidade**, com o nome de uma protecção — que é
/// exactamente a espécie que a `07` §13.4 pagou.
///
/// ⇒ **a lei do corte não se gateia aqui.** Ela é a `property_box::fit_label`, partilhada e não
/// copiada (é essa a razão de ela ter passado a `pub(crate)`), e quem a mudar mexe num sítio só.
///
/// ⏳ O que destravaria um gate a sério é um arnês com uma fonte **vendorizada** — o mesmo que a
/// paridade de texto precisaria. Não existe hoje, e inventá-lo aqui era outra obra.
#[test]
fn the_label_truncation_is_not_measurable_here_and_that_is_recorded() {
    let ink = |label: &str| {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        paint_checkbox(
            &Checkbox::new(ID, label),
            Rect::new(0.0, 0.0, 120.0, 22.0),
            &mut scene,
            &mut text,
            Theme::Forge,
        );
        scene.inner().encoding().path_data.clone()
    };
    assert_eq!(
        ink("On"),
        ink("Cast shadows onto every receiving surface"),
        "o arnes GANHOU glifos: a truncagem passou a ser observavel, e este gate deve ser \
         substituido por um que a MEÇA em vez de registar que nao a mede"
    );
}

/// ⭐⭐⭐ **O INTERRUPTOR DESLIZANTE PINTA A MESMA MARCA QUE A CAIXA** — tinta byte a byte igual.
///
/// Decisão do Enio (2026-09): *«as pílulas e o interruptor deslizante podem sair»*. A execução é a
/// da pesquisa `07` §5.5 — **fusão de PINTURA**, porque o `WidgetKind::Toggle` tem `code() == 2` e
/// esse número viaja em documento: apagar o widget partiria todo painel autorado já gravado.
///
/// ⚠️ **Este gate mede a IGUALDADE, que é a afirmação forte.** Um gate que só dissesse *«o toggle
/// já não desenha um círculo»* ficaria verde sobre qualquer desenho novo — inclusive um terceiro,
/// que é precisamente o que a fusão existe para impedir.
///
/// **Mutação que deve sangrar:** dar ao `paint_toggle` qualquer tinta própria (uma pílula, um raio
/// diferente, um `Radius::Full`) — as duas deixam de coincidir.
#[test]
fn the_switch_paints_the_very_same_mark_as_the_checkbox() {
    use ph2d_editor_core::widget::{Toggle, ToggleState, paint_toggle};
    let r = Rect::new(0.0, 0.0, 60.0, 24.0);
    let switch_ink = |on: bool, st: ToggleState| {
        let mut scene = VectorScene::new();
        paint_toggle(
            &Toggle::new(ID, "").on(on).state(st),
            r,
            &mut scene,
            Theme::Forge,
        );
        let e = scene.inner().encoding();
        (e.path_data.clone(), e.draw_data.clone())
    };
    let box_ink = |v: CheckboxValue, st: CheckboxState| {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        // ⚠️ Rótulo VAZIO: o `Toggle` nunca pintou texto (o `label` dele é só a11y), então a
        // comparação justa é contra a caixa sem rótulo. Os três painéis que o usam pintam o
        // rótulo eles próprios, ao lado.
        paint_checkbox(
            &Checkbox::new(ID, "").value(v).state(st),
            r,
            &mut scene,
            &mut text,
            Theme::Forge,
        );
        let e = scene.inner().encoding();
        (e.path_data.clone(), e.draw_data.clone())
    };
    for (on, v) in [
        (false, CheckboxValue::Unchecked),
        (true, CheckboxValue::Checked),
    ] {
        for (ts, cs) in [
            (ToggleState::Normal, CheckboxState::Normal),
            (ToggleState::Hovered, CheckboxState::Hovered),
            (ToggleState::Focused, CheckboxState::Focused),
            (ToggleState::Disabled, CheckboxState::Disabled),
        ] {
            assert_eq!(
                switch_ink(on, ts),
                box_ink(v, cs),
                "o interruptor ({ts:?}, on={on}) pintou tinta PROPRIA — a fusao de pintura \
                 desfez-se, e o app volta a ter duas linguagens para o mesmo booleano"
            );
        }
    }
}
