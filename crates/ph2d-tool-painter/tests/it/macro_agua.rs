//! ⭐ **Gate 1 das macros: o Wet Paint gravado com a água no WORKER reproduz-se AO BYTE** (plano
//! `docs/Macros/01_plano_onda1.md` §5.1).
//!
//! A fixture tem o fenómeno: traços molhados com esperas REAIS, a água a correr na thread dela, e a
//! gravação repete-se até um traço pousar com um passo EM VOO (a marca com estágio) — é aí que a
//! reprodução por segundos, ou por passos inteiros, divergia (sonda `sonda_macro_bcd_reproducao`:
//! 24 239 texels entre corridas, 29 891 na reprodução sem drenar). Verde = 0 texels no fim e depois
//! de +40 passos. Controlo positivo: uma marca da fita adulterada num estágio ⇒ a reprodução muda.

use super::macro_comum::{Gravador, cp, texels_diferentes, toca};
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool, Tool};
use ph2d_tool_painter::PaintMedia;
use ph2d_tool_painter::PainterTool;
use ph2d_tool_painter::macro_track::{MarcaDaAgua, Passo, RespostaDoWorker};
use std::time::{Duration, Instant};

const N: u32 = 512;
const QUADRO: Duration = Duration::from_micros(16_666);
const AMOSTRAS: u32 = 30;
const POR_QUADRO: u32 = 3;
/// `(de, até, cor)` em fracções da tela. Seis toques por gravação: o toque cai a meio de um passo da
/// água só quando o worker está A CORRER um (ele dorme entre passos), então cada toque é uma tentativa.
const ROTEIRO: [([f32; 2], [f32; 2], [u8; 3]); 6] = [
    ([0.15, 0.4], [0.85, 0.4], [200, 50, 30]),
    ([0.2, 0.2], [0.8, 0.7], [30, 80, 200]),
    ([0.5, 0.15], [0.5, 0.85], [230, 200, 30]),
    ([0.15, 0.6], [0.85, 0.62], [40, 160, 70]),
    ([0.3, 0.8], [0.7, 0.3], [150, 40, 160]),
    ([0.25, 0.5], [0.75, 0.5], [20, 20, 20]),
];
const ESPERAS_MS: [u64; 6] = [150, 300, 120, 250, 90, 200];

fn ferramenta() -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (N * N * 4) as usize], N, N);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_brush_size_px(30.0 * N as f32 / 1024.0);
    t
}

/// Um quadro do app: os eventos, a pré-visualização que a ponte drena, o tique, e o resto do quadro.
fn quadro(t: &mut PainterTool, g: &mut Gravador, eventos: &[CanvasPointer]) {
    let a = Instant::now();
    for &e in eventos {
        t.on_canvas_pointer(e);
    }
    let _ = t.take_preview_arc();
    t.on_tick(1000.0 / 60.0);
    g.quadro(t);
    if let Some(r) = QUADRO.checked_sub(a.elapsed()) {
        std::thread::sleep(r);
    }
}

fn corrida() -> (PainterTool, ph2d_macro::MacroDoc) {
    let mut t = ferramenta();
    let mut g = Gravador::arma(&mut t);
    let s = N as f32;
    for (k, (de, ate, cor)) in ROTEIRO.iter().enumerate() {
        t.set_brush_color_srgb8(*cor);
        let p = |i: u32| {
            let f = i as f32 / AMOSTRAS as f32;
            [
                s * (de[0] + (ate[0] - de[0]) * f),
                s * (de[1] + (ate[1] - de[1]) * f),
            ]
        };
        quadro(&mut t, &mut g, &[cp(p(0), PointerPhase::Down)]);
        for q in 0..AMOSTRAS / POR_QUADRO {
            let ev: Vec<_> = (1..=POR_QUADRO)
                .map(|m| cp(p(q * POR_QUADRO + m), PointerPhase::Move))
                .collect();
            quadro(&mut t, &mut g, &ev);
        }
        quadro(&mut t, &mut g, &[cp(p(AMOSTRAS), PointerPhase::Up)]);
        let fim = Instant::now() + Duration::from_millis(ESPERAS_MS[k]);
        while Instant::now() < fim {
            quadro(&mut t, &mut g, &[]);
        }
    }
    let doc = g.para(&mut t);
    (t, doc)
}

fn respostas(doc: &ph2d_macro::MacroDoc) -> Vec<Passo> {
    doc.tracks[0]
        .events
        .iter()
        .map(|e| Passo::descodifica(&e.payload).expect("carga lê-se"))
        .filter(|p| matches!(p, Passo::Agua(_)))
        .collect()
}

/// O índice do evento que DECIDE: o motor volta com um passo EM VOO no quadro de um `Down` (o 1.º lote
/// do traço pousa a meio do passo). Uma marca entre traços não decide nada — a água anda o mesmo e só o
/// composite muda de quadro.
fn marca_no_toque(doc: &ph2d_macro::MacroDoc) -> Option<usize> {
    let ev = &doc.tracks[0].events;
    let passo = |i: usize| Passo::descodifica(&ev[i].payload).expect("carga lê-se");
    let downs: Vec<_> = (0..ev.len())
        .filter(|&i| {
            matches!(
                passo(i),
                Passo::Ponteiro {
                    fase: PointerPhase::Down,
                    ..
                }
            )
        })
        .map(|i| ev[i].t)
        .collect();
    (0..ev.len()).find(|&i| {
        downs.contains(&ev[i].t)
            && matches!(
                passo(i),
                Passo::Agua(RespostaDoWorker::Voltou(MarcaDaAgua {
                    estagio: Some(_),
                    ..
                }))
            )
    })
}

/// Grava até a fixture conter o fenómeno (um toque a meio de um passo).
fn corrida_com_o_fenomeno() -> (PainterTool, ph2d_macro::MacroDoc, usize) {
    for _ in 0..10 {
        let (t, doc) = corrida();
        if let Some(i) = marca_no_toque(&doc) {
            return (t, doc, i);
        }
    }
    panic!(
        "em 10 gravações (60 toques) nenhum traço pousou com a água a meio de um passo: a fixture não \
         contém o fenómeno (a água não correu no worker?)"
    )
}

/// +40 passos no relógio fixo e o bake — a água escondida que a tela ainda não mostra.
fn depois_de_40(t: &mut PainterTool) -> Vec<u8> {
    t.set_wet_relogio_fixo(true);
    for _ in 0..40 {
        t.on_tick(1000.0 / 40.0);
    }
    t.run_full().0
}

#[test]
fn o_wet_paint_gravado_no_worker_reproduz_se_ao_byte() {
    let (mut original, doc, _) = corrida_com_o_fenomeno();
    assert!(
        respostas(&doc).len() > 3,
        "a gravação quase não tem respostas do worker — a água não correu na thread dela"
    );
    let codificado = ph2d_macro::codec::write(&doc);
    let doc = ph2d_macro::codec::read(&codificado).expect("o ficheiro lê-se");

    let mut toc = toca(&doc).expect("a reprodução não diverge");
    assert_eq!(
        toc.respostas_por_pedir(),
        0,
        "a reprodução terminou com respostas do worker por pedir: ela fez MENOS perguntas que a gravação"
    );
    let a = original
        .take_preview_arc()
        .map(|(p, ..)| p.as_ref().clone());
    let b = toc
        .tool_mut()
        .take_preview_arc()
        .map(|(p, ..)| p.as_ref().clone());
    assert_eq!(
        a.is_some(),
        b.is_some(),
        "uma das telas tem composite pendente e a outra não"
    );
    if let (Some(a), Some(b)) = (&a, &b) {
        assert_eq!(
            texels_diferentes(a, b),
            0,
            "a tela no fim da gravação difere"
        );
    }
    let (fa, fb) = (depois_de_40(&mut original), depois_de_40(toc.tool_mut()));
    assert_eq!(
        texels_diferentes(&fa, &fb),
        0,
        "depois de +40 passos a água das duas difere (a reprodução não é a gravação)"
    );
}

/// Controlo positivo da régua: a mesma gravação com UMA marca da fita adiantada um estágio muda a
/// reprodução (ou acusa a divergência) — a régua vê a diferença que o gate afirma não haver.
#[test]
fn uma_marca_da_fita_adulterada_muda_a_reproducao() {
    let (_, doc, i) = corrida_com_o_fenomeno();
    let limpo = toca(&doc).map(|mut t| depois_de_40(t.tool_mut()));
    let mut adulterado = doc.clone();
    let alvo = &mut adulterado.tracks[0].events[i];
    let Ok(Passo::Agua(RespostaDoWorker::Voltou(m))) = Passo::descodifica(&alvo.payload) else {
        unreachable!()
    };
    let mais_um = MarcaDaAgua {
        frame: m.frame + 1,
        estagio: m.estagio,
    };
    alvo.payload = Passo::Agua(RespostaDoWorker::Voltou(mais_um)).codifica();
    let sujo = toca(&adulterado).map(|mut t| depois_de_40(t.tool_mut()));
    let limpo = limpo.expect("a reprodução limpa corre");
    match sujo {
        Err(_) => {} // a divergência foi acusada: a régua viu
        Ok(sujo) => assert!(
            texels_diferentes(&limpo, &sujo) > 0,
            "uma marca adulterada deu a MESMA tela: a fita não decide a água, e o gate verde não prova nada"
        ),
    }
}
