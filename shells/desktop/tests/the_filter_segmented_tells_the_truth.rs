//! **O SEGMENTADO DE TEXTURE FILTER TEM DE DIZER O QUE O ECRÃ DESENHA — E OFERECER TUDO O QUE O
//! MOTOR SABE FAZER.**
//!
//! # O defeito que este gate existe para impedir
//!
//! Auditoria de 2026-08-21 ([`docs/Sprite_projeto/20`](../../../docs/Sprite_projeto/20_auditoria_do_inspector_2026-08-21.md) §2.2):
//! o painel pintava três abas e escolhia a acesa com `.selected((filter_tag as usize).min(2))`.
//! Como o renderer manda as tags **`1 | 3 | 5`** para `Nearest`, o `.min(2)` acendia **«Linear»**
//! para as tags 3 e 5 — ou seja, **o painel afirmava o oposto do que a sprite desenhava**, e só
//! naquelas sprites cuja aparência (pixel duro) é a razão de o controlo existir.
//!
//! Duas coisas o tornavam invisível:
//!
//! 1. **A lei vivia dentro do `sampler_from_tags`, que precisa de um `wgpu::Device`** — medi-la
//!    exigia GPU, por isso ninguém a media. Hoje é a `const fn`
//!    [`filter_tag_magnifies_by_point`], e este gate corre sem adapter nenhum.
//! 2. **Nenhuma crate via os dois lados.** O painel é chrome e não depende do `ph2d-ecs` (as
//!    etiquetas são hardcoded de propósito, como o `BLEND_LABELS` da §10); o `ph2d-render` não
//!    conhece o painel. O shell vê os três — é aqui que a conta fecha, o mesmo sítio e a mesma
//!    razão do `the_emissive_ceiling_is_one_law`.
//!
//! # E o teto (CLAUDE.md §0.0)
//!
//! ⚠️ O motor entrega **sete** modos — mipmap trilinear real e anisotropia 16× desde 2026-06-18 —
//! e o painel oferecia **três**. As outras quatro eram inexprimíveis por gesto nenhum, embora o
//! componente `TextureFilter` esteja no registry de cena (uma tag ≥3 escrita por script sobrevivia
//! ao save/load e o painel não a sabia mostrar). *O teto era o do painel, não o do hardware.*

use ph2d_ecs::FilterMode;
use ph2d_panel_inspector::FILTER_LABELS;
use ph2d_render::image_filter::filter_tag_magnifies_by_point;

/// Toda variante de [`FilterMode`], **exaustiva por construção**: acrescentar uma variante torna o
/// `match` abaixo não-exaustivo e isto deixa de compilar. *Uma lista à mão que o compilador guarda
/// não é uma lista à mão.*
fn every_filter_mode() -> Vec<FilterMode> {
    let all = vec![
        FilterMode::Inherit,
        FilterMode::Nearest,
        FilterMode::Linear,
        FilterMode::NearestMipmap,
        FilterMode::LinearMipmap,
        FilterMode::NearestAniso,
        FilterMode::LinearAniso,
    ];
    for m in &all {
        // ⚠️ Exaustivo de propósito, e sem braço `_`: é este `match` que faz uma variante nova
        // parar o build em vez de nascer inalcançável no painel.
        match m {
            FilterMode::Inherit
            | FilterMode::Nearest
            | FilterMode::Linear
            | FilterMode::NearestMipmap
            | FilterMode::LinearMipmap
            | FilterMode::NearestAniso
            | FilterMode::LinearAniso => {}
        }
    }
    all
}

/// **(1) O painel oferece um segmento por modo que o motor tem** — o teto é o do hardware.
#[test]
fn the_filter_segmented_offers_every_mode_the_engine_has() {
    let modes = every_filter_mode();
    assert_eq!(
        FILTER_LABELS.len(),
        modes.len(),
        "o Inspector oferece {} opcoes de Texture Filter e o motor tem {} modos — as que sobram \
         sao INALCANCAVEIS por gesto nenhum, embora o componente `TextureFilter` viaje no save.\n\
         Acrescente o rotulo em `sections/sampling.rs::FILTER_LABELS` E o id na posicao \
         correspondente de `ids::INSP_SAMPLE_FILTER` (a POSICAO e' a tag).\n\
         ⚠️ CLAUDE.md §0.0: o teto e' o do HARDWARE, nunca o do painel.",
        FILTER_LABELS.len(),
        modes.len()
    );
}

/// **(2) A posição é a tag** — o despacho conta com isso, e sem esta conta o segmento aceso e a
/// escrita divergem em silêncio.
#[test]
fn the_position_in_the_segmented_is_the_tag_itself() {
    for (i, mode) in every_filter_mode().iter().enumerate() {
        assert_eq!(
            mode.tag() as usize,
            i,
            "a variante {mode:?} tem tag {} mas esta' na posicao {i} da lista — o despacho \
             (`event_ordering.rs`) deriva a tag da POSICAO do id em `ids::INSP_SAMPLE_FILTER`, \
             entao esta divergencia faz o clique escrever outro modo",
            mode.tag()
        );
    }
}

/// **(3) O rótulo não mente sobre o que o ecrã desenha** — o defeito exato que shipou.
///
/// ⚠️ A regra é de **ampliação**: um rótulo que começa por `Near` promete pixel duro, e só pode
/// estar sobre uma tag que o renderer amplia por ponto. `Inherit` fica de fora porque não promete
/// nada — ele delega.
#[test]
fn the_filter_segmented_tells_the_truth_about_what_renders() {
    for (i, label) in FILTER_LABELS.iter().enumerate() {
        if *label == "Inherit" {
            continue;
        }
        let tag = u8::try_from(i).expect("as tags cabem num u8");
        let label_promises_point = label.starts_with("Near");
        assert_eq!(
            label_promises_point,
            filter_tag_magnifies_by_point(tag),
            "o rotulo «{label}» (tag {tag}) promete {} mas o renderer amplia por {}.\n\
             Foi exatamente esta divergencia que shipou: `.min(2)` acendia «Linear» sobre as tags \
             3 e 5, que desenham com pixel duro.\n\
             ⚠️ A lei do renderer e' `ph2d_render::image_filter::filter_tag_magnifies_by_point`; \
             o rotulo tem de a seguir, nunca o contrario.",
            if label_promises_point {
                "pixel duro"
            } else {
                "interpolacao"
            },
            if filter_tag_magnifies_by_point(tag) {
                "ponto"
            } else {
                "interpolacao"
            }
        );
    }
}

/// **(4) Nenhum rótulo se repete.** Dois segmentos com o mesmo nome são indistinguíveis para quem
/// clica — e foi assim que as tags 3 e 5 se disfarçaram de «Linear» durante meses.
#[test]
fn no_two_filter_segments_carry_the_same_label() {
    let mut seen: Vec<&str> = Vec::new();
    for label in FILTER_LABELS {
        assert!(
            !seen.contains(&label),
            "o rotulo «{label}» aparece duas vezes no segmentado de Texture Filter — o artista \
             nao consegue distinguir os dois modos, e o que ele escolhe passa a ser sorte"
        );
        seen.push(label);
    }
}
