//! ⏱️ **O QUE A PELE DE IMAGEM CUSTA À CPU NUM QUADRO** — irmão do [`super`] pelo tecto de LOC,
//! cortado por RESPONSABILIDADE: *a 2.ª mídia desenha o que deve* é uma pergunta, *quanto ela custa
//! por peça* é outra, e a segunda é uma sonda sem barra.
//!
//! ⚠️ Filho do arnês do irmão (`#[path]`) para herdar as fixturas dele — uma cópia divergiria no
//! primeiro ajuste.

use super::*;

/// Uma malha SINTÉTICA de `cols × rows` células (dois triângulos cada) sobre a imagem `size_px` — o
/// que a sonda de custo precisa para escolher o número de peças.
fn malha_grelha(size_px: [u32; 2], cols: u32, rows: u32) -> Mesh2d {
    let (w, h) = (f64::from(size_px[0]), f64::from(size_px[1]));
    let mut m = Mesh2d {
        rest: Vec::new(),
        tris: Vec::new(),
        size: size_px,
    };
    for j in 0..=rows {
        for i in 0..=cols {
            m.rest.push([
                w * f64::from(i) / f64::from(cols),
                h * f64::from(j) / f64::from(rows),
            ]);
        }
    }
    let id = |i: u32, j: u32| j * (cols + 1) + i;
    for j in 0..rows {
        for i in 0..cols {
            m.tris.push([id(i, j), id(i + 1, j), id(i + 1, j + 1)]);
            m.tris.push([id(i, j), id(i + 1, j + 1), id(i, j + 1)]);
        }
    }
    m
}

/// ⏱️ **SONDA (`--ignored`) — O QUE A PELE DE IMAGEM CUSTA À CPU NUM QUADRO** (plano 03, W4).
///
/// O `SKIN_FRAME_PIECES` foi derivado do buffer do Vello, que este caminho já não gasta. O recurso
/// que sobra do lado da CPU é o TEMPO do quadro, e é isto que ele mede, por número de peças:
/// descodificar a malha guardada (postcard, **por quadro**), deformar cada vértice, refinar
/// (`Smooth`) e montar o `SpriteMesh`.
///
/// ⚠️ **Acima de `load ~5` uma leitura de relógio desta workstation não vale nada** (`CLAUDE.md`
/// §5.0) — a carga é impressa ao lado. Corre com:
/// `cargo test -p ph2d-skeleton-live --lib -- --ignored --nocapture measure_the_cpu_cost`
#[test]
#[ignore = "sonda: imprime a tabela do custo, sem barra"]
fn measure_the_cpu_cost_of_a_skinned_frame() {
    use std::time::Instant;

    let carga = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!(
        "carga: {}",
        carga
            .split_whitespace()
            .take(3)
            .collect::<Vec<_>>()
            .join(" ")
    );
    println!("ms: MINIMO/mediana de 40 corridas (a media sob contencao mede o vizinho)");
    println!(
        "{:>8} | {:>13} | {:>13} | {:>13} | {:>12}",
        "pecas", "descodificar", "Fast", "Smooth", "pecas Smooth"
    );

    for (cols, rows) in [(6_u32, 6_u32), (12, 12), (24, 24), (48, 48), (72, 72)] {
        let mut sim = SimWorld::default();
        let raiz = crate::bone::create(&mut sim, None, [-2.0, 0.0], [0.0, 0.0]).expect("raiz");
        let raiz = Entity::from_bits(raiz);
        let ponta =
            crate::bone::create(&mut sim, Some(raiz), [0.0, 0.0], [2.0, 0.0]).expect("ponta");
        let e = sim
            .world_mut()
            .spawn((Transform::IDENTITY, sprite(4.0, 2.0, 0.0, 0.0)))
            .id();
        assert!(crate::skin_live::bind_image(
            &mut sim,
            e,
            &tinta(40, 20, 2, 4),
            [40, 20],
            PPM,
            GridOptions::default(),
            Some(raiz),
        ));
        // A malha guardada é substituída pela sintética: é ela que fixa o número de peças.
        //
        // ⚠️⚠️ **Ela leva uma TABELA DE PESOS, e a razão é que o produto mudou por baixo desta
        // sonda:** desde que a pele de imagem passou ao padrão-ouro o `source` guarda uma
        // [`SkinnedMesh`], e escrever aqui uma `Mesh2d` crua produzia bytes que o
        // `skinned_mesh_of` **recusa** — a sonda é `#[ignore]`, logo isso ficava verde a medir
        // ZERO peles. *Uma sonda que o CI nunca corre é o sítio onde um formato novo se esconde.*
        //
        // ⭐ Os pesos são UNIFORMES de propósito: o relógio não depende do VALOR deles, e uniforme
        // é o **pior caso** do `blend` (nenhum osso é saltado por peso zero, o que a solução real
        // faz na maioria dos vértices). *Medir o limite superior é o lado certo para errar.*
        let malha = malha_grelha([40, 20], cols, rows);
        let pecas = malha.tris.len();
        let ossos_n = sim
            .world()
            .get::<ph2d_skeleton_ecs::SkinBind>(e)
            .expect("pele")
            .tendons
            .len();
        #[expect(clippy::cast_precision_loss, reason = "duas casas")]
        let uniforme = 1.0 / ossos_n as f64;
        let guardada = crate::skinned_mesh::SkinnedMesh {
            pesos: vec![uniforme; malha.rest.len() * ossos_n],
            mesh: malha,
        };
        let bytes = postcard::to_allocvec(&guardada).expect("serializa");
        {
            let mut skin = sim
                .world_mut()
                .get_mut::<ph2d_skeleton_ecs::SkinBind>(e)
                .expect("pele");
            skin.source = bytes;
        }
        sim.world_mut()
            .get_mut::<Transform>(Entity::from_bits(ponta))
            .expect("Transform da ponta")
            .rotation = 1.0;
        let s = sprite(4.0, 2.0, 0.0, 0.0);
        let mut present = PresentWorld::new();
        let p = present
            .world_mut()
            .spawn((SimRef(e), instancia_de(&s)))
            .id();

        const RONDAS: usize = 40;
        let mut descodif = Vec::with_capacity(RONDAS);
        for _ in 0..RONDAS {
            let t = Instant::now();
            let m = mesh_of(&sim, e).expect("malha");
            descodif.push(t.elapsed().as_secs_f64() * 1e3);
            assert_eq!(m.tris.len(), pecas);
        }
        let mut medir = |modo: Option<RefineOptions>| -> Vec<f64> {
            let mut ms = Vec::with_capacity(RONDAS);
            for _ in 0..RONDAS {
                present.world_mut().entity_mut(p).remove::<SpriteMesh>();
                let t = Instant::now();
                attach_skin_meshes(&sim, &mut present, PPM, modo, PX_POR_METRO, None);
                ms.push(t.elapsed().as_secs_f64() * 1e3);
            }
            ms
        };
        let descodif = melhor(descodif);
        let fast = melhor(medir(None));
        let suave = melhor(medir(Some(RefineOptions {
            tolerance_px: 0.5,
            max_pieces: SKIN_FRAME_PIECES,
        })));
        let saiu = present
            .world()
            .get::<SpriteMesh>(p)
            .map_or(0, |m| m.tris.len());
        println!(
            "{pecas:>8} | {:>6.3}/{:<6.3} | {:>6.3}/{:<6.3} | {:>6.3}/{:<6.3} | {saiu:>12}",
            descodif.0, descodif.1, fast.0, fast.1, suave.0, suave.1
        );
    }
}

/// `(mínimo, mediana)` de amostras de relógio, em ms.
///
/// ⚠️⚠️ **O MÍNIMO é a leitura que sobrevive a esta workstation.** O `CLAUDE.md` §5.0 diz que acima
/// de `load ~5` nenhum relógio daqui vale nada — e a carga de FUNDO desta máquina (o editor, o
/// rust-analyzer, o sccache, as outras sessões) fica em `~7` sem ninguém compilar. O mínimo de `N`
/// corridas é o custo quando o escalonador deu o núcleo; a mediana ao lado diz quanto a máquina
/// estava a roubar. *Uma média sob contenção mede o vizinho.*
fn melhor(mut ms: Vec<f64>) -> (f64, f64) {
    ms.sort_by(|a, b| a.partial_cmp(b).expect("sem NaN no relogio"));
    (ms[0], ms[ms.len() / 2])
}
