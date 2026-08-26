//! **A CÂMERA do passe de Flip** — como o uniform que o shader recebe é construído.
//!
//! Irmão do `flip_pass.rs`, e o corte é por responsabilidade: lá mora *o que é composto* (as
//! camadas, o motor, as chaves do compositor); aqui mora *de que ponto de vista* — a paralaxe
//! multiplano, o fold do `model` do objeto e a conversão da `Camera2d` no uniform.
//!
//! ⚠️ As três são chamadas pelo `composite_layers` e re-exportadas pelo pai, então
//! `super::parallax_model` segue resolvendo (os testes não mudam de endereço).

use super::{Camera2d, CameraRaw, WindowSize, Xform};

/// **A porta ÚNICA da paralaxe multiplano** (2.5D, ADR-0114 §Decisão 3): desloca a
/// TRANSLAÇÃO do `model` do objeto por `(cam_center − origem)·(1 − depth)`, uma translação
/// de MUNDO. A camada passa a renderizar como se a câmera estivesse a
/// `lerp(cam_center, origem, depth)` — `depth = 1` (flat, o comum) devolve o model
/// **intacto** (`is_identity` segue verdadeiro ⇒ caminho byte-idêntico); `depth = 0` fixa a
/// origem do objeto no centro da tela (fundo estático). Só a translação muda; a parte linear
/// (rotação/escala) do gizmo fica. **Uma porta** — a arte assada, o fantasma e o traço de
/// preview desta camada TODOS passam por aqui, senão o esboço vivo folgaria da arte.
///
/// A âncora é a origem do objeto `(e, f)`: enquadrado de frente (a câmera sobre ela), todos
/// os planos coincidem; panhar os separa por `depth` (o deslocamento de tela = `depth × pan`).
pub(super) fn parallax_model(model: &Xform, cam_center: [f32; 2], depth: f32) -> Xform {
    if depth == 1.0 {
        return *model; // flat: intacto (byte-idêntico ao pré-multiplano)
    }
    let [a, b, c, d, e, f] = model.0;
    let k = 1.0 - depth as f64;
    Xform([
        a,
        b,
        c,
        d,
        e + (cam_center[0] as f64 - e) * k,
        f + (cam_center[1] as f64 - f) * k,
    ])
}

/// A câmera do passe com o `model` LOCAL→mundo do objeto dobrado: `world_to_clip ·
/// model`, e a espessura escalada pela escala média do objeto (`px_per_world ·
/// mean_scale`) — para o traço engrossar junto quando o gizmo escala. É isto que
/// deixa o gizmo de sprite mover/girar/escalar a arte SEM reescrever geometria.
pub(crate) fn fold_model(base: &CameraRaw, model: &Xform) -> CameraRaw {
    let [a, b, c, d, e, f] = model.0;
    // `model` como 4×4 col-major (`m[col][row]`): local (x, y, 0, 1) → mundo.
    let m: [[f32; 4]; 4] = [
        [a as f32, b as f32, 0.0, 0.0],
        [c as f32, d as f32, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [e as f32, f as f32, 0.0, 1.0],
    ];
    let p = base.world_to_clip; // col-major (mundo→clip)
    // combined = P · M (col-major): combined[j][row] = Σ_k P[k][row] · M[j][k].
    let mut w = [[0.0f32; 4]; 4];
    for (j, wj) in w.iter_mut().enumerate() {
        for (row, wjr) in wj.iter_mut().enumerate() {
            let mut s = 0.0;
            for k in 0..4 {
                s += p[k][row] * m[j][k];
            }
            *wjr = s;
        }
    }
    CameraRaw::new(
        w,
        base.viewport,
        base.px_per_world * model.mean_scale() as f32,
    )
}

/// Converte a `Camera2d` (mundo→clip ortográfico) no uniform do passe. O
/// `view_proj` é o MESMO afim que os sprites usam (as POSIÇÕES acompanham o zoom).
///
/// O 3º campo é a **escala de espessura** = **`px_per_world`** (ADR-0114 §4.C.6): a
/// largura do traço é guardada em unidades de MUNDO e o render a projeta como qualquer
/// outra grandeza geométrica (`thickness_px = raio_mundo · px_per_world`, que é o que o
/// `ph2d-flip-render` sempre documentou querer). Dar zoom engrossa o traço na tela —
/// arte é arte, não chrome.
///
/// Antes daqui passava `1.0`, que forçava a largura a ser lida como PX DE TELA (brush
/// absoluto, Enio 2026-07-11). Enio 2026-07-17 reverteu: *"a largura do traço está
/// relativa ao zoom do canvas e não é fixa no mundo"*. O `fold_model` de um objeto
/// escalado multiplica por `mean_scale` por cima (a arte engrossa junto com o gizmo).
pub(crate) fn camera_raw(camera: &Camera2d, window: WindowSize) -> CameraRaw {
    let vp = camera.view_proj(window).to_cols_array_2d();
    let px_per_world = window.height as f32 / camera.height_world.max(f32::EPSILON);
    CameraRaw::new(
        vp,
        [window.width as f32, window.height as f32],
        px_per_world,
    )
}

/// **A câmera do passe de Flip sob o SPLIT da tool Motion** — o QUARTO consumidor da mesma
/// lei, e o único que ficou de fora das três curas anteriores.
///
/// Sob o split a cena vive num sub-retângulo do alvo (`CenterSplit::scene_viewport`), e quem
/// projeta a JANELA CHEIA ali dentro fica com outra escala px/mundo — a `t = 0,55` isso é
/// `1/t ≈ 1,82×`. Como o pan converte o arrasto em mundo pela altura DA CENA
/// ([`crate::field_gizmo::pan_scene_camera`]), o erro não é um offset: ele **multiplica a
/// distância arrastada**, então a arte do Flip escorrega em relação a tudo o resto. Foi o
/// report do Enio de 2026-08-25 (*"a imagem de referência sofre um drift no pan"*).
///
/// ⚠️ **A cura NÃO é um `set_viewport`**, e é de propósito: este passe compõe em alvos
/// intermédios do tamanho do alvo final e termina num blit de alvo INTEIRO — recortar aqui
/// obrigaria a redimensionar a cadeia toda. A rota do Vello já resolve isto sem recorte
/// nenhum (projeta com as dims DA CENA num alvo cheio, e o conteúdo cai no canto), e é essa
/// que se copia: projeta-se pelo sub-retângulo e **remapeia-se o NDC** para a região que ele
/// ocupa no alvo. O remapeamento é afim em `x`/`y` (`ndc' = a·ndc + b`) e **preserva a escala
/// em PIXEL** — o que encolhe é o NDC, porque o alvo é maior que a cena.
///
/// ⚠️ O `viewport` do uniform continua a ser o do ALVO: o shader converte `in.clip` (que é
/// coordenada de fragmento do alvo cheio) por ele (`flip.wgsl:581`). Já o `px_per_world` passa
/// a ser o DA CENA — é ele que dá a espessura do traço, que tem de acompanhar a mesma escala
/// da geometria.
///
/// `None` (sem split) devolve [`camera_raw`] **ao bit** — o caminho comum não muda.
pub(crate) fn camera_scene(
    camera: &Camera2d,
    window: WindowSize,
    scene_viewport: Option<[f32; 4]>,
) -> CameraRaw {
    let Some([x0, y0, sw, sh]) = scene_viewport else {
        return camera_raw(camera, window);
    };
    let (tw, th) = (window.width.max(1) as f32, window.height.max(1) as f32);
    let (sw, sh) = (sw.max(1.0), sh.max(1.0));
    let m = camera.view_proj_for_subrect(sw, sh).to_cols_array_2d();
    // A região que o sub-retângulo ocupa em NDC do alvo. Escrita com `x0`/`y0` embora hoje
    // sejam sempre 0: um sub-retângulo ancorado noutro canto sairia silenciosamente errado.
    let (ax, ay) = (sw / tw, sh / th);
    let bx = (2.0 * x0 + sw) / tw - 1.0;
    let by = 1.0 - (2.0 * y0 + sh) / th;
    let mut r = m;
    for (col, src) in r.iter_mut().zip(m.iter()) {
        col[0] = ax * src[0] + bx * src[3];
        col[1] = ay * src[1] + by * src[3];
    }
    CameraRaw::new(r, [tw, th], sh / camera.height_world.max(f32::EPSILON))
}

#[cfg(test)]
#[path = "flip_pass_camera_tests.rs"]
mod tests;
