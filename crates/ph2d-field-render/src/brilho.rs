//! ⭐⭐⭐ **O BRILHO chega à imagem** — o passe da `W7` (`docs/Render3d/12`).
//!
//! # ⚠️ Porque é um ficheiro irmão do [`super::shade_render`]
//!
//! ⛔ **Corte por RESPONSABILIDADE, forçado pelo tecto de `700` linhas** e melhor por isso: o irmão
//! responde *«que luz é que este pixel devolve ao olho»* e isto responde *«e que luz é que os
//! VIZINHOS dele derramam por cima»*. São perguntas de granularidade diferente — uma é por pixel,
//! a outra é do quadro —, e é exactamente essa diferença que faz o brilho não caber na
//! [`ph2d_style`] (§4 daquele doc).
//!
//! # ⭐⭐ A REGRA DA COMPOSIÇÃO é UMA SÓ, e a alternativa está refutada por medição
//!
//! O halo nasce em **cena-linear** (lido do HDR, que é o que o torna honesto) e é somado **depois
//! do olhar**, com a mesma aritmética sobre a peça e sobre o fundo.
//!
//! ⛔⛔ A alternativa — somar em cena-linear onde há peça e em ecrã onde há fundo — **está
//! refutada neste mesmo módulo** (`docs/Render3d/10` §11.3): *um `if` por pixel desenha a fronteira
//! entre os dois ramos*, e a cura que o fez pintou um **fio escuro serrilhado** na silhueta. Aqui o
//! motivo do `if` seria ainda mais forte, porque o fundo **nunca passou pelo olhar** (o artista dá-o
//! em bytes) e não há como o converter de volta.
//!
//! ⚠️ **O preço declarado:** `look(a) + look(b) ≠ look(a + b)`, logo o halo não estoura o branco
//! como estouraria se entrasse antes do tonemapper. É a mesma escolha que um compositor faz, e a
//! saída que a desfaz — um olhar **invertível** para trazer o fundo à cena — fica NOMEADA e por
//! medir.

use crate::{Gbuffer, Lighting, Orbit, Presentation, Surfaces};

/// ⭐⭐⭐ **O QUADRO EM CENA-LINEAR** — o que o brilho lê.
///
/// ⚠️ **Só os píxeis da PEÇA entram.** O fundo é uma cor de bytes que nunca passou pelo olhar, e
/// pô-lo aqui seria inventar uma luz de cena que ninguém autorou. ⛔ E isso **não** abre a fronteira
/// que o cabeçalho recusa: quem não entra contribui com `0` para o borrão, que é o que «não há luz
/// aqui» significa — a descontinuidade fica no que é DERRAMADO, nunca em como é somado.
///
/// ⚠️⚠️ **Ele re-avalia o sombreamento dos píxeis da peça, e isso é dívida MEDIDA e NOMEADA:** o
/// laço do byte não pode escrever aqui sem partir o corpo dele em dois despachos, e o preço de o
/// fazer mal (dois programas a pintar a mesma imagem) é maior do que o de o correr duas vezes numa
/// rota que o artista LIGOU. *A fusão dos dois é uma optimização com endereço, não um defeito.*
#[must_use]
pub(crate) fn campo_de_cena(
    g: &Gbuffer,
    cam: &Orbit,
    surfaces: &Surfaces<'_>,
    light: &Lighting<'_>,
    pres: &Presentation,
) -> Vec<[f32; 3]> {
    use rayon::prelude::{IndexedParallelIterator, ParallelIterator, ParallelSliceMut};

    let (w, h) = (g.width as usize, g.height as usize);
    let mut out = vec![[0.0f32; 3]; w * h];
    if w == 0 || h == 0 {
        return out;
    }
    let screen = crate::camera::Screen::new(g.width, g.height, cam.half_extent);
    let basis = crate::shade_render::ViewBasis::of(cam);
    #[allow(clippy::cast_possible_truncation)]
    let pixel_world = crate::shade_render::boundary_world(cam.half_extent, w.min(h) as u32);

    out.par_chunks_mut(w).enumerate().for_each(|(y, row)| {
        for (x, px) in row.iter_mut().enumerate() {
            let i = y * w + x;
            if !g.hit[i] {
                continue;
            }
            let v = crate::shade_render::view_direction(cam, &screen, x, y);
            *px = crate::shade_render::mixed_radiance_scene(
                surfaces,
                crate::shade_render::PixelGeom {
                    i,
                    p: g.point[i],
                    n: g.normal[i],
                    v,
                    k: g.curvature.get(i).copied().unwrap_or(0.0),
                    k_estilo: g.curvature_style.get(i).copied().unwrap_or(0.0),
                    basis,
                },
                pixel_world,
                light,
                pres,
            );
        }
    });
    out
}

/// ⭐⭐⭐ **SOMA O HALO À IMAGEM JÁ PINTADA**, em bytes.
///
/// ```text
/// byte → linear de ecrã → + look(halo) → byte
/// ```
///
/// ⛔⛔ **A 1.ª redacção subtraía `look([0,0,0])` daqui, e a prova de mutação REFUTOU a premissa
/// dela.** Ela dizia que o [`ph2d_view_transform::Look`] *«tem um desvio para o preto, logo
/// `look([0,0,0])` não é necessariamente zero»* — medido, ele é `[0,0,0]` **ao bit**, e não só no
/// olhar de omissão: a `to_display` passa todo canal por uma higiene que manda `c <= 0` para `0`, e
/// as duas transformações mandam `0` em `0`. ⇒ *a subtracção era provadamente morta em todo o
/// espaço de `Look`, e nenhuma mutação a podia matar* — que é a definição de comentário com
/// sintaxe de código.
///
/// ⚠️ **A propriedade de que este passe DEPENDE não desapareceu — mudou de sítio:** ela é gateada
/// onde vive, em [`crate::brilho_tests::o_olhar_manda_o_preto_em_preto`], sobre as duas
/// transformações e uma escada de exposições. *No dia em que alguém acrescentar um olhar com
/// levantamento, aquele gate reprova e diz que este passe conta com isto.*
///
/// ⚠️ **O ALFA não se mexe**, e é a mesma lei que a luz que a peça põe no chão já segue
/// (`docs/Render3d/09`): *luz acrescentada com alfa inalterado é o que um compositor lê como luz*.
pub(crate) fn soma_halo(out: &mut [u8], halo: &[[f32; 3]], pres: &Presentation) {
    if halo.is_empty() {
        return;
    }
    // ⚠️ O olhar corre UMA vez por pixel e não uma por canal — a 1.ª redacção chamava-o dentro do
    // laço dos canais, logo `3×`, e ele é a lei inteira da apresentação. *Uma conta cara escrita
    // dentro do laço mais interno corre tantas vezes quantas esse laço.*
    let (pixeis, _) = out.as_chunks_mut::<4>();
    for (px, h) in pixeis.iter_mut().zip(halo) {
        let d = pres.look.apply(*h);
        for (canal, &acrescimo) in px.iter_mut().zip(&d) {
            if acrescimo <= 0.0 {
                continue;
            }
            let base = ph2d_color::srgb::srgb_to_linear_byte(*canal);
            *canal = ph2d_color::srgb::linear_to_srgb_byte(base + acrescimo);
        }
    }
}

#[cfg(test)]
#[path = "brilho_tests.rs"]
mod brilho_tests;
