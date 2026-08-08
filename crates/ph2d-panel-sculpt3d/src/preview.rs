//! **O PREVIEW DO PADRÃO** — ver o alpha antes de encostar no barro.
//!
//! Pedido do Enio no smoke da W11: *"não seria interessante um preview ao mover
//! os sliders das alphas? Desse modo o usuário veria como está antes de
//! esculpir"*.
//!
//! # Por que ele mede UNIDADES DE OBJETO, e por que essa é a wave inteira
//!
//! Um swatch em unidades arbitrárias mostraria sempre a mesma imagem: bastaria
//! ele abranger `N × escala` para o padrão ter sempre `N` features, e arrastar a
//! pista de escala **não mudaria nada na tela**. Ele responderia *"que padrão é
//! este?"* — que os nove nomes já respondem — e ficaria mudo sobre a única
//! pergunta que o artista não consegue responder sozinho: ***este tamanho está
//! certo para o MEU modelo?***
//!
//! ⚠️ **E é exatamente essa pergunta que já reprovou um smoke deste módulo**
//! (*"os poros são gigantescos"*, 2026-08-05): uma escala é absoluta, mas QUAL
//! número serve depende do tamanho e da densidade da peça. Então o swatch abrange
//! um pedaço do MODELO — [`SPAN_FRACTION`] do maior lado dele —, e a densidade
//! que ele desenha é a densidade que o pincel vai depositar.
//!
//! # O plano, e o limite dele DITO
//!
//! A fatia é o plano **XY do objeto**, que é o plano que a câmera de repouso
//! mostra (a vista olha por `−Z`). ⚠️ **Com o eixo inclinado para fora de XY o
//! swatch mostra a FATIA, não a face que a peça exibe** — o padrão é um campo 3-D
//! e um corte oblíquo dele estica, como estica no barro.
//!
//! ⚠️ **A câmera VIVA foi descartada de propósito**, e o motivo é o cache: a
//! chave incluiria a base da vista, então **toda órbita re-renderizaria** um
//! swatch que ninguém está olhando enquanto orbita — e o número abaixo diz o
//! preço disso.
//!
//! # O CACHE não é otimização, é requisito
//!
//! Medido (`weight_at` sobre a grade inteira, release):
//!
//! | grade | Noise | Strata | Scales | Scratches | Weave |
//! |---|---|---|---|---|---|
//! | 48² | 0,201 ms | 0,208 | **0,358** | 0,020 | 0,026 |
//! | 96² | 0,814 | 0,834 | **1,438** | 0,081 | 0,101 |
//!
//! ⚠️ **A 96² o pior padrão custa 1,44 ms — 8,6% de um quadro de 60 fps, todo
//! quadro.** E a hora em que o cache MENOS ajuda é justamente a que importa:
//! arrastando a pista, a chave muda a cada quadro e ele nunca acerta. Por isso a
//! grade é [`SWATCH`] e não maior — a 64² o pior caso fica em ~0,64 ms, e a
//! família DIRECIONAL, que é a razão de este preview existir, custa 0,02-0,10.
//!
//! ⚠️ **O Scales é o pior por MECANISMO, não por acaso:** o Worley visita as 27
//! células vizinhas por amostra.
//!
//! ⚠️ **E a chave carrega TODA entrada que muda a imagem.** O modo de falha de
//! esquecer uma não é um erro — é um preview VELHO que ninguém vê que é velho, a
//! mesma frase que o cache dos planos da luz do impasto já pagou.

use ph2d_editor_core::paint::resolve;
use ph2d_sculpt3d::{Alpha, Brush};
use ph2d_tokens::{ColorToken, Theme};
use std::cell::RefCell;

use crate::state::Sculpt3dUi;
use ph2d_i18n::tr;

/// O lado da grade que o swatch renderiza. Ver o cabeçalho — **é um número de
/// CUSTO, medido, não uma escolha de nitidez.**
pub(crate) const SWATCH: usize = 64;

/// Que fração do maior lado do modelo o swatch abrange.
///
/// ⚠️ **Escolhido pela ARITMÉTICA da recomendação, não pelo gosto:** a
/// `recommended_scale` põe ~33 features atravessando o modelo, então um oitavo
/// dele mostra ~4 features — grande o bastante para a forma de cada uma ser
/// legível (16 px por feature na grade de 64) e pequeno o bastante para a
/// densidade ser contável de relance. Um swatch do modelo INTEIRO teria 33
/// features em 64 px: dois pixels cada, que é o aliasing que a lei das dez
/// arestas existe para nomear.
pub(crate) const SPAN_FRACTION: f32 = 0.125;

/// Tudo o que muda a imagem. Ver o cabeçalho — esquecer um campo aqui é um
/// preview velho que ninguém vê que é velho.
#[derive(Clone, Copy, PartialEq)]
struct Key {
    alpha: Alpha,
    scale: f32,
    az: u16,
    elev: u16,
    span: f32,
    /// ⚠️ **O TEMA entra na chave, e a alternativa não existia.** A primeira
    /// versão deste módulo dizia que a rampa saía em cinza e *"quem a tinge é o
    /// `paint`"* — e o `draw_image_rgba` só BLITA: não há estágio de tint depois
    /// dele. Ou a cor final sai daqui, ou o swatch é cinza sobre um painel claro.
    /// O preço é re-renderizar ao trocar de tema, que acontece uma vez e custa os
    /// mesmos 0,64 ms de um passo de slider.
    theme: Theme,
}

thread_local! {
    static CACHE: RefCell<Option<(Key, std::sync::Arc<Vec<u8>>)>> = const { RefCell::new(None) };
}

/// **A imagem do padrão**, entregue a quem a desenha.
///
/// ⚠️ **Um `with` e não um getter**, porque o buffer é do CACHE: devolvê-lo por
/// referência exigiria vazar o empréstimo do `thread_local`, e devolvê-lo por
/// valor seria uma cópia de 16 KB por quadro — que é o oposto do que o cache
/// existe para fazer.
///
/// `None` quando não há padrão armado: um preview de coisa nenhuma é uma moldura
/// vazia, e o painel simplesmente não a pinta.
pub(crate) fn with_swatch<R>(
    ui: &Sculpt3dUi,
    model_span: f32,
    theme: Theme,
    f: impl FnOnce(&std::sync::Arc<Vec<u8>>) -> R,
) -> Option<R> {
    let alpha = ui.brush.alpha?;
    let span = span_of(model_span);
    let key = Key {
        alpha,
        scale: ui.brush.alpha_scale,
        az: ui.brush.alpha_az_deg,
        elev: ui.brush.alpha_elev_deg,
        span,
        theme,
    };
    Some(CACHE.with(|c| {
        let mut slot = c.borrow_mut();
        if slot.as_ref().is_none_or(|(k, _)| *k != key) {
            // ⚠️ **O buffer é REUSADO quando ninguém mais o segura** — o
            // `draw_image_rgba` clona o `Arc` para dentro da cena do frame, então
            // o `try_unwrap` só devolve o `Vec` depois de a cena anterior ter
            // sido descartada. Quando ele recusa, alocamos um: a alternativa
            // (escrever por baixo de uma cena viva) seria mudar pixels que já
            // foram entregues ao desenho.
            let reuse = slot
                .take()
                .and_then(|(_, a)| std::sync::Arc::try_unwrap(a).ok())
                .unwrap_or_default();
            let mut px = reuse;
            render(key, &mut px);
            *slot = Some((key, std::sync::Arc::new(px)));
        }
        let (_, px) = slot.as_ref().expect("acabou de ser preenchido");
        f(px)
    }))
}

/// O pedaço do modelo que o swatch abrange, em unidades de OBJETO.
///
/// ⚠️ **O piso existe porque um modelo pode ter lado zero** (uma peça vazia, uma
/// malha recém-aberta), e um span zero faria toda amostra cair no mesmo ponto —
/// o swatch sairia de uma cor só, que é indistinguível de *"o padrão não
/// funciona"*.
pub(crate) fn span_of(model_span: f32) -> f32 {
    let s = model_span * SPAN_FRACTION;
    if s.is_finite() && s > 0.0 {
        s
    } else {
        SPAN_FRACTION
    }
}

/// Preenche `out` com RGBA de [`SWATCH`]².
///
/// ⚠️ **A cor sai de dois TOKENS** (HR-15: zero hex no código de UI). O peso `0`
/// é o fundo do painel e o `1` é o texto — os dois extremos que o tema já
/// garante contrastados entre si, em qualquer variante clara ou escura. Um par
/// escolhido à mão sairia ilegível no tema oposto.
fn render(key: Key, out: &mut Vec<u8>) {
    // ⚠️ **O frame vem da PORTA do pincel** (`Brush::alpha_frame`), e não de um
    // construtor próprio: ele é `pub(crate)` no motor justamente para que o
    // preview não possa desenhar um eixo que o dab não usa. *Duas respostas para
    // "em que direção este padrão corre?" divergiriam, e a que mente é a que o
    // artista está olhando.*
    let frame = Brush {
        alpha_az_deg: key.az,
        alpha_elev_deg: key.elev,
        ..Brush::default()
    }
    .alpha_frame();

    let lo = rgba(ColorToken::PanelBg, key.theme);
    let hi = rgba(ColorToken::Text1, key.theme);

    out.clear();
    out.reserve(SWATCH * SWATCH * 4);
    let half = key.span * 0.5;
    let step = key.span / SWATCH as f32;
    for row in 0..SWATCH {
        for col in 0..SWATCH {
            // ⚠️ **A linha 0 é o TOPO** (é onde o `draw_image_rgba` a coloca),
            // então o `y` de objeto DESCE com a linha — sem esta inversão o
            // swatch mostraria o padrão espelhado na vertical, e um artista que
            // inclinasse o eixo veria a inclinação ao contrário da que o barro
            // recebe.
            let px = -half + step * (col as f32 + 0.5);
            let py = half - step * (row as f32 + 0.5);
            let w = key.alpha.weight_at([px, py, 0.0], key.scale, &frame);
            let t = w.clamp(0.0, 1.0);
            out.extend_from_slice(&shade(t, lo, hi));
        }
    }
}

/// Um peso → o RGBA daquele texel, entre os dois extremos do tema.
fn shade(t: f32, lo: [u8; 4], hi: [u8; 4]) -> [u8; 4] {
    let mut out = [255u8; 4];
    for c in 0..4 {
        // O `round` e não o truncamento: `t = 1` tem de pousar EXATAMENTE em
        // `hi`, e `254,999 as u8` é 254 — o mesmo degrau que a conversão de graus
        // deste painel já pagou, pego por um gate.
        let v = f32::from(lo[c]) + (f32::from(hi[c]) - f32::from(lo[c])) * t;
        out[c] = v.round().clamp(0.0, 255.0) as u8;
    }
    out
}

/// O RGBA de oito bits de um token, no tema dado.
///
/// ⚠️ **A conversão mora aqui e não numa assinatura que nomeie o `Color`**, para
/// esta crate não precisar de uma aresta ao `peniko` só para escrever um tipo de
/// parâmetro. O painel fala TOKENS; o tipo do renderer é detalhe de quem resolve.
fn rgba(token: ColorToken, theme: Theme) -> [u8; 4] {
    resolve(token, theme).to_rgba8().to_u8_array()
}

/// **DESENHA o preview**, se houver padrão armado.
///
/// ⚠️ **Ele mora aqui e não no `paint/body.rs`**, e é o mesmo corte que separa o
/// `rows.rs` do pintor: quem sabe *o que a imagem é* é este módulo, e um pintor
/// que a montasse precisaria conhecer o span, o cache e a chave — três coisas que
/// ele não tem por que saber.
pub(crate) fn paint(
    ctx: &mut ph2d_editor_core::panel::PaintCtx,
    snap: &crate::state::Sculpt3dSnapshot,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    use ph2d_editor_core::paint::{fill_rounded_rect, stroke_rounded_rect};
    use ph2d_editor_core::zones::Rect;
    use ph2d_tokens::Spacing;
    use ph2d_vector::ImageQuality;

    let theme = ctx.host.theme();
    // ⚠️ **QUADRADO, e o lado sai da LARGURA disponível** — um preview de padrão
    // que não fosse quadrado mostraria mais features num eixo que no outro, e o
    // artista leria o recorte como anisotropia do padrão. Justamente com uma
    // família DIRECIONAL na tela, essa confusão seria a pior possível.
    let side = w.min(PREVIEW_MAX_PX);
    let rect = Rect {
        x: x + (w - side) * 0.5,
        y,
        w: side,
        h: side,
    };
    let Some(()) = with_swatch(&snap.ui, snap.model_span, theme, |px| {
        fill_rounded_rect(
            ctx.scene,
            rect,
            PREVIEW_RADIUS,
            resolve(ColorToken::PanelBg, theme),
        );
        ctx.scene.draw_image_rgba(
            px,
            SWATCH as u32,
            SWATCH as u32,
            (
                f64::from(rect.x),
                f64::from(rect.y),
                f64::from(rect.x + rect.w),
                f64::from(rect.y + rect.h),
            ),
            // ⚠️ **`Medium` (bilinear) e NÃO `nearest`**, e a diferença é o que o
            // oráculo do artista é. O espectrograma do áudio escolheu nearest com
            // a razão escrita — *"uma célula é uma MEDIÇÃO"* —, e aqui é o
            // oposto: uma célula é uma AMOSTRA de um campo contínuo, e o barro
            // vai amostrá-lo nos vértices, não nesta grade. Nearest mostraria a
            // grade do preview, que é a única coisa na tela que o pincel não vai
            // desenhar.
            ImageQuality::Medium,
        );
        stroke_rounded_rect(
            ctx.scene,
            rect,
            PREVIEW_RADIUS,
            PREVIEW_BORDER_PX,
            resolve(ColorToken::Border, theme),
        );
    }) else {
        return y;
    };
    let y = rect.y + rect.h + Spacing::Sm.px();
    // ⚠️ **O AVISO, e ele é a metade que impede o preview de MENTIR.** O swatch
    // desenha o campo CONTÍNUO; o barro o amostra nos VÉRTICES. Abaixo da escala
    // que a malha resolve, o artista veria um padrão lindo e depositaria
    // chuvisco — e a lei das dez arestas é literalmente o que já reprovou um
    // smoke deste módulo. A condição é DITA, no molde do `ao_stale`, e a linha só
    // existe quando há o que avisar: um aviso permanente vira moldura.
    if snap.ui.brush.alpha_scale < snap.alpha_seed {
        return crate::paint::readout_at(ctx, tr("panel.sculpt3d.alpha.too_fine"), x, w, y);
    }
    y
}

/// O maior lado que o preview ocupa.
///
/// ⚠️ **Um TETO e não um tamanho:** o painel pode ficar largo, e um swatch que
/// crescesse com ele passaria de metade da altura visível — o artista rolaria
/// para achar a pista que ele acabou de arrastar. A régua é a mesma do preview do
/// `motion-graph`.
const PREVIEW_MAX_PX: f32 = 160.0; // LITERAL-PX-OK: teto de um preview, nao metrica de design
/// O raio da moldura, espelhando o preview do `motion-graph`.
const PREVIEW_RADIUS: f32 = 4.0; // LITERAL-PX-OK: raio de moldura
/// A espessura da moldura.
const PREVIEW_BORDER_PX: f32 = 1.0; // LITERAL-PX-OK: espessura de borda

#[cfg(test)]
#[path = "preview_tests.rs"]
mod tests;
