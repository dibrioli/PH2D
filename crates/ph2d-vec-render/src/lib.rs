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

/// ⚠️ **Só os GATES o nomeiam agora**: o desenho do documento mudou-se para o [`dispatch`], e este
/// ficheiro deixou de o mencionar. Sem o `cfg(test)` a importação seria um aviso de não-usado; sem
/// a importação, os testes deste ficheiro deixam de compilar.
#[cfg(test)]
use ph2d_vec_scene::VecViewState;
use ph2d_vec_scene::{
    BoundStyle, FillRule as VecFillRule, LineCap, LineJoin, Paint, Rgba8, StrokeSpec, VecPath,
    VecPathId, VecScene, VecXforms,
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

/// **O preenchimento com PADRÃO** (plano 33) — a tradução do modo e a chamada da porta de imagem.
pub mod pattern;

#[cfg(test)]
#[path = "pattern_stroke_tests.rs"]
mod pattern_stroke_tests;
#[cfg(test)]
mod pattern_tests;
/// Os gates da estampa do traço sob a POSE da entidade — irmão do [`pattern_stroke_tests`] pelo
/// teto de LOC, e o corte é por SUJEITO: ali a estampa é a tinta de uma faixa, aqui é o que lhe
/// acontece quando a forma tem um `Transform` não-uniforme.
#[cfg(test)]
mod stroke_pattern_pose_tests;
pub use gradient::{GradHandle, drag_gradient_handle, draw_gradient_handles, hit_gradient_handle};

/// **O recorte da moldura** (plano UI/UX W0) — irmão pelo teto de LOC, e a única peça que sabe que
/// um caminho pode CONTER os que vêm depois dele na pilha de z.
mod frame_clip;

/// Smart guides (o feedback visual do snap), likewise a sibling.
mod cut_line;

/// **O que o GESTO desenha** — o retângulo e o laço da região em curso. Irmão pelo teto de 700
/// LOC, e o corte é por assunto: tudo o mais neste arquivo desenha o que o DOCUMENTO é (ou as
/// alças que o editam), e estes dois desenham uma coisa que ainda não existe — a região que a mão
/// está a delimitar, em px de tela, e que some ao soltar.
mod hover_outline;
mod marquee;
mod standalone;
mod stroke_uniform;
pub use cut_line::draw_cut_line;
pub use hover_outline::{draw_bucket_face, draw_hover_outline, draw_trim_piece};
pub use marquee::{draw_lasso, draw_marquee};
pub use standalone::{draw_path_isolated, draw_path_standalone};

/// A tradução do modo de mistura do documento para o par do Vello (módulo irmão: ele é a fonte da
/// lista que o painel oferece, e o `dispatch` é só um dos consumidores).
pub mod blend;
pub use stroke_uniform::{is_conformal, stroke_uniform, uniform_scale};
mod guides;
pub use guides::{
    Guide, GuideKind, GuideLabel, draw_document_guides, draw_snap_guides, draw_text_caret,
    snap_labels,
};
/// As linhas de SIMETRIA no canvas (plano 25 W6.3) — irmão de `guides` e consumidor do mesmo
/// recortador de reta: um eixo e uma guia atravessam a tela pela mesma aritmética.
mod symmetry_overlay;
pub use symmetry_overlay::{SymmetryAxis, draw_symmetry_axes};

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

// ⛔ **O desenho dos OSSOS mudou-se em 2026-09-06** para a `ph2d-skeleton-render`, quando o
// esqueleto virou módulo próprio: ele nunca importou a cena vectorial (só a stack de desenho e os
// tokens), então um esqueleto sobre uma imagem ou sobre uma malha desenha-se com o MESMO código.
// ⇒ `ph2d_skeleton_render::{draw_bones, joint_radius_px}`.

/// A **alça do texto em caminho** (plano 22, W5) — módulo irmão (LOC cap). A bolinha onde o
/// texto começa no caminho; arrastá-la corre o texto ao longo dele.
mod text_handle;
pub use text_handle::draw_text_handle;
mod width_handle;
pub use width_handle::draw_width_handle;
mod weld_mark;
pub use weld_mark::draw_weld_marks;

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
pub use build::{build_bezpath, build_contours, build_fill_bezpath, build_lines_bezpath};

/// **A METADE DO TRAÇO** — módulo irmão pelo teto de LOC, e o corte por RESPONSABILIDADE já estava
/// escrito no doc-comment da função: quem traça passa por uma porta só, incluindo a rota de
/// instância de Motion.
mod stroke_draw;
pub(crate) use stroke_draw::draw_stroke_with;

/// **AS CAIXAS em px de tela** — módulo irmão pelo teto de LOC, e o corte é por RESPONSABILIDADE:
/// ali ninguém desenha, só se pergunta *onde, na tela, esta forma vive*.
mod path_bounds;
pub use path_bounds::{path_bounds_under, path_screen_bounds, standalone_path_screen_bounds};

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

/// **O LADRILHO ASSADO de um padrão** neste quadro — runtime-only, como a [`FxImage`].
///
/// O documento guarda a RELAÇÃO (qual arte, que reticulado, que tamanho); isto é o desenho
/// derivado dela. ⚠️ Carrega um [`ph2d_vector::StableImage`] pela MESMA razão que a `FxImage`: o
/// id de Blob tem de ser **estável** para o Vello reusar a textura do atlas em vez de a re-enviar a
/// cada quadro. Quem o produz constrói-o UMA vez (no memo) e clona o handle por quadro.
#[derive(Clone)]
pub struct PatternTile {
    /// O rectângulo que a GPU repete.
    pub image: ph2d_vector::StableImage,
    /// Quantas células ele contém (`[1,1]` na grade, `[1,n]` no tijolo por linha) — o
    /// [`ph2d_vec_scene::PatternFill::placement`] precisa disto para saber quantos PERÍODOS o
    /// rectângulo cobre.
    pub cells: [u32; 2],
    /// A resolução do assado, em pixels.
    pub tile_px: [u32; 2],
    /// O filtro de amostragem, **derivado do modo de imagem do app** (PixelArt -> `Low`).
    ///
    /// ⚠️ Ele vem de fora porque esta crate não conhece as preferências do app — a mesma razão pela
    /// qual o `draw_image_rgba` já o recebe em vez de o adivinhar.
    pub quality: ph2d_vector::ImageQuality,
    /// ⭐⭐⭐ **O SALTO deste ladrilho na volta** ([`ph2d_vec_pattern::wrap_seam`], plano 33 W10) — o
    /// maior degrau que aparece quando ele encosta numa cópia de si mesmo. `0` = fecha exactamente.
    ///
    /// ⚠️ **Mede-se no ASSADO, não na arte**, e a diferença é o produto: um vão positivo separa as
    /// cópias com espaço transparente, então a mesma arte que não encaixa colada **encaixa** com vão.
    /// A pergunta do artista é sobre o que ele vê, e o que ele vê é o ladrilho.
    ///
    /// ⚠️ Viaja aqui porque é medido **uma vez**, no assado — e o assado é memoizado. Recalculá-lo
    /// por quadro seria varrer o perímetro de todo ladrilho da cena para responder à mesma coisa.
    pub wrap_seam: u8,
}

/// ⭐ **QUAL das duas tintas de uma forma** este ladrilho serve (plano 35, wave B).
///
/// ⚠️ Uma forma pode ter padrão no preenchimento **e** no traço, e são dois `PatternFill`
/// independentes ⇒ o mapa não pode ser indexado só pela forma. *Uma chave que não distingue os dois
/// sujeitos entrega o ladrilho do preenchimento ao traço, e o desenho fica certo por acidente
/// enquanto os dois forem iguais.*
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PatternSlot {
    /// O padrão do `Paint::Pattern` do preenchimento.
    Fill,
    /// O padrão do `StrokePaint::Pattern` do traço.
    Stroke,
}

/// Os ladrilhos de padrão deste quadro, por forma **e por slot**. Vazio = nenhum padrão resolvido, e
/// toda tinta de padrão pinta a `fallback` dela — que é desenho CERTO, não uma desistência.
pub type PatternTiles = std::collections::BTreeMap<(VecPathId, PatternSlot), PatternTile>;

/// ⭐⭐⭐ **A ARTE COZIDA de cada PINCEL deste quadro**, pela forma HOSPEDEIRA (plano 36, W3).
///
/// ⚠️ **Resolvida pela shell, como o ladrilho do padrão** — e pela mesma razão: esta crate não
/// alcança a cena, e ir buscar a forma-fonte aqui dentro poria a resolução (com o guarda de ciclo,
/// a geometria viva e o cozimento) num sítio que não a pode medir.
///
/// Vazio = nenhum pincel resolvido, e todo traço de pincel pinta a **cor de recurso** dele — que é
/// desenho CERTO, não desistência.
/// ⭐⭐⭐ **A arte de um pincel é uma LISTA**, porque ela pode ser um GRUPO (report do Enio,
/// 2026-08-30, na estampa; o pincel é a mesma metade noutra tinta).
///
/// ⛔ **Fundir os membros num `VecPath` colapsaria as tintas**: cada cópia carrega o `fill`/`stroke`
/// do SEU motivo, e um `VecPath` tem um de cada — um triângulo azul com um círculo laranja sairia
/// de uma cor só. ⇒ a lista, e um referencial partilhado
/// ([`ph2d_vec_scene::pattern_path::motif_frame`]) para que os membros mantenham a disposição.
pub type BrushArts = std::collections::BTreeMap<VecPathId, Vec<ph2d_vec_scene::VecPath>>;

/// ⭐ **Os artefactos DERIVADOS que a shell coze por quadro** — irmão pelo tecto de 700 LOC, e o
/// corte é por RESPONSABILIDADE: aqui fica o motor de desenho; ali, o vocabulário de *"o que esta
/// forma recebeu de fora neste quadro"*.
#[path = "derived.rs"]
mod derived;
pub(crate) use derived::Derived;
pub use derived::DilatedPaints;

/// Os FX raster deste frame, por forma. Vazio = nenhum FX na cena, e o desenho é o de sempre —
/// **byte-idêntico** ao mundo pré-FX (o caminho comum não paga nada).
pub type FxImages = std::collections::BTreeMap<VecPathId, FxImage>;

/// As PELES de widget deste frame, por forma (plano UI/UX W6.2). Vazio = nenhuma forma veste um
/// widget, e o desenho é o de sempre — **byte-idêntico** ao mundo pré-pele.
///
/// ⚠️ **O fragmento é OPACO de propósito.** Esta crate não sabe o que é um botão e não pode saber:
/// o catálogo mora na `ph2d-editor-core`, que é UI, e a seta dela para cá seria a errada. O shell
/// — que alcança as duas — pinta o widget pelo **pintor REAL** numa cena de rascunho e a entrega
/// pronta; aqui ela só é anexada no **z da forma**, exactamente como uma [`FxImage`].
///
/// ⚠️ E ela **substitui** o desenho, não o acompanha: uma forma que veste um widget é a MOLDURA
/// dele (onde e que tamanho), não uma silhueta a desenhar por baixo — pintar as duas mostraria o
/// retângulo do artista sangrando pelas bordas do controle.
pub type WidgetSkins = std::collections::BTreeMap<VecPathId, ph2d_vector::VectorScene>;

/// ⭐⭐⭐ **A porta que desenha o DOCUMENTO** — as duas metades (o mundo · a receita aberta).
mod dispatch;
pub use dispatch::{dispatch, dispatch_isolated};

/// **A geometria TESSELADA de um path** — os `BezPath`s que o [`draw_path`] constrói de `cooked()`
/// (o preenchimento e, quando difere, o traço), extraídos para que quem desenha a MESMA geometria
/// N vezes (N instâncias de um objeto/forma de Motion) a construa UMA vez e reuse, em vez de
/// re-tesselar por instância (o congelamento das 160k estrelas, ADR-0154). Opaca de propósito: só
/// [`tessellate_shape_instance`] a produz e [`draw_shape_instance_tessellated`] a consome.
pub(crate) struct PathTess {
    /// Os contornos FECHADOS (o preenchimento). `None` quando o path não tem `fill`.
    fill_bp: Option<BezPath>,
    /// O `stroke_own`: TODOS os contornos, presente **só quando o desenho do traço DIFERE do
    /// preenchimento** (há contorno aberto, ou não há fill). `None` ⇒ o traço reusa `fill_bp`.
    stroke_bp: Option<BezPath>,
    /// O tracejado JÁ AJUSTADO ao comprimento deste caminho ([`ph2d_vec_scene::dash_fit`]),
    /// ou `None` para linha contínua.
    ///
    /// ⚠️ **Mora aqui porque o ajuste é do caminho COZIDO**, e o cozimento é o que esta
    /// estrutura já pagou. Quem desenha só tem o `tess` em mão; se ele tivesse de cozer outra
    /// vez para medir, o ajuste custaria uma pilha de efeitos por instância — ou, pior,
    /// alguém mediria o caminho de ORIGEM e o tracejado sairia noutra cadência que o desenho.
    dash: Option<[f64; 2]>,
}

/// Constrói a [`PathTess`] de um path: coze UMA vez e tessela o(s) `BezPath`(s) que o desenho
/// precisa. É a metade CARA do [`draw_path`] — a única que roda `cooked()` + `build_contours` —, e
/// separá-la é o que deixa um lote de instâncias da mesma geometria pagá-la uma vez ([`PathTess`]).
///
/// ⚠️ **UM cozimento por forma, e nada é construído para quem não vai desenhar.** A versão
/// anterior fazia `build_bezpath` INCONDICIONALMENTE e depois `build_fill_bezpath`: numa forma
/// só-preenchida (a arte comum, e a cena inteira do spike de escala) o primeiro era construído
/// e **jogado fora**, e numa forma preenchida-e-traçada sem contorno aberto os dois eram o
/// MESMO desenho. Medido no `encode_cost_by_n`: 10k formas custavam 1,323 ms/frame de re-encode.
pub(crate) fn path_tess(path: &VecPath) -> PathTess {
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
    let stroke_bp = (path.stroke.is_some() && (open || fill_bp.is_none()))
        .then(|| build_contours(&cooked, None));
    PathTess {
        fill_bp,
        stroke_bp,
        dash: dash_of(&cooked, path.stroke.as_ref()),
    }
}

/// O tracejado AJUSTADO deste caminho cozido — `None` sem traço ou sem tracejado.
///
/// ⚠️ Mede o COZIDO (um Trim muda o comprimento) e só mede quando há tracejado: a medição é
/// um `arclen` por segmento, e uma linha contínua não tem o que ajustar.
pub(crate) fn dash_of(cooked: &VecPath, stroke: Option<&StrokeSpec>) -> Option<[f64; 2]> {
    ph2d_vec_scene::dash_fit::dash_lengths_for(cooked, stroke?)
}

/// Desenha UM path já posicionado — o `transform` leva a geometria dele à tela. Fill primeiro,
/// stroke por cima; pontas por último.
///
/// É o corpo de um item de [`dispatch`], **extraído de propósito**: os passos VIRTUAIS de um
/// Blend Object (ADR-0128, [`draw_blend_overlay`]) não estão na cena, mas são arte de verdade — e
/// desenhá-los por uma segunda porta faria a transição divergir do que a MESMA forma pareceria
/// como path real ([[feedback_two_doors_to_the_same_question_diverge]]). Os dois passam por AQUI.
///
/// Tessela sua própria geometria ([`path_tess`]) e delega a [`draw_path_with`] — byte-idêntico ao
/// desenho de antes: 1 cozimento + as construções de sempre por chamada.
pub(crate) fn draw_path(path: &VecPath, transform: Affine, target: &mut VectorScene) {
    draw_path_tiled(path, transform, target, Derived::NONE);
}

/// Igual a [`draw_path`], mas com o LADRILHO de padrão desta forma neste quadro.
///
/// ⚠️ **Duas portas para a mesma pergunta divergem** — é por isso que a `draw_path` delega aqui em
/// vez de ter um corpo próprio: um ladrilho ausente é *literalmente* o `None`, e não outro caminho.
pub(crate) fn draw_path_tiled(
    path: &VecPath,
    transform: Affine,
    target: &mut VectorScene,
    derived: Derived<'_>,
) {
    let tess = path_tess(path);
    draw_path_with(path, &tess, transform, target, derived);
}

/// Desenha um path a partir da geometria JÁ TESSELADA (`tess`) — a metade barata do [`draw_path`],
/// que só emite os comandos Vello (`fill`/`stroke`) e não constrói nada. É por AQUI que um lote de
/// instâncias da mesma geometria desenha, cada uma com o próprio `transform`, sem re-tesselar.
///
/// Ao contrário dos overlays (gizmos, véu do Build), aqui a espessura do traço **escala com o
/// mundo** — é o contorno de uma forma, como o de um sprite ampliado, não uma borda de px.
pub(crate) fn draw_path_with(
    path: &VecPath,
    tess: &PathTess,
    transform: Affine,
    target: &mut VectorScene,
    derived: Derived<'_>,
) {
    let Derived {
        tile,
        stroke_tile,
        brush_art,
        dilated,
    } = derived;
    let fill_bp = tess.fill_bp.as_ref();
    if let Some(fill) = &path.fill {
        let fp = fill_bp.expect("fill => fill_bp construido");
        // ⭐ **O padrão desenha a IMAGEM quando o ladrilho existe, e a `fallback` quando não.**
        // As duas metades são desenho certo: a segunda é o que o artista vê enquanto a arte carrega,
        // ou quando a forma-fonte desapareceu — e ⛔ desenhar NADA seria pior (uma forma invisível
        // não se distingue de um preenchimento vazio).
        if let (Paint::Pattern(pat), Some(t)) = (fill, tile) {
            pattern::fill_pattern(
                target,
                fp,
                fill_rule(path),
                transform,
                &t.image,
                // ⚠️ A caixa vem do desenho de PREENCHIMENTO (espaço das âncoras, como a
                // colocação): é ela que o `Clamp` enquadra. `bounding_box` de um `BezPath` vazio
                // devolve uma caixa degenerada, e o `placement_in` recusa-a e cai na autorada.
                pat.placement_in(t.cells, t.tile_px, {
                    let b = fp.bounding_box();
                    ([b.x0, b.y0], [b.x1, b.y1])
                }),
                pat.mode,
                t.quality,
                pat.alpha,
            );
        } else if let Paint::MultiPoint { points } = fill {
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
    draw_stroke_with(path, tess, transform, target, stroke_tile, brush_art);
    // ⭐⭐⭐ **E A PILHA DE APARÊNCIA POR CIMA** (v20) — as camadas extras, de baixo para cima,
    // reusando esta mesma tesselação. No-op sem pilha, que é o caminho comum.
    stack_draw::draw_extra_paints(path, tess, transform, target, dilated);
}

mod instance;
/// **A camada de INSTÂNCIA de Motion** — módulo irmão pelo teto de 700 LOC. O corte é por assunto:
/// aqui o motor de path (acima); ali o desenho de UMA instância de Motion e o LOTE que compartilha
/// geometria, tesselando cada handle uma vez (o congelamento das 160k estrelas, ADR-0154).
/// ⭐ **A PILHA DE APARÊNCIA** (v20) — módulo irmão pelo tecto de LOC: aqui desenha-se o CHÃO de
/// uma forma, ali o que está por cima dele.
mod stack_draw;
pub use instance::{draw_shape_instance, draw_shared_instances};

/// **O overlay de EDIÇÃO** (as âncoras, os handles) — módulo irmão pelo teto de 700 LOC. O corte
/// é por assunto: aqui o desenho da ARTE, ali o dos controles que a editam.
mod overlays;
pub use overlays::{draw_overlays, overlay_transform};

/// `StrokeSpec` → `kurbo::Stroke` (ponta/junção + dash). Larguras/dashes ficam em
/// world-units; o `transform` do render escala p/ screen.
pub(crate) fn kurbo_stroke(s: &StrokeSpec, dash: Option<[f64; 2]>) -> Stroke {
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
    // ⚠️ **O tracejado chega JÁ AJUSTADO ao comprimento do caminho** — quem o mede é a
    // [`ph2d_vec_scene::dash_fit`], a porta única que o Outline Stroke também usa. Ler o
    // `s.dash_lengths()` cru aqui poria de volta a emenda visível na junção do contorno.
    match dash {
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
        // ⭐ **Um padrão sem ladrilho resolvido pinta a `fallback`, e isso é DESENHO CERTO, não uma
        // desistência.** A arte pode ainda não ter carregado, a forma-fonte pode ter desaparecido, o
        // assado pode ter recusado por tamanho — e em todos esses casos desenhar NADA seria pior: uma
        // forma invisível lê-se como *"a ferramenta está partida"* e não se distingue de um
        // preenchimento vazio. É o mesmo papel do `fallback` do `ProceduralFill`
        // (ADR-0056-amendment-3), pela mesma razão.
        //
        // ⚠️ Quando o ladrilho EXISTE, quem desenha é o `pattern::fill_pattern` (a rota de imagem),
        // e este braço não é alcançado — exactamente como o `MultiPoint` abaixo.
        Paint::Pattern(p) => Brush::Solid(color(p.fallback)),
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
#[path = "open_contour_tests.rs"]
mod open_contour_tests;

#[cfg(test)]
#[path = "standalone_tests.rs"]
mod standalone_tests;

/// ⭐⭐⭐ Os gates do ISOLAMENTO do *Edit Prefab* — irmão por assunto.
#[cfg(test)]
#[path = "isolation_tests.rs"]
mod isolation_tests;
