//! **Godot SpriteFrames (`.tres`)** (P14d, [plano §3](../../../docs/PixelArt/P14_plano.md)): cada
//! tag vira uma animação. O Godot não tem PingPong nem repeat: a sequência DESENROLA-SE como o
//! Aseprite a toca (ping-pong sem repetir as pontas; N passadas ⇒ `loop = false`; repeat 0 ⇒ um
//! ciclo com `loop = true`). `speed` = 1000 / mdc das durações da tag, cada quadro com `duration` =
//! múltiplo EXATO. Oráculo: o Godot corrido (`a_sequencia_e_a_do_godot`, `o_tres_abre_no_godot`).

use ph2d_pixel_core::{Direction, Sprite};

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

/// O pivô do quadro 0 em píxeis do sprite: a origem de um `AnimatedSprite2D` com `centered = false`
/// é `offset = -pivô` (o Godot não muda o offset por quadro; o pivô de cada quadro vai no JSON).
#[must_use]
pub fn offset(s: &Sprite) -> (f64, f64) {
    let (x, y) = ph2d_pixel_core::slice::pivot_px(s, 0);
    (-x, -y)
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
/// não tem uma peça por quadro (split de camadas/tags/slices/grelha: o Godot não tem onde as pôr).
#[must_use]
pub fn tres(s: &Sprite, f: &Folha, imagem: &str, normal: Option<&str>) -> Option<String> {
    if f.entradas.len() != s.frames().len() {
        return None;
    }
    let anims = animacoes(s, "default");
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
    for (i, e) in f.entradas.iter().enumerate() {
        let pad = (e.frame[2] - e.origem[2]) / 2;
        o.push_str(&format!(
            "[sub_resource type=\"AtlasTexture\" id=\"q{i}\"]\natlas = {atlas}\n\
             region = Rect2({}, {}, {}, {})\nmargin = Rect2({}, {}, {}, {})\n\n",
            e.frame[0] + pad,
            e.frame[1] + pad,
            e.origem[2],
            e.origem[3],
            e.origem[0],
            e.origem[1],
            e.tamanho[0] - e.origem[2],
            e.tamanho[1] - e.origem[3],
        ));
    }
    let (ox, oy) = offset(s);
    o.push_str("[resource]\n");
    o.push_str(&format!("metadata/ph2d_offset = Vector2({ox:?}, {oy:?})\n"));
    o.push_str("animations = [");
    for (k, a) in anims.iter().enumerate() {
        if k > 0 {
            o.push_str(", ");
        }
        o.push_str("{\n\"frames\": [");
        for (j, (&q, &d)) in a.quadros.iter().zip(&a.duracoes).enumerate() {
            if j > 0 {
                o.push_str(", ");
            }
            o.push_str(&format!(
                "{{\n\"duration\": {:?},\n\"texture\": SubResource(\"q{q}\")\n}}",
                f64::from(d)
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
    Some(o)
}
