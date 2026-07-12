//! **Sólidos isométricos** (2.5D) — módulo irmão de [`crate::shapes`].
//!
//! Não são 3D: são polígonos planos desenhados em projeção, que é o que os apps de
//! diagrama chamam de "3D shapes". O que se espera deles é que **as arestas internas
//! existam** — sem elas o cubo é um hexágono e a pirâmide é um triângulo.
//!
//! Autorado no espaço unitário ([`crate::space`]), `v = 0` no TOPO. A versão anterior
//! nascia de cabeça para baixo (ápices apontando para o chão) e o cubo carregava uma
//! **diagonal solta** cortando uma face.
//!
//! ## Os dois parâmetros que governam a projeção
//!
//! Ninguém concorda sobre a projeção: o `cube` do OOXML e o do draw.io são **oblíquos a
//! 45°** (a face frontal é um retângulo comum); o `isoCube` do draw.io é isométrico de
//! 30°. Em vez de escolher uma, o módulo expõe a identidade que governa todas:
//!
//! ```text
//! ângulo do eixo = atan(rise · altura_px / largura_px)
//! ```
//!
//! - **`rise`** — a altura (fração) do vértice interno. É o que inclina os eixos.
//!   `rise = 0,5` numa caixa quadrada dá **26,57°** (o dimétrico 2:1 do pixel art);
//!   `rise = 0,5` numa caixa de proporção `2/√3` dá **30°** (isométrico verdadeiro).
//! - **`skew`** — reparte largura e profundidade. **Não** muda o ângulo do eixo.
//!
//! ## Sombreamento por face: pendência consciente
//!
//! O padrão-ouro (e o que o OOXML faz no `cube`) é emitir **cada face como um caminho
//! preenchível próprio** — topo claro, lateral média, frontal escura —, porque é o
//! sombreamento que faz o sólido parecer sólido; só o contorno nunca lê como volume.
//! Aqui uma forma cozinha para UM `VecPath`, então as faces saem como silhueta preenchida
//! + arestas internas em traço. Faces independentes exigem `cook` emitir várias entidades
//! (uma por face, agrupadas) — mudança de arquitetura, anotada como follow-up.

use crate::VecPath;
use crate::space::{Unit, Uv, add_sub, closed, fit, poly};

/// O vértice interno e os seis da silhueta, dados `rise` e `skew`.
///
/// A silhueta é um hexágono e o vértice interno `M = (skew, rise)` é onde as três faces se
/// encontram. **É o único ponto de onde saem arestas internas** — e são exatamente três,
/// para os vértices alternados do hexágono. Qualquer outra aresta (uma diagonal de face,
/// por exemplo) é ruído: foi o que apareceu na foto do cubo.
fn cube_frame(rise: f64, skew: f64) -> ([Uv; 6], Uv) {
    let (r, t) = (rise, skew);
    let hex = [
        (1.0 - t, 0.0),             // V0 — o topo
        (1.0, r * t),               // V1 — ombro direito
        (1.0, 1.0 - r * (1.0 - t)), // V2 — quadril direito
        (t, 1.0),                   // V3 — a base
        (0.0, 1.0 - r * t),         // V4 — quadril esquerdo
        (0.0, r * (1.0 - t)),       // V5 — ombro esquerdo
    ];
    (hex, (t, r))
}

/// **Cubo isométrico.** Silhueta hexagonal + as TRÊS arestas internas, todas incidentes num
/// único vértice interno.
///
/// **`from_below` é a ambiguidade de Necker, resolvida.** O mesmo hexágono lê como cubo
/// visto de cima OU de baixo; o que decide é para onde as três arestas apontam:
///
/// - **de cima** (default): o vértice interno é `M = (skew, rise)` e as arestas vão para os
///   vértices ímpares (V1, V3, V5) — vê-se o topo e as duas laterais da frente;
/// - **de baixo**: o vértice interno é o OPOSTO, `M' = (1 − skew, 1 − rise)`, e as arestas
///   vão para os PARES (V0, V2, V4) — vê-se a face de baixo.
///
/// Nos defaults (`0,5 / 0,5`) os dois vértices coincidem no centro da caixa — que é
/// precisamente por que um cubo em traço puro é ambíguo a olho nu.
#[must_use]
pub fn iso_cube(a: [f64; 2], b: [f64; 2], rise: f64, skew: f64, from_below: bool) -> VecPath {
    let u = Unit::of(a, b);
    let (r, t) = (rise.clamp(0.1, 0.9), skew.clamp(0.1, 0.9));
    let (hex, m_above) = cube_frame(r, t);
    // O vértice interno e o trio de arestas dependem do lado de onde se olha.
    let (m, spokes) = if from_below {
        ((1.0 - t, 1.0 - r), [0_usize, 2, 4])
    } else {
        (m_above, [1_usize, 3, 5])
    };

    let mut p = poly(&u, &hex);
    // Uma polilinha aberta A→M→B mais um traço M→C: cobre as três sem repetir nenhuma.
    add_sub(
        &mut p,
        vec![
            u.corner(hex[spokes[0]]),
            u.corner(m),
            u.corner(hex[spokes[1]]),
        ],
        false,
    );
    add_sub(&mut p, vec![u.corner(m), u.corner(hex[spokes[2]])], false);
    p
}

/// **Cone.** Ápice para CIMA; base elíptica; e — o ponto que quase todo mundo erra — as
/// geratrizes são **tangentes** à elipse da base, não retas até os extremos dela.
///
/// Com o ápice em `(0,5, 0)`, a base centrada em `(0,5, 1 − lip)` de raios `(0,5, lip)` e
/// altura `h = 1 − lip`, a tangência sai de
///
/// ```text
/// φ = asin(ry / h)          →  θ = −φ  e  θ = 180° + φ
/// T = (0,5 ± rx·√(h² − ry²)/h,  (1 − lip) − ry²/h)
/// ```
///
/// e o arco da FRENTE varre `180° + 2φ` (mais que meia elipse — é justamente o excedente
/// que revela a barriga da base). Ligar o ápice aos extremos laterais `(0/1, 1 − lip)` dá
/// um cone visivelmente "quebrado" no encontro: com `lip = 0,25` o erro é de 8,3% da
/// altura. O stencil de cone do draw.io tem exatamente esse defeito.
///
/// **`from_below` decide se a borda de TRÁS da base existe.** Ela é a meia-elipse que
/// fecharia o disco por cima:
///
/// - **de cima** (default): o corpo do cone TAPA essa borda — ela não é desenhada. É o cone
///   sólido clássico: duas geratrizes e o arco da frente.
/// - **de baixo**: vê-se a face inferior do disco, que fica *na frente* do cone — e a borda
///   de trás aparece cruzando o corpo.
///
/// Não é cosmético: a linha existe ou não existe, conforme o ponto de vista.
#[must_use]
pub fn iso_cone(a: [f64; 2], b: [f64; 2], lip: f64, from_below: bool) -> VecPath {
    let u = Unit::of(a, b);
    let ry = lip.clamp(0.05, 0.45);
    let (rx, h) = (0.5, 1.0 - ry);
    let c: Uv = (0.5, 1.0 - ry);
    let phi = (ry / h).asin().to_degrees();

    let mut verts = vec![u.corner((0.5, 0.0))]; // o ápice
    verts.extend(u.arc(c, rx, ry, -phi, 180.0 + 2.0 * phi)); // a barriga da base
    let mut p = closed(verts);
    if from_below {
        add_sub(
            &mut p,
            u.arc(c, rx, ry, 180.0 + phi, 180.0 - 2.0 * phi),
            false,
        );
    }
    fit(&u, &mut p);
    p
}

/// **Pirâmide.** Ápice para CIMA sobre o centro da base; a silhueta é um QUADRILÁTERO
/// (ápice → direita → frente → esquerda).
///
/// **`from_below` troca quais arestas internas existem.**
///
/// - **de cima** (default): veem-se as duas faces da FRENTE, e a quina entre elas é a aresta
///   ápice → vértice frontal. As três arestas de trás ficam escondidas pelo corpo.
/// - **de baixo**: a base tapa o corpo, e a única aresta escondida passa a ser a da frente.
///   Aparecem **três**: as duas arestas de TRÁS da base (esquerda → fundo → direita), que
///   desenham o losango da base vista por baixo, **e a aresta ápice → fundo**.
///
/// Essa terceira é fácil de esquecer, e o olho a cobra: a região acima do losango da base
/// **não é uma face, são DUAS** — as duas faces laterais traseiras —, e o que as separa é
/// exatamente a aresta ápice → fundo. Sem ela, aquele pedaço lê como um triângulo chapado e
/// a pirâmide perde o volume.
#[must_use]
pub fn iso_pyramid(a: [f64; 2], b: [f64; 2], rise: f64, skew: f64, from_below: bool) -> VecPath {
    let u = Unit::of(a, b);
    let (r, t) = (rise.clamp(0.1, 0.9), skew.clamp(0.1, 0.9));
    let (hex, _) = cube_frame(r, t);
    let apex: Uv = (0.5, 0.0);
    let (right, front, left) = (hex[2], hex[3], hex[4]);
    // O 4º canto da base — o oposto do da frente no losango da base.
    let back: Uv = (1.0 - t, 1.0 - r);

    let mut p = poly(&u, &[apex, right, front, left]);
    if from_below {
        // As duas arestas de trás da base…
        add_sub(
            &mut p,
            vec![u.corner(left), u.corner(back), u.corner(right)],
            false,
        );
        // …e a quina entre as duas faces traseiras.
        add_sub(&mut p, vec![u.corner(back), u.corner(apex)], false);
    } else {
        add_sub(&mut p, vec![u.corner(apex), u.corner(front)], false);
    }
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: [f64; 2] = [-2.0, -2.0];
    const B: [f64; 2] = [2.0, 2.0];

    /// **Os sólidos ficam de PÉ.** O ápice do cone e o da pirâmide têm de estar ACIMA de
    /// todo o resto da forma (mundo Y-para-cima). Nasceram os dois apontando para o chão.
    #[test]
    fn the_apex_points_up_not_down() {
        for (name, p) in [
            ("cone", iso_cone(A, B, 0.15, false)),
            ("pyramid", iso_pyramid(A, B, 0.5, 0.5, false)),
        ] {
            let apex = p
                .verts
                .iter()
                .max_by(|x, y| x.anchor[1].total_cmp(&y.anchor[1]))
                .expect("tem vertices");
            assert!(
                apex.anchor[0].abs() < 1e-6,
                "{name}: o apice fica no eixo, nao no canto: {:?}",
                apex.anchor
            );
            let base = p.verts_all().map(|v| v.anchor[1]).fold(f64::MAX, f64::min);
            assert!(
                apex.anchor[1] > base + 1.0,
                "{name}: o apice ({}) tem de estar bem ACIMA da base ({base})",
                apex.anchor[1]
            );
        }
    }

    /// **A aresta solta do cubo, executável.** O cubo tem 6 vértices de silhueta e as
    /// arestas internas são exatamente 3 — todas incidentes no vértice central `M`. A
    /// versão antiga desenhava uma diagonal que cortava uma face ao meio (foi o que o Enio
    /// fotografou); este teste rejeita qualquer aresta que não toque `M`.
    #[test]
    fn every_internal_edge_of_the_cube_touches_the_centre() {
        let p = iso_cube(A, B, 0.5, 0.5, false);
        assert_eq!(p.verts.len(), 6, "a silhueta e um hexagono");

        let u = Unit::of(A, B);
        let m = u.p((0.5, 0.5));
        let near = |q: [f64; 2]| (q[0] - m[0]).hypot(q[1] - m[1]) < 1e-9;

        let mut edges = 0;
        for c in &p.subpaths {
            for w in c.verts.windows(2) {
                edges += 1;
                assert!(
                    near(w[0].anchor) || near(w[1].anchor),
                    "aresta interna solta {:?}→{:?}: nao toca o centro {m:?}",
                    w[0].anchor,
                    w[1].anchor
                );
            }
        }
        assert_eq!(edges, 3, "sao exatamente TRES arestas internas");
    }

    /// **As geratrizes do cone são TANGENTES à base.** O teste é o discriminante: a reta do
    /// ápice ao ponto de tangência toca a elipse em UM ponto (raiz dupla). Ligar o ápice ao
    /// extremo lateral da elipse — o erro clássico — dá duas raízes e o cone sai quebrado.
    #[test]
    fn the_cone_generatrices_are_tangent_to_its_base() {
        let lip = 0.25_f64;
        let (rx, ry) = (0.5, lip);
        let h = 1.0 - ry;
        // O ponto de tangência, em espaço de autoria (base centrada em (0.5, 1−lip)).
        let tx = 0.5 + rx * (h * h - ry * ry).sqrt() / h;
        let ty = (1.0 - ry) - ry * ry / h;

        // A reta ápice→T, parametrizada; substituída na elipse, tem de dar raiz DUPLA.
        let (ax, ay) = (0.5, 0.0);
        let (dx, dy) = (tx - ax, ty - ay);
        let (ex, ey) = (0.5, 1.0 - ry); // centro da elipse
        let qa = (dx / rx).powi(2) + (dy / ry).powi(2);
        let qb = 2.0 * ((ax - ex) * dx / (rx * rx) + (ay - ey) * dy / (ry * ry));
        let qc = ((ax - ex) / rx).powi(2) + ((ay - ey) / ry).powi(2) - 1.0;
        let disc = qb * qb - 4.0 * qa * qc;
        assert!(
            disc.abs() < 1e-9,
            "discriminante {disc} != 0 — a geratriz corta a base em vez de tangenciar"
        );

        // E o erro que se comete ligando ao extremo lateral é grande o bastante para ver.
        let naive_y = 1.0 - ry; // o "equador" da elipse
        assert!(
            (naive_y - ty).abs() > 0.08,
            "o ponto ingenuo estaria a {} da tangencia — se fosse pequeno, o bug seria invisivel e o teste, inutil",
            (naive_y - ty).abs()
        );
    }

    /// Todos cabem na caixa do gesto — senão a bbox mente e o gizmo desalinha do desenho.
    #[test]
    fn every_solid_fits_inside_the_gesture_box() {
        for (name, p) in [
            ("cube", iso_cube(A, B, 0.5, 0.5, false)),
            ("cone", iso_cone(A, B, 0.25, false)),
            ("pyramid", iso_pyramid(A, B, 0.5, 0.5, false)),
        ] {
            for v in p.verts_all() {
                assert!(
                    v.anchor[0].abs() <= 2.0 + 1e-6 && v.anchor[1].abs() <= 2.0 + 1e-6,
                    "{name}: ancora {:?} fora da caixa",
                    v.anchor
                );
            }
        }
    }

    /// **O ponto de vista MUDA a geometria, não a aparência.** É a diferença entre uma
    /// opção de verdade e um enfeite.
    ///
    /// - **Cone:** visto de CIMA o corpo tapa a borda de trás da base — ela não existe. De
    ///   BAIXO vê-se a face inferior do disco, que fica na frente do cone, e a borda aparece
    ///   cruzando o corpo. (Foi o que o Enio apontou: "se é por cima essa linha some".)
    /// - **Cubo:** é a ambiguidade de Necker. O hexágono é o MESMO; o que muda é o vértice
    ///   interno — `(skew, rise)` ou o oposto — e, com ele, para onde apontam as três
    ///   arestas. O teste usa `rise ≠ skew` de propósito: nos defaults `0,5/0,5` os dois
    ///   vértices coincidem no centro e as duas leituras seriam indistinguíveis (que é
    ///   exatamente por que um cubo em traço puro engana o olho).
    /// - **Pirâmide:** de cima a quina viva é a da FRENTE; de baixo a base tapa o corpo e o
    ///   que aparece são as duas arestas de TRÁS.
    #[test]
    fn the_viewpoint_changes_which_edges_exist() {
        // Cone: de cima a boca da base NAO existe; de baixo, existe.
        assert_eq!(
            iso_cone(A, B, 0.2, false).subpaths.len(),
            0,
            "visto de cima, o corpo do cone TAPA a borda de tras da base"
        );
        assert_eq!(
            iso_cone(A, B, 0.2, true).subpaths.len(),
            1,
            "visto de baixo, a borda de tras aparece cruzando o corpo"
        );

        // Cubo: mesmo hexagono, vertice interno OPOSTO. Com rise != skew eles nao coincidem.
        let (up, down) = (
            iso_cube(A, B, 0.3, 0.7, false),
            iso_cube(A, B, 0.3, 0.7, true),
        );
        let hull: Vec<[f64; 2]> = up.verts.iter().map(|v| v.anchor).collect();
        let hull_d: Vec<[f64; 2]> = down.verts.iter().map(|v| v.anchor).collect();
        assert_eq!(hull, hull_d, "a SILHUETA e a mesma — e a de Necker");
        let m_up = up.subpaths[0].verts[1].anchor;
        let m_down = down.subpaths[0].verts[1].anchor;
        let d = (m_up[0] - m_down[0]).hypot(m_up[1] - m_down[1]);
        assert!(
            d > 0.5,
            "o vertice interno tem de SALTAR para o oposto (saltou {d})"
        );
        // E as arestas partem de vertices DIFERENTES do hexagono.
        assert_ne!(
            up.subpaths[0].verts[0].anchor, down.subpaths[0].verts[0].anchor,
            "as tres arestas apontam para o outro trio de vertices"
        );

        // Piramide: de cima a quina da FRENTE; de baixo, as duas arestas de tras da base
        // MAIS a que divide as duas faces traseiras.
        let py_up = iso_pyramid(A, B, 0.5, 0.5, false);
        let py_down = iso_pyramid(A, B, 0.5, 0.5, true);
        assert_eq!(edges_of(&py_up).len(), 1, "de cima: a quina da frente");
        assert_eq!(
            edges_of(&py_down).len(),
            3,
            "de baixo: as duas arestas de tras da base MAIS a que vai ao apice"
        );
    }

    /// Todas as arestas internas do path (pares consecutivos dos sub-contornos abertos).
    fn edges_of(p: &VecPath) -> Vec<([f64; 2], [f64; 2])> {
        p.subpaths
            .iter()
            .flat_map(|c| {
                c.verts
                    .windows(2)
                    .map(|w| (w[0].anchor, w[1].anchor))
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    /// **Onde tres faces se encontram, tres arestas se encontram.** A invariante que faltava
    /// — e ela e geral, nao um caso particular.
    ///
    /// O vertice interno de um solido em projecao e o ponto onde tres faces se tocam; dele
    /// tem de sair exatamente **tres** arestas, uma por par de faces. A piramide vista de
    /// baixo nascia com o vertice de tras em grau **2** (so as duas arestas da base): a
    /// regiao acima dele lia como um triangulo chapado, quando na verdade sao DUAS faces
    /// traseiras, e a quina entre elas — apice ate o fundo — nao estava sendo desenhada. O
    /// Enio viu na tela ("faltou o traco central que vai ate o topo") antes de qualquer teste
    /// ver.
    ///
    /// Um teste que apenas CONTASSE arestas nao pegaria isso (duas arestas ja parecem
    /// plausiveis). Contar o GRAU do vertice interno, sim.
    #[test]
    fn three_faces_meet_at_the_internal_vertex_so_three_edges_do_too() {
        let cases: [(&str, VecPath, [f64; 2]); 3] = [
            (
                "cubo de cima",
                iso_cube(A, B, 0.3, 0.7, false),
                Unit::of(A, B).p((0.7, 0.3)),
            ),
            (
                "cubo de baixo",
                iso_cube(A, B, 0.3, 0.7, true),
                Unit::of(A, B).p((0.3, 0.7)),
            ),
            (
                "piramide de baixo",
                iso_pyramid(A, B, 0.3, 0.7, true),
                Unit::of(A, B).p((0.3, 0.7)),
            ),
        ];
        for (name, p, m) in cases {
            let near = |q: [f64; 2]| (q[0] - m[0]).hypot(q[1] - m[1]) < 1e-9;
            let degree = edges_of(&p)
                .iter()
                .filter(|(a, b)| near(*a) || near(*b))
                .count();
            assert_eq!(
                degree, 3,
                "{name}: do vertice interno {m:?} saem {degree} arestas, e tem de ser TRES \
                 (tres faces se tocam ali, e cada par delas tem uma quina)"
            );
        }
    }
}
