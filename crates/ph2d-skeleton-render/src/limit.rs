//! ⭐⭐⭐ **O ARCO DE LIMITE de uma junta** — irmão do [`super`] pelo teto de 700 LOC, e o corte é o
//! mesmo que o [`super::goal`] já fez: ali mora *onde a corrente quer chegar*, aqui *até onde ela
//! pode ir*.

use super::{BonePart, INFLUENCE_HANDLE_R_PX, LINE_PX};
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::{Affine, BezPath, Brush, Color as VelloColor, Fill, Point, Stroke, VectorScene};

/// ⭐⭐⭐ **O ARCO DE LIMITE de uma junta** — o setor por onde a ponta do osso pode passar, já em
/// MUNDO.
///
/// ⭐⭐ **O raio é o COMPRIMENTO DO OSSO, e isso elimina toda a arbitrariedade:** o arco é
/// literalmente o caminho que a ponta percorre, e arrastar uma alça é *«leve a ponta até aqui e
/// trave»*. ⛔ Um raio escolhido em píxeis seria um número sem dono, e num osso curto cobriria o
/// esqueleto inteiro.
///
/// ⚠️ **O leque vem AMOSTRADO em mundo, e não como (centro, raio, dois ângulos)**: sob um afim
/// não-conforme (o pai escalado em X e não em Y) um arco de circunferência é uma ELIPSE, e
/// reconstruí-lo de um raio só desenharia a coisa errada exactamente onde o artista mais precisa de
/// confiar no que vê. Transformar PONTOS está certo com qualquer afim — é a mesma lei que o
/// `aim_rotation` já segue.
#[derive(Clone, Debug, PartialEq)]
pub struct LimitArc {
    /// O vértice do setor: a origem do osso, em mundo.
    pub apex: [f64; 2],
    /// A borda horária (`min`) do setor, no raio do osso — onde a PONTA dele pára.
    pub edge_min: [f64; 2],
    /// A borda anti-horária (`max`). Ver [`LimitArc::edge_min`].
    pub edge_max: [f64; 2],
    /// ⭐⭐⭐ **A ALÇA da parede horária — FORA do alcance do osso.**
    ///
    /// ⛔⛔ **Ela não pode estar em `edge_min`, e isso é um defeito MEDIDO** (report do dono,
    /// 2026-09-07: *«os gizmos de limite mudam de posição sozinho após mover a cadeia»*): quando o
    /// osso encosta na parede, a ponta dele e a borda do setor ocupam **o mesmo ponto** — medido,
    /// `distância ponta→parede = 0,000000` — e o dedo apanhava `LimitMax` onde o artista queria a
    /// ponta. Ele movia a cadeia até ao limite, agarrava para continuar, e **arrastava a parede**.
    ///
    /// ⚠️ **Priorizar não cura: só troca a vítima** — é a mesma lição que a colisão entre a alça da
    /// força e a parede já tinha dado nesta wave. A cura é geométrica: a alça sai para fora do raio
    /// que o osso alcança, e aí ela não coincide com **nenhum** ponto dele, seja qual for a pose.
    pub handle_min: [f64; 2],
    /// A alça da parede anti-horária. Ver [`LimitArc::handle_min`].
    pub handle_max: [f64; 2],
    /// O arco entre as duas, amostrado em mundo — `min` primeiro, `max` no fim.
    pub fan: Vec<[f64; 2]>,
}

/// Quão translúcido é o setor. ⚠️ Mais fraco que a região de influência (`0,18`), de propósito: o
/// arco vive **por cima** dela num osso em foco, e dois véus somados dariam uma mancha opaca.
const LIMIT_ALPHA: f32 = 0.12;

/// Meia-aresta das duas alças do arco, em píxeis de tela.
///
/// ⚠️ **Do tamanho da alça da força** (`5`) e não da bolinha da junta (`12`): as três são chrome do
/// mesmo osso, e o que as distingue é a FORMA e o SÍTIO, nunca o tamanho — dois alvos do mesmo
/// tamanho em sítios diferentes obrigariam o artista a decorar qual é qual.
pub const LIMIT_HANDLE_R_PX: f64 = INFLUENCE_HANDLE_R_PX;

/// ⭐⭐⭐ **DESENHA O ARCO DE LIMITE** — o setor e as duas alças.
///
/// ⛔ **UM arco de cada vez**, o do osso em foco — a mesma lei do [`draw_influence`], e pela mesma
/// razão: todos ao mesmo tempo seriam sopa.
pub fn draw_limit(
    arc: Option<&LimitArc>,
    hovered: Option<BonePart>,
    transform: Affine,
    theme: Theme,
    target: &mut VectorScene,
) {
    let Some(arc) = arc else {
        return;
    };
    if arc.fan.len() < 2 {
        return;
    }
    let c = ColorToken::Accent.resolve(theme);
    let cor = VelloColor::from_rgba8(c.r, c.g, c.b, c.a);
    // O SETOR: vértice na junta, o leque, e de volta. É o que se vê.
    let mut setor = BezPath::new();
    setor.move_to(Point::new(arc.apex[0], arc.apex[1]));
    for p in &arc.fan {
        setor.line_to(Point::new(p[0], p[1]));
    }
    setor.close_path();
    target.inner_mut().fill(
        Fill::NonZero,
        transform,
        &Brush::Solid(cor.multiply_alpha(LIMIT_ALPHA)),
        None,
        &setor,
    );
    // As duas PAREDES, traçadas do vértice até à ALÇA — o traço atravessa o setor e sai por fora
    // dele, que é o que liga visualmente a alça à parede que ela comanda.
    let mut paredes = BezPath::new();
    for e in [arc.handle_min, arc.handle_max] {
        paredes.move_to(Point::new(arc.apex[0], arc.apex[1]));
        paredes.line_to(Point::new(e[0], e[1]));
    }
    target.inner_mut().stroke(
        &Stroke::new(
            LINE_PX
                * transform.as_coeffs()[0]
                    .hypot(transform.as_coeffs()[1])
                    .recip(),
        ),
        transform,
        &Brush::Solid(cor.multiply_alpha(0.5)),
        None,
        &paredes,
    );
    // AS ALÇAS — triângulos, e a forma é que as distingue: a junta é um círculo e a força é um
    // quadrado. Três alças do mesmo osso, três formas.
    for (p, qual) in [
        (arc.handle_min, BonePart::LimitMin),
        (arc.handle_max, BonePart::LimitMax),
    ] {
        let q = transform * Point::new(p[0], p[1]);
        let r = LIMIT_HANDLE_R_PX;
        let mut tri = BezPath::new();
        tri.move_to(Point::new(q.x, q.y - r));
        tri.line_to(Point::new(q.x + r, q.y + r));
        tri.line_to(Point::new(q.x - r, q.y + r));
        tri.close_path();
        if hovered == Some(qual) {
            target.inner_mut().fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &Brush::Solid(cor),
                None,
                &tri,
            );
        }
        target.inner_mut().stroke(
            &Stroke::new(LINE_PX),
            Affine::IDENTITY,
            &Brush::Solid(cor),
            None,
            &tri,
        );
    }
}
