//! Os gates da cena das QUINAS (`PH2D_BUILD_SMOKE=78`, plano 36 W5).
//!
//! ⚠️ **Uma cena de smoke é uma FIXTURA, e uma fixtura tem de conter o fenómeno.** Estes gates não
//! medem o motor (isso é a `ph2d-vec-scene`): medem que **esta cena** põe quinas a sério debaixo do
//! pincel — e que o artista vê cópias que chegam a elas.

use super::{RAIO, STEP, arte, pincel};
use ph2d_vec_scene::VecPathId;

/// O id que a arte tem nesta fixtura (na cena real ela é a 1.ª forma).
const ART: VecPathId = 7;

/// ⭐⭐⭐ **TODA FORMA DESTA CENA TEM QUINAS A SÉRIO** — e uma tem lados de comprimentos
/// DIFERENTES.
///
/// ⚠️ **A quarta forma é a que separa as duas leis.** Num quadrado todos os lados medem o mesmo,
/// então um encaixe global e um por-lado dão quase a mesma coisa e a cena não distinguiria nada.
/// *Uma fixtura simétrica não distingue os dois lados de uma lei que tem dois lados* — foi a mesma
/// lição na `ph2d-arclen`, no mesmo dia.
#[test]
fn every_shape_in_the_corner_smoke_actually_has_corners() {
    let limiar = 1.0_f64.to_radians();
    let x = |i: usize| -1.5 * STEP + (i as f64) * STEP;
    let esperado = [
        ("quadrado", 4),
        ("estrela", 10),
        ("triangulo", 3),
        ("retangulo achatado", 4),
    ];
    let formas = [
        ph2d_vec_scene::rectangle([x(0) - RAIO, -RAIO], [x(0) + RAIO, RAIO]),
        ph2d_vec_scene::star([x(1), 0.0], RAIO, RAIO, 5, 0.45),
        ph2d_vec_scene::regular_polygon([x(2), 0.0], RAIO, RAIO, 3),
        ph2d_vec_scene::rectangle(
            [x(3) - RAIO * 1.6, -RAIO * 0.5],
            [x(3) + RAIO * 1.6, RAIO * 0.5],
        ),
    ];
    for ((nome, n), forma) in esperado.into_iter().zip(&formas) {
        let g = ph2d_vec_scene::arc_path::ArcPath::from_contour(&forma.verts, true)
            .unwrap_or_else(|| panic!("{nome}: sem guia"));
        assert_eq!(g.corner_arcs(limiar).len(), n, "{nome}: quinas a menos");
    }
    // ⚠️ **A metade que dá SUJEITO à cena**: a última forma tem lados DESIGUAIS, e é por isso que
    // ela está lá. Sem esta linha alguém podia «arrumá-la» para um quadrado e a cena continuaria
    // verde sem provar o ritmo por-lado.
    let achatado = &formas[3];
    let lados: Vec<f64> = {
        let v = &achatado.verts;
        (0..v.len())
            .map(|i| {
                let a = v[i].anchor;
                let b = v[(i + 1) % v.len()].anchor;
                (b[0] - a[0]).hypot(b[1] - a[1])
            })
            .collect()
    };
    let (lo, hi) = (
        lados.iter().copied().fold(f64::MAX, f64::min),
        lados.iter().copied().fold(0.0, f64::max),
    );
    assert!(
        hi / lo > 2.0,
        "a 4.a forma tem lados quase iguais ({lo:.3} contra {hi:.3}) - ela deixou de distinguir o \
         encaixe global do por-lado"
    );
}

/// ⭐⭐ **NENHUMA CÓPIA FICA SENTADA EM CIMA DE UMA QUINA** — a régua da cena, sobre a SAÍDA.
///
/// Com o corte nas quinas, as cópias **abutam** o canto: o centro da mais próxima fica a ~meio
/// avanço dele. Uma cópia que **atravessasse** a quina teria o centro **em cima** dela — e é isso
/// que o artista vê como *«a arte cortou a esquina»*.
///
/// ⚠️⚠️ **DUAS réguas minhas falharam antes desta, e as duas por emprestar a pergunta errada.**
/// A 1.ª mediu a distância à **espinha** (dois vértices extremos), que nesta arte corre pela borda
/// de baixo e não pelo meio — acusou produto correcto a `0,96`. A 2.ª perguntou *«a guia está
/// coberta por arte?»* e acusou a `0,73`, porque **esta arte é uma folha**: à altura da linha
/// central ela mede `0,60` de `1,00` de caixa, então entre duas cópias há vão **por desenho**, em
/// toda parte, também numa curva suave. *A pergunta certa não é «a linha está coberta», é «a arte
/// segue a linha»* — e a lei geométrica disso já está gateada no motor
/// (`the_guide_is_covered_by_the_copies_the_product_emitted`), com uma arte que a pode medir.
#[test]
fn no_copy_in_the_corner_smoke_sits_on_a_corner() {
    let x = |i: usize| -1.5 * STEP + (i as f64) * STEP;
    let art = arte(0.0, 0.0);
    let s = pincel(ART);
    let formas = [
        ph2d_vec_scene::rectangle([x(0) - RAIO, -RAIO], [x(0) + RAIO, RAIO]),
        ph2d_vec_scene::star([x(1), 0.0], RAIO, RAIO, 5, 0.45),
        ph2d_vec_scene::regular_polygon([x(2), 0.0], RAIO, RAIO, 3),
        ph2d_vec_scene::rectangle(
            [x(3) - RAIO * 1.6, -RAIO * 0.5],
            [x(3) + RAIO * 1.6, RAIO * 0.5],
        ),
    ];
    for (i, forma) in formas.iter().enumerate() {
        let g = ph2d_vec_scene::arc_path::ArcPath::from_contour(&forma.verts, true)
            .expect("guia com comprimento");
        let copias = ph2d_vec_scene::brush_along_path(forma, &art, &s);
        assert!(copias.len() > 6, "forma {i}: so' {} copias", copias.len());
        let centros: Vec<[f64; 2]> = copias.iter().map(centro_de).collect();
        // O avanço EFECTIVO desta forma: a mediana do passo entre centros consecutivos.
        let mut passos: Vec<f64> = centros
            .windows(2)
            .map(|w| (w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1]))
            .collect();
        passos.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let avanco = passos[passos.len() / 2];
        assert!(avanco > 0.0, "forma {i}: avanco nulo");
        for arco in g.corner_arcs(1.0_f64.to_radians()) {
            let (q, _) = g.frame_at(arco);
            let d = centros
                .iter()
                .map(|c| (c[0] - q[0]).hypot(c[1] - q[1]))
                .fold(f64::MAX, f64::min)
                / avanco;
            assert!(
                d > 0.25,
                "forma {i}: ha' uma copia centrada a {d:.3} avancos da quina em {q:?} - ela esta' \
                 sentada em cima do canto, a cortá-lo"
            );
        }
    }
}

/// O centro da caixa de uma cópia.
fn centro_de(p: &ph2d_vec_scene::VecPath) -> [f64; 2] {
    let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
    for v in &p.verts {
        for k in 0..2 {
            lo[k] = lo[k].min(v.anchor[k]);
            hi[k] = hi[k].max(v.anchor[k]);
        }
    }
    [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5]
}
