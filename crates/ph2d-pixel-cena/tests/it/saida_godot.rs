//! P14d — a saída: Normal map sheet (`_n.png`) e Godot SpriteFrames (`.tres`), e os oráculos do
//! Godot 4.7.2 (MIT) gravados em `tests/fixtures/oracle_godot_p14d.json` por
//! `docs/PixelArt/ferramentas/oraculo_godot_p14d.py` (os gates leem a fixture; correm sem Godot).

use ph2d_pixel_cena::fixture::heroi_p14;
use ph2d_pixel_core::{Sprite, compose_normal};
use ph2d_pixel_io::exporta::{self, Export, ExportFolha};
use ph2d_pixel_io::sheet::{OpcoesFolha, OpcoesJson, montar};
use serde_json::Value;

/// As opções do export do oráculo: com recorte, borda e margem — o lugar de cada peça não é trivial.
fn opcoes() -> OpcoesFolha {
    OpcoesFolha {
        trim_cels: true,
        border: 2,
        shape: 1,
        inner: 1,
        normal: true,
        godot: true,
        ..OpcoesFolha::default()
    }
}

fn oraculo() -> Value {
    serde_json::from_str(include_str!("../fixtures/oracle_godot_p14d.json")).expect("fixture")
}

/// **A `_n.png` tem o recorte da cor**: peça a peça, o mesmo lugar e o mesmo alfa da folha de cor, e
/// o RGB é o `compose_normal` do quadro no mesmo canto.
#[test]
fn a_folha_normal_tem_o_recorte_da_cor() {
    let s = heroi_p14();
    let f = montar(&s, &opcoes(), "heroi", "png");
    let n = f.normal.as_ref().expect("a folha normal");
    assert_eq!(n.len(), f.rgba.len());
    assert_eq!(f.entradas.len(), s.frames().len());
    let mut cheios = 0;
    for (q, e) in f.entradas.iter().enumerate() {
        let quer = compose_normal(&s, s.frames()[q].id);
        let pad = (e.frame[2] - e.origem[2]) / 2;
        for dy in 0..e.origem[3] {
            for dx in 0..e.origem[2] {
                let i = (((e.frame[1] + pad + dy) * f.w + e.frame[0] + pad + dx) * 4) as usize;
                let o = (((e.origem[1] + dy) * s.width() + e.origem[0] + dx) * 4) as usize;
                assert_eq!(
                    n[i + 3],
                    f.rgba[i + 3],
                    "quadro {q} ({dx},{dy}): o alfa da cor"
                );
                if f.rgba[i + 3] > 0 {
                    assert_eq!(&n[i..i + 4], &quer[o..o + 4], "quadro {q} ({dx},{dy})");
                    cheios += 1;
                }
            }
        }
    }
    assert!(cheios > 6 * 100, "o disco em todos os quadros: {cheios}");
    let sem = montar(
        &s,
        &OpcoesFolha {
            normal: false,
            ..opcoes()
        },
        "heroi",
        "png",
    );
    assert!(sem.normal.is_none(), "sem pedido, sem `_n.png`");
}

/// A sequência que o NOSSO `.tres` desenrola, tag a tag: o ping-pong do Aseprite sem repetir as
/// pontas, as durações como múltiplos exatos de `1/speed`.
#[test]
fn o_tres_desenrola_as_tags_como_o_aseprite() {
    let s = heroi_p14();
    let a = ph2d_pixel_io::godot::animacoes(&s, "default");
    let v: Vec<_> = a
        .iter()
        .map(|a| {
            (
                a.nome.as_str(),
                a.quadros.clone(),
                a.laco,
                a.fps,
                a.duracoes.clone(),
            )
        })
        .collect();
    assert_eq!(
        v,
        vec![
            ("idle", vec![0, 1], true, 12.5, vec![1, 1]),
            (
                "run",
                vec![2, 3, 4, 3, 2],
                false,
                1000.0 / 60.0,
                vec![2, 2, 1, 2, 2]
            ),
            ("hit", vec![5], true, 5.0, vec![1]),
        ]
    );
}

/// **A sequência é a do Godot**: o `AnimatedSprite2D` com o NOSSO `.tres`, a passos de 1 ms, mostra
/// a mesma ORDEM de quadros com as mesmas durações que o `SpriteAnimator` do jogo. Medido (fixture,
/// `diferenca`): o Godot troca 1 passo DEPOIS da fronteira, também em aritmética exata (`exato`:
/// 64 passos de 1/64 ⇒ troca no 65.º); o produto segue o Aseprite (troca NA fronteira), logo o
/// Godot em `t` é o nosso em `t − 1`.
#[test]
fn a_sequencia_e_a_do_godot() {
    let o = oraculo();
    let x = o["exato"]["trocas"].as_array().expect("exato");
    assert_eq!(
        x.first().and_then(Value::as_u64),
        Some(65),
        "a convenção medida do Godot"
    );
    let s = heroi_p14();
    let mut diverge = Vec::new();
    for tag in ["idle", "run", "hit"] {
        let godot: Vec<usize> = o["sequencia"][tag]
            .as_array()
            .unwrap_or_else(|| panic!("{tag} na fixture"))
            .iter()
            .map(|v| v.as_u64().expect("quadro") as usize)
            .collect();
        assert_eq!(godot.len(), 2001, "{tag}: t = 0..=2000");
        let nosso = super::derivado::sequencia_do_jogo(&s, tag);
        assert_eq!(godot[0], nosso[0], "{tag}: o primeiro quadro");
        for t in 1..godot.len() {
            let (g, n) = (&godot[t], &nosso[t - 1]);
            if g != n && diverge.len() < 12 {
                diverge.push((tag, t, *g, *n));
            }
        }
    }
    assert!(diverge.is_empty(), "(tag, t ms, Godot, nosso): {diverge:?}");
}

/// **O `.tres` abre no Godot**: carregou sem erro e lista as 3 animações da `heroi_p14`; o que ele
/// leu é o texto que o nosso export grava HOJE (o sha casa).
#[test]
fn o_tres_abre_no_godot() {
    let o = oraculo();
    let g = &o["abre"];
    assert_eq!(g["erro"], Value::Null, "{g}");
    let mut nomes: Vec<&str> = g["animacoes"]
        .as_array()
        .expect("animacoes")
        .iter()
        .map(|v| v.as_str().expect("nome"))
        .collect();
    nomes.sort_unstable();
    assert_eq!(nomes, ["hit", "idle", "run"], "o Godot lista por nome");
    assert_eq!(g["normal_texture"], Value::Bool(true), "{g}");
    let d = std::env::temp_dir().join(format!("ph2d_p14d_tres_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    let tres = grava(&heroi_p14(), &d);
    use sha2::Digest;
    let sha: String = sha2::Sha256::digest(&tres)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    let _ = std::fs::remove_dir_all(&d);
    assert_eq!(
        o["entrada"]["tres_sha"].as_str(),
        Some(sha.as_str()),
        "regrave o oráculo"
    );
}

/// **O Godot recorta a `_n.png` igual**: cada AtlasTexture do `.tres`, lida da normal_texture, bate
/// píxel a píxel com o nosso `compose_normal` do quadro (onde a cor tem alfa).
#[test]
fn o_godot_recorta_a_normal_igual() {
    let o = oraculo();
    let r = o["recorte_normal"].as_array().expect("recorte");
    assert_eq!(r.len(), 6, "um por quadro");
    for (q, v) in r.iter().enumerate() {
        assert_eq!(v["diferem"].as_u64(), Some(0), "quadro {q}: {v}");
        assert!(
            v["comparados"].as_u64().unwrap_or(0) > 100,
            "quadro {q}: {v}"
        );
    }
}

/// **O sinal da normal é o do Godot**: hemisfério + `PointLight2D` à esquerda ⇒ no Godot o lado
/// esquerdo é o claro (sem normal, os dois lados iguais: o controlo). No nosso, a codificação da
/// roda diz o mesmo: no lado esquerdo do disco `n.x < 0`, de frente para a lâmpada em `-x`. Só o
/// SINAL, não a lei (§4.L).
#[test]
fn o_sinal_da_normal_e_o_do_godot() {
    let o = oraculo();
    let g = &o["sinal"];
    let (e, d) = (
        g["esquerda"].as_f64().expect("e"),
        g["direita"].as_f64().expect("d"),
    );
    let (ce, cd) = (
        g["controlo_esquerda"].as_f64().expect("ce"),
        g["controlo_direita"].as_f64().expect("cd"),
    );
    assert!(
        e > d * 1.5 && e - d > 20.0,
        "Godot: o lado esquerdo é o claro: {g}"
    );
    assert!(
        (ce - cd).abs() < 0.1 * ce.max(cd).max(1.0),
        "o controlo sem normal: {g}"
    );
    let s = heroi_p14();
    let n = compose_normal(&s, s.frames()[0].id);
    let lado = |x0: u32, x1: u32| -> f64 {
        let mut soma = 0.0;
        for y in 4..12u32 {
            for x in x0..x1 {
                let i = ((y * 16 + x) * 4) as usize;
                // A lâmpada em -x: N·L = -n.x.
                soma += -(f64::from(n[i]) / 255.0 * 2.0 - 1.0);
            }
        }
        soma
    };
    assert!(
        lado(1, 7) > 0.0 && lado(9, 15) < 0.0,
        "o nosso: o lado esquerdo olha a lâmpada"
    );
}

/// Grava a entrada do oráculo na pasta `d` (o export real: folha, `_n.png`, JSON, `.tres`) e
/// devolve os bytes do `.tres`.
fn grava(s: &Sprite, d: &std::path::Path) -> Vec<u8> {
    std::fs::create_dir_all(d).expect("pasta");
    let e = Export::Folha(ExportFolha {
        opcoes: opcoes(),
        json: Some(OpcoesJson {
            array: true,
            tags: true,
            slices: true,
            ..OpcoesJson::default()
        }),
    });
    exporta::gravar(s, &e, &d.join("heroi.png")).expect("export");
    std::fs::read(d.join("heroi.tres")).expect("o .tres")
}

/// A entrada do oráculo: `PH2D_P14D_SAIDA=<pasta>`; também os `n<q>.rgba` (o nosso `compose_normal`
/// de cada quadro) e `cor<q>.rgba`, para o recorte.
#[test]
#[ignore = "escreve a entrada do oráculo do Godot"]
fn escreve_a_entrada_do_oraculo_p14d() {
    let d = std::path::PathBuf::from(std::env::var("PH2D_P14D_SAIDA").expect("PH2D_P14D_SAIDA"));
    let s = heroi_p14();
    grava(&s, &d);
    for (q, f) in s.frames().iter().enumerate() {
        std::fs::write(d.join(format!("n{q}.rgba")), compose_normal(&s, f.id)).expect("n");
        let cor =
            ph2d_pixel_io::render::composto(&s, f.id, ph2d_pixel_io::render::Camadas::Visiveis);
        std::fs::write(d.join(format!("cor{q}.rgba")), cor).expect("cor");
    }
}

/// Repeat Last Export lembra as duas caixas novas (o texto do preset vai e volta).
#[test]
fn o_repeat_lembra_normal_e_godot() {
    let mut s = heroi_p14();
    let e = Export::Folha(ExportFolha {
        opcoes: opcoes(),
        json: Some(OpcoesJson::default()),
    });
    exporta::lembrar(&mut s, &e, std::path::Path::new("/x/heroi.png"));
    let (volta, _) = exporta::o_ultimo(&s).expect("lembrado");
    assert_eq!(volta, Some(e));
}
