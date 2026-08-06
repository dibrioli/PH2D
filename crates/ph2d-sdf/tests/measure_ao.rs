//! **O QUE CUSTA UM BAKE DE AO** — o campo, o traço, o orçamento e o paralelismo,
//! medidos ANTES de a wave ser desenhada.
//!
//! ⚠️ A pergunta não é *"quanto custa"* e sim **QUEM PAGA**: se o campo couber no
//! pen-up de um traço, a oclusão acompanha a escultura e o artista nunca vê um
//! número velho. Se não couber, o AO é um **BOTÃO** — e aí a obsolescência é
//! inerente ao desenho e tem de ser DITA, não escondida.
//!
//! # O que foi medido (RTX / 32 núcleos, 2026-08-06)
//!
//! | malha | verts | res | criar | voxelizar | flood | células |
//! |---|---|---|---|---|---|---|
//! | uv_sphere(96,144)  |  13 682 |  64 | 0,2 | **20,2**  |  2,1 |   328 509 |
//! | uv_sphere(96,144)  |  13 682 | 128 | 1,1 | **40,8**  | 12,1 | 2 352 637 |
//! | uv_sphere(96,144)  |  13 682 | 192 | 3,5 | **71,4**  | 55,4 | 7 645 373 |
//! | uv_sphere(320,480) | 153 122 |  64 | 0,0 | **104,9** |  1,7 |   328 509 |
//! | uv_sphere(320,480) | 153 122 | 128 | 0,7 | **143,2** | 14,5 | 2 352 637 |
//! | uv_sphere(320,480) | 153 122 | 192 | 5,1 | **186,6** | 41,6 | 7 645 373 |
//! | uv_sphere(533,800) | 425 602 |  64 | 0,0 | **229,2** |  1,8 |   328 509 |
//! | uv_sphere(533,800) | 425 602 | 128 | 0,3 | **289,1** | 11,8 | 2 352 637 |
//! | uv_sphere(533,800) | 425 602 | 192 | 4,5 | **347,4** | 38,6 | 7 645 373 |
//!
//! # As DUAS leituras, e a segunda contradiz o palpite
//!
//! **(1) O veredito de produto: o campo NÃO cabe num pen-up.** Na malha que a cena
//! `=16` abre (425 k vértices) o par voxelizar+flood custa **231 a 386 ms** — de
//! 14× a 23× um quadro de 60 fps, e isso ANTES de o cone tracing começar. ⇒ o AO
//! assado é um **BOTÃO explícito**, como o REMESH da W7, e a nota do
//! `06.1-Waves-riscos-e-alvos` §W10.1 (*"o AO assado é o irmão desta wave"*)
//! precisa dizer isso: irmão no CANAL de vértice, não no ciclo de vida.
//!
//! **(2) A decomposição, que é o oposto do que um campo de voxel sugere.**
//! Voxelizar é limitado por **TRIÂNGULO**, não por célula: com a malha fixa em
//! 425 k, subir a resolução de 64 para 192 (**23× as células**) custa só
//! **1,52×**; com a resolução fixa em 64, subir a malha de 13,7 k para 425 k
//! (**31× os triângulos**) custa **11,3×**. O flood fill é o inverso — limitado
//! por CÉLULA, quase exatamente linear nelas (1 : 6,6 : 21 contra 1 : 7,2 : 23).
//!
//! ⇒ **Um campo FINO é barato; uma malha densa é cara.** Quem quiser AO de alta
//! frequência sobe a resolução (quase de graça); quem esculpir denso paga na
//! voxelização, e a alavanca ali é o `rayon` — que a `ph2d-sdf` **não tem, e a
//! ausência está escrita no `Cargo.toml` dela com o mecanismo** (as caixas de dois
//! triângulos se SOBREPÕEM ⇒ a escrita não é disjunta, a condição que o ADR-0109
//! exige). Paralelizar isto quer ADR próprio, não uma linha.
//!
//! # O TRAÇO, medido depois (RTX / 32 núcleos, 2026-08-06)
//!
//! O handoff de continuação §4.1 nomeava isto como *"o que ainda NÃO foi
//! medido"*, e a resposta muda o retrato: **o traço é a metade GRANDE.**
//!
//! | malha | verts | cones | steps | traço ms | ns/vértice |
//! |---|---|---|---|---|---|
//! | uv_sphere(96,144)  |  13 682 |  8 | 24 |   8,0 |   585 |
//! | uv_sphere(96,144)  |  13 682 | 32 | 24 |  31,6 | 2 312 |
//! | uv_sphere(96,144)  |  13 682 | 32 | 48 |  32,8 | 2 398 |
//! | uv_sphere(320,480) | 153 122 |  8 | 24 |  74,2 |   485 |
//! | uv_sphere(320,480) | 153 122 | 32 | 24 | 303,8 | 1 984 |
//! | uv_sphere(533,800) | 425 602 |  8 | 24 | 192,0 |   451 |
//! | **uv_sphere(533,800)** | **425 602** | **32** | **24** | **786,1** | **1 847** |
//! | uv_sphere(533,800) | 425 602 | 32 | 48 | 780,2 | 1 833 |
//!
//! **(3) O traço é 2,6× o campo.** Na malha da cena `=16` o campo a `res 128`
//! custa 301 ms e o traço **786 ms** ⇒ um bake completo é **~1,09 s**. A nota
//! antiga (*"231-386 ms, não cabe num pen-up"*) contava só a metade barata; o
//! veredito do BOTÃO não muda, ele fica mais forte.
//!
//! **(4) Linear nos dois eixos que importam, e o `ns/vértice` é a prova:** ele
//! fica em ~1 850 de 13 k a 425 k vértices (a pegada de um vértice não sabe o
//! tamanho da malha), e sobe 4,1× para 4× os cones.
//!
//! # O ORÇAMENTO — onde a resposta PARA de mudar (aro interno de um toro)
//!
//! | radius | steps 12 | 24 | 48 | 96 | traço ms |
//! |---|---|---|---|---|---|
//! | 0,34 | 0,9511 | 0,9498 | 0,9498 | 0,9498 | 5,7-6,3 |
//! | 0,50 | 0,9422 | 0,9395 | 0,9395 | 0,9395 | 5,6-5,9 |
//! | 1,00 | 0,9119 | 0,9091 | 0,9091 | 0,9091 | 5,9 |
//! | 1,50 | 0,8597 | 0,8569 | 0,8569 | 0,8569 | 6,0-6,1 |
//! | 2,00 | 0,8411 | 0,8384 | 0,8384 | 0,8384 | 6,1-6,2 |
//!
//! **(5) Os passos SATURAM em 24, medido.** De 24 para 96 a resposta não move
//! **um dígito** em nenhum raio ⇒ `max_steps = 24` está no joelho, e não é um
//! número escolhido. O teto de passos **não é a restrição que morde**: a marcha
//! termina por `t > radius`, não por acabar o orçamento.
//!
//! **(6) ⚠️ O ALCANCE É DE GRAÇA, e isso contradiz a intuição.** O custo é
//! **plano** (5,6 a 6,3 ms) enquanto o raio cresce 6× — porque a marcha de
//! esfera acelera em espaço aberto: um raio maior não compra mais passos, ele
//! deixa os mesmos passos irem mais longe. ⇒ o default `maior lado ÷ 8` é
//! tímido **sem economizar nada**, e onde ele deve pousar é decisão de LOOK
//! (smoke), não de custo.
//!
//! # O PARALELISMO — 425 602 vértices, 32 cones, 24 passos
//!
//! | threads | ms | speedup | bit-idêntico |
//! |---|---|---|---|
//! |  1 | 807,7 |  1,00 | — |
//! |  2 | 403,5 |  2,00 | sim |
//! |  4 | 226,5 |  3,57 | sim |
//! |  8 | 133,4 |  6,06 | sim |
//! | 16 |  72,5 | 11,14 | sim |
//! | **32** | **43,7** | **18,49** | **sim** |
//!
//! **(7) 18,49×, e byte-idêntico em TODA contagem de threads.** O laço é um
//! *gather* — cada vértice escreve só o seu, contra um campo imutável, e a soma
//! sobre cones acontece **dentro** de um vértice (a ordem dela não muda com o
//! escalonamento). São os três invariantes do ADR-0109, e a identidade não é
//! argumentada: foi medida em 2, 4, 8, 16 e 32.
//!
//! ⚠️ **Este número foi o que ESCREVEU o ADR-0156**, e não o que o dispensou: a
//! cerca de contenção do ADR-0109 §2 diz que *qualquer novo uso de
//! rayon/threading, nesta ou em outra crate, exige novo ADR*. O ADR existe, com
//! a tabela acima dentro dele, e autoriza **`bake_ao` e nada mais**.
//!
//! **(8) E o ganho MOVE a fronteira.** Medido pela rota que shipa, o traço cai
//! para **36,9 ms** a 425 k (87 ns/vértice) ⇒ o bake completo vai de **~1,09 s
//! para ~338 ms**, e o **campo passa a ser 89% dele**. A voxelização é
//! justamente a metade que não paraleliza sem resolver a sobreposição de
//! escrita. **Quem for atrás do próximo ganho vai ali, não aqui.**
//!
//! Rodar: `cargo test -p ph2d-sdf --release --test measure_ao -- --ignored --nocapture`

use ph2d_mesh::{Mesh, shapes};
use ph2d_sdf::{AoParams, VoxelField, bake_ao};
use std::time::Instant;

/// O campo pronto, do jeito que um bake o constroi.
fn field_of(mesh: &Mesh, res: u32) -> VoxelField {
    let mut f = VoxelField::for_bounds(mesh.bounds(), res);
    f.voxelize(mesh);
    f.flood_fill();
    f
}

fn ms(t: Instant) -> f64 {
    t.elapsed().as_secs_f64() * 1000.0
}

fn mean(xs: &[f32]) -> f32 {
    xs.iter().sum::<f32>() / xs.len() as f32
}

#[test]
#[ignore = "sonda"]
fn measure_the_field_a_bake_would_need() {
    println!("\n== O CAMPO (voxelizar + flood fill), o pre-requisito de um AO assado ==");
    println!(
        "{:>20} {:>9} {:>6} {:>12} {:>12} {:>12} {:>10}",
        "malha", "verts", "res", "criar ms", "voxelizar", "flood ms", "celulas"
    );
    for (u, v) in [(96usize, 144usize), (320, 480), (533, 800)] {
        let mesh = shapes::uv_sphere(u, v, 1.0);
        for res in [64u32, 128, 192] {
            let t = Instant::now();
            let mut f = VoxelField::for_bounds(mesh.bounds(), res);
            let make = t.elapsed().as_secs_f64() * 1000.0;
            let t = Instant::now();
            f.voxelize(&mesh);
            let vox = t.elapsed().as_secs_f64() * 1000.0;
            let t = Instant::now();
            f.flood_fill();
            let flood = t.elapsed().as_secs_f64() * 1000.0;
            println!(
                "{:>20} {:>9} {:>6} {make:>12.1} {vox:>12.1} {flood:>12.1} {:>10}",
                format!("uv_sphere({u},{v})"),
                mesh.vert_count(),
                res,
                f.cell_count()
            );
        }
    }
    println!(
        "\nLeitura: voxelizar escala com TRIANGULOS (11,3x para 31x a malha), o flood com\n\
         CELULAS (21x para 23x as celulas). Um campo FINO e' barato; uma malha densa nao.\n\
         E 231-386 ms na malha de 425k NAO cabe num pen-up ==> o AO assado e' um BOTAO."
    );
}

/// **O QUE O CONE TRACING CUSTA** — a metade que a tabela do cabeçalho não
/// tinha, e a que o handoff de continuação §4.1 manda medir ANTES de desenhar o
/// resto.
#[test]
#[ignore = "sonda"]
fn measure_the_cone_trace() {
    println!("\n== O TRACO (cone tracing contra o campo), por vertice ==");
    println!(
        "{:>20} {:>9} {:>7} {:>7} {:>12} {:>14}",
        "malha", "verts", "cones", "steps", "traco ms", "ns/vertice"
    );
    for (u, v) in [(96usize, 144usize), (320, 480), (533, 800)] {
        let mesh = shapes::uv_sphere(u, v, 1.0);
        let field = field_of(&mesh, 128);
        let base = AoParams::for_bounds(mesh.bounds());
        for (cones, steps) in [(8usize, 24usize), (32, 24), (32, 48)] {
            let p = AoParams {
                cones,
                max_steps: steps,
                ..base
            };
            let t = Instant::now();
            let ao = bake_ao(&field, mesh.positions(), mesh.normals(), p);
            let el = ms(t);
            println!(
                "{:>20} {:>9} {:>7} {:>7} {el:>12.1} {:>14.0}",
                format!("uv_sphere({u},{v})"),
                mesh.vert_count(),
                cones,
                steps,
                el * 1e6 / mesh.vert_count() as f64
            );
            std::hint::black_box(ao);
        }
    }
    println!(
        "\nLeitura: o custo por vertice e' CONSTANTE (a pegada de um vertice nao sabe o\n\
         tamanho da malha), entao o total e' linear em vertices e linear em cones."
    );
}

/// **O ORÇAMENTO MUDA A RESPOSTA?** — um teto de passos que corta a marcha antes
/// do oclusor não fica mais barato: ele fica ERRADO, e de um jeito que parece
/// uma peça mais clara em vez de um bug.
#[test]
#[ignore = "sonda"]
fn measure_when_the_budget_stops_changing_the_answer() {
    let mesh = shapes::torus(64, 32, 1.0, 0.5);
    let field = field_of(&mesh, 128);
    let base = AoParams::for_bounds(mesh.bounds());

    // O aro interno: o lugar onde a resposta de fato depende de atravessar o furo.
    let aro: Vec<usize> = (0..mesh.vert_count())
        .filter(|&i| {
            let p = mesh.positions()[i];
            p[2].abs() < 0.12 && (p[0] * p[0] + p[1] * p[1]).sqrt() < 0.62
        })
        .collect();

    println!(
        "\n== O ORCAMENTO, medido no ARO INTERNO de um toro ({} verts) ==",
        aro.len()
    );
    println!(
        "{:>10} {:>8} {:>14} {:>12}",
        "radius", "steps", "AO do aro", "traco ms"
    );
    for radius in [0.34f32, 0.5, 1.0, 1.5, 2.0] {
        for steps in [12usize, 24, 48, 96] {
            let p = AoParams {
                radius,
                max_steps: steps,
                ..base
            };
            let t = Instant::now();
            let ao = bake_ao(&field, mesh.positions(), mesh.normals(), p);
            let el = ms(t);
            let no_aro: Vec<f32> = aro.iter().map(|&i| ao[i]).collect();
            println!(
                "{radius:>10.2} {steps:>8} {:>14.4} {el:>12.1}",
                mean(&no_aro)
            );
        }
    }
    println!(
        "\nLeitura: onde a coluna do AO PARA de mudar, o orcamento acima disso e' so custo.\n\
         O default nasce no joelho, nao no maior numero que couber."
    );
}

/// **O QUE O TRAÇO QUE SHIPA CUSTA** — a rota paralela do ADR-0156.
///
/// ⚠️ **Esta sonda mudou de pergunta, e o nome antigo teria passado a MENTIR.**
/// Ela nascia como *"vale paralelizar?"* e cronometrava `bake_ao` como a
/// baseline SERIAL. Desde o ADR-0156 o `bake_ao` **é** a rota paralela, então a
/// mesma medição reportaria ~1× de speedup contra ela mesma. A comparação
/// serial×paralelo mudou-se para junto da rota congelada que a torna possível
/// (`ao_tests::measure_the_parallel_gain`), e aqui fica o custo do que ships.
#[test]
#[ignore = "sonda"]
fn measure_the_shipped_trace() {
    println!("\n== O TRACO QUE SHIPA (paralelo, ADR-0156) ==");
    println!(
        "{:>20} {:>9} {:>12} {:>14}",
        "malha", "verts", "traco ms", "ns/vertice"
    );
    for (u, v) in [(96usize, 144usize), (320, 480), (533, 800)] {
        let mesh = shapes::uv_sphere(u, v, 1.0);
        let field = field_of(&mesh, 128);
        let params = AoParams::for_bounds(mesh.bounds());
        let t = Instant::now();
        let ao = bake_ao(&field, mesh.positions(), mesh.normals(), params);
        let el = ms(t);
        println!(
            "{:>20} {:>9} {el:>12.1} {:>14.0}",
            format!("uv_sphere({u},{v})"),
            mesh.vert_count(),
            el * 1e6 / mesh.vert_count() as f64
        );
        std::hint::black_box(ao);
    }
    println!(
        "\nLeitura: a referencia serial esta na tabela do cabecalho (807,7 ms a 425k) e\n\
         e' re-medivel por `ao_tests::measure_the_parallel_gain`, que compara contra a\n\
         rota congelada. Aqui so' vive o custo do que o artista de fato paga."
    );
}
