//! **SONDA — o raio ainda acerta onde a superfície está, depois de a topologia
//! mudar sob ele?**
//!
//! O report do Enio é *"o lugar onde o mouse toca não corresponde ao local na
//! malha"*, e o raycast lê o OCTREE. A jornada não-integrada (W9) reescreveu a
//! forma como a malha absorve topologia — o refino APENDA e o colapso RENUMERA
//! —, e as duas mexem no índice espacial. Se o octree passar a descrever a malha
//! de antes, o raio pousa noutro triângulo e o dab cai fora do cursor.
//!
//! Ela dirige as portas do PRODUTO na ordem do `refine_for_dab` (colapso, depois
//! refino) e mede a distância entre dois acertos do MESMO raio.

use ph2d_mesh::{
    Birth, Ray, RegionScratch, Remap, collapse_in_sphere, collapse_target, edge_target,
    refine_in_sphere, shapes,
};

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

#[test]
#[ignore = "sonda"]
fn the_same_ray_lands_in_the_same_place_after_the_topology_moves() {
    // ⚠️ **Esfera GROSSA de propósito.** A primeira versão desta sonda usava
    // 48×72, cuja aresta (~0,087) já é menor que o alvo do refino (0,121) e
    // maior que o do colapso (0,059): a contagem saía IDÊNTICA nos dois lados e
    // a sonda media a topologia parada — verde sobre nada. A fixture tem de
    // conter o fenômeno.
    let mut mesh = shapes::uv_sphere(12, 16, 1.0);
    mesh.triangulate();

    // Sete raios espalhados pela calota frontal — um só acerto pode estar certo
    // por sorte de qual folha o octree consultou.
    let aims = [
        [0.0f32, 0.0],
        [0.30, 0.20],
        [-0.35, 0.10],
        [0.15, -0.40],
        [-0.20, -0.25],
        [0.45, 0.05],
        [0.05, 0.45],
    ];

    let mut births: Vec<Birth> = Vec::new();
    let mut remap = Remap::default();
    let mut region = RegionScratch::default();

    let radius = 0.35f32;
    let target = edge_target(radius, 0.5);

    println!(
        "malha inicial: {} vertices / {} faces",
        mesh.vert_count(),
        mesh.faces().len()
    );

    let mut worst = 0.0f32;
    let mut misses = 0;
    for (k, [x, y]) in aims.into_iter().enumerate() {
        let ray = Ray::new([x, y, 5.0], [0.0, 0.0, -1.0]);
        let Some(before) = mesh.raycast(&ray) else {
            println!("  [{k}] o raio errou a esfera ANTES — aim ({x}, {y})");
            misses += 1;
            continue;
        };

        // A ordem do `refine_for_dab`: colapso primeiro, refino depois.
        let shrunk = collapse_in_sphere(
            &mut mesh,
            before.point,
            radius,
            collapse_target(target),
            &mut remap,
            &mut region,
        );
        let grown = refine_in_sphere(
            &mut mesh,
            before.point,
            radius,
            target,
            &mut births,
            &mut region,
        );

        let Some(after) = mesh.raycast(&ray) else {
            println!(
                "  [{k}] aim ({x}, {y}) -> ACERTO SUMIU depois da topologia \
                 (colapso {shrunk:?}, refino {grown:?})"
            );
            misses += 1;
            continue;
        };
        let d = dist(before.point, after.point);
        worst = worst.max(d);
        println!(
            "  [{k}] aim ({x:>5.2}, {y:>5.2}) antes {:?} depois {:?} desvio {d:.6} \
             | malha {} v / {} f",
            before.point,
            after.point,
            mesh.vert_count(),
            mesh.faces().len()
        );
    }
    println!("PIOR DESVIO: {worst:.6} | acertos perdidos: {misses}");
}
