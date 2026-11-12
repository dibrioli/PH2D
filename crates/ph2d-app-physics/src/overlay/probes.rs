//! **Os sensores na tela** (`W-Probes`): onde cada um olha (o ALCANCE, sempre: o censo `measure_player_probes`
//! mediu que a parede só é perguntada em 13,5 % dos tiques e a quina em 6 %) e o que respondeu.
//!
//! O ESTADO lê-se pela intensidade (parado · nada · viu) e por um tique na distância do acerto. Os sensores do
//! personagem têm cor fixa; o `RaySensor` do objecto veste a cor da sua CAMADA, com a sombra do contorno (emenda 4
//! da GEF02, dono 08/11; `ph2d_shape_handles::contorno::Tinta::Sonda`).
//!
//! Nada aqui deriva geometria: toda posição vem da [`ph2d_physics_ecs::ProbeMark`] que a ponte publica, preenchida
//! pelas mesmas portas que lançam os raios.

use ph2d_ecs::SimWorld;
use ph2d_host::WindowSize;
use ph2d_physics_ecs::{Collider, ProbeKind, ProbeMark, ProbeShape, ProbeState};
use ph2d_render::Camera2d;
use ph2d_vector::{BezPath, Point};

use crate::overlay::shapes::shape_outline;
use ph2d_shape_handles::contorno::Tinta;

/// As marcas do personagem: cor fixa, sem camada.
fn fixa(v: Vec<(BezPath, [f32; 4])>) -> impl Iterator<Item = (BezPath, Tinta)> {
    v.into_iter()
        .map(|(p, rgba)| (p, Tinta::Sonda { camada: None, rgba }))
}

/// **O hue dos sensores** — aço frio, de baixa croma.
///
/// Escolhido por **não** disputar com nenhuma das nove famílias saturadas que o
/// overlay já usa (ver o cabeçalho): um sensor é informação de fundo sobre onde
/// o personagem OLHA, não um evento.
const PROBE_RGB: [f32; 3] = [0.60, 0.76, 0.92]; // LITERAL-COLOR-OK: overlay de sensor

/// A lei inteira do estado, num lugar: **inerte · perguntou · achou**.
///
/// ⚠️ O `Idle` é a metade que o artista não consegue ver hoje — ele responde
/// *"a capacidade está desligada / não é a hora"* contra *"o alcance é curto
/// demais"*, dois vereditos opostos que produzem o mesmo nada na tela.
fn probe_alpha(state: ProbeState) -> f32 {
    match state {
        ProbeState::Idle => 0.45,  // LITERAL-ALPHA-OK: overlay de sensor
        ProbeState::Clear => 0.72, // LITERAL-ALPHA-OK: overlay de sensor
        ProbeState::Hit => 1.0,    // LITERAL-ALPHA-OK: overlay de sensor
    }
}

/// Meio comprimento do tique que marca ONDE o sensor achou, em px de tela.
const HIT_TICK_PX: f64 = 5.0; // LITERAL-PX-OK: chrome de overlay

/// Meio comprimento do tique que fecha a PONTA do alcance, em px de tela.
///
/// ⚠️ **Menor que o do acerto de propósito, e é o que os mantém distinguíveis:**
/// a ponta diz *até onde ele olha* (o número autorado) e o tique de acerto diz
/// *onde ele achou* (a resposta) — se medissem igual, um raio que acha
/// exactamente no fim do alcance seria indistinguível de um que não achou nada.
const END_TICK_PX: f64 = 3.0; // LITERAL-PX-OK: chrome de overlay

/// Espessura das marcas de sensor, em px de tela: mais fina que o contorno (um sensor é uma pergunta).
pub const PROBE_PX: f64 = ph2d_shape_handles::contorno::SONDA_PX;

/// A ordem entre as duas espessuras, afirmada em tempo de compilação.
const _: () = assert!(PROBE_PX < crate::overlay::outline::OUTLINE_PX);

/// **O que desenhar para os sensores deste quadro**: pares `(path, tinta)` para o `stroke` sob `Affine::IDENTITY`
/// (espaço de TELA: no Vello o transform do `stroke` multiplica a largura). `show` falso (o interruptor do
/// contorno, tecla `B`) devolve vazio.
pub fn probe_marks(
    show: bool,
    marks: &[ProbeMark],
    sim: &SimWorld,
    camera: &Camera2d,
    window: WindowSize,
) -> Vec<(BezPath, Tinta)> {
    if !show || marks.is_empty() {
        return Vec::new();
    }
    let to_screen = |wx: f32, wy: f32| {
        let (sx, sy) = camera.world_to_screen([wx, wy], window);
        Point::new(f64::from(sx), f64::from(sy))
    };
    let mut out = Vec::new();
    for m in marks {
        let a = probe_alpha(m.state);
        let rgba = [PROBE_RGB[0], PROBE_RGB[1], PROBE_RGB[2], a];
        match m.shape {
            ProbeShape::Ray {
                origin,
                dir,
                reach,
                hit,
                skin,
            } => {
                let mut p = BezPath::new();
                // ⚠️ **O desenho começa na BORDA do corpo, não no centro.** O
                // cast parte do centro porque o `exclude_body` precisa disso;
                // medido, num raio de parede 20 dos 35 px caíam por baixo do (…)
                let skin = skin.clamp(0.0, reach);
                let from = [origin[0] + dir[0] * skin, origin[1] + dir[1] * skin];
                let end = [origin[0] + dir[0] * reach, origin[1] + dir[1] * reach];
                let s = to_screen(from[0], from[1]);
                let e = to_screen(end[0], end[1]);
                p.move_to(s);
                p.line_to(e);
                // ⚠️ Os TIQUES são desenhados em px de TELA e não em mundo: são
                // chrome (têm de medir o mesmo em qualquer zoom), enquanto o raio
                // é geometria (mede o alcance real).
                let (vx, vy) = (e.x - s.x, e.y - s.y);
                let n = (vx * vx + vy * vy).sqrt();
                // ⚠️ **Uma parede da área andável não tem ponta nem acerto** (plano 30, W4): ela é
                // um troço de um CONTORNO, e um tique em cada vértice picotava o mapa.
                if n > f64::EPSILON && m.kind != ProbeKind::NavEdge {
                    let tick = |p: &mut BezPath, at: Point, half: f64| {
                        let (px, py) = (-vy / n * half, vx / n * half);
                        p.move_to(Point::new(at.x - px, at.y - py));
                        p.line_to(Point::new(at.x + px, at.y + py));
                    };
                    // A PONTA — sem ela um alcance curto é um risco solto, e um
                    // risco solto de 15 px não lê como uma medida.
                    tick(&mut p, e, END_TICK_PX);
                    if let Some(d) = hit {
                        let at = to_screen(origin[0] + dir[0] * d, origin[1] + dir[1] * d);
                        tick(&mut p, at, HIT_TICK_PX);
                    }
                }
                out.push((
                    p,
                    Tinta::Sonda {
                        camada: m.layer,
                        rgba,
                    },
                ));
            }
            ProbeShape::Profile {
                top,
                half_width,
                reach,
                rise,
                ref blocked,
                samples,
            } => {
                out.extend(fixa(profile_path(
                    top, half_width, reach, rise, blocked, samples, m.state, &to_screen,
                )));
            }
            ProbeShape::Sweep { body, offset } => {
                out.extend(fixa(sweep_ghost(sim, body, offset, rgba, camera, window)));
            }
        }
    }
    out
}

/// O **PERFIL** do teto: a barra do vão que ele varre, com as células tapadas
/// destacadas, e as duas hastes que mostram quanto ele sobe.
///
/// ⚠️ **Os deslocamentos vêm da porta da LEI** ([`corner_offsets`]) — desenhar 65
/// posições com uma aritmética local seria a segunda resposta a *"onde cada raio
/// do leque nasce?"*, e meia célula de desvio poria o desenho ao lado do que a
/// assistência de facto leu.
///
/// [`corner_offsets`]: ph2d_physics_ecs::corner_offsets
#[allow(clippy::too_many_arguments)]
fn profile_path(
    top: [f32; 2],
    half_width: f32,
    reach: f32,
    rise: f32,
    blocked: &[bool],
    samples: usize,
    state: ProbeState,
    to_screen: &impl Fn(f32, f32) -> Point,
) -> Vec<(BezPath, [f32; 4])> {
    // ⚠️ **O VÃO nunca veste o brilho de acerto, e é isso que torna o perfil
    // legível.** O estado do perfil é *achou* assim que UMA célula está tapada;
    // se a barra inteira acendesse junto, a resposta (*onde* há teto) ficaria (…)
    let span_a = probe_alpha(if state == ProbeState::Idle {
        ProbeState::Idle
    } else {
        ProbeState::Clear
    });
    let rgba = [PROBE_RGB[0], PROBE_RGB[1], PROBE_RGB[2], span_a];
    let offs = ph2d_physics_ecs::corner_offsets(half_width, reach, samples);
    let cells = samples.clamp(1, offs.len());
    let y = top[1] + rise;
    let mut out = Vec::new();

    // A barra do vão, na altura que o leque alcança, mais as duas hastes que a
    // ligam ao topo da cabeça (é isso que mostra quanto ele olha para CIMA).
    // (…)
    let mut span = BezPath::new();
    let (lo, hi) = (top[0] + offs[0], top[0] + offs[cells - 1]);
    let (a, b) = (to_screen(lo, y), to_screen(hi, y));
    span.move_to(a);
    span.line_to(b);
    span.move_to(to_screen(lo, top[1]));
    span.line_to(a);
    span.move_to(to_screen(hi, top[1]));
    span.line_to(b);
    let (vx, vy) = (b.x - a.x, b.y - a.y);
    let n = (vx * vx + vy * vy).sqrt();
    if n > f64::EPSILON {
        let (px, py) = (-vy / n * END_TICK_PX, vx / n * END_TICK_PX);
        for at in [a, b] {
            span.move_to(Point::new(at.x - px, at.y - py));
            span.line_to(Point::new(at.x + px, at.y + py));
        }
    }
    out.push((span, rgba));

    // As células TAPADAS, como hastes verticais próprias — a resposta do sensor.
    // Sem obstrução nenhuma esta parte não existe, que é o certo: um teto livre
    // não tem nada a dizer além do vão.
    let mut hits = BezPath::new();
    let mut any = false;
    for (b, off) in blocked.iter().zip(offs.iter()).take(cells) {
        if !*b {
            continue;
        }
        any = true;
        hits.move_to(to_screen(top[0] + off, top[1]));
        hits.line_to(to_screen(top[0] + off, y));
    }
    if any {
        out.push((
            hits,
            [rgba[0], rgba[1], rgba[2], probe_alpha(ProbeState::Hit)],
        ));
    }
    out
}

/// A **VARREDURA** do agachar: o corpo desenhado onde ele quer ficar de pé.
///
/// ⚠️ **A forma sai da MESMA porta do contorno vivo** (`shape_outline`),
/// deslocada pelo `offset` que o sensor varreu — é isso que
/// torna o fantasma *o que o produto perguntou* em vez de uma segunda ideia de
/// como aquele corpo é. Um overlay que só soubesse linhas desenharia este sensor
/// como um raio que ele não é, e o artista afinaria o `crouch_height` contra um
/// desenho que mente.
///
/// ⚠️ **Desenha TODAS as formas do corpo** (W-Compound: um corpo tem várias, e a
/// varredura considera todas) — a premissa *"um corpo tem exatamente um
/// collider"* já custou quatro defeitos a esta linha.
fn sweep_ghost(
    sim: &SimWorld,
    body: ph2d_ecs::Entity,
    offset: [f32; 2],
    rgba: [f32; 4],
    camera: &Camera2d,
    window: WindowSize,
) -> Vec<(BezPath, [f32; 4])> {
    let world = sim.world();
    let mut chain = Vec::new();
    let mut out = Vec::new();
    let mut q = world
        .try_query::<(ph2d_ecs::Entity, &Collider)>()
        .expect("query de collider");
    for (e, col) in q.iter(world) {
        let mine = e == body || ph2d_physics_ecs::owner_body(world, e) == Some(body);
        if !mine {
            continue;
        }
        let Some(t) = ph2d_ecs::world_transform_into(world, e, &mut chain) else {
            continue;
        };
        // O MESMO dobramento de offset do contorno vivo: a escala é sincada e a
        // rotação do corpo gira o deslocamento.
        let c = ph2d_physics_ecs::forma_resolvida(world, e, col).offset;
        let (ox, oy) = (c[0] * t.scale.x, c[1] * t.scale.y);
        let (sin_r, cos_r) = t.rotation.sin_cos();
        let (wox, woy) = (ox * cos_r - oy * sin_r, ox * sin_r + oy * cos_r);
        out.push((
            shape_outline(
                world,
                e,
                col.shape,
                t.scale,
                t.translation.x + wox + offset[0],
                t.translation.y + woy + offset[1],
                t.rotation,
                camera,
                window,
            ),
            rgba,
        ));
    }
    out
}

#[cfg(test)]
#[path = "probes_tests.rs"]
mod tests;
