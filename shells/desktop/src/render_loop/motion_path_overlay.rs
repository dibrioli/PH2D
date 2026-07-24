//! **A trajetória na tela** — o motion path do objeto selecionado (ADR-0141, Fatia 3).
//!
//! Um binding `PropKind::Position` guarda um caminho e uma track que mede *distância*
//! ao longo dele. Sem isto, essa geometria é invisível: o artista vê o objeto
//! aparecer noutro lugar a cada frame e não tem onde pegar a curva.
//!
//! # Os PONTOS são a leitura, não a linha
//!
//! Um ponto por **quadro de exibição**, entre a primeira e a última key. O
//! **espaçamento entre eles É a velocidade** — juntos, aglomerados; rápidos,
//! esparramados —, então uma figura carrega as duas informações, que é exatamente o
//! que impede a trajetória de virar um desenho que não diz nada sobre timing. É a
//! leitura do After Effects, e lá ela é a ÚNICA coisa desenhada.
//!
//! ⚠️ **Nós desenhamos também um fio contínuo, e é uma divergência deliberada.** O AE
//! pode viver só de pontos porque a taxa dele é alta; um documento a 24 fps com um
//! movimento rápido deixa **três** pontos na tela, e aí a FORMA da curva — que o
//! artista está justamente moldando com as alças — some. O fio é fraco de propósito:
//! ele dá a forma, os pontos dão o tempo, e o fio nunca compete com eles.
//!
//! # Espaço de TELA, deliberado
//!
//! Cada PONTO sobe pelo afim da câmera, mas o traço sai sob `Affine::IDENTITY`: no
//! Vello o transform do `stroke` **multiplica a largura**, e foi assim que o realce do
//! Flip virou um borrão (smoke, 2026-07-13). Mesma lei do `physics_overlay`.
//!
//! # Só o SELECIONADO
//!
//! Desenhar toda trajetória do documento encheria a tela de espaguete no primeiro
//! projeto com dez objetos animados. O AE mostra o motion path da camada selecionada,
//! e a razão é a mesma: a trajetória é uma coisa que se **edita**, e só se edita o que
//! está na mão.

use ph2d_anim::{AnimValue, AttributeEvaluator};
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_timeline::{AnimTarget, PropKind, TimelineDoc};
use ph2d_vector::{BezPath, Point, VectorScene};

/// A espessura do fio da trajetória, em px de tela. Mais fino que o contorno de
/// collider (1,5): o fio existe para dar FORMA, e quem se lê são os pontos.
const THREAD_PX: f64 = 1.0; // LITERAL-PX-OK: chrome de overlay, espessura de tela

/// O traço com que um ponto de tempo é desenhado.
const DOT_PX: f64 = 2.0; // LITERAL-PX-OK: chrome de overlay, espessura de tela

/// Meia-largura de um ponto de tempo, em px de tela.
const DOT_HALF: f64 = 1.6; // LITERAL-PX-OK: chrome de overlay, geometria de tela

/// Meia-largura do quadrado de uma âncora. Maior que um ponto de tempo porque é a
/// coisa que se PEGA — um alvo de mouse, não uma marca de leitura.
const ANCHOR_HALF: f64 = 4.0; // LITERAL-PX-OK: chrome de overlay, geometria de tela

/// Raio de pega de uma âncora, em px de tela. Maior que a metade desenhada (4): um
/// alvo de mouse quer folga, e esta é a mesma generosidade que a alça do texto em
/// caminho e a do conector já assumem.
const HIT_R_PX: f64 = 7.0; // LITERAL-PX-OK: chrome de overlay, alvo de mouse

/// Quantos segmentos de reta aproximam a curva por trecho entre pontos de tempo.
/// O fio é redesenhado por frame e só precisa parecer liso.
const THREAD_SAMPLES_PER_SECOND: f64 = 120.0;

/// Teto de pontos de tempo desenhados. Um documento longo a 60 fps daria milhares, e
/// além de algumas centenas eles se fundem numa linha — a leitura de velocidade morre
/// junto, então mais pontos não compram nada. Nesse regime o FIO é o que sobra, e ele
/// continua desenhado.
const MAX_DOTS: usize = 600;

/// A âmbar do realce de seleção (`flip_selection_overlay::HALO_RGBA`), porque a
/// trajetória **é** uma extensão de *"esta é a coisa que você tem na mão"*: ela só
/// aparece para o objeto selecionado, e some com ele.
///
/// Fica inteiramente fora do vocabulário do `physics_overlay` (verde/ciano/violeta/
/// magenta/branco/laranja), que é uma família FECHADA sobre *o que uma coisa é
/// fisicamente* — uma trajetória não é uma resposta a essa pergunta.
const PATH_RGBA: [f32; 4] = [1.0, 0.72, 0.2, 0.95]; // LITERAL-COLOR-OK: overlay de trajetoria

/// O fio: a MESMA âmbar, fraca. Ele dá a forma e nunca disputa com os pontos.
const THREAD_RGBA: [f32; 4] = [1.0, 0.72, 0.2, 0.35]; // LITERAL-COLOR-OK: overlay de trajetoria

/// O que desenhar para a trajetória do objeto `selected`: `(caminho, cor)`, já em px
/// de tela.
///
/// Vazio quando não há seleção, quando o selecionado não tem binding Position, quando
/// esse binding não tem caminho, ou quando a track está vazia — em nenhum desses casos
/// existe uma trajetória a mostrar, e desenhar algo seria inventar uma.
pub(crate) fn marks(
    show: bool,
    doc: &TimelineDoc,
    selected: Option<u64>,
    camera: &Camera2d,
    window: WindowSize,
) -> Vec<(BezPath, [f32; 4])> {
    let mut out = Vec::new();
    if !show {
        return out;
    }
    let Some(entity) = selected else { return out };
    let Some(b) = doc
        .bindings()
        .iter()
        .find(|b| b.entity == entity && b.prop == PropKind::Position && !b.missing)
    else {
        return out;
    };
    let Some(path) = b.path.as_ref() else {
        return out;
    };
    let Some(track) = doc.active_clip().track(b.target) else {
        return out;
    };
    let keys = track.keys();
    let (Some(first), Some(last)) = (keys.first(), keys.last()) else {
        return out;
    };

    let to_screen = |p: [f32; 2]| {
        let (sx, sy) = camera.world_to_screen([p[0], p[1]], window);
        Point::new(f64::from(sx), f64::from(sy))
    };
    // Onde o objeto está no instante `t` do clip: a MESMA composição que o apply faz
    // (track → distância → ponto). Uma segunda derivação aqui desenharia a trajetória
    // onde o objeto não está.
    let at = |t: f64| -> Option<[f32; 2]> {
        let AnimValue::Float(s) = track.sample(t) else {
            return None;
        };
        path.at(f64::from(s)).map(|k| k.point)
    };

    let (t0, t1) = (first.t.to_seconds(), last.t.to_seconds());
    let span = t1 - t0;

    // 1. O FIO — a forma. Amostrado no TEMPO, não no arco: é o percurso que o objeto
    //    de fato faz, então um trecho que a track nunca alcança não é desenhado.
    if span > 0.0 {
        let n = ((span * THREAD_SAMPLES_PER_SECOND).ceil() as usize).clamp(2, 4096);
        let mut thread = BezPath::new();
        let mut started = false;
        for k in 0..=n {
            let Some(p) = at(t0 + span * k as f64 / n as f64) else {
                continue;
            };
            let sp = to_screen(p);
            if started {
                thread.line_to(sp);
            } else {
                thread.move_to(sp);
                started = true;
            }
        }
        if started {
            out.push((thread, THREAD_RGBA));
        }
    }

    // 2. OS PONTOS — o tempo. Um por quadro de exibição; o espaçamento é a velocidade.
    let fps = doc.fps_display.max(1.0);
    let frames = ((span * fps).round() as usize).min(MAX_DOTS);
    let mut dots = BezPath::new();
    let mut any_dot = false;
    for k in 0..=frames {
        let t = if frames == 0 {
            t0
        } else {
            t0 + span * k as f64 / frames as f64
        };
        let Some(p) = at(t) else { continue };
        push_diamond(&mut dots, to_screen(p), DOT_HALF);
        any_dot = true;
    }
    if any_dot {
        out.push((dots, PATH_RGBA));
    }

    // 3. AS ÂNCORAS — o que se pega, pela MESMA porta que o hit-test consulta.
    let mut anchors = BezPath::new();
    let mut any_anchor = false;
    for (_, _, p) in anchor_screen(doc, selected, camera, window) {
        push_square(&mut anchors, p, ANCHOR_HALF);
        any_anchor = true;
    }
    if any_anchor {
        out.push((anchors, PATH_RGBA));
    }
    out
}

/// **Onde estão as âncoras agarráveis, em px de tela** — `(alvo, índice, ponto)`.
///
/// ⚠️ **A porta ÚNICA**, e é a razão de esta função existir separada do desenho: quem
/// PINTA e quem faz HIT-TEST têm de concordar sobre onde a âncora está. Duas derivações
/// divergem, e o modo de falha é a alça pintada num sítio e agarrada noutro — o dedo
/// erra e o artista conclui que a feature está quebrada. É a mesma lei que a alça do
/// texto em caminho já paga (`handle::world`).
///
/// Lê o ponto **autorado** (`anchors()[i].anchor`), não uma re-amostragem por
/// distância: a âncora é a coisa que o artista pôs ali, e a distância é derivada dela.
pub(crate) fn anchor_screen(
    doc: &TimelineDoc,
    selected: Option<u64>,
    camera: &Camera2d,
    window: WindowSize,
) -> Vec<(AnimTarget, usize, Point)> {
    let Some(entity) = selected else {
        return Vec::new();
    };
    let Some(b) = doc
        .bindings()
        .iter()
        .find(|b| b.entity == entity && b.prop == PropKind::Position && !b.missing)
    else {
        return Vec::new();
    };
    let Some(path) = b.path.as_ref() else {
        return Vec::new();
    };
    path.anchors()
        .iter()
        .enumerate()
        .map(|(i, a)| {
            let (sx, sy) = camera.world_to_screen(a.anchor, window);
            (b.target, i, Point::new(f64::from(sx), f64::from(sy)))
        })
        .collect()
}

/// **A âncora sob o cursor**, se houver — o que o press agarra.
///
/// A MAIS PRÓXIMA dentro do raio, nunca a primeira encontrada: num caminho apertado
/// duas âncoras podem estar as duas dentro do alvo, e "a primeira da lista" faria o
/// dedo pegar a de trás sem nada na tela explicando por quê.
pub(crate) fn anchor_at(
    doc: &TimelineDoc,
    selected: Option<u64>,
    camera: &Camera2d,
    window: WindowSize,
    x: f32,
    y: f32,
) -> Option<(AnimTarget, usize)> {
    let (px, py) = (f64::from(x), f64::from(y));
    anchor_screen(doc, selected, camera, window)
        .into_iter()
        .map(|(t, i, p)| ((p.x - px).powi(2) + (p.y - py).powi(2), t, i))
        .filter(|(d2, _, _)| *d2 <= HIT_R_PX * HIT_R_PX)
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, t, i)| (t, i))
}

/// Um losango de meia-largura `half` centrado em `c`. Losango e não círculo: um ponto
/// de tempo mede 3 px e um disco desse tamanho vira uma mancha, enquanto quatro
/// arestas ainda leem como uma marca discreta quando dois deles quase se tocam.
fn push_diamond(p: &mut BezPath, c: Point, half: f64) {
    p.move_to(Point::new(c.x, c.y - half));
    p.line_to(Point::new(c.x + half, c.y));
    p.line_to(Point::new(c.x, c.y + half));
    p.line_to(Point::new(c.x - half, c.y));
    p.close_path();
}

/// Um quadrado alinhado ao eixo — a forma que TODO editor vetorial usa para "âncora
/// que se arrasta", e por isso distinta do losango que é só leitura.
fn push_square(p: &mut BezPath, c: Point, half: f64) {
    p.move_to(Point::new(c.x - half, c.y - half));
    p.line_to(Point::new(c.x + half, c.y - half));
    p.line_to(Point::new(c.x + half, c.y + half));
    p.line_to(Point::new(c.x - half, c.y + half));
    p.close_path();
}

/// Pinta. No-op quando [`marks`] não devolve nada.
pub(super) fn draw(
    show: bool,
    doc: &TimelineDoc,
    selected: Option<u64>,
    camera: &Camera2d,
    window: WindowSize,
    vector_scene: &mut VectorScene,
) {
    use ph2d_vector::{Affine, Brush, Color, Stroke};
    for (path, rgba) in marks(show, doc, selected, camera, window) {
        // O fio é mais fino que as marcas: a forma é fundo, o tempo é leitura.
        let px = if rgba == THREAD_RGBA {
            THREAD_PX
        } else {
            DOT_PX
        };
        vector_scene.inner_mut().stroke(
            &Stroke::new(px),
            Affine::IDENTITY,
            &Brush::Solid(Color::new(rgba)),
            None,
            &path,
        );
    }
}

#[cfg(test)]
#[path = "motion_path_overlay_tests.rs"]
mod tests;
