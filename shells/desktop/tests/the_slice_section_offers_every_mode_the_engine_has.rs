//! **A §5 9-Slice oferece TODOS os modos que o motor tem — e a posição É a tag.**
//!
//! ⚠️ **Este gate só pode viver aqui.** O painel do Inspector é chrome e **não depende do
//! `ph2d-ecs`**, de propósito: os seus rótulos são constantes locais. A shell é a única crate que
//! vê as duas metades, e é por isso que o gate irmão da §9
//! ([`the_filter_segmented_tells_the_truth`]) também mora neste diretório.
//!
//! # A classe de defeito que isto apanha
//!
//! É exatamente a que a auditoria de 2026-08-21 mediu na §9: o painel capava o Texture Filter em
//! **três** segmentos enquanto o motor tinha **sete**, e pintava «Linear» sobre sprites que
//! desenhavam com pixel duro. Quatro modos eram inalcançáveis por gesto nenhum, e todo gate de
//! compilação continuava verde — *um teto escrito à mão abaixo do que o motor faz*
//! (CLAUDE.md §0.0).
//!
//! A §5 nasce com o gate no mesmo commit da seção, e não dois meses depois.

use ph2d_ecs::{SliceDrawMode, SliceRegion, SliceTileMode, TileRegionMode};
use ph2d_editor::ids;

/// **(1) Um botão por variante — nos três enums com escolha na UI.**
///
/// Menos botões que variantes = modos que o artista não consegue escolher. Mais botões que
/// variantes = um botão que despacha uma tag que o motor não sabe ler.
#[test]
fn every_engine_variant_has_exactly_one_control() {
    assert_eq!(
        ids::INSP_SLICE_MODE.len(),
        SliceDrawMode::ALL.len(),
        "o Draw Mode tem {} botoes para {} variantes do motor",
        ids::INSP_SLICE_MODE.len(),
        SliceDrawMode::ALL.len()
    );
    assert_eq!(
        ids::INSP_SLICE_TILE_MODE.len(),
        SliceTileMode::ALL.len(),
        "o Tile Mode tem {} botoes para {} variantes",
        ids::INSP_SLICE_TILE_MODE.len(),
        SliceTileMode::ALL.len()
    );
    assert_eq!(
        ids::INSP_SLICE_REGION.len(),
        SliceRegion::ALL.len(),
        "a grelha tem {} celulas para {} regioes",
        ids::INSP_SLICE_REGION.len(),
        SliceRegion::ALL.len()
    );
}

/// **(2) A POSIÇÃO no array de ids é a TAG do enum.**
///
/// O despacho deriva a tag de `position(|&o| o == id)` e a shell fecha com `from_tag`. Nada no
/// tipo garante isto: reordenar o array de ids sem reordenar o enum põe cada botão a escrever o
/// modo do vizinho — silenciosamente, porque ambos são `u8` válidos.
#[test]
fn the_position_in_the_array_is_the_tag() {
    for (i, m) in SliceDrawMode::ALL.iter().enumerate() {
        assert_eq!(m.tag() as usize, i, "SliceDrawMode {m:?} fora da posicao");
        assert_eq!(SliceDrawMode::from_tag(i as u8), *m);
    }
    for (i, m) in SliceTileMode::ALL.iter().enumerate() {
        assert_eq!(m.tag() as usize, i, "SliceTileMode {m:?} fora da posicao");
        assert_eq!(SliceTileMode::from_tag(i as u8), *m);
    }
    for (i, m) in TileRegionMode::ALL.iter().enumerate() {
        assert_eq!(m.tag() as usize, i, "TileRegionMode {m:?} fora da posicao");
        assert_eq!(TileRegionMode::from_tag(i as u8), *m);
    }
}

/// **(3) A grelha 3×3 do painel e as regiões do motor concordam CÉLULA A CÉLULA.**
///
/// ⚠️ Duas listas descrevem a mesma grelha em crates diferentes (o painel não pode importar o
/// motor). Esta é a única coisa que impede a célula «TR» do painel escrever no modo da «BL» do
/// motor — um erro que o artista veria como «cliquei no canto e o outro canto mudou», e que
/// nenhum teste de crate isolada pode ver.
#[test]
fn the_painted_grid_agrees_with_the_engine_cells() {
    // ⚠️ **Lê a tabela REAL** — `ph2d_panel_inspector::REGION_CELLS` é a mesma constante que o
    // pintor itera. A primeira versão deste gate tinha uma cópia local, e uma mutação que
    // trocava duas células no painel passava VERDE: ele comparava a sua própria cópia com o
    // motor. *Um gate que guarda a sua cópia mede-se a si mesmo.*
    let painted = ph2d_panel_inspector::REGION_CELLS;
    for (i, r) in SliceRegion::ALL.iter().enumerate() {
        assert_eq!(
            painted[i],
            r.cell(),
            "a celula {i} pintada e' {:?} mas a regiao {:?} do motor mora em {:?}",
            painted[i],
            r,
            r.cell()
        );
        assert_eq!(*r as usize, i, "{r:?} nao esta na sua propria posicao");
    }
}

/// **(4) Os rótulos do painel dizem a verdade sobre o que o motor faz.**
///
/// O rótulo `Simple` tem de ser o único modo que **não** entra no caminho de nove quads, e
/// `Sliced`/`Tiled` têm de entrar. Um rótulo que promete mais do que o modelo entrega é a classe
/// de defeito nº 2 da auditoria.
#[test]
fn the_labels_promise_what_the_model_delivers() {
    assert!(
        !SliceDrawMode::Simple.is_nine(),
        "«Simple» entrou no caminho de nove quads — anexar o componente deixaria de ser inerte"
    );
    assert!(SliceDrawMode::Sliced.is_nine(), "«Sliced» nao fatia nada");
    assert!(SliceDrawMode::Tiled.is_nine(), "«Tiled» nao fatia nada");
    // E o componente recém-anexado nasce no modo que não muda nada.
    assert_eq!(ph2d_ecs::SliceNine::INERT.draw_mode, SliceDrawMode::Simple);
}
