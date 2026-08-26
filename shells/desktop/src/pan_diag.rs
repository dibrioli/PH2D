//! **A SONDA DO DRIFT DE PAN** (`PH2D_PAN_DIAG=1`) — report do Enio, 2026-08-25:
//! *«no modo motion a imagem de referência sofre um drift no pan com o mouse»*, refinado
//! para *«acontece para Object e Chip, não para Star»*.
//!
//! ## Por que uma sonda e não um terceiro palpite
//!
//! Duas hipóteses foram **refutadas por medição** antes desta:
//!
//! 1. *«as duas rotas projectam com janelas diferentes»* — MEDIDO e falso: para o mesmo
//!    ponto de mundo, `view_proj_for_subrect(w,h)` (a rota das SPRITES, com o `set_viewport`)
//!    e `world_to_screen_affine(WindowSize{w,h})` (a rota VECTORIAL) caem **no mesmo pixel**,
//!    a `0,000` em nove pares centro×ponto.
//! 2. *«o quadro tem clip ou máscara e o passe de sprites cai para a janela cheia»* —
//!    o ramo existe (`renderer_draw`, `scene_viewport.filter(|_| !has_clip && !has_mask)`),
//!    mas um papel de clip/máscara só nasce de um `Mask2D`/`MaskInteraction`/`ClipChildren`
//!    numa entidade, e a cena `=9` e o documento de arranque não têm nenhum.
//!
//! ⚠️ *Um terceiro palpite custaria outra viagem ao Enio.* Esta sonda imprime, **numa linha
//! por quadro de pan**, as três respostas à MESMA pergunta — *onde cai a origem do mundo?* —
//! e o que cada rota usou para a responder. Uma corrida dela nomeia o mecanismo.
//!
//! ⚠️ **Ela não altera nada**: só lê e imprime, e só com a env var.

use ph2d_host::WindowSize;
use ph2d_render::Camera2d;

/// `true` se `PH2D_PAN_DIAG` está ligada (lida uma vez).
pub(crate) fn on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("PH2D_PAN_DIAG").is_ok_and(|v| v != "0"))
}

/// Onde a origem do mundo cai pela rota das SPRITES (a projecção que o
/// `renderer_draw` envia, mais o `set_viewport` que a mapeia em pixels).
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

/// **O CENTRO DA CÂMERA COM QUE A CENA VELLO FOI CONSTRUÍDA**, guardado no instante em que
/// o `cam_affine` é montado.
///
/// ⚠️ **Esta é a metade que a 1.ª versão da sonda não tinha, e ela cobre a hipótese mais
/// forte:** a cena do Vello é construída na CPU com o mundo→tela **já aplicado**, enquanto
/// as sprites viajam em coordenadas de MUNDO e recebem a câmera num *uniform* escrito
/// noutro ponto do quadro. Se os dois pontos virem câmeras diferentes — uma delas de um
/// quadro atrás —, o vector acompanha o cursor e as sprites ficam **um quadro para trás**:
/// invisível parado, e um drift enquanto se arrasta. *Uma sonda que lê a MESMA câmera para
/// as duas rotas não pode ver isto, por construção.*
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
/// ⚠️ **É a pergunta que sobrou depois de a 1.ª corrida ilibar a projecção.** A sonda
/// mediu `Δcam = 0` e `Δpx = 0` em todos os quadros de um arrasto: as duas rotas usam a
/// MESMA câmera no MESMO instante e põem a origem do mundo no MESMO pixel. ⇒ o que deriva
/// não é onde a câmera está — é **o que cada peça diz que a posição dela é**. Se o `P` de
/// uma instância de sprite mudar enquanto se arrasta e o de uma vectorial não, o defeito
/// está a MONTANTE do desenho, no que alimenta o cozimento.
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

/// Uma linha por quadro, com as TRÊS respostas e o que cada uma usou.
///
/// ⚠️ **O `Δ` é o que decide**: se ele for `0` e a imagem ainda derivar, o defeito não está
/// na projecção — está no que cada passe DESENHA (a instância, o ladrilho, o compositor), e
/// a sonda tê-lo-á excluído. Se ele crescer ao arrastar, o defeito é a projecção e a linha
/// diz qual das duas janelas está errada.
pub(crate) fn frame(
    cam: &Camera2d,
    window: WindowSize,
    scene: WindowSize,
    motion_active: bool,
    has_scene_viewport: bool,
    split_is_split: bool,
) {
    if !on() {
        return;
    }
    let (w, h) = (window.width as f32, window.height as f32);
    let (sw, sh) = (scene.width as f32, scene.height as f32);
    let sp = sprite_px(cam, sw, sh);
    let ve = vector_px(cam, sw, sh);
    // O que o pan aplica hoje, e o que ele aplicaria com a janela cheia.
    let per_px_scene = cam.height_world / sh.max(1.0);
    let per_px_window = cam.height_world / h.max(1.0);
    // ⚠️ **A CÂMERA DAS DUAS ROTAS, lado a lado.** Um `Δcam` diferente de zero enquanto se
    // arrasta É o defeito: as sprites estariam a ser desenhadas com a câmera de outro
    // instante que o vector.
    let vc = vello_center();
    let dcam = (cam.center[0] - vc[0], cam.center[1] - vc[1]);
    // ⚠️ **O MUNDO de uma amostra de cada rota.** Se estes números andarem enquanto se
    // arrasta, o defeito está no COZIMENTO e não no desenho — e a linha di-lo-á.
    let (sw_pos, vw_pos, ns, nv) =
        SAMPLE
            .lock()
            .ok()
            .and_then(|g| *g)
            .unwrap_or(([f32::NAN; 2], [f32::NAN; 2], 0, 0));
    eprintln!(
        "[pan.diag] centro sprite ({:.4}, {:.4}) vello ({:.4}, {:.4}) Δcam ({:.4}, {:.4}) \
         | altura {:.3} | janela {w:.0}x{h:.0} cena {sw:.0}x{sh:.0} \
         | motion={motion_active} viewport={has_scene_viewport} split={split_is_split} \
         | origem: sprite ({:.2}, {:.2}) vector ({:.2}, {:.2}) Δpx ({:.3}, {:.3}) \
         | mundo/px cena {per_px_scene:.5} janela {per_px_window:.5} \
         | MUNDO sprite[0] ({:.4}, {:.4}) de {ns} · vector[0] ({:.4}, {:.4}) de {nv}",
        cam.center[0],
        cam.center[1],
        vc[0],
        vc[1],
        dcam.0,
        dcam.1,
        cam.height_world,
        sp.0,
        sp.1,
        ve.0,
        ve.1,
        sp.0 - ve.0,
        sp.1 - ve.1,
        sw_pos[0],
        sw_pos[1],
        vw_pos[0],
        vw_pos[1],
    );
}
