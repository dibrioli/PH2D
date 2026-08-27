//! As provas da resolução da máscara — a metade que se mede sem uma GPU.
//!
//! ⚠️ **O que este arquivo NÃO pode testar é a metade da view**, e a razão é a mesma que o
//! `appearance_of` já documenta: um `TextureAtlas` só se constrói com um contexto de GPU. O que
//! se prova aqui é a **chave** — a coisa que decide se o passe refaz os bind groups —, e ela é
//! aritmética pura.

use super::{Resolved, key_of};

fn r(texture_id: u32, uv_rect: [f32; 4]) -> Resolved {
    Resolved {
        texture_id,
        uv_rect,
    }
}

/// **A chave TEM de ver o sub-rect, não só o id.**
///
/// ⚠️ Este é o gate que impede o defeito mais silencioso desta feature: duas células do atlas
/// partilhado têm o MESMO `texture_id` (`0`), então uma chave que fosse só o id leria *"a mesma
/// imagem"* ao trocar entre duas sprites empacotadas — o artista escolheria outra coisa e a tela
/// não mudaria, sem nada vermelho em lado nenhum.
#[test]
fn two_atlas_cells_are_two_different_masks() {
    // ⚠️ **Duas células a SÉRIO.** A 1.ª versão escrevia a segunda como `[0.5, 0.5, 0.25, 0.25]`,
    // que em `[u0, v0, u1, v1]` é um rect invertido e não uma célula — inofensivo para o que este
    // gate mede (o `key_of` come bits), mas *a fixtura não continha o fenómeno que o doc nomeia*,
    // e é a mesma classe de fixtura que já custou esta feature inteira uma vez.
    let a = key_of(r(0, [0.0, 0.0, 0.25, 0.25]));
    let b = key_of(r(0, [0.5, 0.5, 0.75, 0.75]));
    assert_ne!(a, b, "duas celulas do MESMO atlas colidiram na chave");
    // E o mesmo rect no mesmo id é a mesma escolha — senão os bind groups refar-se-iam
    // por quadro, que é exactamente o que a chave existe para evitar.
    assert_eq!(a, key_of(r(0, [0.0, 0.0, 0.25, 0.25])));
}

#[test]
fn two_individual_textures_are_two_different_masks() {
    let full = [0.0, 0.0, 1.0, 1.0];
    assert_ne!(key_of(r(7, full)), key_of(r(8, full)));
    assert_eq!(key_of(r(7, full)), key_of(r(7, full)));
}

/// Um id assado e um individual com o mesmo número baixo não existem ao mesmo tempo (o bit 31
/// separa-os), mas a chave não depende disso — ela come o id inteiro.
#[test]
fn the_cooked_namespace_does_not_collide_with_the_individual_one() {
    let full = [0.0, 0.0, 1.0, 1.0];
    let cooked = 1_u32 << 31 | 7;
    assert_ne!(key_of(r(7, full)), key_of(r(cooked, full)));
}

/// **Controle da própria régua:** uma chave que ignorasse tudo passaria os testes de igualdade
/// acima e falharia este. Ela tem de DISTINGUIR num varrimento largo.
#[test]
fn the_key_separates_a_whole_sweep_of_choices() {
    let mut seen = std::collections::BTreeSet::new();
    for id in [0_u32, 1, 2, 1 << 31] {
        for x in 0..4 {
            for y in 0..4 {
                let f = |v: i32| v as f32 * 0.25;
                seen.insert(key_of(r(id, [f(x), f(y), 0.25, 0.25])));
            }
        }
    }
    assert_eq!(seen.len(), 4 * 4 * 4, "a chave colidiu no varrimento");
}

/// **A CÉLULA DE ATLAS É A FONTE QUE DISTINGUE AS TRÊS.**
///
/// ⚠️ Este é o gate que a célula da folha 11 pedia sem lhe saber o nome: com `Individual` e
/// `CookedTexture` (rect inteiro) o aspecto da textura É o da imagem, então uma implementação
/// que lesse o da textura passaria as duas e falharia só no atlas — *"funciona com umas imagens
/// e falha em silêncio com outras"*, literal.
///
/// ⚠️ **O rect é DERIVADO de uma [`ph2d_render::AtlasRegion`] real, e é essa a correcção que
/// interessa.** A 1.ª versão deste gate escrevia `[0.25, 0.5, 512/2048, 256/2048]` à mão, lendo
/// o rect como `[x, y, w, h]` — a mesma leitura errada que o código fazia —, e por isso ele
/// passava sobre um defeito que apagava a feature inteira. *Uma fixtura escrita na convenção do
/// autor concorda com o código do autor.* Derivá-la da função que a PRODUZ é o que impede isso.
#[test]
fn an_atlas_cell_reports_its_own_aspect_not_the_shared_sheets() {
    // O átlas é QUADRADO. Uma célula de 512×256 dentro dele é 2:1.
    const SHEET: u32 = 2048;
    let cell = ph2d_render::AtlasRegion {
        x: 512,
        y: 1024,
        w: 512,
        h: 256,
    }
    .uv(SHEET);
    let got = super::image_aspect(cell, SHEET, SHEET);
    assert!(
        (got - 2.0).abs() < 0.02,
        "a celula e' 2:1 e leu {got} (rect {cell:?}) — o aspecto veio da FOLHA, nao da imagem"
    );
    // E uma célula QUADRADA num átlas quadrado le^ 1:1 — o caso desta cena, e o que a
    // leitura errada dava como 5,26:1.
    let square = ph2d_render::AtlasRegion {
        x: 1088,
        y: 0,
        w: 256,
        h: 256,
    }
    .uv(8192);
    let got = super::image_aspect(square, 8192, 8192);
    assert!((got - 1.0).abs() < 0.02, "celula quadrada leu {got}");
    // E uma textura que é dela própria devolve o aspecto dela.
    assert!((super::image_aspect([0.0, 0.0, 1.0, 1.0], 640, 480) - 4.0 / 3.0).abs() < 1e-4);
    assert!((super::image_aspect([0.0, 0.0, 1.0, 1.0], 256, 256) - 1.0).abs() < 1e-6);
}

#[test]
fn a_degenerate_rect_is_square_never_a_nan() {
    for r in [
        [0.0, 0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 0.0],
        // E um rect INVERTIDO — o que a leitura errada produzia.
        [0.5, 0.5, 0.25, 0.25],
    ] {
        let a = super::image_aspect(r, 1024, 1024);
        assert_eq!(a, 1.0, "{r:?}");
        assert!(a.is_finite());
    }
    assert_eq!(super::image_aspect([0.0, 0.0, 1.0, 1.0], 0, 0), 1.0);
}

/// ⛔⛔ **UM RECT DEGENERADO É *NENHUMA* MÁSCARA, NÃO UMA MÁSCARA COLAPSADA.**
///
/// A auditoria de 2026-08-27 seguiu a cadeia até ao pixel: `TextureAtlas::region_uv` devolve
/// `[0,0,0,0]` para uma chave desconhecida (nome escrito antes de a imagem carregar, célula
/// libertada) e o `sprite_appearance` **nunca** devolve `None` para `SpriteSource::Atlas` ⇒ o
/// `mask` entregava uma máscara com o rect neutro **e a textura do artista ligada**, e o shader
/// fazia `dirt_uv = (0,0)` para todos os pixels: o texel `(0,0)` do átlas — o canto da primeira
/// imagem importada — somado chapado sobre o halo inteiro, vezes a intensidade.
///
/// ⚠️ **O modo de falha depende dos DADOS**: átlas vazio ⇒ transparente e invisível; átlas cheio
/// ⇒ o brilho fica de uma cor sólida. *Um defeito que só aparece com a segunda imagem importada
/// é o que um smoke de uma imagem só nunca vê.*
///
/// ⚠️ O gate que existia (`the_neutral_framing_is_not_what_makes_the_frame_identical`) afirmava o
/// neutro **em isolamento**, raciocinando sobre *«quando não há imagem escolhida»* — um gate
/// sobre a DECLARAÇÃO enquanto o executor emparelhava o neutro com uma view real.
#[test]
fn a_degenerate_rect_is_no_mask_at_all_not_a_collapsed_one() {
    for r in [
        [0.0, 0.0, 0.0, 0.0],      // o que `region_uv` devolve para chave desconhecida
        [0.25, 0.25, 0.25, 0.5],   // largura zero (célula de 1 px, o inset colapsa)
        [0.25, 0.25, 0.5, 0.25],   // altura zero
        [0.5, 0.5, 0.25, 0.25],    // invertido — o que a leitura `[x,y,w,h]` produzia
        [0.0, 0.0, f32::NAN, 1.0], // não-finito
        [0.0, 0.0, f32::INFINITY, 1.0], // idem
    ] {
        assert!(
            !super::rect_is_a_mask(r),
            "{r:?} tinha de ser LIDO como ausencia de mascara"
        );
    }
    // ⚠️ **CONTROLE — sem ele um `rect_is_a_mask` que devolvesse `false` sempre passaria**, e a
    // feature inteira ficaria desligada em silêncio. As duas fontes que dão `[0,0,1,1]` e uma
    // célula de átlas a sério.
    for r in [[0.0, 0.0, 1.0, 1.0], [0.25, 0.5, 0.5, 0.75]] {
        assert!(super::rect_is_a_mask(r), "{r:?} e' uma mascara legitima");
    }
}
