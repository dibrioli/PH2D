//! **Gravar um export no disco e lembrar dele** (File ▸ Export As, Export Sprite Sheet, Repeat Last
//! Export). O que sai em cada quadro é de [`crate::render`], cada formato é de [`crate::saida`], e a
//! folha é de [`crate::sheet`]: aqui só o caminho de cada ficheiro e as opções em texto.

use std::path::{Path, PathBuf};

use ph2d_pixel_core::{ExportPreset, LayerId, Sprite};

use crate::render::{Area, Camadas, Direcao, Pedido, Quadros};
use crate::saida::{self, AvisoSaida, ErroSaida, Formato};
use crate::sheet::{self, Limite, OpcoesFolha, OpcoesJson, Tipo};
use crate::tiles_saida::{self, ExportTiles};

/// Export As.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExportAs {
    pub pedido: Pedido,
    pub formato: Formato,
    /// 0 = para sempre.
    pub repeat: u16,
}

/// Export Sprite Sheet; `json: None` = só a imagem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportFolha {
    pub opcoes: OpcoesFolha,
    pub json: Option<OpcoesJson>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Export {
    Ficheiro(ExportAs),
    Folha(ExportFolha),
    /// Os tiles em PNG e o mapa em JSON, ao lado.
    Tiles(ExportTiles),
}

/// O que ficou no disco.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gravado {
    pub ficheiros: Vec<PathBuf>,
    pub avisos: Vec<AvisoSaida>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErroGravar {
    Saida(ErroSaida),
    /// O disco recusou este ficheiro.
    Disco(PathBuf, String),
    /// A camada ou a tag que o último export escolheu já não existe.
    AlvoSumiu,
    /// As opções guardadas não se leem.
    Preset,
    /// Godot SpriteFrames pede uma peça por quadro (sem split).
    GodotSemQuadros,
}

/// `meta.pivots`: o pivô de cada quadro, em píxeis do sprite, para quem lê a folha com o `.tres`.
fn com_pivos(s: &Sprite, json: &str) -> String {
    let Ok(mut v) = serde_json::from_str::<serde_json::Value>(json) else {
        return json.to_owned();
    };
    let p: Vec<serde_json::Value> = crate::godot::pivos(s)
        .into_iter()
        .map(|(x, y)| serde_json::json!({ "x": x, "y": y }))
        .collect();
    if let Some(m) = v.get_mut("meta").and_then(|m| m.as_object_mut()) {
        m.insert("pivots".to_owned(), serde_json::Value::Array(p));
    }
    serde_json::to_string_pretty(&v).unwrap_or_else(|_| json.to_owned())
}

/// A camada e os quadros escolhidos ainda existem?
fn alvo_existe(s: &Sprite, c: Camadas, q: Quadros) -> bool {
    let camada = match c {
        Camadas::Uma(id) => s.layer(id).is_some(),
        _ => true,
    };
    let quadros = match q {
        Quadros::Tag(i) => i < s.tags().len(),
        _ => true,
    };
    camada && quadros
}

fn escreve(c: &Path, bytes: &[u8]) -> Result<(), ErroGravar> {
    // Por um ficheiro ao lado: uma falha a meio nunca deixa o destino cortado.
    let err = |e: std::io::Error| ErroGravar::Disco(c.to_owned(), e.to_string());
    let mut tmp = c.as_os_str().to_owned();
    tmp.push(".tmp");
    std::fs::write(&tmp, bytes).map_err(err)?;
    std::fs::rename(&tmp, c).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        err(e)
    })
}

/// O caminho de um ficheiro da sequência: `nome1.png` a partir de `nome.png`.
fn com_sufixo(caminho: &Path, sufixo: &str) -> PathBuf {
    if sufixo.is_empty() {
        return caminho.to_owned();
    }
    let raiz = caminho.file_stem().unwrap_or_default().to_string_lossy();
    let mut nome = format!("{raiz}{sufixo}");
    if let Some(e) = caminho.extension() {
        nome.push('.');
        nome.push_str(&e.to_string_lossy());
    }
    caminho.with_file_name(nome)
}

/// **Grava.** A folha leva o `.json` ao lado, com o mesmo nome.
///
/// # Errors
/// [`ErroGravar`]; nada é gravado se o pedido não se pode cumprir.
pub fn gravar(s: &Sprite, e: &Export, caminho: &Path) -> Result<Gravado, ErroGravar> {
    gravar_com(s, e, caminho, &|_| {})
}

/// [`gravar`], dizendo a fração feita (a barra: exportar corre fora da linha da tela).
///
/// # Errors
/// [`ErroGravar`]; nada é gravado se o pedido não se pode cumprir.
pub fn gravar_com(
    s: &Sprite,
    e: &Export,
    caminho: &Path,
    feito: &dyn Fn(f32),
) -> Result<Gravado, ErroGravar> {
    match e {
        Export::Ficheiro(x) => {
            let area_existe = match x.pedido.area {
                Area::Canvas => true,
                Area::Slice(id) => s.slice(id).is_some(),
            };
            if !area_existe || !alvo_existe(s, x.pedido.camadas, x.pedido.quadros) {
                return Err(ErroGravar::AlvoSumiu);
            }
            let r = saida::exportar_com(s, &x.pedido, x.formato, x.repeat, feito)
                .map_err(ErroGravar::Saida)?;
            let mut ficheiros = Vec::with_capacity(r.ficheiros.len());
            for (sufixo, bytes) in &r.ficheiros {
                let c = com_sufixo(caminho, sufixo);
                escreve(&c, bytes)?;
                ficheiros.push(c);
            }
            Ok(Gravado {
                ficheiros,
                avisos: r.avisos,
            })
        }
        Export::Folha(x) => {
            if !alvo_existe(s, x.opcoes.camadas, x.opcoes.quadros) {
                return Err(ErroGravar::AlvoSumiu);
            }
            let titulo = caminho.file_stem().unwrap_or_default().to_string_lossy();
            let f = sheet::montar_com(s, &x.opcoes, &titulo, "png", feito);
            let (modo, dados) = saida::png_do_documento(s, &f.rgba);
            let png = ph2d_imageio_png::encode_pixel(f.w, f.h, &modo, &dados)
                .map_err(|e| ErroGravar::Saida(ErroSaida::Formato(e.to_string())))?;
            escreve(caminho, &png)?;
            let mut ficheiros = vec![caminho.to_owned()];
            let imagem = caminho.file_name().unwrap_or_default().to_string_lossy();
            let normal = match &f.normal {
                Some(n) => {
                    let c = com_sufixo(caminho, "_n");
                    let png = ph2d_imageio_png::encode_pixel(
                        f.w,
                        f.h,
                        &ph2d_imageio_png::PixelPng::Rgba,
                        n,
                    )
                    .map_err(|e| ErroGravar::Saida(ErroSaida::Formato(e.to_string())))?;
                    escreve(&c, &png)?;
                    let nome = c
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into_owned();
                    ficheiros.push(c);
                    Some(nome)
                }
                None => None,
            };
            if let Some(j) = x.json {
                let t = sheet::json_da_folha(s, &f, &imagem, saida::formato_json(&modo), j);
                // O pivô por quadro sai sempre que o artista escolheu um (o import o lê de volta).
                let t = if x.opcoes.godot || s.pivot_slice().is_some() {
                    com_pivos(s, &t)
                } else {
                    t
                };
                let c = caminho.with_extension("json");
                escreve(&c, t.as_bytes())?;
                ficheiros.push(c);
            }
            if x.opcoes.godot {
                let tres = caminho.with_extension("tres");
                let lista = if f.de_onde.iter().any(|d| d.1.is_some()) {
                    crate::godot::tres_por_slice(s, &f, &imagem, normal.as_deref()).map(|v| {
                        v.into_iter()
                            .map(|(r, t)| (format!("_{}", r.replace(['/', '\\'], "_")), t))
                            .collect()
                    })
                } else {
                    crate::godot::tres(s, &f, &imagem, normal.as_deref())
                        .map(|t| vec![(String::new(), t)])
                };
                for (sufixo, t) in lista.ok_or(ErroGravar::GodotSemQuadros)? {
                    let c = com_sufixo(&tres, &sufixo);
                    escreve(&c, t.as_bytes())?;
                    ficheiros.push(c);
                }
            }
            Ok(Gravado {
                ficheiros,
                avisos: Vec::new(),
            })
        }
        Export::Tiles(x) => {
            let set = s.tileset(x.conjunto).ok_or(ErroGravar::AlvoSumiu)?;
            let (w, h, rgba) = tiles_saida::atlas(s, set, x.colunas);
            let (modo, dados) = saida::png_do_documento(s, &rgba);
            let png = ph2d_imageio_png::encode_pixel(w, h, &modo, &dados)
                .map_err(|e| ErroGravar::Saida(ErroSaida::Formato(e.to_string())))?;
            escreve(caminho, &png)?;
            let imagem = caminho.file_name().unwrap_or_default().to_string_lossy();
            let t = tiles_saida::json_do_mapa(s, set, &imagem, x.colunas);
            let c = caminho.with_extension("json");
            escreve(&c, t.as_bytes())?;
            feito(1.0);
            Ok(Gravado {
                ficheiros: vec![caminho.to_owned(), c],
                avisos: Vec::new(),
            })
        }
    }
}

/// **Repeat Last Export**: as opções guardadas no sprite, para o caminho guardado nele; `None` = o
/// sprite ainda não exportou.
///
/// # Errors
/// [`ErroGravar`].
pub fn repetir(s: &Sprite) -> Option<Result<Gravado, ErroGravar>> {
    let (e, c) = o_ultimo(s)?;
    Some(match e {
        Some(e) => gravar(s, &e, &c),
        None => Err(ErroGravar::Preset),
    })
}

/// O último export deste sprite, lido; `None` por dentro = as opções não se leem.
#[must_use]
pub fn o_ultimo(s: &Sprite) -> Option<(Option<Export>, PathBuf)> {
    let p = s.export_preset()?;
    Some((de_texto(s, &p.options), PathBuf::from(&p.path)))
}

/// Guarda o export no sprite.
pub fn lembrar(s: &mut Sprite, e: &Export, caminho: &Path) {
    let options = para_texto(s, e);
    s.set_export_preset(Some(ExportPreset {
        path: caminho.display().to_string(),
        options,
    }));
}

/// Um nome numa linha do texto: um nome com quebra de linha não vira outro campo.
fn esc(t: &str) -> String {
    t.replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

fn sem_esc(t: &str) -> String {
    let mut out = String::with_capacity(t.len());
    let mut it = t.chars();
    while let Some(c) = it.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match it.next() {
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some(o) => out.push(o),
            None => {}
        }
    }
    out
}

fn formato_txt(f: Formato) -> &'static str {
    match f {
        Formato::Png => "png",
        Formato::Gif => "gif",
        Formato::Apng => "apng",
        Formato::Webp => "webp",
        Formato::Ase => "ase",
    }
}

fn direcao_txt(d: Direcao) -> &'static str {
    match d {
        Direcao::Forward => "forward",
        Direcao::Reverse => "reverse",
        Direcao::PingPong => "pingpong",
        Direcao::PingPongReverse => "pingpong_reverse",
    }
}

fn tipo_txt(t: Tipo) -> &'static str {
    match t {
        Tipo::Horizontal => "horizontal",
        Tipo::Vertical => "vertical",
        Tipo::Rows => "rows",
        Tipo::Columns => "columns",
        Tipo::Packed => "packed",
    }
}

fn camadas_txt(c: Camadas) -> String {
    match c {
        Camadas::Visiveis => "visible".to_owned(),
        Camadas::Todas => "all".to_owned(),
        Camadas::Uma(id) => format!("layer {}", id.0),
    }
}

/// A tag vai pelo NOME: inserir outra tag muda os índices, e o Repeat tem de exportar a mesma.
fn quadros_txt(s: &Sprite, q: Quadros) -> String {
    match q {
        Quadros::Todos => "all".to_owned(),
        Quadros::Tag(i) => format!(
            "tag {}",
            s.tags().get(i).map_or_else(String::new, |t| esc(&t.name))
        ),
        Quadros::Intervalo { de, ate } => format!("range {de} {ate}"),
    }
}

fn para_texto(s: &Sprite, e: &Export) -> String {
    let mut l: Vec<String> = Vec::new();
    match e {
        Export::Ficheiro(x) => {
            l.push("kind=file".to_owned());
            l.push(format!("format={}", formato_txt(x.formato)));
            l.push(format!("layers={}", camadas_txt(x.pedido.camadas)));
            l.push(format!("frames={}", quadros_txt(s, x.pedido.quadros)));
            l.push(format!("direction={}", direcao_txt(x.pedido.direcao)));
            l.push(format!("scale={}", x.pedido.escala));
            if let Area::Slice(id) = x.pedido.area {
                l.push(format!("area={}", id.0));
            }
            l.push(format!("repeat={}", x.repeat));
        }
        Export::Folha(x) => {
            let o = &x.opcoes;
            l.push("kind=sheet".to_owned());
            l.push(format!("type={}", tipo_txt(o.tipo)));
            l.push(format!(
                "limit={}",
                match o.limite {
                    Limite::Nenhum => "none".to_owned(),
                    Limite::Colunas(n) => format!("columns {n}"),
                    Limite::Linhas(n) => format!("rows {n}"),
                    Limite::Largura(n) => format!("width {n}"),
                    Limite::Altura(n) => format!("height {n}"),
                }
            ));
            l.push(format!("layers={}", camadas_txt(o.camadas)));
            l.push(format!("frames={}", quadros_txt(s, o.quadros)));
            for (k, v) in [
                ("pow2", o.potencia_de_2),
                ("merge", o.merge_duplicates),
                ("ignore_empty", o.ignore_empty),
                ("split_layers", o.split_layers),
                ("split_tags", o.split_tags),
                ("split_slices", o.split_slices),
                ("split_grid", o.split_grid),
                ("trim_by_grid", o.trim_by_grid),
                ("trim_cels", o.trim_cels),
                ("trim_sprite", o.trim_sprite),
            ] {
                l.push(format!("{k}={}", u8::from(v)));
            }
            for (k, v) in [
                ("extrude", o.extrude),
                ("border", o.border),
                ("shape", o.shape),
                ("inner", o.inner),
            ] {
                l.push(format!("{k}={v}"));
            }
            if let Some(n) = &o.nome {
                l.push(format!("name={}", esc(n)));
            }
            l.push(format!("normal={}", u8::from(o.normal)));
            l.push(format!("godot={}", u8::from(o.godot)));
            if let Some(j) = x.json {
                l.push(format!(
                    "json={} {} {} {}",
                    u8::from(j.array),
                    u8::from(j.tags),
                    u8::from(j.layers),
                    u8::from(j.slices)
                ));
            }
        }
        Export::Tiles(x) => {
            l.push("kind=tiles".to_owned());
            l.push(format!("tileset={}", x.conjunto.0));
            l.push(format!("columns={}", x.colunas));
        }
    }
    l.join("\n")
}

fn de_camadas(v: &str) -> Option<Camadas> {
    Some(match v {
        "visible" => Camadas::Visiveis,
        "all" => Camadas::Todas,
        o => Camadas::Uma(LayerId(o.strip_prefix("layer ")?.parse().ok()?)),
    })
}

fn de_quadros(s: &Sprite, v: &str) -> Option<Quadros> {
    if v == "all" {
        return Some(Quadros::Todos);
    }
    if let Some(n) = v.strip_prefix("tag ") {
        // Tag apagada: um índice que não existe, e o `gravar` recusa.
        let n = sem_esc(n);
        let i = s.tags().iter().position(|t| t.name == n);
        return Some(Quadros::Tag(i.unwrap_or(usize::MAX)));
    }
    let (de, ate) = v.strip_prefix("range ")?.split_once(' ')?;
    Some(Quadros::Intervalo {
        de: de.parse().ok()?,
        ate: ate.parse().ok()?,
    })
}

fn de_limite(v: &str) -> Option<Limite> {
    if v == "none" {
        return Some(Limite::Nenhum);
    }
    let (k, n) = v.split_once(' ')?;
    let n = n.parse().ok()?;
    Some(match k {
        "columns" => Limite::Colunas(n),
        "rows" => Limite::Linhas(n),
        "width" => Limite::Largura(n),
        "height" => Limite::Altura(n),
        _ => return None,
    })
}

fn de_texto(s: &Sprite, t: &str) -> Option<Export> {
    let campo = |k: &str| {
        t.lines()
            .find_map(|l| l.split_once('=').filter(|(a, _)| *a == k).map(|(_, v)| v))
    };
    let num = |k: &str| campo(k)?.parse::<u32>().ok();
    let sim = |k: &str| campo(k) == Some("1");
    match campo("kind")? {
        "tiles" => Some(Export::Tiles(ExportTiles {
            conjunto: ph2d_pixel_core::TilesetId(num("tileset")?),
            colunas: num("columns")?,
        })),
        "file" => Some(Export::Ficheiro(ExportAs {
            pedido: Pedido {
                camadas: de_camadas(campo("layers")?)?,
                quadros: de_quadros(s, campo("frames")?)?,
                direcao: match campo("direction")? {
                    "forward" => Direcao::Forward,
                    "reverse" => Direcao::Reverse,
                    "pingpong" => Direcao::PingPong,
                    "pingpong_reverse" => Direcao::PingPongReverse,
                    _ => return None,
                },
                escala: campo("scale")?.parse().ok()?,
                area: match campo("area") {
                    None => Area::Canvas,
                    Some(id) => Area::Slice(ph2d_pixel_core::SliceId(id.parse().ok()?)),
                },
            },
            formato: Formato::de_extensao(campo("format")?)?,
            repeat: campo("repeat")?.parse().ok()?,
        })),
        "sheet" => {
            let json = match campo("json") {
                Some(v) => {
                    let b: Vec<bool> = v.split(' ').map(|x| x == "1").collect();
                    let [array, tags, layers, slices] = <[bool; 4]>::try_from(b).ok()?;
                    Some(OpcoesJson {
                        array,
                        tags,
                        layers,
                        slices,
                    })
                }
                None => None,
            };
            Some(Export::Folha(ExportFolha {
                opcoes: OpcoesFolha {
                    tipo: match campo("type")? {
                        "horizontal" => Tipo::Horizontal,
                        "vertical" => Tipo::Vertical,
                        "rows" => Tipo::Rows,
                        "columns" => Tipo::Columns,
                        "packed" => Tipo::Packed,
                        _ => return None,
                    },
                    limite: de_limite(campo("limit")?)?,
                    potencia_de_2: sim("pow2"),
                    merge_duplicates: sim("merge"),
                    ignore_empty: sim("ignore_empty"),
                    camadas: de_camadas(campo("layers")?)?,
                    quadros: de_quadros(s, campo("frames")?)?,
                    split_layers: sim("split_layers"),
                    split_tags: sim("split_tags"),
                    split_slices: sim("split_slices"),
                    split_grid: sim("split_grid"),
                    trim_by_grid: sim("trim_by_grid"),
                    trim_cels: sim("trim_cels"),
                    trim_sprite: sim("trim_sprite"),
                    extrude: num("extrude")?,
                    border: num("border")?,
                    shape: num("shape")?,
                    inner: num("inner")?,
                    nome: campo("name").map(sem_esc),
                    normal: sim("normal"),
                    godot: sim("godot"),
                },
                json,
            }))
        }
        _ => None,
    }
}
