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

/// A entrada do oráculo: a `heroi_p14` e mais duas tags que só o desenrolar de repeat N exerce —
/// `walk3` 0–2 Forward repeat 3 e `back2` 3–5 Reverse repeat 2 (P14d-2).
fn heroi_oraculo() -> Sprite {
    use ph2d_pixel_core::{Direction, FrameSet, Op, Tag, UserData, apply};
    let mut s = heroi_p14();
    let mut tags = s.tags().to_vec();
    for (name, from, to, direction, repeat) in [
        ("walk3", 0, 2, Direction::Forward, 3),
        ("back2", 3, 5, Direction::Reverse, 2),
    ] {
        tags.push(Tag {
            name: name.into(),
            from,
            to,
            direction,
            repeat,
            color: [0, 0, 0, 255],
            user: UserData::default(),
        });
    }
    let (frames, slices) = (s.frames().to_vec(), s.slices().to_vec());
    apply(
        &mut s,
        Op::Frames(Box::new(FrameSet {
            frames,
            cels: Vec::new(),
            tags,
            slices,
        })),
    )
    .expect("as tags do oráculo");
    s
}

const TAGS_DO_ORACULO: [&str; 5] = ["idle", "run", "hit", "walk3", "back2"];

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

/// `(t, quadro)` de cada troca de quadro antes do último ms (a troca no último pode cair fora).
fn trocas(v: &[usize]) -> Vec<(usize, usize)> {
    (1..v.len() - 1)
        .filter(|&t| v[t] != v[t - 1])
        .map(|t| (t, v[t]))
        .collect()
}

/// **A sequência é a do Godot**: o `AnimatedSprite2D` com o NOSSO `.tres`, a passos de 1 ms, mostra
/// a mesma ORDEM de quadros com as mesmas durações que o `SpriteAnimator` do jogo. Medido (fixture,
/// `diferenca`): o Godot troca 1 passo DEPOIS da fronteira, também em aritmética exata (`exato`:
/// 64 passos de 1/64 ⇒ troca no 65.º); o produto segue o Aseprite (troca NA fronteira), logo o
/// Godot troca em `t` ou `t + 1` do nosso. A `walk3` (25 fps, `duration` 2 e 3) mostrou o
/// float32 do `frame_progress` a adiantar a troca de 1 ms (280, não 281): a fase de 1 ms é ruído;
/// a ORDEM e o número de trocas são exatos (P14d-2).
#[test]
fn a_sequencia_e_a_do_godot() {
    let o = oraculo();
    let x = o["exato"]["trocas"].as_array().expect("exato");
    assert_eq!(
        x.first().and_then(Value::as_u64),
        Some(65),
        "a convenção medida do Godot"
    );
    let s = heroi_oraculo();
    let mut diverge = Vec::new();
    for tag in TAGS_DO_ORACULO {
        let godot: Vec<usize> = o["sequencia"][tag]
            .as_array()
            .unwrap_or_else(|| panic!("{tag} na fixture"))
            .iter()
            .map(|v| v.as_u64().expect("quadro") as usize)
            .collect();
        assert_eq!(godot.len(), 2001, "{tag}: t = 0..=2000");
        let nosso = super::derivado::sequencia_do_jogo(&s, tag);
        assert_eq!(godot[0], nosso[0], "{tag}: o primeiro quadro");
        let (g, n) = (trocas(&godot), trocas(&nosso));
        // A ordem que o NOSSO `.tres` desenrola é a que o Godot tocou (o laço repete-se).
        let a = ph2d_pixel_io::godot::animacoes(&s, "default")
            .into_iter()
            .find(|a| a.nome == tag)
            .expect("a animação");
        let voltas = if a.laco { g.len() + 1 } else { 1 };
        let mut tres: Vec<usize> = a
            .quadros
            .iter()
            .copied()
            .cycle()
            .take(a.quadros.len() * voltas)
            .collect();
        tres.dedup();
        let tocou: Vec<usize> = std::iter::once(godot[0])
            .chain(g.iter().map(|x| x.1))
            .collect();
        assert_eq!(
            &tres[..tocou.len().min(tres.len())],
            &tocou[..],
            "{tag}: a ordem do nosso .tres × a que o Godot tocou"
        );
        assert_eq!(
            g.len(),
            n.len(),
            "{tag}: o número de trocas: Godot {g:?} × nosso {n:?}"
        );
        for ((tg, qg), (tn, qn)) in g.iter().zip(&n) {
            if qg != qn || !(*tn..=tn + 1).contains(tg) {
                diverge.push((tag, *tg, *qg, *tn, *qn));
            }
        }
    }
    assert!(
        diverge.is_empty(),
        "(tag, t Godot, quadro Godot, t nosso, quadro nosso): {diverge:?}"
    );
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
    let mut quer = TAGS_DO_ORACULO.to_vec();
    quer.sort_unstable();
    assert_eq!(nomes, quer, "o Godot lista por nome");
    assert_eq!(g["normal_texture"], Value::Bool(true), "{g}");
    let d = std::env::temp_dir().join(format!("ph2d_p14d_tres_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    let tres = grava(&heroi_oraculo(), &d);
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
    let s = heroi_oraculo();
    grava(&s, &d);
    // Antes da P14d-2 o split recusava o Godot: a pasta fica vazia e o oráculo diz `{}`.
    let _ = grava_split(&s, &d.join("split"));
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

/// O export do oráculo com Split slices, na pasta `d`.
fn grava_split(s: &Sprite, d: &std::path::Path) -> Result<exporta::Gravado, exporta::ErroGravar> {
    std::fs::create_dir_all(d).expect("pasta");
    let e = Export::Folha(ExportFolha {
        opcoes: OpcoesFolha {
            split_slices: true,
            ..opcoes()
        },
        json: Some(OpcoesJson {
            array: true,
            tags: true,
            slices: true,
            ..OpcoesJson::default()
        }),
    });
    exporta::gravar(s, &e, &d.join("heroi.png"))
}

/// Os dois números de `chave = Vector2(` / `Rect2(` a seguir a `depois` em `t`.
fn dois(t: &str, depois: &str, chave: &str) -> (f64, f64) {
    let i = t.find(depois).unwrap_or_else(|| panic!("{depois}")) + depois.len();
    let j = i + t[i..].find(chave).unwrap_or_else(|| panic!("{chave}")) + chave.len();
    let v: Vec<f64> = t[j..]
        .split([',', ')'])
        .take(2)
        .map(|x| x.trim().parse().expect("número"))
        .collect();
    (v[0], v[1])
}

/// **O pivô de cada quadro cai no nó do Godot** (P14d-2): o `offset` do `AnimatedSprite2D` é do nó
/// inteiro, e a `heroi_p14` muda o pivô no quadro 4 — o export ASSA o pivô na margem de cada
/// AtlasTexture. Medido (fixture `pivo`): render real, o pivô de cada quadro a 0 px do nó. O nosso
/// `.tres`, lido como o Godot o lê: `offset + margin + (pivô − origem) = 0` em todo quadro.
#[test]
fn o_pivo_de_cada_quadro_cai_no_no_do_godot() {
    let o = oraculo();
    let g = o["pivo"].as_array().expect("pivo");
    assert_eq!(g.len(), 6, "um por quadro");
    for v in g {
        assert_eq!(
            (v["dx"].as_f64(), v["dy"].as_f64()),
            (Some(0.0), Some(0.0)),
            "Godot: {v}"
        );
    }
    let s = heroi_oraculo();
    let pivos = ph2d_pixel_io::godot::pivos(&s);
    assert!(
        pivos.iter().any(|p| *p != pivos[0]),
        "a fixture tem o fenómeno: o pivô muda"
    );
    let d = std::env::temp_dir().join(format!("ph2d_p14d_pivo_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    let tres = String::from_utf8(grava(&s, &d)).expect("utf8");
    let _ = std::fs::remove_dir_all(&d);
    let f = montar(&s, &opcoes(), "heroi", "png");
    let (ox, oy) = dois(&tres, "[resource]", "ph2d_offset = Vector2(");
    let mut fora = Vec::new();
    for (q, e) in f.entradas.iter().enumerate() {
        let (mx, my) = dois(&tres, &format!("id=\"q{q}\"]"), "margin = Rect2(");
        let x = ox + mx + pivos[q].0 - f64::from(e.origem[0]);
        let y = oy + my + pivos[q].1 - f64::from(e.origem[1]);
        if (x, y) != (0.0, 0.0) {
            fora.push((q, x, y));
        }
    }
    assert!(fora.is_empty(), "(quadro, dx, dy) do pivô ao nó: {fora:?}");
    // Pivô fixo (sem slice de pivô: o centro) ⇒ nada assado: margin = origem, célula = o sprite.
    let mut fixo = heroi_oraculo();
    let op = ph2d_pixel_core::slice::set_pivot(&fixo, None).expect("sem pivô");
    ph2d_pixel_core::apply(&mut fixo, op).expect("sem pivô");
    let d = std::env::temp_dir().join(format!("ph2d_p14d_fixo_{}", std::process::id()));
    let tres = String::from_utf8(grava(&fixo, &d)).expect("utf8");
    let _ = std::fs::remove_dir_all(&d);
    for (q, e) in f.entradas.iter().enumerate() {
        let quer = format!(
            "margin = Rect2({}, {}, {}, {})",
            e.origem[0],
            e.origem[1],
            16 - e.origem[2],
            16 - e.origem[3]
        );
        let bloco = &tres[tres.find(&format!("id=\"q{q}\"]")).expect("atlas")..];
        assert!(
            bloco[..bloco.find("\n\n").expect("fim")].contains(&quer),
            "quadro {q}: {quer}"
        );
    }
    assert!(
        tres.contains("ph2d_offset = Vector2(-8.0, -8.0)"),
        "o centro"
    );
}

/// **Split slices grava um `.tres` por slice** (`<base>_<slice>.tres`, P14d-2), cada um com as tags,
/// sobre a MESMA folha (e `_n.png`) — os retângulos do JSON da slice. O Godot abre cada um (fixture
/// `split`) e conta os mesmos quadros por animação.
#[test]
fn o_split_de_slices_grava_um_tres_por_slice() {
    let s = heroi_oraculo();
    let d = std::env::temp_dir().join(format!("ph2d_p14d_split_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    let g = grava_split(&s, &d).expect("export com split");
    let tres = std::fs::read_to_string(d.join("heroi_body.tres"));
    let json = std::fs::read_to_string(d.join("heroi.json"));
    let _ = std::fs::remove_dir_all(&d);
    let tres = tres.expect("heroi_body.tres");
    let json: Value = serde_json::from_str(&json.expect("json")).expect("json");
    assert!(
        g.ficheiros.iter().any(|c| c.ends_with("heroi_body.tres")),
        "{:?}",
        g.ficheiros
    );
    assert!(tres.contains("path=\"heroi.png\"") && tres.contains("path=\"heroi_n.png\""));
    let quadros = json["frames"].as_array().expect("frames");
    assert_eq!(quadros.len(), 6, "a body em cada quadro");
    for (i, q) in quadros.iter().enumerate() {
        let pad = (q["frame"]["w"].as_u64().expect("w")
            - q["spriteSourceSize"]["w"].as_u64().expect("w"))
            / 2;
        let quer = format!(
            "region = Rect2({}, {}, {}, {})",
            q["frame"]["x"].as_u64().expect("x") + pad,
            q["frame"]["y"].as_u64().expect("y") + pad,
            q["spriteSourceSize"]["w"],
            q["spriteSourceSize"]["h"]
        );
        let bloco = &tres[tres.find(&format!("id=\"q{i}\"]")).expect("atlas")..];
        assert!(
            bloco.starts_with(&format!("id=\"q{i}\"]")) && bloco.contains(&quer),
            "{quer}"
        );
    }
    let o = oraculo();
    let godot = &o["split"]["heroi_body.tres"];
    assert_eq!(godot["erro"], Value::Null, "{godot}");
    for a in ph2d_pixel_io::godot::animacoes(&s, "default") {
        assert_eq!(
            godot["animacoes"][&a.nome].as_u64(),
            Some(a.quadros.len() as u64),
            "{}: {godot}",
            a.nome
        );
    }
}
