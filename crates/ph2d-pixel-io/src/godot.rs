//! **Godot SpriteFrames (`.tres`)** (P14d, [plano §3](../../../docs/PixelArt/P14_plano.md)): cada
//! tag vira uma animação. O Godot não tem PingPong nem repeat: a sequência DESENROLA-SE como o
//! Aseprite a toca (ping-pong sem repetir as pontas; N passadas ⇒ `loop = false`; repeat 0 ⇒ um
//! ciclo com `loop = true`). `speed` = 1000 / mdc das durações da tag, cada quadro com `duration` =
//! múltiplo EXATO. Oráculo: o Godot corrido (`a_sequencia_e_a_do_godot`, `o_tres_abre_no_godot`).

use ph2d_pixel_core::{Direction, SliceId, Sprite};

use crate::sheet::Folha;

/// Uma animação desenrolada: os índices de quadro do sprite, em ordem de toque.
#[derive(Debug, Clone, PartialEq)]
pub struct Animacao {
    pub nome: String,
    pub quadros: Vec<usize>,
    pub laco: bool,
    /// Quadros por segundo do Godot (`speed`).
    pub fps: f64,
    /// A `duration` de cada quadro de `quadros` (múltiplo de `1/fps`).
    pub duracoes: Vec<u32>,
}

fn mdc(a: u32, b: u32) -> u32 {
    if b == 0 { a } else { mdc(b, a % b) }
}

/// Uma passada `k` (0, 1, …) da tag `de..=ate`: as de ida e volta largam a ponta já tocada.
fn passada(de: usize, ate: usize, dir: Direction, k: u32, primeira: bool) -> Vec<usize> {
    let ida: Vec<usize> = (de..=ate).collect();
    let volta: Vec<usize> = (de..=ate).rev().collect();
    let (frente, tras) = match dir {
        Direction::Forward => return ida,
        Direction::Reverse => return volta,
        Direction::PingPong => (ida, volta),
        Direction::PingPongReverse => (volta, ida),
    };
    let p = if k % 2 == 0 { frente } else { tras };
    if primeira || p.len() < 2 {
        p
    } else {
        p[1..].to_vec()
    }
}

/// As animações das tags (sem tags: uma, `nome_sem_tags`, sobre todos os quadros, em laço).
#[must_use]
pub fn animacoes(s: &Sprite, nome_sem_tags: &str) -> Vec<Animacao> {
    let dur: Vec<u32> = s.frames().iter().map(|f| f.duration_ms.max(1)).collect();
    let ultimo = dur.len().saturating_sub(1);
    let lista: Vec<(String, usize, usize, Direction, u16)> = if s.tags().is_empty() {
        vec![(nome_sem_tags.to_owned(), 0, ultimo, Direction::Forward, 0)]
    } else {
        s.tags()
            .iter()
            .map(|t| {
                let (a, b) = (t.from.min(t.to) as usize, t.from.max(t.to) as usize);
                (t.name.clone(), a, b.min(ultimo), t.direction, t.repeat)
            })
            .collect()
    };
    lista
        .into_iter()
        .map(|(nome, de, ate, dir, repeat)| {
            let vai_e_volta = matches!(dir, Direction::PingPong | Direction::PingPongReverse);
            let mut quadros = Vec::new();
            if repeat == 0 {
                quadros = passada(de, ate, dir, 0, true);
                if vai_e_volta && ate > de + 1 {
                    let v = passada(de, ate, dir, 1, false);
                    quadros.extend_from_slice(&v[..v.len() - 1]);
                }
            } else {
                for k in 0..u32::from(repeat) {
                    quadros.extend(passada(de, ate, dir, k, k == 0));
                }
            }
            let g = quadros.iter().fold(0, |g, &i| mdc(g, dur[i]));
            let g = g.max(1);
            Animacao {
                nome,
                duracoes: quadros.iter().map(|&i| dur[i] / g).collect(),
                quadros,
                laco: repeat == 0,
                fps: 1000.0 / f64::from(g),
            }
        })
        .collect()
}

/// **O pivô ASSADO** (P14d-2): o `offset` de um `AnimatedSprite2D` é do nó, não do quadro. Cada
/// célula ganha, na `margin` do seu AtlasTexture, o deslocamento inteiro `P − ⌊pivô(q)⌋` (`P` = o
/// maior `⌊pivô⌋` de todos os quadros), e todas crescem ao mesmo tamanho: o pivô de cada quadro cai
/// no MESMO ponto da célula, e `offset = −(P + fração do pivô 0)` o põe no nó (`centered = false`).
/// Pivô fixo ⇒ deslocamento zero e os bytes de antes.
struct Ancora {
    desloca: Vec<(u32, u32)>,
    tamanho: (u32, u32),
    offset: (f64, f64),
}

fn ancora(s: &Sprite) -> Ancora {
    let p = pivos(s);
    let chao: Vec<(i64, i64)> = p
        .iter()
        .map(|&(x, y)| (x.floor() as i64, y.floor() as i64))
        .collect();
    let mx = chao.iter().map(|c| c.0).max().unwrap_or(0);
    let my = chao.iter().map(|c| c.1).max().unwrap_or(0);
    let (w, h) = (i64::from(s.width()), i64::from(s.height()));
    let ex = chao.iter().map(|c| w - c.0).max().unwrap_or(w);
    let ey = chao.iter().map(|c| h - c.1).max().unwrap_or(h);
    let (f0x, f0y) = p
        .first()
        .map_or((0.0, 0.0), |&(x, y)| (x - x.floor(), y - y.floor()));
    Ancora {
        desloca: chao
            .iter()
            .map(|c| ((mx - c.0) as u32, (my - c.1) as u32))
            .collect(),
        tamanho: ((mx + ex) as u32, (my + ey) as u32),
        offset: (-(mx as f64 + f0x), -(my as f64 + f0y)),
    }
}

/// O pivô de cada quadro (`meta.pivots` do JSON).
#[must_use]
pub fn pivos(s: &Sprite) -> Vec<(f64, f64)> {
    (0..s.frames().len() as u32)
        .map(|i| ph2d_pixel_core::slice::pivot_px(s, i))
        .collect()
}

fn esc(t: &str) -> String {
    t.replace('\\', "\\\\").replace('"', "\\\"")
}

/// **O texto do `.tres`.** `imagem` e `normal` são caminhos relativos ao `.tres`. `None` se a folha
/// não tem uma peça por quadro (split de camadas/tags/grelha: o Godot não tem onde as pôr; Split
/// slices é [`tres_por_slice`]).
#[must_use]
pub fn tres(s: &Sprite, f: &Folha, imagem: &str, normal: Option<&str>) -> Option<String> {
    if f.entradas.len() != s.frames().len() || f.de_onde.iter().any(|d| d.1.is_some()) {
        return None;
    }
    let celulas = celulas(s, f, None)?;
    celulas
        .iter()
        .all(Option::is_some)
        .then(|| texto(s, f, &celulas, imagem, normal))
}

/// **Split slices** (P14d-2): um `.tres` por slice — `(rótulo, texto)`, na ordem do JSON —, cada um
/// com as tags sobre as peças DESSA slice (o quadro em que ela se esconde fica fora da animação).
/// `None` se a folha não é de Split slices só (com camadas/tags/grelha: várias peças por quadro).
#[must_use]
pub fn tres_por_slice(
    s: &Sprite,
    f: &Folha,
    imagem: &str,
    normal: Option<&str>,
) -> Option<Vec<(String, String)>> {
    if f.de_onde.iter().all(|d| d.1.is_none()) {
        return None;
    }
    crate::sheet::rotulos_das_slices(s)
        .into_iter()
        .filter(|(id, _)| f.de_onde.iter().any(|d| d.1 == Some(*id)))
        .map(|(id, rotulo)| {
            let c = celulas(s, f, Some(id))?;
            Some((rotulo, texto(s, f, &c, imagem, normal)))
        })
        .collect()
}

/// A entrada de cada quadro (da slice `fatia`); `None` se algum quadro tem duas.
fn celulas(s: &Sprite, f: &Folha, fatia: Option<SliceId>) -> Option<Vec<Option<usize>>> {
    let mut c = vec![None; s.frames().len()];
    for (i, &(quadro, sl)) in f.de_onde.iter().enumerate() {
        if sl != fatia {
            continue;
        }
        let q = s.frames().iter().position(|fr| fr.id == quadro)?;
        if c[q].replace(i).is_some() {
            return None;
        }
    }
    Some(c)
}

fn texto(
    s: &Sprite,
    f: &Folha,
    celulas: &[Option<usize>],
    imagem: &str,
    normal: Option<&str>,
) -> String {
    let anims = animacoes(s, "default");
    let a = ancora(s);
    let mut o = String::new();
    o.push_str("[gd_resource type=\"SpriteFrames\" format=3]\n\n");
    o.push_str(&format!(
        "[ext_resource type=\"Texture2D\" path=\"{}\" id=\"1\"]\n",
        esc(imagem)
    ));
    let atlas = if let Some(n) = normal {
        o.push_str(&format!(
            "[ext_resource type=\"Texture2D\" path=\"{}\" id=\"2\"]\n\n",
            esc(n)
        ));
        o.push_str("[sub_resource type=\"CanvasTexture\" id=\"CanvasTexture_n\"]\n");
        o.push_str("diffuse_texture = ExtResource(\"1\")\nnormal_texture = ExtResource(\"2\")\n");
        "SubResource(\"CanvasTexture_n\")"
    } else {
        "ExtResource(\"1\")"
    };
    o.push('\n');
    for (q, i) in celulas.iter().enumerate() {
        let Some(i) = *i else { continue };
        let e = &f.entradas[i];
        let pad = (e.frame[2] - e.origem[2]) / 2;
        let (dx, dy) = a.desloca[q];
        o.push_str(&format!(
            "[sub_resource type=\"AtlasTexture\" id=\"q{i}\"]\natlas = {atlas}\n\
             region = Rect2({}, {}, {}, {})\nmargin = Rect2({}, {}, {}, {})\n\n",
            e.frame[0] + pad,
            e.frame[1] + pad,
            e.origem[2],
            e.origem[3],
            e.origem[0] + dx,
            e.origem[1] + dy,
            a.tamanho.0 - e.origem[2],
            a.tamanho.1 - e.origem[3],
        ));
    }
    let (ox, oy) = a.offset;
    o.push_str("[resource]\n");
    o.push_str(&format!("metadata/ph2d_offset = Vector2({ox:?}, {oy:?})\n"));
    o.push_str("animations = [");
    let mut k = 0;
    for a in &anims {
        let quadros: Vec<(usize, u32)> = a
            .quadros
            .iter()
            .zip(&a.duracoes)
            .filter_map(|(&q, &d)| celulas[q].map(|i| (i, d)))
            .collect();
        if quadros.is_empty() {
            continue;
        }
        if k > 0 {
            o.push_str(", ");
        }
        k += 1;
        o.push_str("{\n\"frames\": [");
        for (j, (i, d)) in quadros.iter().enumerate() {
            if j > 0 {
                o.push_str(", ");
            }
            o.push_str(&format!(
                "{{\n\"duration\": {:?},\n\"texture\": SubResource(\"q{i}\")\n}}",
                f64::from(*d)
            ));
        }
        o.push_str(&format!(
            "],\n\"loop\": {},\n\"name\": &\"{}\",\n\"speed\": {:?}\n}}",
            a.laco,
            esc(&a.nome),
            a.fps
        ));
    }
    o.push_str("]\n");
    o
}
