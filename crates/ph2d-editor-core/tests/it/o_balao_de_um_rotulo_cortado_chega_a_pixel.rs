//! ⭐⭐⭐ **O BALÃO DE UMA PALAVRA CORTADA CHEGA A PIXEL** — a metade que nenhuma contagem prova.
//!
//! ⛔⛔ **O censo da varredura (`toda_palavra_cortada_tem_balao`) afirma que o app SABE o texto
//! inteiro e onde ele está.** Isso é necessário e não é suficiente: um pintor que lesse o
//! [`ph2d_editor_core::text_elide::balao::sob`] e não desenhasse nada deixaria aquele gate verde
//! sobre uma tela onde o artista continua sem conseguir ler a palavra. *Faixa reservada não é
//! faixa pintada*, e este repo já pagou essa lição num gate que prometia medir «chega a PIXEL» e
//! media a linha reservada.
//!
//! ⚠️⚠️ **A régua é o número de GLIFOS**, e não caminhos: o Vello encaminha texto por
//! `draw_glyphs`, cuja saída não entra na contagem de caminhos — uma régua de caminhos leria o
//! mesmo número com e sem letras.
//!
//! # As três metades
//!
//! 1. **Com o rato em cima de um rótulo cortado, o balão pinta letras.**
//! 2. **O texto do balão é o INTEIRO** — um nome mais longo dá mais glifos. Sem esta metade, um
//!    balão que mostrasse o texto JÁ CORTADO (que é o que está na tela, logo inútil) passava a
//!    primeira.
//! 3. **O CONTROLO: com o rato longe, o balão não pinta nada.** Sem ela, um balão que aparecesse
//!    sempre — sobre rótulos legíveis, no meio do ecrã — passava as duas de cima.

use ph2d_editor_core::screens::hero::topbar;
use ph2d_editor_core::text_elide::balao;
use ph2d_editor_core::widget::paint_property_label;
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::{Color, VectorScene};

/// A coluna do nome no degrau estreito do dock — o sítio onde o dono trabalha.
const COLUNA: Rect = Rect {
    x: 20.0,
    y: 100.0,
    w: 78.0,
    h: 12.0,
};
const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1366.0,
    h: 768.0,
};
const FONTE: f32 = 12.0;

/// Pinta `texto` na coluna estreita com o rato em `ponteiro`, e devolve os glifos que o BALÃO
/// deixou numa cena própria.
///
/// ⚠️ **Duas cenas de propósito:** a do rótulo e a do balão. Contar as duas juntas mediria
/// também as letras do rótulo, e a pergunta é sobre o balão.
fn glifos_do_balao(texto: &str, ponteiro: Option<(f32, f32)>) -> usize {
    let mut text = TextSystem::without_system_fonts();
    balao::onde_esta_o_rato(ponteiro);
    balao::novo_quadro();
    let mut do_rotulo = VectorScene::new();
    paint_property_label(
        &mut text,
        &mut do_rotulo,
        texto,
        COLUNA.x,
        COLUNA.y,
        FONTE,
        COLUNA.w,
        Color::from_rgba8(255, 255, 255, 255),
    );
    let mut do_balao = VectorScene::new();
    topbar::paint_elision_balloon(&mut do_balao, &mut text, Theme::Dark, VIEWPORT);
    do_balao.inner().encoding().resources.glyphs.len()
}

/// ⭐ **(1) Com o rato em cima, a palavra cortada volta a ser legível.**
#[test]
fn com_o_rato_em_cima_o_balao_pinta_letras() {
    let dentro = (COLUNA.x + COLUNA.w * 0.5, COLUNA.y + COLUNA.h * 0.5);
    let n = glifos_do_balao("Non-Spatialized Radius", Some(dentro));
    assert!(
        n > 0,
        "o balao de um rotulo cortado nao pintou um unico glifo — o artista continua sem o poder ler"
    );
}

/// ⭐⭐ **(2) O que ele mostra é o texto INTEIRO, não o que já está na tela.**
///
/// ⚠️ A régua é COMPARATIVA de propósito: um número absoluto de glifos depende da fonte e da
/// caixa, e o que se quer afirmar é *«mais texto ⇒ mais letras no balão»*.
#[test]
fn o_balao_mostra_o_texto_inteiro_e_nao_o_cortado() {
    let dentro = (COLUNA.x + COLUNA.w * 0.5, COLUNA.y + COLUNA.h * 0.5);
    let curto = glifos_do_balao("Crouch Height", Some(dentro));
    let longo = glifos_do_balao("Non-Spatialized Radius of the Audio Source", Some(dentro));
    assert!(
        longo > curto,
        "o balao pintou {longo} glifos para o nome longo e {curto} para o curto — ele esta a \
         mostrar o texto JA CORTADO, que e o que o artista ja tem na tela"
    );
}

/// ⭐⭐⭐ **(3) O CONTROLO: com o rato longe, ele cala-se.**
#[test]
fn com_o_rato_longe_o_balao_nao_pinta_nada() {
    let n = glifos_do_balao("Non-Spatialized Radius", Some((900.0, 700.0)));
    assert_eq!(
        n, 0,
        "o balao apareceu com o rato noutro sitio — ele passaria a tapar a tela sobre rotulos \
         que se leem perfeitamente"
    );
}

/// ⚠️ **E sem rato nenhum também** — antes do primeiro movimento o app não tem onde o pôr.
#[test]
fn sem_ponteiro_nao_ha_balao() {
    assert_eq!(glifos_do_balao("Non-Spatialized Radius", None), 0);
}

/// ⛔ **Um rótulo que CABE não ganha balão** — a outra ponta do controlo, medida pelo produto.
#[test]
fn um_rotulo_que_cabe_nao_ganha_balao() {
    let dentro = (COLUNA.x + COLUNA.w * 0.5, COLUNA.y + COLUNA.h * 0.5);
    assert_eq!(
        glifos_do_balao("Mass", Some(dentro)),
        0,
        "um nome que cabe na coluna nao tem nada a mostrar"
    );
}

/// ⭐⭐⭐ **A ORDEM NO QUADRO É LOAD-BEARING, e nenhuma das cinco de cima a vê.**
///
/// ⛔⛔ O balão é uma lista que os pintores DESTE quadro enchem e que um pintor do FIM lê. Duas
/// coisas têm de ser verdade no `paint_hero_screen` e nenhum teste de unidade as pode afirmar,
/// porque nenhum deles corre o quadro:
///
/// 1. **`novo_quadro` corre ANTES do primeiro pintor.** Depois dele, metade dos rótulos deste
///    quadro seriam esvaziados da lista **já depois de terem sido medidos** — e o balão mostraria
///    só o que o resto do quadro pintou, ou nada.
/// 2. **O balão corre DEPOIS da dica de widget**, senão as duas pintam no mesmo sítio.
///
/// ⚠️ **A régua é o TEXTO do ficheiro** (`include_str!`), e isso é deliberado: um `mod` que
/// mudasse de sítio deixa de compilar, e a ORDEM de duas chamadas dentro de uma função de 700
/// linhas não é observável de mais nenhuma maneira sem um device.
#[test]
fn o_quadro_esvazia_o_balao_antes_de_pintar_e_le_o_no_fim() {
    let fonte = include_str!("../../src/screens/hero/paint.rs");
    let novo = fonte
        .find("balao::novo_quadro(")
        .expect("o quadro tem de esvaziar o balao — sem isso ele mostra o de ha dez segundos");
    let primeiro_pintor = fonte
        .find("paint_canvas_bg(")
        .expect("o primeiro pintor do quadro mudou de nome — reancore esta regua");
    assert!(
        novo < primeiro_pintor,
        "`balao::novo_quadro` corre DEPOIS do primeiro pintor: os rotulos medidos antes dele \
         seriam apagados da lista no mesmo quadro"
    );
    // ⚠️ **A precedencia mudou-se para junto dos dois pintores** (tecto de LOC, 19/09), logo e
    //    LA que ela se le — e o quadro tem de chamar a porta que a carrega.
    assert!(
        fonte.contains("paint_hover_overlays("),
        "o quadro nao pinta as bolhas — o app sabe o texto inteiro e nao o mostra"
    );
    let porta = include_str!("../../src/screens/hero/topbar/mod.rs");
    let dica = porta
        .find("if !paint_hover_tooltip(")
        .expect("a dica de widget deixou de decidir primeiro");
    let bolha = porta
        .find("paint_elision_balloon(scene")
        .expect("a porta das bolhas nao chama o balao");
    assert!(
        dica < bolha,
        "o balao pinta ANTES da dica de widget: as duas usam a mesma geometria e ficariam uma \
         por cima da outra"
    );
}
