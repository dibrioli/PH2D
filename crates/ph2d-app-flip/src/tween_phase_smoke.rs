//! **A cena pronta para o smoke da FASE da costura** (`PH2D_FLIP_TWEEN_PHASE_SMOKE=1`).
//!
//! O tween v2 pareia dois anéis por índice, e num traço FECHADO o índice 0 é a costura
//! arbitrária onde o artista fechou o traço — não uma ponta. Se as duas chaves começam o
//! anel em lugares diferentes do contorno, o pareamento fica girado e a forma do meio
//! **torce**. Medido: ela NÃO colapsa — a espiral lê o par nariz<->costas como um giro de ~180°
//! e leva o anel num LAÇO (mergulha para fora da reta e volta rodado). Esta cena arma esse
//! caso: as duas chaves são o MESMO blob, desenhado a partir de pontos de partida diferentes.
//!
//! Com o alinhamento de fase ([`ph2d_flip`] `tween_phase`), o inbetween desliza em linha reta;
//! sem ele, faz o laço. O gate mede a aparência pelo CAMINHO (o centróide na reta, `y≈0`) — a
//! área não denuncia a torção, porque a forma só gira e desloca, não encolhe.

use ph2d_core::Vec2;
use ph2d_flip::{FlipStroke, Hold, KeyKind, Point, Rgba};
use std::f32::consts::TAU;
use std::sync::OnceLock;
use std::sync::atomic::AtomicU32;

pub static FRAME: AtomicU32 = AtomicU32::new(0);

pub fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("PH2D_FLIP_TWEEN_PHASE_SMOKE").is_some())
}

pub const INK: Rgba = Rgba::new(0.92, 0.92, 0.95, 1.0);
/// Quantos pontos no blob (denso o bastante para a fase ter sentido — bem acima do piso).
pub const N: usize = 40;

/// **Um blob-vírgula**: um círculo de raio `r` com um "nariz" apontando para `+X`, começando
/// pelo vértice `start` (a COSTURA), transladado por `off`. A e B usam o MESMO blob (mesma
/// forma), só a costura e a posição mudam — é isso que torna a fase o único jeito de o meio
/// sair limpo. É a MESMA arte que o gate do motor usa, na `ph2d-flip`.
pub fn blob(r: f32, start: usize, off: Vec2) -> FlipStroke {
    let mut s = FlipStroke::new();
    for i in 0..N {
        let a = ((i + start) % N) as f32 / N as f32 * TAU;
        let bump = 1.0 + 0.5 * a.cos().max(0.0).powi(6); // nariz assimétrico
        s.push_point(Point {
            pos: off + Vec2::new(a.cos() * r * bump, a.sin() * r * bump),
            width: 0.06,
            opacity: 1.0,
            color: INK,
        });
    }
    s.closed = true;
    s.hardness = 0.4;
    s
}

/// **Monta as duas chaves** — o mesmo blob, costura movida meia-volta, deslizando da esquerda
/// para a direita. Porta única: o gate encena por AQUI (senão a mensagem impressa descreveria
/// um desenho que ninguém mais produz).
pub fn stage(obj: &mut ph2d_flip::FlipObject) -> ph2d_flip::LayerId {
    let l = obj.add_layer("L");
    // CHAVE 0: blob à ESQUERDA, costura no nariz (índice 0).
    if let Some(d0) = obj.insert_frame(l, 0, Hold::Implicit, KeyKind::Keyframe) {
        obj.drawing_mut(d0)
            .expect("desenho")
            .strokes
            .push(blob(1.2, 0, Vec2::new(-2.2, 0.0)));
    }
    // CHAVE 8: o MESMO blob à DIREITA, costura movida meia-volta (índice N/2 = nas costas).
    if let Some(d8) = obj.insert_frame(l, 8, Hold::Implicit, KeyKind::Keyframe) {
        obj.drawing_mut(d8)
            .expect("desenho")
            .strokes
            .push(blob(1.2, N / 2, Vec2::new(2.2, 0.0)));
    }
    l
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_flip::{FlipDoc, FlipObjectId, LayerId, TweenOptions, TweenRequest};

    /// O centróide (x, y) do traço 0 no quadro `f`.
    fn ring_centroid(doc: &FlipDoc, oid: FlipObjectId, l: LayerId, f: i32) -> Vec2 {
        let obj = doc.object(oid).expect("objeto");
        let d = obj
            .layer(l)
            .expect("camada")
            .drawing_at(f)
            .expect("desenho");
        let p = obj.drawing(d).expect("arte").strokes[0].positions();
        p.iter().fold(Vec2::ZERO, |a, &q| a + q) / p.len() as f32
    }

    fn staged() -> (FlipDoc, FlipObjectId, LayerId) {
        let mut doc = FlipDoc::default();
        let oid = doc.push_object("P");
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

    /// 🔴 **O que a cena MANDA o artista olhar, medido:** A e B são o MESMO blob, ambos em
    /// `y=0`, então um tween correto o desliza em LINHA RETA (o centróide fica na reta). Sem a
    /// fase, o pareamento nariz<->costas vira uma rotação de ~180° e a espiral leva o blob num
    /// DESVIO curvo — ele mergulha para fora da reta e chega rodado. (A ÁREA não denuncia isso:
    /// a forma continua coerente, só rotacionada e deslocada — foi por isso que o 1º oráculo,
    /// de área, nasceu verde sobre o bug; o defeito é o centróide sair da reta.)
    ///
    /// Mutação que sangra (no motor, `tween_phase::seam_shift` devolvendo 0 sempre): o meio
    /// mergulha para `y ≈ −2,2` e o gate falha. É a wave inteira.
    #[test]
    fn the_phase_smoke_keeps_the_ring_on_the_straight_path() {
        let (doc, oid, l) = staged();
        let mut prev_x = ring_centroid(&doc, oid, l, 0).x;
        for f in [2, 4, 6] {
            let c = ring_centroid(&doc, oid, l, f);
            // Na reta A->B (ambos em y=0): sem a fase o blob mergulha (y ≈ −2,2).
            assert!(
                c.y.abs() < 0.4,
                "o blob saiu da reta (y={:.2}) no quadro {f} — a fase não alinhou a costura \
                 (o par nariz<->costas virou um giro de 180°)",
                c.y
            );
            // E desliza da esquerda para a direita, monotônico (nada de vaivém).
            assert!(
                c.x > prev_x - 1e-3,
                "o blob voltou para trás no quadro {f}: x={:.2} (antes {prev_x:.2})",
                c.x
            );
            prev_x = c.x;
        }
        // E chega à direita: o extremo B está em x ~ +2.2.
        assert!(
            (ring_centroid(&doc, oid, l, 8).x - 2.2).abs() < 0.3,
            "o blob não chegou à chave B (direita)"
        );
    }
}

/// **A SONDA: o que o artista vai ver, em números** (render-and-look, headless).
///
/// `cargo test -p ph2d-host-desktop --release the_phase_smoke_look -- --ignored --nocapture`
///
/// Roda o MESMO `stage()` + `tween` e imprime, quadro a quadro, a ÁREA do anel (o tamanho) e
/// o centróide (a posição) — as duas coisas que a mensagem manda conferir.
#[cfg(test)]
#[test]
#[ignore = "sonda: imprime o que a cena de fase mostra, quadro a quadro"]
pub fn the_phase_smoke_look() {
    use ph2d_flip::{FlipDoc, TweenOptions, TweenRequest};

    let mut doc = FlipDoc::default();
    let oid = doc.push_object("P");
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
    println!("\n  quadro   area do anel   centroide (x, y)   (colapso = torceu)");
    for f in [0, 2, 4, 6, 8] {
        let Some(d) = obj.layer(l).expect("camada").drawing_at(f) else {
            continue;
        };
        let p = obj.drawing(d).expect("arte").strokes[0].positions();
        let n = p.len();
        let area = 0.5
            * (0..n)
                .map(|i| p[i].x * p[(i + 1) % n].y - p[(i + 1) % n].x * p[i].y)
                .sum::<f32>();
        let cx = p.iter().map(|q| q.x).sum::<f32>() / n as f32;
        let cy = p.iter().map(|q| q.y).sum::<f32>() / n as f32;
        println!("  {f:^6}   {:^12.3}   ({cx:5.2}, {cy:5.2})", area.abs());
    }
    println!(
        "\n  o centroide tem de andar em LINHA RETA (y ~ 0) da esquerda para a direita. Se o\n\
         'y' mergulhar no meio (ex.: -2.2 no quadro 4), o blob fez um laço/cambalhota -- a\n\
         fase nao alinhou a costura. (A area NAO denuncia: a forma so gira, nao colapsa.)\n"
    );
}
