//! **W0 — o spike do caminho implícito.** Mede a `fidget` contra as 6 perguntas de
//! `docs/3DModeling/03_plano_implicito.md` §6, e produz a imagem que o Enio julga.
//!
//! ```text
//! cargo run --release                 # intérprete (o default; sem `unsafe`)
//! cargo run --release --features jit  # com o JIT (assembly à mão) — a comparação do §6.5
//! ```
//!
//! Escreve em `out/`: um PNG por cena, uma folha de contato, e os STL correspondentes.
//! ⚠️ O relatório vai para o **stdout** em tabela — ele é a fonte de `01_resultados_spike.md`,
//! que **não** se escreve de memória.

mod probe;
mod render;
mod sdf;

use std::path::PathBuf;
use std::time::{Duration, Instant};

use fidget::context::Tree;
use fidget::mesh::{Octree, Settings};
use fidget::vm::VmShape;

/// A peça de teste é a que quebra o Bevel do Blender: **três volumes no mesmo vértice**.
/// Três cilindros ortogonais que se cruzam na origem dão, de uma vez: arestas curvas onde dois se
/// encontram, e um **vértice triplo** no meio — o caso que o rolling-ball do CAD costuma recusar.
fn junction(op: &dyn Fn(&Tree, &Tree) -> Tree) -> Tree {
    let r = 0.22;
    let h = 0.78;
    let a = sdf::sd_capped_cylinder_x(r, h);
    let b = sdf::sd_capped_cylinder_y(r, h);
    let c = sdf::sd_capped_cylinder_z(r, h);
    op(&op(&a, &b), &c)
}

struct Scene {
    key: &'static str,
    label: &'static str,
    tree: Tree,
}

fn scenes() -> Vec<Scene> {
    // ⚠️ O `k` do suave NÃO é o raio: ele é o alcance da mistura. Foi escolhido igual ao raio do
    // exato de propósito, para a imagem comparar dois operadores com o MESMO número na mão — que é
    // exatamente como um utilizador os encontraria num painel. Que eles NÃO produzam o mesmo
    // tamanho de filete é um dos achados, não um erro de montagem.
    let r = 0.12;

    vec![
        Scene {
            key: "1_cubo",
            label: "cubo — a prova da quina viva",
            tree: sdf::sd_box(0.45, 0.45, 0.45),
        },
        Scene {
            key: "2_juncao_sem_arredondar",
            label: "junção de 3 — união dura",
            tree: junction(&|a, b| sdf::union_sharp(a, b)),
        },
        Scene {
            key: "3_juncao_arredondamento_exato",
            label: "junção de 3 — arredondamento EXATO (r = 0,12)",
            tree: junction(&move |a, b| sdf::union_round(a, b, r)),
        },
        Scene {
            key: "4_juncao_arredondamento_organico",
            label: "junção de 3 — arredondamento ORGÂNICO (k = 0,12)",
            tree: junction(&move |a, b| sdf::union_smooth(a, b, r)),
        },
    ]
}

/// Malha a árvore e devolve `(vértices, triângulos, tempo)`.
fn mesh_at(tree: &Tree, depth: u8) -> (Vec<[f32; 3]>, Vec<[u32; 3]>, Duration) {
    let shape = VmShape::from(tree.clone());
    let bound_shape = shape.try_into().expect("a árvore não tem variáveis extras");
    let settings = Settings {
        depth,
        ..Default::default()
    };
    let t0 = Instant::now();
    let octree = Octree::build(&bound_shape, &settings).expect("meshing cancelado");
    let mesh = octree.walk_dual();
    let elapsed = t0.elapsed();

    let verts: Vec<[f32; 3]> = mesh.vertices.iter().map(|v| [v.x, v.y, v.z]).collect();
    let tris: Vec<[u32; 3]> = mesh
        .triangles
        .iter()
        .map(|t| [t.x as u32, t.y as u32, t.z as u32])
        .collect();
    (verts, tris, elapsed)
}

/// A ponte que a W2 vai precisar: malha da `fidget` -> `ph2d_mesh::Mesh`.
/// ⚠️ Está aqui **por medição**, não por conveniência: se ela não existir, o plano tem um buraco.
fn to_ph2d(verts: &[[f32; 3]], tris: &[[u32; 3]]) -> Result<ph2d_mesh::Mesh, String> {
    let faces: Vec<ph2d_mesh::Face> = tris
        .iter()
        .map(|t| ph2d_mesh::Face::tri(t[0], t[1], t[2]))
        .collect();
    ph2d_mesh::Mesh::from_parts(verts.to_vec(), faces).map_err(|e| format!("{e:?}"))
}

/// Os 8 cantos teóricos de um cubo centrado na origem, meia-aresta `half`.
fn cube_corners(half: f64) -> Vec<[f64; 3]> {
    let mut v = Vec::with_capacity(8);
    for sx in [-1.0, 1.0] {
        for sy in [-1.0, 1.0] {
            for sz in [-1.0, 1.0] {
                v.push([sx * half, sy * half, sz * half]);
            }
        }
    }
    v
}

/// Pico de memória residente do processo, em MiB (Linux). Devolve `None` noutros sistemas.
fn peak_rss_mib() -> Option<f64> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    for line in status.lines() {
        if let Some(rest) = line.strip_prefix("VmHWM:") {
            let kb: f64 = rest.trim().trim_end_matches(" kB").trim().parse().ok()?;
            return Some(kb / 1024.0);
        }
    }
    None
}

fn main() {
    let out = PathBuf::from("out");
    let backend = if cfg!(feature = "jit") {
        "JIT (assembly à mão)"
    } else {
        "intérprete (sem unsafe)"
    };

    println!("# Spike do campo implícito — fidget 0.5.0");
    println!("backend: {backend}\n");

    // ── 1. Quina viva + erro de superfície + a imagem ───────────────────────────────
    const IMAGE_DEPTH: u8 = 7;
    println!("## 1. Quina viva e erro de superfície (profundidade {IMAGE_DEPTH})\n");
    let cell = 2.0 / (1u32 << IMAGE_DEPTH) as f64;
    println!("célula da grade: {cell:.5} (o mundo vai de −1 a 1)\n");
    println!(
        "| cena | triângulos | vértices | tempo | erro médio nos vértices | erro máx | máx em células |"
    );
    println!("|---|---:|---:|---:|---:|---:|---:|");

    let mut panels = Vec::new();
    let mut zooms = Vec::new();
    for scene in scenes() {
        let (verts, tris, dt) = mesh_at(&scene.tree, IMAGE_DEPTH);
        let field = probe::Field::new(&scene.tree);
        // ⚠️ Nos VÉRTICES, não no baricentro — ver `probe::vertex_error`.
        let err = probe::vertex_error(&field, &verts);

        println!(
            "| {} | {} | {} | {:.1} ms | {:.2e} | {:.2e} | {:.3} |",
            scene.label,
            tris.len(),
            verts.len(),
            dt.as_secs_f64() * 1000.0,
            err.mean_abs,
            err.max_abs,
            err.max_abs / cell
        );

        // A ponte para a malha da casa — se falhar, é achado.
        match to_ph2d(&verts, &tris) {
            Ok(m) => {
                let piece = ph2d_mesh::ExportPiece {
                    name: Some(scene.key),
                    mesh: &m,
                    pose: ph2d_mesh::Pose::default(),
                };
                let stl = ph2d_mesh::write_stl(&[piece]);
                std::fs::create_dir_all(&out).ok();
                std::fs::write(out.join(format!("{}.stl", scene.key)), stl).ok();
            }
            Err(e) => println!(
                "\n⚠️ ponte fidget -> ph2d_mesh RECUSOU `{}`: {e}\n",
                scene.key
            ),
        }

        let view = render::View::default();
        let rgba = render::render(&view, &verts, &tris);
        render::write_png(
            &out.join(format!("{}.png", scene.key)),
            view.width,
            view.height,
            &rgba,
        )
        .expect("gravar png");
        panels.push((view.width, view.height, rgba));

        // ⚠️ A aproximação NÃO é enfeite: o defeito que a vista geral esconde é o dente-de-serra
        // da aresta, e ele só se julga de perto. Uma imagem que não pode reprovar não serve.
        let zoom = render::View {
            scale: 7.0,
            target: [0.30, 0.30, 0.22],
            ..Default::default()
        };
        let rgba_z = render::render(&zoom, &verts, &tris);
        render::write_png(
            &out.join(format!("{}_zoom.png", scene.key)),
            zoom.width,
            zoom.height,
            &rgba_z,
        )
        .expect("gravar png de zoom");
        zooms.push((zoom.width, zoom.height, rgba_z));
    }

    let (cw, ch, sheet) = render::contact_sheet(&panels, 10);
    render::write_png(&out.join("00_comparativo.png"), cw, ch, &sheet).expect("gravar folha");
    let (zw, zh, zsheet) = render::contact_sheet(&zooms, 10);
    render::write_png(&out.join("00_comparativo_zoom.png"), zw, zh, &zsheet).expect("gravar folha");
    println!(
        "\n→ `out/00_comparativo.png` (geral) e `out/00_comparativo_zoom.png` (a aresta de perto)\n"
    );

    // ── 1b. A quina viva, em número ─────────────────────────────────────────────────
    println!("## 1b. A quina viva do cubo, medida\n");
    println!("Tudo em **frações de célula** — é a unidade que diz se refinar cura.\n");
    println!(
        "| meia-aresta | prof. | célula | canto médio | canto pior | aresta média | aresta pior | fatias capturadas | vazias |"
    );
    println!("|---:|---:|---:|---:|---:|---:|---:|---:|---:|");
    let cube_probe = |half: f64, depth: u8| {
        let (verts, _t, _d) = mesh_at(&sdf::sd_box(half, half, half), depth);
        let cell = 2.0 / (1u32 << depth) as f64;
        let corners = cube_corners(half);
        let (mean_c, worst_c) = probe::corner_capture(&verts, &corners);
        let e = probe::edge_capture(&verts, half, half, half * 0.85, cell);
        println!(
            "| {half} | {depth} | {cell:.5} | {:.2} | {:.2} | {:.2} | {:.2} | {}/{} | {} |",
            mean_c / cell,
            worst_c / cell,
            e.mean_cells,
            e.worst_cells,
            e.captured,
            e.slabs,
            e.empty
        );
    };
    // Refinar cura? (mesma peça, três resoluções)
    for depth in [6u8, 7, 8] {
        cube_probe(0.45, depth);
    }
    // ⚠️ E o MECANISMO: o serrilhado é do algoritmo, ou de a peça não cair na grade?
    // 0.5 e 0.25 caem em fronteira de célula em qualquer profundidade; 0.45 e 0.4703125 não.
    // Se os alinhados saírem limpos e os outros não, a causa está NOMEADA — e o caso do
    // utilizador real é o desalinhado, porque ninguém modela em múltiplos da grade.
    println!(
        "\n**O mecanismo** — a mesma sonda, mudando só o alinhamento com a grade (prof. 7):\n"
    );
    println!(
        "| meia-aresta | face em células | fração | canto médio | canto pior | aresta média | aresta pior | capturadas |"
    );
    println!("|---:|---:|---:|---:|---:|---:|---:|---:|");
    for half in [0.5_f64, 0.25, 0.45, 0.4703125, 0.4609375, 0.4765625] {
        let (verts, _t, _d) = mesh_at(&sdf::sd_box(half, half, half), 7);
        let cell = 2.0 / (1u32 << 7u8) as f64;
        let corners = cube_corners(half);
        let (mean_c, worst_c) = probe::corner_capture(&verts, &corners);
        let e = probe::edge_capture(&verts, half, half, half * 0.85, cell);
        // ⚠️ A variável que EXPLICA: onde a face cai dentro da célula. 0,0 = exatamente sobre a
        // fronteira da grade (o caso degenerado do Dual Contouring, em que a superfície passa
        // pelos próprios cantos da célula e o sinal fica ambíguo).
        let in_cells = half / cell;
        println!(
            "| {half} | {in_cells:.2} | {:.2} | {:.2} | {:.2} | {:.2} | {:.2} | {}/{} |",
            in_cells.fract(),
            mean_c / cell,
            worst_c / cell,
            e.mean_cells,
            e.worst_cells,
            e.captured,
            e.slabs
        );
    }

    // ── 2. O campo continua sendo uma DISTÂNCIA? ────────────────────────────────────
    println!("## 2. Propriedade de distância (Eikonal: ‖∇f‖ deve ser 1)\n");
    println!("| cena | amostras | média | mín | máx | desvio máx | pior ponto |");
    println!("|---|---:|---:|---:|---:|---:|---|");
    for scene in scenes() {
        let field = probe::Field::new(&scene.tree);
        let e = probe::eikonal(&field, 0.05, 4000, 0x5EED);
        println!(
            "| {} | {} | {:.4} | {:.4} | {:.4} | **{:.4}** | ({:.2}, {:.2}, {:.2}) f={:.3} |",
            scene.label,
            e.samples,
            e.mean_norm,
            e.min_norm,
            e.max_norm,
            e.max_deviation,
            e.worst_at[0],
            e.worst_at[1],
            e.worst_at[2],
            e.worst_f
        );
    }

    // ── 2b. Isolar a CULPA: é o operador, ou é encadeá-lo? ──────────────────────────
    // A junção de 3 aplica o operador DUAS vezes (`op(op(a,b), c)`). Se o campo degrada, é preciso
    // saber se uma aplicação já degrada ou se o dano nasce da segunda — são curas diferentes.
    println!("\n## 2b. Uma aplicação contra duas encadeadas (2 cilindros vs 3)\n");
    println!("| operador | aplicações | média | mín | máx | desvio máx |");
    println!("|---|---:|---:|---:|---:|---:|");
    let rr = 0.12;
    let (cx, cy, cz) = (
        sdf::sd_capped_cylinder_x(0.22, 0.78),
        sdf::sd_capped_cylinder_y(0.22, 0.78),
        sdf::sd_capped_cylinder_z(0.22, 0.78),
    );
    for (name, one, two) in [
        (
            "exato",
            sdf::union_round(&cx, &cy, rr),
            sdf::union_round(&sdf::union_round(&cx, &cy, rr), &cz, rr),
        ),
        (
            "orgânico",
            sdf::union_smooth(&cx, &cy, rr),
            sdf::union_smooth(&sdf::union_smooth(&cx, &cy, rr), &cz, rr),
        ),
    ] {
        for (n, tree) in [(1, one), (2, two)] {
            let f = probe::Field::new(&tree);
            let e = probe::eikonal(&f, 0.05, 4000, 0x5EED);
            println!(
                "| {name} | {n} | {:.4} | {:.4} | {:.4} | **{:.4}** |",
                e.mean_norm, e.min_norm, e.max_norm, e.max_deviation
            );
        }
    }

    // ── 3. O raio pedido é o raio entregue? ─────────────────────────────────────────
    println!("\n## 3. O raio pedido é o raio entregue? (sonda analítica, sem malha)\n");
    println!("| operador | raio pedido | raio entregue | erro | erro relativo |");
    println!("|---|---:|---:|---:|---:|");
    for r in [0.05_f64, 0.12, 0.25] {
        let e_exact = probe::radius_error(&|a, b| sdf::union_round(a, b, r), r);
        println!(
            "| exato | {r:.2} | {:.4} | {:.3e} | {:.2} % |",
            r + e_exact,
            e_exact,
            (e_exact / r).abs() * 100.0
        );
        let e_smooth = probe::radius_error(&|a, b| sdf::union_smooth(a, b, r), r);
        println!(
            "| orgânico | {r:.2} | {:.4} | {:.3e} | **{:.1} %** |",
            r + e_smooth,
            e_smooth,
            (e_smooth / r).abs() * 100.0
        );
    }

    // ── 4. Resolução × tempo × memória ──────────────────────────────────────────────
    println!("\n## 4. Resolução × tempo × memória (junção com arredondamento exato)\n");
    println!("| profundidade | grade | triângulos | tempo | pico RSS |");
    println!("|---:|---:|---:|---:|---:|");
    let r = 0.12;
    let tree = junction(&move |a, b| sdf::union_round(a, b, r));
    for depth in [5u8, 6, 7, 8] {
        let (_v, tris, dt) = mesh_at(&tree, depth);
        println!(
            "| {} | {}³ | {} | {:.1} ms | {} |",
            depth,
            1usize << depth,
            tris.len(),
            dt.as_secs_f64() * 1000.0,
            peak_rss_mib().map_or("—".into(), |m| format!("{m:.0} MiB"))
        );
    }

    // ── 5. Determinismo (HR-5) ──────────────────────────────────────────────────────
    println!("\n## 5. Determinismo — a mesma entrada dá a mesma malha?\n");
    let (v1, t1, _) = mesh_at(&tree, 6);
    let (v2, t2, _) = mesh_at(&tree, 6);
    let same = v1 == v2 && t1 == t2;
    println!(
        "duas corridas na profundidade 6: **{}** ({} vértices, {} triângulos)",
        if same {
            "byte-idênticas"
        } else {
            "⚠️ DIVERGIRAM"
        },
        v1.len(),
        t1.len()
    );
}
