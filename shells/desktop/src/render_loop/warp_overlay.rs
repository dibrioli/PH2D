//! **O DESENHO do gizmo dos deformadores de quadrilátero** — o contorno, os braços e as
//! alças que o `motion.four_point_warp` e o `motion.bezier_warp` passam a ter na tela.
//!
//! A geometria vive em [`super::warp_gizmo`], pura e testada; aqui mora só tinta.
//!
//! ## O que se desenha, e por que cada peça
//!
//! - **O CONTORNO** — as quatro arestas, avaliadas pela cúbica do próprio nó. É a peça
//!   que responde *"que forma é esta?"*, e sem ela doze pontos soltos não dizem nada.
//! - **OS BRAÇOS** — do canto até cada tangente dele. ⚠️ Sem eles, oito pontos à volta
//!   de um quadrilátero não dizem a que aresta pertencem, e o artista arrasta o errado.
//!   É a mesma razão por que o editor de curvas da casa desenha as suas alças ligadas.
//! - **AS ALÇAS** — quadrado para canto, círculo para tangente. ⚠️ **A forma distingue o
//!   TIPO**, e não só a posição: com a fronteira quase recta a tangente nasce a um terço
//!   do canto, e duas marcas iguais ali seriam indistinguíveis.
//!
//! ## ⚠️ Tudo em pixels de TELA
//!
//! O caminho é construído já em coordenadas de tela e traçado com `Affine::IDENTITY`,
//! porque `stroke` **multiplica** a espessura pelo transform: entregar o afim mundo→tela
//! como transform faria a linha engordar com o zoom. É a lei que o `anchor_overlay`
//! escreveu no cabeçalho dele, e ela vale igual aqui.

use super::warp_gizmo::{self, MAX_HANDLES, WarpHandle, WarpHandleKind};
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_vector::{Affine, BezPath, Brush, Color, Point, Stroke, VectorScene};

/// A espessura do contorno, em pixels de tela.
const OUTLINE_PX: f64 = 1.5;
/// A espessura de um braço — mais fina que o contorno: ele é ANDAIME, não a figura.
const ARM_PX: f64 = 1.0;

/// ⭐⭐ **O MEIO-LADO DO QUADRADO DE UM CANTO — e ele é DERIVADO do raio de agarre.**
///
/// ⛔⛔ **Ele era `4,5` contra um agarre de `11`, e isso é um defeito com report** (Enio,
/// 2026-09-08: *«tem ponto e handles/alças muito pequenos»*). O alvo era **2,4×** maior do que a
/// tinta: *o artista via um ponto e apontava para outra coisa*.
///
/// ⚠️ **A casa já tinha a lei, e este gizmo era o único fora dela:** o `connector` pinta `7` e
/// agarra `7`; o `envelope` pinta `6` e agarra `6`; o `vec_text_ride` pinta `15` e agarra `15`.
/// **Uma constante, dois consumidores.** Aqui eram dois números, e o que o artista vê é sempre o
/// que envelhece.
///
/// ⚠️ O factor `0,85` **também é da casa**, com a razão escrita no
/// `ph2d_vec_render::draw_connector_waypoints`: *«um quadrado de mesma área "pesa" mais que o
/// círculo»*.
pub(super) const CORNER_PX: f64 = super::warp_gizmo::GRAB_PX as f64 * 0.85;

/// O raio (do centro ao vértice) do losango de uma tangente.
///
/// ⚠️ **A tangente NÃO é menor por ser secundária**, e o cabeçalho deste módulo já o dizia antes
/// de o report chegar: *«a hierarquia tem de estar na TINTA e não só no tamanho»*. Ela agarra-se
/// pelo mesmo `GRAB_PX` que um canto, então pintá-la menor era a mesma divergência, mais funda.
///
/// ⚠️ **O `√2` é geometria, não gosto:** um losango de raio `r` tem a apótema em `r/√2`, e é a
/// APÓTEMA — o pior caso do que a marca cobre — que decide se o dedo cai dentro do que o olho vê.
/// Multiplicá-lo põe a apótema do losango exactamente no meio-lado do quadrado, então as duas
/// marcas cobrem o MESMO raio e distinguem-se só pela forma e pela tinta. *A primeira redacção
/// usava `GRAB_PX` cru e o gate reprovou-a a `0,71×` da barra — a régua apanhou-me a confundir o
/// raio com o alcance.*
pub(super) const TANGENT_PX: f64 =
    super::warp_gizmo::GRAB_PX as f64 * 0.85 * std::f64::consts::SQRT_2;

/// A cor do contorno e das alças de canto.
const HANDLE_RGBA: [f32; 4] = [0.35, 0.78, 1.0, 1.0];
/// A cor dos braços e das tangentes — a mesma matiz, um pouco apagada: elas são o segundo
/// nível de leitura, e a hierarquia mora aqui e na FORMA, nunca no tamanho.
///
/// ⚠️ **Era `0,55` e some sobre conteúdo claro** — a metade da queixa de *«está sendo desenhado
/// por trás das shapes»*. Com o casing abaixo, `0,9` é legível sobre os dois fundos e continua
/// atrás do canto na hierarquia.
const TANGENT_RGBA: [f32; 4] = [0.35, 0.78, 1.0, 0.9];

/// ⭐⭐ **O CASING — o contorno escuro por baixo de tudo o que o gizmo desenha.**
///
/// ⛔⛔ **É a outra metade do report de 2026-09-08** (*«está sendo desenhado por trás das
/// shapes»*). O gizmo **está** por cima — a legenda da cena, que é chrome como ele, aparece sobre
/// os mesmos objectos —, mas um traço de `1,5 px` e uma alça de `4,5` em ciano claro **somem**
/// sobre um pano de discos brancos. *Um manipulador que desaparece sobre o conteúdo que ele
/// manipula é indistinguível de um que está por baixo, e o report que volta é o mesmo.*
///
/// ⚠️ **A casa já tinha a cura, e o comentário dela nomeia este defeito:** o
/// `draw_connector_handles` põe um anel branco em cada bolinha *«— sem o anel a bolinha some
/// sobre um traço claro»*. Aqui o anel é ESCURO e não branco, porque o conteúdo sobre o qual este
/// gizmo tem de ser lido é claro (uma folha de objectos) tanto quanto escuro (o fundo do canvas):
/// ciano sobre escuro lê-se pela luminosidade, e é a borda escura que o salva sobre o branco.
const CASE_RGBA: [f32; 4] = [0.04, 0.04, 0.06, 0.85];
/// Quanto o casing sobressai de cada lado do traço que ele protege, em pixels.
const CASE_PX: f64 = 1.25;

/// **Desenha o gizmo do nó de warp seleccionado.** No-op quando não há nenhum, quando a
/// tomada ainda não trouxe a caixa, ou quando o layout é degenerado.
///
/// ⚠️ **O `active` é a tool Motion**, e ele é gateado pelo chamador como no
/// `field_gizmo`: fora dela o canvas é dos sprites, e alças de nó ali seriam alvos que
/// roubam o clique de outra ferramenta.
/// ⛔⛔ **A SONDA DO QUADRO — `PH2D_WARP_DIAG=1`.**
///
/// Report do Enio, 2026-09-08, **duas vezes** (*«sumiu com o gizmo»* e, depois da minha primeira
/// cura, *«ainda invisível»*). As duas curas saíram de hipóteses LIDAS do código, e as duas
/// estavam erradas — enquanto isso, tudo o que se consegue medir de fora diz que o gizmo devia
/// estar lá: a sonda `why_the_warp_gizmo_is_not_there` mostra o `resolve` a devolver `Some` na
/// cena EXACTA do report (com o warp antes e depois do espelho), e o gate
/// `the_gizmo_paints_geometry_and_paints_none_when_inactive` mostra o pintor a emitir tinta.
///
/// ⇒ o que falta medir é o **QUADRO**, e para isso não há arnês: o `run_render_frame` pede uma
/// janela real. Esta linha é o instrumento mínimo que fecha o buraco — ela diz, do sítio onde a
/// tinta sai, se ela saiu e quanta. *Uma pergunta sobre o quadro responde-se no quadro.*
pub(super) fn diag(marca: &str) -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    if !*ON.get_or_init(|| std::env::var("PH2D_WARP_DIAG").is_ok_and(|v| v != "0")) {
        return false;
    }
    // Uma linha por segundo, não por quadro: a 60 fps o terminal deixaria de ser legível.
    static ULTIMA: std::sync::Mutex<Option<std::time::Instant>> = std::sync::Mutex::new(None);
    let Ok(mut u) = ULTIMA.lock() else {
        return false;
    };
    let agora = std::time::Instant::now();
    if u.is_none_or(|t| agora.duration_since(t).as_secs_f32() >= 1.0) {
        *u = Some(agora);
        eprintln!("[warp-diag] {marca}");
        return true;
    }
    false
}

pub(super) fn draw_warp_gizmo(
    active: bool,
    v: &warp_gizmo::WarpGizmoView,
    param: &dyn Fn(&str) -> f32,
    camera: &Camera2d,
    center_split: ph2d_editor::screens::layout::CenterSplit,
    full_window: WindowSize,
    vector_scene: &mut VectorScene,
) {
    if !active {
        diag("a tool nao e' a Motion — nada desenhado");
        return;
    }
    // ⚠️ **A janela da CENA, resolvida AQUI e não pelo chamador.** Passar a janela cheia
    // desloca e encolhe tudo o que é desenhado em coordenadas de mundo — e, pior, faz a
    // tinta discordar do hit-test, que usa a janela certa. Ver
    // [`super::warp_gizmo::scene_window`], que carrega o relato do defeito.
    let to_screen =
        camera.world_to_screen_affine(warp_gizmo::scene_window(center_split, full_window));
    let pt = |w: [f32; 2]| to_screen * Point::new(f64::from(w[0]), f64::from(w[1]));

    // ── o CONTORNO, já no espaço do que se VÊ (a cadeia de jusante aplicada) ──
    let (ring, corners) = warp_gizmo::view_outline(v, param);
    let mut path = BezPath::new();
    for (i, w) in ring.iter().enumerate() {
        let p = pt(*w);
        if i == 0 {
            path.move_to(p);
        } else {
            path.line_to(p);
        }
    }
    let brush = Brush::Solid(Color::new(HANDLE_RGBA));
    let case = Brush::Solid(Color::new(CASE_RGBA));
    // O casing PRIMEIRO, mais grosso — ver [`CASE_RGBA`].
    vector_scene.inner_mut().stroke(
        &Stroke::new(OUTLINE_PX + CASE_PX * 2.0),
        Affine::IDENTITY,
        &case,
        None,
        &path,
    );
    vector_scene.inner_mut().stroke(
        &Stroke::new(OUTLINE_PX),
        Affine::IDENTITY,
        &brush,
        None,
        &path,
    );

    let (hs, n) = warp_gizmo::view_handles(v, param);
    let live: &[WarpHandle] = &hs[..n.min(MAX_HANDLES)];

    // ── os BRAÇOS, antes das alças (elas pousam por cima) ──
    let dim = Brush::Solid(Color::new(TANGENT_RGBA));
    let mut arms = BezPath::new();
    let mut any_arm = false;
    for h in live {
        if let Some(c) = warp_gizmo::tangent_arm(h.kind) {
            arms.move_to(pt(corners[c]));
            arms.line_to(pt(h.world));
            any_arm = true;
        }
    }
    if any_arm {
        vector_scene.inner_mut().stroke(
            &Stroke::new(ARM_PX + CASE_PX * 2.0),
            Affine::IDENTITY,
            &case,
            None,
            &arms,
        );
        vector_scene
            .inner_mut()
            .stroke(&Stroke::new(ARM_PX), Affine::IDENTITY, &dim, None, &arms);
    }

    if diag("desenhando") {
        let (cx, cy) = (v.bbox.lo, v.bbox.hi);
        let a = pt(ring[0]);
        eprintln!(
            "[warp-diag]   caixa de mundo {cx:?}..{cy:?} · 1.o ponto do contorno em TELA \
             ({:.1}, {:.1}) · {} alcas · alca de canto {CORNER_PX:.1} px",
            a.x,
            a.y,
            live.len()
        );
    }
    // ── as ALÇAS ──
    for h in live {
        let c = pt(h.world);
        match h.kind {
            WarpHandleKind::Corner(_) => {
                let mut sq = BezPath::new();
                sq.move_to(Point::new(c.x - CORNER_PX, c.y - CORNER_PX));
                sq.line_to(Point::new(c.x + CORNER_PX, c.y - CORNER_PX));
                sq.line_to(Point::new(c.x + CORNER_PX, c.y + CORNER_PX));
                sq.line_to(Point::new(c.x - CORNER_PX, c.y + CORNER_PX));
                sq.close_path();
                vector_scene.inner_mut().fill(
                    ph2d_vector::Fill::NonZero,
                    Affine::IDENTITY,
                    &brush,
                    None,
                    &sq,
                );
                // O casing por CIMA do preenchimento: uma borda, não uma sombra — assim ele
                // separa a alça do fundo sem lhe comer área.
                vector_scene.inner_mut().stroke(
                    &Stroke::new(CASE_PX * 2.0),
                    Affine::IDENTITY,
                    &case,
                    None,
                    &sq,
                );
            }
            WarpHandleKind::Tangent(..) => {
                let mut ci = BezPath::new();
                // Um losango: quatro linhas, sem arco — o círculo exacto custaria quatro
                // cúbicas e a distinção que interessa (canto × tangente) é a FORMA, não a
                // suavidade dela.
                ci.move_to(Point::new(c.x, c.y - TANGENT_PX));
                ci.line_to(Point::new(c.x + TANGENT_PX, c.y));
                ci.line_to(Point::new(c.x, c.y + TANGENT_PX));
                ci.line_to(Point::new(c.x - TANGENT_PX, c.y));
                ci.close_path();
                vector_scene.inner_mut().fill(
                    ph2d_vector::Fill::NonZero,
                    Affine::IDENTITY,
                    &dim,
                    None,
                    &ci,
                );
                vector_scene.inner_mut().stroke(
                    &Stroke::new(CASE_PX * 2.0),
                    Affine::IDENTITY,
                    &case,
                    None,
                    &ci,
                );
            }
        }
    }
}
