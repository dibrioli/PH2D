//! ⭐⭐ **UMA QUEIXA DE CARTÃO É LIDA INTEIRA** (doc 132 §5, a lei do P5, no CARTÃO): a linha de
//! leitura (`readout`) é UMA linha de largura fixa e o pintor corta com reticências — a foto do
//! tutorial 14 lia a frase cortada a meio da palavra «folded» («History before 2.32 s was …»).
//!
//! ⚠️ **Porque nenhum censo a via:** a varredura das elisões (`nenhum_rotulo_do_app_pinta_nada`)
//! pinta o painel do grafo no estado de FÁBRICA — sem cartões —, e estas frases só existem em
//! EXECUÇÃO (montadas pela shell a partir do `MotionState`, com o número do momento). ⇒ aqui cada
//! queixa é montada pelo PRODUTOR real com o PIOR valor plausível e pintada num cartão pelo
//! painel real ([`ph2d_ui_testkit::MockPanelHost`]); quem responde é o censo das elisões do
//! `ph2d-editor-core`, o MESMO medidor que o pintor consulta.
//!
//! A população é o FONTE: toda chave `app.motion.motion_{historia,lens,filter}.*` escrita nos três
//! produtores tem de estar medida aqui — uma queixa nova sem medição é vermelho.

use ph2d_editor_core::text_elide::elisao;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_motion_graph::{MotionGraphPanel, MotionGraphPanelState, set_current_motion_graph};
use std::collections::BTreeSet;

/// O pior segundo plausível da Timeline (2 h 46 min) — sete caracteres com o `{:.2}`.
const PIOR_SEGUNDO: f64 = 9_999.99;
/// O pior `LUT_3D_SIZE` fora de `2..=256` que um ficheiro escreve (cinco dígitos).
const PIOR_TAMANHO: usize = 99_999;
/// A pior linha / contagem: uma LUT de `256³` tem `16 777 216` entradas (oito dígitos).
const PIOR_LINHA: usize = 99_999_999;

/// A janela do tutorial 14 (`1600 × 1400`), com o painel do grafo largo — um cartão só, ao
/// enquadramento automático (zoom ≤ `1,2`, acima do limiar da cápsula: a leitura pinta-se).
const JANELA: Rect = Rect::new(0.0, 0.0, 1600.0, 1400.0);
const PAINEL: Rect = Rect::new(0.0, 0.0, 1200.0, 600.0);

/// Os produtores e as chaves que eles podem pintar na linha de leitura.
const FONTES: [(&str, &str); 3] = [
    ("motion_historia.rs", include_str!("motion_historia.rs")),
    ("motion_lens.rs", include_str!("motion_lens.rs")),
    ("motion_filter.rs", include_str!("motion_filter.rs")),
];

/// Toda queixa de cartão, montada pelo produtor real com o pior valor: `(chave, frase)`.
fn queixas() -> Vec<(&'static str, String)> {
    use crate::motion_lens::{ErroLut, frase};
    use ph2d_lens::ErroCubo;
    let mut v = vec![
        (
            crate::motion_historia::CHAVE_DA_QUEIXA,
            crate::motion_historia::frase_dobrada(PIOR_SEGUNDO),
        ),
        (
            crate::motion_historia::CHAVE_NAO_GRAVADA,
            ph2d_i18n::tr(crate::motion_historia::CHAVE_NAO_GRAVADA).to_owned(),
        ),
        (
            "app.motion.motion_lens.lut_unreadable",
            frase(&ErroLut::Ilegivel),
        ),
        (
            "app.motion.motion_lens.lut_1d",
            frase(&ErroLut::Cubo(ErroCubo::So1d)),
        ),
        (
            "app.motion.motion_lens.lut_no_size",
            frase(&ErroLut::Cubo(ErroCubo::SemTamanho)),
        ),
        (
            "app.motion.motion_lens.lut_bad_size",
            frase(&ErroLut::Cubo(ErroCubo::TamanhoInvalido(PIOR_TAMANHO))),
        ),
        (
            "app.motion.motion_lens.lut_bad_line",
            frase(&ErroLut::Cubo(ErroCubo::LinhaInvalida(PIOR_LINHA))),
        ),
        (
            "app.motion.motion_lens.lut_count",
            frase(&ErroLut::Cubo(ErroCubo::Contagem {
                esperado: PIOR_LINHA,
                lido: PIOR_LINHA,
            })),
        ),
    ];
    // O `MotionFiltro::queixa` devolve `tr(chave)` sem argumentos.
    for chave in [
        "app.motion.motion_filter.off_the_wire",
        "app.motion.motion_filter.after_whole_layer",
        "app.motion.motion_filter.outputs_land_differently",
        "app.motion.motion_filter.each_shape_sees_the_vector_path_as_one",
    ] {
        v.push((chave, ph2d_i18n::tr(chave).to_owned()));
    }
    v
}

/// As chaves escritas nos produtores (o literal entre aspas, e as `const` da história).
fn chaves_no_fonte() -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for (_, fonte) in FONTES {
        for pedaco in fonte.split('"').skip(1).step_by(2) {
            let e_queixa = ["historia", "lens", "filter"]
                .iter()
                .any(|f| pedaco.starts_with(&format!("app.motion.motion_{f}.")));
            if e_queixa {
                out.insert(pedaco.to_owned());
            }
        }
    }
    out
}

/// Pinta UM cartão `Input` com `texto` na linha de leitura e devolve a medição do pintor.
fn pinta(texto: &str) -> elisao::Medido {
    let m = crate::motion_state::MotionState::new();
    let mut g = ph2d_nodegraph::graph::Graph::new();
    let no = g.add_node("value.input");
    let mut snap = ph2d_panel_motion_graph::snapshot_from(&g, &m.registry);
    let cartao = snap
        .nodes
        .iter_mut()
        .find(|n| n.id == no.0)
        .expect("o cartão");
    cartao.readout = Some(texto.to_owned());
    set_current_motion_graph(Some(snap));
    let mut layout = ph2d_editor_core::screens::HeroLayout::for_viewport(JANELA);
    layout.motion_graph = PAINEL;
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<MotionGraphPanel>();
    let mut state = MotionGraphPanelState::default();
    let ((), medidos) = elisao::medindo(|| {
        host.paint_with_layout::<MotionGraphPanel>(&mut state, layout, JANELA);
    });
    set_current_motion_graph(None);
    medidos
        .into_iter()
        .find(|md| md.texto == texto)
        .unwrap_or_else(|| panic!("o pintor não mediu «{texto}» — a leitura não se pintou"))
}

/// A largura do texto no orçamento do pintor, como fracção (> 1 = corta).
fn ocupa(md: &elisao::Medido) -> f32 {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    ts.prefix_width_weighted(&md.texto, md.fonte, md.peso) / md.largura
}

/// ⭐⭐ **Toda queixa de cartão do Motion, com o pior valor, cabe INTEIRA na linha de leitura** —
/// medida pelo pintor do cartão (zero elisão). CONTROLO: a frase antiga da história (a da foto)
/// pintada no MESMO cartão sai cortada — o gate vê um corte quando ele existe.
#[test]
fn toda_queixa_de_cartao_cabe_inteira_na_linha_de_leitura() {
    let qs = queixas();
    let medidas: BTreeSet<String> = qs.iter().map(|(k, _)| (*k).to_owned()).collect();
    assert_eq!(
        medidas,
        chaves_no_fonte(),
        "a população: toda chave de queixa dos produtores tem de estar medida aqui"
    );
    let mut cortadas = Vec::new();
    for (chave, texto) in &qs {
        let md = pinta(texto);
        let fr = ocupa(&md);
        eprintln!(
            "{fr:>5.2} × {:>5.1} px @ {:>4.1} px  {chave}: «{texto}»",
            md.largura, md.fonte
        );
        if !md.coube() {
            cortadas.push(format!("{chave}: «{texto}» → «{}» ({fr:.2})", md.pintado));
        }
    }
    let antiga = pinta("History before 9999.99 s was folded - showing 9999.99 s");
    assert!(
        !antiga.coube(),
        "CONTROLO: a frase da foto tem de sair cortada neste cartão ({:.2})",
        ocupa(&antiga)
    );
    assert!(
        cortadas.is_empty(),
        "queixas cortadas na linha de leitura do cartão:\n  {}",
        cortadas.join("\n  ")
    );
}
