//! **Export e Repeat até ao disco.** Sprite NOSSO, ficheiros reais numa pasta temporária.

use std::path::PathBuf;

use ph2d_pixel_core::{ColorMode, NewSprite, Op, Patch, Rect, Sprite, frames};
use ph2d_pixel_io::exporta::{self, ErroGravar, Export, ExportAs, ExportFolha};
use ph2d_pixel_io::render::{Camadas, Pedido, Quadros};
use ph2d_pixel_io::saida::Formato;
use ph2d_pixel_io::sheet::{OpcoesFolha, OpcoesJson, Tipo};

fn pasta(nome: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("ph2d_exporta_{nome}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).expect("pasta");
    d
}

/// 8 × 8, 3 quadros, um ponto por quadro; `mode` à escolha.
fn sprite(mode: ColorMode) -> Sprite {
    let mut s = Sprite::new(NewSprite {
        width: 8,
        height: 8,
        mode,
        ..NewSprite::default()
    })
    .expect("sprite");
    let cor: &[u8] = match mode {
        ColorMode::Rgba => &[200, 30, 40, 255],
        ColorMode::Gray => &[120, 255],
        ColorMode::Indexed => &[1],
    };
    let layer = s.layers()[0].id;
    for i in 0..3_i32 {
        if i > 0 {
            let f = *s.frames().last().expect("quadro");
            let op = frames::new_frame(&mut s, f.id, frames::NewFrame::Empty)
                .expect("novo")
                .0;
            ph2d_pixel_core::apply(&mut s, op).expect("aplica");
        }
        let frame = s.frames().last().expect("quadro").id;
        let r = Rect::new(i, 0, 1, 1);
        let op = Op::Patch(Patch {
            layer,
            frame,
            bbox: Some(r),
            rect: r,
            pixels: cor.to_vec(),
        });
        ph2d_pixel_core::apply(&mut s, op).expect("aplica");
    }
    s
}

fn export_as(formato: Formato) -> Export {
    Export::Ficheiro(ExportAs {
        pedido: Pedido {
            escala: 1.0,
            ..Pedido::default()
        },
        formato,
        repeat: 0,
    })
}

/// Vermelho se um formato parar de gravar ou trocar de assinatura.
#[test]
fn cada_formato_grava_o_seu_ficheiro() {
    let d = pasta("formatos");
    let s = sprite(ColorMode::Rgba);
    for (f, nome, magia) in [
        (Formato::Gif, "a.gif", &b"GIF89a"[..]),
        (Formato::Apng, "a.apng", &b"\x89PNG"[..]),
        (Formato::Webp, "a.webp", &b"RIFF"[..]),
    ] {
        let c = d.join(nome);
        let g = exporta::gravar(&s, &export_as(f), &c).expect("grava");
        assert_eq!(g.ficheiros, vec![c.clone()]);
        let b = std::fs::read(&c).expect("lê");
        assert_eq!(&b[..magia.len()], magia, "{nome}");
    }
    let c = d.join("a.ase");
    exporta::gravar(&s, &export_as(Formato::Ase), &c).expect("grava");
    let (lido, _) = ph2d_pixel_io::ase::ler(&std::fs::read(&c).expect("lê")).expect("volta");
    assert_eq!(lido.frames().len(), 3);
}

/// Vários quadros em PNG = um ficheiro numerado por quadro, com o nome pedido como raiz.
#[test]
fn a_sequencia_de_png_numera_os_ficheiros() {
    let d = pasta("sequencia");
    let g = exporta::gravar(
        &sprite(ColorMode::Rgba),
        &export_as(Formato::Png),
        &d.join("walk.png"),
    )
    .expect("grava");
    let nomes: Vec<_> = g
        .ficheiros
        .iter()
        .map(|c| c.file_name().expect("nome").to_string_lossy().into_owned())
        .collect();
    assert_eq!(nomes, ["walk1.png", "walk2.png", "walk3.png"]);
    assert!(g.ficheiros.iter().all(|c| c.exists()));
}

/// A folha leva o `.json` ao lado, e o JSON nomeia o ficheiro da imagem. Sem JSON, só a imagem.
#[test]
fn a_folha_leva_o_json_ao_lado() {
    let d = pasta("folha");
    let s = sprite(ColorMode::Rgba);
    let c = d.join("folha.png");
    let e = Export::Folha(ExportFolha {
        opcoes: OpcoesFolha {
            tipo: Tipo::Horizontal,
            ..OpcoesFolha::default()
        },
        json: Some(OpcoesJson {
            array: true,
            ..OpcoesJson::default()
        }),
    });
    let g = exporta::gravar(&s, &e, &c).expect("grava");
    assert_eq!(g.ficheiros, vec![c.clone(), d.join("folha.json")]);
    let j = std::fs::read_to_string(d.join("folha.json")).expect("lê");
    assert!(j.contains("\"image\": \"folha.png\""), "{j}");
    let sem = Export::Folha(ExportFolha {
        opcoes: OpcoesFolha::default(),
        json: None,
    });
    let c2 = d.join("so.png");
    assert_eq!(
        exporta::gravar(&s, &sem, &c2).expect("grava").ficheiros,
        vec![c2]
    );
    assert!(!d.join("so.json").exists());
}

/// **O Repeat dá os mesmos bytes.** Todas as opções passam pelo texto guardado no sprite, e pelo
/// projecto: mudar uma opção no escritor e esquecer o leitor deixa vermelho.
#[test]
fn o_repeat_grava_os_mesmos_bytes_depois_de_passar_pelo_projeto() {
    let d = pasta("repeat");
    let mut s = sprite(ColorMode::Rgba);
    let op = frames::new_tag(&s, "walk", 1, 2).expect("tag");
    ph2d_pixel_core::apply(&mut s, op).expect("tag");
    let (op, recorte) =
        ph2d_pixel_core::slice::create(&mut s, 0, ph2d_pixel_core::Rect::new(1, 1, 2, 2))
            .expect("a slice");
    ph2d_pixel_core::apply(&mut s, op).expect("a slice");
    let casos = [
        Export::Ficheiro(ExportAs {
            pedido: Pedido {
                camadas: Camadas::Visiveis,
                quadros: Quadros::Todos,
                direcao: ph2d_pixel_io::render::Direcao::Forward,
                escala: 1.0,
                area: ph2d_pixel_io::render::Area::Slice(recorte),
            },
            formato: Formato::Png,
            repeat: 0,
        }),
        Export::Ficheiro(ExportAs {
            pedido: Pedido {
                camadas: Camadas::Todas,
                quadros: Quadros::Tag(0),
                direcao: ph2d_pixel_io::render::Direcao::PingPong,
                escala: 2.0,
                area: Default::default(),
            },
            formato: Formato::Gif,
            repeat: 3,
        }),
        Export::Folha(ExportFolha {
            opcoes: OpcoesFolha {
                tipo: Tipo::Columns,
                limite: ph2d_pixel_io::sheet::Limite::Linhas(2),
                potencia_de_2: true,
                merge_duplicates: true,
                ignore_empty: true,
                camadas: Camadas::Uma(s.layers()[0].id),
                quadros: Quadros::Intervalo { de: 0, ate: 2 },
                split_layers: false,
                split_tags: true,
                split_slices: true,
                normal: true,
                // Com split o Godot recusa (`GodotSemQuadros`): o par godot vai e volta em `saida_godot`.
                godot: false,
                split_grid: true,
                trim_by_grid: true,
                trim_cels: true,
                trim_sprite: false,
                extrude: 1,
                border: 2,
                shape: 3,
                inner: 4,
                nome: Some("{frame01}=x".to_owned()),
            },
            json: Some(OpcoesJson {
                array: true,
                tags: true,
                layers: true,
                slices: false,
            }),
        }),
    ];
    for (n, e) in casos.iter().enumerate() {
        let c = d.join(format!("r{n}.png"));
        let g = exporta::gravar(&s, e, &c).expect("grava");
        let antes: Vec<_> = g
            .ficheiros
            .iter()
            .map(|f| std::fs::read(f).expect("lê"))
            .collect();
        for f in &g.ficheiros {
            std::fs::remove_file(f).expect("apaga");
        }
        exporta::lembrar(&mut s, e, &c);
        let lib = std::collections::BTreeMap::from([(1_u64, s.clone())]);
        let bytes = ph2d_pixel_core::codec::write_library(&lib);
        let lido = ph2d_pixel_core::codec::read_library(&bytes).expect("lê")[&1].clone();
        let r = exporta::repetir(&lido).expect("tem").expect("grava");
        assert_eq!(r.ficheiros, g.ficheiros);
        let depois: Vec<_> = r
            .ficheiros
            .iter()
            .map(|f| std::fs::read(f).expect("lê"))
            .collect();
        assert_eq!(antes, depois, "caso {n}");
    }
}

/// Sem export não há o que repetir; camada apagada ou tag renomeada recusam, sem gravar outra coisa.
#[test]
fn o_repeat_recusa_o_que_sumiu() {
    let d = pasta("sumiu");
    let mut s = sprite(ColorMode::Rgba);
    assert!(exporta::repetir(&s).is_none());
    let op = frames::new_tag(&s, "walk", 0, 1).expect("tag");
    ph2d_pixel_core::apply(&mut s, op).expect("tag");
    let c = d.join("t.gif");
    let e = Export::Ficheiro(ExportAs {
        pedido: Pedido {
            quadros: Quadros::Tag(0),
            escala: 1.0,
            ..Pedido::default()
        },
        formato: Formato::Gif,
        repeat: 0,
    });
    exporta::lembrar(&mut s, &e, &c);
    let op = frames::delete_tag(&s, 0).expect("apaga");
    ph2d_pixel_core::apply(&mut s, op).expect("sem tag");
    assert_eq!(exporta::repetir(&s), Some(Err(ErroGravar::AlvoSumiu)));
    assert!(!c.exists());
}

/// Escala acima do lado máximo recusa em vez de pedir terabytes.
#[test]
fn escala_grande_demais_recusa() {
    let s = sprite(ColorMode::Rgba);
    let e = Export::Ficheiro(ExportAs {
        pedido: Pedido {
            escala: 100_000.0,
            ..Pedido::default()
        },
        formato: Formato::Png,
        repeat: 0,
    });
    let r = exporta::gravar(&s, &e, &pasta("grande").join("g.png"));
    assert!(matches!(r, Err(ErroGravar::Saida(_))), "{r:?}");
}

/// Um indexado escalado por inteiro continua com paleta no PNG.
#[test]
fn o_indexado_escalado_continua_indexado() {
    let d = pasta("indexado");
    let s = sprite(ColorMode::Indexed);
    let e = Export::Ficheiro(ExportAs {
        pedido: Pedido {
            quadros: Quadros::Intervalo { de: 0, ate: 0 },
            escala: 3.0,
            ..Pedido::default()
        },
        formato: Formato::Png,
        repeat: 0,
    });
    let c = d.join("i.png");
    exporta::gravar(&s, &e, &c).expect("grava");
    let b = std::fs::read(&c).expect("lê");
    let (w, h, modo, _) = ph2d_imageio_png::decode_pixel(&b).expect("decodifica");
    assert_eq!((w, h), (24, 24));
    assert!(
        matches!(modo, ph2d_imageio_png::PixelPng::Indexed(_)),
        "{modo:?}"
    );
}

/// M10: quanto a folha pesa. `--ignored --nocapture`, perfil smoke.
#[test]
#[ignore = "medição, não portão"]
fn diag_quanto_custa_a_folha_grande() {
    let mut s = Sprite::new(NewSprite {
        width: 1024,
        height: 1024,
        ..NewSprite::default()
    })
    .expect("sprite");
    let layer = s.layers()[0].id;
    let r = Rect::new(0, 0, 1024, 1024);
    for i in 0..256_u32 {
        if i > 0 {
            let f = *s.frames().last().expect("quadro");
            let op = frames::new_frame(&mut s, f.id, frames::NewFrame::Empty)
                .expect("novo")
                .0;
            ph2d_pixel_core::apply(&mut s, op).expect("aplica");
        }
        let frame = s.frames().last().expect("quadro").id;
        let px: Vec<u8> = (0..1024 * 1024)
            .flat_map(|p: u32| [(p ^ i) as u8, (p >> 8) as u8, i as u8, 255])
            .collect();
        let op = Op::Patch(Patch {
            layer,
            frame,
            bbox: Some(r),
            rect: r,
            pixels: px,
        });
        ph2d_pixel_core::apply(&mut s, op).expect("aplica");
    }
    let o = OpcoesFolha::default();
    for caso in ["sem nada", "trim", "merge"] {
        let mut o = o.clone();
        o.trim_cels = caso == "trim";
        o.merge_duplicates = caso == "merge";
        let mut melhor = f64::MAX;
        for _ in 0..3 {
            let t = std::time::Instant::now();
            let f = ph2d_pixel_io::sheet::montar(&s, &o, "t", "png");
            melhor = melhor.min(t.elapsed().as_secs_f64() * 1e3);
            assert!(f.w > 0);
        }
        println!("montar {caso}: {melhor:.0} ms");
    }
}

/// Um nome de tag com quebra de linha não reescreve as opções do Repeat, e a tag continua achada.
#[test]
fn nome_com_quebra_de_linha_nao_vira_outro_campo() {
    let d = pasta("quebra");
    let mut s = sprite(ColorMode::Rgba);
    let op = frames::new_tag(&s, "a\\b\nscale=0.5", 0, 1).expect("tag");
    ph2d_pixel_core::apply(&mut s, op).expect("tag");
    let e = Export::Ficheiro(ExportAs {
        pedido: Pedido {
            quadros: Quadros::Tag(0),
            escala: 2.0,
            ..Pedido::default()
        },
        formato: Formato::Gif,
        repeat: 0,
    });
    let c = d.join("t.gif");
    exporta::lembrar(&mut s, &e, &c);
    let (lido, caminho) = exporta::o_ultimo(&s).expect("tem");
    assert_eq!(caminho, c);
    assert_eq!(lido, Some(e));
}

/// Gravar não deixa o ficheiro de passagem para trás.
#[test]
fn gravar_nao_deixa_ficheiro_de_passagem() {
    let d = pasta("passagem");
    exporta::gravar(
        &sprite(ColorMode::Rgba),
        &export_as(Formato::Gif),
        &d.join("a.gif"),
    )
    .expect("grava");
    let nomes: Vec<_> = std::fs::read_dir(&d)
        .expect("lê")
        .flatten()
        .map(|e| e.file_name())
        .collect();
    assert_eq!(nomes, ["a.gif"]);
}

/// O `.json` de outra folha, com o mesmo nome, não entra.
#[test]
fn o_json_de_outra_folha_nao_entra() {
    let d = pasta("outro_json");
    let c = d.join("f.png");
    std::fs::write(d.join("f.json"), r#"{"meta":{"size":{"w":16,"h":8}}}"#).expect("grava");
    assert!(ph2d_pixel_io::abrir::dados_ao_lado(&c, (16, 8)).is_some());
    assert!(ph2d_pixel_io::abrir::dados_ao_lado(&c, (32, 8)).is_none());
    assert!(ph2d_pixel_io::abrir::dados_ao_lado(&d.join("sem.png"), (16, 8)).is_none());
}

/// O Repeat acha a tag pelo NOME: a segunda continua a segunda, e inserir outra antes dela não muda
/// o que ele exporta.
#[test]
fn o_repeat_acha_a_tag_pelo_nome() {
    let mut s = sprite(ColorMode::Rgba);
    for (nome, de, ate) in [("a", 0, 0), ("b", 1, 2)] {
        let op = frames::new_tag(&s, nome, de, ate).expect("tag");
        ph2d_pixel_core::apply(&mut s, op).expect("tag");
    }
    let quadros_de = |x: &Export| match x {
        Export::Ficheiro(f) => f.pedido.quadros,
        Export::Folha(f) => f.opcoes.quadros,
        Export::Tiles(_) => Quadros::Todos,
    };
    let e = Export::Ficheiro(ExportAs {
        pedido: Pedido {
            quadros: Quadros::Tag(1),
            escala: 1.0,
            ..Pedido::default()
        },
        formato: Formato::Gif,
        repeat: 0,
    });
    exporta::lembrar(&mut s, &e, &pasta("nome").join("t.gif"));
    let lido = exporta::o_ultimo(&s).expect("tem").0.expect("lê");
    assert_eq!(quadros_de(&lido), Quadros::Tag(1));
}
