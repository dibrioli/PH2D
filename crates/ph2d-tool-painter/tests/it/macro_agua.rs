//! ⭐ **Gate 1 das macros: o Wet Paint gravado com a água no WORKER reproduz-se AO BYTE** (plano
//! `docs/Macros/01_plano_onda1.md` §5.1).
//!
//! A fixture tem o fenómeno: traços molhados, a água a correr na thread dela, e traços que pousam com
//! um passo EM VOO (a marca com estágio) — é aí que a reprodução por segundos, ou por passos inteiros,
//! divergia (sonda `sonda_macro_bcd_reproducao`: 24 239 texels entre corridas, 29 891 na reprodução
//! sem drenar). Verde = 0 texels no fim e depois de +40 passos. Controlo positivo: uma marca da fita
//! adulterada num estágio ⇒ a reprodução muda.
//!
//! O worker anda num RITMO ESCRITO ([`RitmoDaAgua`], W7), não ao relógio: com esperas reais o toque a
//! meio de um passo era sorte da máquina (vermelho sozinho a `load ~4`, verde sob carga). O ritmo põe
//! o fenómeno na fixture À PRIMEIRA, com pedidos que voltam de mãos vazias pelo meio.

use super::macro_comum::{grava_no_worker_e_o_tool, texels_diferentes, toca};
use ph2d_editor_core::tool::{PointerPhase, RasterEditTool, Tool};
use ph2d_tool_painter::PainterTool;
use ph2d_tool_painter::macro_track::{MarcaDaAgua, Passo, RespostaDoWorker};

const N: u32 = 512;

fn corrida() -> (PainterTool, ph2d_macro::MacroDoc) {
    grava_no_worker_e_o_tool(N)
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

/// A gravação, que TEM de conter o fenómeno (um toque a meio de um passo) e um pedido sem espera que
/// voltou de mãos vazias.
fn corrida_com_o_fenomeno() -> (PainterTool, ph2d_macro::MacroDoc, usize) {
    let (t, doc) = corrida();
    let Some(i) = marca_no_toque(&doc) else {
        panic!(
            "nenhum dos 6 traços pousou com a água a meio de um passo: a fixture não contém o fenómeno \
             (o ritmo do worker deu passos inteiros?)"
        )
    };
    assert!(
        doc.tracks[0].events.iter().any(|e| matches!(
            Passo::descodifica(&e.payload),
            Ok(Passo::Agua(RespostaDoWorker::NaoVoltou))
        )),
        "nenhum pedido voltou de mãos vazias: a fixture não exercita a resposta que adia o lote"
    );
    (t, doc, i)
}

/// O ritmo é a lei do gate: a mesma gravação duas vezes, ao byte (com o relógio, cada corrida era outra).
#[test]
fn a_gravacao_no_worker_ritmado_e_a_mesma_duas_vezes() {
    let (_, a) = corrida();
    let (_, b) = corrida();
    assert_eq!(
        ph2d_macro::codec::write(&a),
        ph2d_macro::codec::write(&b),
        "duas gravações com o mesmo ritmo diferem: algo da água ainda anda ao relógio da máquina"
    );
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
