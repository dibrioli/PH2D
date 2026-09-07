//! ⭐⭐⭐ **A CÂMERA VAI À RECEITA no instante em que ela ABRE** (Enio, 2026-09-07: *«o prefab não
//! apareceu no centro relativo ao canvas visível»*).
//!
//! # Porque a câmera se move, e não a receita
//!
//! A receita é um objecto do documento como qualquer outro: deslocá-la para o meio do ecrã seria
//! uma **edição** — entra no undo, viaja no ficheiro e chega a todas as cópias. E desenhá-la
//! deslocada sem a mover partiria a regra-mãe do módulo vetorial (*«o que se vê/aponta/encaixa é
//! MUNDO»*): o rato iria procurá-la onde ela não está. ⇒ o que se move é a **vista**.
//!
//! # A área é a VISÍVEL, não a janela
//!
//! ⚠️ O canvas é *full-bleed* — a [`ph2d_editor::screens::layout::HeroLayout::canvas`] É o viewport
//! inteiro e os painéis flutuam por cima dele. Centrar na janela põe a receita **debaixo de uma
//! coluna docada**, que é exactamente o report. A área que conta é a que sobra depois do chrome
//! docado e das réguas, e ela tem porta: [`crate::canvas_area::visible`].
//!
//! # O zoom só AFASTA
//!
//! Aproximar mudaria a escala de trabalho do artista sem ele pedir; afastar é o que torna a
//! promessa *«a receita aparece»* verdadeira quando ela é maior que a área. ⇒ a lei é
//! `max(zoom_actual, o_que_falta_para_caber)`, e numa receita que já cabe ela é **inerte**.

use ph2d_editor::zones::Rect;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;

/// **Que fracção da área visível a receita pode ocupar antes de a vista afastar.**
///
/// ⚠️ Não é um teto de recurso — é composição, e as duas falhas que ela evita têm nome: a `1,0`
/// encosta a receita nas arestas (e o anel de mundo recuado à volta dela, que é o que diz *«isto
/// está isolado»*, desaparece); muito abaixo disto, abrir uma receita pequena afastaria a vista
/// sem necessidade — e é por isso que a lei nunca **aproxima**.
pub(crate) const FILL: f32 = 0.7;

/// ⭐ **A LEI, sem o app** — a câmera que põe a caixa `min..max` (mundo) no centro de `area`
/// (pixels da janela), afastando só o necessário para ela caber.
///
/// ⚠️ **A ordem é load-bearing:** o zoom decide-se PRIMEIRO, porque o passo de centrar pergunta
/// *«que ponto de mundo está debaixo deste pixel?»* e essa resposta depende do zoom. Centrar antes
/// deixaria a receita fora do centro exactamente na proporção em que a vista tivesse afastado.
///
/// ⚠️ E o centro sai de [`Camera2d::screen_to_world`] em vez de uma conta à mão: é a **mesma**
/// porta que o rato usa para saber onde clicou, então a receita cai debaixo do pixel que o artista
/// vê no meio, por construção.
pub(crate) fn framed(
    cam: Camera2d,
    window: WindowSize,
    area: Rect,
    min: [f32; 2],
    max: [f32; 2],
) -> Camera2d {
    let mut out = cam;
    let area = usable(area, window);
    // — 1. O ZOOM, e só para afastar.
    let win_h = window.height.max(1) as f32;
    let box_w = (max[0] - min[0]).abs();
    let box_h = (max[1] - min[1]).abs();
    // Quantos metros de mundo a área mostra por pixel: `height_world / altura da janela`. Para a
    // caixa caber em `FILL` da área, a altura de mundo tem de ser pelo menos isto.
    let need_w = if area.w > 0.0 {
        box_w * win_h / (area.w * FILL)
    } else {
        0.0
    };
    let need_h = if area.h > 0.0 {
        box_h * win_h / (area.h * FILL)
    } else {
        0.0
    };
    let need = need_w.max(need_h);
    if need.is_finite() && need > out.height_world {
        out.height_world = need.clamp(
            Camera2d::ZOOM_MIN_HEIGHT_WORLD,
            Camera2d::ZOOM_MAX_HEIGHT_WORLD,
        );
    }
    // — 2. O CENTRO: o ponto de mundo que hoje está debaixo do centro da área tem de passar a ser
    // o centro da caixa. Como a projecção é afim no centro da câmera com gradiente unitário, a
    // correcção é a diferença dos dois pontos — sem inverter matriz nenhuma.
    let target_px = (area.x + area.w * 0.5, area.y + area.h * 0.5);
    let under = out.screen_to_world(target_px, window);
    let box_c = [(min[0] + max[0]) * 0.5, (min[1] + max[1]) * 0.5];
    out.center = [
        out.center[0] + (box_c[0] - under[0]),
        out.center[1] + (box_c[1] - under[1]),
    ];
    out
}

/// A área utilizável — a publicada, ou a janela quando ela ainda não existe.
///
/// ⚠️ **O primeiro quadro publica uma área degenerada** (o painel ainda não pintou), e uma divisão
/// por zero ali daria uma câmera `NaN` que nenhum gesto recupera.
fn usable(area: Rect, window: WindowSize) -> Rect {
    if area.w > 1.0 && area.h > 1.0 {
        area
    } else {
        Rect::new(
            0.0,
            0.0,
            window.width.max(1) as f32,
            window.height.max(1) as f32,
        )
    }
}

/// **A porta do quadro** — consome o pedido pendente quando a caixa da receita já foi publicada.
///
/// ⚠️ **Ela lê a caixa do GIZMO, e isso não é atalho:** o gizmo é a resposta desta casa a *«onde
/// está este objecto?»* — para um grupo ele é a união dos filhos visíveis
/// ([`crate::group_gizmo_view`]), para uma forma a caixa da curva, para uma sprite o quad. Derivar
/// aqui uma segunda caixa seria uma segunda resposta à mesma pergunta, e elas divergiriam no dia
/// em que uma receita tivesse uma peça de um tipo novo.
///
/// ⚠️ **E o pedido espera pela caixa em vez de a exigir:** ele só é consumido quando a vista
/// publicada já é a da receita — enquanto não for, ele fica pendente e tenta no quadro seguinte.
pub(crate) fn apply(
    pending: &mut Option<u64>,
    hero: &ph2d_editor::screens::hero::HeroScreen,
    viewport: Rect,
    window: WindowSize,
    camera: &mut Camera2d,
) -> bool {
    let Some(bits) = *pending else {
        return false;
    };
    if hero.gizmo.selection != Some(bits) {
        return false;
    }
    let Some(view) = hero.gizmo.view.as_ref() else {
        return false;
    };
    *camera = framed(
        *camera,
        window,
        crate::canvas_area::visible(hero, viewport),
        view.bbox_min_world,
        view.bbox_max_world,
    );
    *pending = None;
    true
}

#[cfg(test)]
#[path = "prefab_framing_tests.rs"]
mod tests;
