//! **A BANCADA DO EXAME** (`docs/Painter/Pinceis/04_BANCADA.md`, missão 00 §3): a BATERIA de gestos
//! (`docs/Painter/Pinceis/ferramentas/bateria.json`, a fonte única — os mesmos pontos que o Rebelle, o Krita e o
//! MyPaint recebem) pintada pelos NOSSOS motores com qualquer pincel, e o custo dele a 60 Hz.
//!
//! ```text
//! PH2D_EXAME_PRESETS="Graphite 8B|Oil Bristle" PH2D_EXAME_OUT=/tmp/x \
//!   cargo test -p ph2d-tool-painter --profile smoke --test it exame_bancada::exame_pinta -- --ignored --nocapture
//! ```
//! O comando de uso é `docs/Painter/Pinceis/ferramentas/exame.sh` (a régua, a matriz e as pranchas são Python).
//!
//! Um pincel (`PH2D_EXAME_PRESETS`, separados por `|`): o NOME de um preset da biblioteca · o caminho de um
//! `.ph2d-brush.png` · `meio:Digital|Watercolor|Impasto|WetPaint` (o pincel de fábrica do meio, receita em código:
//! [`RECEITAS_DE_MEIO`]) · `todos` (a biblioteca inteira). `PH2D_EXAME_EXTRA=<raiz>` junta uma 2.ª raiz de
//! presets (a de utilizador do `Biblioteca::carregar`). `PH2D_EXAME_ESCALA` multiplica o diâmetro de cada traço
//! (1 = o `size_px` da bateria; a largura que se vê é a do slider, `wetpaint/largura.rs`).
//!
//! As ops: `simula N` = N quadros de 60 Hz; `seca` = o «Dry Canvas» do Wet Paint (o `DRY` do Rebelle), o «Dry now»
//! da Watercolor, e no Digital e no Impasto nada (não há água: a faixa «seca» é igual à «molhada»).

use ph2d_brush_presets::{Biblioteca, Preset};
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{
    CanvasPaintTool, CanvasPointer, PanelEvent, PointerPhase, RasterEditTool,
};
use ph2d_tool_painter::{PaintMedia, PainterTool, ids};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

/// Um pincel da bancada fora dos ficheiros: o meio e o que se ajusta nele por código.
pub struct ReceitaDeMeio {
    pub nome: &'static str,
    pub meio: PaintMedia,
    pub prepara: fn(&mut PainterTool),
}

fn nada(_: &mut PainterTool) {}

/// Os pincéis de fábrica de cada meio (o que `set_paint_media` dá). Os meios Digital, Watercolor e Impasto ainda
/// não têm presets em ficheiro: uma receita nova é uma linha aqui.
pub const RECEITAS_DE_MEIO: &[ReceitaDeMeio] = &[
    ReceitaDeMeio {
        nome: "meio:Digital",
        meio: PaintMedia::Digital,
        prepara: nada,
    },
    ReceitaDeMeio {
        nome: "meio:Watercolor",
        meio: PaintMedia::Watercolor,
        prepara: nada,
    },
    ReceitaDeMeio {
        nome: "meio:Impasto",
        meio: PaintMedia::Impasto,
        prepara: nada,
    },
    ReceitaDeMeio {
        nome: "meio:WetPaint",
        meio: PaintMedia::WetPaint,
        prepara: nada,
    },
];

pub enum Fonte {
    Preset {
        preset: Box<Preset>,
        relativo: String,
    },
    Meio(&'static ReceitaDeMeio),
}

pub struct Pincel {
    pub nome: String,
    pub fonte: Fonte,
}

impl Pincel {
    pub fn slug(&self) -> String {
        self.nome
            .chars()
            .map(|c| {
                if c.is_alphanumeric() {
                    c.to_ascii_lowercase()
                } else {
                    '_'
                }
            })
            .collect()
    }

    fn meio(&self) -> PaintMedia {
        match &self.fonte {
            // O meio do FICHEIRO (L1): um preset de qualquer meio; um desconhecido é recusado no `aplica`.
            Fonte::Preset { preset, .. } => ph2d_tool_painter::meio_do_preset(&preset.parametros.media)
                .unwrap_or(PaintMedia::WetPaint),
            Fonte::Meio(r) => r.meio,
        }
    }

    /// Uma FERRAMENTA que não pinta (J4 `mistura`): um modo que não é o Paint (Smear, Blur, as ferramentas molhadas
    /// Blend/Wet/Dry…) ou a borracha (`EraseAlpha`). Sobre o papel ela não deixa nada: as faixas da cena «sobre» são
    /// de outro pincel ([`base_padrao`]).
    pub fn nao_pinta(&self) -> bool {
        let Fonte::Preset { preset, .. } = &self.fonte else {
            return false;
        };
        let p = &preset.parametros;
        p.paint_mode.as_deref().is_some_and(|m| m != "Paint")
            || p.brush.get("blend").and_then(Value::as_str) == Some("EraseAlpha")
    }

    /// `relogio_fixo`: a água em passos síncronos (a mesma tinta duas vezes — as fotos); sem ele, a do ecrã (o custo).
    pub fn aplica(&self, t: &mut PainterTool, relogio_fixo: bool) {
        t.set_semente_dos_tracos(0);
        match &self.fonte {
            Fonte::Preset { preset, .. } => {
                t.set_wet_relogio_fixo(relogio_fixo);
                let falhas = t
                    .preset_aplicar(preset)
                    .unwrap_or_else(|e| panic!("{}: meio desconhecido {e:?}", self.nome));
                assert!(falhas.is_empty(), "{}: {falhas:?}", self.nome);
            }
            Fonte::Meio(r) => {
                t.set_paint_media(r.meio);
                if r.meio == PaintMedia::WetPaint {
                    t.set_wet_relogio_fixo(relogio_fixo);
                }
                t.set_brush_strength(1.0);
                (r.prepara)(t);
            }
        }
    }
}

fn raiz_da_fabrica() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes")
}

fn biblioteca() -> Biblioteca {
    let extra = std::env::var("PH2D_EXAME_EXTRA").ok().map(PathBuf::from);
    let b = Biblioteca::carregar(&raiz_da_fabrica(), extra.as_deref());
    assert!(b.erros.is_empty(), "presets ilegíveis: {:?}", b.erros);
    b
}

/// Os pincéis de `spec` (ver o doc do módulo).
pub fn pinceis(spec: &str) -> Vec<Pincel> {
    let bib = biblioteca();
    let de_entrada = |e: &ph2d_brush_presets::Entrada| Pincel {
        nome: e.preset.parametros.name.clone(),
        fonte: Fonte::Preset {
            preset: Box::new(e.preset.clone()),
            relativo: e.relativo.clone(),
        },
    };
    let mut v = Vec::new();
    for s in spec.split('|').map(str::trim).filter(|s| !s.is_empty()) {
        if s == "todos" {
            v.extend(bib.entradas.iter().map(de_entrada));
        } else if let Some(r) = RECEITAS_DE_MEIO.iter().find(|r| r.nome == s) {
            v.push(Pincel {
                nome: s.to_owned(),
                fonte: Fonte::Meio(r),
            });
        } else if s.ends_with(".ph2d-brush.png") {
            let bytes = std::fs::read(s).unwrap_or_else(|e| panic!("{s}: {e}"));
            let preset = ph2d_brush_presets::ler(&bytes).unwrap_or_else(|e| panic!("{s}: {e:?}"));
            v.push(Pincel {
                nome: preset.parametros.name.clone(),
                fonte: Fonte::Preset {
                    preset: Box::new(preset),
                    relativo: s.to_owned(),
                },
            });
        } else {
            let e = bib
                .entradas
                .iter()
                .find(|e| e.preset.parametros.name == s)
                .unwrap_or_else(|| panic!("o pincel {s:?} não está na biblioteca"));
            v.push(de_entrada(e));
        }
    }
    v
}

/// As VARIANTES de cada preset (`PH2D_EXAME_VARIANTES` = um JSON `{"<nome>"|"*": [{"<knob>": v, "brush.<campo>": v},
/// …]}`): cada mapa sobrepõe-se ao `engine` (ou, com `brush.`, ao `brush`) do preset e pinta-se como
/// `<nome>~v<k>` — o ajuste de uma lei (03_MOTORES §7) mede N valores num só processo. Sem a variável, os presets.
fn variantes(pinceis: Vec<Pincel>) -> Vec<Pincel> {
    let Ok(f) = std::env::var("PH2D_EXAME_VARIANTES") else {
        return pinceis;
    };
    let v: Value = serde_json::from_str(&std::fs::read_to_string(&f).unwrap()).unwrap();
    let mut out = Vec::new();
    for p in pinceis {
        let Fonte::Preset { preset, relativo } = &p.fonte else {
            out.push(p);
            continue;
        };
        let lista = v.get(&p.nome).or_else(|| v.get("*"));
        for (k, m) in lista.and_then(Value::as_array).into_iter().flatten().enumerate() {
            let mut pr = preset.clone();
            for (chave, valor) in m.as_object().unwrap() {
                if let Some(campo) = chave.strip_prefix("brush.") {
                    pr.parametros.brush.insert(campo.to_owned(), valor.clone());
                } else {
                    pr.parametros.engine.insert(chave.clone(), valor.as_f64().unwrap());
                }
            }
            out.push(Pincel {
                nome: format!("{}~v{k}", p.nome),
                fonte: Fonte::Preset {
                    preset: pr,
                    relativo: relativo.clone(),
                },
            });
        }
    }
    out
}

/// A bateria (`PH2D_EXAME_BATERIA`, por omissão a do EXAME).
pub fn bateria() -> Value {
    let f = std::env::var("PH2D_EXAME_BATERIA").map_or_else(
        |_| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../docs/Painter/Pinceis/ferramentas/bateria.json")
        },
        PathBuf::from,
    );
    serde_json::from_str(&std::fs::read_to_string(&f).unwrap_or_else(|e| panic!("{f:?}: {e}")))
        .unwrap()
}

fn escala() -> f32 {
    std::env::var("PH2D_EXAME_ESCALA")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1.0)
}

/// Um ponto `[x, y, pressão]` da bateria, ou `[x, y, pressão, tilt_x°, tilt_y°]` (a caneta DEITADA, em graus à
/// vertical: `(t, 0)` ao longo de +x, `(0, t)` através — `exames/lapis/bateria_deitado.json`); sem os dois, de pé.
fn cp(p: &Value, phase: PointerPhase) -> CanvasPointer {
    let grau = |i: usize| p.get(i).and_then(Value::as_f64).unwrap_or(0.0) as f32 / 90.0;
    CanvasPointer {
        pos: [p[0].as_f64().unwrap() as f32, p[1].as_f64().unwrap() as f32],
        pressure: p[2].as_f64().unwrap() as f32,
        tilt: [grau(3), grau(4)],
        phase,
    }
}

/// Um traço da bateria tal como está: os pontos EXACTOS, um evento cada, 8 ms entre eles (o `sonda_gestos::gesto`).
fn traco(t: &mut PainterTool, pts: &[Value]) {
    t.on_canvas_pointer(cp(&pts[0], PointerPhase::Down));
    for p in &pts[1..] {
        t.on_canvas_pointer(cp(p, PointerPhase::Move));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(cp(&pts[pts.len() - 1], PointerPhase::Up));
}

fn seca(t: &mut PainterTool, meio: PaintMedia) {
    match meio {
        PaintMedia::WetPaint => {
            t.handle_panel_event(PanelEvent::Click(ids::PAINTER_WETPAINT_DRYCANVAS))
        }
        PaintMedia::Watercolor => {
            t.handle_panel_event(PanelEvent::Click(ids::PAINTER_WATERCOLOR_DRY_NOW))
        }
        PaintMedia::Digital | PaintMedia::Impasto => {}
    }
}

/// Uma cena da bateria com `pincel` (a «base» da cena `sobre` é o mesmo pincel, ou `base`) até à foto n.º `foto`
/// (0 = a 1.ª): devolve o nome e os pixels dela. ⚠️ Uma corrida por foto: o `run_full` é o Apply, que LEVA a tela
/// (`trait_impls_raster.rs`), e o motor é determinístico (relógio fixo, semente 0) — a foto B repinta a cena.
pub fn pinta_cena(
    pincel: &Pincel,
    base: Option<&Pincel>,
    cena: &Value,
    tela: [u32; 2],
    papel: [u8; 3],
    foto: usize,
) -> (String, Vec<u8>) {
    let [w, h] = tela;
    let mut t = PainterTool::default();
    let fundo: Vec<u8> = (0..w * h)
        .flat_map(|_| [papel[0], papel[1], papel[2], 255])
        .collect();
    t.set_source(fundo, w, h);
    let mut vistas = 0;
    let mut actual: Option<&Pincel> = None;
    let mut meio = pincel.meio();
    for op in cena["ops"].as_array().unwrap() {
        match op["op"].as_str().unwrap() {
            "pincel" => {
                let quem = if op["quem"] == "base" {
                    base.unwrap_or(pincel)
                } else {
                    pincel
                };
                if !actual.is_some_and(|a| std::ptr::eq(a, quem)) {
                    quem.aplica(&mut t, true);
                    meio = quem.meio();
                    actual = Some(quem);
                }
            }
            "cor" => {
                let c = &op["rgb"];
                t.set_brush_color_srgb8([0, 1, 2].map(|i| c[i].as_u64().unwrap() as u8));
            }
            "traco" => {
                t.set_brush_size_px(op["size_px"].as_f64().unwrap() as f32 * escala() * 0.5);
                traco(&mut t, op["pts"].as_array().unwrap());
            }
            "simula" => {
                for _ in 0..op["quadros"].as_u64().unwrap() {
                    t.on_tick(1000.0 / 60.0);
                }
            }
            "seca" => {
                seca(&mut t, meio);
                // O quadro seguinte ao clique: é nele que o produto recompõe a tela.
                t.on_tick(1000.0 / 60.0);
            }
            "foto" if vistas == foto => {
                return (op["nome"].as_str().unwrap().to_owned(), t.run_full().0);
            }
            "foto" => vistas += 1,
            o => panic!("op da bateria desconhecida: {o}"),
        }
    }
    panic!("a cena {} não tem a foto n.º {foto}", cena["nome"])
}

fn fotos_da_cena(cena: &Value) -> usize {
    cena["ops"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|o| o["op"] == "foto")
        .count()
}

/// A foto como o artista a vê: a camada SOBRE o papel da bateria. Uma borracha (`EraseAlpha`) só mexe no alfa e
/// deixa o RGB de antes; sem compor, o apagado lia-se como tinta intacta. Com alfa 255 (todo pincel que pinta) o
/// píxel sai ao byte.
fn escreve_ppm(path: &Path, rgba: &[u8], w: u32, h: u32, papel: [u8; 3]) {
    let mut f = format!("P6\n{w} {h}\n255\n").into_bytes();
    for px in rgba.as_chunks::<4>().0 {
        let a = u32::from(px[3]);
        f.extend((0..3).map(|i| {
            ((u32::from(px[i]) * a + u32::from(papel[i]) * (255 - a) + 127) / 255) as u8
        }));
    }
    std::fs::write(path, f).unwrap();
}

fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, &b| {
        (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

fn fios() -> usize {
    std::env::var("PH2D_EXAME_FIOS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or_else(|| {
            std::thread::available_parallelism().map_or(4, |n| (n.get() / 2).clamp(1, 8))
        })
}

/// O pico de memória do processo (`VmHWM`) e a de agora (`VmRSS`), em KiB.
fn memoria_kib() -> (u64, u64) {
    let s = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    let campo = |k: &str| {
        s.lines()
            .find_map(|l| l.strip_prefix(k))
            .and_then(|r| r.split_whitespace().next()?.parse().ok())
            .unwrap_or(0)
    };
    (campo("VmHWM:"), campo("VmRSS:"))
}

/// O tempo de CPU do processo (todos os fios: o da água fora do ecrã também), em s.
fn cpu_s() -> f64 {
    let s = std::fs::read_to_string("/proc/self/stat").unwrap_or_default();
    let campos: Vec<&str> = s
        .rsplit_once(')')
        .map_or("", |(_, r)| r)
        .split_whitespace()
        .collect();
    let tiques = |i: usize| {
        campos
            .get(i)
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(0.0)
    };
    // utime e stime são os campos 14 e 15 do `stat` (11 e 12 depois do `)`), em tiques de 100 Hz.
    (tiques(11) + tiques(12)) / 100.0
}

#[test]
#[ignore = "bancada do EXAME: pinta a bateria; corre-se pelo exame.sh"]
fn exame_pinta() {
    let spec = std::env::var("PH2D_EXAME_PRESETS").expect("PH2D_EXAME_PRESETS");
    let out = PathBuf::from(std::env::var("PH2D_EXAME_OUT").expect("PH2D_EXAME_OUT"));
    let todos = variantes(pinceis(&spec));
    let base = std::env::var("PH2D_EXAME_BASE")
        .ok()
        .map(|b| pinceis(&b).remove(0));
    // Sem `PH2D_EXAME_BASE`, a ferramenta que não pinta leva a base do seu meio (a biblioteca e a matriz também): um
    // pincel que pinta continua com as faixas dele, ao byte.
    let padroes: Vec<Option<Pincel>> = todos
        .iter()
        .map(|p| {
            (base.is_none() && p.nao_pinta()).then(|| pinceis(base_padrao(p.meio())).remove(0))
        })
        .collect();
    let bat = bateria();
    let tela = [0, 1].map(|i| bat["tela"][i].as_u64().unwrap() as u32);
    let papel = [0, 1, 2].map(|i| bat["papel"][i].as_u64().unwrap() as u8);
    let cenas = bat["cenas"].as_array().unwrap();
    let so: Option<Vec<String>> = std::env::var("PH2D_EXAME_CENAS")
        .ok()
        .map(|c| c.split(',').map(str::to_owned).collect());
    let cenas: Vec<&Value> = cenas
        .iter()
        .filter(|c| {
            so.as_ref()
                .is_none_or(|s| s.iter().any(|n| c["nome"] == n.as_str()))
        })
        .collect();
    // `PH2D_EXAME_SO_A=1`: só a 1.ª foto de cada cena (a molhada) — o ajuste de uma lei não precisa da seca.
    let so_a = std::env::var("PH2D_EXAME_SO_A").is_ok_and(|v| v == "1");
    let tarefas: Vec<(usize, usize, usize)> = (0..todos.len())
        .flat_map(|p| {
            let cenas = &cenas;
            (0..cenas.len()).flat_map(move |c| {
                let n = if so_a { 1 } else { fotos_da_cena(cenas[c]) };
                (0..n).map(move |f| (p, c, f))
            })
        })
        .collect();
    let proxima = AtomicUsize::new(0);
    let metas: Mutex<Vec<Value>> = Mutex::new(vec![json!({}); todos.len()]);
    let t0 = Instant::now();
    std::thread::scope(|s| {
        for _ in 0..fios().min(tarefas.len()) {
            s.spawn(|| {
                loop {
                    let i = proxima.fetch_add(1, Ordering::Relaxed);
                    let Some(&(p, c, f)) = tarefas.get(i) else {
                        break;
                    };
                    let (pincel, cena) = (&todos[p], cenas[c]);
                    let nome_cena = cena["nome"].as_str().unwrap();
                    let dir = out.join(pincel.slug());
                    std::fs::create_dir_all(&dir).unwrap();
                    let t1 = Instant::now();
                    let base = base.as_ref().or(padroes[p].as_ref());
                    let (foto, px) = pinta_cena(pincel, base, cena, tela, papel, f);
                    let ms = t1.elapsed().as_secs_f64() * 1e3;
                    let chave = format!("{nome_cena}_{foto}");
                    escreve_ppm(&dir.join(format!("{chave}.ppm")), &px, tela[0], tela[1], papel);
                    println!("[exame] {} · {chave}: {ms:.0} ms", pincel.nome);
                    let mut m = metas.lock().unwrap();
                    let e = &mut m[p];
                    e["nome"] = json!(pincel.nome);
                    e["meio"] = json!(format!("{:?}", pincel.meio()));
                    e["escala"] = json!(escala());
                    if let Fonte::Preset { relativo, .. } = &pincel.fonte {
                        e["relativo"] = json!(relativo);
                    }
                    if let Some(b) = base {
                        e["base"] = json!(b.nome);
                    }
                    e["fnv"][&chave] = json!(format!("{:#018x}", fnv1a(&px)));
                    e["ms"][&chave] = json!(ms.round());
                }
            });
        }
    });
    for (p, m) in todos.iter().zip(metas.into_inner().unwrap()) {
        std::fs::write(
            out.join(p.slug()).join("meta.json"),
            serde_json::to_string_pretty(&m).unwrap(),
        )
        .unwrap();
    }
    println!(
        "[exame] {} pincéis × {} cenas em {:.1} s",
        todos.len(),
        cenas.len(),
        t0.elapsed().as_secs_f64()
    );
}

/// O CUSTO a 60 Hz (§3.6): a cena `gestos` com TODOS os traços ao diâmetro `PH2D_EXAME_TAMANHO`, `PH2D_EXAME_
/// EVENTOS_POR_QUADRO` eventos (3: uma caneta a ~180 Hz) e um tique de 60 Hz por quadro, ao ritmo da parede; depois
/// 120 quadros parados (a água a correr). A água do ecrã (sem relógio fixo). O `ms/quadro` é o fio do pincel; o
/// `nucleos` é o CPU do processo inteiro (a água no fio dela) ÷ a parede. Um pincel e um tamanho por processo: o
/// `VmHWM` é o pico DESTE pincel a este tamanho.
#[test]
#[ignore = "bancada do EXAME: o custo a 60 Hz; corre-se pelo exame.sh"]
fn exame_custo() {
    let (_, rss0) = memoria_kib();
    let spec = std::env::var("PH2D_EXAME_PRESETS").expect("PH2D_EXAME_PRESETS");
    let tam: f32 = std::env::var("PH2D_EXAME_TAMANHO").map_or(40.0, |v| v.parse().unwrap());
    let por_quadro: usize =
        std::env::var("PH2D_EXAME_EVENTOS_POR_QUADRO").map_or(3, |v| v.parse().unwrap());
    let bat = bateria();
    let [w, h] = [0, 1].map(|i| bat["tela"][i].as_u64().unwrap() as u32);
    let gestos = bat["cenas"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["nome"] == "gestos")
        .unwrap();
    for pincel in pinceis(&spec) {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (w * h * 4) as usize], w, h);
        pincel.aplica(&mut t, false);
        t.set_brush_size_px(tam * 0.5);
        t.set_brush_color_srgb8([23, 101, 179]);
        let (mut quadros, mut parados) = (Vec::new(), Vec::new());
        // Os eventos da cena, por ordem (o Down e o Up de cada traço contam como eventos).
        let eventos: Vec<CanvasPointer> = gestos["ops"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|o| o["op"] == "traco")
            .flat_map(|op| {
                let pts = op["pts"].as_array().unwrap();
                let n = pts.len();
                pts.iter()
                    .enumerate()
                    .map(move |(i, p)| {
                        cp(
                            p,
                            if i == 0 {
                                PointerPhase::Down
                            } else {
                                PointerPhase::Move
                            },
                        )
                    })
                    .chain(std::iter::once(cp(&pts[n - 1], PointerPhase::Up)))
            })
            .collect();
        // O ritmo de 60 Hz no relógio de PAREDE (`PH2D_EXAME_RITMO=0` desliga): a água do ecrã anda num fio
        // próprio ao relógio real, e sem o ritmo ela faria em 1 s o que o artista faz em 8.
        let ritmo = std::env::var("PH2D_EXAME_RITMO").as_deref() != Ok("0");
        let periodo = std::time::Duration::from_secs_f64(1.0 / 60.0);
        let mut prazo = Instant::now();
        let mut espera = || {
            prazo += periodo;
            match prazo.checked_duration_since(Instant::now()) {
                Some(d) if ritmo => std::thread::sleep(d),
                _ => prazo = Instant::now(),
            }
        };
        let (t0, cpu0) = (Instant::now(), cpu_s());
        for quadro in eventos.chunks(por_quadro) {
            let q0 = Instant::now();
            for e in quadro {
                t.on_canvas_pointer(e.clone());
            }
            t.on_tick(1000.0 / 60.0);
            quadros.push(q0.elapsed().as_secs_f64() * 1e3);
            espera();
        }
        for _ in 0..120 {
            let q0 = Instant::now();
            t.on_tick(1000.0 / 60.0);
            parados.push(q0.elapsed().as_secs_f64() * 1e3);
            espera();
        }
        let _ = t.run_full();
        let total = t0.elapsed().as_secs_f64();
        let cpu = cpu_s() - cpu0;
        let (hwm, _) = memoria_kib();
        let est = |v: &mut Vec<f64>| {
            v.sort_by(f64::total_cmp);
            let q = |f: f64| v[((v.len() - 1) as f64 * f) as usize];
            json!({"n": v.len(), "mediana": q(0.5), "p95": q(0.95), "pior": q(1.0),
                   "acima_de_16_7": v.iter().filter(|&&x| x > 1000.0 / 60.0).count()})
        };
        let r = json!({
            "pincel": pincel.nome, "tamanho_px": tam, "eventos_por_quadro": por_quadro,
            "quadros_a_pintar_ms": est(&mut quadros), "quadros_parados_ms": est(&mut parados),
            "total_s": total, "cpu_s": cpu, "nucleos": cpu / total, "vm_hwm_kib": hwm, "vm_rss_inicio_kib": rss0,
            "load": std::fs::read_to_string("/proc/loadavg").unwrap_or_default().trim(),
        });
        println!("[custo] {r}");
    }
}

/// A base das faixas da cena «sobre» de uma ferramenta que não pinta ([`Pincel::nao_pinta`]): a tinta lisa do meio dela
/// — o disco duro no Digital (as faixas lisas do Round Hard do Rebelle), a acrílica no Wet e nos outros (a tinta que as
/// ferramentas molhadas movem).
pub fn base_padrao(meio: PaintMedia) -> &'static str {
    match meio {
        PaintMedia::Digital => "Hard Round",
        _ => "Acrylic Round",
    }
}
