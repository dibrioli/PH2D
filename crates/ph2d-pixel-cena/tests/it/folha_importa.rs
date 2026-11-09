//! P14e-2: a folha que o Pixel exporta (`png` + `json` + `_n.png`) importa-se no Sprite Inspector
//! igual ao `.ase` — campo a campo, nos 6 quadros da `heroi_p14`.

use ph2d_pixel_cena::fixture::{body, heroi_p14};
use ph2d_pixel_cena::{com_colliders, importa_ase, importa_folha};
use ph2d_pixel_io::exporta::{self, Export, ExportFolha};
use ph2d_pixel_io::sheet::{OpcoesFolha, OpcoesJson};

const M: f64 = 1.0 / 16.0;

#[test]
fn a_folha_e_o_ase_importam_igual() {
    // O que o Pixel grava com o Add Collider em `body` (o conjunto vem do mundo, D2).
    let s = heroi_p14();
    let gravado = com_colliders(&s, &std::collections::BTreeSet::from([body(&s).0]));
    let (ase, _) = ph2d_pixel_io::ase::escrever(&gravado).expect("grava o .ase");
    let d = std::env::temp_dir().join(format!("ph2d_p14e2_folha_{}", std::process::id()));
    std::fs::create_dir_all(&d).expect("pasta");
    // Com recorte, borda e margem: o lugar de cada peça não é trivial.
    let e = Export::Folha(ExportFolha {
        opcoes: OpcoesFolha {
            trim_cels: true,
            border: 2,
            shape: 1,
            inner: 1,
            normal: true,
            godot: true,
            ..OpcoesFolha::default()
        },
        json: Some(OpcoesJson {
            array: true,
            tags: true,
            slices: true,
            ..OpcoesJson::default()
        }),
    });
    exporta::gravar(&gravado, &e, &d.join("heroi.png")).expect("export");
    let ler = |n: &str| std::fs::read(d.join(n)).expect(n);
    let json = String::from_utf8(ler("heroi.json")).expect("utf-8");
    assert!(
        json.contains("\"ph2d.collider\": true"),
        "o JSON leva a marca"
    );
    let a = importa_ase(&ase, "heroi_p14", M).expect("import .ase");
    let f = importa_folha(
        &ler("heroi.png"),
        &json,
        Some(&ler("heroi_n.png")),
        "heroi_p14",
        M,
    )
    .unwrap_or_else(|e| panic!("import da folha: {e}"));
    let _ = std::fs::remove_dir_all(&d);

    assert_eq!((f.largura, f.altura), (a.largura, a.altura), "tamanho");
    assert_eq!(f.clips, a.clips, "tags");
    assert_eq!(a.clips.iter().count(), 3);
    assert_eq!(f.sem_tags, a.sem_tags);
    assert_eq!(f.cor.len(), 6);
    let (na, nf) = (
        a.normal.as_ref().expect("normal do .ase"),
        f.normal.as_ref().expect("normal da folha"),
    );
    for i in 0..6 {
        assert_eq!(f.cor[i], a.cor[i], "cor do quadro {i}");
        assert_eq!(nf[i], na[i], "normal do quadro {i}");
        assert_eq!(f.pivos_px[i], a.pivos_px[i], "pivô do quadro {i}");
    }
    assert_eq!(a.colliders.len(), 1, "um collider: body");
    let col = |v: &[ph2d_pixel_cena::ColliderImportado]| {
        v.iter()
            .map(|c| (c.nome.clone(), c.por_quadro.clone()))
            .collect::<Vec<_>>()
    };
    assert_eq!(col(&f.colliders), col(&a.colliders), "colliders por quadro");
    assert_eq!(f.celulas, a.celulas, "células (pivô e formas)");
    // A fixture contém o fenómeno: o pivô muda no 4.
    assert_ne!(a.pivos_px[3], a.pivos_px[4]);
}
