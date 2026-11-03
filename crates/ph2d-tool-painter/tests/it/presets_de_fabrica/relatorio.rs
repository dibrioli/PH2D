//! **O RELATÓRIO dos pincéis que mudaram** (P5, [PLANO_ACELERACAO_2026-11](../../../../../docs/DevOps/PLANO_ACELERACAO_2026-11.md)):
//! uma página que o dono abre — `target/relatorio-pinceis/index.html`. Os alarmes baratos (a miniatura, a largura)
//! escrevem nela só os pincéis que mudaram; quem decide é o dono, a olho, e a aprovação vira a referência
//! (`PH2D_PINCEIS_APROVA=1`, como o `cargo insta accept`). Cada alarme escreve a SUA parte de cada pincel
//! (`<pincel>/<parte>.frag`) e refaz o índice com as partes que houver — os dois correm em paralelo.

use std::fmt::Write as _;
use std::path::PathBuf;

/// A variável que APROVA o que os alarmes acusaram: regrava a referência e o alarme passa.
pub(super) const APROVA: &str = "PH2D_PINCEIS_APROVA";

pub(super) fn aprova() -> bool {
    std::env::var(APROVA).as_deref() == Ok("1")
}

pub(super) fn pasta() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/relatorio-pinceis")
}

/// Uma imagem do relatório: RGBA `w × h`, a escala com que se mostra e a legenda.
pub(super) struct Figura {
    pub(super) rgba: Vec<u8>,
    pub(super) w: u32,
    pub(super) h: u32,
    pub(super) escala: u32,
    pub(super) legenda: String,
}

/// O que uma `parte` (o alarme) diz de um pincel: o título, a linha de texto e as figuras.
pub(super) struct Parte {
    pub(super) pincel: String,
    pub(super) texto: String,
    pub(super) figuras: Vec<Figura>,
}

fn pasta_do_pincel(nome: &str) -> PathBuf {
    let seguro: String = nome
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    pasta().join(seguro)
}

/// Grava as `partes` da `parte` (`"1-miniatura"`, `"2-largura"`: o prefixo é a ordem na página), apaga as dessa parte
/// que ficaram de uma corrida anterior e refaz o índice. Devolve o caminho do índice.
pub(super) fn grava(parte: &str, titulo: &str, partes: &[Parte]) -> PathBuf {
    let raiz = pasta();
    std::fs::create_dir_all(&raiz).unwrap();
    let frag = format!("{parte}.frag");
    for e in std::fs::read_dir(&raiz).unwrap().flatten() {
        for f in std::fs::read_dir(e.path()).into_iter().flatten().flatten() {
            let nome = f.file_name().to_string_lossy().into_owned();
            if nome == frag || (nome.starts_with(&format!("{parte}-")) && nome.ends_with(".png")) {
                let _ = std::fs::remove_file(f.path());
            }
        }
    }
    for p in partes {
        let dir = pasta_do_pincel(&p.pincel);
        std::fs::create_dir_all(&dir).unwrap();
        let mut html = format!("<h3>{titulo}</h3><p>{}</p><div class=fig>", p.texto);
        for (k, f) in p.figuras.iter().enumerate() {
            let png = format!("{parte}-{k}.png");
            std::fs::write(
                dir.join(&png),
                ph2d_brush_presets::escrever_textura_rgba(&f.rgba, f.w, f.h),
            )
            .unwrap();
            let pasta = dir.file_name().unwrap().to_string_lossy();
            let _ = write!(
                html,
                "<figure><img src=\"{pasta}/{png}\" width={} height={}><figcaption>{}</figcaption></figure>",
                f.w * f.escala,
                f.h * f.escala,
                f.legenda
            );
        }
        html.push_str("</div>");
        std::fs::write(dir.join(&frag), html).unwrap();
    }
    indice()
}

/// O índice: um bloco por pincel com as partes que houver, pela ordem do nome da parte.
fn indice() -> PathBuf {
    let raiz = pasta();
    let mut pinceis: Vec<(String, Vec<(String, String)>)> = Vec::new();
    for e in std::fs::read_dir(&raiz).unwrap().flatten() {
        if !e.path().is_dir() {
            continue;
        }
        let mut frags: Vec<(String, String)> = std::fs::read_dir(e.path())
            .unwrap()
            .flatten()
            .filter(|f| f.path().extension().is_some_and(|x| x == "frag"))
            .map(|f| {
                (
                    f.file_name().to_string_lossy().into_owned(),
                    std::fs::read_to_string(f.path()).unwrap(),
                )
            })
            .collect();
        if frags.is_empty() {
            continue;
        }
        frags.sort();
        pinceis.push((e.file_name().to_string_lossy().replace('_', " "), frags));
    }
    pinceis.sort();
    let mut html = String::from(
        "<!doctype html><meta charset=utf-8><title>Pincéis que mudaram</title><style>\
         body{font-family:sans-serif;margin:16px;max-width:1400px}h2{border-top:1px solid silver;padding-top:12px}\
         .fig{display:flex;flex-wrap:wrap;gap:16px}figure{margin:0}img{image-rendering:pixelated;\
         border:1px solid silver;background:white}figcaption{font-size:13px;max-width:420px}</style>\
         <h1>Pincéis que mudaram</h1>",
    );
    let _ = write!(
        html,
        "<p>{} pincel(éis). A miniatura: antes, depois e a diferença (escuro = mudou). O traço: o anel cor-de-rosa é o \
         cursor no tamanho do pincel — o traço deve enchê-lo. Aprovar: <code>{APROVA}=1</code> (regrava a referência).</p>",
        pinceis.len()
    );
    for (nome, frags) in pinceis {
        let _ = write!(html, "<h2>{nome}</h2>");
        for (_, f) in frags {
            html.push_str(&f);
        }
    }
    let tmp = raiz.join(format!("index.html.{}", std::process::id()));
    std::fs::write(&tmp, html).unwrap();
    std::fs::rename(&tmp, raiz.join("index.html")).unwrap();
    raiz.join("index.html")
}

/// A diferença de duas imagens do mesmo tamanho: branco onde são iguais, escuro onde mudou (×4, para se ver).
pub(super) fn diferenca(a: &[u8], b: &[u8]) -> Vec<u8> {
    a.chunks(4)
        .zip(b.chunks(4))
        .flat_map(|(p, q)| {
            let d = (0..4).map(|k| p[k].abs_diff(q[k])).max().unwrap_or(0);
            let v = 255 - d.saturating_mul(4);
            [v, v, v, 255]
        })
        .collect()
}

/// O traço de `px` (RGBA `w × h`) recortado à volta do centro da tinta, com o anel do CURSOR de raio `raio` (o tamanho
/// do pincel) por cima — o que o dono compara: o traço enche o anel?
pub(super) fn traco_com_cursor(px: &[u8], w: u32, h: u32, raio: f32) -> (Vec<u8>, u32, u32) {
    let (mut sx, mut sy, mut s) = (0.0f64, 0.0f64, 0.0f64);
    for y in 0..h {
        for x in 0..w {
            let i = ((y * w + x) * 4) as usize;
            let tinta =
                255.0 - (f64::from(px[i]) + f64::from(px[i + 1]) + f64::from(px[i + 2])) / 3.0;
            if tinta > 8.0 {
                sx += tinta * f64::from(x);
                sy += tinta * f64::from(y);
                s += tinta;
            }
        }
    }
    let (cx, cy) = if s > 0.0 {
        (sx / s, sy / s)
    } else {
        (f64::from(w) * 0.5, f64::from(h) * 0.5)
    };
    let (mx, my) = (
        f64::from(raio * 4.0).max(48.0),
        f64::from(raio * 2.5).max(32.0),
    );
    let x0 = (cx - mx).max(0.0) as u32;
    let y0 = (cy - my).max(0.0) as u32;
    let x1 = ((cx + mx) as u32).min(w);
    let y1 = ((cy + my) as u32).min(h);
    let (cw, ch) = (x1 - x0, y1 - y0);
    let mut out = Vec::with_capacity((cw * ch * 4) as usize);
    for y in y0..y1 {
        for x in x0..x1 {
            let d = ((f64::from(x) + 0.5 - cx).powi(2) + (f64::from(y) + 0.5 - cy).powi(2)).sqrt();
            if (d - f64::from(raio)).abs() <= 0.5 {
                out.extend_from_slice(&[255, 0, 200, 255]);
            } else {
                let i = ((y * w + x) * 4) as usize;
                out.extend_from_slice(&px[i..i + 4]);
            }
        }
    }
    (out, cw, ch)
}
