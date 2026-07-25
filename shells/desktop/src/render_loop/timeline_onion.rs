//! **O onion da timeline — poses-fantasma do objeto animado** (ADR-0142).
//!
//! Para autorar pose-a-pose o animador precisa VER onde o objeto estava e estará. Este
//! módulo constrói os *fantasmas*: silhuetas recoloridas do objeto selecionado nas poses
//! de `t ± k` quadros, injetadas no slot `extra` do passe de sprite (o mesmo que o Motion
//! usa). A pose vem de [`ph2d_timeline::pose_at`] — não-destrutiva, então o objeto vivo
//! nunca se move; a silhueta vem do `tint_fill` que o shader já tem (RGB da tinta, forma
//! do alfa do texel) ⇒ recoloração 100% (GP), sem passe de render novo.
//!
//! # Vocabulário compartilhado com o Flip, código separado (Chesterton, ADR-0142 §3)
//!
//! Os defaults de cor espelham `ph2d_flip::OnionSettings` (passado verde, futuro azul)
//! para o app ter UM vocabulário de fantasma. O código é separado porque a FONTE da pose
//! e o passe de render diferem (o Flip composita camadas de pixels; aqui injetamos
//! instâncias de sprite). Unificar em crate é follow-up se um 3º consumidor aparecer.

use ph2d_ecs::{GlobalTransform, PresentWorld, SimRef, World};
use ph2d_render::RenderInstance;
use ph2d_timeline::{TimelineDoc, animated_entities, entity_key_times, pose_at};

/// **O que os fantasmas vizinhos SÃO** (ADR-0142 §4). O onion serve os dois fluxos de um
/// timeline de keyframes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OnionMode {
    /// Fantasmas a `t ± k` QUADROS — mostra o espaçamento dos inbetweens (o ritmo).
    Frames,
    /// Fantasmas nas KEYFRAMES vizinhas — o pose-a-pose, o modelo do animador. O default.
    Keys,
}

/// Piso de opacidade de um fantasma — o mais distante ainda tem de ser visível. Espelha o
/// `GHOST_MIN_ALPHA` do onion do Flip.
pub(crate) const GHOST_MIN_ALPHA: f32 = 0.06;

/// Como os fantasmas do onion da timeline se parecem. Estado de VISTA (ADR-0142 W3): não
/// é serializado — a resposta a *"o que a tela mostra"* não deve mudar sozinha após um
/// load (a classe do toggle Physics / Speed graph).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct OnionSettings {
    /// Desligado por default: um onion que se arma sozinho é cena que já mudou ao olhar.
    pub enabled: bool,
    /// Quantos quadros ANTES do playhead ganham fantasma.
    pub frames_before: u32,
    /// Quantos quadros DEPOIS.
    pub frames_after: u32,
    /// A opacidade do fantasma mais PRÓXIMO; os mais distantes desvanecem a partir dela.
    pub opacity: f32,
    /// A cor (RGB) de um fantasma do passado — frio, o vocabulário do Flip.
    pub color_before: [f32; 3],
    /// A cor de um fantasma do futuro — o azul do Flip.
    pub color_after: [f32; 3],
    /// Quadros por segundo, para converter `frames_before/after` em tempo de clip
    /// (modo `Frames`).
    pub fps: f64,
    /// Fantasmas por QUADRO ou por KEYFRAME (ADR-0142 §4). Default `Keys`.
    pub mode: OnionMode,
}

impl Default for OnionSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            frames_before: 2,
            frames_after: 2,
            opacity: 0.5,
            // Os defaults do `ph2d_flip::OnionSettings` (ADR-0142 §3).
            color_before: [0.145, 0.420, 0.137], // LITERAL-COLOR-OK: onion, espelha o Flip
            color_after: [0.125, 0.082, 0.529],  // LITERAL-COLOR-OK: onion, espelha o Flip
            fps: 24.0,
            // Pose-a-pose é o modelo do animador num timeline de keyframes (ADR-0142 §4).
            mode: OnionMode::Keys,
        }
    }
}

/// A opacidade de um fantasma a `k` quadros de distância, de um total de `n`: o mais
/// próximo (`k=1`) recebe `opacity` cheia, o mais distante desvanece, com piso
/// `GHOST_MIN_ALPHA` (o mesmo falloff do Flip).
fn ghost_alpha(settings: &OnionSettings, k: u32, n: u32) -> f32 {
    let fade = 1.0 - f32::from((k - 1) as u16) / n as f32;
    (fade * settings.opacity).clamp(GHOST_MIN_ALPHA, 1.0)
}

/// Constrói o [`RenderInstance`] de UM fantasma: o `template` (os campos de sprite do
/// objeto vivo) com a pose de `t` e a cor/opacidade do onion, em modo silhueta.
/// `None` quando `pose_at` não resolve (a entidade não existe naquele instante).
fn ghost_instance(
    sim: &World,
    doc: &TimelineDoc,
    entity: u64,
    template: &RenderInstance,
    t: f64,
    tint: [f32; 4],
) -> Option<RenderInstance> {
    let pose = pose_at(sim, doc, entity, t)?;
    // A MESMA aritmética do extract: `affine()` = `[a,b,c,d,e,f]` coluna-a-coluna, então
    // o basis 2×2 é `[0..4]` e a translação é `[4..6]`. Para um objeto RAIZ o Transform É
    // o GlobalTransform (rigs parenteados são wave futura, ADR-0142).
    let a = GlobalTransform::from_transform(pose).affine();
    let mut g = *template;
    g.world_pos = [a[4], a[5]];
    g.basis = [a[0], a[1], a[2], a[3]];
    g.tint = tint;
    // Silhueta: o shader ignora o RGB do texel e usa o da tinta (a cor do onion), com a
    // forma vinda do alfa do texel — recoloração 100% (GP), reusando o `tint_fill`.
    g.flip_uv |= RenderInstance::TINT_FILL_BIT;
    // A cor do fantasma é SÓ a do onion: um per-corner-tint ou uma opacity herdados do
    // vivo escureceriam/apagariam a silhueta. O falloff mora na tinta (`tint.a`).
    g.per_corner_tint = [[1.0, 1.0, 1.0, 1.0]; 4];
    g.opacity = 1.0;
    Some(g)
}

/// Os instantes de clip a ghostar de UM lado, do mais PRÓXIMO ao mais distante do vivo.
/// `Frames`: `live ± k·dt`. `Keys`: as keyframes vizinhas de `entity` (o pose-a-pose).
fn ghost_times(
    settings: &OnionSettings,
    doc: &TimelineDoc,
    entity: u64,
    live_clip_t: f64,
    past: bool,
) -> Vec<f64> {
    let count = if past {
        settings.frames_before
    } else {
        settings.frames_after
    } as usize;
    match settings.mode {
        OnionMode::Frames => {
            if settings.fps <= 0.0 {
                return Vec::new();
            }
            let dt = 1.0 / settings.fps;
            (1..=count)
                .map(|k| {
                    let off = k as f64 * dt;
                    if past { live_clip_t - off } else { live_clip_t + off }
                })
                .collect()
        }
        OnionMode::Keys => {
            // Um `eps` exclui o próprio instante vivo se ele cair EXATO sobre uma key (o
            // playhead num keyframe): a pose viva não é um fantasma.
            let eps = 1e-6;
            let times = entity_key_times(doc, entity);
            if past {
                times
                    .iter()
                    .rev()
                    .filter(|&&t| t < live_clip_t - eps)
                    .take(count)
                    .copied()
                    .collect()
            } else {
                times
                    .iter()
                    .filter(|&&t| t > live_clip_t + eps)
                    .take(count)
                    .copied()
                    .collect()
            }
        }
    }
}

/// Constrói TODOS os fantasmas do onion e os acrescenta a `out`. `targets` = as entidades
/// a ghostar com o `RenderInstance` VIVO de cada uma (os campos de sprite — textura, uv,
/// tamanho, anchor). No-op quando desligado ou sem alvos.
pub(crate) fn build_ghosts(
    settings: &OnionSettings,
    sim: &World,
    doc: &TimelineDoc,
    targets: &[(u64, RenderInstance)],
    live_clip_t: f64,
    out: &mut Vec<RenderInstance>,
) {
    if !settings.enabled {
        return;
    }
    for (entity, template) in targets {
        for (past, color) in [(true, settings.color_before), (false, settings.color_after)] {
            let times = ghost_times(settings, doc, *entity, live_clip_t, past);
            // O falloff é sobre a contagem DE FATO encontrada (em Keys pode faltar key de
            // um lado): o mais próximo é o mais forte, o mais distante encontrado o mais
            // fraco.
            let n = times.len().max(1) as u32;
            for (i, &t) in times.iter().enumerate() {
                let a = ghost_alpha(settings, i as u32 + 1, n);
                let tint = [color[0], color[1], color[2], a];
                if let Some(g) = ghost_instance(sim, doc, *entity, template, t, tint) {
                    out.push(g);
                }
            }
        }
    }
}

/// O `RenderInstance` VIVO de `sel` no `present` — os campos de sprite (textura, uv,
/// tamanho, anchor) que o fantasma herda. Achado pelo `SimRef` (present → sim), o
/// back-pointer que o extract carimba. `None` se o objeto não tem sprite na cena.
fn live_template(present: &mut PresentWorld, sel: u64) -> Option<RenderInstance> {
    // A `QueryState` constrói-se com `&mut` mas itera com `&` — o mesmo padrão do
    // `sprite_collect` (o `q` é owned, então o borrow mutável acaba antes do `iter`).
    let mut q = present.world_mut().query::<(&SimRef, &RenderInstance)>();
    q.iter(present.world())
        .find(|(sref, _)| sref.0.to_bits() == sel)
        .map(|(_, ri)| *ri)
}

/// **A porta do shell:** monta os fantasmas do onion do objeto SELECIONADO e os acrescenta
/// a `out`. No-op quando desligado, sem seleção, quando o selecionado não é animado (um
/// objeto parado não tem passado nem futuro a mostrar), ou quando ele não tem sprite.
///
/// O escopo é o SELECIONADO (ADR-0142): edita-se o que está na mão, como o motion path.
pub(crate) fn collect_onion_ghosts(
    settings: &OnionSettings,
    sim: &World,
    present: &mut PresentWorld,
    doc: &TimelineDoc,
    selected: Option<u64>,
    live_clip_t: f64,
    out: &mut Vec<RenderInstance>,
) {
    if !settings.enabled {
        return;
    }
    let Some(sel) = selected else { return };
    if !animated_entities(doc).contains(&sel) {
        return;
    }
    let Some(template) = live_template(present, sel) else {
        return;
    };
    build_ghosts(settings, sim, doc, &[(sel, template)], live_clip_t, out);
}

#[cfg(test)]
#[path = "timeline_onion_tests.rs"]
mod timeline_onion_tests;
