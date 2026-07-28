//! **O NÚMERO ao lado do joint** (W-J7b) — o que ele está segurando, e o que
//! ele aguenta.
//!
//! Report do Enio, pós-smoke da W-J7: *"é extremamente difícil configurar o
//! valor exato de quebra que se deseja, necessitando de uma enorme quantidade de
//! tentativas"*. Isso não é afinação, é **informação que falta**
//! ([[feedback_ergonomics_verdict_is_a_design_bug]]): o artista digita um teto,
//! dá Play e recebe uma resposta BINÁRIA — rompeu ou não. Sem saber que carga o
//! joint de fato carrega, escolher o número é busca binária feita à mão.
//!
//! O dado já existia e é EXATO (`PhysicsWorld::joint_load` lê o peso pendurado
//! com razão 1,0000 — a tabela do W-J7). Ele só nunca chegava ao artista.
//!
//! ## O que o readout diz, e por que são esses números
//!
//! | estado | mostra | por quê |
//! |---|---|---|
//! | segurando, sem teto | `58.9 N` | **o número que se DIGITA.** Sem ele o primeiro teto é sempre um chute — e é preciso poder ler a carga ANTES de armar qualquer coisa |
//! | segurando, com teto | `58.9 / 60 N` | a comparação sai da cabeça do artista e vai para a tela |
//! | pico acima do vivo | `+ max 87.2` | um tranco acaba antes de dar para ler; a marca d'água é o que se digita |
//! | **rompido** | `87.2 / 60 N` em VERMELHO | **a carga que provocou a fratura**, ao lado do que estava configurado |
//!
//! ⚠️ **O `max` só aparece quando diz algo novo.** Num rig parado — o caso comum
//! — o pico da corrida É a carga viva, e repetir o mesmo número duas vezes é
//! ruído. Ele nasce quando o joint levou um tranco, que é exatamente quando a
//! carga viva não serve para nada.
//!
//! ⚠️ **Num joint rompido o número principal é o PICO, e ele se congela sozinho.**
//! O wrapper pula um joint desabilitado, então a carga viva de um rompido lê zero
//! enquanto a marca d'água guarda a carga que cruzou — sem caso especial nenhum.
//!
//! ## Quem ganha readout
//!
//! **Um joint QUEBRÁVEL, ou o SELECIONADO.** As duas metades têm motivo próprio:
//! um quebrável tem um teto para comparar (e numa corrente é assim que se vê qual
//! elo está mais perto do dele); o selecionado é o caso do *bootstrap* — para
//! escolher um teto é preciso ler a carga **antes** de armar, e sem essa metade o
//! laço continua começando por um chute.
//!
//! Uma cena sem nada armado e sem seleção não desenha número nenhum.

use ph2d_ecs::Entity;
use ph2d_host::WindowSize;
use ph2d_physics_ecs::JointView;
use ph2d_render::Camera2d;
use ph2d_vector::Point;

use super::physics_overlay_joint_glyphs::screen_of;
use super::physics_overlay_joints::{JOINT_BROKEN_RGBA, JOINT_RGBA};

/// Altura do texto, px de tela. O mesmo corpo dos rótulos de dimensão do Line
/// overlay — pequeno o bastante para não cobrir a arte, grande o bastante para
/// um número de quatro dígitos ser legível sem zoom.
pub(super) const READOUT_PX: f32 = 11.0; // LITERAL-PX-OK: chrome de overlay

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
    s.strip_suffix(".0").map_or(s.clone(), str::to_string)
}

/// Uma linha de readout, pronta para pintar.
pub(super) struct Readout {
    pub(super) text: String,
    pub(super) at: Point,
    pub(super) rgba: [f32; 4],
}

/// **Os rótulos de todos os joints que devem mostrar um.**
///
/// Devolve texto + posição em px de TELA, como todo o resto deste overlay. O
/// desenho é feito pelo chamador depois do último uso do `VectorScene` — a
/// mesma ordem que o overlay de dimensões do Line respeita, porque a cena tem
/// de estar livre para o renderizador de texto.
pub(super) fn joint_readouts(
    show: bool,
    views: &[JointView],
    selected: Option<Entity>,
    camera: &Camera2d,
    window: WindowSize,
) -> Vec<Readout> {
    if !show {
        return Vec::new();
    }
    let mut out = Vec::new();
    for v in views {
        // **Um joint DESLIGADO não mostra carga** (W-J8), e não é higiene: ele
        // não está segurando nada, então o número vivo é zero por construção — e
        // a marca d'água ao lado dele descreveria uma corrida que o próprio
        // interruptor encerrou, que é exatamente a figura de um joint ROMPIDO.
        // Duas coisas diferentes não podem imprimir o mesmo par. O glifo apagado
        // já diz por que não há número.
        if !v.active {
            continue;
        }
        // ⚠️ Aqui havia um `if !v.kind.can_break() { continue }`, e ele existia
        // porque a POLIA não vivia no `ImpulseJointSet`: nada media a reação
        // dela, o par de números era estruturalmente zero, e um zero permanente
        // é a forma de readout que não responde a nada. O W-Pulley W2 fez o
        // passe dela publicar a própria tensão, então a pergunta deixou de ter
        // um NÃO — e um guard que não pode disparar é o que apodrece calado.
        // Quem gateia de verdade é o `breakable` abaixo, que é a pergunta certa:
        // *há um limiar a mostrar ao lado da carga?*
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
            push(format!("max {}", n(v.peak.force)), rgba, &mut line);
        }
    }
    out
}

#[cfg(test)]
#[path = "physics_overlay_joint_readout_tests.rs"]
mod tests;
