//! ⭐⭐⭐ **O RESPIRO de um rótulo dentro de uma caixa — e a centragem que o gasta.**
//!
//! ⚠️ **Saiu do [`super`] por TETO DE LOC** (712 > 700, 2026-09-18) e o corte é de **assunto**: o
//! ficheiro-mãe responde *como uma superfície se pinta*; este responde *quanto de uma caixa é do
//! TEXTO*. As quatro funções são uma lei só, com ida ([`rect_for_label`]), volta
//! ([`label_budget`]) e os dois pintores que a gastam — e é por isso que elas viajam juntas.

use super::{Rect, TextSystem, VectorScene, text::paint_text};
use ph2d_vector::Color;

/// Center `text` inside `rect` (horizontally + vertically).
/// ⭐⭐ **O que sobra da largura de uma caixa depois do RESPIRO das duas bordas.**
///
/// Enio, 2026-09-06, com duas fotos: *«algumas palavras ou mesmo os 3 pontos ficam muito próximos
/// da borda do botão. Deveria ter algum espaço.»* — `Surface Smo…` acabava colado à moldura.
///
/// ⚠️ **Ele só MORDE quando o texto não caberia à mesma**: a centragem é simétrica, logo deflacionar
/// os dois lados não move um rótulo que cabe — muda apenas o orçamento com que ele é elidido. *Um
/// recuo que só se paga no caso apertado é o único que não custa nada no caso comum.*
///
/// **O número é do modelo:** `base_margin · 2` = 8 px, a margem de conteúdo horizontal de um botão
/// no Godot 4.6 «Modern» (`theme_modern.cpp:289`) — e é o `Spacing::Md`, que a escada desta casa
/// já nomeia *«default inline padding»*. ⛔ Não é o `Button::padding()` (12 px): aquele é o recuo
/// de um botão que dimensiona a si próprio, e aqui a largura vem de fora.
///
/// ⚠️⚠️ **PÚBLICO desde 2026-09-18, e a razão é um defeito meu:** o Audio Mixer passou a MEDIR a
/// coluna dos nomes dele e continuou a cortar, porque a coluna media o **rect** e o pintor gasta o
/// rect **menos este respiro** — `Return` pede `35,9` px, a coluna dava `35,9` e o orçamento era
/// `19,9`. *Quem dimensiona uma coluna tem de perguntar quanto dela vai ser GASTA*, e o gate que
/// a confere tem de perguntar o mesmo — senão ele mede a mesma grandeza errada que a cura.
#[must_use]
pub fn label_budget(w: f32) -> f32 {
    // ⛔⛔ **E O RESPIRO NUNCA COME MAIS DE METADE DA CAIXA** (2026-09-18). Ele era uma subtracção
    //    CONSTANTE, logo numa caixa pequena tomava-a quase toda: o botão de silenciar de um strip
    //    do mixer mede `25,0 px`, o respiro levava `16,0` e sobravam `9,0` para uma letra `M` que
    //    precisa de `10,1` ⇒ **nem a reticência cabia e o botão saía VAZIO**. *Um controlo sem
    //    legenda e um controlo morto dão o mesmo report* — e era isso que a foto do dono mostrava
    //    como `…` entre dois botões com letra.
    //
    // ⭐ **A fronteira é DERIVADA, não escolhida:** as duas leis cruzam-se exactamente em `32 px`
    //    (`w − 16 = w/2`), logo **acima de 32 nada muda** — toda caixa normal deste app continua
    //    byte a byte como estava — e abaixo o respiro passa a ser proporcional em vez de engolir a
    //    palavra. ⇒ é uma melhoria ESTRITA: o que cabia continua a caber.
    let respiro = ph2d_tokens::Spacing::Md.px() * 2.0;
    (w - respiro).max(w * 0.5).max(1.0)
}

/// ⭐⭐⭐ **O CAMINHO INVERSO: que largura de caixa é precisa para um texto de `text_w` CABER.**
///
/// ⚠️ Ela é a irmã de [`label_budget`] e existe pela razão que a tornou pública: *quem dimensiona
/// uma coluna precisa da pergunta ao contrário*, e derivá-la à mão no painel seria a segunda cópia
/// do respiro — que divergiria no dia em que o `Spacing::Md` mudasse.
///
/// ⭐ As duas são uma lei só, e há gate a provar a ida-e-volta.
#[must_use]
pub fn rect_for_label(text_w: f32) -> f32 {
    // ⚠️ A inversa das DUAS leis do [`label_budget`]: a caixa mais pequena que serve é a menor das
    //    duas soluções — `t + respiro` (o regime normal) ou `2t` (o regime pequeno).
    let respiro = ph2d_tokens::Spacing::Md.px() * 2.0;
    let w = (text_w + respiro).min(text_w * 2.0).max(1.0);
    // ⛔⛔⛔ **E ela confere-se contra a LEI, nunca contra a álgebra que a escreveu.** Medido em
    //    2026-09-19 varrendo o domínio em passos de `0,0007 px`: `(t + respiro) − respiro` fica
    //    **abaixo** de `t` em **`2,65 %`** dos casos, com um défice de até `3,05e-5 px` — e a
    //    elisão compara `<=`, logo *um défice de um ULP corta a palavra inteira*. ⚠️ O gate que
    //    guardava este par amostrava **seis** valores e passava nos seis: *uma prova por amostras
    //    sobre uma lei que falha em 2,65 % do domínio lê-se como prova*.
    if label_budget(w) < text_w {
        w.next_up()
    } else {
        w
    }
}

pub fn paint_text_centered(
    text_system: &mut TextSystem,
    scene: &mut VectorScene,
    text: &str,
    rect: Rect,
    font_size: f32,
    color: Color,
) {
    paint_text_centered_com_orcamento(
        text_system,
        scene,
        text,
        rect,
        label_budget(rect.w),
        font_size,
        color,
    );
}

/// ⭐⭐⭐ **O mesmo, com o orçamento DADO** — para quem já sabe quanta borda a caixa dele tem.
///
/// ⛔⛔ **Ela existe por um defeito medido** (2026-09-18, varredura das elisões): a caixa de número
/// reserva a **coluna do stepper** e depois pedia o orçamento de um RÓTULO sobre o que sobrou,
/// gastando `Md` de cada lado por cima disso — `56 px` de caixa entregavam **`24`** ao número, e
/// `0.500` (que mede `31,0`) saía **`0.…`**. *Um número que se lê `0.…` mente sobre si próprio.*
///
/// ⚠️ **O respiro conta as BORDAS QUE EXISTEM.** Do lado do stepper a borda é a própria coluna
/// dele (`16`–`22 px` de separação já lá está), logo o que falta contar é **uma** — e com o texto
/// CENTRADO isso deixa `Md/2 = Xs` livre de cada lado, que é exactamente o recuo que esta caixa já
/// usa na vertical para a selecção.
#[allow(clippy::too_many_arguments)]
pub fn paint_text_centered_com_orcamento(
    text_system: &mut TextSystem,
    scene: &mut VectorScene,
    text: &str,
    rect: Rect,
    orcamento: f32,
    font_size: f32,
    color: Color,
) {
    // ⚠️ **Centra o que vai ser PINTADO, não o que foi pedido.** O `paint_text` elide desde
    // 2026-09-06 (report do dono: a palavra que não cabe *«passa para baixo e some»*), e medir
    // aqui o texto INTEIRO punha um rótulo elidido fora do centro — pior, com `rect.w` como
    // orçamento de quebra o layout devolvia DUAS linhas e o `y` centrava-as, deixando a primeira
    // acima do topo da caixa. *Uma centragem que mede outra coisa do que se pinta é um deslocamento
    // com cara de arredondamento.*
    // ⭐ **O BALÃO**: a área é a CAIXA, não o texto — hoverar a caixa inteira é o que o artista
    //    faz, e o texto cortado vive dentro dela. Ver [`crate::text_elide::balao`].
    let shown = crate::text_elide::balao::na_area(rect, || {
        crate::text_elide::fit(text_system, text, font_size, orcamento)
    });
    let layout = text_system.layout(&shown, font_size, f32::INFINITY);
    let text_w = layout.width();
    let text_h = layout.height();
    let x = rect.x + (rect.w - text_w) / 2.0;
    let y = rect.y + (rect.h - text_h) / 2.0;
    paint_text(text_system, scene, &shown, x, y, font_size, rect.w, color);
}

// -----------------------------------------------------------------------
// Layout — 4 zones backdrop
