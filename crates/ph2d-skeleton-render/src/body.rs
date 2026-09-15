//! ⭐⭐⭐ **A SILHUETA DO CORPO DE UM OSSO** — a seta que o artista vê, agora sobre uma POLILINHA.
//!
//! ⚠️⚠️ **Um osso recto sai daqui BYTE A BYTE como saía da versão que só sabia dois pontos**, e não
//! por promessa: com dois nós há **um** ombro interior, ele cai a `0,25` do único troço, a normal é
//! a perpendicular daquele troço, e o caminho emitido é `raiz → ombro+w → ponta → ombro−w`, que é
//! literalmente o que o losango sempre foi. As expressões são as mesmas — `dx = pb.x − pa.x`,
//! `pa.x + dx·0,25`, `(−dy/comp, dx/comp)` — e é isso que torna a igualdade exacta em vez de
//! parecida.
//!
//! ⛔ **Este ficheiro é um corte por RESPONSABILIDADE, não por tamanho:** *que forma tem o corpo de
//! um osso* e *como o overlay do esqueleto se compõe* são duas perguntas, e a segunda vive no
//! `lib.rs` com o hover, a selecção, as pontas de corrente e os temas.

use ph2d_vector::{BezPath, Point};

/// Onde o **ombro** da seta fica, em fracção do comprimento ANDADO.
///
/// ⚠️ É a proporção que as três referências usam (Blender, Spine, Moho), e o que faz a silhueta ler
/// como uma **seta** em vez de um triângulo: o bico curto de um lado, a cauda longa do outro.
const OMBRO: f64 = 0.25;

/// A meia-largura do corpo na fracção `f` do arco — a tenda que sobe até ao [`OMBRO`] e desce até à
/// ponta.
///
/// ⚠️ **No ombro devolve `w` AO BIT** (`f / OMBRO` é `1.0` exacto ali, e `w * 1.0 == w`), que é uma
/// das duas metades da igualdade com o losango antigo.
fn meia_largura(f: f64, w: f64) -> f64 {
    if f <= OMBRO {
        w * (f / OMBRO)
    } else {
        w * ((1.0 - f) / (1.0 - OMBRO))
    }
}

/// Um ponto do contorno: onde ele está, que fracção do arco ele é, e para onde a normal aponta.
struct No {
    p: Point,
    f: f64,
    n: (f64, f64),
}

/// **O COMPRIMENTO ANDADO** da polilinha, em píxeis de tela.
///
/// ⚠️ **É a soma das cordas e não a distância raiz→ponta**, e a diferença é o produto: num osso
/// muito arqueado a corda encolhe, e dimensionar a largura ou a bolinha por ela faria o osso ficar
/// mais fino e mais difícil de agarrar exactamente quando ele dobra mais. Num osso recto os dois
/// números são o mesmo `hypot`, ao bit.
#[must_use]
pub fn arc_px(pts: &[Point]) -> f64 {
    pts.windows(2)
        .map(|w| (w[1].x - w[0].x).hypot(w[1].y - w[0].y))
        .sum()
}

/// Os nós INTERIORES do contorno, em ordem de arco: os nós próprios da polilinha mais o **ombro**
/// inserido onde ele cai.
///
/// ⚠️ **A raiz e a ponta ficam de fora de propósito** — a largura ali é zero, e elas são as duas
/// pontas do bico; quem as emite é o [`outline`], uma vez cada.
fn interiores(pts: &[Point], comp: f64) -> Vec<No> {
    let alvo = OMBRO * comp;
    let mut out: Vec<No> = Vec::with_capacity(pts.len() + 1);
    let mut andado = 0.0_f64;
    for (i, par) in pts.windows(2).enumerate() {
        let (dx, dy) = (par[1].x - par[0].x, par[1].y - par[0].y);
        let corda = dx.hypot(dy);
        if corda <= f64::EPSILON {
            continue;
        }
        let normal = (-dy / corda, dx / corda);
        // O OMBRO, se ele cai dentro deste troço. ⚠️ `t` é calculado a partir do arco JÁ andado, e
        // num osso de um troço só isso dá `(OMBRO·comp − 0) / comp`, que é `0,25` exacto.
        if andado <= alvo && alvo <= andado + corda {
            let t = (alvo - andado) / corda;
            out.push(No {
                p: Point::new(par[0].x + dx * t, par[0].y + dy * t),
                f: OMBRO,
                n: normal,
            });
        }
        andado += corda;
        // O nó SEGUINTE da polilinha, se não for a ponta. A normal dele é a média das duas cordas
        // que ali se encontram — ⛔ usar só uma das duas abriria um degrau no contorno em cada
        // junta, que é exactamente o que este corpo existe para não ter.
        if i + 2 < pts.len() {
            let (ex, ey) = (pts[i + 2].x - par[1].x, pts[i + 2].y - par[1].y);
            let seguinte = ex.hypot(ey);
            let media = if seguinte > f64::EPSILON {
                let (ux, uy) = (dx / corda + ex / seguinte, dy / corda + ey / seguinte);
                let m = ux.hypot(uy);
                if m > f64::EPSILON {
                    (-uy / m, ux / m)
                } else {
                    normal
                }
            } else {
                normal
            };
            out.push(No {
                p: par[1],
                f: andado / comp,
                n: media,
            });
        }
    }
    out.sort_by(|a, b| a.f.total_cmp(&b.f));
    out
}

/// ⭐⭐⭐ **O CONTORNO FECHADO do corpo de um osso.** `pts` é a polilinha já em píxeis de TELA, `w` a
/// meia-largura no ombro.
///
/// A ida percorre o arco a `+w(f)` e a volta a `−w(f)` ⇒ a figura é simétrica em torno da linha e
/// fecha nas duas pontas.
#[must_use]
pub fn outline(pts: &[Point], comp: f64, w: f64) -> BezPath {
    let nos = interiores(pts, comp);
    let mut p = BezPath::new();
    p.move_to(pts[0]);
    for no in &nos {
        let m = meia_largura(no.f, w);
        p.line_to(Point::new(no.p.x + no.n.0 * m, no.p.y + no.n.1 * m));
    }
    p.line_to(*pts.last().expect("a polilinha tem ao menos dois nos"));
    for no in nos.iter().rev() {
        let m = meia_largura(no.f, w);
        p.line_to(Point::new(no.p.x - no.n.0 * m, no.p.y - no.n.1 * m));
    }
    p.close_path();
    p
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bone_half_width_px;

    /// O losango de sempre, escrito **à mão** exactamente como o código o escrevia antes de existir
    /// polilinha — é ele a referência, e não um golden gravado.
    fn losango_de_sempre(pa: Point, pb: Point) -> BezPath {
        let (dx, dy) = (pb.x - pa.x, pb.y - pa.y);
        let comp = dx.hypot(dy);
        let (nx, ny) = (-dy / comp, dx / comp);
        let ombro = Point::new(pa.x + dx * 0.25, pa.y + dy * 0.25);
        let w = bone_half_width_px(comp).min(comp * 0.25);
        let mut p = BezPath::new();
        p.move_to(pa);
        p.line_to(Point::new(ombro.x + nx * w, ombro.y + ny * w));
        p.line_to(pb);
        p.line_to(Point::new(ombro.x - nx * w, ombro.y - ny * w));
        p.close_path();
        p
    }

    /// ⭐⭐⭐ **UM OSSO RECTO DESENHA-SE PONTO A PONTO COMO SEMPRE SE DESENHOU** — a lei da casa
    /// (todo motor novo é no-op no ponto neutro), aqui sobre a GEOMETRIA emitida.
    ///
    /// ⚠️ **A referência é o código ANTIGO reescrito à mão, não um golden gravado:** um golden
    /// mediria a minha aritmética de hoje; o que se afirma é que as duas construções **coincidem**.
    /// ⛔ Sem isto, todo rig do app mudava de silhueta no último bit e ninguém via — até um teste de
    /// imagem de outra linha reprovar sem uma linha de produto ter mudado.
    #[test]
    fn a_straight_bone_is_drawn_point_for_point_as_it_always_was() {
        for (ax, ay, bx, by) in [
            (0.0, 0.0, 100.0, 0.0),
            (10.0, 20.0, 10.0, 200.0),
            (-30.0, 5.0, 77.0, -91.0),
            (0.0, 0.0, 4.0, 3.0),
        ] {
            let (pa, pb) = (Point::new(ax, ay), Point::new(bx, by));
            let comp = arc_px(&[pa, pb]);
            let w = bone_half_width_px(comp).min(comp * 0.25);
            let nova = outline(&[pa, pb], comp, w);
            let antiga = losango_de_sempre(pa, pb);
            assert_eq!(
                nova.elements(),
                antiga.elements(),
                "osso de ({ax},{ay}) a ({bx},{by})"
            );
        }
    }

    /// ⭐⭐ **O COMPRIMENTO DE UM OSSO RECTO É O `hypot` DE SEMPRE, AO BIT** — a outra metade do
    /// neutro: é dele que saem a largura do corpo e o raio da bolinha.
    #[test]
    fn a_straight_bones_walk_is_exactly_its_chord() {
        for (bx, by) in [(100.0, 0.0), (3.0, 4.0), (-17.0, 91.5)] {
            let (pa, pb) = (Point::new(0.0, 0.0), Point::new(bx, by));
            assert_eq!(arc_px(&[pa, pb]), bx.hypot(by));
        }
    }

    /// ⭐⭐⭐ **O CONTORNO DE UM OSSO ARQUEADO PASSA PELO ARCO, E NÃO PELA CORDA** — o controlo
    /// positivo: sem ele a generalização podia estar escrita e nunca ser exercida.
    ///
    /// A régua é o número de pontos emitidos (um nó da polilinha por lado, mais o ombro e as duas
    /// extremidades) e o facto de o contorno **sair** da faixa da corda.
    #[test]
    fn a_bowed_bone_is_drawn_along_its_arc() {
        // Um arco de cinco nós, a subir e a descer.
        let pts: Vec<Point> = (0..=4)
            .map(|k| {
                let t = f64::from(k) / 4.0;
                Point::new(t * 100.0, (t * std::f64::consts::PI).sin() * 40.0)
            })
            .collect();
        let comp = arc_px(&pts);
        let w = bone_half_width_px(comp).min(comp * 0.25);
        let p = outline(&pts, comp, w);
        // 5 nós ⇒ 3 interiores + o ombro = 4 amostras; ida e volta ⇒ 8, mais a raiz e a ponta.
        assert_eq!(p.elements().len(), 11, "{p:?}");
        let mais_alto = p
            .elements()
            .iter()
            .filter_map(|e| match e {
                ph2d_vector::PathEl::MoveTo(q) | ph2d_vector::PathEl::LineTo(q) => Some(q.y),
                _ => None,
            })
            .fold(f64::NEG_INFINITY, f64::max);
        assert!(
            mais_alto > 35.0,
            "o contorno nao subiu com o arco: {mais_alto}"
        );
    }
}

#[cfg(test)]
mod tests_juntas {
    use super::*;

    /// ⭐⭐⭐ **A NORMAL DE UMA JUNTA REPARTE O ÂNGULO ENTRE AS DUAS CORDAS** — sem isso o contorno
    /// ganha um degrau em cada junta, e o corpo do osso deixa de se ler como uma peça só.
    ///
    /// ⚠️⚠️ **Este gate nasceu de uma MUTAÇÃO SOBREVIVENTE:** trocar a média das duas cordas pela
    /// corda de ENTRADA sozinha deixava a suíte inteira verde — o contorno continuava a ter o número
    /// certo de pontos e a subir com o arco, que era tudo o que as réguas mediam. *Uma régua que
    /// conta pontos não vê para onde eles apontam.*
    ///
    /// A propriedade medida é exacta: a normal faz o MESMO ângulo com a corda que entra e com a que
    /// sai (`|n · d_entra| == |n · d_sai|`). Com uma corda só, o primeiro produto é `0` e o segundo
    /// é o seno da dobra.
    #[test]
    fn a_joints_normal_splits_the_angle_between_its_two_chords() {
        let pts: Vec<Point> = (0..=6)
            .map(|k| {
                let t = f64::from(k) / 6.0;
                Point::new(t * 120.0, (t * std::f64::consts::PI).sin() * 50.0)
            })
            .collect();
        let comp = arc_px(&pts);
        let nos = interiores(&pts, comp);
        let mut vistos = 0;
        for no in &nos {
            // O nó da polilinha (o ombro cai DENTRO de um troço e não tem duas cordas).
            let Some(i) = pts.iter().position(|q| *q == no.p) else {
                continue;
            };
            if i == 0 || i + 1 >= pts.len() {
                continue;
            }
            let unit = |a: Point, b: Point| {
                let (dx, dy) = (b.x - a.x, b.y - a.y);
                let m = dx.hypot(dy);
                (dx / m, dy / m)
            };
            let (ix, iy) = unit(pts[i - 1], pts[i]);
            let (ox, oy) = unit(pts[i], pts[i + 1]);
            let entra = (no.n.0 * ix + no.n.1 * iy).abs();
            let sai = (no.n.0 * ox + no.n.1 * oy).abs();
            assert!(
                (entra - sai).abs() < 1e-12,
                "a junta {i} olha para um lado: entra {entra}, sai {sai}"
            );
            // E o controlo: a dobra existe mesmo, senão o gate afirmaria `0 == 0`.
            assert!(
                (ix * ox + iy * oy) < 0.999,
                "a fixtura nao dobra na junta {i}"
            );
            vistos += 1;
        }
        assert!(
            vistos >= 4,
            "so' {vistos} juntas medidas - a fixtura encolheu"
        );
    }
}
