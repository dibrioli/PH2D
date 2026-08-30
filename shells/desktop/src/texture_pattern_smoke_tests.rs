//! **Os gates da cena do TEXTURE PATTERN** (`PH2D_BUILD_SMOKE=76`) — irmãos do [`super`].
//!
//! ⚠️ O corte é por RESPONSABILIDADE: o irmão **constrói** a cena, este **mede-a**. E ela tem de
//! ser medida porque uma cena de smoke é a única coisa que o Enio lê — *uma cena que deixa de
//! conter o fenómeno aprova a ausência dele*, e nenhum gate do produto dá por isso.
//!
//! ⚠️ O ficheiro único batia no tecto de LOC do shell, que vive em `shells/desktop/tests/` e **não
//! é alcançado por `cargo test --bins`**.

use super::*;

#[cfg(test)]
mod lattice_tests {
    use super::{BOX, lei};
    use ph2d_vec_pattern::{PatternMode, TileKind};
    use ph2d_vec_scene::{PatternFill, PatternSource, Rgba8, VecPathId};

    fn fonte() -> PatternSource {
        PatternSource::Shape(VecPathId::from(1u64))
    }

    fn da_cena(kind: TileKind) -> PatternFill {
        lei(
            fonte(),
            kind,
            PatternMode::Tile,
            [1, 2, 3],
            BOX / 3.0,
            [0.0, 0.0],
        )
    }

    /// ⛔⛔⛔ **A CENA NÃO COMPENSA O PRODUTO** — o gate que faltava, e a ausência dele custou a
    /// vida inteira de uma feature.
    ///
    /// Esta cena escrevia `f.offset_denom = 2` à mão, e o construtor do produto nascia com `1`.
    /// ⇒ ela demonstrava tijolos e colmeias a ladrilhar **sobre um produto em que os chips *Brick*
    /// e *Column* eram inertes** — o artista carregava neles e via uma grade. A cena esteve verde
    /// o tempo todo, porque não tinha gate nenhum.
    ///
    /// ⚠️ **A lei geral:** uma cena de smoke tem de nascer no estado em que o artista a
    /// encontraria. Já custou um report do Enio uma vez (as formas desta mesma cena nasciam **sem
    /// contorno**, e a secção *Stroke* ficava inerte só aqui); esta é a segunda ocorrência no MESMO
    /// ficheiro, com o sujeito trocado.
    #[test]
    fn the_scene_does_not_hand_set_what_the_constructor_decides() {
        let cru = PatternFill::new(fonte(), [1.0, 1.0], Rgba8::new(1, 2, 3, 255));
        for kind in [
            TileKind::Grid,
            TileKind::BrickRow,
            TileKind::BrickCol,
            TileKind::Hex,
        ] {
            assert_eq!(
                da_cena(kind).offset_denom,
                cru.offset_denom,
                "{kind:?}: a cena escreve um desfasamento que o produto nao escreve - ela esta' a \
                 compensar o construtor, e um chip morto passaria por ela"
            );
        }
    }

    /// ⭐⭐⭐ **OS QUATRO RETICULADOS DESTA CENA LADRILHAM DE FACTO** — a régua sobre a lei ASSADA.
    ///
    /// A grade é uma célula por construção; os outros três **têm** de precisar de mais do que uma,
    /// senão são uma grade com outro nome. É esta linha que apanha o defeito se ele voltar por
    /// qualquer caminho (um construtor mudado, um `period()` mudado, uma cena mudada).
    #[test]
    fn every_lattice_in_this_scene_actually_tiles() {
        let px = [16u32, 16];
        assert_eq!(
            da_cena(TileKind::Grid).law(px).cells(),
            [1, 1],
            "a grade deixou de ser o neutro"
        );
        for kind in [TileKind::BrickRow, TileKind::BrickCol, TileKind::Hex] {
            let cells = da_cena(kind).law(px).cells();
            assert!(
                cells[0] * cells[1] > 1,
                "{kind:?} assa {cells:?} - e' uma grade com outro nome, e o chip nao muda um pixel"
            );
        }
    }
}

/// ⭐⭐⭐ **A ARTE DESTA CENA NÃO ENCAIXA CONSIGO PRÓPRIA — e é ISSO que a torna o smoke do aviso**
/// (plano 33, W10).
///
/// A dica de costura do painel só tem sujeito quando o ladrilho salta na volta. Esta arte salta —
/// a barra laranja do topo encosta no fundo branco/azul, e a meia-diagonal encosta no vazio — e o
/// salto não foi escolhido para isso: ele já lá estava desde a W5, porque a arte foi desenhada
/// **assimétrica nos dois eixos** para denunciar rotação e espelho.
///
/// ⚠️ **Sem este gate a cena podia ficar muda sem ninguém dar por isso.** Alguém a "arrumar" —
/// fechar a diagonal, uniformizar a barra — apagaria o aviso do smoke e o próximo leitor concluiria
/// que a feature não funciona. *Uma cena de smoke que deixa de conter o fenómeno aprova a ausência
/// dele.*
#[cfg(test)]
mod seam_hint_tests {
    use super::{ART, art_rgba};

    #[test]
    fn this_scenes_art_does_not_tile_and_that_is_what_the_hint_needs() {
        let t = ph2d_vec_pattern::bake(&art_rgba(), ART, ART, &ph2d_vec_pattern::TileLaw::grid())
            .expect("a grade encostada assa");
        let salto = ph2d_vec_pattern::wrap_seam(&t);
        assert!(
            ph2d_vec_pattern::seam_is_visible(salto),
            "a arte da cena passou a ENCAIXAR (salto {salto}, joelho {}) - o aviso de costura \
             deixou de ter sujeito e o smoke dele ficou mudo",
            ph2d_vec_pattern::SEAM_VISIBLE
        );
        println!("salto da arte da cena =76: {salto} niveis");
    }
}

/// ⭐⭐⭐ **A CENA TEM UMA ESTAMPA CUJA ARTE É UMA FORMA — e é ela que torna o aviso smokável**
/// (plano 33, W11).
///
/// O aviso de *arte apagada* só existe para a fonte-FORMA (uma fonte-IMAGEM que não resolve pode
/// estar a carregar). ⇒ sem um par **motivo + forma que o veste** nesta cena, o gesto do smoke —
/// escolher o motivo, apagá-lo, ver a estampa ficar chapada e o painel dizer porquê — deixa de
/// ter sujeito, e ninguém dá por isso.
///
/// ⚠️ **É um gate de FONTE, e ele DESCASCA os comentários antes de medir.** A cena não é
/// construível de um teste (o `build` pede um `App` com uma surface real), e sem o descascador este
/// próprio doc-comment — que cita a expressão que o gate procura — passaria por código. *Um gate
/// textual que não descasca comentários aprova-se a si mesmo.*
#[cfg(test)]
mod art_source_tests {
    /// O fonte sem comentários de linha — `//`, `///` e `//!`.
    fn sem_comentarios(src: &str) -> String {
        src.lines()
            .filter(|l| !l.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn this_scene_dresses_a_shape_in_another_shape() {
        let src = include_str!("texture_pattern_smoke.rs");
        let codigo = sem_comentarios(src);
        assert!(
            codigo.contains("PatternSource::Shape(motivo)"),
            "a cena deixou de vestir uma forma com OUTRA forma - o smoke do aviso de arte apagada \
             ficou sem sujeito, porque nao ha' o que apagar para o disparar"
        );
    }

    /// ⭐ O CONTROLO do descascador — sem ele, um bug que devolvesse vazio faria o gate acima
    /// passar sobre qualquer coisa. *Um filtro que casa zero imprime aprovado.*
    #[test]
    fn the_comment_stripper_keeps_code_and_drops_comments() {
        let s = "// PatternSource::Shape(motivo) num comentario\nlet x = 1;";
        let c = sem_comentarios(s);
        assert!(c.contains("let x = 1;"), "comeu o codigo");
        assert!(
            !c.contains("PatternSource::Shape(motivo)"),
            "nao descascou o comentario"
        );
    }
}
