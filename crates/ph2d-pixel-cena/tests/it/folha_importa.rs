//! P14e-2: a folha que o Pixel exporta (`png` + `json` + `_n.png`) importa-se no Sprite Inspector
//! igual ao `.ase` — campo a campo, nos 6 quadros da `heroi_p14`.

use ph2d_pixel_cena::fixture::{body, heroi_p14};
use ph2d_pixel_cena::{SpriteImportado, com_colliders, importa_ase, importa_folha};
use ph2d_pixel_io::abrir::ErroAbrir;
use ph2d_pixel_io::exporta::{self, Export, ExportFolha};
use ph2d_pixel_io::sheet::{OpcoesFolha, OpcoesJson};

const M: f64 = 1.0 / 16.0;

/// O que o Pixel grava com o Add Collider em `body` (o conjunto vem do mundo, D2).
fn gravado() -> ph2d_pixel_core::Sprite {
    let s = heroi_p14();
    com_colliders(&s, &std::collections::BTreeSet::from([body(&s).0]))
}

/// Exporta a folha com `opcoes` e `array`, e importa-a.
fn pela_folha(
    nome: &str,
    opcoes: OpcoesFolha,
    array: bool,
) -> Result<(SpriteImportado, String), ErroAbrir> {
    let d = std::env::temp_dir().join(format!("ph2d_p14e2_{nome}_{}", std::process::id()));
    std::fs::create_dir_all(&d).expect("pasta");
    let e = Export::Folha(ExportFolha {
        opcoes,
        json: Some(OpcoesJson {
            array,
            tags: true,
            slices: true,
            ..OpcoesJson::default()
        }),
    });
    exporta::gravar(&gravado(), &e, &d.join("heroi.png")).expect("export");
    let ler = |n: &str| std::fs::read(d.join(n)).expect(n);
    let json = String::from_utf8(ler("heroi.json")).expect("utf-8");
    let normal = d.join("heroi_n.png").exists().then(|| ler("heroi_n.png"));
    let r = importa_folha(&ler("heroi.png"), &json, normal.as_deref(), "heroi_p14", M);
    let _ = std::fs::remove_dir_all(&d);
    r.map(|f| (f, json))
}

/// Com recorte, borda e margem: o lugar de cada peça não é trivial.
fn recortada(godot: bool) -> OpcoesFolha {
    OpcoesFolha {
        trim_cels: true,
        border: 2,
        shape: 1,
        inner: 1,
        normal: true,
        godot,
        ..OpcoesFolha::default()
    }
}

fn igual(f: &SpriteImportado) {
    let (ase, _) = ph2d_pixel_io::ase::escrever(&gravado()).expect("grava o .ase");
    let a = importa_ase(&ase, "heroi_p14", M).expect("import .ase");
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

#[test]
fn a_folha_e_o_ase_importam_igual() {
    let (f, json) = pela_folha("folha", recortada(true), true).expect("import da folha");
    assert!(
        json.contains("\"ph2d.collider\": true"),
        "o JSON leva a marca"
    );
    igual(&f);
}

/// (a) O pivô por quadro vai no JSON sempre que o sprite tem pivô escolhido — sem Godot também.
#[test]
fn sem_godot_o_pivo_de_cada_quadro_volta() {
    let (f, json) = pela_folha("sem_godot", recortada(false), true).expect("import");
    assert!(json.contains("\"pivots\""), "meta.pivots sem Godot");
    igual(&f);
}

/// (b) JSON `hash`: os quadros na ordem do ficheiro. O nome `{tag}-{frame}` ordena por NOME de
/// outra maneira (`hit-5` antes de `idle-0`).
#[test]
fn o_hash_le_os_quadros_na_ordem_do_ficheiro() {
    let o = OpcoesFolha {
        nome: Some("{tag}-{frame}.{extension}".to_owned()),
        ..recortada(false)
    };
    let (f, json) = pela_folha("hash", o, false).expect("import");
    assert!(json.contains("\"hit-5.png\""), "o nome do teste");
    igual(&f);
}

/// Folha exportada com Split: recusada pelo motivo próprio (a frase é conferida no import da shell).
#[test]
fn a_folha_com_split_e_recusada() {
    for (nome, o) in [
        (
            "split_layers",
            OpcoesFolha {
                split_layers: true,
                ..OpcoesFolha::default()
            },
        ),
        (
            "split_tags",
            OpcoesFolha {
                split_tags: true,
                ..OpcoesFolha::default()
            },
        ),
    ] {
        let e = pela_folha(nome, o, true).err();
        assert_eq!(e, Some(ErroAbrir::FolhaPartida), "{nome}");
    }
}
