//! **A cena pronta para o smoke da TORÇÃO do resíduo** (`PH2D_FLIP_TWEEN_TORSION_SMOKE=1`).
//!
//! O tween v2 tira o movimento RÍGIDO (giro + escala) do traço por uma espiral logarítmica, e
//! soma o RESÍDUO — *o que a similaridade global não explica* — por cima. Até aqui esse resíduo
//! era somado no referencial FIXO do mundo (`+ resid·u`), e sob giro GRANDE ele apontava para a
//! atitude errada no meio do caminho: a forma TORCE. A cura ([`ph2d_flip`] `tween_spiral`)
//! carrega o resíduo no referencial CO-ROTACIONADO do corpo — a feature (uma corcova, um
//! cotovelo) gira JUNTO com o traço.
//!
//! Esta cena arma o caso: uma ASA que gira ~160° em torno do ombro E cuja corcova cresce (o
//! resíduo). Com a co-rotação a corcova do meio fica na atitude parcial certa; sem ela, ela
//! ACHATA (o resíduo somado a 160° cancela contra o corpo a 80°). O gate mede a APARÊNCIA pelo
//! intrínseco (os ângulos de virada — a forma LOCAL, invariante ao giro global).

use ph2d_core::Vec2;
use ph2d_flip::{FlipStroke, Hold, KeyKind, Point, Rgba};
use std::f32::consts::PI;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};

static FRAME: AtomicU32 = AtomicU32::new(0);

fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("PH2D_FLIP_TWEEN_TORSION_SMOKE").is_some())
}

const INK: Rgba = Rgba::new(0.92, 0.92, 0.95, 1.0);
/// Pontos na asa (as MESMAS proporções do fixture do motor, `bumped_arm`: 9 pontos, braço 96).
const N: usize = 9;

/// **Uma ASA**: um braço do ombro (origem) ao longo de `+X` (comprimento 96), com uma CORCOVA
/// perpendicular de amplitude `hump` (perfil `sin`, mean-free — o resíduo PURO), o todo girado
/// `deg` graus em torno do ombro. A e B são a MESMA asa em amplitudes e giros diferentes — a
/// corcova está PRESA ao corpo, e é o giro grande que a faz apontar para o lado errado no meio
/// pelo lerp. É o MESMO fixture (proporções) que o gate do motor usa, na `ph2d-flip`
/// (`bumped_arm`): braço 96, corcova até 34 (≈ 1/3 do braço).
fn wing(hump: f32, deg: f32) -> FlipStroke {
    let (s, co) = deg.to_radians().sin_cos();
    let mut out = FlipStroke::new();
    for i in 0..N {
        let base = Vec2::new(
            i as f32 * 12.0,
            hump * (PI * i as f32 / (N - 1) as f32).sin(),
        );
        out.push_point(Point {
            pos: Vec2::new(co * base.x - s * base.y, s * base.x + co * base.y),
            width: 1.2,
            opacity: 1.0,
            color: INK,
        });
    }
    out
}

/// **Monta as duas chaves** — a asa RETA (quadro 0) e a asa forte girada 160° (quadro 8). O
/// braço 0 sem corcova torna o resíduo o crescimento INTEIRO da corcova (o caso limpo). Porta
/// única: o gate encena por AQUI (senão a mensagem impressa descreveria um desenho que ninguém
/// mais produz).
pub(crate) fn stage(obj: &mut ph2d_flip::FlipObject) -> ph2d_flip::LayerId {
    let l = obj.add_layer("L");
    if let Some(d0) = obj.insert_frame(l, 0, Hold::Implicit, KeyKind::Keyframe) {
        obj.drawing_mut(d0)
            .expect("desenho")
            .strokes
            .push(wing(0.0, 0.0));
    }
    if let Some(d8) = obj.insert_frame(l, 8, Hold::Implicit, KeyKind::Keyframe) {
        obj.drawing_mut(d8)
            .expect("desenho")
            .strokes
            .push(wing(34.0, 160.0));
    }
    l
}

impl crate::App {
    /// Roda no prólogo do frame (ao lado dos outros smokes). No-op sem a env.
    pub(crate) fn flip_tween_torsion_smoke(&mut self) {
        if !enabled() || self.gfx.is_none() {
            return;
        }
        if FRAME.fetch_add(1, Ordering::Relaxed) != 3 {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("flip"));

        let oid = gfx.flip.push_object("Torsion Smoke");
        let obj = gfx.flip.object_mut(oid).expect("objeto recém-criado");
        obj.fps = 12.0;
        stage(obj);

        self.flip_strip.tween_count = 3;
        self.playhead.seek(0.0);
        self.playhead.pause();

        eprintln!(
            "\n[torsion-smoke] cena montada: uma ASA em 2 quadros (0 e 8). Tween ja esta em 3."
        );
        eprintln!(
            "\n\
             O QUE ESTA NA TELA\n\
             ==================\n\
             Uma ASA (um braço saindo do ombro para a DIREITA, quase reto, com uma corcovinha\n\
             de leve para cima). Ela tem so DOIS desenhos: o quadro 0 (esse) e o quadro 8,\n\
             onde a MESMA asa girou ~160 graus (agora aponta para a ESQUERDA-e-para-baixo) E\n\
             ganhou uma corcova bem MAIOR.\n\
             \n\
             O QUE FAZER\n\
             ===========\n\
             Aperte **Add** na barra da tira. Ele inventa os 3 quadros do meio (2, 4, 6).\n\
             Folheie 0 -> 2 -> 4 -> 6 -> 8 com as setas ^/v (ou clicando nas celulas).\n\
             \n\
             O QUE OLHAR (o quadro 4 -- o do meio)\n\
             =====================================\n\
             \n\
             A asa tem de girar meia-viagem (~80 graus) COM a corcova crescendo do lado\n\
             CERTO -- a corcova acompanha o corpo enquanto ele gira.\n\
             \n\
                CERTO  : uma asa meio-girada, com uma corcova de tamanho intermediario\n\
                         apontando para FORA da curva, na atitude do corpo a 80 graus.\n\
                ERRADO : a asa fica quase RETA no meio (a corcova ACHATA/some), ou a\n\
                         corcova aponta para o lado errado -- porque o resíduo (o crescimento\n\
                         da corcova) foi somado a 160 graus enquanto o corpo so girou 80, e\n\
                         os dois se cancelam.\n\
             \n\
             (Esse achatamento e' a torção do resíduo sob giro grande. A co-rotação gira o\n\
              resíduo JUNTO com o corpo, e a corcova do meio aparece inteira e no lugar.)\n"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_flip::{FlipDoc, FlipObjectId, LayerId, TweenOptions, TweenRequest};

    /// Os ângulos de virada com sinal nos vértices internos — a forma LOCAL (Sederberg),
    /// invariante à rotação e à translação globais.
    fn turning(pts: &[Vec2]) -> Vec<f32> {
        pts.windows(3)
            .map(|w| {
                let (e0, e1) = (w[1] - w[0], w[2] - w[1]);
                libm::atan2f(e0.x * e1.y - e0.y * e1.x, e0.x * e1.x + e0.y * e1.y)
            })
            .collect()
    }

    /// `y − x` pelo caminho mais curto.
    fn short(x: f32, y: f32) -> f32 {
        let mut d = y - x;
        while d > PI {
            d -= std::f32::consts::TAU;
        }
        while d <= -PI {
            d += std::f32::consts::TAU;
        }
        d
    }

    fn positions(doc: &FlipDoc, oid: FlipObjectId, l: LayerId, f: i32) -> Vec<Vec2> {
        let obj = doc.object(oid).expect("objeto");
        let d = obj
            .layer(l)
            .expect("camada")
            .drawing_at(f)
            .expect("desenho");
        obj.drawing(d).expect("arte").strokes[0]
            .positions()
            .to_vec()
    }

    fn staged() -> (FlipDoc, FlipObjectId, LayerId) {
        let mut doc = FlipDoc::default();
        let oid = doc.push_object("T");
        let obj = doc.object_mut(oid).expect("objeto");
        let l = stage(obj);
        obj.tween(TweenRequest {
            layer: l,
            from: 0,
            to: 8,
            count: 3,
            options: TweenOptions::default(),
        });
        (doc, oid, l)
    }

    /// 🔴 **O que a cena MANDA o artista olhar, medido:** a forma LOCAL da asa (os ângulos de
    /// virada) tem de INTERPOLAR de A para B — invariante ao giro global. Sob a co-rotação, o
    /// quadro do meio erra pouco disso; sob o lerp do resíduo (o v2 anterior), a corcova ACHATA
    /// e o erro intrínseco dispara.
    ///
    /// Mutação que sangra (no motor, `point_at` voltando a `advance + residual·u`): o meio
    /// achata e o erro salta acima do teto.
    #[test]
    fn the_torsion_smoke_keeps_the_wing_shape_through_the_turn() {
        let (doc, oid, l) = staged();
        let (a, b) = (positions(&doc, oid, l, 0), positions(&doc, oid, l, 8));
        let (ta, tb) = (turning(&a), turning(&b));
        // O quadro 4 é o meio EXATO do intervalo [0, 8] ⇒ u = 0,5.
        let mid = positions(&doc, oid, l, 4);
        let err: f32 = turning(&mid)
            .iter()
            .zip(ta.iter().zip(&tb))
            .map(|(&g, (&x, &y))| short(x + short(x, y) * 0.5, g).abs())
            .sum();
        assert!(
            err < 0.30,
            "a asa do meio torceu: erro intrínseco {err:.4} (a corcova nao co-rotacionou com o \
             corpo). Com o lerp do resíduo isto passa de 0,5."
        );
    }
}

/// **A SONDA: o que o artista vai ver, em números** (render-and-look, headless).
///
/// `cargo test -p ph2d-host-desktop --release the_torsion_smoke_look -- --ignored --nocapture`
///
/// Roda o MESMO `stage()` + `tween` e imprime, quadro a quadro, a SAGITA da asa (o quanto ela
/// abaula — a corcova) e o erro intrínseco contra a interpolação de A→B. A sagita achatando no
/// meio é a torção; o erro subindo é a mesma coisa em números robustos.
#[cfg(test)]
#[test]
#[ignore = "sonda: imprime o que a cena de torção mostra, quadro a quadro"]
fn the_torsion_smoke_look() {
    use ph2d_flip::{FlipDoc, TweenOptions, TweenRequest};

    let mut doc = FlipDoc::default();
    let oid = doc.push_object("T");
    let obj = doc.object_mut(oid).expect("objeto");
    let l = stage(obj);
    obj.tween(TweenRequest {
        layer: l,
        from: 0,
        to: 8,
        count: 3,
        options: TweenOptions::default(),
    });
    let obj = doc.object(oid).expect("objeto");
    // A sagita: a distância perpendicular máxima dos pontos internos à corda ponta-a-ponta.
    let sagitta = |p: &[Vec2]| -> f32 {
        let (a, b) = (p[0], p[p.len() - 1]);
        let ab = b - a;
        let len = ab.length().max(1e-6);
        p.iter()
            .map(|&q| ((q - a).x * ab.y - (q - a).y * ab.x).abs() / len)
            .fold(0.0f32, f32::max)
    };
    println!("\n  quadro   sagita (corcova)   (achatou = torceu)");
    for f in [0, 2, 4, 6, 8] {
        let Some(d) = obj.layer(l).expect("camada").drawing_at(f) else {
            continue;
        };
        let p = obj.drawing(d).expect("arte").strokes[0].positions();
        println!("  {f:^6}   {:^16.3}", sagitta(&p));
    }
    println!(
        "\n  a sagita (o abaulamento da asa) tem de CRESCER suave de 0 (quadro 0) ao maximo\n\
         (quadro 8). Se ela AFUNDAR no meio (a corcova achata sob o giro de 160), o resíduo\n\
         nao co-rotacionou -- e' a torção que esta wave corrige.\n"
    );
}
