//! **O NÚMERO ao lado do joint** (W-J7b) — a carga que ele segura e o que aguenta, para o artista
//! digitar o teto de quebra em vez de o caçar (lido de `PhysicsWorld::joint_load`, exato).
//!
//! Mostra `58.9 N` (sem teto) · `58.9 / 60 N` (com teto) · `+ max 87.2` só quando o pico difere da
//! carga viva · `87.2 / 60 N` em vermelho num joint rompido (o pico congela sozinho).
//! Ganha readout um joint QUEBRÁVEL ou o SELECIONADO; sem nada disso, nenhum número.

use ph2d_ecs::Entity;
use ph2d_host::WindowSize;
use ph2d_physics_ecs::JointView;
use ph2d_render::Camera2d;
use ph2d_vector::Point;

use crate::overlay::joint_glyphs::screen_of;
use crate::overlay::joints::{JOINT_BROKEN_RGBA, JOINT_RGBA};

/// Altura do texto, px de tela. O mesmo corpo dos rótulos de dimensão do Line
/// overlay — pequeno o bastante para não cobrir a arte, grande o bastante para
/// um número de quatro dígitos ser legível sem zoom.
pub const READOUT_PX: f32 = 11.0; // LITERAL-PX-OK: chrome de overlay

/// Deslocamento do texto a partir da âncora, px de tela: para cima e para a
/// direita, fora do glifo (que ocupa o entorno imediato do ponto).
const OFFSET_X_PX: f64 = 12.0; // LITERAL-PX-OK: chrome de overlay
const OFFSET_Y_PX: f64 = -14.0; // LITERAL-PX-OK: chrome de overlay
/// Espaçamento entre a 1ª e a 2ª linha (o `max`, e o torque).
const LINE_PX: f64 = 12.0; // LITERAL-PX-OK: chrome de overlay

/// Acima de quantos por cento o pico tem de estar sobre a carga viva para valer
/// uma linha própria.
///
/// **Não é limiar de gosto: é a pergunta "isto diz algo novo?"**. Num rig parado
/// os dois números são o mesmo e a segunda linha seria ruído; 10% é onde um
/// tranco começa a ser um fato diferente do repouso, e abaixo disso a diferença
/// é o assentamento do solver.
const PEAK_MARGIN: f32 = 1.10;

/// Um número como o artista o lê: uma casa decimal enquanto ele cabe nela,
/// nenhuma quando o valor fica grande (um teto de 1200 N não ganha nada com
/// `1200.0`), e **nenhuma quando ela seria zero**.
///
/// ⚠️ A última regra não é enfeite: o teto é um número que o ARTISTA digitou.
/// Devolvê-lo como `60.0` quando ele escreveu `60` põe um dígito que ninguém
/// pediu bem ao lado do número que muda — e é a carga que tem de puxar o olho.
fn n(v: f32) -> String {
    if !v.is_finite() {
        // Nunca chega aqui pela via do produto (um teto infinito não é
        // desenhado), mas um `inf` impresso seria pior que ausente.
        return "-".to_string();
    }
    if v.abs() >= 100.0 {
        return format!("{v:.0}");
    }
    let s = format!("{v:.1}");
    ph2d_editor_core::widget::zero_sem_sinal(s.strip_suffix(".0").unwrap_or(&s)).into_owned()
}

/// Uma linha de readout, pronta para pintar.
pub struct Readout {
    pub text: String,
    pub at: Point,
    pub rgba: [f32; 4],
}

/// **Os rótulos de todos os joints que devem mostrar um.**
///
/// Devolve texto + posição em px de TELA, como todo o resto deste overlay. O
/// desenho é feito pelo chamador depois do último uso do `VectorScene` — a
/// mesma ordem que o overlay de dimensões do Line respeita, porque a cena tem
/// de estar livre para o renderizador de texto.
pub fn joint_readouts(
    show: bool,
    views: &[JointView],
    selected: Option<Entity>,
    camera: &Camera2d,
    window: WindowSize,
    // As cordas cuja ROTA não resolve (W-RopeSays), publicadas pela MESMA
    // chamada que as desenhou vermelhas.
    not_acting: &[Entity],
) -> Vec<Readout> {
    if !show {
        return Vec::new();
    }
    let mut out = Vec::new();
    for v in views {
        // **Um joint DESLIGADO não mostra carga** (W-J8), e não é higiene: ele
        // não está segurando nada, então o número vivo é zero por construção — e
        // a marca d'água ao lado dele descreveria uma corrida que o próprio (…)
        if !v.active {
            continue;
        }
        // ⚠️ Aqui havia um `if !v.kind.can_break() { continue }`, e ele existia
        // porque a POLIA não vivia no `ImpulseJointSet`: nada media a reação
        // dela, o par de números era estruturalmente zero, e um zero permanente (…)
        let breakable = v.break_force.is_finite() || v.break_torque.is_finite();
        if !breakable && selected != Some(v.entity) {
            continue;
        }
        let a = screen_of(camera, window, v.anchor_a);
        let mut line = 0.0;
        let mut push = |text: String, rgba: [f32; 4], line: &mut f64| {
            out.push(Readout {
                text,
                at: Point::new(a.x + OFFSET_X_PX, a.y + OFFSET_Y_PX + *line * LINE_PX),
                rgba,
            });
            *line += 1.0;
        };
        // **Uma corda que não resolve a rota DIZ ISSO** (W-RopeSays), em vez de
        // um `0 N` em âmbar.
        // (…)
        if not_acting.contains(&v.entity) {
            push(
                ph2d_i18n::tr("app.physics.joint_readout.no_route").to_string(),
                JOINT_BROKEN_RGBA,
                &mut line,
            );
            continue;
        }
        let rgba = if v.broken {
            JOINT_BROKEN_RGBA
        } else {
            JOINT_RGBA
        };
        // ⚠️ Num rompido o número principal é o PICO: a carga viva já é zero
        // (ele não segura mais nada) e o que o artista quer ver é o que cruzou.
        let force = if v.broken { v.peak.force } else { v.load.force };
        push(
            if v.break_force.is_finite() {
                format!("{} / {} N", n(force), n(v.break_force))
            } else {
                format!("{} N", n(force))
            },
            rgba,
            &mut line,
        );
        // O torque só entra quando há um teto dele — fora do Pin ele é
        // estruturalmente zero ou invisível, e um `0.0 N.m` permanente seria um
        // número que não responde a nada.
        if v.break_torque.is_finite() {
            let torque = if v.broken {
                v.peak.torque
            } else {
                v.load.torque
            };
            push(
                format!("{} / {} N.m", n(torque), n(v.break_torque)),
                rgba,
                &mut line,
            );
        }
        // A marca d'água, só quando ela diz algo que a carga viva não diz.
        if !v.broken && v.peak.force > v.load.force * PEAK_MARGIN {
            push(
                ph2d_i18n::tr_with(
                    "app.physics.joint_readout.max",
                    &[("force", &(n(v.peak.force)))],
                ),
                rgba,
                &mut line,
            );
        }
    }
    out
}

#[cfg(test)]
#[path = "joint_readout_tests.rs"]
mod tests;
