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
const CURVE_HIT_R_PX: f64 = 6.0; // LITERAL-PX-OK: alvo do duplo-clique na curva (ADR-0141)

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

/// A LINHA de uma alça de tangente — a mesma âmbar, meia força: ela sai da âncora e
/// termina na ponta agarrável, e é a coisa que diz *"esta âncora tem uma tangente
/// para puxar"*. Mais forte que o fio (a alça se edita), mais fraca que os pontos.
const TANGENT_LINE_RGBA: [f32; 4] = [1.0, 0.72, 0.2, 0.6]; // LITERAL-COLOR-OK: overlay de trajetoria

/// Raio do círculo da ponta de uma alça de tangente. Círculo, não quadrado: a âncora é
/// o quadrado (o que translada a curva), a ponta é o círculo (o que a MOLDA) — a mesma
/// distinção que todo editor vetorial faz.
const TANGENT_TIP_R: f64 = 2.5; // LITERAL-PX-OK: chrome de overlay, geometria de tela

/// A espessura com que a linha e a ponta de uma alça são traçadas.
const TANGENT_PX: f64 = 1.0; // LITERAL-PX-OK: chrome de overlay, espessura de tela

/// A alça só é oferecida (desenhada E agarrável) quando a ponta está a pelo menos isto
/// da âncora, em px de tela. Duas razões, uma só: uma ponta colada na âncora vira ruído
/// visual, E deixaria os dois alvos de mouse (âncora × ponta) disputarem o mesmo pixel.
/// `2 × HIT_R_PX` é onde os dois alvos de raio `HIT_R_PX` só se tocam, sem sobrepor —
/// então perto do quadrado pega-se a âncora, na ponta pega-se a alça. Numa alça curta
/// demais (auto num zoom afastado) perder a alça é honesto; perder a âncora seria bug.
const TANGENT_MIN_PX: f64 = 14.0; // LITERAL-PX-OK: chrome de overlay, alvo de mouse

/// Quantos segmentos aproximam o círculo de uma ponta. 12 é liso num alvo de ~5 px.
const TIP_SEGS: usize = 12;

// A fita de velocidade e os papéis dos grupos de desenho moram no módulo irmão (o teto
// de LOC do shell): consts `RIBBON_*`/rampa, `heat_ramp`, `MarkRole`, `OverlayMark`.
#[path = "motion_path_overlay_marks.rs"]
mod marks_mod;
use marks_mod::{
    heat_ramp, MarkRole, OverlayMark, RIBBON_BANDS, RIBBON_FLAT_REL, RIBBON_GLOW_PX,
    RIBBON_GLOW_RGBA, RIBBON_PX,
};

/// O que o ponteiro agarrou na trajetória — a âncora (translada a curva) ou uma das
/// duas alças de tangente (molda a curva). Um enum e não dois `Option` porque é UMA
/// pergunta ("o que está sob o cursor?") com uma resposta, e o gesto de arrasto despacha
/// sobre ela.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MotionPathGrab {
    /// A âncora `i` de `target` — arrastar translada a curva ali.
    Anchor { target: AnimTarget, i: usize },
    /// A alça de tangente da âncora `i` — `out` escolhe a de saída (senão a de entrada).
    Tangent {
        target: AnimTarget,
        i: usize,
        out: bool,
    },
}

/// O que desenhar para a trajetória do objeto `selected`: `(caminho, cor, espessura)`,
/// já em px de tela.
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
) -> Vec<OverlayMark> {
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

    // 1. A FITA — a forma, colorida pelo RITMO. Amostrada no TEMPO, não no arco: é o
    //    percurso que o objeto de fato faz, então um trecho que a track nunca alcança não
    //    é desenhado. Cada segmento ganha a cor da sua velocidade (comprimento de MUNDO ∝
    //    velocidade, já que o `dt` entre amostras é constante), normalizada pelo PRÓPRIO
    //    caminho — um GLOW largo e fraco por baixo, N bandas nítidas por cima.
    if span > 0.0 {
        let n = ((span * THREAD_SAMPLES_PER_SECOND).ceil() as usize).clamp(2, 4096);
        // Segmentos `(a_tela, b_tela, comprimento_mundo)`; um `None` (track não alcança)
        // quebra a fita ali, sem ligar por cima do vão.
        let mut segs: Vec<(Point, Point, f64)> = Vec::with_capacity(n);
        let mut prev: Option<([f32; 2], Point)> = None;
        for k in 0..=n {
            let cur = at(t0 + span * k as f64 / n as f64).map(|p| (p, to_screen(p)));
            if let (Some((pw, ps)), Some((cw, cs))) = (prev, cur) {
                let len = f64::from(((cw[0] - pw[0]).powi(2) + (cw[1] - pw[1]).powi(2)).sqrt());
                segs.push((ps, cs, len));
            }
            prev = cur;
        }
        if !segs.is_empty() {
            // O GLOW: todos os segmentos num traço largo e fraco, por baixo de tudo.
            let mut glow = BezPath::new();
            for (a, b, _) in &segs {
                glow.move_to(*a);
                glow.line_to(*b);
            }
            out.push(OverlayMark {
                path: glow,
                rgba: RIBBON_GLOW_RGBA,
                width: RIBBON_GLOW_PX,
                role: MarkRole::Glow,
            });

            // Normaliza a velocidade pelo próprio caminho; velocidade ~constante (range
            // desprezível) ⇒ tudo na âmbar do meio, uma fita uniforme (a leitura honesta).
            let (mut lo, mut hi) = (f64::INFINITY, 0.0f64);
            for (_, _, len) in &segs {
                lo = lo.min(*len);
                hi = hi.max(*len);
            }
            // A flatness guard that is RELATIVE, not absolute: a straight constant-speed
            // path has segment lengths that agree to ~1e-5 in relative terms (pure `f64`
            // noise from `to_screen`), and an absolute `> 1e-9` would amplify that noise
            // across the whole ramp. A speed swing under `RIBBON_FLAT_REL` reads as
            // uniform anyway — it is below what the eye separates over 24 bands, and two
            // orders of magnitude above the measured noise floor.
            let range = hi - lo;
            let flat = hi <= 0.0 || range <= hi * RIBBON_FLAT_REL;
            let mut bands: Vec<BezPath> = vec![BezPath::new(); RIBBON_BANDS];
            for (a, b, len) in &segs {
                let u = if flat {
                    0.5
                } else {
                    ((len - lo) / range) as f32
                };
                let band = ((u * RIBBON_BANDS as f32) as usize).min(RIBBON_BANDS - 1);
                bands[band].move_to(*a);
                bands[band].line_to(*b);
            }
            for (band, path) in bands.into_iter().enumerate() {
                if path.elements().is_empty() {
                    continue;
                }
                let u = (band as f32 + 0.5) / RIBBON_BANDS as f32;
                out.push(OverlayMark {
                    path,
                    rgba: heat_ramp(u),
                    width: RIBBON_PX,
                    role: MarkRole::Ribbon,
                });
            }
        }
    }

    // 2. AS LINHAS DAS TANGENTES — sob os pontos e as âncoras, porque a alça sai da
    //    âncora e o quadrado dela tem de ficar por cima. Pela MESMA porta que o hit-test.
    let mut tan_lines = BezPath::new();
    let mut tan_tips = BezPath::new();
    let mut any_tan = false;
    for (_, _, _, anchor_pt, tip) in tangent_screen(doc, selected, camera, window) {
        tan_lines.move_to(anchor_pt);
        tan_lines.line_to(tip);
        push_circle(&mut tan_tips, tip, TANGENT_TIP_R);
        any_tan = true;
    }
    if any_tan {
        out.push(OverlayMark {
            path: tan_lines,
            rgba: TANGENT_LINE_RGBA,
            width: TANGENT_PX,
            role: MarkRole::TangentLine,
        });
    }

    // 3. OS PONTOS — o tempo. Um por quadro de exibição; o espaçamento é a velocidade.
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
        out.push(OverlayMark {
            path: dots,
            rgba: PATH_RGBA,
            width: DOT_PX,
            role: MarkRole::Dot,
        });
    }

    // 4. AS ÂNCORAS — o que translada a curva, pela MESMA porta que o hit-test consulta.
    let mut anchors = BezPath::new();
    let mut any_anchor = false;
    for (_, _, p) in anchor_screen(doc, selected, camera, window) {
        push_square(&mut anchors, p, ANCHOR_HALF);
        any_anchor = true;
    }
    if any_anchor {
        out.push(OverlayMark {
            path: anchors,
            rgba: PATH_RGBA,
            width: DOT_PX,
            role: MarkRole::Anchor,
        });
    }

    // 5. AS PONTAS DAS TANGENTES — por cima de tudo, porque são o alvo mais fino de
    //    agarrar e não podem ser enterradas pela linha nem pela âncora.
    if any_tan {
        out.push(OverlayMark {
            path: tan_tips,
            rgba: PATH_RGBA,
            width: TANGENT_PX,
            role: MarkRole::TangentTip,
        });
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

/// **As pontas de alça de tangente agarráveis**, em px de tela —
/// `(alvo, índice, out, ponto_da_âncora, ponta)`.
///
/// A MESMA porta única do desenho e do hit-test, e pela mesma razão que a das âncoras.
/// A alça oferecida só existe quando a ponta está ≥ [`TANGENT_MIN_PX`] da âncora: numa
/// alça curta demais ela some (sem competir com o alvo da âncora) — o mesmo pudor que o
/// grip de fade do strip tem ao não se oferecer numa strip estreita.
///
/// Lê os handles **autorados** (relativos à âncora), não uma re-amostragem: a alça é a
/// coisa que o artista molda, e o objeto seguir a curva é derivado dela.
pub(crate) fn tangent_screen(
    doc: &TimelineDoc,
    selected: Option<u64>,
    camera: &Camera2d,
    window: WindowSize,
) -> Vec<(AnimTarget, usize, bool, Point, Point)> {
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
    let to_screen = |p: [f32; 2]| {
        let (sx, sy) = camera.world_to_screen(p, window);
        Point::new(f64::from(sx), f64::from(sy))
    };
    let mut out = Vec::new();
    for (i, a) in path.anchors().iter().enumerate() {
        let anchor_pt = to_screen(a.anchor);
        for (out_side, h) in [(false, a.in_handle), (true, a.out_handle)] {
            let tip = to_screen([a.anchor[0] + h[0], a.anchor[1] + h[1]]);
            if anchor_pt.distance(tip) >= TANGENT_MIN_PX {
                out.push((b.target, i, out_side, anchor_pt, tip));
            }
        }
    }
    out
}

/// **O que o cursor agarrou na trajetória**, se algo — a âncora ou uma alça de
/// tangente, a MAIS PRÓXIMA dentro do raio de pega.
///
/// Uma busca só sobre a união (âncoras + pontas de alça), porque a pergunta é uma:
/// *"o que está sob o cursor?"*. O filtro de [`TANGENT_MIN_PX`] em [`tangent_screen`]
/// já mantém as pontas fora do alvo da âncora, então perto do quadrado a âncora vence e
/// na ponta a alça vence, sem regra de prioridade a manter.
pub(crate) fn motion_path_hit(
    doc: &TimelineDoc,
    selected: Option<u64>,
    camera: &Camera2d,
    window: WindowSize,
    x: f32,
    y: f32,
) -> Option<MotionPathGrab> {
    let (px, py) = (f64::from(x), f64::from(y));
    let d2 = |p: Point| (p.x - px).powi(2) + (p.y - py).powi(2);
    let mut best: Option<(f64, MotionPathGrab)> = None;
    let mut consider = |dist2: f64, grab: MotionPathGrab| {
        if dist2 <= HIT_R_PX * HIT_R_PX && best.is_none_or(|(bd, _)| dist2 < bd) {
            best = Some((dist2, grab));
        }
    };
    for (target, i, p) in anchor_screen(doc, selected, camera, window) {
        consider(d2(p), MotionPathGrab::Anchor { target, i });
    }
    for (target, i, out, _, tip) in tangent_screen(doc, selected, camera, window) {
        consider(d2(tip), MotionPathGrab::Tangent { target, i, out });
    }
    best.map(|(_, g)| g)
}

/// **Onde na CURVA o cursor está**, se sobre ela — o alvo e a distância de arco do ponto
/// mais próximo. É o que o duplo-clique de "adicionar ponto" usa (ADR-0141). `None` quando
/// não há trajetória, quando o cursor está sobre uma ÂNCORA/alça (essas têm prioridade — o
/// duplo-clique nelas não é "na curva") ou quando está LONGE da curva.
///
/// ⚠️ O `project` sozinho aceita qualquer clique (devolve o ponto mais próximo do caminho
/// esteja onde estiver o cursor). Então a distância volta pra TELA e se confere que o
/// cursor está mesmo em cima da curva, dentro de [`CURVE_HIT_R_PX`] — senão um duplo-clique
/// no vazio inseriria um ponto.
pub(crate) fn motion_path_curve_hit(
    doc: &TimelineDoc,
    selected: Option<u64>,
    camera: &Camera2d,
    window: WindowSize,
    x: f32,
    y: f32,
) -> Option<(AnimTarget, f64)> {
    // Âncora/alça vence: sobre uma delas, o clique não conta como "na curva".
    if motion_path_hit(doc, selected, camera, window, x, y).is_some() {
        return None;
    }
    let entity = selected?;
    let b = doc
        .bindings()
        .iter()
        .find(|b| b.entity == entity && b.prop == PropKind::Position && !b.missing)?;
    let path = b.path.as_ref()?;
    if path.len() < 2 {
        return None;
    }
    let w = camera.screen_to_world((x, y), window);
    let d = path.project([w[0], w[1]])?;
    let on = path.at(d)?.point;
    let (sx, sy) = camera.world_to_screen(on, window);
    let dist2 = (f64::from(sx) - f64::from(x)).powi(2) + (f64::from(sy) - f64::from(y)).powi(2);
    (dist2 <= CURVE_HIT_R_PX * CURVE_HIT_R_PX).then_some((b.target, d))
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

/// Um círculo aproximado por [`TIP_SEGS`] arestas — a ponta de uma alça de tangente.
/// Sem transcendental no laço: os cossenos/senos das arestas são constantes de tabela
/// pequenas, então uso um passo incremental de rotação (Chebyshev), fiel o bastante a
/// ~5 px e livre de `sin`/`cos` por chamada (HR-5 em espírito, e é chrome de tela).
fn push_circle(p: &mut BezPath, c: Point, r: f64) {
    // Rotação incremental por (cos, sin) do passo, tabelados uma vez.
    let (cs, sn) = STEP_COS_SIN;
    let (mut dx, mut dy) = (r, 0.0);
    p.move_to(Point::new(c.x + dx, c.y + dy));
    for _ in 0..TIP_SEGS {
        let nx = dx * cs - dy * sn;
        let ny = dx * sn + dy * cs;
        dx = nx;
        dy = ny;
        p.line_to(Point::new(c.x + dx, c.y + dy));
    }
    p.close_path();
}

/// `(cos, sin)` de `2π / TIP_SEGS` (30° para 12 arestas), tabelado — nenhum `sin`/`cos`
/// roda por frame.
const STEP_COS_SIN: (f64, f64) = (0.866_025_403_784_438_6, 0.5); // cos30°, sin30°

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
    // A ordem de pintura é o PAPEL de cada marca, não a ordem em que `marks` a empurrou:
    // glow por baixo, a ponta da alça por cima. Estável, então bandas de mesma camada
    // mantêm a ordem em que nasceram.
    let mut ms = marks(show, doc, selected, camera, window);
    ms.sort_by_key(|m| m.role.layer());
    for m in ms {
        vector_scene.inner_mut().stroke(
            &Stroke::new(m.width),
            Affine::IDENTITY,
            &Brush::Solid(Color::new(m.rgba)),
            None,
            &m.path,
        );
    }
}

#[cfg(test)]
#[path = "motion_path_overlay_tests.rs"]
mod tests;
