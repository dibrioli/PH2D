//! **Os gates do leitor de `.ase`**, sobre ficheiros **escritos aqui**.
//!
//! ⚠️ **Um ESCRITOR no teste, e não um `.ase` binário no repo**, por três razões medidas:
//! 1. um blob binário não diz o que contém — daqui a seis meses ninguém sabe se o ficheiro é que
//!    está errado ou o leitor;
//! 2. um caso novo (uma camada escondida, um cel ligado, uma paleta) pede um ficheiro novo, e
//!    pedi-lo ao Aseprite exige o Aseprite;
//! 3. o escritor **é** a spec escrita em código: cada campo aparece uma vez, com o nome que a
//!    especificação lhe dá, e um desalinhamento de um byte parte tudo de uma vez.
//!
//! ⛔ O que ele NÃO é: um oráculo. Ele afirma que sabemos ler o que descrevemos — a fidelidade ao
//! Aseprite real vem do smoke, com um ficheiro do artista.

use ph2d_aseprite::{AseError, parse};

// ─── O escritor: a spec, em código ───

#[derive(Default)]
struct Chunk {
    kind: u16,
    body: Vec<u8>,
}

struct Frame {
    duration_ms: u16,
    chunks: Vec<Chunk>,
}

struct Ase {
    width: u16,
    height: u16,
    depth: u16,
    transparent_index: u8,
    frames: Vec<Frame>,
}

fn w16(v: &mut Vec<u8>, x: u16) {
    v.extend_from_slice(&x.to_le_bytes());
}
fn w32(v: &mut Vec<u8>, x: u32) {
    v.extend_from_slice(&x.to_le_bytes());
}
fn wstr(v: &mut Vec<u8>, s: &str) {
    w16(v, s.len() as u16);
    v.extend_from_slice(s.as_bytes());
}

impl Ase {
    fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            depth: 32,
            transparent_index: 0,
            frames: Vec::new(),
        }
    }

    fn bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        w32(&mut out, 0); // tamanho do ficheiro: o leitor nao confia nele, de proposito
        w16(&mut out, 0xA5E0);
        w16(&mut out, self.frames.len() as u16);
        w16(&mut out, self.width);
        w16(&mut out, self.height);
        w16(&mut out, self.depth);
        w32(&mut out, 0); // flags
        w16(&mut out, 100); // speed, obsoleto
        w32(&mut out, 0);
        w32(&mut out, 0);
        out.push(self.transparent_index);
        out.extend_from_slice(&[0; 3]);
        w16(&mut out, 0); // numero de cores
        out.resize(128, 0); // o resto do cabecalho e' reservado
        for f in &self.frames {
            let mut body = Vec::new();
            for c in &f.chunks {
                w32(&mut body, (c.body.len() + 6) as u32);
                w16(&mut body, c.kind);
                body.extend_from_slice(&c.body);
            }
            w32(&mut out, (body.len() + 16) as u32);
            w16(&mut out, 0xF1FA);
            w16(&mut out, 0); // contador velho: zero manda usar o novo
            w16(&mut out, f.duration_ms);
            out.extend_from_slice(&[0; 2]);
            w32(&mut out, f.chunks.len() as u32);
            out.extend_from_slice(&body);
        }
        out
    }
}

/// Uma camada normal, visível, opaca.
fn layer(name: &str) -> Chunk {
    layer_full(name, true, 0, 0, 255, 0)
}

fn layer_full(name: &str, visible: bool, kind: u16, child_level: u16, opacity: u8, blend: u16) -> Chunk {
    let mut b = Vec::new();
    w16(&mut b, u16::from(visible)); // flags: bit 0 = visivel
    w16(&mut b, kind); // 0 imagem · 1 grupo · 2 tilemap
    w16(&mut b, child_level);
    w16(&mut b, 0); // largura default, ignorada
    w16(&mut b, 0); // altura default, ignorada
    w16(&mut b, blend);
    b.push(opacity);
    b.extend_from_slice(&[0; 3]);
    wstr(&mut b, name);
    Chunk { kind: 0x2004, body: b }
}

/// Um cel de imagem crua (tipo 0) com `w × h` pixels RGBA8.
fn cel(layer_index: u16, x: i16, y: i16, w: u16, h: u16, rgba: &[u8]) -> Chunk {
    cel_kind(layer_index, x, y, 255, 0, w, h, rgba.to_vec())
}

/// O mesmo cel, comprimido (tipo 2) — o caso normal de um ficheiro real.
fn cel_zlib(layer_index: u16, x: i16, y: i16, w: u16, h: u16, rgba: &[u8]) -> Chunk {
    let z = miniz_oxide::deflate::compress_to_vec_zlib(rgba, 6);
    cel_kind(layer_index, x, y, 255, 2, w, h, z)
}

fn cel_kind(
    layer_index: u16,
    x: i16,
    y: i16,
    opacity: u8,
    kind: u16,
    w: u16,
    h: u16,
    payload: Vec<u8>,
) -> Chunk {
    let mut b = Vec::new();
    w16(&mut b, layer_index);
    w16(&mut b, x as u16);
    w16(&mut b, y as u16);
    b.push(opacity);
    w16(&mut b, kind);
    b.extend_from_slice(&[0; 7]); // z-index (1.3) + reservados
    w16(&mut b, w);
    w16(&mut b, h);
    b.extend_from_slice(&payload);
    Chunk { kind: 0x2005, body: b }
}

/// Um cel LIGADO (tipo 1): «repete o que esta camada tem no quadro N».
fn cel_link(layer_index: u16, src_frame: u16) -> Chunk {
    let mut b = Vec::new();
    w16(&mut b, layer_index);
    w16(&mut b, 0);
    w16(&mut b, 0);
    b.push(255);
    w16(&mut b, 1);
    b.extend_from_slice(&[0; 7]);
    w16(&mut b, src_frame);
    Chunk { kind: 0x2005, body: b }
}

fn tags(list: &[(&str, u16, u16, u8, u16)]) -> Chunk {
    let mut b = Vec::new();
    w16(&mut b, list.len() as u16);
    b.extend_from_slice(&[0; 8]);
    for (name, from, to, dir, repeat) in list {
        w16(&mut b, *from);
        w16(&mut b, *to);
        b.push(*dir);
        w16(&mut b, *repeat);
        b.extend_from_slice(&[0; 6]);
        b.extend_from_slice(&[0; 3]); // cor obsoleta
        b.push(0);
        wstr(&mut b, name);
    }
    Chunk { kind: 0x2018, body: b }
}

fn palette(colors: &[[u8; 4]]) -> Chunk {
    let mut b = Vec::new();
    w32(&mut b, colors.len() as u32);
    w32(&mut b, 0);
    w32(&mut b, (colors.len() - 1) as u32);
    b.extend_from_slice(&[0; 8]);
    for c in colors {
        w16(&mut b, 0);
        b.extend_from_slice(c);
    }
    Chunk { kind: 0x2019, body: b }
}

const RED: [u8; 4] = [255, 0, 0, 255];
const BLUE: [u8; 4] = [0, 0, 255, 255];

fn solid(n: usize, c: [u8; 4]) -> Vec<u8> {
    c.iter().copied().cycle().take(n * 4).collect()
}

fn px(doc: &ph2d_aseprite::AseDoc, frame: usize, x: usize, y: usize) -> [u8; 4] {
    let i = (y * usize::from(doc.width) + x) * 4;
    let p = &doc.frames[frame].rgba[i..i + 4];
    [p[0], p[1], p[2], p[3]]
}

// ─── Os gates ───

/// **O caso mínimo: um quadro, uma camada, um cel.** Se este falhar, todo o resto é ruído.
#[test]
fn a_one_frame_file_reads_its_pixels() {
    let mut a = Ase::new(2, 2);
    a.frames.push(Frame {
        duration_ms: 83,
        chunks: vec![layer("Layer 1"), cel(0, 0, 0, 2, 2, &solid(4, RED))],
    });
    let doc = parse(&a.bytes()).expect("o ficheiro minimo tem de ler");
    assert_eq!((doc.width, doc.height), (2, 2));
    assert_eq!(doc.frames.len(), 1);
    assert_eq!(doc.frames[0].duration_ms, 83);
    assert_eq!(px(&doc, 0, 0, 0), RED);
    assert_eq!(px(&doc, 0, 1, 1), RED);
    assert!(doc.notes.is_empty(), "nada ficou por tras: {:?}", doc.notes);
}

/// **Um cel comprimido lê igual a um cru.** É a forma que TODO ficheiro real usa; um erro de zlib
/// aqui daria «o Aseprite exporta em branco».
#[test]
fn a_compressed_cel_reads_the_same_as_a_raw_one() {
    let pixels = solid(4, BLUE);
    let mut raw = Ase::new(2, 2);
    raw.frames.push(Frame {
        duration_ms: 100,
        chunks: vec![layer("L"), cel(0, 0, 0, 2, 2, &pixels)],
    });
    let mut zip = Ase::new(2, 2);
    zip.frames.push(Frame {
        duration_ms: 100,
        chunks: vec![layer("L"), cel_zlib(0, 0, 0, 2, 2, &pixels)],
    });
    assert_eq!(
        parse(&raw.bytes()).unwrap(),
        parse(&zip.bytes()).unwrap(),
        "o cel comprimido tem de dar exactamente o mesmo quadro"
    );
}

/// **UM CEL LIGADO REPETE O QUADRO QUE ELE APONTA.** ⚠️ É o modo de falha mais caro deste formato:
/// o Aseprite guarda um quadro não-redesenhado como uma referência, e tratá-la como ausente faz a
/// animação **piscar** exactamente nos quadros que o artista deixou como estavam — que são a
/// maioria.
#[test]
fn a_linked_cel_repeats_the_frame_it_points_at() {
    let mut a = Ase::new(1, 1);
    a.frames.push(Frame {
        duration_ms: 100,
        chunks: vec![layer("L"), cel(0, 0, 0, 1, 1, &solid(1, RED))],
    });
    a.frames.push(Frame {
        duration_ms: 100,
        chunks: vec![cel_link(0, 0)],
    });
    let doc = parse(&a.bytes()).unwrap();
    assert_eq!(doc.frames.len(), 2);
    assert_eq!(px(&doc, 1, 0, 0), RED, "o 2o quadro piscou em vez de repetir");
}

/// **Uma camada escondida não desenha — e um GRUPO escondido esconde os filhos.** A segunda metade
/// é a que se esquece: os filhos de um grupo são camadas irmãs no ficheiro, com um nível a mais.
#[test]
fn hidden_layers_and_hidden_groups_do_not_draw() {
    let mut a = Ase::new(1, 1);
    a.frames.push(Frame {
        duration_ms: 100,
        chunks: vec![
            layer_full("hidden", false, 0, 0, 255, 0),
            cel(0, 0, 0, 1, 1, &solid(1, RED)),
        ],
    });
    assert_eq!(
        px(&parse(&a.bytes()).unwrap(), 0, 0, 0),
        [0, 0, 0, 0],
        "uma camada escondida desenhou"
    );

    let mut g = Ase::new(1, 1);
    g.frames.push(Frame {
        duration_ms: 100,
        chunks: vec![
            layer_full("group", false, 1, 0, 255, 0), // grupo ESCONDIDO
            layer_full("child", true, 0, 1, 255, 0),  // filho visivel, dentro dele
            cel(1, 0, 0, 1, 1, &solid(1, RED)),
        ],
    });
    assert_eq!(
        px(&parse(&g.bytes()).unwrap(), 0, 0, 0),
        [0, 0, 0, 0],
        "o filho de um grupo escondido desenhou"
    );

    // CONTROLO POSITIVO: com o grupo VISÍVEL o mesmo filho aparece — senão este gate ficaria
    // verde numa implementação que nunca desenha nada.
    let mut ok = Ase::new(1, 1);
    ok.frames.push(Frame {
        duration_ms: 100,
        chunks: vec![
            layer_full("group", true, 1, 0, 255, 0),
            layer_full("child", true, 0, 1, 255, 0),
            cel(1, 0, 0, 1, 1, &solid(1, RED)),
        ],
    });
    assert_eq!(px(&parse(&ok.bytes()).unwrap(), 0, 0, 0), RED);
}

/// **A pilha é a ordem do ficheiro, de baixo para cima.** Compor ao contrário dá um desenho
/// plausível e errado — e num ficheiro de uma camada só ninguém repara.
#[test]
fn the_layer_order_in_the_file_is_bottom_to_top() {
    let mut a = Ase::new(1, 1);
    a.frames.push(Frame {
        duration_ms: 100,
        chunks: vec![
            layer("bottom"),
            layer("top"),
            cel(0, 0, 0, 1, 1, &solid(1, RED)),
            cel(1, 0, 0, 1, 1, &solid(1, BLUE)),
        ],
    });
    assert_eq!(
        px(&parse(&a.bytes()).unwrap(), 0, 0, 0),
        BLUE,
        "a camada de CIMA e' a ultima do ficheiro"
    );
}

/// **Um cel meio fora da tela é recortado, não descartado nem invadido.** O Aseprite guarda só o
/// rectângulo desenhado, e ele pode ficar parcialmente fora depois de o artista mexer na tela.
#[test]
fn a_cel_partly_outside_the_canvas_is_clipped() {
    let mut a = Ase::new(2, 2);
    a.frames.push(Frame {
        duration_ms: 100,
        chunks: vec![layer("L"), cel(0, 1, 1, 2, 2, &solid(4, RED))],
    });
    let doc = parse(&a.bytes()).unwrap();
    assert_eq!(px(&doc, 0, 1, 1), RED, "o pixel de dentro tinha de entrar");
    assert_eq!(px(&doc, 0, 0, 0), [0, 0, 0, 0], "e nada mais");

    // E um cel INTEIRAMENTE fora não escreve nada nem entra em pânico.
    let mut out = Ase::new(2, 2);
    out.frames.push(Frame {
        duration_ms: 100,
        chunks: vec![layer("L"), cel(0, 90, 90, 2, 2, &solid(4, RED))],
    });
    assert!(parse(&out.bytes()).unwrap().frames[0].rgba.iter().all(|&b| b == 0));
}

/// **As tags voltam inteiras** — nome, intervalo, direcção e repetições. É por isto que o `.ase`
/// foi pedido: o par `.png`+`.json` traz rectângulos, este traz a AUTORIA.
#[test]
fn the_tags_come_back_with_name_range_direction_and_repeat() {
    let mut a = Ase::new(1, 1);
    a.frames.push(Frame {
        duration_ms: 100,
        chunks: vec![
            layer("L"),
            cel(0, 0, 0, 1, 1, &solid(1, RED)),
            tags(&[("idle", 0, 3, 0, 0), ("hit", 4, 6, 2, 1)]),
        ],
    });
    let doc = parse(&a.bytes()).unwrap();
    assert_eq!(doc.tags.len(), 2);
    assert_eq!(doc.tags[0].name, "idle");
    assert_eq!((doc.tags[0].from, doc.tags[0].to), (0, 3));
    assert_eq!(doc.tags[0].direction, 0);
    assert_eq!(doc.tags[0].repeat, 0, "zero = para sempre");
    assert_eq!(doc.tags[1].name, "hit");
    assert_eq!(doc.tags[1].direction, 2, "vai-e-volta");
    assert_eq!(doc.tags[1].repeat, 1);
}

/// **Um ficheiro indexado lê pela paleta, e o índice transparente é BURACO.** Sem a segunda
/// metade, o fundo de todo ficheiro indexado vira uma cor sólida.
#[test]
fn an_indexed_file_reads_through_its_palette() {
    let mut a = Ase::new(2, 1);
    a.depth = 8;
    a.transparent_index = 0;
    a.frames.push(Frame {
        duration_ms: 100,
        chunks: vec![
            palette(&[[9, 9, 9, 255], BLUE]),
            layer("L"),
            cel(0, 0, 0, 2, 1, &[0, 1]),
        ],
    });
    let doc = parse(&a.bytes()).unwrap();
    assert_eq!(px(&doc, 0, 0, 0), [0, 0, 0, 0], "o indice transparente e' buraco");
    assert_eq!(px(&doc, 0, 1, 0), BLUE, "e o resto sai da paleta");
}

/// **Escala de cinza (16 bpp) espalha o cinza pelos três canais** e mantém o alfa.
#[test]
fn a_greyscale_file_reads_grey_plus_alpha() {
    let mut a = Ase::new(1, 1);
    a.depth = 16;
    a.frames.push(Frame {
        duration_ms: 100,
        chunks: vec![layer("L"), cel(0, 0, 0, 1, 1, &[120, 200])],
    });
    assert_eq!(px(&parse(&a.bytes()).unwrap(), 0, 0, 0), [120, 120, 120, 200]);
}

/// **Um tilemap sai numa NOTA, não em silêncio.** ⛔ Um importador que ignora sem dizer é pior que
/// um que recusa: o desenho aparece quase certo e ninguém sabe porquê.
#[test]
fn a_tilemap_layer_says_so_instead_of_vanishing_quietly() {
    let mut a = Ase::new(1, 1);
    a.frames.push(Frame {
        duration_ms: 100,
        chunks: vec![
            layer_full("tiles", true, 2, 0, 255, 0),
            cel_kind(0, 0, 0, 255, 3, 1, 1, vec![0; 8]),
        ],
    });
    let doc = parse(&a.bytes()).unwrap();
    assert_eq!(doc.notes.len(), 1, "esperava UMA nota: {:?}", doc.notes);
    assert!(doc.notes[0].contains("tiles"), "a nota tem de NOMEAR a camada");
}

/// **Um chunk desconhecido é saltado, não fatal.** O formato cresce a cada versão do Aseprite, e
/// recusar o ficheiro por um chunk novo faria o importador envelhecer sozinho.
#[test]
fn an_unknown_chunk_is_skipped() {
    let mut a = Ase::new(1, 1);
    a.frames.push(Frame {
        duration_ms: 100,
        chunks: vec![
            Chunk { kind: 0x7777, body: vec![1, 2, 3, 4, 5] },
            layer("L"),
            cel(0, 0, 0, 1, 1, &solid(1, RED)),
        ],
    });
    assert_eq!(px(&parse(&a.bytes()).unwrap(), 0, 0, 0), RED);
}

/// **Lixo e ficheiros cortados dão ERRO NOMEADO, nunca pânico** — a entrada é um ficheiro que o
/// utilizador largou na janela.
#[test]
fn broken_input_is_a_named_error_never_a_panic() {
    assert_eq!(parse(&[]), Err(AseError::Truncated("header")));
    assert_eq!(parse(&[0; 128]), Err(AseError::NotAseprite));

    let mut a = Ase::new(2, 2);
    a.frames.push(Frame {
        duration_ms: 100,
        chunks: vec![layer("L"), cel(0, 0, 0, 2, 2, &solid(4, RED))],
    });
    let full = a.bytes();
    // Cortar em CADA sítio: nenhum pode entrar em pânico.
    for cut in 0..full.len() {
        let _ = parse(&full[..cut]);
    }

    let mut empty = Ase::new(2, 2);
    empty.frames.clear();
    assert_eq!(parse(&empty.bytes()), Err(AseError::NoFrames));

    let mut zero = Ase::new(0, 5);
    zero.frames.push(Frame { duration_ms: 1, chunks: vec![] });
    assert_eq!(parse(&zero.bytes()), Err(AseError::EmptyCanvas));

    let mut deep = Ase::new(1, 1);
    deep.depth = 24;
    deep.frames.push(Frame { duration_ms: 1, chunks: vec![] });
    assert_eq!(parse(&deep.bytes()), Err(AseError::UnknownColorDepth(24)));
}

/// **A DURAÇÃO POR-QUADRO chega inteira, e a tag sabe dizer quando ela NÃO é uma só.**
///
/// ⚠️ É a recusa medida que este formato reabre (spec §8.12): a §11 guarda **um** `frame_ms` por
/// tag. `uniform_duration_ms` devolver `None` **é a informação** — quem chama decide se aproxima
/// (e avisa) ou se abre a wave que põe a duração por-quadro no modelo.
#[test]
fn per_frame_durations_survive_and_the_tag_can_tell() {
    let mut a = Ase::new(1, 1);
    for ms in [50_u16, 50, 200, 50] {
        a.frames.push(Frame {
            duration_ms: ms,
            chunks: vec![],
        });
    }
    a.frames[0].chunks.push(layer("L"));
    a.frames[0].chunks.push(tags(&[
        ("flat", 0, 1, 0, 0),
        ("bumpy", 0, 3, 0, 0),
    ]));
    let doc = parse(&a.bytes()).unwrap();
    let ms: Vec<u16> = doc.frames.iter().map(|f| f.duration_ms).collect();
    assert_eq!(ms, vec![50, 50, 200, 50]);
    assert_eq!(doc.tags[0].uniform_duration_ms(&doc.frames), Some(50));
    assert_eq!(
        doc.tags[1].uniform_duration_ms(&doc.frames),
        None,
        "a tag com um HOLD no meio nao tem uma duracao so'"
    );
    assert_eq!(
        doc.tags[1].dominant_duration_ms(&doc.frames),
        50,
        "e a aproximacao honesta e' a mais comum"
    );
}

/// **A OPACIDADE DA CAMADA E A DO CEL MULTIPLICAM-SE** — são dois números diferentes no formato, e
/// o Aseprite mostra-os em dois sítios diferentes da UI.
///
/// ⚠️ Este gate nasceu de uma **mutação que sobreviveu**: pôr a opacidade a `255` fixo passava a
/// suíte inteira. Os outros gates usam sempre camadas opacas, e uma cobertura que só vê o caso
/// neutro não vê o cálculo nenhum.
#[test]
fn the_layer_opacity_and_the_cel_opacity_multiply() {
    let half = |layer_op: u8, cel_op: u8| {
        let mut a = Ase::new(1, 1);
        a.frames.push(Frame {
            duration_ms: 100,
            chunks: vec![
                layer_full("L", true, 0, 0, layer_op, 0),
                cel_kind(0, 0, 0, cel_op, 0, 1, 1, RED.to_vec()),
            ],
        });
        px(&parse(&a.bytes()).unwrap(), 0, 0, 0)[3]
    };
    assert_eq!(half(255, 255), 255, "opaco com opaco fica opaco");
    assert_eq!(half(128, 255), 128, "a opacidade da CAMADA chega ao alfa");
    assert_eq!(half(255, 128), 128, "a do CEL tambem");
    // 128·128/255 ≈ 64: o produto, e não a menor das duas nem a soma.
    let both = half(128, 128);
    assert!(
        (63..=65).contains(&both),
        "as duas opacidades tinham de MULTIPLICAR: deu {both}, e 128x128 da' ~64"
    );
}

/// **Uma camada em Multiply mistura, e não só desenha por cima.** ⚠️ Sem este gate, o compositor
/// podia ignorar o campo de modo do ficheiro e passar tudo — os gates de `blend.rs` provam as
/// fórmulas, este prova que o número da CAMADA chega a elas.
#[test]
fn the_blend_mode_of_the_layer_reaches_the_pixels() {
    let mut a = Ase::new(1, 1);
    a.frames.push(Frame {
        duration_ms: 100,
        chunks: vec![
            layer("bottom"),
            layer_full("multiply", true, 0, 0, 255, 1), // 1 = Multiply
            cel(0, 0, 0, 1, 1, &solid(1, [255, 128, 0, 255])),
            cel(1, 0, 0, 1, 1, &solid(1, [128, 255, 255, 255])),
        ],
    });
    let got = px(&parse(&a.bytes()).unwrap(), 0, 0, 0);
    // 255x128 = 128 · 128x255 = 128 · 0x255 = 0.
    assert_eq!(
        got,
        [128, 128, 0, 255],
        "a camada Multiply desenhou por cima em vez de multiplicar"
    );
}
