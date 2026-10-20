//! **Os presets nos QUATRO meios** (L1) e **a mão nos três antigos** (L2) — `docs/Painter/Pinceis/03_MOTORES.md`
//! §4. Cada portão pinta a cena `gestos` da BATERIA (`exame_bancada::pinta_cena`, a fonte única dos gestos: a
//! rampa de pressão 0→1 está nela) e compara-a AO BYTE.

use crate::exame_bancada::{Fonte, Pincel, RECEITAS_DE_MEIO, bateria, pinta_cena};
use ph2d_tool_painter::PaintMedia;
use serde_json::Value;

fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, &b| {
        (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

/// A cena `gestos` da bateria, a foto A (molhada), com `pincel`.
pub fn gestos(pincel: &Pincel) -> Vec<u8> {
    let bat = bateria();
    let tela = [0, 1].map(|i| bat["tela"][i].as_u64().unwrap() as u32);
    let papel = [0, 1, 2].map(|i| bat["papel"][i].as_u64().unwrap() as u8);
    let cena: &Value = bat["cenas"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["nome"] == "gestos")
        .unwrap();
    pinta_cena(pincel, None, cena, tela, papel, 0).1
}

fn meio(m: PaintMedia) -> Pincel {
    let r = RECEITAS_DE_MEIO.iter().find(|r| r.meio == m).unwrap();
    Pincel {
        nome: r.nome.to_owned(),
        fonte: Fonte::Meio(r),
    }
}

/// A cena `gestos` (a rampa 0→1 incluída) nos três meios antigos ANTES da L2 — `pressureSize 0,9` (o
/// `size_min 0,1` de sempre), sem pressão na opacidade, a inclinação de pé. Medido em `0797d99072`.
const NEUTRO_DE_HOJE: [(PaintMedia, u64); 3] = [
    (PaintMedia::Digital, 0x84f5_0263_d09d_3843),
    (PaintMedia::Watercolor, 0x2a62_0899_d312_8981),
    (PaintMedia::Impasto, 0xf545_285d_938d_8abf),
];

/// ⭐ **O neutro da mão é o traço de hoje**: com a mão de fábrica, os três meios antigos pintam a cena da bateria
/// ao byte do que pintavam antes de a mão ser um campo do pincel.
#[test]
fn o_neutro_da_mao_e_o_traco_de_hoje() {
    let mut erros = Vec::new();
    for (m, esperado) in NEUTRO_DE_HOJE {
        let h = fnv1a(&gestos(&meio(m)));
        if h != esperado {
            erros.push(format!("{m:?}: 0x{h:016x} (esperado 0x{esperado:016x})"));
        }
        // E pela LEI da mão: o pincel de fábrica gravado e carregado (a mão volta por `mao::escreve`).
        let mut t = ph2d_tool_painter::PainterTool::default();
        t.set_paint_media(m);
        let (parametros, texturas) = t.preset_capturar(&ph2d_brush_presets::Parametros::novo(
            ph2d_tool_painter::nome_do_meio(m),
            "G",
            "N",
        ));
        let preset = ph2d_brush_presets::Preset {
            parametros,
            origem: ph2d_brush_presets::Origem::Utilizador,
            miniatura: ph2d_brush_presets::Miniatura {
                largura: 1,
                altura: 1,
                rgba: vec![0; 4],
            },
            texturas,
        };
        let h = fnv1a(&gestos(&Pincel {
            nome: format!("{m:?} gravado"),
            fonte: Fonte::Preset {
                preset: Box::new(preset),
                relativo: String::new(),
            },
        }));
        if h != esperado {
            erros.push(format!("{m:?} gravado: 0x{h:016x} (esperado 0x{esperado:016x})"));
        }
    }
    assert!(erros.is_empty(), "o neutro mudou o traço: {erros:#?}");
}

// ── L1: o preset nos quatro meios ────────────────────────────────────────────────────────────────────

use crate::exame_bancada::ReceitaDeMeio;
use ph2d_brush_presets::{Biblioteca, Miniatura, Origem, Parametros, Preset};
use ph2d_tool_painter::{
    LinhaDaMao, PainterTool, meio_do_preset, miniatura_do_preset, motor_do_meio, nome_do_meio,
};
use ph2d_wet_paint::tuning::Knob;
use std::path::PathBuf;

fn lapis(t: &mut PainterTool) {
    t.set_brush_texture_kind(14); // Grain
    t.set_brush_grain_depth(0.6);
    t.set_brush_spacing(0.05);
    t.set_brush_strength(0.85);
    t.set_mao(LinhaDaMao::Knob(Knob::PressureSize), 0.6);
    t.set_mao(LinhaDaMao::Knob(Knob::PressureOpacity), 0.5);
    t.set_mao(LinhaDaMao::TiltOpacity, 0.4);
}

fn aguada(t: &mut PainterTool) {
    t.set_brush_depth(1.6);
    t.set_brush_edge_gain(2.0);
    t.set_brush_granulation(0.4);
    t.set_brush_wet_dilution(0.3);
    t.set_mao(LinhaDaMao::Knob(Knob::PressureSize), 0.3);
}

fn empaste(t: &mut PainterTool) {
    t.set_brush_impasto_depth(0.6);
    t.set_brush_impasto_body(0.7);
    t.set_impasto_shine(0.5);
    t.set_brush_dab_flatten(0.4);
    t.set_brush_dab_angle(30.0);
    t.set_mao(LinhaDaMao::Knob(Knob::PressureOpacity), 0.4);
}

/// Os três pincéis de EXEMPLO (um por meio antigo) — provam o caminho de ponta a ponta; os pincéis «a sério»
/// são das janelas 3–4. `(receita, motor/meio/grupo, nome)`.
pub static EXEMPLOS: [(ReceitaDeMeio, &str, &str); 3] = [
    (
        ReceitaDeMeio {
            nome: "exemplo:lapis",
            meio: PaintMedia::Digital,
            prepara: lapis,
        },
        "Pencil/Graphite",
        "Sketch Pencil",
    ),
    (
        ReceitaDeMeio {
            nome: "exemplo:aguada",
            meio: PaintMedia::Watercolor,
            prepara: aguada,
        },
        "Watercolor/Round",
        "Flat Wash",
    ),
    (
        ReceitaDeMeio {
            nome: "exemplo:empaste",
            meio: PaintMedia::Impasto,
            prepara: empaste,
        },
        "Oil/Bristle",
        "Thick Impasto",
    ),
];

/// O pincel de uma receita CAPTURADO e gravado: os bytes do `.ph2d-brush.png` (com a miniatura do motor).
fn grava(r: &'static ReceitaDeMeio, grupo: &str, nome: &str) -> Vec<u8> {
    let mut t = PainterTool::default();
    Pincel {
        nome: r.nome.to_owned(),
        fonte: Fonte::Meio(r),
    }
    .aplica(&mut t, true);
    let mut base = Parametros::novo(nome_do_meio(r.meio), grupo, nome);
    base.name = nome.to_owned();
    let (parametros, texturas) = t.preset_capturar(&base);
    let mut p = Preset {
        parametros,
        origem: Origem::Fabrica,
        miniatura: Miniatura {
            largura: 1,
            altura: 1,
            rgba: vec![0; 4],
        },
        texturas,
    };
    p.miniatura = miniatura_do_preset(&p);
    ph2d_brush_presets::escrever(&p).unwrap()
}

/// ⭐ **Capturar → gravar → carregar = a cena da bateria AO BYTE**, nos três meios antigos (L1).
fn volta_ao_bit(i: usize) {
    let (r, grupo, nome) = &EXEMPLOS[i];
    let antes = gestos(&Pincel {
        nome: r.nome.to_owned(),
        fonte: Fonte::Meio(r),
    });
    let preset = ph2d_brush_presets::ler(&grava(r, grupo, nome)).unwrap();
    assert_eq!(meio_do_preset(&preset.parametros.media), Some(r.meio));
    let depois = gestos(&Pincel {
        nome: nome.to_string(),
        fonte: Fonte::Preset {
            preset: Box::new(preset),
            relativo: String::new(),
        },
    });
    assert!(
        antes == depois,
        "{:?}: o preset gravado pinta outra cena (0x{:016x} → 0x{:016x})",
        r.meio,
        fnv1a(&antes),
        fnv1a(&depois)
    );
}

#[test]
fn um_preset_digital_volta_ao_bit() {
    volta_ao_bit(0);
}

#[test]
fn um_preset_watercolor_volta_ao_bit() {
    volta_ao_bit(1);
}

#[test]
fn um_preset_impasto_volta_ao_bit() {
    volta_ao_bit(2);
}

/// ⭐ **O meio do preset é o meio que pinta**: aplicar um preset de cada meio a um pincel de OUTRO meio troca o
/// meio; um `media` desconhecido é RECUSADO e não muda nada (nunca cai no Digital calado).
#[test]
fn o_meio_do_preset_e_o_meio_que_pinta() {
    let todos = [
        PaintMedia::Digital,
        PaintMedia::Watercolor,
        PaintMedia::Impasto,
        PaintMedia::WetPaint,
    ];
    for de in todos {
        let mut fonte = PainterTool::default();
        fonte.set_paint_media(de);
        let (prm, tex) = fonte.preset_capturar(&Parametros::novo(nome_do_meio(de), "G", "N"));
        assert_eq!(prm.media, nome_do_meio(de));
        let p = Preset {
            parametros: prm,
            origem: Origem::Utilizador,
            miniatura: Miniatura {
                largura: 1,
                altura: 1,
                rgba: vec![0; 4],
            },
            texturas: tex,
        };
        for em in todos.into_iter().filter(|m| *m != de) {
            let mut t = PainterTool::default();
            t.set_paint_media(em);
            assert_eq!(t.preset_aplicar(&p), Ok(Vec::new()), "{de:?} em {em:?}");
            assert_eq!(t.paint_media(), de, "o preset de {de:?} aplicado em {em:?}");
        }
        let mut estranho = p.clone();
        estranho.parametros.media = "Fluid".to_owned();
        let mut t = PainterTool::default();
        t.set_paint_media(de);
        let antes = t.preset_capturar(&Parametros::novo("x", "G", "N"));
        let r = t.preset_aplicar(&estranho);
        assert_eq!(r, Err(ph2d_tool_painter::MeioDesconhecido("Fluid".to_owned())));
        assert_eq!(t.paint_media(), de, "a recusa mudou o meio");
        assert_eq!(
            t.preset_capturar(&Parametros::novo("x", "G", "N")),
            antes,
            "a recusa mudou o pincel"
        );
    }
}

fn fabrica() -> Biblioteca {
    let raiz = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let b = Biblioteca::carregar(&raiz, None);
    assert!(b.erros.is_empty(), "{:?}", b.erros);
    b
}

/// **No Wet, a porta dos quatro meios É a do Wet**: cada preset de fábrica, aplicado por `preset_aplicar`, deixa o
/// pincel que `wet_preset_aplicar` deixa (a captura inteira, o mapa do motor e as texturas ao byte), e a captura
/// dos quatro meios é a do Wet. ⚠️ Prova a PORTA, não «igual a antes da L1»: essa guarda é a das miniaturas de
/// fábrica (`a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje`, os ficheiros gravados antes, repintados pela porta
/// nova) e a da cena (`sem_glaze_cada_preset_de_fabrica_pinta_a_cena_de_antes`).
#[test]
fn a_porta_dos_quatro_meios_e_a_do_wet_no_wet() {
    for e in &fabrica().entradas {
        let p = &e.preset;
        if p.parametros.media != nome_do_meio(PaintMedia::WetPaint) {
            continue;
        }
        let mut a = PainterTool::default();
        a.set_paint_media(PaintMedia::WetPaint);
        let fa = a.wet_preset_aplicar(p);
        let mut b = PainterTool::default();
        assert_eq!(b.preset_aplicar(p), Ok(fa), "{}", e.relativo);
        assert_eq!(b.paint_media(), PaintMedia::WetPaint);
        let ca = a.wet_preset_capturar(&p.parametros);
        assert!(
            b.preset_capturar(&p.parametros) == ca,
            "{}: o pincel aplicado pelos quatro meios difere",
            e.relativo
        );
    }
}

/// ⭐ **A contagem por meio**: todo preset da fábrica e dos exemplos tem um meio conhecido e mora na pasta
/// desse meio (`<motor>/…`); a fábrica tem, em cada meio, os que as receitas contam (as famílias da J3–J4,
/// `receitas_de_fabrica::contagem_por_meio`), os exemplos um por meio antigo.
#[test]
fn a_contagem_por_meio() {
    let conta = |b: &Biblioteca| {
        let mut n = [0usize; 4];
        for e in &b.entradas {
            let m = meio_do_preset(&e.preset.parametros.media)
                .unwrap_or_else(|| panic!("{}: meio desconhecido", e.relativo));
            assert_eq!(e.partes()[0], motor_do_meio(m), "{}: fora da pasta do meio", e.relativo);
            n[usize::from(m.to_u8())] += 1;
        }
        n
    };
    // Digital · Watercolor · Impasto · Wet Paint
    assert_eq!(
        conta(&fabrica()),
        crate::receitas_de_fabrica::contagem_por_meio()
    );
    let ex = Biblioteca::carregar(&raiz_dos_exemplos(), None);
    assert!(ex.erros.is_empty(), "{:?}", ex.erros);
    assert_eq!(conta(&ex), [1, 1, 1, 0]);
}

fn raiz_dos_exemplos() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/Painter/Pinceis/fixtures/presets_exemplo")
}

fn caminho_do_exemplo(i: usize) -> PathBuf {
    let (r, grupo, nome) = &EXEMPLOS[i];
    raiz_dos_exemplos().join(format!(
        "{}/{grupo}/{nome}.ph2d-brush.png",
        motor_do_meio(r.meio)
    ))
}

/// Os exemplos guardados são os que o motor grava hoje (miniatura incluída) — a mesma lei das miniaturas de
/// fábrica: uma mudança de lei nos meios antigos acusa aqui.
#[test]
fn os_exemplos_sao_os_que_o_motor_grava() {
    for i in 0..EXEMPLOS.len() {
        let (r, grupo, nome) = &EXEMPLOS[i];
        let f = caminho_do_exemplo(i);
        let guardado = std::fs::read(&f).unwrap_or_else(|e| panic!("{f:?}: {e}"));
        assert!(guardado == grava(r, grupo, nome), "{f:?} mudou — regenere");
    }
}

#[test]
#[ignore = "gerador: escreve os três presets de exemplo em docs/Painter/Pinceis/fixtures/presets_exemplo"]
fn gera_os_presets_de_exemplo() {
    for i in 0..EXEMPLOS.len() {
        let (r, grupo, nome) = &EXEMPLOS[i];
        let f = caminho_do_exemplo(i);
        std::fs::create_dir_all(f.parent().unwrap()).unwrap();
        std::fs::write(&f, grava(r, grupo, nome)).unwrap();
        eprintln!("{}", f.display());
    }
}

// ── L2: a mão nos três meios antigos ─────────────────────────────────────────────────────────────────

use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};

/// Um recto de 160 px a pressão 1 com a caneta inclinada `tilt`, num papel branco: a soma da tinta (255 − R).
fn recto_inclinado(meio_: PaintMedia, opacidade: f64, tilt: [f32; 2]) -> (u64, Vec<u8>) {
    let (w, h) = (200u32, 80u32);
    let mut t = PainterTool::default();
    t.set_source(vec![255; (w * h * 4) as usize], w, h);
    t.set_paint_media(meio_);
    t.set_brush_size_px(8.0);
    t.set_brush_color_srgb8([20, 20, 20]);
    t.set_mao(LinhaDaMao::TiltOpacity, opacidade);
    let cp = |x: f32, phase| CanvasPointer {
        pos: [x, 40.0],
        pressure: 1.0,
        tilt,
        phase,
    };
    t.on_canvas_pointer(cp(20.0, PointerPhase::Down));
    let mut x = 20.0;
    while x < 180.0 {
        x += 2.0;
        t.on_canvas_pointer(cp(x, PointerPhase::Move));
    }
    t.on_canvas_pointer(cp(x, PointerPhase::Up));
    let px = t.run_full().0;
    let tinta = px
        .as_chunks::<4>()
        .0
        .iter()
        .map(|p| u64::from(255 - p[0]))
        .sum();
    (tinta, px)
}

/// ⭐ **A inclinação tira opacidade** (L2, o «Tilt → Opacity» do Procreate) no Digital e no Impasto: com a caneta
/// deitada inteira (81°, acima do `tiltFull` 67,5°) cada dab leva `1 − tiltOpacity` da cobertura — a 1 nada pinta,
/// a 0,5 menos (a Strength 1 os dabs ACUMULAM, como a pressão na opacidade: medido 0,81 da tinta); de pé, ou com a
/// linha a 0, é o traço de sempre AO BYTE.
#[test]
fn a_caneta_deitada_tira_a_opacidade() {
    for m in [PaintMedia::Digital, PaintMedia::Impasto] {
        let (cheia, _) = recto_inclinado(m, 0.0, [0.9, 0.0]);
        let (meia, _) = recto_inclinado(m, 0.5, [0.9, 0.0]);
        let (nada, _) = recto_inclinado(m, 1.0, [0.9, 0.0]);
        let r = meia as f64 / cheia as f64;
        assert!((0.3..0.9).contains(&r), "{m:?}: deitada a 0,5 leva {r:.3} da tinta");
        assert_eq!(nada, 0, "{m:?}: deitada inteira a 1 não pinta");
        assert_eq!(
            recto_inclinado(m, 0.5, [0.0, 0.0]).1,
            recto_inclinado(m, 0.0, [0.0, 0.0]).1,
            "{m:?}: de pé a linha não muda um byte"
        );
    }
}

/// A mão do cartão chega ao traço nos três meios antigos: a cena da bateria (a rampa 0→1) com `pressureSize`
/// 0,3 já não é a do neutro.
#[test]
fn a_mao_do_cartao_chega_ao_traco() {
    for (m, neutro) in NEUTRO_DE_HOJE {
        static RECEITAS: [ReceitaDeMeio; 3] = [
            ReceitaDeMeio {
                nome: "mao:Digital",
                meio: PaintMedia::Digital,
                prepara: pressao_03,
            },
            ReceitaDeMeio {
                nome: "mao:Watercolor",
                meio: PaintMedia::Watercolor,
                prepara: pressao_03,
            },
            ReceitaDeMeio {
                nome: "mao:Impasto",
                meio: PaintMedia::Impasto,
                prepara: pressao_03,
            },
        ];
        let r = RECEITAS.iter().find(|r| r.meio == m).unwrap();
        let h = fnv1a(&gestos(&Pincel {
            nome: r.nome.to_owned(),
            fonte: Fonte::Meio(r),
        }));
        assert_ne!(h, neutro, "{m:?}: a pressão do cartão não chegou ao traço");
    }
}

fn pressao_03(t: &mut PainterTool) {
    t.set_mao(LinhaDaMao::Knob(Knob::PressureSize), 0.3);
}

// ── O custo (sonda, `--ignored`) ─────────────────────────────────────────────────────────────────────

/// O custo da miniatura de cada meio (`miniatura_do_preset`, a que o «Save as New» pinta): o MÍNIMO de 5
/// corridas, em ms, os três exemplos e o Wet Round de fábrica ao lado. Perfil `smoke`.
#[test]
#[ignore = "sonda: o custo da miniatura por meio"]
fn sonda_custo_da_miniatura_por_meio() {
    let mut presets: Vec<(String, Preset)> = (0..EXEMPLOS.len())
        .map(|i| {
            let b = std::fs::read(caminho_do_exemplo(i)).unwrap();
            (
                format!("{:?}", EXEMPLOS[i].0.meio),
                ph2d_brush_presets::ler(&b).unwrap(),
            )
        })
        .collect();
    let round = fabrica()
        .entradas
        .into_iter()
        .find(|e| e.preset.parametros.name == "Wet Round")
        .unwrap();
    presets.push(("WetPaint (Wet Round)".to_owned(), round.preset));
    for (nome, p) in &presets {
        let ms = (0..5)
            .map(|_| {
                let t0 = std::time::Instant::now();
                std::hint::black_box(miniatura_do_preset(p));
                t0.elapsed().as_secs_f64() * 1000.0
            })
            .fold(f64::INFINITY, f64::min);
        eprintln!("miniatura {nome}: {ms:.1} ms (mín. de 5)");
    }
}

// ── A revisão da J2 (20/10): o que a mão diz e o que o meio lê ────────────────────────────────────────

use ph2d_editor_core::Tool;
use ph2d_tool_painter::Dependente;

/// Um recto inclinado (81°) com as linhas `mao` no cartão Pen, no meio `m`: os pixels e o que o painel esmaece.
fn deitado(
    m: PaintMedia,
    mao: &[(LinhaDaMao, f64)],
    automatica: bool,
) -> (Vec<u8>, ph2d_tool_painter::Inercias) {
    let (w, h) = (200u32, 80u32);
    let mut t = PainterTool::default();
    t.set_source(vec![255; (w * h * 4) as usize], w, h);
    t.set_paint_media(m);
    if m == PaintMedia::Watercolor && t.brush_settings().watercolor_shape_auto != automatica {
        t.toggle_brush_watercolor_shape_auto();
    }
    t.set_brush_size_px(8.0);
    t.set_brush_color_srgb8([20, 20, 20]);
    for &(l, v) in mao {
        t.set_mao(l, v);
    }
    let inercias = t.brush_settings().inercias;
    let cp = |x: f32, phase| CanvasPointer {
        pos: [x, 40.0],
        pressure: 0.3 + 0.7 * (x - 20.0) / 160.0,
        tilt: [0.9, 0.0],
        phase,
    };
    t.on_canvas_pointer(cp(20.0, PointerPhase::Down));
    let mut x = 20.0;
    while x < 180.0 {
        x += 2.0;
        t.on_canvas_pointer(cp(x, PointerPhase::Move));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(cp(x, PointerPhase::Up));
    for _ in 0..30 {
        t.on_tick(1000.0 / 60.0);
    }
    (t.run_full().0, inercias)
}

/// ⭐ **A Watercolor com a Shape «Automatic» não lê a pegada deitada** (03_MOTORES §6, doc 47 §6): o carimbo dela é
/// o redondo próprio (`wet_shape_stamp` só com a Automatic DESLIGADA) — Tilt width/length não mudam a tinta e
/// esmaecem; com a Automatic desligada agem e acendem.
#[test]
fn a_aquarela_automatica_nao_le_a_pegada_deitada() {
    let pegada = [
        (LinhaDaMao::Knob(Knob::TiltWidth), 2.0),
        (LinhaDaMao::Knob(Knob::TiltLength), 3.0),
    ];
    let (neutro, _) = deitado(PaintMedia::Watercolor, &[], true);
    let (com, i) = deitado(PaintMedia::Watercolor, &pegada, true);
    assert!(com == neutro, "Automatic: a pegada deitada mudou a tinta");
    assert!(i.inerte(Dependente::PegadaDaAutomatica), "Automatic: as linhas acesas");
    let (neutro, _) = deitado(PaintMedia::Watercolor, &[], false);
    let (com, i) = deitado(PaintMedia::Watercolor, &pegada, false);
    assert!(com != neutro, "sem Automatic a pegada deitada age");
    assert!(!i.inerte(Dependente::PegadaDaAutomatica));
}

/// A inclinação→opacidade não acende as linhas da caneta na Watercolor (o pincel é um nos três meios, e a lavagem não
/// lê a cobertura do dab).
#[test]
fn a_opacidade_deitada_nao_acende_a_caneta_na_aquarela() {
    // Mexida no Digital, a opacidade deitada viaja no pincel para a Watercolor.
    let mut t = PainterTool::default();
    t.set_mao(LinhaDaMao::TiltOpacity, 0.5);
    t.set_paint_media(PaintMedia::Watercolor);
    assert!(t.brush_settings().inercias.inerte(Dependente::CanetaDePe));
    let (_, i) = deitado(PaintMedia::Digital, &[(LinhaDaMao::TiltOpacity, 0.5)], true);
    assert!(!i.inerte(Dependente::CanetaDePe));
}

/// Sem pressão no tamanho nem na opacidade, a curva do tamanho, o piso, o tecto e a suavização não agem — e
/// esmaecem; com a pressão no tamanho, agem.
#[test]
fn as_linhas_da_pressao_apagam_sem_pressao() {
    let sem = [(LinhaDaMao::Knob(Knob::PressureSize), 0.0)];
    let mexe = [
        (LinhaDaMao::Knob(Knob::PressureSizeCurve), 2.0),
        (LinhaDaMao::Knob(Knob::PressureFloor), 0.4),
        (LinhaDaMao::Knob(Knob::PressureCeiling), 0.7),
        (LinhaDaMao::Knob(Knob::PressureSmoothing), 0.5),
    ];
    for m in [PaintMedia::Digital, PaintMedia::Impasto] {
        let (base, i) = deitado(m, &sem, true);
        assert!(i.inerte(Dependente::CurvaDoTamanhoSemPressao), "{m:?}");
        assert!(i.inerte(Dependente::MaoSemPressao), "{m:?}");
        let todas: Vec<_> = sem.iter().chain(&mexe).copied().collect();
        assert!(deitado(m, &todas, true).0 == base, "{m:?}: sem pressão as linhas mudaram a tinta");
        let com = [(LinhaDaMao::Knob(Knob::PressureSize), 0.5)];
        let (base, i) = deitado(m, &com, true);
        assert!(!i.inerte(Dependente::CurvaDoTamanhoSemPressao) && !i.inerte(Dependente::MaoSemPressao));
        let todas: Vec<_> = com.iter().chain(&mexe).copied().collect();
        assert!(deitado(m, &todas, true).0 != base, "{m:?}: com pressão as linhas agem");
    }
}

/// «Base de outro meio só empresta nome e grupo» também no Wet: capturar o pincel molhado sobre um preset Digital
/// não leva o gesto nem a mão do Digital para o ficheiro.
#[test]
fn a_captura_no_wet_sobre_base_de_outro_meio_so_empresta_o_nome() {
    let digital = ph2d_brush_presets::ler(&std::fs::read(caminho_do_exemplo(0)).unwrap()).unwrap();
    let mut t = PainterTool::default();
    t.set_paint_media(PaintMedia::WetPaint);
    let (p, _) = t.preset_capturar(&digital.parametros);
    assert_eq!(p.media, nome_do_meio(PaintMedia::WetPaint));
    assert_eq!((p.name.as_str(), p.group.as_str()), (digital.parametros.name.as_str(), digital.parametros.group.as_str()));
    assert!(!p.brush.contains_key("stroke_method"), "o gesto do Digital entrou no Wet");
    assert!(!p.engine.contains_key("tiltOpacity"), "a mão do Digital entrou no Wet");
}
