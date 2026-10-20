//! A costura do painel MACROS: cada controlo carregado no MEIO, como o rato, chega ao barramento
//! como exactamente o pedido dele.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::macro_edits::{
    CampoDaEtapa, EtapaView, FamiliaDaMacro, MacroObjetoView, MacroPanelInfo, MacroRequest,
    PropsDaEtapa, TipoDaEtapaView,
};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_macros::{MacrosPanel, MacrosPanelState, ids, set_current_macros};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 320.0,
    h: 1600.0,
};
const BITS: u64 = 42;

fn etapa(k: usize, props: PropsDaEtapa) -> EtapaView {
    #[expect(clippy::cast_precision_loss, reason = "poucas etapas")]
    let t = k as f64;
    EtapaView {
        trilha: 0,
        k,
        familia: FamiliaDaMacro::Painter,
        tipo: TipoDaEtapaView::Pincelada(k + 1),
        nome: None,
        dica: None,
        t0_s: t,
        t1_s: t + 0.5,
        espera_s: 0.0,
        props,
    }
}

fn info(pode: Option<FamiliaDaMacro>, gravando: Option<FamiliaDaMacro>) -> MacroPanelInfo {
    MacroPanelInfo {
        gravando,
        pode_gravar: pode,
        recusa: None,
        objeto: Some(MacroObjetoView {
            entity_bits: BITS,
            duracao_s: 3.0,
            etapas: (0..3)
                .map(|k| {
                    etapa(
                        k,
                        PropsDaEtapa::Pincelada {
                            cor: [10, 20, 30],
                            tamanho_px: 12.0,
                        },
                    )
                })
                .collect(),
            editada_desde: None,
            erro: None,
        }),
    }
}

struct Palco {
    host: MockPanelHost,
    st: MacrosPanelState,
    rects: Vec<(ph2d_a11y::NodeId, Rect)>,
}

impl Palco {
    fn novo(i: MacroPanelInfo) -> Self {
        let mut p = Self {
            host: MockPanelHost::with_panel::<MacrosPanel>(),
            st: MacrosPanelState::default(),
            rects: Vec::new(),
        };
        set_current_macros(i);
        p.pinta();
        p
    }

    fn pinta(&mut self) {
        self.rects = self.host.paint::<MacrosPanel>(&mut self.st, VIEWPORT);
    }

    fn rect(&self, id: ph2d_a11y::NodeId) -> Option<Rect> {
        self.rects.iter().find(|(n, _)| *n == id).map(|(_, r)| *r)
    }

    fn carrega(&mut self, id: ph2d_a11y::NodeId) -> Vec<EditorAction> {
        let r = self
            .rect(id)
            .unwrap_or_else(|| panic!("o painel nunca pintou {id:?}"));
        let eventos = self.host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5);
        assert!(
            eventos
                .iter()
                .any(|e| matches!(e, WidgetEvent::Click(c) if *c == id)),
            "carregar no meio de {id:?} deu {eventos:?}: MORTO sob o dedo"
        );
        for ev in eventos {
            let _ = self.host.apply_panel_event::<MacrosPanel>(&mut self.st, ev);
        }
        self.host.drained_actions()
    }
}

fn so(a: &[EditorAction], r: MacroRequest) {
    assert_eq!(a, [EditorAction::Macro(r)]);
}

#[test]
fn gravar_e_parar_chegam_ao_barramento_e_so_um_deles_se_pinta() {
    let mut p = Palco::novo(info(Some(FamiliaDaMacro::Painter), None));
    assert!(p.rect(ids::MACROS_STOP).is_none());
    so(&p.carrega(ids::MACROS_RECORD), MacroRequest::Grava);

    let mut p = Palco::novo(info(None, Some(FamiliaDaMacro::Painter)));
    assert!(p.rect(ids::MACROS_RECORD).is_none());
    so(&p.carrega(ids::MACROS_STOP), MacroRequest::Para);

    // Sem quem grave, o Gravar não se pinta (a frase diz porquê).
    let p = Palco::novo(info(None, None));
    assert!(p.rect(ids::MACROS_RECORD).is_none() && p.rect(ids::MACROS_STOP).is_none());
}

#[test]
fn clicar_numa_etapa_escolhe_a_e_leva_a_cabeca_ao_fim_dela() {
    let mut p = Palco::novo(info(None, None));
    let a = p.carrega(ids::row_id(0, 2));
    so(
        &a,
        MacroRequest::VaiPara {
            entity_bits: BITS,
            t_s: 2.5,
        },
    );
    assert_eq!(p.st.escolhida.map(|e| e.k), Some(2));
    // As propriedades da escolhida pintam-se: renomear, a espera, a cor, o tamanho.
    p.pinta();
    for id in [
        ids::MACROS_RENAME,
        ids::MACROS_DELETE,
        ids::MACROS_HINT,
        ids::MACROS_WAIT,
        ids::MACROS_COLOR,
        ids::MACROS_SIZE,
    ] {
        assert!(p.rect(id).is_some(), "{id:?} não se pintou");
    }
    assert!(
        p.rect(ids::MACROS_VALUE).is_none(),
        "uma pincelada não tem valor"
    );
}

/// O ColorDrop gravado (W11) chama-se «Fill» e mostra a cor, sem o tamanho: o pincel não lhe chega.
#[test]
fn um_fill_mostra_a_cor_e_nao_o_tamanho() {
    let mut i = info(None, None);
    let o = i.objeto.as_mut().expect("o objecto");
    o.etapas[1].tipo = TipoDaEtapaView::Preenchimento(1);
    assert_eq!(ph2d_panel_macros::nomes::nome(&o.etapas[1]), "Fill 1");
    let mut p = Palco::novo(i);
    let _ = p.carrega(ids::row_id(0, 1));
    p.pinta();
    assert!(
        p.rect(ids::MACROS_COLOR).is_some(),
        "a cor do Fill não se pintou"
    );
    assert!(
        p.rect(ids::MACROS_SIZE).is_none(),
        "um Fill não tem tamanho"
    );
}

/// ⭐ «Balloon text…» abre o campo da linha com o texto do balão, e o Submit chega como a edição do
/// texto — não do nome (o mesmo campo serve os dois).
#[test]
fn o_texto_do_balao_submete_a_dica_e_nao_o_nome() {
    let mut p = Palco::novo(info(None, None));
    let _ = p.carrega(ids::row_id(0, 1));
    p.pinta();
    let _ = p.carrega(ids::MACROS_HINT);
    assert!(p.st.renomeando && p.st.dica);
    p.host
        .set_text(ids::MACROS_RENAME_INPUT, "Ligue a caixa ao vento");
    let _ = p
        .host
        .apply_panel_event::<MacrosPanel>(&mut p.st, WidgetEvent::Submit(ids::MACROS_RENAME_INPUT));
    so(
        &p.host.drained_actions(),
        MacroRequest::Edita {
            entity_bits: BITS,
            trilha: 0,
            k: 1,
            campo: CampoDaEtapa::Dica(Some("Ligue a caixa ao vento".into())),
        },
    );
    assert!(!p.st.dica, "o campo fechou");
    // Controlo: o Renomear seguinte volta a escrever o NOME.
    p.pinta();
    let _ = p.carrega(ids::MACROS_RENAME);
    assert!(p.st.renomeando && !p.st.dica);
}

#[test]
fn renomear_submete_o_nome_e_a_espera_editada_chega_como_edicao() {
    let mut p = Palco::novo(info(None, None));
    let _ = p.carrega(ids::row_id(0, 1));
    p.pinta();
    let _ = p.carrega(ids::MACROS_RENAME);
    assert!(p.st.renomeando);
    p.host.set_text(ids::MACROS_RENAME_INPUT, "Céu");
    let _ = p
        .host
        .apply_panel_event::<MacrosPanel>(&mut p.st, WidgetEvent::Submit(ids::MACROS_RENAME_INPUT));
    so(
        &p.host.drained_actions(),
        MacroRequest::Edita {
            entity_bits: BITS,
            trilha: 0,
            k: 1,
            campo: CampoDaEtapa::Nome(Some("Céu".into())),
        },
    );
    p.pinta();
    p.host.store_mut().set_number_value(ids::MACROS_WAIT, 0.75);
    let _ = p
        .host
        .apply_panel_event::<MacrosPanel>(&mut p.st, WidgetEvent::ValueChanged(ids::MACROS_WAIT));
    so(
        &p.host.drained_actions(),
        MacroRequest::Edita {
            entity_bits: BITS,
            trilha: 0,
            k: 1,
            campo: CampoDaEtapa::Espera(0.75),
        },
    );
}

#[test]
fn apagar_a_etapa_escolhida_chega_como_pedido_e_larga_a_escolha() {
    let mut p = Palco::novo(info(None, None));
    let _ = p.carrega(ids::row_id(0, 2));
    p.pinta();
    so(
        &p.carrega(ids::MACROS_DELETE),
        MacroRequest::Apaga {
            entity_bits: BITS,
            trilha: 0,
            k: 2,
        },
    );
    assert!(
        p.st.escolhida.is_none(),
        "a etapa apagada deixa de estar escolhida"
    );
}

#[test]
fn o_x_fecha_o_painel() {
    let mut p = Palco::novo(info(None, None));
    let _ = p.carrega(ids::MACROS_CLOSE);
    assert!(!p.host.panel_visible("macros"));
}
