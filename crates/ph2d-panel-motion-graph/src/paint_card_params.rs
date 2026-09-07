//! ⭐⭐⭐ **OS PARAMS DESENHADOS NO CARTÃO** — o ciclo 1 da dinâmica
//! ([doc 103](../../../docs/Motion%20Nodes/103_dinamica_dos_ciclos.md)), decisão do Enio de
//! 2026-09-05: *«como no Blender, os parâmetros dos nós devem ser desenhados nos nós e vamos
//! retirar o painel lateral»*.
//!
//! **Uma row = a faixa inteira do cartão**, com o rótulo à esquerda, o valor à direita e o
//! nível como PREENCHIMENTO da própria faixa — é o número do Blender (arrasta-se em qualquer
//! ponto) e não o par «rótulo em cima, calha em baixo» do Mini Cavalry, que gasta **duas**
//! linhas por param. Num cartão que vai hospedar até 24 params, a diferença é o dobro da
//! altura; e uma faixa inteira é também o maior alvo de toque possível (44 pt a partir de
//! `zoom 2`), que é a régua do tablet.
//!
//! ⚠️ **Este ficheiro é irmão do [`super::paint`] por RESPONSABILIDADE:** o pai desenha o que
//! um cartão É (moldura, cabeçalho, sockets, véu), este desenha o que ele CONTROLA. O pai
//! está a 589 linhas de um tecto de 700 — mas o corte é por assunto, não por contagem.

use crate::geom::{self, View};
use crate::snapshot::{CardParam, GraphNodeView};
use ph2d_editor_core::paint::{fill_rounded_rect, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::text_elide::paint_text_title_elided;
use ph2d_editor_core::zones::Rect;
use ph2d_node_registry::ParamWidget;
use ph2d_tokens::{ColorToken, Theme};

/// ⚠️ **Os recuos da faixa mudaram-se para o [`geom`]** quando o clique passou a depender de
/// ONDE dentro da row ele caiu: o pintor e o gesto medem a mesma faixa ou a seta desenhada
/// deixa de ser a seta clicada.
use geom::TRACK_INSET_X;
/// Raio da faixa — o mesmo do cartão dividido por dois, para a peça pequena não parecer um
/// cartão pequeno.
const TRACK_R: f32 = 4.0; // LITERAL-PX-OK: card param track corner radius
/// Recuo do texto dentro da faixa.
const TEXT_PAD_X: f32 = 7.0; // LITERAL-PX-OK: card param text x-inset
/// Descida do texto dentro da fileira, para a linha de base ficar centrada.
const TEXT_PAD_Y: f32 = 5.0; // LITERAL-PX-OK: card param text y-inset
/// Folga da amostra de cor dentro da faixa (ela é um quadrado, não uma barra).
const SWATCH_INSET: f32 = 3.0; // LITERAL-PX-OK: card colour swatch inset
const SWATCH_R: f32 = 3.0; // LITERAL-PX-OK: card colour swatch corner radius
/// Meio-lado do chevron de uma secção.
const CHEVRON_R: f32 = 3.5; // LITERAL-PX-OK: section chevron half-size
/// **A FORMA da setinha, em proporções do seu meio-lado** — não são medidas de desenho, são a
/// geometria do triângulo (a mesma família dos `TAPER`/`CIGAR_H` das silhuetas).
/// `▾ aberta`: base achatada a meia altura, bico abaixo. `▸ fechada`: espelhada no eixo.
const CHEVRON_FLAT: f32 = 0.5; // LITERAL-PX-OK: proporcao da forma, nao medida
const CHEVRON_TIP: f32 = 0.7; // LITERAL-PX-OK: proporcao da forma, nao medida

/// A largura que o texto de facto ocupa — pela porta que vive **ao lado do pintor**
/// ([`ph2d_editor_core::text_elide::title_elided_width`]), e não pelo `prefix_width` cru.
///
/// ⚠️ Foi assim que os números dos cartões apareciam como `0....` e sumiam ao afastar
/// (report do Enio, 2026-09-05): medidos no peso normal, pintados em semi-negrito.
fn painted_width(ctx: &mut PaintCtx, text: &str, size: f32) -> f32 {
    ph2d_editor_core::text_elide::title_elided_width(ctx.text_system, text, size)
}

/// ⭐⭐⭐ **O QUE UMA ROW MOSTRA, por ESPÉCIE de widget — e o `match` é EXAUSTIVO de propósito.**
///
/// O registry declara **catorze** espécies de controlo, e a primeira versão desta função
/// desenhava um número para todas: uma cor lia-se `0.50` (o canal vermelho), uma curva lia-se
/// `0.00`. ⚠️ *Uma row que mostra um número onde não há número é a mesma mentira que um knob
/// morto* — e com um `match` sem `_` uma espécie NOVA é erro de compilação aqui, que é o aviso
/// certo.
enum Shown {
    /// Um NÍVEL: a faixa preenche-se e o número lê-se à direita.
    Level { text: String, fill: f32 },
    /// Um ESTADO sem nível — um interruptor, uma opção de enum, um nome de canal.
    State(String),
    /// Uma COR: uma amostra, nunca um número.
    Swatch([u8; 4]),
    /// Um EDITOR RICO (curva, gradiente, paleta, texto, ficheiro, fonte). O cartão diz que o
    /// controlo existe; abri-lo é obra do passo seguinte do ciclo 1. ⛔ **Não inventa valor.**
    Editor,
}

fn shown(p: &CardParam) -> Shown {
    let level = |text: String| {
        // ⚠️ A faixa **RESOLVIDA** (`p.min`/`p.max`), nunca a do hint: numa magnitude ligada a
        // um canal angular o hint descreve o outro canal, e o nível saía saturado.
        let span = p.max - p.min;
        let f = if span.abs() > f32::EPSILON {
            ((p.value - p.min) / span).clamp(0.0, 1.0)
        } else {
            0.0
        };
        Shown::Level { text, fill: f }
    };
    // ⭐⭐ **UMA LEI PARA COMO UM NÚMERO LÊ** — o formatador da casa
    // ([`ph2d_editor_core::interaction::format_number`], o mesmo do painel) mais o sufixo da
    // FACE. ⛔ Um segundo formatador aqui seria uma segunda forma de arredondar, e o mesmo param
    // leria `0.50` no cartão e `0,503 px` no painel.
    let com_face = |v: f32| {
        let n = ph2d_editor_core::interaction::format_number(f64::from(v));
        if p.face_suffix.is_empty() {
            n
        } else {
            format!("{n} {}", p.face_suffix)
        }
    };
    match p.hint.widget {
        // Um contínuo, um inteiro, uma semente e um ângulo são todos NÍVEIS — o que muda entre
        // eles é a face, e ela já vem resolvida na row.
        ParamWidget::Slider | ParamWidget::Angle => level(com_face(p.value)),
        ParamWidget::IntSlider | ParamWidget::Seed => level(com_face(p.value.round())),
        ParamWidget::Toggle => Shown::State(if p.value >= 0.5 { "On" } else { "Off" }.to_string()),
        ParamWidget::Enum { labels } => Shown::State(
            labels
                .get(p.value.round().max(0.0) as usize)
                .map_or_else(|| p.value.to_string(), |s| (*s).to_string()),
        ),
        // Sem amostra (a shell não a preencheu) a row diz que há uma cor, não uma cor errada.
        ParamWidget::Color { .. } => p.swatch.map_or(Shown::Editor, Shown::Swatch),
        // ⭐⭐ **O QUE ESTÁ ESCOLHIDO LÊ-SE NA ROW** quando a shell soube dizê-lo — o nome da
        // forma, o do ficheiro, o da coluna. Sem isso o artista muda uma coisa que não consegue
        // ler: o clique funciona, a tela responde, e a row continua a mostrar o mesmo selo.
        //
        // ⛔ **O selo NÃO é o caso de erro** — ele é a resposta certa para o texto de MÁQUINA
        // (uma curva, um gradiente e uma paleta são serializações) e para o que ainda não foi
        // escolhido. Quem decide qual é qual é a shell, que tem o documento; aqui só se desenha.
        ParamWidget::Channels { .. }
        | ParamWidget::Source
        | ParamWidget::Text
        | ParamWidget::Curve
        | ParamWidget::Gradient
        | ParamWidget::Palette
        | ParamWidget::File { .. } => {
            if p.text.is_empty() {
                Shown::Editor
            } else {
                Shown::State(p.text.as_str().to_string())
            }
        }
    }
}

/// ⭐⭐ **UMA ROW ACEITA UM NÚMERO ESCRITO EXACTAMENTE QUANDO MOSTRA UM NÍVEL** — e a pergunta
/// responde-se pela MESMA função que decide o que a row desenha ([`shown`]), nunca por uma
/// segunda lista de espécies.
///
/// ⚠️ *Duas listas para uma pergunta é como uma espécie nova ganha um número no ecrã e não o
/// deixa escrever* — o `match` de [`shown`] é exaustivo de propósito, e esta resposta vem
/// atrelada a ele: uma variante nova de `ParamWidget` é erro de compilação lá, e daí sai o
/// veredito aqui de graça.
pub(crate) fn shows_a_level(p: &CardParam) -> bool {
    matches!(shown(p), Shown::Level { .. })
}

/// **O CABEÇALHO DE UMA SECÇÃO** — um galão discreto com o nome, o chevron do estado e, quando
/// fechada, **quantas rows esconde** (uma secção dobrada não pode parecer uma secção vazia).
///
/// ⚠️ Sem fundo próprio: ele separa por TIPOGRAFIA e pelo chevron, não por mais uma caixa. Um
/// cartão com seis secções teria seis caixas dentro de uma caixa.
fn draw_section_header(
    ctx: &mut PaintCtx,
    sec: &crate::snapshot::CardSection,
    row: Rect,
    com_texto: bool,
    z: f32,
    theme: Theme,
) {
    // O chevron desenha-se SEMPRE (é a única marca de que ali há uma dobra); o nome segue o
    // LOD, como todo texto do cartão.
    let cx = row.x + (TRACK_INSET_X + CHEVRON_R) * z;
    let cy = row.y + row.h * 0.5;
    let r = CHEVRON_R * z;
    let pts = if sec.open {
        // ▾ aberta
        [
            (cx - r, cy - r * CHEVRON_FLAT),
            (cx + r, cy - r * CHEVRON_FLAT),
            (cx, cy + r * CHEVRON_TIP),
        ]
    } else {
        // ▸ fechada
        [
            (cx - r * CHEVRON_FLAT, cy - r),
            (cx + r * CHEVRON_TIP, cy),
            (cx - r * CHEVRON_FLAT, cy + r),
        ]
    };
    ph2d_editor_core::paint_shapes::fill_polygon(
        ctx.scene,
        &pts,
        resolve(ColorToken::Text3, theme),
    );
    if !com_texto {
        return;
    }
    let x = cx + (CHEVRON_R + TEXT_PAD_X) * z;
    let size = geom::PARAM_LABEL_SIZE * z;
    let contagem = (!sec.open && sec.hidden > 0).then(|| sec.hidden.to_string());
    let right_w = contagem.as_ref().map_or(0.0, |t| {
        let w = painted_width(ctx, t, size);
        paint_text_title_elided(
            ctx.text_system,
            ctx.scene,
            t,
            row.x + row.w - (TRACK_INSET_X + TEXT_PAD_X) * z - w,
            row.y + TEXT_PAD_Y * z,
            size,
            w,
            resolve(ColorToken::Text3, theme),
        );
        w
    });
    paint_text_title_elided(
        ctx.text_system,
        ctx.scene,
        sec.title,
        x,
        row.y + TEXT_PAD_Y * z,
        size,
        (row.x + row.w - (TRACK_INSET_X + TEXT_PAD_X) * z - right_w - x).max(0.0),
        resolve(ColorToken::Text2, theme),
    );
}

/// **Esta row é um SELECTOR?** — a porta do clique, lida aqui para o desenho e o gesto nunca
/// discordarem sobre onde estão as setas.
fn seletor(p: &CardParam) -> Option<()> {
    matches!(
        crate::interact::param_row::click_does(p),
        crate::ClickDoes::Cycle(_) | crate::ClickDoes::CycleSource | crate::ClickDoes::CycleChannel
    )
    .then_some(())
}

/// As duas setas, uma em cada ponta da faixa — triângulos da mesma família do chevron de secção
/// (o cartão desenha as suas marcas em geometria, nunca em glifos).
fn draw_arrows(ctx: &mut PaintCtx, track: Rect, seta: f32, z: f32, theme: Theme) {
    let r = geom::ARROW_R * z;
    let cy = track.y + track.h * 0.5;
    let cor = resolve(ColorToken::Text3, theme);
    for (cx, dir) in [
        (track.x + seta * 0.5, -1.0),
        (track.x + track.w - seta * 0.5, 1.0),
    ] {
        ph2d_editor_core::paint_shapes::fill_polygon(
            ctx.scene,
            &[
                (cx + r * CHEVRON_TIP * dir, cy),
                (cx - r * CHEVRON_FLAT * dir, cy - r),
                (cx - r * CHEVRON_FLAT * dir, cy + r),
            ],
            cor,
        );
    }
}

/// Desenha a faixa de params de um cartão. **Nada acontece abaixo do LOD**
/// ([`geom::params_are_drawn`]) — nem o desenho nem, do lado do hit-test, o registo: uma row
/// pintada onde não se clica é um controlo morto, e uma registada onde não se vê é um alvo
/// invisível.
pub(super) fn draw_card_params(ctx: &mut PaintCtx, n: &GraphNodeView, view: &View, theme: Theme) {
    if n.params.is_empty() {
        return;
    }
    // ⚠️ **A BARRA pinta-se sempre; só o TEXTO passa pelo LOD** — ver
    // [`geom::param_text_is_drawn`], e o smoke que o ensinou.
    let com_texto = geom::param_text_is_drawn(view);
    let z = view.zoom;
    for i in 0..geom::band_len(n) {
        let row = geom::param_row_rect(n, view, i);
        let p = match geom::band_at(n, i) {
            Some(geom::BandRow::Param(k)) => &n.params[k],
            // ⭐ **O cabeçalho de uma SECÇÃO** — o «painel dentro do nó» do Blender 4.x.
            Some(geom::BandRow::Header(k)) => {
                draw_section_header(ctx, &n.sections[k], row, com_texto, z, theme);
                continue;
            }
            None => break,
        };
        let track = geom::param_track_rect(row, z);
        fill_rounded_rect(
            ctx.scene,
            track,
            TRACK_R * z,
            resolve(ColorToken::Bg0, theme),
        );
        let what = shown(p);
        // O NÍVEL, dentro da mesma faixa. ⚠️ Um param DIRIGIDO não desenha nível: o número vem
        // de um fio e não obedece ao dedo — mostrar um nível arrastável seria a mentira que o
        // painel já aprendeu a não contar (a row dirigida, doc 88 B3).
        if let Shown::Level { fill, .. } = &what
            && !p.driven
            && *fill > 0.0
        {
            let bar = Rect::new(track.x, track.y, track.w * fill, track.h);
            fill_rounded_rect(
                ctx.scene,
                bar,
                TRACK_R * z,
                resolve(ColorToken::AccentSoft, theme),
            );
        }
        if !com_texto {
            continue;
        }
        // ⭐⭐⭐ **AS DUAS SETAS DE UM SELECTOR** (report do Enio, 2026-09-07, com a foto do
        // selector do Blender). Elas dizem, sem uma palavra, que ali há uma LISTA — e que o
        // centro é outro alvo.
        //
        // ⚠️ **Quem responde *«isto é um selector?»* é a porta do CLIQUE** (`click_does`), não
        // uma segunda lista de espécies aqui: uma espécie nova de selector ganha as setas no
        // mesmo dia em que ganha o gesto, e nunca uma sem a outra.
        let track = match seletor(p).and_then(|_| geom::arrow_slot(track, z)) {
            Some(seta) => {
                draw_arrows(ctx, track, seta, z, theme);
                Rect::new(track.x + seta, track.y, track.w - 2.0 * seta, track.h)
            }
            None => track,
        };
        let text_y = row.y + TEXT_PAD_Y * z;
        let size = geom::PARAM_LABEL_SIZE * z;
        let (label_tone, value_tone) = if p.driven {
            (ColorToken::Text3, ColorToken::PortValue)
        } else {
            (ColorToken::Text2, ColorToken::Text1)
        };
        // A AMOSTRA de cor ocupa o lugar do número, encostada à direita como ele.
        let right_w = match &what {
            Shown::Swatch(rgba) => {
                let side = track.h - 2.0 * SWATCH_INSET * z;
                let sw = Rect::new(
                    track.x + track.w - TEXT_PAD_X * z - side,
                    track.y + SWATCH_INSET * z,
                    side,
                    side,
                );
                ph2d_editor_core::paint_shapes::fill_rounded_rect_srgb8(
                    ctx.scene,
                    sw,
                    SWATCH_R * z,
                    *rgba,
                );
                side
            }
            Shown::Level { text, .. } | Shown::State(text) => {
                let vw = painted_width(ctx, text, size);
                // O VALOR primeiro, encostado à direita — é o que o artista procura, e
                // alinhá-lo à direita é o que faz uma coluna de números ler-se como coluna.
                paint_text_title_elided(
                    ctx.text_system,
                    ctx.scene,
                    text,
                    track.x + track.w - TEXT_PAD_X * z - vw,
                    text_y,
                    size,
                    vw,
                    resolve(value_tone, theme),
                );
                vw
            }
            Shown::Editor => 0.0,
        };
        // E o rótulo, elidido no espaço que SOBRA — quando os dois disputam o pixel, quem
        // encolhe é o nome, nunca o número.
        let label_x = track.x + TEXT_PAD_X * z;
        let label_w =
            (track.x + track.w - TEXT_PAD_X * z - right_w - TEXT_PAD_X * z - label_x).max(0.0);
        paint_text_title_elided(
            ctx.text_system,
            ctx.scene,
            p.hint.label,
            label_x,
            text_y,
            size,
            label_w,
            resolve(label_tone, theme),
        );
    }
}
