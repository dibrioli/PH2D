//! ⭐ **A SONDA DA SAÍDA** — escreve os TRÊS ficheiros pelo caminho do produto,
//! para fora da árvore, e imprime o que eles são.
//!
//! ⚠️ **Ela fica VERSIONADA e não se apaga depois da cura:** o report do dono de
//! 22/09 (*«o Blender não consegue importar e não dá nenhuma mensagem»*) é sobre
//! um FICHEIRO, e a única forma honesta de o estudar é **produzi-lo pelas mesmas
//! chamadas que a saída faz** e depois dá-lo ao alvo **a correr** (§0.9 — o alvo
//! é um oráculo que se CORRE, e ler o que ele diz sobre um ficheiro NOSSO é
//! livre).
//!
//! Corre-se com a variável `PH2D_SONDA_SAIDA=<pasta>` e `-- --nocapture`.
//!
//! ⛔ **Sem a variável ela não escreve nada** — uma sonda que despeja ficheiros
//! por omissão suja a árvore de quem corre a suíte.

use ph2d_mesh::{ExportPiece, MeshFormat, Pose};
use ph2d_mesh_colors::Tinta;

#[test]
fn diag_a_saida_escreve_os_tres() {
    let Ok(dir) = std::env::var("PH2D_SONDA_SAIDA") else {
        return;
    };
    let dir = std::path::Path::new(&dir);
    std::fs::create_dir_all(dir).expect("a pasta da sonda");

    // A peça da cena `=52`, e o degrau que o roteiro manda escolher (`8x`).
    let mesh = crate::scenes::tinta_fina::peca();
    let mut tinta = Tinta::nova(
        mesh.vert_count(),
        mesh.faces().iter().map(ph2d_mesh::Face::verts),
        3,
    );
    // Uma marca FINA: faixas de amostras, que é o que o `8x` compra.
    for (i, a) in tinta.amostras_mut().iter_mut().enumerate() {
        *a = if (i / 7) % 2 == 0 {
            [0.85, 0.15, 0.10]
        } else {
            [0.95, 0.95, 0.92]
        };
    }

    let assado = ph2d_mesh_colors::assar(
        &tinta,
        mesh.faces().iter().map(ph2d_mesh::Face::verts),
        super::TECTO_DE_TEXELS,
    )
    .expect("a peça assa");

    let caminho = dir.join("teste.obj");
    let (mtl_nome, pngs) = super::nomes(&caminho, std::slice::from_ref(&Some(assado.clone())));
    let mats = [super::material(0)];
    let pieces = [ExportPiece {
        name: None,
        mesh: &mesh,
        pose: Pose::default(),
    }];
    let uvs = [Some(ph2d_mesh::UvDaPeca {
        uv: &assado.uv,
        off: &assado.off_uv,
        material: &mats[0],
    })];
    let obj = ph2d_mesh::write_obj_com_uv(&pieces, &uvs, &mtl_nome);
    let mtl = ph2d_mesh::write_mtl(&[(mats[0].clone(), pngs[0].clone())]);
    std::fs::write(&caminho, &obj).expect("obj");
    std::fs::write(dir.join(&mtl_nome), &mtl).expect("mtl");
    image::save_buffer(
        dir.join(&pngs[0]),
        &assado.rgb(),
        assado.lado_px,
        assado.lado_px,
        image::ColorType::Rgb8,
    )
    .expect("png");

    eprintln!("\n-- A SAIDA, COMO ELA SAI ----------------------------------");
    eprintln!(
        "peca: {} vertices - {} faces",
        mesh.vert_count(),
        mesh.faces().len()
    );
    eprintln!(
        "textura: {}x{} px - aproveitamento {:.1} %",
        assado.lado_px,
        assado.lado_px,
        assado.relatorio.aproveitamento() * 100.0
    );
    eprintln!("obj: {} bytes - mtl: {} bytes", obj.len(), mtl.len());
    for rot in ["v ", "vt ", "f "] {
        let n = obj.lines().filter(|l| l.starts_with(rot)).count();
        eprintln!("  {rot:>4}{n}");
    }
    eprintln!("\n-- as 12 primeiras linhas do .obj --");
    for l in obj.lines().take(12) {
        eprintln!("  {l}");
    }
    eprintln!("\n-- a 1.a linha `f` --");
    if let Some(l) = obj.lines().find(|l| l.starts_with("f ")) {
        eprintln!("  {l}");
    }
    eprintln!("\n-- o .mtl inteiro --");
    for l in mtl.lines() {
        eprintln!("  {l}");
    }
    eprintln!("\nescrito em {}", dir.display());
    eprintln!("-----------------------------------------------------------\n");

    // ⚠️ A sonda só descreve o produto se o produto ainda disser que o OBJ
    //    carrega a tinta fina — sem isto ela pode escrever um ficheiro que a
    //    saída de verdade já não produz.
    assert!(MeshFormat::Obj.keeps_fine_paint());
}
