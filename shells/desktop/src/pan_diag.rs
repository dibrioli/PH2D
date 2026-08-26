//! **A SONDA DO DRIFT DE PAN** (`PH2D_PAN_DIAG=1`) — report do Enio, 2026-08-25:
//! *«no modo motion a imagem de referência sofre um drift no pan com o mouse»*, refinado
//! para *«acontece para Object e Chip, não para Star»*.
//!
//! ## ⚠️ A 1.ª versão desta sonda imprimia ZEROS sobre o defeito, e a auditoria disse porquê
//!
//! Ela comparava a rota das SPRITES com a rota do VELLO. Essas duas são a **mesma
//! expressão**, termo a termo:
//!
//! ```text
//! sprites:  x = W/2 + (X − cx)·Hs/hw        (orthographic_rh + set_viewport)
//! vello:    x = W/2 + (X − cx)·Hs/hw        (world_to_screen_affine)
//! ```
//!
//! — o `orthographic_rh` divide pelo half-extent, que traz o `w` do sub-retângulo, e ele
//! **cancela** contra o `w` da conversão NDC→pixel. ⇒ `Δpx` e `Δtrunc` só podem ser
//! diferentes de zero se as duas rotas receberem **dimensões diferentes**, que foi o Bug #9
//! (`422,4` contra `422`) e está curado. Depois disso são **identidades algébricas**: nenhuma
//! máquina, nenhum `t` e nenhum centro de câmera as faz imprimir outra coisa.
//!
//! *Uma sonda que só pode imprimir zero não é uma sonda.* A aritmética das rotas passou a ser
//! defendida por um GATE (`flip_pass_camera_tests.rs`, três portas × dois centros de câmera),
//! que é onde ela pertence. O que fica aqui é só o que um teste **não pode** ver:
//!
//! 1. **O sub-retângulo APLICADO contra o PEDIDO.** O passe de sprites larga o
//!    sub-retângulo em silêncio quando o quadro tem clip ou máscara
//!    (`scene_viewport.filter(|_| !has_clip && !has_mask)`) — decidido por CONTEÚDO do
//!    quadro, portanto imprevisível para quem chama. A 1.ª sonda imprimia o valor **pedido**
//!    e concluía que o passe o honrava. Agora ela lê o **efeito**
//!    (`SpriteRenderer::applied_subrect`), e grita `⚠️ CAIU` quando os dois divergem.
//! 2. **O MUNDO de uma amostra de cada rota** — se a posição de uma instância andar durante
//!    o arrasto, o defeito está no cozimento, a montante de toda projeção.
//! 3. **A câmera das duas rotas.** Hoje é identidade (o mesmo `&Camera2d`, o mesmo quadro);
//!    fica como sentinela para o dia em que alguém mover a montagem da cena Vello.
//!
//! ⚠️ **Ela não altera nada**: só lê e imprime, e só com a env var.

use ph2d_host::WindowSize;
use ph2d_render::Camera2d;

/// `true` se `PH2D_PAN_DIAG` está ligada (lida uma vez).
pub(crate) fn on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("PH2D_PAN_DIAG").is_ok_and(|v| v != "0"))
}

/// Onde a origem do mundo cai pela rota das SPRITES, pelo sub-retângulo **aplicado**.
fn sprite_px(cam: &Camera2d, w: f32, h: f32) -> (f64, f64) {
    let m = cam.view_proj_for_subrect(w, h).to_cols_array_2d();
    // Coluna-maior; a origem do mundo é `(0, 0)`, então sobra a translação.
    let (x, y) = (m[3][0], m[3][1]);
    (
        (f64::from(x) * 0.5 + 0.5) * f64::from(w),
        (0.5 - f64::from(y) * 0.5) * f64::from(h),
    )
}

/// Onde a MESMA origem cai pela rota VECTORIAL (o `cam_affine` do Vello).
fn vector_px(cam: &Camera2d, w: f32, h: f32) -> (f64, f64) {
    #[expect(clippy::cast_sign_loss, reason = "dims de janela sao positivas")]
    #[expect(clippy::cast_possible_truncation, reason = "px inteiros")]
    let a = cam.world_to_screen_affine(WindowSize::new(w as u32, h as u32));
    // A translação do afim É onde a origem do mundo cai (coeffs `[a,b,c,d,e,f]`).
    let c = a.as_coeffs();
    (c[4], c[5])
}

/// Onde a MESMA origem cai pela rota do FLIP — a TERCEIRA, a que ninguém tinha contado e a
/// que de facto quebrava (ela projetava a janela CHEIA sob o split, `1/t ≈ 1,82×`).
fn flip_px(cam: &Camera2d, window: WindowSize, sub: Option<[f32; 4]>) -> (f64, f64) {
    let c = crate::render_loop::flip_pass::camera::camera_scene(cam, window, sub);
    let m = c.world_to_clip;
    (
        (f64::from(m[3][0]) * 0.5 + 0.5) * f64::from(c.viewport[0]),
        (0.5 - f64::from(m[3][1]) * 0.5) * f64::from(c.viewport[1]),
    )
}

/// **O CENTRO DA CÂMERA COM QUE A CENA VELLO FOI CONSTRUÍDA**, guardado no instante em que
/// o `cam_affine` é montado. Sentinela: hoje as duas rotas leem o mesmo `&Camera2d` no mesmo
/// quadro, então `Δcam ≡ 0` por construção — o valor está aqui para o dia em que uma delas
/// passar a montar-se noutro ponto do quadro.
static VELLO_CENTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Chamado onde o `cam_affine` do Vello é montado.
pub(crate) fn note_vello_camera(center: [f32; 2]) {
    if !on() {
        return;
    }
    let bits = u64::from(center[0].to_bits()) << 32 | u64::from(center[1].to_bits());
    VELLO_CENTER.store(bits, std::sync::atomic::Ordering::Relaxed);
}

fn vello_center() -> [f32; 2] {
    let bits = VELLO_CENTER.load(std::sync::atomic::Ordering::Relaxed);
    [
        f32::from_bits((bits >> 32) as u32),
        f32::from_bits(bits as u32),
    ]
}

/// **A POSIÇÃO DE MUNDO de uma amostra de cada rota**, guardada no quadro.
///
/// Se o `P` de uma instância de sprite mudar enquanto se arrasta e o de uma vectorial não, o
/// defeito está a MONTANTE do desenho, no que alimenta o cozimento.
/// `(mundo da 1.ª sprite, mundo do 1.º vector, quantas sprites, quantos vectores)`.
type Sample = ([f32; 2], [f32; 2], usize, usize);
static SAMPLE: std::sync::Mutex<Option<Sample>> = std::sync::Mutex::new(None);

/// Chamado depois do cozimento, com a 1.ª instância de cada rota.
pub(crate) fn note_instances(sprites: &[ph2d_render::RenderInstance], vectors: &[[f32; 2]]) {
    if !on() {
        return;
    }
    let s = sprites.first().map_or([f32::NAN; 2], |i| i.world_pos);
    let v = vectors.first().copied().unwrap_or([f32::NAN; 2]);
    if let Ok(mut g) = SAMPLE.lock() {
        *g = Some((s, v, sprites.len(), vectors.len()));
    }
}

/// Uma linha por quadro. ⚠️ **Chamada DEPOIS do passe de sprites**, senão o `aplicado` seria
/// o do quadro anterior — e uma sonda um quadro atrasada sobre um defeito de um quadro é
/// exatamente o erro que ela existe para não repetir.
pub(crate) fn frame(
    cam: &Camera2d,
    window: WindowSize,
    motion_active: bool,
    pedido: Option<[f32; 4]>,
    aplicado: Option<[f32; 4]>,
) {
    if !on() {
        return;
    }
    let (w, h) = (window.width as f32, window.height as f32);
    // A rota das sprites, pelo que o passe APLICOU (não pelo que lhe pediram).
    let (rw, rh) = aplicado.map_or((w, h), |r| (r[2], r[3]));
    let sp = sprite_px(cam, rw, rh);
    let ve = vector_px(cam, rw, rh);
    let fl = flip_px(cam, window, aplicado);
    let caiu = if pedido.is_some() && aplicado.is_none() {
        " ⚠️ CAIU (clip/mascara derrubou o sub-retangulo)"
    } else {
        ""
    };
    let fmt = |r: Option<[f32; 4]>| {
        r.map_or_else(
            || "janela-cheia".to_string(),
            |v| format!("{:.0},{:.0} {:.0}x{:.0}", v[0], v[1], v[2], v[3]),
        )
    };
    let vc = vello_center();
    let (sw_pos, vw_pos, ns, nv) =
        SAMPLE
            .lock()
            .ok()
            .and_then(|g| *g)
            .unwrap_or(([f32::NAN; 2], [f32::NAN; 2], 0, 0));
    eprintln!(
        "[pan.diag] centro ({:.4}, {:.4}) altura {:.3} | janela {w:.0}x{h:.0} motion={motion_active} \
         | subrect pedido [{}] aplicado [{}]{caiu} \
         | origem: sprite ({:.2}, {:.2}) vector ({:.2}, {:.2}) flip ({:.2}, {:.2}) \
         | Δvector ({:.3}, {:.3}) Δflip ({:.3}, {:.3}) Δcam ({:.4}, {:.4}) \
         | MUNDO sprite[0] ({:.4}, {:.4}) de {ns} · vector[0] ({:.4}, {:.4}) de {nv}",
        cam.center[0],
        cam.center[1],
        cam.height_world,
        fmt(pedido),
        fmt(aplicado),
        sp.0,
        sp.1,
        ve.0,
        ve.1,
        fl.0,
        fl.1,
        sp.0 - ve.0,
        sp.1 - ve.1,
        sp.0 - fl.0,
        sp.1 - fl.1,
        cam.center[0] - vc[0],
        cam.center[1] - vc[1],
        sw_pos[0],
        sw_pos[1],
        vw_pos[0],
        vw_pos[1],
    );
}
