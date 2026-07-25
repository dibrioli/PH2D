//! O gate do **Envelope sobre uma ESTRELA côncava** — o caso que o Enio reportou (2026-07-24), com
//! a cena de smoke `=27` como espelho visual. Módulo FILHO de [`super`] pelo teto de LOC (HR-18);
//! herda os fixtures (`star`/`envelope_over`/`frame`/…) por `use super::*`.

use super::*;

/// Uma ESTRELA de 5 pontas (fonte CÔNCAVA), razão interna 0.45 — a forma do caso do Enio.
fn star(c: [f64; 2], r: f64) -> VecPath {
    cook(
        ShapeKind::Star,
        [c[0] - r, c[1] - r],
        [c[0] + r, c[1] + r],
        &[5.0, 0.45, 0.0],
    )
}

/// **UMA GAIOLA PUXADA SOBRE UMA ESTRELA (fonte CÔNCAVA) deforma LIMPO.** A gaiola envolve a
/// **BBOX** (um retângulo, igual à elipse), então a concavidade da FONTE não muda a gaiola nem as
/// alças de canto: cada ponto da estrela mapeia pelo MESMO `QuadWarp`. O oráculo é a APARÊNCIA — a
/// saída é FINITA e cabe dentro da gaiola —, então nenhum ponto foge (a "alça solta + linha reta à
/// deriva" da foto veio de um build velho da pasta primária, não desta linha). Par do gate da
/// elipse (`a_pulled_cage_deforms_the_child_through_the_engine`).
#[test]
fn a_pulled_cage_deforms_a_concave_star_cleanly() {
    let src = star([5.0, 3.0], 2.0);
    let (mut sim, mut scene, _map, container, ids) = envelope_over(vec![src]);
    let (origin, size) =
        super::super::union_control_bbox([&source_of(&env_of(&sim, container).children[0])])
            .unwrap();
    // O MESMO trapézio do smoke =27: topo estreitado a 40% da base (perspectiva convexa).
    let [bl, br, ..] = bbox_corners(origin, size);
    let pulled = [
        bl,
        br,
        [origin[0] + size[0] * 0.7, origin[1] + size[1]],
        [origin[0] + size[0] * 0.3, origin[1] + size[1]],
    ];
    assert!(QuadWarp::is_convex(&pulled), "o fixture devia ser convexo");
    set_corners(&mut sim, container, pulled);

    let out = frame(&mut sim, &mut scene, ids[0]);
    // Todo ponto (âncora E alças) FINITO e dentro da gaiola dilatada por UMA largura — folga ampla
    // para as alças de Bézier, mas MUITO menor que uma fuga (a foto tinha um ponto a grid-distância).
    let (lo_x, hi_x) = (origin[0] - size[0], origin[0] + size[0] * 2.0);
    let (lo_y, hi_y) = (origin[1] - size[1], origin[1] + size[1] * 2.0);
    for v in &out.verts {
        for p in [v.anchor, v.in_handle, v.out_handle] {
            assert!(
                p[0].is_finite() && p[1].is_finite(),
                "ponto não-finito: {p:?}"
            );
            assert!(
                p[0] >= lo_x && p[0] <= hi_x && p[1] >= lo_y && p[1] <= hi_y,
                "ponto {p:?} fugiu da gaiola [{lo_x:.2},{lo_y:.2}]..[{hi_x:.2},{hi_y:.2}] — alça à deriva"
            );
        }
    }
    // E a estrela DE FATO deformou (não ficou em repouso).
    assert_ne!(
        out.verts,
        star([5.0, 3.0], 2.0).verts,
        "a gaiola foi puxada e a estrela não mudou"
    );
}

/// **`is_envelope` distingue o container do filho** — a porta que faz o overlay de nós SUMIR sob a
/// gaiola (`render_loop`: `if !envelope_selected`). O container carrega o `VecEnvelope`; o filho é
/// forma comum. Sem esta distinção o overlay desenharia as alças da forma DERIVADA — a "alça à
/// deriva" que o Enio viu numa estrela sob envelope.
#[test]
fn is_envelope_is_true_for_the_container_false_for_the_child() {
    let (sim, _scene, map, container, ids) = envelope_over(vec![star([5.0, 3.0], 2.0)]);
    assert!(
        crate::envelope_gesture::is_envelope(&sim, container),
        "o container carrega o VecEnvelope"
    );
    let child = *map.get(&ids[0]).expect("bits da entidade do filho");
    assert!(
        !crate::envelope_gesture::is_envelope(&sim, child),
        "o filho é forma comum, não um envelope"
    );
}

/// SONDA (`--ignored`): dump das alças da estrela deformada — QUAL vértice tem a alça que dispara,
/// e quão longe (relativo à diagonal da gaiola). Rodar:
/// `cargo test -p ph2d-host-desktop --bin ph2d-host-desktop probe_star_handle_lengths -- --ignored --nocapture`
#[test]
#[ignore = "sonda visual"]
fn probe_star_handle_lengths() {
    // A estrela EXATA do smoke =27: bbox [-2.4,-1.8]..[2.4,1.8].
    let src = cook(ShapeKind::Star, [-2.4, -1.8], [2.4, 1.8], &[5.0, 0.45, 0.0]);
    let (mut sim, mut scene, _map, container, ids) = envelope_over(vec![src]);
    let (origin, size) =
        super::super::union_control_bbox([&source_of(&env_of(&sim, container).children[0])])
            .unwrap();
    let diag = size[0].hypot(size[1]);
    let [bl, br, ..] = bbox_corners(origin, size);
    // Varre a FORÇA da perspectiva: topo cada vez mais estreito (o artista arrastando os cantos
    // pra dentro). Reporta a MAIOR alça em cada força — se ela dispara, é o bug.
    for k in [0.40, 0.25, 0.15, 0.08, 0.03] {
        let pulled = [
            bl,
            br,
            [origin[0] + size[0] * (0.5 + k), origin[1] + size[1]],
            [origin[0] + size[0] * (0.5 - k), origin[1] + size[1]],
        ];
        set_corners(&mut sim, container, pulled);
        let out = frame(&mut sim, &mut scene, ids[0]);
        let mut worst = 0.0f64;
        let mut worst_v = 0usize;
        for (i, v) in out.verts.iter().enumerate() {
            let li = (v.in_handle[0] - v.anchor[0]).hypot(v.in_handle[1] - v.anchor[1]);
            let lo = (v.out_handle[0] - v.anchor[0]).hypot(v.out_handle[1] - v.anchor[1]);
            if li.max(lo) > worst {
                worst = li.max(lo);
                worst_v = i;
            }
        }
        let flag = if worst > diag * 0.5 {
            "  <<<<< ALCA DISPAROU"
        } else {
            ""
        };
        println!(
            "topo={:.0}%  diag={diag:.2}  {} verts  MAIOR alca = {worst:.2} (v{worst_v}, {:.0}% do diag){flag}",
            (0.5 - k) * 100.0,
            out.verts.len(),
            worst / diag * 100.0
        );
    }
}
