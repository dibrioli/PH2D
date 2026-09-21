//! ⭐⭐⭐ **O SPRITE SELECIONADO VIRA O PADRÃO DO PINCEL** — a lei do *alpha por imagem*, na crate
//! da família.
//!
//! ⛔⛔ **Ela vivia na shell, e o que a trouxe para cá foi uma CATRACA** (`the_shell_only_shrinks`,
//! 21/09). ⭐ O molde é o que o [`super::bake::drain`] já usava: *as duas fontes de pixels chegam
//! por ASSINATURA, como fechos*, porque quem sabe **ler** um sprite é a shell (o `PainterTool`, o
//! `read_sprite_source`) e quem sabe **o que fazer** com os pixels é esta família.
//!
//! ⚠️ **A ORDEM das duas fontes é a lei, e não uma preferência:** primeiro o que o artista **VÊ**
//! (as camadas vivas do Painter), depois o que o sprite **GUARDA**. Um sprite cuja aparência vem
//! do sistema de camadas ainda aponta para a imagem de origem, e ler a origem devolve outra
//! textura — *o padrão sairia diferente do que está na tela* (Enio, 2026-08-09: *«veja a textura
//! ao lado e veja a textura no preview»*).
//!
//! ⚠️ **E a 2.ª fonte é PREGUIÇOSA** (um fecho, não um valor): ler os pixels de um sprite
//! `Individual` é um `readback` — uma volta completa à placa —, e com camadas vivas ela nunca
//! chega a ser perguntada.

use crate::bake::Veredito;

/// ⭐⭐⭐ **QUAL DAS DUAS FONTES DÁ O PADRÃO** — a decisão, **pura**.
///
/// ⛔ **Ela é uma função própria e não duas linhas dentro do [`drain`], e a razão é a lei desta
/// casa:** *quando um gate precisa de um device para medir uma decisão que não tem pixel nenhum, a
/// lei está no sítio errado*. Escrita lá dentro, a ORDEM das duas leituras só era afirmável com um
/// `Sculpt3dScene`, que pede um `wgpu::Device` — o gate nasceria `#[ignore]` e **o CI nunca o
/// correria**.
///
/// ⚠️ **O `or_else` é a lei e não um idioma:** com um `or(` as duas fontes seriam lidas sempre, e
/// ler um sprite `Individual` é um `readback` — uma volta completa à placa por gesto.
pub(crate) fn escolhe_a_fonte(
    ler_camadas: &mut dyn FnMut() -> Option<ph2d_sculpt3d::AlphaImage>,
    ler_sprite: &mut dyn FnMut() -> Option<ph2d_sculpt3d::AlphaImage>,
) -> Option<(ph2d_sculpt3d::AlphaImage, &'static str)> {
    ler_camadas()
        .map(|a| (a, ph2d_i18n::tr("app.sculpt3d.alpha.as_camadas")))
        .or_else(|| ler_sprite().map(|a| (a, ph2d_i18n::tr("app.sculpt3d.alpha.a_imagem"))))
}

/// **O gesto do padrão** — a decisão, com as duas fontes por assinatura.
///
/// `ler_camadas` devolve o que a tela MOSTRA (o composite do Painter, já em luminância);
/// `ler_sprite` devolve o que o sprite GUARDA. Os dois devolvem `None` quando não há nada ali.
///
/// ⚠️ **O `Veredito` é o mesmo do bake, e de propósito:** as duas recusas deste gesto saíam com o
/// ✓ verde de sucesso até 21/09 — o mesmo defeito, no mesmo ficheiro, que a foto do dono expôs no
/// bake. *Um tipo partilhado é o que impede a terceira ocorrência.*
pub fn drain(
    cena: &mut crate::Sculpt3dScene,
    nome_do_objecto: Option<std::sync::Arc<str>>,
    ha_seleccao: bool,
    ler_camadas: &mut dyn FnMut() -> Option<ph2d_sculpt3d::AlphaImage>,
    ler_sprite: &mut dyn FnMut() -> Option<ph2d_sculpt3d::AlphaImage>,
) -> Veredito {
    match escolhe_a_fonte(ler_camadas, ler_sprite) {
        Some((a, what)) => {
            // ⚠️ **O nome vem do `Name` do objecto, com o fallback a dizer o que ele É.** Um
            // sprite pode não ter nome, e um chip em branco seria o mesmo defeito do «None» que
            // esta wave conserta, com outra roupa.
            let from = nome_do_objecto.unwrap_or_else(|| std::sync::Arc::from("Sprite"));
            let scale = cena.set_alpha_image(a, from);
            // ⚠️ **O readout diz a ESCALA, não os pixels.** A resolução da fonte tem efeito ZERO
            // sobre o tamanho do padrão no modelo (o `AlphaImage::sample` mapeia em unidades de
            // LADRILHO: a mesma imagem a 64² e a 4096² dá 80 transições ao longo das mesmas 2
            // unidades de objecto). O número que governa o que o artista vê é o `Alpha Scale`, e
            // era justamente ele que mudava sem aparecer em lugar nenhum.
            Veredito::Assado(ph2d_i18n::tr_with(
                "app.sculpt3d.alpha.padrao_definido",
                &[("what", &what), ("scale", &format!("{scale:.3}"))],
            ))
        }
        None if ha_seleccao => Veredito::Recusado(
            ph2d_i18n::tr("app.sculpt3d.alpha.nao_descreve_uma_imagem").to_string(),
        ),
        None => {
            Veredito::Recusado(ph2d_i18n::tr("app.sculpt3d.alpha.selecione_um_sprite").to_string())
        }
    }
}
