//! ⭐⭐ **A LINHA DE COR QUE O SELECTOR ESTÁ A EDITAR TRAÇA O ANEL DE FOCO.**
//!
//! ⛔⛔ O selector de cor da casa é UM e flutua sobre o canvas: com ele aberto, o anel na amostra é a
//! única resposta a *«o que é que esta roda está a mudar?»*. As linhas de cor escritas à mão que a
//! porta [`ph2d_editor_core::property_row::paint_color_row`] substituiu mostravam-no (a borda
//! `Accent` do Pincel e do Papel do Painter; o `Focused` da cor do modelador), e a porta NÃO — cada
//! conversão para ela tirava ao artista essa resposta (2026-09-23).
//!
//! ⚠️ **A régua é a contagem de caminhos**: num tema da família moderna a amostra em repouso não tem
//! anel e a focada tem (o preenchimento do anel é um caminho a mais). ⭐ O CONTROLO é o selector
//! aberto noutra amostra — ela tem de pintar exactamente o que pinta com ele fechado, senão o gate
//! mediria «há um selector aberto» e não «é ESTA amostra».

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::property_row::{Seccao, paint_color_row};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

const ID: NodeId = NodeId(7);

fn segmentos(aberto_em: Option<NodeId>) -> u32 {
    ph2d_editor_core::paint::set_ui_look(ph2d_tokens::UiLook::Redesign);
    let mut store = WidgetStore::default();
    store.set_picker_target(aberto_em);
    let mut scene = VectorScene::new();
    let mut ts = TextSystem::new();
    let mut hits = HitIndex::default();
    let _ = paint_color_row(
        &mut scene,
        &mut ts,
        // ⚠️ Um tema da família MODERNA, de propósito: no clássico o repouso e o foco têm os DOIS
        //    anel (só a cor muda), e a contagem de caminhos não vê uma cor.
        Theme::Dark,
        &mut hits,
        &store,
        0.0,
        280.0,
        0.0,
        "Color",
        ID,
        [200, 40, 40, 255],
        false,
        Seccao::apenas_campos(1),
    );
    scene.inner().encoding().n_path_segments
}

#[test]
fn a_linha_de_cor_aberta_no_seletor_traca_o_anel() {
    assert!(
        Theme::Dark.is_modern(),
        "o tema da régua deixou de ser moderno — no clássico ela não distingue foco de repouso"
    );
    let fechado = segmentos(None);
    let noutra = segmentos(Some(NodeId(99)));
    let nesta = segmentos(Some(ID));
    assert!(
        fechado > 0,
        "o controlo falhou: a linha de cor não pintou nada"
    );
    assert_eq!(
        noutra, fechado,
        "com o selector aberto NOUTRA amostra esta linha mudou de desenho — o anel não sabe de quem é"
    );
    assert!(
        nesta != fechado,
        "a linha de cor que o selector está a editar pinta o mesmo que em repouso ({nesta} \
         segmentos) — o artista não vê QUAL cor a roda está a mudar"
    );
}
