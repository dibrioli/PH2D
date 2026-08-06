//! **O QUE CUSTA O CAMPO QUE UM BAKE DE AO PRECISA** — a medição que decide o
//! desenho da wave, feita ANTES de uma linha dela ser escrita.
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
//! Rodar: `cargo test -p ph2d-sdf --release --test measure_ao -- --ignored --nocapture`

use ph2d_mesh::shapes;
use ph2d_sdf::VoxelField;
use std::time::Instant;

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
