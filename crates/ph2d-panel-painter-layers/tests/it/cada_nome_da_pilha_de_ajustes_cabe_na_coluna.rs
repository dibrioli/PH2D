//! ⭐⭐⭐ **O NOME DE CADA LINHA DA PILHA DE AJUSTES CABE NA COLUNA — medido no painel PINTADO.**
//!
//! ⛔⛔ Até 2026-09-16 a coluna do nome das pilhas de ajuste era o literal `ADJ_LABEL_W = 44,0`
//! (*«slider-param label column ("Contrast")»*), pintado em `Base`. Medido nesse dia com o sistema
//! de texto do produto: **17 dos 44** nomes que as pilhas genéricas pintam não cabiam — o próprio
//! `Contrast` do comentário incluído (`53,3 px`) — e em TODA largura de painel, porque um literal
//! não cresce com o dock. Os nomes da tabela já vêm abreviados (`Shad Amt`, `High Wid`) para caber
//! ali, e mesmo assim não cabiam.
//!
//! ⚠️ **A régua lê o PRODUTO.** Pinta o painel de camadas a sério, com uma camada de ajuste de cada
//! espécie, em cada largura do curso do dock; lê onde cada barra e cada interruptor foram
//! registados; e compara a largura do nome (no peso em que a porta o pinta) com o espaço que sobra
//! entre o início da pilha e o controlo. ⛔ Nenhuma aritmética de coluna é repetida aqui — só a do
//! recuo da pilha, que é uma porta de tokens.

use ph2d_editor_core::screens::layout::{CenterSplit, ChromeBands, DockSides, HeroLayout};
use ph2d_editor_core::widget::NUMBER_INPUT_MIN_W_PX;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_painter_layers::PainterLayersPanel;
use ph2d_panel_painter_layers::state::{
    PainterLayersPanelState, set_current_dock_shows_layers, set_current_layers,
};
use ph2d_text::TextSystem;
use ph2d_tokens::{Spacing, TypeToken};
use ph2d_tool_painter::ids::{PainterLayerWidget, painter_layer_widget_id};
use ph2d_tool_painter::{AdjustmentKind, AdjustmentParams, LayerStack};
use ph2d_ui_testkit::MockPanelHost;

/// ⏳ **QUANTOS NOMES ELIDEM, POR LARGURA DO DOCK — e só ENCOLHE.**
///
/// O curso do dock: o mínimo, dois degraus estreitos (um deles a largura datada do dono), a
/// omissão e o máximo — a mesma escada do gate irmão das secções do painel.
///
/// ⚠️ **Os dois do mínimo são o TECTO, não uma falha:** a pilha partilha UMA coluna, e as barras
/// dela não descem do piso do dono ([`NUMBER_INPUT_MIN_W_PX`]); a `220` isso deixa `78,0` ao nome
/// e `Preserve Lum.` (Color Balance, Photo Filter) mede `82,7`. *Nessa ponta o nome corta, e é a
/// troca que o dono escolheu.*
const ELIDEM_POR_LARGURA: &[(f32, usize)] = &[
    (220.0, 2),
    (245.0, 0),
    // Amostra DATADA da largura do dono (lida em 2026-09-14).
    (273.3, 0),
    (304.0, 0),
    (720.0, 0),
];

/// As larguras da escada.
const LARGURAS: [f32; 5] = [
    ELIDEM_POR_LARGURA[0].0,
    ELIDEM_POR_LARGURA[1].0,
    ELIDEM_POR_LARGURA[2].0,
    ELIDEM_POR_LARGURA[3].0,
    ELIDEM_POR_LARGURA[4].0,
];

const BARRAS: [PainterLayerWidget; 8] = [
    PainterLayerWidget::AdjParam0,
    PainterLayerWidget::AdjParam1,
    PainterLayerWidget::AdjParam2,
    PainterLayerWidget::AdjParam3,
    PainterLayerWidget::AdjParam4,
    PainterLayerWidget::AdjParam5,
    PainterLayerWidget::AdjParam6,
    PainterLayerWidget::AdjParam7,
];

const INTERRUPTORES: [PainterLayerWidget; 2] = [
    PainterLayerWidget::AdjToggle0,
    PainterLayerWidget::AdjToggle1,
];

/// Os nomes que a pilha desta espécie pinta, slot a slot — das MESMAS funções que o pintor lê, com
/// o separador activo no de omissão (o primeiro).
///
/// `None` para a espécie que não tem pilha de linhas (as *Curves* pintam um editor).
fn nomes(params: &AdjustmentParams) -> Option<(Vec<&'static str>, Vec<&'static str>)> {
    let barras = match params {
        AdjustmentParams::Curves(_) => return None,
        AdjustmentParams::ChannelMixer(m) => ph2d_tool_painter::channel_mixer_slider_params(m, 0),
        AdjustmentParams::SelectiveColor(s) => {
            ph2d_tool_painter::selective_color_slider_params(s, 0)
        }
        AdjustmentParams::GradientMap(g) => ph2d_tool_painter::gradient_stop_color_params(g, 0),
        p => ph2d_tool_painter::adjustment_slider_params(p),
    };
    let interruptores = match params {
        AdjustmentParams::SelectiveColor(_) | AdjustmentParams::GradientMap(_) => Vec::new(),
        p => ph2d_tool_painter::adjustment_toggle_params(p),
    };
    Some((
        barras.into_iter().map(|(n, _)| n).collect(),
        interruptores.into_iter().map(|(n, _)| n).collect(),
    ))
}

/// Uma linha pintada: o nome, o rect do controlo, e se o controlo é uma barra.
struct Linha {
    nome: &'static str,
    controlo: Rect,
    barra: bool,
}

/// Uma pilha pintada: onde ela começa e as linhas dela.
struct Pilha {
    especie: AdjustmentKind,
    x: f32,
    linhas: Vec<Linha>,
}

/// Pinta o painel de camadas, com uma camada de ajuste de `especie`, num dock de `dock` px.
fn pintada(especie: AdjustmentKind, dock: f32) -> Option<Pilha> {
    let mut stack = LayerStack::new();
    stack.add_raster("Base", 8, 8).expect("a base nasce");
    let id = stack
        .add_adjustment(especie)
        .expect("a camada de ajuste nasce");
    let params = stack
        .adjustment_mut(id)
        .expect("é uma camada de ajuste")
        .params
        .clone();
    let (barras, interruptores) = nomes(&params)?;
    set_current_dock_shows_layers(true);
    set_current_layers(Some(stack));

    let viewport = Rect::new(0.0, 0.0, 1600.0, 900.0);
    let layout = HeroLayout::for_viewport_bands(
        viewport,
        false,
        ChromeBands {
            right_dock_w: dock,
            ..ChromeBands::DEFAULT
        },
        CenterSplit::None,
        DockSides::BOTH,
    );
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let rects = host.paint_with_layout::<PainterLayersPanel>(&mut st, layout, viewport);
    set_current_layers(None);

    let achar = |w: PainterLayerWidget| {
        let alvo = painter_layer_widget_id(id.0, w);
        rects
            .iter()
            .find(|(n, r)| *n == alvo && r.w > 0.0 && r.h > 0.0)
            .map(|(_, r)| *r)
    };
    // ⚠️ **A pilha começa UM recuo depois da linha da camada**, e a linha da camada começa onde o
    //    olho dela está registado.
    let olho = achar(PainterLayerWidget::Visibility)
        .unwrap_or_else(|| panic!("{especie:?} a {dock}: a linha da camada não foi pintada"));
    let x = olho.x + ph2d_tokens::list_indent_px();

    let mut linhas = Vec::new();
    for (nome, w, barra) in barras.iter().zip(BARRAS).map(|(n, w)| (*n, w, true)).chain(
        interruptores
            .iter()
            .zip(INTERRUPTORES)
            .map(|(n, w)| (*n, w, false)),
    ) {
        let controlo = achar(w).unwrap_or_else(|| {
            panic!("{especie:?} a {dock}: a linha «{nome}» não foi pintada nem registada")
        });
        linhas.push(Linha {
            nome,
            controlo,
            barra,
        });
    }
    Some(Pilha { especie, x, linhas })
}

/// Todas as pilhas, a uma largura.
fn todas(dock: f32) -> Vec<Pilha> {
    AdjustmentKind::ALL
        .iter()
        .filter_map(|k| pintada(*k, dock))
        .filter(|p| !p.linhas.is_empty())
        .collect()
}

/// ⚠️ **O sistema de texto do ARNÊS** — a porta mede com o que o pintor recebe, e o arnês pinta com
/// este; medir com outro compararia duas fontes.
fn texto() -> TextSystem {
    TextSystem::without_system_fonts()
}

#[test]
fn cada_nome_da_pilha_de_ajustes_cabe_na_coluna() {
    let mut ts = texto();
    let fonte = TypeToken::Sm.px();
    let mut medidas = 0usize;
    for (dock, tecto) in ELIDEM_POR_LARGURA {
        let pilhas = todas(*dock);
        // ⚠️ **Piso de população**: 24 espécies, e só as *Curves* e as que não têm parâmetro
        //    nenhum ficam de fora. Uma varredura partida devolveria zero pilhas e passaria.
        assert!(
            pilhas.len() >= 18,
            "a {dock}: só {} pilhas pintadas — a varredura partiu-se",
            pilhas.len()
        );
        let mut cortados = Vec::new();
        for p in &pilhas {
            for l in &p.linhas {
                medidas += 1;
                let coluna = l.controlo.x - p.x - Spacing::Md.px();
                let nome = ts.prefix_width(l.nome, fonte);
                if nome > coluna + 0.5 {
                    cortados.push(format!(
                        "{:?} «{}»: {nome:.1} numa coluna de {coluna:.1}",
                        p.especie, l.nome
                    ));
                }
            }
        }
        assert!(
            cortados.len() <= *tecto,
            "a {dock}: {} nomes da pilha de ajustes não cabem na coluna e o tecto é {tecto}:\n  {}",
            cortados.len(),
            cortados.join("\n  ")
        );
        // ⚠️ **A metade de OBSOLESCÊNCIA** (`CLAUDE.md` §5.0): se melhorou, o número desce AQUI.
        assert!(
            cortados.len() == *tecto,
            "a {dock}: elidem {} e a tabela ainda diz {tecto} — aperte o número",
            cortados.len()
        );
    }
    assert!(
        medidas >= 300,
        "só {medidas} linhas medidas em todo o curso"
    );
}

/// ⭐⭐ **UMA coluna por pilha — as barras e os interruptores começam no MESMO `x`.**
///
/// ⛔ O interruptor era `nome à esquerda … chave encostada à direita`; com as barras a ganhar
/// coluna, as duas famílias passariam a ter o nome em sítios diferentes no mesmo cartão, que é o
/// defeito que o §6-quinquies da spec existe para matar.
#[test]
fn a_pilha_inteira_partilha_uma_coluna() {
    let mut mistas = 0usize;
    for dock in &LARGURAS {
        for p in todas(*dock) {
            let x0 = p.linhas[0].controlo.x;
            for l in &p.linhas {
                assert!(
                    (l.controlo.x - x0).abs() < 0.5,
                    "{:?} a {dock}: «{}» começa em {:.1} e a primeira linha em {x0:.1}",
                    p.especie,
                    l.nome,
                    l.controlo.x
                );
            }
            if p.linhas.iter().any(|l| l.barra) && p.linhas.iter().any(|l| !l.barra) {
                mistas += 1;
            }
        }
    }
    // ⚠️ A lei só se mede numa pilha que tenha as DUAS famílias.
    assert!(
        mistas >= LARGURAS.len() * 2,
        "só {mistas} pilhas com barras E interruptores — a fixtura já não contém o fenómeno"
    );
}

/// ⭐ **E a barra nunca fica abaixo do piso que o dono declarou** (2026-05-24).
#[test]
fn nenhuma_barra_fica_abaixo_do_piso_do_dono() {
    for dock in &LARGURAS {
        for p in todas(*dock) {
            for l in p.linhas.iter().filter(|l| l.barra) {
                assert!(
                    l.controlo.w >= NUMBER_INPUT_MIN_W_PX - 0.5,
                    "{:?} a {dock}: a barra «{}» fica com {:.1} e o piso é {NUMBER_INPUT_MIN_W_PX}",
                    p.especie,
                    l.nome,
                    l.controlo.w
                );
            }
        }
    }
}

/// ⭐⭐⭐ **O CONTROLO: a coluna de `44 px` pintada em `Base` cortava estes nomes.**
///
/// ⛔ Sem esta metade a fixtura podia deixar de conter o fenómeno (nomes curtos que cabem em
/// qualquer coluna) e o gate de cima ficaria verde a medir nada.
#[test]
fn e_a_coluna_antiga_cortava_nomes_desta_pilha() {
    let mut ts = texto();
    let base = TypeToken::Base.px();
    let antiga = 44.0; // o literal que saiu, lido do diff de 2026-09-16
    let mut nomes_vistos = std::collections::BTreeSet::new();
    for p in todas(304.0) {
        for l in &p.linhas {
            if l.barra {
                nomes_vistos.insert(l.nome);
            }
        }
    }
    let cortava = nomes_vistos
        .iter()
        .filter(|n| ts.prefix_width(n, base) > antiga)
        .count();
    assert!(
        cortava >= 10,
        "só {cortava} de {} nomes passariam da coluna antiga — a fixtura perdeu o fenómeno",
        nomes_vistos.len()
    );
}
