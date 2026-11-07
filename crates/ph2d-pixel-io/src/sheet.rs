//! **Onde cai cada quadro na folha.** Leis medidas no LibreSprite (`oracle_export.json`, `folhas`):
//! só cabe quem cabe com as DUAS bordas; uma peça maior que o limite fica sozinha na linha; a folha
//! encolhe ao que usou; `--trim` larga os vazios; em `split` os ligados partilham retângulo.
//! Sem oráculo (ausentes dele), definidos pelo CLI do Aseprite: 2ⁿ, merge, extrude, split de tags.

use std::collections::BTreeMap;

use ph2d_pixel_core::{FrameId, Layer, LayerId, LayerKind, Sprite};
use ph2d_sprite_sheet::pack::{LayoutItem, PackOptions, layout};

use crate::render::{Camadas, Direcao, Quadros, composto, entra, ordem};

/// Sheet Type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tipo {
    Horizontal,
    Vertical,
    #[default]
    Rows,
    Columns,
    Packed,
}

/// Constraints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Limite {
    #[default]
    Nenhum,
    Colunas(u32),
    Linhas(u32),
    Largura(u32),
    Altura(u32),
}

/// As abas Layout, Sprite e Borders.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OpcoesFolha {
    pub tipo: Tipo,
    pub limite: Limite,
    pub potencia_de_2: bool,
    pub merge_duplicates: bool,
    pub ignore_empty: bool,
    pub camadas: Camadas,
    pub quadros: Quadros,
    pub split_layers: bool,
    pub split_tags: bool,
    /// `--split-grid`: cada célula da grelha é uma peça da folha.
    pub split_grid: bool,
    /// `--trim-by-grid`: apara, com o recorte alargado às linhas da grelha.
    pub trim_by_grid: bool,
    /// `--trim`: cada quadro ao seu conteúdo.
    pub trim_cels: bool,
    /// `--trim-sprite`: todos ao retângulo que une os conteúdos.
    pub trim_sprite: bool,
    pub extrude: u32,
    pub border: u32,
    pub shape: u32,
    pub inner: u32,
    /// `None` = o padrão do Aseprite para o caso.
    pub nome: Option<String>,
}

/// Uma linha do JSON.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entrada {
    pub nome: String,
    /// `[x, y, w, h]` na folha, com a margem interna.
    pub frame: [u32; 4],
    pub trimmed: bool,
    /// `[x, y, w, h]` no sprite: o que sobrou depois do recorte.
    pub origem: [u32; 4],
    pub tamanho: [u32; 2],
    pub duration_ms: u32,
}

/// A folha pronta.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Folha {
    pub w: u32,
    pub h: u32,
    pub rgba: Vec<u8>,
    pub entradas: Vec<Entrada>,
}

/// Uma peça antes de ter lugar.
struct Peca {
    nome: String,
    grupo: usize,
    rect: [u32; 4],
    px: Vec<u8>,
    duration_ms: u32,
    /// O dono do cel ligado, em `split`: quem aponta para o mesmo partilha retângulo (medido).
    dono: Option<(LayerId, FrameId)>,
}

/// A grelha de `--split-grid` e `--trim-by-grid`: a da primeira camada de tilemap que entra na saída,
/// lida como retângulos: `(tile_w, tile_h, fase_x, fase_y)`. O formato de folha não tem outra forma.
fn grelha_da_saida(s: &Sprite, o: &OpcoesFolha) -> Option<(u32, u32, i32, i32)> {
    let (l, set) = s.layers().iter().find_map(|l| {
        let set = s.tileset(l.tileset()?)?;
        entra(s, l, o.camadas).then_some((l, set))
    })?;
    let (tw, th) = (i32::from(set.tile_w), i32::from(set.tile_h));
    let fase = s
        .entries()
        .filter(|(k, _)| k.0 == l.id)
        .find_map(|(_, c)| c.tiles.as_deref())
        .map_or((0, 0), |g| g.ancora(set.forma, tw, th).1);
    Some((
        tw as u32,
        th as u32,
        fase.0.rem_euclid(tw),
        fase.1.rem_euclid(th),
    ))
}

/// As caixas da grelha que tocam o canvas, linha a linha, cortadas por ele.
fn caixas_da_grelha((tw, th, fx, fy): (u32, u32, i32, i32), w: u32, h: u32) -> Vec<[u32; 4]> {
    let corte = |fase: i32, lado: u32, total: u32| -> Vec<(u32, u32)> {
        let mut v = Vec::new();
        let mut a = fase as u32;
        if a > 0 {
            v.push((0, a.min(total)));
        }
        while a < total {
            v.push((a, lado.min(total - a)));
            a += lado;
        }
        v
    };
    let (xs, ys) = (corte(fx, tw, w), corte(fy, th, h));
    let mut out = Vec::with_capacity(xs.len() * ys.len());
    for &(y, ch) in &ys {
        for &(x, cw) in &xs {
            out.push([x, y, cw, ch]);
        }
    }
    out
}

/// Alarga `r` às linhas da grelha, sem sair de `dentro`.
fn ao_multiplo(r: [u32; 4], (tw, th, fx, fy): (u32, u32, i32, i32), dentro: [u32; 4]) -> [u32; 4] {
    let baixo = |v: u32, lado: u32, fase: i32| {
        let (v, lado) = (i64::from(v), i64::from(lado));
        (v - i64::from(fase)).div_euclid(lado) * lado + i64::from(fase)
    };
    let cima = |v: u32, lado: u32, fase: i32| {
        let (b, l) = (baixo(v, lado, fase), i64::from(lado));
        if b == i64::from(v) { b } else { b + l }
    };
    let x0 = baixo(r[0], tw, fx).max(i64::from(dentro[0]));
    let y0 = baixo(r[1], th, fy).max(i64::from(dentro[1]));
    let x1 = cima(r[0] + r[2], tw, fx).min(i64::from(dentro[0] + dentro[2]));
    let y1 = cima(r[1] + r[3], th, fy).min(i64::from(dentro[1] + dentro[3]));
    [x0 as u32, y0 as u32, (x1 - x0) as u32, (y1 - y0) as u32]
}

fn limites_do_conteudo(px: &[u8], w: u32, h: u32) -> Option<[u32; 4]> {
    let (mut x0, mut y0, mut x1, mut y1) = (w, h, 0, 0);
    let mut algum = false;
    for y in 0..h {
        for x in 0..w {
            if px[(y as usize * w as usize + x as usize) * 4 + 3] != 0 {
                (x0, y0) = (x0.min(x), y0.min(y));
                (x1, y1) = (x1.max(x + 1), y1.max(y + 1));
                algum = true;
            }
        }
    }
    algum.then(|| [x0, y0, x1 - x0, y1 - y0])
}

fn recorte(px: &[u8], w: u32, r: [u32; 4]) -> Vec<u8> {
    let mut out = Vec::with_capacity(r[2] as usize * r[3] as usize * 4);
    for y in r[1]..r[1] + r[3] {
        let i = (y as usize * w as usize + r[0] as usize) * 4;
        out.extend_from_slice(&px[i..i + r[2] as usize * 4]);
    }
    out
}

/// Tokens do `--filename-format`. `{frame}` conta na saída; `{frameNN}`: largura pelos dígitos, e o
/// valor deles é o primeiro número (`{frame1}` começa em 1, `{frame01}` tem 2 dígitos).
#[must_use]
pub fn nome_do_quadro(
    fmt: &str,
    titulo: &str,
    camada: &str,
    tag: &str,
    n: usize,
    ext: &str,
) -> String {
    let mut out = String::new();
    let mut resto = fmt;
    while let Some(a) = resto.find('{') {
        let Some(b) = resto[a..].find('}').map(|b| a + b) else {
            break;
        };
        out.push_str(&resto[..a]);
        let t = &resto[a + 1..b];
        match t {
            "title" | "name" => out.push_str(titulo),
            "layer" => out.push_str(camada),
            "tag" => out.push_str(tag),
            "extension" => out.push_str(ext),
            _ => match t.strip_prefix("frame") {
                Some(d) if d.bytes().all(|c| c.is_ascii_digit()) => {
                    let n = n + d.parse::<usize>().unwrap_or(0);
                    out.push_str(&format!("{n:0w$}", w = d.len()));
                }
                _ => {
                    out.push('{');
                    out.push_str(t);
                    out.push('}');
                }
            },
        }
        resto = &resto[b + 1..];
    }
    out.push_str(resto);
    out
}

fn padrao(o: &OpcoesFolha, com_camada: bool, com_tag: bool, varios: bool) -> String {
    let mut f = String::from("{title}");
    if com_camada {
        f.push_str(" ({layer})");
    }
    if com_tag {
        f.push_str(" #{tag}");
    }
    if varios {
        f.push_str(" {frame}");
    }
    f.push_str(".{extension}");
    o.nome.clone().unwrap_or(f)
}

fn tag_de(s: &Sprite, i: usize) -> String {
    let mut ts: Vec<_> = s.tags().iter().collect();
    ts.sort_by_key(|t| t.from);
    ts.iter()
        .find(|t| t.from as usize <= i && i <= t.to as usize)
        .map_or_else(String::new, |t| t.name.clone())
}

fn pecas(s: &Sprite, o: &OpcoesFolha, titulo: &str, ext: &str, feito: &dyn Fn(f32)) -> Vec<Peca> {
    let (w, h) = (s.width(), s.height());
    // Cada grupo de saída: uma camada (ou todas) × uma lista de quadros.
    let camadas: Vec<Option<&Layer>> = if o.split_layers {
        s.layers()
            .iter()
            .filter(|l| matches!(l.kind, LayerKind::Image) && entra(s, l, o.camadas))
            .map(Some)
            .collect()
    } else {
        vec![None]
    };
    let listas: Vec<(Option<&str>, Vec<usize>)> = if o.split_tags {
        s.tags()
            .iter()
            .enumerate()
            .map(|(i, t)| {
                (
                    Some(t.name.as_str()),
                    ordem(s, Quadros::Tag(i), Direcao::Forward),
                )
            })
            .collect()
    } else {
        let nome = match o.quadros {
            Quadros::Tag(i) => s.tags().get(i).map(|t| t.name.as_str()),
            _ => None,
        };
        vec![(nome, ordem(s, o.quadros, Direcao::Forward))]
    };
    let total: usize = camadas.len() * listas.iter().map(|(_, l)| l.len()).sum::<usize>();
    let cortes = grelha_da_saida(s, o)
        .filter(|_| o.split_grid)
        .map(|g| caixas_da_grelha(g, w, h));
    let mut out = Vec::new();
    let mut grupo = 0;
    for camada in &camadas {
        let so = match (camada, o.camadas) {
            (Some(l), _) => Camadas::Uma(l.id),
            (None, c) => c,
        };
        let nome_camada = match (camada, o.camadas) {
            (Some(l), _) => l.name.clone(),
            (None, Camadas::Uma(id)) => s.layer(id).map_or_else(String::new, |l| l.name.clone()),
            _ => String::new(),
        };
        let so_uma = camada.is_some() || matches!(o.camadas, Camadas::Uma(_));
        for (tag, lista) in &listas {
            let varios = lista.len() > 1 || cortes.is_some();
            let fmt = padrao(o, so_uma, tag.is_some(), varios);
            let mut item = 0;
            for &i in lista {
                feito(out.len() as f32 / total.max(1) as f32);
                let fr = s.frames()[i];
                let px = composto(s, fr.id, so);
                let t = tag.map_or_else(|| tag_de(s, i), str::to_owned);
                if let Some(caixas) = &cortes {
                    for &rect in caixas {
                        out.push(Peca {
                            nome: nome_do_quadro(&fmt, titulo, &nome_camada, &t, item, ext),
                            grupo,
                            rect,
                            px: recorte(&px, w, rect),
                            duration_ms: fr.duration_ms,
                            dono: None,
                        });
                        item += 1;
                    }
                    continue;
                }
                let nome = nome_do_quadro(&fmt, titulo, &nome_camada, &t, item, ext);
                item += 1;
                // Só partilha quem tem o dono na saída (medido: `tag_split`, o dono fora da tag não une).
                let dono = camada.and_then(|l| {
                    let c = s.entry((l.id, fr.id))?;
                    let d = c.link.unwrap_or(fr.id);
                    lista.contains(&s.frame_index(d)?).then_some((l.id, d))
                });
                out.push(Peca {
                    nome,
                    grupo,
                    rect: [0, 0, w, h],
                    px,
                    duration_ms: fr.duration_ms,
                    dono,
                });
            }
            grupo += 1;
        }
    }
    out
}

/// O conteúdo da peça, em coordenadas do sprite.
fn conteudo(p: &Peca) -> Option<[u32; 4]> {
    let c = limites_do_conteudo(&p.px, p.rect[2], p.rect[3])?;
    Some([c[0] + p.rect[0], c[1] + p.rect[1], c[2], c[3]])
}

/// O recorte e os vazios (medido: `--trim` larga os vazios mesmo sem `--ignore-empty`).
fn apara(mut v: Vec<Peca>, o: &OpcoesFolha, grade: Option<(u32, u32, i32, i32)>) -> Vec<Peca> {
    let uniao = o.trim_sprite.then(|| {
        v.iter()
            .filter_map(conteudo)
            .fold(None, |a: Option<[u32; 4]>, r| {
                Some(a.map_or(r, |a| {
                    let (x0, y0) = (a[0].min(r[0]), a[1].min(r[1]));
                    let (x1, y1) = (
                        (a[0] + a[2]).max(r[0] + r[2]),
                        (a[1] + a[3]).max(r[1] + r[3]),
                    );
                    [x0, y0, x1 - x0, y1 - y0]
                }))
            })
    });
    let por_grelha = grade.filter(|_| o.trim_by_grid);
    let cada = o.trim_cels || por_grelha.is_some();
    v.retain_mut(|p| {
        let c = conteudo(p);
        if c.is_none() && (o.ignore_empty || cada || o.trim_sprite) {
            return false;
        }
        let r = match (uniao, cada) {
            (Some(Some(u)), _) => Some(u),
            (_, true) => c,
            _ => None,
        };
        let r = match (r, por_grelha) {
            (Some(r), Some(g)) => Some(ao_multiplo(r, g, p.rect)),
            (r, _) => r,
        };
        if let Some(r) = r {
            let dentro = [r[0] - p.rect[0], r[1] - p.rect[1], r[2], r[3]];
            p.px = recorte(&p.px, p.rect[2], dentro);
            p.rect = r;
        }
        true
    });
    v
}

/// As caixas (com extrude e margem interna) postas em grelha.
fn grelha(tam: &[[u32; 2]], grupos: &[usize], o: &OpcoesFolha) -> (Vec<[u32; 2]>, u32, u32) {
    let (b, g) = (o.border, o.shape);
    // Tudo se escreve como «linhas»; Columns troca os eixos à entrada e à saída.
    let colunar = matches!(o.tipo, Tipo::Columns | Tipo::Vertical);
    let lado = |t: [u32; 2]| if colunar { [t[1], t[0]] } else { t };
    let n = tam.len() as u32;
    let max = tam
        .iter()
        .map(|&t| lado(t))
        .fold([0, 0], |a, t| [a[0].max(t[0]), a[1].max(t[1])]);
    // O limite no eixo em que a linha corre, ou quantas por linha.
    let (por_px, por_n): (Option<u32>, Option<u32>) = match (o.tipo, o.limite) {
        (Tipo::Horizontal | Tipo::Vertical, _) | (_, Limite::Nenhum) => (None, None),
        (Tipo::Rows, Limite::Largura(p)) | (Tipo::Columns, Limite::Altura(p)) => (Some(p), None),
        (Tipo::Rows, Limite::Colunas(c)) | (Tipo::Columns, Limite::Linhas(c)) => {
            (None, Some(c.max(1)))
        }
        (Tipo::Rows, Limite::Linhas(r)) | (Tipo::Columns, Limite::Colunas(r)) => {
            (None, Some(n.div_ceil(r.max(1)).max(1)))
        }
        (Tipo::Rows, Limite::Altura(p)) | (Tipo::Columns, Limite::Largura(p)) => {
            let faixas = (p.saturating_sub(2 * b) + g) / (max[1] + g).max(1);
            (None, Some(n.div_ceil(faixas.max(1)).max(1)))
        }
        (Tipo::Packed, _) => (None, None),
    };
    let quebra_por_grupo =
        por_px.is_none() && por_n.is_none() && matches!(o.tipo, Tipo::Rows | Tipo::Columns);
    let mut out = Vec::with_capacity(tam.len());
    let (mut u, mut v, mut alto, mut na_linha, mut maior_u) = (b, b, 0, 0, 0);
    for (i, &t) in tam.iter().enumerate() {
        let [cu, cv] = lado(t);
        let cabe = por_px.is_none_or(|p| u + cu + b <= p);
        let conta = por_n.is_none_or(|c| na_linha < c);
        let muda_grupo = quebra_por_grupo && i > 0 && grupos[i] != grupos[i - 1];
        if na_linha > 0 && (!cabe || !conta || muda_grupo) {
            v += alto + g;
            (u, alto, na_linha) = (b, 0, 0);
        }
        out.push(if colunar { [v, u] } else { [u, v] });
        maior_u = maior_u.max(u + cu);
        alto = alto.max(cv);
        u += cu + g;
        na_linha += 1;
    }
    let (fu, fv) = (maior_u + b, v + alto + b);
    if colunar {
        (out, fv, fu)
    } else {
        (out, fu, fv)
    }
}

fn empacotado(tam: &[[u32; 2]], o: &OpcoesFolha) -> (Vec<[u32; 2]>, u32, u32) {
    let itens: Vec<_> = tam
        .iter()
        .enumerate()
        .map(|(i, t)| LayoutItem {
            name: format!("{i:08}"),
            width: t[0],
            height: t[1],
        })
        .collect();
    let opts = PackOptions {
        padding: o.shape,
        max_size: u32::from(u16::MAX),
    };
    match layout(&itens, opts) {
        Ok(l) => (
            l.places
                .iter()
                .map(|(_, p)| [p[0] + o.border, p[1] + o.border])
                .collect(),
            l.size + 2 * o.border,
            l.size + 2 * o.border,
        ),
        // Não coube num lado de 65 535 (o limite do formato do documento): recai em linhas.
        Err(_) => grelha(tam, &vec![0; tam.len()], o),
    }
}

/// **A folha.** `titulo` e `ext` são o que `{title}` e `{extension}` dizem.
#[must_use]
pub fn montar(s: &Sprite, o: &OpcoesFolha, titulo: &str, ext: &str) -> Folha {
    montar_com(s, o, titulo, ext, &|_| {})
}

/// [`montar`], dizendo a fração das peças já compostas.
#[must_use]
pub fn montar_com(
    s: &Sprite,
    o: &OpcoesFolha,
    titulo: &str,
    ext: &str,
    feito: &dyn Fn(f32),
) -> Folha {
    let (w, h) = (s.width(), s.height());
    let v = apara(pecas(s, o, titulo, ext, feito), o, grelha_da_saida(s, o));
    let pad = o.inner + o.extrude;

    // Quem partilha retângulo: `unico[i]` é o índice da peça que pinta a i-ésima.
    let mut por_conteudo: BTreeMap<(u32, u32, &[u8]), usize> = BTreeMap::new();
    let mut por_dono: BTreeMap<(LayerId, FrameId), usize> = BTreeMap::new();
    let mut unico = Vec::with_capacity(v.len());
    for (i, p) in v.iter().enumerate() {
        let j = if o.merge_duplicates {
            *por_conteudo
                .entry((p.rect[2], p.rect[3], &p.px))
                .or_insert(i)
        } else if let Some(d) = p.dono {
            *por_dono.entry(d).or_insert(i)
        } else {
            i
        };
        unico.push(j);
    }
    let pintadas: Vec<usize> = (0..v.len()).filter(|&i| unico[i] == i).collect();
    let tam: Vec<[u32; 2]> = pintadas
        .iter()
        .map(|&i| [v[i].rect[2] + 2 * pad, v[i].rect[3] + 2 * pad])
        .collect();
    let grupos: Vec<usize> = pintadas.iter().map(|&i| v[i].grupo).collect();
    let (lugares, mut fw, mut fh) = if tam.is_empty() {
        (Vec::new(), 2 * o.border, 2 * o.border)
    } else if o.tipo == Tipo::Packed {
        empacotado(&tam, o)
    } else {
        grelha(&tam, &grupos, o)
    };
    (fw, fh) = (fw.max(1), fh.max(1));
    if o.potencia_de_2 {
        (fw, fh) = (fw.next_power_of_two(), fh.next_power_of_two());
    }
    let mut rgba = vec![0_u8; fw as usize * fh as usize * 4];
    let mut lugar_de = vec![[0_u32; 2]; v.len()];
    for (k, &i) in pintadas.iter().enumerate() {
        lugar_de[i] = lugares[k];
        carimba(&mut rgba, fw, lugares[k], &v[i], o);
    }
    let entradas = v
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let l = lugar_de[unico[i]];
            Entrada {
                nome: p.nome.clone(),
                frame: [
                    l[0] + o.extrude,
                    l[1] + o.extrude,
                    p.rect[2] + 2 * o.inner,
                    p.rect[3] + 2 * o.inner,
                ],
                trimmed: o.trim_cels || o.trim_sprite || o.trim_by_grid,
                origem: p.rect,
                tamanho: [w, h],
                duration_ms: p.duration_ms,
            }
        })
        .collect();
    Folha {
        w: fw,
        h: fh,
        rgba,
        entradas,
    }
}

/// Pinta a peça; o extrude repete a borda dela nas células de fora.
fn carimba(fo: &mut [u8], fw: u32, l: [u32; 2], p: &Peca, o: &OpcoesFolha) {
    let (pw, ph) = (p.rect[2] as i64, p.rect[3] as i64);
    let (e, m) = (i64::from(o.extrude), i64::from(o.inner));
    for dy in -e..ph + e {
        for dx in -e..pw + e {
            let (sx, sy) = (dx.clamp(0, pw - 1), dy.clamp(0, ph - 1));
            let si = (sy * pw + sx) as usize * 4;
            let x = i64::from(l[0]) + e + m + dx;
            let y = i64::from(l[1]) + e + m + dy;
            let di = (y * i64::from(fw) + x) as usize * 4;
            fo[di..di + 4].copy_from_slice(&p.px[si..si + 4]);
        }
    }
}

/// O que a aba Output pede ao JSON.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct OpcoesJson {
    pub array: bool,
    pub tags: bool,
    pub layers: bool,
    pub slices: bool,
}

/// O nome de cada modo no JSON (medido no LibreSprite; os 3 últimos ele diz `unknown`, ficam os da spec).
#[must_use]
pub fn nome_do_modo(c: u8) -> &'static str {
    const NOMES: [&str; 19] = [
        "normal",
        "multiply",
        "screen",
        "overlay",
        "darken",
        "lighten",
        "color_dodge",
        "color_burn",
        "hard_light",
        "soft_light",
        "difference",
        "exclusion",
        "hsl_hue",
        "hsl_saturation",
        "hsl_color",
        "hsl_luminosity",
        "addition",
        "subtract",
        "divide",
    ];
    NOMES.get(usize::from(c)).copied().unwrap_or("normal")
}

/// O texto do `.json` da folha. As tags saem por `from` (medido).
#[must_use]
pub fn json_da_folha(
    s: &Sprite,
    f: &Folha,
    image: &str,
    formato: &'static str,
    o: OpcoesJson,
) -> String {
    use ph2d_sprite_sheet::aseprite::{JsonFrame, JsonLayer, JsonTag, SheetJson, sheet_json};
    const DIRECOES: [&str; 4] = ["forward", "reverse", "pingpong", "pingpong_reverse"];
    let frames = f
        .entradas
        .iter()
        .map(|e| JsonFrame {
            filename: e.nome.clone(),
            frame: e.frame,
            trimmed: e.trimmed,
            source: e.origem,
            source_size: e.tamanho,
            duration_ms: e.duration_ms,
        })
        .collect();
    let tags = o.tags.then(|| {
        let mut v: Vec<_> = s.tags().iter().collect();
        v.sort_by_key(|t| t.from);
        v.iter()
            .map(|t| JsonTag {
                name: t.name.clone(),
                from: t.from,
                to: t.to,
                direction: DIRECOES[usize::from(t.direction.code()) % 4],
                repeat: t.repeat,
                color: t.color,
            })
            .collect()
    });
    let layers = o.layers.then(|| {
        s.layers()
            .iter()
            .filter(|l| !l.reference)
            .map(|l| JsonLayer {
                name: l.name.clone(),
                group: l.parent.and_then(|p| s.layer(p)).map(|p| p.name.clone()),
                opacity: (!l.is_group()).then_some(l.opacity),
                blend: (!l.is_group()).then(|| nome_do_modo(l.blend.code())),
            })
            .collect()
    });
    sheet_json(&SheetJson {
        frames,
        array: o.array,
        image: image.to_owned(),
        format: formato,
        size: [f.w, f.h],
        tags,
        layers,
        slices: o.slices,
    })
}
