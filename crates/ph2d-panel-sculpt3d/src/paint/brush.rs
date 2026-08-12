//! **O QUE A SEÇÃO DO PINCEL DESENHA ALÉM DOS KNOBS** — irmão do `body.rs` e do
//! `mask_tools.rs`, cortado por ASSUNTO.
//!
//! As rows contínuas saem da tabela e o `body.rs` as percorre genericamente;
//! aqui mora o que é do PINCEL e de mais ninguém — com que profundidade a seção
//! se mostra, a curva do peso, a família do padrão, o preview e o acumular.
//!
//! ⚠️ **A CABEÇA e a CAUDA da mesma seção viajam juntas**, e não é arrumação: as
//! duas são a moldura em que as rows genéricas caem, e separá-las poria *o que
//! vem antes* e *o que vem depois* em arquivos diferentes — a próxima pessoa a
//! mexer na ordem teria de descobrir isso.

use ph2d_editor_core::ids;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_i18n::tr;
use ph2d_sculpt3d::{Alpha, Falloff};
use ph2d_tokens::Spacing;

use super::body::paint_one_row;
use super::mask_tools::paint_mask_tools;
use super::widgets::{command, labelled_seg, toggle};

use crate::preview;
use crate::rows;
use crate::state::{Sculpt3dSnapshot, UiLevel};

/// **COM QUE PROFUNDIDADE OLHAR** — o par `Basic` · `Pro`, no TOPO da seção do
/// pincel (§2.3 do plano).
///
/// ⚠️ **Ele governa a seção do PINCEL e nada mais.** Sombreamento e topologia
/// descrevem *como a forma é lida* e *quão fino é o barro* — nenhum dos dois é
/// um knob que o verbo armou —, então um interruptor que os alcançasse esconderia
/// controles que ninguém escolheu por você, que é exatamente a linha que separa
/// divulgação progressiva de amputação.
///
/// ⚠️ **E ele fica DENTRO do cabeçalho dobrável**, não acima: quem fecha a seção
/// do pincel fechou o assunto inteiro, e um chip órfão pairando sobre uma seção
/// fechada seria um controle sem sujeito.
pub(super) fn paint_level_row(
    ctx: &mut PaintCtx,
    snap: &Sculpt3dSnapshot,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let selected = UiLevel::ALL
        .iter()
        .position(|&l| l == snap.ui.ui_level)
        .unwrap_or(0);
    let labels: Vec<&str> = UiLevel::ALL.iter().map(|l| l.label()).collect();
    labelled_seg(
        ctx,
        tr("panel.sculpt3d.ui_level"),
        ids::SCULPT3D_SEC_BRUSH,
        &ids::SCULPT3D_UI_LEVEL,
        &labels,
        selected,
        x,
        w,
        y,
    )
}

/// O que vem DEPOIS dos knobs do pincel: a curva e as operações de máscara.
pub(super) fn paint_brush_tail(
    ctx: &mut PaintCtx,
    snap: &Sculpt3dSnapshot,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    // O falloff logo abaixo dos knobs: ele é a FORMA do peso, e a força é
    // quanto dele se aplica.
    //
    // ⚠️ **PRO, e é o exemplo que dá nome à regra:** a curva é escolhida pelo
    // `arm_verb_defaults` a cada troca de ferramenta (`VerbProfile::falloff`),
    // então em Basic o artista está com a curva que a REFERÊNCIA escolheu para
    // aquele verbo — não com um vazio. Em Pro ele ganha ACESSO ao número, não um
    // número novo.
    let y = if snap.ui.ui_level.shows(UiLevel::Pro) {
        let selected = Falloff::ALL
            .iter()
            .position(|&f| f == snap.ui.brush.falloff)
            .unwrap_or(0);
        let labels: Vec<&str> = Falloff::ALL.iter().map(|f| f.label()).collect();
        labelled_seg(
            ctx,
            tr("panel.sculpt3d.falloff"),
            ids::SCULPT3D_SEC_BRUSH,
            &ids::SCULPT3D_FALLOFF,
            &labels,
            selected,
            x,
            w,
            y,
        )
    } else {
        y
    };
    // **O PADRÃO**, logo abaixo do falloff — os dois moldam o MESMO peso: o
    // falloff diz como ele cai do centro à borda, o alpha diz onde ele age
    // dentro disso. A primeira opção é NENHUM, e o deslocamento de um é a mesma
    // aritmética que o seletor de matcap usa (o `event` a desfaz com
    // `checked_sub`; as duas metades vivem uma ao lado da outra de propósito).
    let mut labels: Vec<&str> = vec![tr("panel.sculpt3d.alpha.none")];
    labels.extend(Alpha::ALL.iter().map(|a| a.label()));
    // ⚠️ **O SLOT DE IMAGEM tem o nome do SPRITE**, e ele é o ÚLTIMO chip.
    //
    // ⚠️ **Isto REVOGA a decisão que estava escrita aqui.** A versão anterior
    // dizia que uma imagem nunca está na `ALL`, que o `position` devolve `None`
    // para ela e que cair em *nenhum* era honesto — *"o chip aceso não mente
    // sobre um padrão que aquela fileira não oferece"*. A premissa era que a
    // fileira não o oferecia; agora ela oferece, e o que sobrava era um painel
    // dizendo **None** com um padrão vivo e um preview desenhado logo abaixo:
    // o artista lia o painel como quebrado antes de olhar para a miniatura.
    //
    // ⚠️ **O nome vem do RETRATO, não do `Alpha`.** O motor guarda os pixels; de
    // onde eles vieram é proveniência da CENA. Um `label()` que devolvesse o
    // nome do sprite obrigaria o enum a carregar uma `String` que kernel nenhum
    // lê.
    if let Some(name) = snap.alpha_image_name.as_deref() {
        labels.push(name);
    }
    let selected = crate::state::alpha_chip_index(snap);
    let mut y = labelled_seg(
        ctx,
        tr("panel.sculpt3d.alpha"),
        ids::SCULPT3D_SEC_BRUSH,
        &ids::SCULPT3D_ALPHA,
        &labels,
        selected,
        x,
        w,
        y,
    );
    // **O ALPHA POR IMAGEM**, logo abaixo da fileira de nomes — e ele é um
    // BOTÃO, não um décimo chip. A fileira lista NOMES (as nove fórmulas); uma
    // imagem não é um nome, é uma coisa para a qual se aponta. Um chip "Image"
    // teria de existir antes de haver pixels, e é justamente esse estado que o
    // `Alpha::Image` torna inexprimível ao carregar a imagem dentro de si.
    //
    // ⚠️ **Sem sprite selecionado ele NÃO é pintado**, e não é dimming: um botão
    // que só pode falhar é como o artista aprende que ele não funciona. É a
    // mesma decisão do "Bake to Sprite" logo acima, que mostra uma dica no lugar.
    if snap.has_bake_target {
        y = command(
            ctx,
            ids::SCULPT3D_ALPHA_SPRITE,
            tr("panel.sculpt3d.alpha_sprite"),
            x,
            w,
            y,
        );
    }
    // ⚠️ **A pista de escala vem AQUI, colada nos chips que a governam** — e não
    // no bloco de knobs acima, que é onde ela nasceu e onde o smoke a perdeu:
    // lá ela aparecia do nada, separada do seletor pela fileira do Falloff, e o
    // artista lia um número sem saber de que ele era. A row continua na tabela
    // (ver `Row::place`); o que mudou é onde ela é desenhada.
    for row in rows::rows().filter(|r| r.place == rows::Place::AfterAlpha && r.visible(&snap.ui)) {
        y = paint_one_row(ctx, snap, row, x, w, y);
    }
    // **O PREVIEW**, logo ABAIXO das pistas que o mudam — e a posição é a mesma
    // decisão que moveu a pista de escala para cá: um controle e o que ele
    // governa têm de estar no campo de visão um do outro, senão o artista arrasta
    // um número olhando para outro lugar.
    // **O interruptor do preview NO BARRO**, entre as pistas e o quadro — os
    // dois mostram o mesmo padrão e a caixa governa o de FORA, então ela fica
    // onde o olho já está. ⚠️ Só com padrão armado, pela mesma razão da pista de
    // escala: sem padrão ele é um interruptor de coisa nenhuma.
    if snap.ui.brush.alpha.is_some() {
        y = toggle(
            ctx,
            ids::SCULPT3D_ALPHA_PREVIEW,
            tr("panel.sculpt3d.alpha_preview"),
            snap.ui.alpha_preview,
            x,
            w,
            y,
        );
    }
    y = preview::paint(ctx, snap, x, w, y);
    // **ACUMULAR**, e só onde ele faz alguma coisa. ⚠️ A pergunta é feita à
    // PORTA do motor (`Verb::accumulates`) e não a uma lista de nomes aqui: o
    // aplicador pergunta à mesma para honrar o clique, e duas cópias divergiriam
    // num interruptor que aparece e não muda nada — quem tem âncora carrega o
    // gesto TOTAL desde o pen-down, e somar totais não significa nada.
    let y = if snap.ui.brush.verb.accumulates() {
        toggle(
            ctx,
            ids::SCULPT3D_ACCUMULATE,
            tr("panel.sculpt3d.accumulate"),
            snap.ui.brush.accumulate,
            x,
            w,
            y,
        ) + Spacing::Sm.px()
    } else {
        y
    };
    paint_mask_tools(ctx, snap, x, w, y)
}
