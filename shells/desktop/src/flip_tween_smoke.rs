//! **A cena pronta para o smoke do TWEEN v2** (`PH2D_FLIP_TWEEN_SMOKE=1`).
//!
//! Duas chaves com um bonequinho, e a arte é escolhida para que **cada metade da wave
//! tenha um jeito de estar errada que se VÊ**:
//!
//! 1. **O BRAÇO GIRA 120° em torno do ombro.** Com o lerp de coordenadas (o v1, e o que o
//!    Grease Pencil faz), o inbetween do meio é a corda entre as duas poses e o braço
//!    **encolhe para ~57%**. Com a espiral, ele percorre o ARCO com o comprimento inteiro.
//!    *Se o braço encolher no meio, a espiral não está rodando.*
//! 2. **A chave B foi desenhada na ORDEM TROCADA** (perna primeiro, braço depois). Com o
//!    pareamento por índice, o braço interpola contra a perna e o inbetween é um borrão
//!    atravessando o corpo. *Se algum traço atravessar a figura, a correspondência não
//!    está rodando.*
//! 3. **O CHAPÉU só existe na chave A** (a figura o perde no caminho). Com **Fade**
//!    desmarcado ele é cópia estática; marcado, ele **some esmaecendo E VIAJA junto com a
//!    cabeça**. *Se ele ficar pregado no ar enquanto a cabeça anda, a advecção do órfão não
//!    está rodando.*
//!
//! A cena **ARMA o gesto**: as duas chaves existem, o playhead está na 0, e o `Tween` da
//! barra já está em 3. Basta apertar **Add**.

use ph2d_core::Vec2;
use ph2d_flip::{FlipStroke, Hold, KeyKind, Point, Rgba};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};

static FRAME: AtomicU32 = AtomicU32::new(0);

fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("PH2D_FLIP_TWEEN_SMOKE").is_some())
}

const INK: Rgba = Rgba::new(0.92, 0.92, 0.95, 1.0);

/// Uma polilinha amostrada em `n` pontos, na espessura do traço de linha.
fn line(pts: &[Vec2]) -> FlipStroke {
    let mut s = FlipStroke::new();
    for p in pts {
        s.push_point(Point {
            pos: *p,
            width: 0.22,
            opacity: 1.0,
            color: INK,
        });
    }
    s.hardness = 0.4;
    s
}

fn seg(a: Vec2, b: Vec2, n: usize) -> Vec<Vec2> {
    (0..n)
        .map(|i| {
            let t = i as f32 / (n - 1) as f32;
            Vec2::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t)
        })
        .collect()
}

/// Gira `p` de `deg` graus em torno de `c` — o braço da pose B.
fn turn(p: Vec2, c: Vec2, deg: f32) -> Vec2 {
    let r = deg.to_radians();
    let (s, co) = (r.sin(), r.cos());
    let d = p - c;
    c + Vec2::new(co * d.x - s * d.y, s * d.x + co * d.y)
}

/// O ombro — o pivô que o braço tem de manter parado durante o arco.
const SHOULDER: Vec2 = Vec2::new(0.0, 1.2);

/// **Monta as duas chaves no objeto** — a arte da cena, numa porta só.
///
/// O gate abaixo encena pela MESMA função: uma cena de smoke que afirma três coisas e um
/// gate que monta a arte por conta própria são dois fixtures que divergem, e aí a mensagem
/// impressa passa a descrever um desenho que ninguém mais produz.
pub(crate) fn stage(obj: &mut ph2d_flip::FlipObject) -> ph2d_flip::LayerId {
    let l = obj.add_layer("L");
    let torso = || line(&seg(Vec2::new(0.0, 1.2), Vec2::new(0.0, -0.6), 8));
    let leg = |dx: f32| line(&seg(Vec2::new(0.0, -0.6), Vec2::new(dx, -2.2), 8));
    let arm_a: Vec<Vec2> = seg(SHOULDER, Vec2::new(1.8, 1.2), 10);
    let hat = |c: Vec2| {
        line(&[
            c + Vec2::new(-0.5, 0.0),
            c + Vec2::new(0.5, 0.0),
            c + Vec2::new(0.32, 0.5),
            c + Vec2::new(-0.32, 0.5),
            c + Vec2::new(-0.5, 0.0),
        ])
    };

    // ── CHAVE 0: corpo · braço para +X · perna · chapéu ──────────────────────
    if let Some(d0) = obj.insert_frame(l, 0, Hold::Implicit, KeyKind::Keyframe) {
        let dr = obj.drawing_mut(d0).expect("desenho");
        dr.strokes.push(torso());
        dr.strokes.push(line(&arm_a)); // o braço é o traço 1 em A …
        dr.strokes.push(leg(-0.7));
        dr.strokes.push(hat(Vec2::new(0.0, 1.4))); // … e o chapéu, o 3
    }
    // ── CHAVE 8: braço girado 120°, perna do outro lado, chapéu SUMIU — e a
    //    ordem de desenho TROCADA (perna, corpo, braço). ────────────────────
    if let Some(d8) = obj.insert_frame(l, 8, Hold::Implicit, KeyKind::Keyframe) {
        let dr = obj.drawing_mut(d8).expect("desenho");
        dr.strokes.push(leg(0.7)); // a perna vem PRIMEIRO em B
        dr.strokes.push(torso());
        let arm_b: Vec<Vec2> = arm_a.iter().map(|p| turn(*p, SHOULDER, 120.0)).collect();
        dr.strokes.push(line(&arm_b));
    }
    l
}

impl crate::App {
    /// Roda no prólogo do frame (ao lado dos outros smokes). No-op sem a env.
    pub(crate) fn flip_tween_smoke(&mut self) {
        if !enabled() || self.gfx.is_none() {
            return;
        }
        if FRAME.fetch_add(1, Ordering::Relaxed) != 3 {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("flip"));

        let oid = gfx.flip.push_object("Tween Smoke");
        let obj = gfx.flip.object_mut(oid).expect("objeto recém-criado");
        obj.fps = 12.0;
        stage(obj);

        self.flip_strip.tween_count = 3;
        self.playhead.seek(0.0);
        self.playhead.pause();

        // ⚠️ **A cena DIZ o que construiu** — sem isto, um Add que gera inbetweens tortos é
        // indistinguível de uma cena montada errada ([[feedback_ready_to_smoke_example]]).
        eprintln!(
            "\n[tween-smoke] cena montada: um bonequinho de palito em 2 quadros \
             (0 e 8), Tween ja esta em 3."
        );
        eprintln!(
            "\n\
             O QUE ESTA NA TELA\n\
             ==================\n\
             Um boneco de palito: um TRONCO em pe, um BRACO saindo do ombro para a\n\
             direita (como o ponteiro de um relogio marcando 3 horas), uma PERNA para a\n\
             esquerda, e um CHAPEUZINHO em cima.\n\
             \n\
             Ele tem so DOIS desenhos: o quadro 0 (esse) e o quadro 8. No quadro 8 o\n\
             braco girou para cima-e-para-a-esquerda (marcando ~10 horas), a perna trocou\n\
             de lado, e o chapeu sumiu.\n\
             \n\
             O QUE FAZER\n\
             ===========\n\
             Aperte **Add** na barra da tira. Ele inventa os 3 quadros do meio (2, 4, 6).\n\
             Depois use as setas ^/v (ou clique nas celulas da tira) para folhear\n\
             0 -> 2 -> 4 -> 6 -> 8, ida e volta. E o boneco se mexendo.\n\
             \n\
             O QUE OLHAR (3 coisas, todas no quadro 4 -- o do meio)\n\
             ======================================================\n\
             \n\
             1) O BRACO TEM DE FICAR DO MESMO TAMANHO.\n\
             \n\
                CERTO  : ele varre um arco, como o ponteiro do relogio indo das 3 para as\n\
                         10 horas. Sempre do mesmo comprimento.\n\
                ERRADO : ele ENCOLHE ate a METADE no quadro 4 e volta a crescer -- como\n\
                         uma antena de radio recolhendo e saindo de novo.\n\
             \n\
                (Esse encolhimento e' o que o Blender faz, e o que nos faziamos ate ontem:\n\
                 a ponta do braco corta o caminho em linha reta em vez de dar a volta.)\n\
             \n\
             2) O TRONCO NAO PODE SE MEXER, NEM UM POUCO.\n\
             \n\
                Ele foi desenhado IGUAL nos dois quadros, entao tem de ficar parado.\n\
             \n\
                Se ele escorregar para baixo ou para o lado, e' porque o programa casou o\n\
                tronco com a PERNA -- eu desenhei o quadro 8 na ordem trocada (perna\n\
                primeiro) de proposito, que e' o que um animador faz sem pensar.\n\
             \n\
             3) O CHAPEU (marque **Fade** e aperte Add de novo).\n\
             \n\
                SEM Fade : o chapeu fica parado e inteiro ate o quadro 8, onde some de\n\
                           uma vez. E' o padrao.\n\
                COM Fade : ele vai ficando transparente quadro a quadro E acompanha o\n\
                           movimento -- em vez de ficar pregado no ar enquanto o resto do\n\
                           boneco se mexe embaixo dele.\n\
             \n\
             4) (opcional) O chip **Ease** muda o RITMO, nao o caminho: com 'Ease In' o\n\
                boneco comeca devagar e acelera; com 'Ease Out', o contrario. Aperte Add\n\
                de novo depois de trocar -- ele refaz, nao empilha.\n"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_flip::{FlipDoc, TweenOptions, TweenRequest};

    /// Comprimento de arco do traço `i` do desenho no quadro `f`.
    fn arc_len(
        doc: &FlipDoc,
        oid: ph2d_flip::FlipObjectId,
        l: ph2d_flip::LayerId,
        f: i32,
        i: usize,
    ) -> f32 {
        let obj = doc.object(oid).expect("objeto");
        let d = obj
            .layer(l)
            .expect("camada")
            .drawing_at(f)
            .expect("desenho");
        obj.drawing(d).expect("arte").strokes[i]
            .positions()
            .windows(2)
            .map(|w| (w[1] - w[0]).length())
            .sum()
    }

    fn staged(fade: bool) -> (FlipDoc, ph2d_flip::FlipObjectId, ph2d_flip::LayerId) {
        let mut doc = FlipDoc::default();
        let oid = doc.push_object("T");
        let obj = doc.object_mut(oid).expect("objeto");
        let l = stage(obj);
        obj.tween(TweenRequest {
            layer: l,
            from: 0,
            to: 8,
            count: 3,
            options: TweenOptions {
                fade_orphans: fade,
                ..Default::default()
            },
        });
        (doc, oid, l)
    }

    /// **O que a cena MANDA o artista olhar, medido.**
    ///
    /// Uma cena de smoke que afirma três coisas sem que ninguém as tenha medido é uma
    /// mensagem que pode estar descrevendo outro produto (esta linha já pagou isso duas
    /// vezes). Este gate encena pela MESMA `stage()` e afirma os itens (a), (b) e (c) da
    /// mensagem impressa.
    #[test]
    fn the_smoke_scene_shows_what_its_message_promises() {
        let (doc, oid, l) = staged(false);

        // (a) O BRAÇO não encolhe. Ele é o traço 1 em A (corpo, BRAÇO, perna, chapéu).
        let full = arc_len(&doc, oid, l, 0, 1);
        for f in [2, 4, 6] {
            let mid = arc_len(&doc, oid, l, f, 1);
            assert!(
                (mid - full).abs() < full * 0.05,
                "o braço encolheu no quadro {f}: {mid:.2} de {full:.2} — a espiral não rodou"
            );
        }

        // (b) O OMBRO fica parado (é o ponto fixo do arco).
        let obj = doc.object(oid).expect("objeto");
        for f in [2, 4, 6] {
            let d = obj
                .layer(l)
                .expect("camada")
                .drawing_at(f)
                .expect("desenho");
            let p = obj.drawing(d).expect("arte").strokes[1].positions()[0];
            assert!(
                (p - SHOULDER).length() < 0.05,
                "o ombro andou no quadro {f}: {p:?} (o pivô é {SHOULDER:?})"
            );
        }

        // (c) NADA atravessa a figura: o braço fica no semiplano que o arco de 0→120°
        //     percorre (y ≥ ombro), nunca lá embaixo com a perna.
        for f in [2, 4, 6] {
            let d = obj
                .layer(l)
                .expect("camada")
                .drawing_at(f)
                .expect("desenho");
            let tip = *obj.drawing(d).expect("arte").strokes[1]
                .positions()
                .last()
                .expect("ponta");
            assert!(
                tip.y >= SHOULDER.y - 0.05,
                "a ponta do braço desceu para {tip:?} no quadro {f} — ele foi pareado com a \
                 perna (a correspondência não rodou)"
            );
        }
    }

    /// **(d) O chapéu órfão VIAJA com a cabeça quando o Fade está marcado** — e fica parado
    /// quando não está. As duas metades, porque "viaja" só significa alguma coisa contra o
    /// controle que não viaja.
    #[test]
    fn the_orphan_hat_travels_only_when_fade_is_armed() {
        let x_of = |doc: &FlipDoc, oid, l, f: i32| -> Option<f32> {
            let obj = doc.object(oid)?;
            let d = obj.layer(l)?.drawing_at(f)?;
            let art = obj.drawing(d)?;
            // O chapéu é o único traço com 5 pontos (o retângulo fechado do fixture).
            art.strokes
                .iter()
                .find(|s| s.len() == 5)
                .map(|s| s.positions().iter().map(|p| p.x).sum::<f32>() / 5.0)
        };
        let (still, oid, l) = staged(false);
        let (moved, oid2, l2) = staged(true);
        // ⚠️ O oráculo é o chapéu de A, não o zero: o polígono do fixture REPETE o ponto de
        // fecho, então a média em x dele é −0,1 e não 0 — a 1ª versão deste gate media
        // contra um número que eu supus em vez de ler do fixture.
        let rest = x_of(&still, oid, l, 0).expect("a chave A tem chapéu");
        let (a, b) = (
            x_of(&still, oid, l, 4).expect("o chapéu existe sem fade"),
            x_of(&moved, oid2, l2, 4).expect("o chapéu existe com fade"),
        );
        assert!(
            (a - rest).abs() < 0.01,
            "sem Fade o chapéu é cópia estática: x={a:.3}, e em A está em {rest:.3}"
        );
        assert!(
            (b - a).abs() > 0.1,
            "com Fade o chapéu deveria VIAJAR com o vizinho: x={b:.3} (parado é {a:.3})"
        );
    }
}

/// **A SONDA: o que o artista vai ver, em números** (render-and-look, headless).
///
/// `cargo test -p ph2d-host-desktop --release the_tween_smoke_look -- --ignored --nocapture`
///
/// Existe porque a mensagem que a cena imprime **descreve uma tela**, e uma descrição de
/// tela escrita sem olhar é exatamente como um smoke passa a mandar o artista procurar a
/// coisa errada. Ela roda o MESMO `stage()` + o MESMO `tween`, e imprime, quadro a quadro,
/// as três coisas que a mensagem manda conferir.
#[cfg(test)]
#[test]
#[ignore = "sonda: imprime o que a cena de smoke mostra, quadro a quadro"]
fn the_tween_smoke_look() {
    use ph2d_flip::{FlipDoc, TweenOptions, TweenRequest};

    for fade in [false, true] {
        let mut doc = FlipDoc::default();
        let oid = doc.push_object("T");
        let obj = doc.object_mut(oid).expect("objeto");
        let l = stage(obj);
        obj.tween(TweenRequest {
            layer: l,
            from: 0,
            to: 8,
            count: 3,
            options: TweenOptions {
                fade_orphans: fade,
                ..Default::default()
            },
        });
        println!(
            "\n══ Fade {} ══════════════════════════════════════════════",
            if fade { "MARCADO" } else { "desmarcado" }
        );
        println!("  quadro   ponta do braço      compr.   (o lerp daria)   chapéu x   opacidade");
        let obj = doc.object(oid).expect("objeto");
        for f in [0, 2, 4, 6, 8] {
            let Some(d) = obj.layer(l).expect("camada").drawing_at(f) else {
                continue;
            };
            let art = obj.drawing(d).expect("arte");
            // O braço: o traço de 10 pontos que sai do ombro.
            let arm = art
                .strokes
                .iter()
                .find(|s| s.len() == 10)
                .expect("o braço está lá");
            let tip = *arm.positions().last().expect("ponta");
            let len: f32 = arm
                .positions()
                .windows(2)
                .map(|w| (w[1] - w[0]).length())
                .sum();
            // O que o lerp puro daria: a corda entre as duas poses no mesmo t.
            let t = f as f32 / 8.0;
            let lerp_len = 1.8
                * (1.0 - t + t * (120f32).to_radians().cos())
                    .hypot(t * (120f32).to_radians().sin());
            // O chapéu: o único traço de 5 pontos.
            let hat = art.strokes.iter().find(|s| s.len() == 5);
            let (hx, ho) = hat.map_or((f32::NAN, f32::NAN), |s| {
                (
                    s.positions().iter().map(|p| p.x).sum::<f32>() / 5.0,
                    s.opacities()[0],
                )
            });
            println!(
                "  {f:^6}   ({:5.2}, {:5.2})     {len:5.2}       {lerp_len:5.2}        \
                 {hx:6.2}      {ho:5.2}",
                tip.x, tip.y
            );
        }
    }
    println!(
        "\n  o ombro está em (0.00, 1.20) — a base do braço não pode sair dali.\n\
         'chapéu x' NaN = o chapéu não existe naquele quadro.\n"
    );
}
