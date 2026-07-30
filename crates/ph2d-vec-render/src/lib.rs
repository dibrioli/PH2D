#![forbid(unsafe_code)]
//! ph2d-vec-render — pipeline de render da cena vetorial nova (ADR-0108, Fase 0).
//!
//! Converte o modelo editor-first (`ph2d-vec-scene`) em chamadas Vello, emitindo
//! no **`VectorScene` fundacional compartilhado do frame** (`ph2d-vector`) — sem
//! abrir passe de GPU novo, só anexando comandos de encode à cena que o compositor
//! já rasteriza. Toda a stack Linebender chega pelas re-exports de `ph2d-vector`
//! (gate-proof + skew-proof).
//!
//! Fase 0: draw estático da cena inteira. **Dirty-tracking** (só re-encodar a
//! sub-árvore que mudou — a alavanca de escala do ADR-0108) é o próximo passo.

use std::borrow::Cow;

use ph2d_vec_scene::{
    FillRule as VecFillRule, LineCap, LineJoin, Paint, Rgba8, StrokePiece, StrokeSpec, VecPath,
    VecPathId, VecScene, VecViewState, VecXforms,
};
use ph2d_vector::{
    Affine, BezPath, Brush, Cap, Circle, Color, ColorStop, Fill, Gradient, Join, Point, Rect,
    Shape, Stroke, VectorScene,
};

/// A silhueta em SEGMENTOS, para o campo de distância dos FX raster (módulo irmão pelo LOC cap,
/// e ao lado de `draw_path_isolated` de propósito — as duas resolvem o mesmo transform).
mod silhouette;
pub use silhouette::{MAX_SEGMENTS, silhouette_segments};

/// Gradient rendering (multi-point IDW fill) + on-canvas editing handles live in a
/// sibling module (LOC cap).
mod gradient;
use gradient::fill_multipoint;
pub use gradient::{GradHandle, drag_gradient_handle, draw_gradient_handles, hit_gradient_handle};

/// Smart guides (o feedback visual do snap), likewise a sibling.
mod guides;
pub use guides::{Guide, draw_snap_guides, draw_text_caret};

/// **As alças de ponta do conector** — os dois círculos que dizem onde a linha encosta na
/// forma. O raio vive lá dentro, e o hit-test da shell o importa: desenhar num raio e agarrar
/// noutro faz o usuário clicar no meio da bolinha e não pegar nada.
mod connector;
pub use connector::{HANDLE_R_PX, draw_connector_handles, draw_connector_waypoints};

// NOTA: o `mod corner` (draw_corner_handles, a alça de raio na bissetriz) foi REMOVIDO — o
// arredondar/chanfrar quina virou o par de ferramentas Fillet / Chamfer no shell. O `corner.rs`
// deste crate saiu junto.

/// A **gaiola do Envelope** (ADR-0129, Fatia 1) — módulo irmão (LOC cap). Os 4 cantos que o modo
/// Node arrasta para deformar a forma; o raio da bolinha ([`ENVELOPE_HANDLE_R_PX`]) é o mesmo que o
/// hit-test do host lê.
mod envelope;
pub use envelope::{
    ENVELOPE_HANDLE_R_PX, EnvelopeCageView, draw_envelope_cage, draw_envelope_pins,
};

/// A **alça do texto em caminho** (plano 22, W5) — módulo irmão (LOC cap). A bolinha onde o
/// texto começa no caminho; arrastá-la corre o texto ao longo dele.
mod text_handle;
pub use text_handle::draw_text_handle;
mod width_handle;
pub use width_handle::draw_width_handle;

/// O realce das FACES do Shape Builder — módulo irmão (LOC cap). É a feature: sem ele o
/// artista arrasta às cegas e só descobre o que pegou depois de soltar.
mod build_faces;
pub use build_faces::draw_build_faces;

/// O **overlay do Blend Object** (ADR-0128) — módulo irmão (LOC cap). Desenha o overlay ordenado
/// (passos + fontes reempilhadas) que NÃO está na cena, pela mesma porta ([`draw_path`]) que a
/// arte real.
mod blend_overlay;
pub use blend_overlay::draw_blend_overlay;

/// **De `VecPath` a `BezPath`** — os construtores de desenho (módulo irmão, teto de LOC).
mod build;
pub(crate) use build::build_contours;
pub use build::{build_bezpath, build_fill_bezpath, build_lines_bezpath};

/// A [`Fill`] rule do Vello para o `fill_rule` do path.
pub(crate) fn fill_rule(path: &VecPath) -> Fill {
    match path.fill_rule {
        VecFillRule::NonZero => Fill::NonZero,
        VecFillRule::EvenOdd => Fill::EvenOdd,
    }
}

/// O afim local→tela do path: o `Transform` dele (ADR-0111), depois a câmera.
///
/// A geometria do path é LOCAL; quem a põe no mundo é `xforms`. Path ausente do
/// mapa ⇒ identidade ⇒ local é mundo, que é o estado de todo path recém-criado.
#[must_use]
pub fn path_to_screen(xforms: &VecXforms, id: VecPathId, camera: Affine) -> Affine {
    camera * Affine::new(ph2d_vec_scene::xform_of(xforms, id).0)
}

/// A **geometria DERIVADA** de um caminho neste frame, em MUNDO — o que ele DESENHA quando o que
/// ele desenha não é o que ele guarda.
///
/// Existe porque nem toda geometria viva cabe dentro do [`VecPath::cooked`]: o Offset vivo
/// (`VecOffset`) precisa do motor booleano, que **depende** da `ph2d-vec-scene` e portanto não
/// pode ser chamado de dentro dela (o cargo recusa o ciclo por nome). A shell coze essa metade e
/// a entrega aqui.
///
/// ⚠️ **Um caminho PRESENTE com a lista VAZIA desenha NADA** — é a aniquilação (um offset
/// negativo grande come a forma). É diferente de estar AUSENTE, que desenha a fonte. Colapsar os
/// dois faria a forma reaparecer inteira no instante em que o offset a mata.
pub type LiveGeometry = std::collections::BTreeMap<VecPathId, Vec<VecPath>>;

/// Uma imagem de FX raster já pronta — pixels que o shell produziu (rasterizou a forma isolada,
/// borrou, tingiu) e que o [`dispatch`] injeta no z da forma. O `dispatch` **não a computa** (é
/// encode puro, sem GPU): ele só a desenha, com `rect` já em coordenadas de TELA.
///
/// ⚠️ Carrega um [`ph2d_vector::StableImage`] (id de Blob ESTÁVEL), não RGBA cru: o shell o
/// constrói UMA vez (no memo) e clona por frame, então o Vello reusa a textura do atlas em vez de
/// re-enviá-la a cada frame — a diferença entre 60 fps e uma queda extrema num FX desenhado sempre.
#[derive(Clone)]
pub struct FxImage {
    /// A imagem RGBA reta como recurso estável (o produtor a constrói uma vez).
    pub image: ph2d_vector::StableImage,
    /// O retângulo de destino em pixels de TELA (`x0,y0,x1,y1`) — o shell já cruzou a câmera e
    /// somou a margem do blur (e o deslocamento da sombra).
    pub rect: (f64, f64, f64, f64),
}

/// Os FX raster deste frame, por forma. Vazio = nenhum FX na cena, e o desenho é o de sempre —
/// **byte-idêntico** ao mundo pré-FX (o caminho comum não paga nada).
pub type FxImages = std::collections::BTreeMap<VecPathId, FxImage>;

/// Desenha toda a `scene` no `target` (o `VectorScene` do frame) sob `camera`
/// (o world→screen). Fill primeiro, stroke por cima.
///
/// `view` diz quem a ÁRVORE do editor esconde — a visibilidade é da entidade ECS
/// do path e dos ancestrais dela, não do documento (ADR-0110). `xforms` diz onde
/// cada path está — o `Transform` da entidade dele (ADR-0111). O stroke escala
/// junto com a forma, como o contorno de um sprite escalado.
///
/// `live` ([`LiveGeometry`]) troca a geometria de um caminho pela DERIVADA dele **no z dele** —
/// não num passe por cima de tudo. É o que mantém a promessa do Offset vivo: o documento guarda
/// a curva autorada (o modo Node edita os nós DELA) e o que se vê é o resultado, empilhado
/// exatamente onde a forma sempre esteve.
///
/// `fx` ([`FxImages`]) injeta o FX raster de uma forma **no z dela**, SUBSTITUINDO o desenho
/// vetorial: a pilha de filtros já compôs sombra, brilho e forma numa imagem só (o halo entra por
/// baixo DENTRO do op — ver `ph2d_render::fx_stack`), então não há um "atrás" que o compositor
/// precise conhecer. As imagens já vêm em coordenadas de tela — o `dispatch` só as encoda, sem
/// tocar GPU. Vazio = sem FX (o caminho comum é byte-idêntico ao mundo pré-FX).
pub fn dispatch(
    scene: &VecScene,
    view: &VecViewState,
    xforms: &VecXforms,
    live: &LiveGeometry,
    fx: &FxImages,
    camera: Affine,
    target: &mut VectorScene,
) {
    for path in scene.paths() {
        if view.is_hidden(path.id) {
            continue;
        }
        // O FX da forma, se houver, TOMA o lugar do desenho: a pilha já compôs tudo o que se vê
        // desta forma (halo incluído) numa imagem só, no z dela.
        if let Some(img) = fx.get(&path.id) {
            draw_fx_image(img, target);
        } else {
            // A derivada já está em MUNDO (a shell assou a pose dentro dela), então ela sobe pela
            // CÂMERA e não pelo afim do path — aplicar a pose duas vezes foi bug real desta linha.
            if let Some(items) = live.get(&path.id) {
                for item in items {
                    draw_path(item, camera, target);
                }
            } else {
                let transform = path_to_screen(xforms, path.id, camera);
                draw_path(path, transform, target);
            }
        }
    }
}

/// Encoda uma [`FxImage`] na cena, no retângulo de tela dela. RGBA reta (a mesma política do
/// overlay de Background-Removal).
fn draw_fx_image(img: &FxImage, target: &mut VectorScene) {
    // Id de Blob estável ⇒ o Vello reusa a textura do atlas (sem re-upload por frame).
    target.draw_stable_image(&img.image, img.rect, ph2d_vector::ImageQuality::Medium);
}

/// **O bbox em TELA de um caminho, como o [`dispatch`] o desenharia** — honrando a geometria
/// DERIVADA (`live`) e a pose (`xforms`). É a pergunta que o produtor de FX raster (plano 24) faz
/// para dimensionar o scratch e posicionar a imagem: onde, na tela, esta forma vive?
///
/// Inclui a metade da espessura do traço (o contorno transborda o fill), escalada pelo afim.
/// `None` se o caminho não existe ou não tem geometria que desenhe algo.
#[must_use]
pub fn path_screen_bounds(
    scene: &VecScene,
    xforms: &VecXforms,
    live: &LiveGeometry,
    id: VecPathId,
    camera: Affine,
) -> Option<(f64, f64, f64, f64)> {
    let mut acc: Option<Rect> = None;
    let mut eat = |path: &VecPath, xf: Affine| {
        let mut bp = build_bezpath(path);
        if bp.elements().is_empty() {
            return;
        }
        bp.apply_affine(xf);
        let mut r = bp.bounding_box();
        // O traço transborda o fill por metade da largura; escala com o afim.
        //
        // ⚠️ **E uma junta MITER vai MUITO além disso.** Numa quina de ângulo interno `θ` a ponta do
        // miter fica a `½w / sin(θ/2)` do vértice — numa ponta de estrela (36°), **3,24 × ½w** —, e
        // a kurbo só a corta no `miter_limit`. Inflar por meia largura recortava a ponta contra a
        // borda do scratch, e o efeito visível era **a ponta CEIFADA** (reportado no smoke). O
        // limite é lido do MESMO construtor que o renderer usa, não de uma segunda constante.
        if let Some(s) = path.stroke {
            let [a, b, c, d, _, _] = xf.as_coeffs();
            let sx = (a * a + b * b).sqrt();
            let sy = (c * c + d * d).sqrt();
            let k = kurbo_stroke(&s);
            let reach = if matches!(k.join, Join::Miter) {
                k.miter_limit.max(1.0)
            } else {
                1.0
            };
            let m = 0.5 * s.width * sx.max(sy) * reach;
            r = r.inflate(m, m);
        }
        acc = Some(acc.map_or(r, |cur| cur.union(r)));
    };
    if let Some(items) = live.get(&id) {
        // Derivada já em MUNDO ⇒ sobe pela câmera (como no `dispatch`).
        for item in items {
            eat(item, camera);
        }
    } else {
        let path = scene.paths().iter().find(|p| p.id == id)?;
        eat(path, path_to_screen(xforms, id, camera));
    }
    let r = acc?;
    Some((r.x0, r.y0, r.x1, r.y1))
}

/// **Desenha exatamente UM caminho, como o [`dispatch`] o desenharia, transladado por `offset`
/// (px de tela)** — a rasterização da forma isolada que o produtor de FX (plano 24) lê de volta.
///
/// Honra a geometria DERIVADA (`live`) e a pose, igual ao `dispatch`, então o que o FX borra é
/// exatamente o que a forma É na tela; o `offset` leva o bbox da forma à origem `(0,0)` do scratch.
/// Passa pela MESMA [`draw_path`] do `dispatch` — desenhar por uma 2ª porta faria o FX divergir do
/// que a forma parece de verdade.
pub fn draw_path_isolated(
    scene: &VecScene,
    xforms: &VecXforms,
    live: &LiveGeometry,
    id: VecPathId,
    camera: Affine,
    offset: Affine,
    target: &mut VectorScene,
) {
    if let Some(items) = live.get(&id) {
        for item in items {
            draw_path(item, offset * camera, target);
        }
    } else if let Some(path) = scene.paths().iter().find(|p| p.id == id) {
        draw_path(path, offset * path_to_screen(xforms, id, camera), target);
    }
}

/// Desenha UM path já posicionado — o `transform` leva a geometria dele à tela. Fill primeiro,
/// stroke por cima; pontas por último.
///
/// É o corpo de um item de [`dispatch`], **extraído de propósito**: os passos VIRTUAIS de um
/// Blend Object (ADR-0128, [`draw_blend_overlay`]) não estão na cena, mas são arte de verdade — e
/// desenhá-los por uma segunda porta faria a transição divergir do que a MESMA forma pareceria
/// como path real ([[feedback_two_doors_to_the_same_question_diverge]]). Os dois passam por AQUI.
///
/// Ao contrário dos overlays (gizmos, véu do Build), aqui a espessura do traço **escala com o
/// mundo** — é o contorno de uma forma, como o de um sprite ampliado, não uma borda de px.
pub(crate) fn draw_path(path: &VecPath, transform: Affine, target: &mut VectorScene) {
    // ⚠️ **UM cozimento por forma, e nada é construído para quem não vai desenhar.** A versão
    // anterior fazia `build_bezpath` INCONDICIONALMENTE e depois `build_fill_bezpath`: numa forma
    // só-preenchida (a arte comum, e a cena inteira do spike de escala) o primeiro era construído
    // e **jogado fora**, e numa forma preenchida-e-traçada sem contorno aberto os dois eram o
    // MESMO desenho. Medido no `encode_cost_by_n`: 10k formas custavam 1,323 ms/frame de re-encode.
    let cooked = path.cooked();
    #[cfg(test)]
    encode_cost_tests::count_cook();
    // Há contorno ABERTO? É a única coisa que faz o desenho do traço diferir do do preenchimento.
    let open =
        (0..cooked.contour_count()).any(|c| cooked.contour(c).is_some_and(|(_, closed)| !closed));
    // O preenchimento ignora os contornos ABERTOS (linhas de construção — as arestas internas do
    // cubo, a tampa do cilindro): eles não têm interior, e fechá-los implicitamente recorta a
    // silhueta. Ver [`build_fill_bezpath`].
    let fill_bp = path
        .fill
        .is_some()
        .then(|| build_contours(&cooked, Some(true)));
    // O traço leva TODOS os contornos. Sem contorno aberto ele é **o mesmo desenho** do
    // preenchimento ⇒ os dois compartilham uma construção só; com contorno aberto são dois
    // desenhos diferentes e as duas construções são trabalho honesto.
    let stroke_own = (path.stroke.is_some() && (open || fill_bp.is_none()))
        .then(|| build_contours(&cooked, None));
    if let Some(fill) = &path.fill {
        let fp = fill_bp.as_ref().expect("fill => fill_bp construido");
        if let Paint::MultiPoint { points } = fill {
            // ⚠️ `path`, não `cooked`: o `fill_multipoint` mede a caixa dos pontos de controle da
            // forma AUTORADA (`control_point_bounds`). Passar o cozido moveria o gradiente de toda
            // forma com quina viva ou efeito — mudança de aparência dentro de um fix de custo.
            fill_multipoint(target, fp, path, points, transform);
        } else {
            // `VectorScene::fill_path` assume NonZero; um compound precisa da
            // regra do path (EvenOdd vaza o contorno de dentro).
            target.inner_mut().fill(
                fill_rule(path),
                transform,
                &fill_brush(fill, path),
                None,
                fp,
            );
        }
    }
    if let Some(s) = path.stroke {
        let bp = stroke_own
            .as_ref()
            .or(fill_bp.as_ref())
            .expect("stroke => um dos dois desenhos existe");
        // O QUE um traço desenha é decidido em `ph2d_vec_scene::stroke_plan` — a porta
        // única, que o Outline Stroke também consome. Aqui só se PINTA o que ela lista.
        let brush = Brush::Solid(color(s.color));
        for piece in ph2d_vec_scene::stroke_plan(path, &s) {
            match piece {
                StrokePiece::Line { path: line } => {
                    // Emprestado = a peça É o path, e `bp` já o descreve. É o caso de 99%
                    // dos traços, e o motivo de o plano devolver `Cow`.
                    let line_bp = match line {
                        Cow::Borrowed(_) => bp.clone(),
                        Cow::Owned(p) => build_bezpath(&p),
                    };
                    target
                        .inner_mut()
                        .stroke(&kurbo_stroke(&s), transform, &brush, None, &line_bp);
                }
                StrokePiece::Symbol { path: geo } => {
                    target.inner_mut().stroke(
                        &Stroke::new(s.width),
                        transform,
                        &brush,
                        None,
                        &build_bezpath(&geo),
                    );
                }
                StrokePiece::Fill { path: geo } => {
                    target.inner_mut().fill(
                        Fill::NonZero,
                        transform,
                        &brush,
                        None,
                        &build_bezpath(&geo),
                    );
                }
            }
        }
    }
}

/// Desenha os **gizmos de edição** por cima da cena (screen-space, tamanho
/// constante em px): quadradinho em cada âncora; e — só no path `selected` — as
/// linhas âncora→handle + bolinhas nos handles dos vértices suaves. Cores
/// hardcoded de scaffold (Fase 1); migram p/ tokens no chrome do cutover (Fase R).
#[allow(clippy::too_many_arguments)]
pub fn draw_overlays(
    scene: &VecScene,
    view: &VecViewState,
    selected: Option<VecPathId>,
    selected_paths: &[VecPathId],
    selected_verts: &[usize],
    xforms: &VecXforms,
    camera: Affine,
    target: &mut VectorScene,
) {
    for path in scene.paths() {
        if view.is_hidden(path.id) {
            continue; // um path escondido não mostra âncoras
        }
        // Âncoras e handles são LOCAIS: passam pelo Transform do path antes da câmera.
        let transform = path_to_screen(xforms, path.id, camera);
        let is_sel = Some(path.id) == selected;
        // Any path in the OBJECT selection set is highlighted; the primary also shows
        // its Bézier handles + per-vertex picks.
        let in_set = selected_paths.contains(&path.id);
        if is_sel {
            for (i, v) in path.verts_all().enumerate() {
                let a = transform * Point::new(v.anchor[0], v.anchor[1]);
                // Draw each handle at its DISPLAY position: a real (offset) handle
                // as-is, and a Smooth/Symmetric point's zero-length ("invisible")
                // handle as a grabbable GHOST stub along the smooth tangent — without
                // touching geometry (the curve only changes when the user drags it).
                // A straight Corner's zero handle stays hidden (`ghost_handle` = None).
                for out in [false, true] {
                    let real = if out { v.out_handle } else { v.in_handle };
                    let (dx, dy) = (real[0] - v.anchor[0], real[1] - v.anchor[1]);
                    let is_ghost = dx * dx + dy * dy <= 1e-18;
                    let Some(h) = ph2d_vec_scene::ghost_handle(path, i, out) else {
                        continue;
                    };
                    let hp = transform * Point::new(h[0], h[1]);
                    let mut line = BezPath::new();
                    line.move_to(a);
                    line.line_to(hp);
                    // Ghost stubs are dimmer + hollow so the user reads them as
                    // "suggested, not yet affecting the curve".
                    let line_a = if is_ghost { 120 } else { 200 };
                    target.inner_mut().stroke(
                        &Stroke::new(1.0),
                        Affine::IDENTITY,
                        &Brush::Solid(Color::from_rgba8(120, 190, 230, line_a)),
                        None,
                        &line,
                    );
                    if is_ghost {
                        target.inner_mut().stroke(
                            &Stroke::new(1.2),
                            Affine::IDENTITY,
                            &Brush::Solid(Color::from_rgba8(120, 190, 230, 220)),
                            None,
                            &Circle::new(hp, 3.5),
                        );
                    } else {
                        target.inner_mut().fill(
                            Fill::NonZero,
                            Affine::IDENTITY,
                            &Brush::Solid(Color::from_rgba8(120, 190, 230, 255)),
                            None,
                            &Circle::new(hp, 3.5),
                        );
                    }
                }
            }
        }
        // Flat index across contours — the same space `hit_test` / `selected_verts`
        // address, so a hole's anchors pick exactly like the outer contour's.
        for (i, v) in path.verts_all().enumerate() {
            let a = transform * Point::new(v.anchor[0], v.anchor[1]);
            // A vertex in the multi-selection (selected path only) is drawn bigger
            // + cyan; other anchors of the selected path are orange; other paths gray.
            let picked = is_sel && selected_verts.contains(&i);
            let s = if picked { 4.5 } else { 3.5 };
            let col = if picked {
                Color::from_rgba8(90, 200, 235, 255) // ciano = vértice selecionado (grupo)
            } else if in_set {
                Color::from_rgba8(250, 180, 90, 255) // laranja = path na seleção de objeto
            } else {
                Color::from_rgba8(230, 230, 235, 220)
            };
            target.fill_rect(Rect::new(a.x - s, a.y - s, a.x + s, a.y + s), col);
        }
    }
}

/// Desenha a caixa de **marquee** (box-select) em **screen-space** (o shell
/// passa cantos de tela): preenchimento translúcido + contorno. Chamada só
/// enquanto o Shift+arrasto está ativo.
pub fn draw_marquee(min: [f64; 2], max: [f64; 2], target: &mut VectorScene) {
    let (x0, x1) = (min[0].min(max[0]), min[0].max(max[0]));
    let (y0, y1) = (min[1].min(max[1]), min[1].max(max[1]));
    let rect = Rect::new(x0, y0, x1, y1);
    target.inner_mut().fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Solid(Color::from_rgba8(90, 200, 235, 40)),
        None,
        &rect,
    );
    let mut outline = BezPath::new();
    outline.move_to(Point::new(x0, y0));
    outline.line_to(Point::new(x1, y0));
    outline.line_to(Point::new(x1, y1));
    outline.line_to(Point::new(x0, y1));
    outline.close_path();
    target.inner_mut().stroke(
        &Stroke::new(1.0),
        Affine::IDENTITY,
        &Brush::Solid(Color::from_rgba8(90, 200, 235, 200)),
        None,
        &outline,
    );
}

/// `StrokeSpec` → `kurbo::Stroke` (ponta/junção + dash). Larguras/dashes ficam em
/// world-units; o `transform` do render escala p/ screen.
pub(crate) fn kurbo_stroke(s: &StrokeSpec) -> Stroke {
    let cap = match s.cap {
        LineCap::Butt => Cap::Butt,
        LineCap::Round => Cap::Round,
        LineCap::Square => Cap::Square,
    };
    let join = match s.join {
        LineJoin::Miter => Join::Miter,
        LineJoin::Round => Join::Round,
        LineJoin::Bevel => Join::Bevel,
    };
    let stroke = Stroke::new(s.width).with_caps(cap).with_join(join);
    // Os comprimentos vêm do `StrokeSpec` (que guarda MÚLTIPLOS da largura) — porta única,
    // porque o Outline Stroke assa o mesmo tracejado com outra versão da kurbo.
    match s.dash_lengths() {
        Some(d) => stroke.with_dashes(0.0, d),
        None => stroke,
    }
}

#[inline]
fn pt(p: [f64; 2]) -> Point {
    Point::new(p[0], p[1])
}

#[inline]
pub(crate) fn color(c: Rgba8) -> Color {
    Color::from_rgba8(c.r, c.g, c.b, c.a)
}

/// Peniko color stops from our gradient stops (`(offset f32, Color)` → `ColorStop`),
/// SORTED by offset — interior stops may cross one another in the editor (their Vec
/// order isn't guaranteed monotonic), but peniko wants non-decreasing offsets.
fn stops_of(stops: &[ph2d_vec_scene::GradientStop]) -> Vec<ColorStop> {
    let mut out: Vec<ColorStop> = stops
        .iter()
        .map(|s| ColorStop::from((s.offset as f32, color(s.color))))
        .collect();
    out.sort_by(|a, b| {
        a.offset
            .partial_cmp(&b.offset)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    out
}

/// Build the Vello fill brush for a path's [`Paint`]. Linear/Radial map to native
/// peniko gradients using the paint's OWN world-space geometry (start/end,
/// center/radius) — which transforms rigidly with the path, so the gradient never
/// "breathes" under rotation. The frame's world→screen transform maps them.
/// MultiPoint is handled by `fill_multipoint` (image-clip path), never here.
fn fill_brush(paint: &Paint, _path: &VecPath) -> Brush {
    match paint {
        Paint::Solid(c) => Brush::Solid(color(*c)),
        Paint::Linear { stops, start, end } => {
            let a = Point::new(start[0], start[1]);
            let b = Point::new(end[0], end[1]);
            Brush::Gradient(Gradient::new_linear(a, b).with_stops(stops_of(stops).as_slice()))
        }
        Paint::Radial {
            stops,
            center,
            radius,
        } => {
            let c = Point::new(center[0], center[1]);
            let r = (*radius as f32).max(f32::MIN_POSITIVE);
            Brush::Gradient(Gradient::new_radial(c, r).with_stops(stops_of(stops).as_slice()))
        }
        // MultiPoint is handled by `fill_multipoint` (image-clip path), never here.
        Paint::MultiPoint { .. } => Brush::Solid(color(paint.primary_color())),
    }
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;

/// **O orçamento do re-encode por frame** — arquivo irmão. O spike de escala vivia num
/// `println!` de teste `#[ignore]`, e uma regressão de 1,7× atravessou meses sem ninguém ver.
#[cfg(test)]
#[path = "encode_cost_tests.rs"]
mod encode_cost_tests;

/// Os gates da largura ZERO (o slider chega a 0 = sem traço) — arquivo irmão.
#[cfg(test)]
#[path = "stroke_zero_tests.rs"]
mod stroke_zero_tests;

#[cfg(test)]
mod open_contour_tests {
    use super::*;
    use ph2d_vector::Shape;

    /// **O triângulo escuro do cubo, executável.**
    ///
    /// O cubo isométrico é uma silhueta hexagonal FECHADA + três arestas internas, que são
    /// contornos ABERTOS. Preencher tudo junto faz o rasterizador **fechar cada contorno
    /// aberto implicitamente** (é a semântica de fill, em qualquer engine): a corda que
    /// fecha a polilinha `V1 → M → V3` vira uma região de winding próprio que, com
    /// `NonZero`, CANCELA o hexágono onde coincide. O Enio fotografou exatamente isso — um
    /// triângulo escuro comendo metade da face direita.
    ///
    /// O teste mede o winding NO PONTO do triângulo. Ele prova as duas metades da história:
    /// o path do preenchimento cobre a face (winding ≠ 0) **e** o path completo — que era o
    /// que se preenchia antes — a perfura (winding = 0). Se alguém reverter a correção, a
    /// segunda asserção passa a valer para o primeiro path e o teste cai.
    #[test]
    fn an_open_contour_never_punches_a_hole_in_the_fill() {
        let cube = ph2d_vec_scene::iso_cube([-1.0, -1.0], [1.0, 1.0], 0.5, 0.5, false);
        // As três arestas internas vivem no sub-contorno; o vértice central é o do meio.
        let inner = &cube.subpaths[0].verts;
        let (v1, m, v3) = (inner[0].anchor, inner[1].anchor, inner[2].anchor);
        // Um ponto BEM dentro do triângulo V1–M–V3: o baricentro. É o miolo da mancha.
        let p = Point::new((v1[0] + m[0] + v3[0]) / 3.0, (v1[1] + m[1] + v3[1]) / 3.0);

        let fill = build_fill_bezpath(&cube);
        assert_ne!(
            fill.winding(p),
            0,
            "a face direita do cubo tem de ser PREENCHIDA em {p:?} — o triangulo escuro voltou"
        );

        // E a prova de que o bug era real: com os contornos abertos dentro do preenchimento,
        // o mesmo ponto FICA DE FORA. (É o que se pintava antes.)
        let everything = build_bezpath(&cube);
        assert_eq!(
            everything.winding(p),
            0,
            "o teste perdeu o poder de discriminar: o contorno aberto deveria furar o fill"
        );

        // As linhas de construção são exatamente as arestas internas — e não estão no fill.
        assert!(
            !build_lines_bezpath(&cube).is_empty(),
            "as arestas internas do cubo TEM de ser desenhadas (senao e um hexagono)"
        );
    }

    /// A regra vale para toda forma do catálogo: o path do preenchimento nunca contém um
    /// contorno aberto, e a soma dos dois caminhos é o path inteiro.
    #[test]
    fn fill_and_lines_partition_every_shape_in_the_catalogue() {
        for &kind in ph2d_vec_scene::ALL_SHAPES {
            let path = ph2d_vec_scene::cook(kind, [-1.0, -1.0], [1.0, 1.0], &kind.defaults());
            let (fill, lines, whole) = (
                build_fill_bezpath(&path),
                build_lines_bezpath(&path),
                build_bezpath(&path),
            );
            assert_eq!(
                fill.elements().len() + lines.elements().len(),
                whole.elements().len(),
                "{kind:?}: fill + linhas tem de particionar o path inteiro"
            );
            // Uma forma ABERTA (linha, arco, espiral, chave) não tem nada a preencher.
            if !kind.is_closed() {
                assert!(
                    fill.is_empty(),
                    "{kind:?} e aberta — nao tem interior para preencher"
                );
            }
        }
    }
}
