//! A biblioteca sobre o disco: duas raízes, o do utilizador sobrepõe, o «Reset» apaga-o; o estado do
//! artista (favoritos, o activo, política).

use ph2d_brush_presets::{
    Biblioteca, EstadoDoUtilizador, Miniatura, Origem, Parametros, Politica, Preset, Volumes,
    apagar, caminho_relativo, gravar,
};
use std::path::PathBuf;

fn pasta(nome: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("ph2d_brush_presets_{}_{nome}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn preset(nome: &str, origem: Origem) -> Preset {
    Preset {
        parametros: Parametros::novo("WetPaint", "Fluid", nome),
        origem,
        miniatura: Miniatura {
            largura: 2,
            altura: 2,
            rgba: vec![255; 16],
        },
        texturas: Default::default(),
    }
}

#[test]
fn o_do_utilizador_sobrepoe_e_o_reset_volta_a_fabrica() {
    let fab = pasta("fab");
    let uti = pasta("uti");
    let a = caminho_relativo("WetPainter", "Watercolor", "Fluid", "Wet Round");
    let b = caminho_relativo("WetPainter", "Watercolor", "Fluid", "Wet Wash");
    gravar(&fab, &a, &preset("Wet Round", Origem::Fabrica)).unwrap();
    gravar(&fab, &b, &preset("Wet Wash", Origem::Fabrica)).unwrap();
    let mut meu = preset("Wet Round", Origem::Utilizador);
    meu.parametros.volumes.size_px = Some(12.0);
    gravar(&uti, &a, &meu).unwrap();
    std::fs::write(
        uti.join("WetPainter/Watercolor/Fluid/partido.ph2d-brush.png"),
        b"x",
    )
    .unwrap();

    let bib = Biblioteca::carregar(&fab, Some(&uti));
    let rels: Vec<&str> = bib.entradas.iter().map(|e| e.relativo.as_str()).collect();
    assert_eq!(
        rels,
        [a.as_str(), b.as_str()],
        "ordenados; o partido fica de fora"
    );
    let ea = bib.procurar(&a).unwrap();
    assert_eq!(ea.origem, Origem::Utilizador);
    assert!(ea.tem_fabrica);
    assert_eq!(ea.preset.parametros.volumes.size_px, Some(12.0));
    assert_eq!(
        ea.partes(),
        ["WetPainter", "Watercolor", "Fluid", "Wet Round"]
    );
    assert_eq!(bib.erros.len(), 1, "o ilegível é dito, não engolido");

    apagar(&uti, &a).unwrap();
    apagar(&uti, &a).unwrap(); // ausente não é erro
    let bib = Biblioteca::carregar(&fab, Some(&uti));
    assert_eq!(bib.procurar(&a).unwrap().origem, Origem::Fabrica);
    assert_eq!(
        bib.procurar(&a).unwrap().preset.parametros.volumes,
        Volumes::default()
    );
}

#[test]
fn a_serie_de_lapis_vem_da_mais_dura_a_mais_macia() {
    let fab = pasta("graus");
    let nomes = [
        "Graphite 2B",
        "Graphite 2H",
        "Graphite 8B",
        "Graphite HB",
        "Graphite F",
        "Grain",
    ];
    for n in nomes {
        let rel = caminho_relativo("WetPainter", "Pencil", "Graphite", n);
        gravar(&fab, &rel, &preset(n, Origem::Fabrica)).unwrap();
    }
    let bib = Biblioteca::carregar(&fab, None);
    let ordem: Vec<&str> = bib.entradas.iter().map(|e| e.partes()[3]).collect();
    assert_eq!(
        ordem,
        [
            "Grain",
            "Graphite 2H",
            "Graphite F",
            "Graphite HB",
            "Graphite 2B",
            "Graphite 8B"
        ],
        "o grau ordena a série; o resto vai pelo nome"
    );
}

#[test]
fn guardar_como_novo_nunca_esmaga_um_nome() {
    let fab = pasta("novo");
    for n in ["Wet Round", "Wet Round 2"] {
        let rel = caminho_relativo("WetPainter", "Watercolor", "Fluid", n);
        gravar(&fab, &rel, &preset(n, Origem::Fabrica)).unwrap();
    }
    let bib = Biblioteca::carregar(&fab, None);
    assert_eq!(
        bib.nome_livre("WetPainter", "Watercolor", "Fluid", "Wet Round"),
        "Wet Round 3"
    );
    assert_eq!(
        bib.nome_livre("WetPainter", "Watercolor", "Fluid", "Outro"),
        "Outro"
    );
}

#[test]
fn o_ativo_os_favoritos_e_a_politica() {
    let mut e = EstadoDoUtilizador::default();
    e.usar("p1");
    e.usar("p4");
    assert_eq!(e.ativo.as_deref(), Some("p4"));

    e.alternar_favorito("p1");
    assert!(e.favoritos.contains("p1"));
    e.alternar_favorito("p1");
    assert!(e.favoritos.is_empty());

    let v = Volumes {
        size_px: Some(22.0),
        ..Default::default()
    };
    assert_eq!(e.politica, Politica::VolumesAndMode);
    e.lembrar("p1", v.clone(), Some("Smear".to_owned()));
    assert_eq!(e.lembrados["p1"].paint_mode.as_deref(), Some("Smear"));
    e.politica = Politica::Volumes;
    e.lembrar("p1", v.clone(), Some("Smear".to_owned()));
    assert_eq!(e.lembrados["p1"].paint_mode, None);
    assert_eq!(e.lembrados["p1"].volumes, v);
    e.politica = Politica::None;
    e.lembrar("p1", v, None);
    assert!(e.lembrados.is_empty());

    let raiz = pasta("estado");
    e.alternar_favorito("p2");
    e.gravar(&raiz).unwrap();
    assert_eq!(EstadoDoUtilizador::ler(&raiz).unwrap(), e);
    assert_eq!(
        EstadoDoUtilizador::ler(&pasta("vazio")).unwrap(),
        EstadoDoUtilizador::default()
    );
}
