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

/// ⭐ **Os presets do Wet Paint não mudam**: cada um da fábrica, aplicado pela porta dos quatro meios, deixa o
/// pincel que a porta do Wet deixava (a captura inteira, o mapa do motor e as texturas ao byte), e a captura
/// dos quatro meios é a do Wet.
#[test]
fn os_presets_do_wet_nao_mudam() {
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
/// desse meio (`<motor>/…`); a fábrica tem os do Wet Paint que a receita conta e nenhum dos outros (os pincéis
/// deles chegam com as janelas 3–4, e este portão conta-os), os exemplos um por meio antigo.
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
        [0, 0, 0, crate::receitas_de_fabrica::PRESETS_DE_FABRICA]
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
