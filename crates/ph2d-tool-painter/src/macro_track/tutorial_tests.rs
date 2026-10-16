//! Os gates do tutorial «fazer» do Painter (W6b, `docs/Macros/03_plano_W6.md` §6): o Splash gravado
//! a 512², um aluno no estado inicial.

use super::sonda::{MEIOS, Variante, cada_aluno, criterio_de, quadro, trilha_no_meio, trinta};
use super::*;
use crate::macro_track::roteiro::{CAUDA, SPLASH, grava_nos_meios, grava_pinceladas};
use ph2d_editor_core::tool::{PanelEvent, Tool};

const LADO: u32 = 512;
/// A entidade a que o tool no estado inicial liga o documento.
const ALVO: u64 = inicial::ENTIDADE_DO_TOCADOR;

fn trilha() -> Track {
    let (b, _) = grava_pinceladas(LADO, 0, &SPLASH, &[0; 4]).expect("grava");
    ph2d_macro::codec::read(&b).expect("lê").tracks.remove(0)
}

fn assenta(t: &mut PainterTool) {
    for _ in 0..CAUDA {
        quadro(t);
    }
}

/// Cada pincelada cumpre-se só quando feita («Mostra-me» = a gravada), e o Ctrl+Z volta o passo — nos
/// quatro meios.
#[test]
fn cada_pincelada_so_se_cumpre_feita_e_o_desfazer_a_volta() {
    for m in MEIOS {
        let track = trilha_no_meio(m, &SPLASH, LADO);
        let e = EnsinoDoPainter::medido(&track).expect("ensino");
        assert_eq!(e.criterios.len(), 4, "{m:?}");
        let mut t = inicial::tool_no_inicial(&track.initial).expect("inicial");
        assert_eq!(e.por_cumprir(&t), Some(0), "{m:?}: nada feito");
        for (i, c) in e.criterios.iter().enumerate() {
            mostra(&mut t, &track, c.regiao.k).expect("mostra");
            assenta(&mut t);
            assert_eq!(
                e.por_cumprir(&t),
                (i + 1 < 4).then_some(i + 1),
                "{m:?}: depois da {i}"
            );
        }
        assert!(t.undo_last(), "a última pincelada desfaz-se");
        assenta(&mut t);
        assert_eq!(
            e.por_cumprir(&t),
            Some(3),
            "{m:?}: o desfazer volta o passo"
        );
        assert!(t.redo_last());
        assenta(&mut t);
        assert_eq!(e.por_cumprir(&t), None, "{m:?}: o refazer avança");
    }
}

/// ⭐ A fracção SOBRE A DA GRAVADA separa, nos quatro meios e nas quatro pinceladas do Splash, a mão
/// torta meio raio (cumpre) do pincel fino e do nada (não cumprem).
#[test]
fn a_mao_torta_cumpre_o_pincel_fino_e_o_nada_nao() {
    use Variante::{Desloca, Fina, Nada};
    for m in MEIOS {
        let track = trilha_no_meio(m, &SPLASH, LADO);
        let e = EnsinoDoPainter::medido(&track).expect("ensino");
        cada_aluno(
            &track,
            &SPLASH,
            LADO,
            &[Desloca(0.5), Fina, Nada],
            |k, v, t| {
                let c = criterio_de(&e, &track, k).expect("critério");
                let r = e.razao(t, c).expect("medida");
                let cumpre = r >= FRACAO_DA_GRAVADA_QUE_CUMPRE;
                assert_eq!(
                    cumpre,
                    matches!(v, Desloca(_)),
                    "{m:?} · pincelada {k} · {v:?}: {r}"
                );
            },
        );
    }
}

/// ⭐ **A fatia fina** (W6c): a 2.ª pincelada é a 1.ª deslocada 13 px — a parte nova dela é uma fatia
/// na orla, que nem a gravada pinta toda. Pela fracção absoluta o «Mostra-me» não a cumpria; sobre a
/// da gravada cumpre, nos quatro meios.
#[test]
fn o_mostra_me_cumpre_a_fatia_fina() {
    let ps = [trinta()[0], trinta()[4]];
    for m in MEIOS {
        let track = trilha_no_meio(m, &ps, LADO);
        let e = EnsinoDoPainter::medido(&track).expect("ensino");
        let c = &e.criterios[1];
        let toda = rasteriza(&c.regiao, LADO, LADO).len();
        assert!(c.novos.len() * 10 < toda, "{m:?}: a parte nova é uma fatia");
        let mut t = inicial::tool_no_inicial(&track.initial).expect("inicial");
        for c in &e.criterios {
            mostra(&mut t, &track, c.regiao.k).expect("mostra");
            assenta(&mut t);
        }
        let r = e.razao(&t, c).expect("medida");
        assert!(r >= FRACAO_DA_GRAVADA_QUE_CUMPRE, "{m:?}: {r}");
        assert_eq!(e.por_cumprir(&t), None, "{m:?}");
    }
}

/// ⭐ **Pular a pincelada e esperar não a cumpre** (W6d): nas 30 pinceladas em Wet Paint, o aluno que
/// fez as anteriores certas NÃO faz a `k` e espera a água assentar — a aguada da anterior pinta sozinha
/// a fatia nova. Nenhuma das 30 se cumpre — nem pela razão, nem pela porta do balão
/// ([`EnsinoDoPainter::por_cumprir`] pára na 1.ª com critério desde a `k`: a base, que a aguada das
/// outras pinta sozinha, desconta-se lá também).
#[test]
fn pular_a_pincelada_e_esperar_nao_cumpre_nenhuma() {
    let ps = trinta();
    let track = trilha_no_meio(PaintMedia::WetPaint, &ps, LADO);
    let e = EnsinoDoPainter::medido(&track).expect("ensino");
    let ks: Vec<usize> = regioes(&track, LADO, LADO)
        .expect("regiões")
        .iter()
        .map(|r| r.k)
        .collect();
    let mut cumpre = Vec::new();
    let mut passa = Vec::new();
    cada_aluno(&track, &ps, LADO, &[Variante::Nada], |k, _, t| {
        if let Some(c) = criterio_de(&e, &track, k)
            && e.razao(t, c).expect("medida") >= FRACAO_DA_GRAVADA_QUE_CUMPRE
        {
            cumpre.push(k);
        }
        let esperada = e
            .criterios
            .iter()
            .position(|c| ks[k..].contains(&c.regiao.k));
        let balao = e.por_cumprir(t);
        if balao != esperada {
            passa.push((k, esperada, balao));
        }
    });
    assert!(
        cumpre.is_empty(),
        "o «Nada» cumpre as pinceladas {cumpre:?}"
    );
    assert!(
        passa.is_empty(),
        "o balão passa a pincelada pulada (k, esperada, por_cumprir): {passa:?}"
    );
}

/// ⭐ **Outra tela não se avalia, nem por dentro**: [`EnsinoDoPainter::fracao_pintada`] e
/// [`EnsinoDoPainter::por_cumprir`] chamados com um Painter de outro tamanho (sem passar por
/// [`EnsinoDoPainter::aberto`]) — toda a tela pintada — dão fracção 0 e a 1.ª pincelada por cumprir.
#[test]
fn outra_tela_nao_se_avalia_nem_chamada_por_dentro() {
    let track = trilha();
    let mut e = EnsinoDoPainter::de(&track).expect("ensino");
    let ks: Vec<usize> = e.criterios.iter().map(|c| c.regiao.k).collect();
    for k in ks {
        assert!(e.com_medida(Medida {
            k,
            referencia: Referencia {
                gravada: 0.9,
                base: 0.0,
            },
        }));
    }
    assert!(!e.a_medir());
    // Maior que a gravada: os índices dos critérios caem dentro dela, toda pintada e opaca.
    let lado = LADO + 64;
    let mut t = PainterTool::default();
    t.bind_document(
        ALVO,
        [40, 80, 120, 255].repeat(lado as usize * lado as usize),
        lado,
        lado,
    );
    let pintadas: Vec<(usize, f32)> = e
        .criterios
        .iter()
        .map(|c| e.fracao_pintada(&t, c))
        .enumerate()
        .filter(|&(_, f)| f > 0.0)
        .collect();
    assert_eq!(
        (pintadas, e.por_cumprir(&t)),
        (Vec::new(), Some(0)),
        "outra tela leu-se como pintada (pincelada, fracção) e cumpriu pinceladas"
    );
}

/// Sem as medidas (ainda por chegar), nenhuma pincelada se cumpre — nem as feitas pelo «Mostra-me»;
/// uma medida de outra pincelada recusa-se; elas chegam uma a uma.
#[test]
fn sem_medidas_nenhuma_pincelada_se_cumpre() {
    let track = trilha();
    let mut e = EnsinoDoPainter::de(&track).expect("ensino");
    assert!(e.a_medir());
    let mut t = inicial::tool_no_inicial(&track.initial).expect("inicial");
    for c in &e.criterios {
        mostra(&mut t, &track, c.regiao.k).expect("mostra");
    }
    assenta(&mut t);
    assert_eq!(e.por_cumprir(&t), Some(0));
    let r = Referencia {
        gravada: 0.9,
        base: 0.0,
    };
    assert!(!e.com_medida(Medida {
        k: 999,
        referencia: r
    }));
    let mut ms = Vec::new();
    assert_eq!(
        e.mede(&track, &AtomicBool::new(false), |m| ms.push(m)),
        Ok(true)
    );
    assert_eq!(ms.len(), 4);
    assert!(e.com_medida(ms[0]));
    assert!(!e.com_medida(ms[0]), "a mesma medida duas vezes");
    assert_eq!(e.chegadas(), 1);
    assert_eq!(e.por_cumprir(&t), Some(1), "só a 1.ª medida chegou");
    for m in &ms[1..] {
        assert!(e.com_medida(*m));
    }
    assert!(!e.a_medir());
    assert_eq!(e.por_cumprir(&t), None);
}

/// Uma pincelada que não acrescenta nada à água (a gravada pinta da parte nova o que a trilha sem ela
/// já pinta; ou uma borracha no papel em branco) não tem critério — a lei do Δ vazio.
#[test]
fn uma_pincelada_que_nao_acrescenta_a_agua_nao_tem_criterio() {
    let track = trilha();
    let mut e = EnsinoDoPainter::de(&track).expect("ensino");
    let ks: Vec<usize> = e.criterios.iter().map(|c| c.regiao.k).collect();
    let medidas = [(0.9, 0.1), (0.4, 0.4), (0.8, 0.0), (0.0, 0.0)];
    for (&k, (gravada, base)) in ks.iter().zip(medidas) {
        assert!(e.com_medida(Medida {
            k,
            referencia: Referencia { gravada, base },
        }));
    }
    let ficam: Vec<usize> = e.criterios.iter().map(|c| c.regiao.k).collect();
    assert_eq!(ficam, [ks[0], ks[2]]);
    assert!(!e.a_medir());
}

/// Um pedido de paragem a meio devolve `false` (a `Macro` mudou: a medição é de outra versão).
#[test]
fn a_medicao_para_a_pedido() {
    let track = trilha();
    let e = EnsinoDoPainter::de(&track).expect("ensino");
    let parar = AtomicBool::new(true);
    assert_eq!(
        e.mede(&track, &parar, |_| panic!("nenhuma medida")),
        Ok(false)
    );
}

/// A parte nova de uma pincelada não tem píxeis de uma anterior.
#[test]
fn a_parte_nova_nao_conta_o_que_a_anterior_cobre() {
    let e = EnsinoDoPainter::de(&trilha()).expect("ensino");
    let antes: std::collections::BTreeSet<u32> = rasteriza(&e.criterios[0].regiao, LADO, LADO)
        .into_iter()
        .collect();
    let novos = &e.criterios[1].novos;
    assert!(!novos.is_empty());
    assert!(novos.iter().all(|i| !antes.contains(i)));
    let toda = rasteriza(&e.criterios[1].regiao, LADO, LADO);
    assert!(novos.len() < toda.len(), "as duas cruzam-se no Splash");
}

/// Antes das pinceladas: o Painter na imagem do tutorial, depois o meio gravado.
#[test]
fn o_preparo_pede_a_imagem_e_depois_o_meio() {
    let track = trilha();
    let e = EnsinoDoPainter::de(&track).expect("ensino");
    let mut t = inicial::tool_no_inicial(&track.initial).expect("inicial");
    assert_eq!(e.preparo(None, true, ALVO), Some(Preparo::AbreParaPintar));
    assert_eq!(
        e.preparo(Some(&t), false, ALVO),
        Some(Preparo::AbreParaPintar),
        "outra ferramenta na mão"
    );
    assert_eq!(
        e.preparo(Some(&t), true, ALVO + 1),
        Some(Preparo::AbreParaPintar),
        "outra imagem"
    );
    assert_eq!(e.preparo(Some(&t), true, ALVO), None);
    t.set_paint_media(PaintMedia::Digital);
    assert_eq!(
        e.preparo(Some(&t), true, ALVO),
        Some(Preparo::Meio(PaintMedia::WetPaint))
    );
}

/// Uma imagem de outro tamanho que a gravada não se avalia: o preparo di-lo (com o tamanho gravado), e
/// o «Recomeçar» recusa-a.
#[test]
fn outra_tela_recusa_se_e_diz_o_tamanho_gravado() {
    let track = trilha();
    let e = EnsinoDoPainter::de(&track).expect("ensino");
    let n = 256 * 256 * 4;
    let mut t = PainterTool::default();
    t.bind_document(ALVO, vec![255; n], 256, 256);
    assert_eq!(
        e.preparo(Some(&t), true, ALVO),
        Some(Preparo::OutroTamanho { w: LADO, h: LADO })
    );
    assert!(recomeca(&mut t, &track).is_err());
}

/// ⭐ **O meio é o da PINCELADA** (W6c): a 1.ª gravada em Wet Paint, a troca pelo seletor do painel
/// (um passo gravado), a 2.ª em Watercolor. O aluno que fez a 1.ª e trocou para Watercolor pinta a 2.ª;
/// em Wet Paint, o preparo pede o Watercolor.
#[test]
fn o_meio_pedido_e_o_da_pincelada_corrente() {
    let (b, _) = grava_nos_meios(
        LADO,
        &SPLASH[..2],
        &[PaintMedia::WetPaint, PaintMedia::Watercolor],
    )
    .expect("grava");
    let track = ph2d_macro::codec::read(&b).expect("lê").tracks.remove(0);
    let e = EnsinoDoPainter::medido(&track).expect("ensino");
    let meios: Vec<PaintMedia> = e.criterios.iter().map(|c| c.regiao.meio).collect();
    assert_eq!(meios, [PaintMedia::WetPaint, PaintMedia::Watercolor]);
    let mut t = inicial::tool_no_inicial(&track.initial).expect("inicial");
    assert_eq!(e.preparo(Some(&t), true, ALVO), None, "a 1.ª em Wet Paint");
    mostra(&mut t, &track, e.criterios[0].regiao.k).expect("mostra");
    assenta(&mut t);
    assert_eq!(e.por_cumprir(&t), Some(1));
    assert_eq!(
        e.preparo(Some(&t), true, ALVO),
        Some(Preparo::Meio(PaintMedia::Watercolor)),
        "a 2.ª pede o meio dela"
    );
    t.handle_panel_event(PanelEvent::SelectOption(
        crate::ids::PAINTER_BRUSH_MEDIA,
        PaintMedia::Watercolor.to_u8().to_string(),
    ));
    assert_eq!(e.preparo(Some(&t), true, ALVO), None);
    mostra(&mut t, &track, e.criterios[1].regiao.k).expect("mostra");
    assenta(&mut t);
    assert_eq!(e.por_cumprir(&t), None, "a 2.ª pintada em Watercolor");
}

/// ⭐ Uma pincelada que as anteriores cobrem inteira não tem parte nova: não tem critério (a lei da
/// W6a: etapa com Δ vazio não espera nem mostra balão). O Splash + uma 5.ª igual à 1.ª = 4 critérios.
#[test]
fn uma_pincelada_toda_coberta_nao_tem_criterio() {
    let ps = [SPLASH[0], SPLASH[1], SPLASH[2], SPLASH[3], SPLASH[0]];
    let (b, _) = grava_pinceladas(LADO, 0, &ps, &[0; 5]).expect("grava");
    let track = ph2d_macro::codec::read(&b).expect("lê").tracks.remove(0);
    assert_eq!(regioes(&track, LADO, LADO).expect("regiões").len(), 5);
    let e = EnsinoDoPainter::de(&track).expect("ensino");
    assert_eq!(e.criterios.len(), 4);
    assert!(e.criterios.iter().all(|c| !c.novos.is_empty()));
    let ultima = regioes(&track, LADO, LADO).expect("regiões")[4].k;
    assert!(e.criterios.iter().all(|c| c.regiao.k != ultima));
}

/// O «Recomeçar» com uma macro a gravar grava uma recusa (a reprodução pára aí em voz alta).
#[test]
fn recomecar_a_gravar_grava_uma_recusa() {
    let track = trilha();
    let mut t = inicial::tool_no_inicial(&track.initial).expect("inicial");
    #[expect(clippy::cast_precision_loss, reason = "o lado cabe num f32")]
    let l = LADO as f32;
    let mut rec = ph2d_macro::MacroRecorder::new(ph2d_macro::RefRect { w: l, h: l }, 0);
    t.macro_abre(&mut rec, 0).expect("arma");
    drop(t.macro_passos());
    recomeca(&mut t, &track).expect("recomeça");
    let passos: Vec<Passo> = t
        .macro_passos()
        .iter()
        .map(|c| Passo::descodifica(c).expect("passo"))
        .collect();
    assert!(
        passos.contains(&Passo::Recusada("tutorial_recomeca".into())),
        "{passos:?}"
    );
}

/// «Recomeçar» volta a tela ao início num passo de undo, e o desfazer volta ao feito.
#[test]
fn recomecar_volta_ao_inicio_num_passo_de_undo() {
    let track = trilha();
    let e = EnsinoDoPainter::medido(&track).expect("ensino");
    let mut t = inicial::tool_no_inicial(&track.initial).expect("inicial");
    mostra(&mut t, &track, e.criterios[0].regiao.k).expect("mostra");
    assenta(&mut t);
    assert_eq!(e.por_cumprir(&t), Some(1));
    recomeca(&mut t, &track).expect("recomeça");
    assert_eq!(e.por_cumprir(&t), Some(0), "de volta ao início");
    assert!(t.undo_last(), "o recomeçar é um passo");
    assert_eq!(e.por_cumprir(&t), Some(1), "o desfazer volta ao feito");
}

/// «Mostra-me» pinta com o pincel da gravação e devolve o do aluno.
#[test]
fn mostra_me_devolve_a_cor_e_o_tamanho_do_aluno() {
    let track = trilha();
    let mut t = inicial::tool_no_inicial(&track.initial).expect("inicial");
    t.set_brush_color_srgb8([1, 2, 3]);
    t.set_brush_size_px(9.0);
    let k = EnsinoDoPainter::de(&track).expect("ensino").criterios[2]
        .regiao
        .k;
    mostra(&mut t, &track, k).expect("mostra");
    assert_eq!((t.brush_color_srgb8(), t.brush_size_px()), ([1, 2, 3], 9.0));
    assert!(
        mostra(&mut t, &track, 999).is_err(),
        "uma etapa que não é pincelada recusa-se"
    );
}
