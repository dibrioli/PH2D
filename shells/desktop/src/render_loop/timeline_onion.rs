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

use ph2d_ecs::{Entity, GlobalTransform, PresentWorld, SimRef, SimWorld, Transform};
use ph2d_render::{LiftedInstances, RenderInstance};
use ph2d_timeline::{TimelineDoc, animated_entities, entity_key_times, world_pose_at};

// As configurações do onion (`OnionSettings`/`OnionMode`) moram em `ph2d-timeline` (dados
// puros), para o `TimelineState`, o `apply_intent`, o snapshot e o painel compartilharem a
// MESMA língua (ADR-0142 W3). Aqui fica o MOTOR de fantasmas (silhueta em `RenderInstance`).
pub(crate) use ph2d_timeline::{OnionMode, OnionSettings};

/// Piso de opacidade de um fantasma — o mais distante ainda tem de ser visível. Espelha o
/// `GHOST_MIN_ALPHA` do onion do Flip.
pub(crate) const GHOST_MIN_ALPHA: f32 = 0.06;

/// A opacidade de um fantasma a `k` quadros de distância, de um total de `n`: o mais
/// próximo (`k=1`) recebe `opacity` cheia, o mais distante desvanece, com piso
/// `GHOST_MIN_ALPHA` (o mesmo falloff do Flip).
fn ghost_alpha(settings: &OnionSettings, k: u32, n: u32) -> f32 {
    let fade = 1.0 - f32::from((k - 1) as u16) / n as f32;
    (fade * settings.opacity).clamp(GHOST_MIN_ALPHA, 1.0)
}

/// Constrói o [`RenderInstance`] de UM fantasma: o `template` (os campos de sprite do
/// objeto vivo) com a pose de `t` e a cor/opacidade do onion, em modo silhueta.
/// `None` quando a pose não resolve (a entidade não existe naquele instante).
fn ghost_instance(
    sim: &SimWorld,
    doc: &TimelineDoc,
    entity: u64,
    template: &RenderInstance,
    t: f64,
    tint: [f32; 4],
) -> Option<RenderInstance> {
    // ⭐⭐⭐ **A pose de MUNDO em `t`** ([`world_pose_at`]) — a cadeia inteira posada, e não só a
    // folha. ⚠️ Até 2026-09-13 isto era o `pose_at` LOCAL, com a nota *«para um objeto RAIZ o
    // Transform É o GlobalTransform; rigs parenteados são wave futura»*: um objecto pendurado
    // ghostava a um offset do pai. Para uma raiz as duas respostas são a MESMA (compor com a
    // identidade não move um ULP, e os gates deste ficheiro provam-no), então a nota fechou de graça
    // quando a porta apareceu para os ossos.
    let pose = world_pose_at(sim.world(), doc, entity, t)?;
    // A MESMA aritmética do extract: `affine()` = `[a,b,c,d,e,f]` coluna-a-coluna, então
    // o basis 2×2 é `[0..4]` e a translação é `[4..6]`.
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
/// `Frames`: `live ± k·dt`. `Keys`: as keyframes vizinhas dos **relógios** (o pose-a-pose).
///
/// ⭐⭐⭐ **`relogios` é quem tem as KEYS, que não é sempre quem se DESENHA** (W7). Numa personagem
/// riggada as keys vivem nos **ossos** e o que se vê é a **imagem** — ler as keyframes do alvo
/// desenhado devolveria a lista VAZIA, e o modo `Keys` (que é o de OMISSÃO) não mostraria fantasma
/// nenhum. *O mesmo defeito do escopo, um nível abaixo: a pergunta certa é «quem MOVE isto?».*
///
/// ⚠️ **A UNIÃO, e não o primeiro:** a pose do braço muda quando QUALQUER osso dele tem uma key, e
/// uma lista só do osso na mão saltaria as poses que os vizinhos autoram.
fn ghost_times(
    settings: &OnionSettings,
    doc: &TimelineDoc,
    relogios: &[u64],
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
                    if past {
                        live_clip_t - off
                    } else {
                        live_clip_t + off
                    }
                })
                .collect()
        }
        OnionMode::Keys => {
            // Um `eps` exclui o próprio instante vivo se ele cair EXATO sobre uma key (o
            // playhead num keyframe): a pose viva não é um fantasma.
            let eps = 1e-6;
            let mut times: Vec<f64> = relogios
                .iter()
                .flat_map(|&e| entity_key_times(doc, e))
                .collect();
            times.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            // Duas keys no MESMO instante (dois ossos autorados juntos) são UMA pose-fantasma — a
            // mesma dedup que o `entity_key_times` faz para as colunas `X`+`Y` de um objecto.
            times.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
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

/// ⚠️ **As peças da pele que NÃO mudam entre fantasmas**, numa struct — a lei do `BandGear` vizinho:
/// *a engrenagem viaja numa struct, e não em oito argumentos*. O índice de ossos e a malha de
/// repouso resolvem-se uma vez por quadro; o que varia por fantasma é o **instante**.
struct PeleDoQuadro<'a> {
    ppm: f32,
    index: &'a ph2d_skeleton_live::skin_live::BoneIndex,
    rest: &'a ph2d_poly2d::Mesh2d,
}

/// ⭐⭐⭐ **A MALHA DO FANTASMA — a arte deformada pelo esqueleto NAQUELE instante.**
///
/// `None` quando a entidade não é uma imagem presa (o caminho de toda sprite normal: o fantasma é o
/// quad, como sempre) ou quando a pele não resolve ali.
///
/// ⛔⛔ **O fantasma usa SEMPRE a malha guardada, nunca o refinamento do `Smooth`** (`refine: None`).
/// Duas razões, e as duas medidas: uma **silhueta** chapada não tem detalhe que um quarto de pixel
/// de tolerância salve — o que o olho lê ali é a FORMA —, e o orçamento de peças por quadro
/// (`SKIN_FRAME_PIECES`) foi derivado do tempo do quadro para a arte VIVA; `n` fantasmas a refinar
/// comiam-no `n` vezes. *O fantasma é uma leitura, não a obra.*
fn ghost_mesh(
    sim: &SimWorld,
    doc: &TimelineDoc,
    entity: Entity,
    template: &RenderInstance,
    t: f64,
    pele: &PeleDoQuadro<'_>,
) -> Option<ph2d_render::SpriteMesh> {
    let PeleDoQuadro { ppm, index, rest } = *pele;
    // ⚠️ **A pose de MUNDO de cada elo em `t`**, e não a local: um osso é um elo de uma corrente, e
    // a pele responde a poses de mundo. O fecho responde pelos OSSOS e pela própria ARTE — posar só
    // os ossos deixaria a imagem no sítio de agora com o esqueleto no de `t`.
    let poses = |e: Entity| {
        ph2d_vec_entities::transform::xform_of_transform(
            world_pose_at(sim.world(), doc, e.to_bits(), t).unwrap_or(Transform::IDENTITY),
        )
    };
    let (p2l, pele) = ph2d_skeleton_live::skin_image::deform_field_with(
        sim, entity, rest.size, ppm, index, &poses,
    )?;
    ph2d_skeleton_live::skin_image::posed_sprite_mesh(
        rest.clone(),
        p2l,
        &pele,
        template.anchor,
        template.size,
        None,
    )
    .map(|(m, _k)| m)
}

/// ⭐⭐ **UM ALVO DO ONION: o que se DESENHA, e quem tem as KEYS que dizem em que instantes.**
///
/// ⚠️ **Os dois são a mesma entidade em toda cena SEM rig, e por isso a distinção não existia.** Num
/// personagem riggado eles separam-se: desenha-se a imagem, e quem leva keys são os ossos.
pub(crate) struct GhostTarget {
    /// A entidade DESENHADA (a que dá a pose do fantasma e a malha, se tiver pele).
    pub entity: u64,
    /// Os campos de sprite do vivo — textura, uv, tamanho, anchor.
    pub template: RenderInstance,
    /// Quem tem as KEYS (modo `Keys`). Para a arte animada é ela própria; para um rig, os ossos.
    pub relogios: Vec<u64>,
}

/// Constrói TODOS os fantasmas do onion e os acrescenta a `out`. No-op quando desligado ou sem
/// alvos.
///
/// ⭐ **A malha de repouso é descodificada UMA vez por alvo** (os bytes opacos da pele custam
/// `0,134 µs` por peça, medidos na W4) e posada uma vez por instante — não uma vez por par.
pub(crate) fn build_ghosts(
    settings: &OnionSettings,
    sim: &SimWorld,
    doc: &TimelineDoc,
    targets: &[GhostTarget],
    live_clip_t: f64,
    pixels_per_meter: f32,
    out: &mut LiftedInstances,
) {
    if !settings.enabled {
        return;
    }
    let index = ph2d_skeleton_live::skin_live::bone_index(sim);
    let mut pecas = 0usize;
    for GhostTarget {
        entity,
        template,
        relogios,
    } in targets
    {
        let rest = Entity::try_from_bits(*entity)
            .and_then(|e| ph2d_skeleton_live::skin_image::mesh_of(sim, e));
        for (past, color) in [(true, settings.color_before), (false, settings.color_after)] {
            let times = ghost_times(settings, doc, relogios, live_clip_t, past);
            // O falloff é sobre a contagem DE FATO encontrada (em Keys pode faltar key de
            // um lado): o mais próximo é o mais forte, o mais distante encontrado o mais
            // fraco.
            let n = times.len().max(1) as u32;
            for (i, &t) in times.iter().enumerate() {
                let a = ghost_alpha(settings, i as u32 + 1, n);
                let tint = [color[0], color[1], color[2], a];
                if let Some(g) = ghost_instance(sim, doc, *entity, template, t, tint) {
                    let malha =
                        rest.as_ref()
                            .zip(Entity::try_from_bits(*entity))
                            .and_then(|(rest, e)| {
                                let pele = PeleDoQuadro {
                                    ppm: pixels_per_meter,
                                    index: &index,
                                    rest,
                                };
                                ghost_mesh(sim, doc, e, template, t, &pele)
                            });
                    pecas += malha.as_ref().map_or(0, |m| m.tris.len());
                    out.push(g, malha.as_ref());
                }
            }
        }
    }
    // ⚠️ **O DIAGNÓSTICO, e não um tecto.** Medido (2026-09-13, `load 9,6`, mínimo de 40 corridas):
    // `4` fantasmas × `528` peças custam `0,334 ms` — `2,0 %` de um quadro —, logo uma peça de
    // fantasma vale `0,158 µs`. No extremo dos DOIS sliders (`MAX_GHOSTS = 8` de cada lado) sobre
    // uma pele no tecto dela (`SKIN_FRAME_PIECES`), isso é `~3,9 ms`: **`23 %` de um quadro**.
    //
    // ⛔ **Não se corta nada aqui.** O artista pediu `n` fantasmas; deitar fora os mais distantes é
    // uma decisão de PRODUTO, e um tecto que não nomeia o recurso de outra pessoa é um palpite
    // (§0.0). O que fica é o NÚMERO: quem vir o quadro engasgar com o onion ligado tem-no no log da
    // família, ao lado do orçamento que a pele viva declara para si.
    if pecas > ph2d_skeleton_live::skin_image::SKIN_FRAME_PIECES
        && std::env::var_os("PH2D_BONE_LOG").is_some()
    {
        eprintln!(
            "[bone] onion: {pecas} pecas de fantasma neste quadro (o orcamento da pele VIVA e' {}) \
             — ~{:.2} ms so' nos fantasmas",
            ph2d_skeleton_live::skin_image::SKIN_FRAME_PIECES,
            pecas as f64 * 0.158 / 1000.0
        );
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

/// ⭐⭐⭐ **O QUE O SELECCIONADO FAZ MOVER** — os alvos do onion, com o `RenderInstance` vivo de cada
/// um.
///
/// Duas respostas, e a segunda é a que faltava:
///
/// - o seleccionado **é a arte**, e está animado ⇒ ele próprio (o escopo do ADR-0142);
/// - o seleccionado é um **OSSO** ⇒ as **imagens presas ao esqueleto dele**, se alguma coisa naquele
///   esqueleto estiver animada.
///
/// ⛔⛔ **A segunda não é um alargamento do escopo: é o escopo aplicado a um rig.** *«Edita-se o que
/// está na mão»* — e o que o animador tem na mão quando posa é um osso, cuja silhueta é a arte. Sem
/// ela o onion de um personagem riggado mostrava **nada**, e por duas razões que se somam: a imagem
/// não está animada (quem leva keys são os ossos) e o osso não tem instância de desenho. *Um recurso
/// cujas duas guardas se excluem uma à outra está desligado, não configurado.*
///
/// ⚠️ **A condição de animação é do ESQUELETO, não do osso na mão.** O animador escolhe o osso que
/// vai posar — que pode ainda não ter key nenhuma — e o que ele quer ver é o passado da personagem.
fn ghost_targets(
    sim: &SimWorld,
    present: &mut PresentWorld,
    doc: &TimelineDoc,
    sel: u64,
) -> Vec<GhostTarget> {
    let animadas = animated_entities(doc);
    if animadas.contains(&sel)
        && let Some(template) = live_template(present, sel)
    {
        return vec![GhostTarget {
            entity: sel,
            template,
            relogios: vec![sel],
        }];
    }
    let Some(osso) = Entity::try_from_bits(sel)
        .filter(|&e| sim.world().get::<ph2d_skeleton_ecs::Bone>(e).is_some())
    else {
        return Vec::new();
    };
    let index = ph2d_skeleton_live::skin_live::bone_index(sim);
    // ⭐⭐⭐ **OS RELÓGIOS são os ossos ANIMADOS deste esqueleto** — quem tem as keys. ⛔ Sem eles o
    // modo `Keys` (o de OMISSÃO) devolvia lista vazia: ele pergunta as keyframes do alvo, e a
    // imagem não tem nenhuma. *A pergunta certa é «quem MOVE isto?», e ela vale nos DOIS sítios —
    // no escopo e nos instantes.*
    let relogios: Vec<u64> = ph2d_skeleton_live::skin_live::skeleton_of(sim, Some(osso))
        .iter()
        .map(|b| b.to_bits())
        .filter(|b| animadas.contains(b))
        .collect();
    if relogios.is_empty() {
        return Vec::new();
    }
    ph2d_skeleton_live::skin_live::skinned_images_of_skeleton(sim, osso, &index)
        .into_iter()
        .filter_map(|e| {
            Some(GhostTarget {
                entity: e.to_bits(),
                template: live_template(present, e.to_bits())?,
                relogios: relogios.clone(),
            })
        })
        .collect()
}

/// **A porta do shell:** monta os fantasmas do onion do que o objeto SELECIONADO faz mover e os
/// acrescenta a `out`. No-op quando desligado, sem seleção, ou quando nada do que ele dirige tem
/// passado e futuro a mostrar.
///
/// O escopo é o SELECIONADO (ADR-0142): edita-se o que está na mão, como o motion path — e o que
/// um OSSO tem na mão é a arte que ele deforma ([`ghost_targets`]).
///
/// ⚠️ **Os argumentos são os FACTOS do quadro**, como nos irmãos deste ficheiro e no
/// `snapshots::publish`: agrupá-los numa struct aqui só mudaria o sítio onde eles são escritos (a
/// fase que chama tem-nos todos soltos na mão, e o `gfx` está emprestado ao redor).
#[allow(clippy::too_many_arguments)]
pub(crate) fn collect_onion_ghosts(
    settings: &OnionSettings,
    sim: &SimWorld,
    present: &mut PresentWorld,
    doc: &TimelineDoc,
    selected: Option<u64>,
    live_clip_t: f64,
    pixels_per_meter: f32,
    out: &mut LiftedInstances,
) {
    if !settings.enabled {
        return;
    }
    let Some(sel) = selected else { return };
    let alvos = ghost_targets(sim, present, doc, sel);
    if alvos.is_empty() {
        return;
    }
    build_ghosts(
        settings,
        sim,
        doc,
        &alvos,
        live_clip_t,
        pixels_per_meter,
        out,
    );
}

#[cfg(test)]
#[path = "timeline_onion_tests.rs"]
mod timeline_onion_tests;
