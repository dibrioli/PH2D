//! Os gates de unidade da trilha: o formato de cada passo, a fita, e a versão do documento.

use super::passo::Passo;
use crate::tool::PainterTool;
use crate::tool::paint::fita::{
    Divergencia, Fita, MarcaDaAgua, Pergunta, RespostaDoWorker, anda_ate,
};
use ph2d_a11y::NodeId;
use ph2d_editor_core::tool::RasterEditTool;
use ph2d_editor_core::tool::{PanelEvent, PointerPhase};
use std::collections::VecDeque;

fn todos_os_passos() -> Vec<Passo> {
    let marca = MarcaDaAgua {
        frame: 41,
        estagio: Some(3),
    };
    let mut warp = ph2d_painter_brush::canvas_warp::CanvasWarp::rest();
    warp.linear[0][1] = 0.25;
    warp.curve[1][6] = -1.5;
    vec![
        Passo::Ponteiro {
            pos: [0.123_456_789, 0.987_654_321],
            pressao: 0.7,
            tilt: [0.1, -0.2],
            fase: PointerPhase::Up,
        },
        Passo::Hover([0.5, 0.25]),
        Passo::Tique(16.666),
        Passo::Painel(PanelEvent::Click(NodeId(7))),
        Passo::Painel(PanelEvent::SetValue(NodeId(u64::MAX), -0.5)),
        Passo::Painel(PanelEvent::Toggle(NodeId(9), true)),
        Passo::Painel(PanelEvent::SelectOption(NodeId(11), "Smear".to_owned())),
        Passo::Ativa,
        Passo::Desativa,
        Passo::Desfaz,
        Passo::Refaz,
        Passo::PedeCommit,
        Passo::TiraCommit,
        Passo::Assado,
        Passo::TiraAssadoAdiado,
        Passo::Preview,
        Passo::PreviewBbox,
        Passo::PreviewSujo,
        Passo::PreviewAtual,
        Passo::Completo,
        Passo::Borracha,
        Passo::TamanhoPx(12.5),
        Passo::EmpurraTamanho(-3),
        Passo::Cor([1, 2, 3]),
        Passo::RestringeLinha(true),
        Passo::Warp(warp),
        Passo::GridSnap(Some([0.3, 0.6])),
        Passo::GridSnap(None),
        Passo::GridApaga(true),
        Passo::ComporPorQuadro(false),
        Passo::TolDePega(4.0),
        Passo::Modificadores([true, false, true]),
        Passo::SeguraRascunho(true),
        Passo::PresetAplica {
            parametros: "{\"schema\":1}".to_owned(),
            texturas: vec![("shape".to_owned(), vec![9, 8, 7])],
        },
        Passo::PresetVolumes("{}".to_owned()),
        Passo::PresetModo("Blend".to_owned()),
        Passo::TiraPedidoPreset,
        Passo::TiraPedidoGrao,
        Passo::TiraPedidoForma,
        Passo::TexturaGrao {
            lum: vec![1, 2, 3, 4],
            w: 2,
            h: 2,
        },
        Passo::FormaRgba {
            rgba: vec![5; 16],
            w: 2,
            h: 2,
        },
        Passo::FalloffPonto {
            id: 2,
            d: 0.4,
            s: 0.9,
        },
        Passo::FalloffAcrescenta { d: 0.1, s: 0.2 },
        Passo::FalloffPega { id: 3, handle: 1 },
        Passo::FalloffTira(4),
        Passo::RefrescaFonteDaForma,
        Passo::RefrescaCorDaForma,
        Passo::TiraPreviewDaFonte,
        Passo::TipoDaTextura(2),
        Passo::TipoDaForma(1),
        Passo::SnapDaLinha(true),
        Passo::EscalaUniforme(false),
        Passo::Para,
        Passo::Agua(RespostaDoWorker::Voltou(marca)),
        Passo::Agua(RespostaDoWorker::Voltou(MarcaDaAgua {
            frame: 0,
            estagio: None,
        })),
        Passo::Agua(RespostaDoWorker::NaoVoltou),
        Passo::Agua(RespostaDoWorker::Passos(1 << 40)),
        Passo::Recusada("bind_document".to_owned()),
        Passo::SaiDaGravacao,
    ]
}

#[test]
fn cada_passo_vai_e_volta_ao_bit() {
    for p in todos_os_passos() {
        let b = p.codifica();
        assert_eq!(
            Passo::descodifica(&b).expect("lê-se"),
            p,
            "ida e volta de {p:?}"
        );
    }
}

/// Um tipo de uma versão mais nova lê-se como DESCONHECIDO (o tocador recusa-o), nunca como outro.
#[test]
fn um_tipo_desconhecido_le_se_como_desconhecido() {
    let mut r = ph2d_macro::rec::Record::new();
    r.varint(1, 9_999).f32(2, 1.0);
    assert_eq!(
        Passo::descodifica(&r.finish()).expect("lê-se"),
        Passo::Desconhecido(9_999)
    );
}

/// O campo que um leitor antigo não conhece SALTA: o passo lê-se igual.
#[test]
fn um_campo_novo_dentro_de_um_passo_salta() {
    let mut r = ph2d_macro::rec::Record::new();
    r.varint(1, 3).f32(2, 8.0).bytes(77, b"futuro");
    assert_eq!(
        Passo::descodifica(&r.finish()).expect("lê-se"),
        Passo::Tique(8.0)
    );
}

#[test]
fn a_fita_acusa_a_pergunta_trocada_e_a_fita_acabada() {
    let mut f = Fita::Toca {
        fila: VecDeque::from([RespostaDoWorker::Passos(3)]),
        divergiu: None,
    };
    assert_eq!(f.responde(Pergunta::Volta), None);
    assert!(matches!(
        f,
        Fita::Toca {
            divergiu: Some(Divergencia::PerguntaInesperada),
            ..
        }
    ));
    let mut f = Fita::Toca {
        fila: VecDeque::new(),
        divergiu: None,
    };
    assert_eq!(f.responde(Pergunta::Passos), None);
    assert!(matches!(
        f,
        Fita::Toca {
            divergiu: Some(Divergencia::FitaAcabou),
            ..
        }
    ));
    let mut f = Fita::Toca {
        fila: VecDeque::from([RespostaDoWorker::NaoVoltou]),
        divergiu: None,
    };
    assert_eq!(
        f.responde(Pergunta::TentaVoltar),
        Some(RespostaDoWorker::NaoVoltou)
    );
}

/// O motor anda até a marca pelo MESMO estágio que o worker corre; uma marca atrás, ou além de onde a
/// água pára, é inalcançável (nunca um laço sem fim).
#[test]
fn o_motor_anda_ate_a_marca_e_recusa_a_inalcancavel() {
    let mut e = ph2d_wet_paint::painter::Engine::new(64, 64);
    e.sim.gravity_override = Some([0.0, 1.0]);
    e.begin_direct_stroke(0, 16.0, 24.0);
    for x in [16.0, 20.0, 24.0, 28.0, 32.0] {
        e.direct_segment(0, 4.0);
        e.dispatch_pressure_dab_lane(0, x, 24.0, 5.0, 1.0, 0.0, 8.0, None, None, None);
    }
    e.end_direct_stroke();
    let alvo = MarcaDaAgua {
        frame: 3,
        estagio: Some(2),
    };
    assert!(
        anda_ate(&mut e, alvo),
        "a água chega ao 3.º passo, estágio 2"
    );
    assert_eq!(MarcaDaAgua::de(&e), alvo);
    assert!(
        !anda_ate(
            &mut e,
            MarcaDaAgua {
                frame: 1,
                estagio: None
            }
        ),
        "uma marca para trás é inalcançável"
    );
    let mut seco = ph2d_wet_paint::painter::Engine::new(16, 16);
    assert!(
        !anda_ate(
            &mut seco,
            MarcaDaAgua {
                frame: 5,
                estagio: None
            }
        ),
        "sem água o motor não anda: a marca à frente é inalcançável"
    );
}

/// ⚠️ **O documento do estado inicial é POSTCARD (posicional)**: um campo novo no `PaintedDocument`
/// (ou no `LayerStack`) muda os bytes e um estado inicial antigo deixaria de se ler — ou leria errado.
/// Esta impressão digital prende o formato à [`super::inicial::VERSAO_DO_DOCUMENTO`]: mudou, suba a
/// versão (e escreva a leitura da anterior) e só então troque o número aqui.
#[test]
fn o_documento_da_macro_tem_a_versao_do_seu_formato() {
    let mut t = PainterTool::default();
    t.set_source((0..4 * 4 * 4).map(|i| (i * 7) as u8).collect(), 4, 4);
    let doc = t.documento_vivo(5);
    let b = postcard::to_allocvec(&doc).expect("serializa");
    let fnv = b.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, &x| {
        (h ^ u64::from(x)).wrapping_mul(0x0100_0000_01b3)
    });
    assert_eq!(
        (super::inicial::VERSAO_DO_DOCUMENTO, fnv),
        (1, IMPRESSAO_DO_DOCUMENTO_V1),
        "o formato do documento mudou ({fnv:#018x}): suba `VERSAO_DO_DOCUMENTO`"
    );
}

/// A impressão do `PaintedDocument` da versão 1 (medida em 2026-10-13, base `757afebd58`).
const IMPRESSAO_DO_DOCUMENTO_V1: u64 = 0xf6e6_904f_ffdb_a430;

/// ⭐ **Um estado inicial de ANTES da W6c** (sem `meio`, `modo`, `pinceis`, `ligados`,
/// `falloff_armado`, `metodo_por_meio` nem `tela` no JSON das configurações) lê-se como Wet Paint e
/// reproduz o Splash na MESMA tela que o estado inteiro.
#[test]
fn um_estado_inicial_de_antes_da_w6c_reproduz_o_mesmo_splash() {
    use ph2d_macro::rec::{self, Record};
    let (b, _) =
        super::roteiro::grava_pinceladas(128, 0, &super::roteiro::SPLASH, &[0; 4]).expect("grava");
    let track = ph2d_macro::codec::read(&b).expect("lê").tracks.remove(0);
    let mut velho = Record::new();
    let mut tirou = 0;
    for (tag, v) in rec::fields(&track.initial).expect("campos") {
        match tag {
            1 | 5 => {
                velho.varint(tag, rec::as_varint(v).expect("varint"));
            }
            2 => {
                let mut cfg: serde_json::Value =
                    serde_json::from_str(rec::as_str(v).expect("texto")).expect("json");
                let m = cfg.as_object_mut().expect("objecto");
                for k in [
                    "meio",
                    "modo",
                    "pinceis",
                    "ligados",
                    "falloff_armado",
                    "metodo_por_meio",
                    "tela",
                ] {
                    tirou += usize::from(m.remove(k).is_some());
                }
                velho.str(tag, &cfg.to_string());
            }
            _ => {
                velho.bytes(tag, v);
            }
        }
    }
    assert_eq!(tirou, 7, "o estado de agora leva as sete chaves da W6c");
    let velha = ph2d_macro::Track {
        initial: velho.finish(),
        ..track.clone()
    };
    let t = super::inicial::tool_no_inicial(&velha.initial).expect("o velho lê-se");
    assert_eq!(t.paint_media(), crate::PaintMedia::WetPaint);
    assert!(
        super::etapas::tela_no_fim(&velha).expect("toca")
            == super::etapas::tela_no_fim(&track).expect("toca"),
        "o estado de antes da W6c reproduz outra tela"
    );
}
