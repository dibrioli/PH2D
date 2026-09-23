//! Gates da porta de saída.
//!
//! ⚠️ **O oráculo é o ROUND-TRIP**, e ele só existe porque a wave trouxe os
//! leitores junto: escrever um arquivo e afirmar que os bytes "parecem certos" é
//! um gate que casa com a própria escrita. Aqui o arquivo é lido de volta pelo
//! caminho que o artista usa, e o que se compara é GEOMETRIA.

use super::*;
use crate::face::Face;
// ⚠️ A lei do aviso mora no `read.rs`; o gate dela vive aqui porque o sujeito é a
//    TABELA do formato, que é deste ficheiro.
use crate::lost_by;
use crate::mesh::Mesh;
use crate::ply::PlyError;
use crate::{import_obj, import_ply, import_stl, shapes};

/// Uma peça com pose deslocada e escalada — a fixture TEM de conter a pose,
/// senão o gate do mundo é verde por vácuo.
fn piece(mesh: &Mesh, at: [f32; 3], scale: f32) -> ExportPiece<'_> {
    ExportPiece {
        name: Some("P"),
        mesh,
        pose: Pose::new(at, scale),
    }
}

/// A caixa de um conjunto de pontos.
fn bounds(p: &[[f32; 3]]) -> ([f32; 3], [f32; 3]) {
    let mut lo = [f32::MAX; 3];
    let mut hi = [f32::MIN; 3];
    for v in p {
        for k in 0..3 {
            lo[k] = lo[k].min(v[k]);
            hi[k] = hi[k].max(v[k]);
        }
    }
    (lo, hi)
}

/// **A geometria sai em MUNDO** — a decisão central da wave, nos três formatos.
///
/// ⚠️ Sem isto, duas peças com poses diferentes saem EMPILHADAS na origem: o
/// defeito espelho exato do que o import curou ao centrar cada peça. A fixture
/// tem duas peças bem separadas, porque com uma só *local* e *mundo* diferem
/// apenas por uma translação que ninguém nota num arquivo.
#[test]
fn every_format_writes_the_world_the_artist_sees() {
    let a = shapes::cube(1.0);
    let b = shapes::cube(1.0);
    let pieces = [
        piece(&a, [-5.0, 0.0, 0.0], 1.0),
        piece(&b, [5.0, 0.0, 0.0], 2.0),
    ];

    for fmt in MeshFormat::ALL {
        let bytes = fmt.write(&pieces);
        let mesh = match fmt {
            MeshFormat::Obj => {
                let ps = import_obj(&String::from_utf8(bytes).expect("utf8")).expect("obj");
                // O OBJ preserva peças: junta as duas caixas para comparar.
                let mut all = Vec::new();
                for p in &ps {
                    all.extend_from_slice(p.mesh.positions());
                }
                let (lo, hi) = bounds(&all);
                assert!(
                    lo[0] < -5.0 && hi[0] > 5.0,
                    "{fmt:?}: as peças saíram empilhadas — {lo:?}..{hi:?}"
                );
                continue;
            }
            MeshFormat::Ply => import_ply(&bytes).expect("ply"),
            MeshFormat::Stl => import_stl(&bytes).expect("stl"),
        };
        let (lo, hi) = bounds(mesh.positions());
        assert!(
            lo[0] < -5.0 && hi[0] > 5.0,
            "{fmt:?}: as peças saíram empilhadas — {lo:?}..{hi:?}"
        );
        // E a ESCALA da pose viajou: a peça da direita mede o dobro.
        assert!(
            hi[0] - lo[0] > 11.0,
            "{fmt:?}: a escala da pose não chegou ao arquivo ({})",
            hi[0] - lo[0]
        );
    }
}

/// **A COR sobrevive onde o formato a tem** — e o gate cobre as duas metades,
/// porque a de AUSÊNCIA é a que o aviso ao artista promete.
#[test]
fn colour_survives_exactly_where_the_format_keeps_it() {
    let mut m = shapes::cube(1.0);
    for (i, c) in m.colors_mut().iter_mut().enumerate() {
        *c = if i % 2 == 0 {
            [1.0, 0.0, 0.0]
        } else {
            [0.0, 0.0, 1.0]
        };
    }
    let pieces = [piece(&m, [0.0; 3], 1.0)];

    let obj = import_obj(&write_obj(&pieces)).expect("obj");
    assert!(
        obj[0].mesh.colors().is_some(),
        "o OBJ declara que preserva cor e a perdeu"
    );
    let ply = import_ply(&write_ply(&pieces)).expect("ply");
    let c = ply
        .colors()
        .expect("o PLY declara que preserva cor e a perdeu");
    assert!(
        c.iter().any(|v| v[0] > 0.9 && v[2] < 0.1) && c.iter().any(|v| v[2] > 0.9 && v[0] < 0.1),
        "as DUAS cores têm de atravessar, e vieram {c:?}"
    );

    // A ausência: o STL não tem onde pôr cor, e o `keeps_colour` diz isso.
    let stl = import_stl(&write_stl(&pieces)).expect("stl");
    assert!(
        stl.colors().is_none(),
        "um STL não pode trazer cor de volta"
    );
    assert!(
        !MeshFormat::Stl.keeps_colour(),
        "e a tabela tem de concordar"
    );
    assert!(MeshFormat::Obj.keeps_colour() && MeshFormat::Ply.keeps_colour());
}

/// **As PEÇAS sobrevivem só no OBJ**, e a tabela é quem responde.
///
/// ⚠️ É este par que impede o toast de mentir: se `keeps_pieces` divergir do que
/// o escritor faz, o artista exporta três peças em PLY e o app diz que estão
/// separadas.
#[test]
fn pieces_survive_exactly_where_the_table_says() {
    let a = shapes::cube(1.0);
    let b = shapes::octahedron(1.0);
    let pieces = [
        piece(&a, [-3.0, 0.0, 0.0], 1.0),
        piece(&b, [3.0, 0.0, 0.0], 1.0),
    ];

    let obj = import_obj(&write_obj(&pieces)).expect("obj");
    assert_eq!(obj.len(), 2, "o OBJ preserva peças");
    assert!(MeshFormat::Obj.keeps_pieces());

    // PLY e STL fundem — e a tabela diz.
    assert!(!MeshFormat::Ply.keeps_pieces() && !MeshFormat::Stl.keeps_pieces());
    let ply = import_ply(&write_ply(&pieces)).expect("ply");
    assert_eq!(
        ply.positions().len(),
        a.positions().len() + b.positions().len(),
        "o PLY funde tudo num corpo só"
    );
}

/// **Os QUADS sobrevivem em OBJ e PLY**, e o STL os triangula porque o formato
/// não tem outra forma.
#[test]
fn quads_survive_in_the_indexed_formats_and_the_stl_triangulates() {
    let m = shapes::cube(1.0); // o cubo do módulo é feito de quads
    assert!(
        m.faces().iter().any(|f| !f.is_tri()),
        "a fixture precisa CONTER quads, senão o gate é vácuo"
    );
    let pieces = [piece(&m, [0.0; 3], 1.0)];

    let obj = import_obj(&write_obj(&pieces)).expect("obj");
    assert!(
        obj[0].mesh.faces().iter().any(|f| !f.is_tri()),
        "OBJ perdeu os quads"
    );
    let ply = import_ply(&write_ply(&pieces)).expect("ply");
    assert!(
        ply.faces().iter().any(|f| !f.is_tri()),
        "PLY perdeu os quads"
    );

    let stl = import_stl(&write_stl(&pieces)).expect("stl");
    assert!(
        stl.faces().iter().all(|f| f.is_tri()),
        "um STL só sabe falar em triângulos"
    );
    assert_eq!(
        stl.faces().len(),
        triangle_count(&pieces),
        "e a contagem tem de bater com a que o cabeçalho escreveu"
    );
}

/// **A extensão decide o formato** — e nada mais decide.
#[test]
fn the_extension_names_the_format_in_both_directions() {
    for f in MeshFormat::ALL {
        assert_eq!(
            MeshFormat::from_extension(f.extension()),
            Some(f),
            "{f:?} não sobrevive ao par extensão↔formato"
        );
        assert_eq!(
            MeshFormat::from_extension(&f.extension().to_uppercase()),
            Some(f),
            "a caixa da extensão não pode decidir nada"
        );
    }
    assert_eq!(MeshFormat::from_extension("png"), None);
    assert_eq!(MeshFormat::from_extension(""), None);
}

/// **Um triângulo degenerado escreve normal ZERO, nunca `NaN`.**
///
/// ⚠️ É o que a spec do STL prescreve (o leitor deriva a normal pela regra da
/// mão direita), e normalizar um vetor nulo daria `NaN` — que atravessa o
/// arquivo e reaparece como geometria ausente três programas adiante.
#[test]
fn a_degenerate_triangle_writes_a_zero_normal_not_a_nan() {
    let m = Mesh::from_parts(
        vec![[0.0; 3], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]],
        vec![Face::tri(0, 1, 2)],
    )
    .expect("colinear ainda é malha");
    let bytes = write_stl(&[piece(&m, [0.0; 3], 1.0)]);
    let n: Vec<f32> = (0..3)
        .map(|k| {
            let at = 84 + k * 4;
            f32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
        })
        .collect();
    assert!(
        n.iter().all(|v| v.is_finite()),
        "normal não-finita no arquivo: {n:?}"
    );
    assert_eq!(n, vec![0.0, 0.0, 0.0]);
}

/// **Uma cena VAZIA escreve um arquivo VÁLIDO e vazio** — não um arquivo
/// corrompido nem um pânico.
///
/// ⚠️ O caso existe: exportar logo depois de apagar a última peça. Um cabeçalho
/// PLY com contagens que não batem com o corpo é recusado por todo leitor, longe
/// da causa.
#[test]
fn an_empty_scene_writes_a_valid_empty_file() {
    let stl = write_stl(&[]);
    assert_eq!(stl.len(), 84, "STL vazio é só cabeçalho + contagem");
    assert_eq!(u32::from_le_bytes([stl[80], stl[81], stl[82], stl[83]]), 0);

    let ply = write_ply(&[]);
    let head = String::from_utf8_lossy(&ply);
    assert!(head.contains("element vertex 0") && head.contains("element face 0"));
    // E ele volta como recusa NOMEADA, não como malha fantasma.
    assert!(matches!(import_ply(&ply), Err(PlyError::NoPositions)));
}

/// ⭐⭐⭐⭐ **GATE — o aviso diz que a TINTA FINA não viaja, e só quando ela existe.**
///
/// ⛔⛔ **O defeito que ele fecha é uma PERDA SILENCIOSA:** os três formatos
/// guardam cor **por vértice**, e o que o escritor recebe de uma peça pintada a
/// `8x` é a PROJECÇÃO do plano nos vértices — *a tinta de volta à resolução da
/// malha*. Até aqui o app listava o que se perde (a máscara, a cor num STL) e
/// **não dizia isto**: o artista exportava, abria noutro programa e via a marca
/// grossa, sem uma palavra.
///
/// ⚠️⚠️ **As DUAS metades, e nenhuma basta:**
/// - sem a do FORMATO, o aviso nunca podia deixar de soar no dia em que a
///   exportação levar uma imagem ao lado da malha;
/// - sem a da CENA, ele soaria **sempre** — e um aviso que soa sempre é ruído
///   que o artista aprende a ignorar, exactamente quando ele passar a ser
///   verdade.
///
/// ⭐ **E o CONTROLO é a metade que protege o caso comum:** sem tinta fina a
/// frase tem de ser **exactamente** a de antes desta wave, byte a byte. *Uma
/// cláusula nova que mude o aviso de toda a gente não é uma cláusula, é uma
/// regressão.*
#[test]
fn the_warning_names_fine_paint_only_when_the_scene_carries_some() {
    // ⛔⛔ **A PREMISSA DESTE GATE MORREU, e ele previu-a por escrito.** A
    // redacção de 22/09 acabava em `assert!(!fmt.keeps_fine_paint())` com a
    // frase *«se isso é verdade, a metade de cima deste gate deixou de
    // descrever o produto»* — e passou a ser verdade no dia em que a saída
    // aprendeu a ASSAR a retícula numa textura. ⭐ *Um gate que nomeia a
    // condição em que deixa de valer é o que torna a morte dele legível num
    // diff, em vez de uma barra afrouxada em silêncio.*
    //
    // ⚠️ **A população parte-se pela TABELA e nunca por uma lista à mão:** quem
    // carrega a tinta fina não avisa, quem não carrega avisa sempre. Uma lista
    // escrita aqui divergiria no dia do quarto formato, que é exactamente o
    // que o `lost_by` existe para impedir.
    let mut carregam = 0;
    let mut perdem = 0;
    for fmt in MeshFormat::ALL {
        let sem = lost_by(fmt, false);
        let com = lost_by(fmt, true);

        // O CONTROLO: o caso comum não se mexeu.
        let de_antes = {
            let mut lost = vec!["mask"];
            if !fmt.keeps_colour() {
                lost.push("colour");
            }
            if !fmt.keeps_pieces() {
                lost.push("pieces merged");
            }
            format!("Lost: {}", lost.join(", "))
        };
        assert_eq!(
            sem, de_antes,
            "sem tinta fina o aviso do {fmt:?} tem de ser o de sempre, byte a byte"
        );
        assert!(
            !sem.contains("fine paint"),
            "o {fmt:?} avisou de tinta fina numa cena que não tem nenhuma: o aviso \
             passa a soar sempre e vira ruído"
        );

        if fmt.keeps_fine_paint() {
            carregam += 1;
            assert_eq!(
                com, de_antes,
                "o {fmt:?} CARREGA a tinta fina (ela sai numa textura ao lado) e \
                 mesmo assim avisa que ela se perde: um aviso falso é pior que \
                 nenhum, porque o artista confia nele"
            );
        } else {
            perdem += 1;
            assert!(
                com.contains("fine paint"),
                "o {fmt:?} NÃO avisa que a tinta fina fica para trás, e ele não a \
                 carrega: é a perda silenciosa que este gate existe para fechar"
            );
        }
    }
    // ⚠️ **As duas metades da população têm de EXISTIR**, senão uma das pernas
    // deste gate fica trivialmente verdadeira e ele afirma metade do que diz.
    assert!(carregam >= 1, "nenhum formato carrega a tinta fina");
    assert!(
        perdem >= 1,
        "nenhum formato a perde: o aviso ficou sem sujeito"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
//  O OBJ COM COORDENADAS DE TEXTURA — ver [`write_obj_com_uv`].
// ─────────────────────────────────────────────────────────────────────────────

/// UVs de brincar para uma peça: um par por canto, distintos.
fn uv_falso(mesh: &Mesh) -> (Vec<[f32; 2]>, Vec<u32>) {
    let mut uv = Vec::new();
    let mut off = vec![0u32];
    for f in mesh.faces() {
        for (c, _) in f.verts().iter().enumerate() {
            let n = uv.len() as f32;
            uv.push([n / 1000.0, (c as f32 + 1.0) / 8.0]);
        }
        off.push(uv.len() as u32);
    }
    (uv, off)
}

/// ⭐⭐⭐ **O CONTROLO que vale mais que o resto: sem textura o arquivo é o de
/// SEMPRE, byte a byte.**
///
/// ⛔ O [`write_obj`] delega no irmão, logo toda a família de gates que já
/// media o OBJ passa a medir o caminho novo — *e se a delegação mudasse um
/// único byte, ela estaria a medir outro formato sem ninguém notar*.
#[test]
fn sem_textura_o_obj_e_byte_a_byte_o_de_sempre() {
    let a = shapes::cube(1.0);
    let b = shapes::uv_sphere(8, 12, 0.5);
    let ps = [
        piece(&a, [0.0, 0.0, 0.0], 1.0),
        piece(&b, [2.0, 0.0, 0.0], 2.0),
    ];

    let de_sempre = write_obj(&ps);
    assert_eq!(
        write_obj_com_uv(&ps, &[], ""),
        de_sempre,
        "a lista de uvs VAZIA tem de dar o arquivo de sempre"
    );
    assert_eq!(
        write_obj_com_uv(&ps, &[None, None], ""),
        de_sempre,
        "duas peças SEM textura têm de dar o arquivo de sempre"
    );
    // O CONTROLO do próprio controlo: a fixtura tem de ter conteúdo.
    assert!(de_sempre.len() > 500, "fixtura vazia: {}", de_sempre.len());
    assert!(!de_sempre.contains("vt "), "não devia haver `vt` aqui");
}

/// ⭐⭐ **Com textura, cada canto de cada face aponta para o `vt` DELE.**
///
/// ⚠️ **E os dois acumuladores são INDEPENDENTES:** a 1.ª peça não tem textura
/// e a 2.ª tem, o que é exactamente o arranjo em que um contador partilhado
/// desloca a tinta — *somar os vértices da peça sem textura ao índice de `vt`
/// da seguinte dá um arquivo que abre, com a tinta no sítio errado*.
#[test]
fn cada_canto_aponta_para_o_vt_dele() {
    let a = shapes::cube(1.0);
    let b = shapes::uv_sphere(6, 8, 0.5);
    let ps = [
        piece(&a, [0.0, 0.0, 0.0], 1.0),
        piece(&b, [2.0, 0.0, 0.0], 1.0),
    ];
    let (uv, off) = uv_falso(&b);
    let uvs = [
        None,
        Some(UvDaPeca {
            uv: &uv,
            off: &off,
            material: "ph2d_1",
        }),
    ];

    let obj = write_obj_com_uv(&ps, &uvs, "peca.mtl");
    assert!(obj.starts_with("# PH2D Sculpt\nmtllib peca.mtl\n"));
    assert_eq!(obj.matches("usemtl ph2d_1\n").count(), 1);

    let vts: Vec<&str> = obj.lines().filter(|l| l.starts_with("vt ")).collect();
    assert_eq!(vts.len(), uv.len(), "um `vt` por canto da peça com textura");

    // ⚠️ A 1.ª peça NÃO pode ter `vt` nos cantos dela, e a 2.ª tem de os ter
    //    TODOS — é a metade que um acumulador partilhado passaria na mesma.
    let (mut sem, mut com) = (0usize, 0usize);
    let mut vistos = std::collections::BTreeSet::new();
    let mut base_v = 0usize;
    for l in obj.lines().filter(|l| l.starts_with("f ")) {
        for t in l.split_whitespace().skip(1) {
            match t.split_once('/') {
                None => sem += 1,
                Some((v, vt)) => {
                    com += 1;
                    let vt: usize = vt.parse().expect("índice de vt");
                    assert!(
                        (1..=vts.len()).contains(&vt),
                        "índice de `vt` fora de alcance: {vt} de {}",
                        vts.len()
                    );
                    // O vértice tem de ser da SEGUNDA peça.
                    let v: usize = v.parse().expect("índice de v");
                    base_v = base_v.max(v);
                    vistos.insert(vt);
                }
            }
        }
    }
    // ⚠️ Os cantos CONTAM-SE (um cubo é de quads e uma esfera de triângulos):
    //    `faces * 3` mediria outra malha e reprovaria sobre produto correcto.
    let cantos_a: usize = a.faces().iter().map(|f| f.verts().len()).sum();
    assert_eq!(sem, cantos_a, "a peça sem textura ficou com `vt`");
    assert_eq!(
        com,
        uv.len(),
        "a peça com textura não cobriu os cantos dela"
    );
    assert_eq!(
        vistos.len(),
        vts.len(),
        "algum `vt` escrito nunca é referido: o acumulador deslocou-se"
    );
    assert!(
        base_v > a.positions().len(),
        "os índices de `v` não acumularam"
    );
}

/// ⚠️ **`Kd` BRANCO e `map_Kd` SEM PASTA** — as duas metades que um visualizador
/// obriga, e as duas silenciosas quando erradas (a tinta escurece; o material
/// não resolve no computador de quem abrir).
#[test]
fn o_material_nao_escurece_a_tinta_nem_carrega_um_caminho() {
    let mtl = write_mtl(&[
        ("ph2d_0".into(), "sculpt_0.png".into()),
        ("ph2d_1".into(), "sculpt_1.png".into()),
    ]);
    assert_eq!(mtl.matches("newmtl ").count(), 2);
    assert_eq!(mtl.matches("Kd 1 1 1\n").count(), 2);
    for l in mtl.lines().filter(|l| l.starts_with("map_Kd")) {
        assert!(!l.contains('/'), "o material leva um CAMINHO: {l}");
    }
}
