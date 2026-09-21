//! ⭐⭐⭐ **A COLUNA DO CHIP DA OPERAÇÃO CABE A TODOS OS NOMES** — o report do dono de 2026-09-20
//! (*«nomes achatados»*, com foto: `Br…`, `S…`, `Er…` e só `Blur` inteiro).
//!
//! ⛔⛔ A largura era `const OP_W = 44,0` e o comentário ao lado dela **afirmava** que media
//! (*«fits "Smear"»*). *Uma largura estimada com a afirmação de que foi medida é pior do que uma
//! sem comentário nenhum — ela convida a não re-medir*, e foi preciso o olho do dono para a
//! desmentir.
//!
//! ⚠️ **A régua é a do PINTOR, nunca uma segunda aritmética:** um rótulo de botão é centrado e
//! elidido contra [`label_budget`] da largura do rect, com a fonte do
//! [`Button::label_font_px`] — a porta que existe precisamente para quem pergunta *«este rótulo
//! cabe?»*. Comparar a coluna com ela própria seria vácuo; o que se compara é a **largura do
//! texto** com o **orçamento** que o pintor de facto gasta.

use super::{largura_do_chip_da_operacao, op_name};
use ph2d_editor_core::paint::label_budget;
use ph2d_editor_core::widget::Button;
use ph2d_text::TextSystem;

/// A largura que o cartão dava à coluna antes desta cura — guardada como **CONTROLO**, nunca como
/// produto: é ela que prova que o gate MORDE (com ela pelo menos um nome elide).
const A_LARGURA_QUE_O_DONO_REPROVOU: f32 = 44.0; // LITERAL-PX-OK: o `OP_W` de antes de 2026-09-20

/// ⭐⭐⭐ **Nenhum dos nomes de operação é elidido na coluna que o cartão calcula.**
///
/// ⚠️ **E o CONTROLO é a metade que torna isto uma afirmação:** com a largura antiga pelo menos um
/// nome TEM de elidir. Sem ele, um `largura_do_chip_da_operacao` que devolvesse `f32::MAX` passaria
/// — e um que devolvesse a largura certa por acaso não se distinguiria de um que a mede.
#[test]
fn nenhum_nome_de_operacao_sai_cortado_na_coluna_do_chip() {
    let mut ts = TextSystem::new();
    let fonte = Button::label_font_px();
    let col = largura_do_chip_da_operacao(&mut ts);
    let orcamento = label_budget(col);

    let mut cortados_hoje: Vec<String> = Vec::new();
    let mut cortados_antes: Vec<String> = Vec::new();
    for op in 0..ph2d_tool_painter::N_COMPOSITE_OPS {
        let nome = op_name(op as u8);
        let w = ts.prefix_width(nome, fonte);
        if w > orcamento {
            cortados_hoje.push(format!("{nome} pede {w:.1} de {orcamento:.1}"));
        }
        if w > label_budget(A_LARGURA_QUE_O_DONO_REPROVOU) {
            cortados_antes.push(nome.to_string());
        }
    }

    assert!(
        cortados_hoje.is_empty(),
        "a coluna do chip mede {col:.1} (orçamento {orcamento:.1}) e estes nomes não cabem: {}",
        cortados_hoje.join(" · ")
    );
    assert!(
        !cortados_antes.is_empty(),
        "CONTROLO: com a largura reprovada ({A_LARGURA_QUE_O_DONO_REPROVOU}) nenhum nome elidia — \
         a régua deixou de conter o fenómeno que o report descreve, e este gate passou a não \
         afirmar nada"
    );
}

/// ⭐⭐ **Há um nome por operação, e eles são DISTINTOS.**
///
/// ⚠️ Ele apanha DOIS defeitos com uma régua só, e eles vêm de lados opostos: uma
/// [`ph2d_tool_painter::N_COMPOSITE_OPS`] maior do que o `match` do [`op_name`] cobre cai no
/// braço `_ => Brush` e devolve um nome REPETIDO; e uma contagem menor deixaria uma operação
/// alcançável pelo ciclo sem nome próprio. *A coluna acima é medida sobre esta lista — uma lista
/// com um buraco mede uma coluna estreita de mais e ninguém repara.*
#[test]
fn o_painel_tem_um_nome_por_operacao() {
    let n = ph2d_tool_painter::N_COMPOSITE_OPS;
    assert!(n >= 4, "piso de população: o catálogo tem ao menos os quatro");
    let nomes: Vec<&str> = (0..n).map(|op| op_name(op as u8)).collect();
    for i in 0..nomes.len() {
        for j in (i + 1)..nomes.len() {
            assert_ne!(
                nomes[i], nomes[j],
                "as operações {i} e {j} mostram o mesmo nome ({}) — ou falta um braço no `op_name`, \
                 ou o `N_COMPOSITE_OPS` conta mais do que o catálogo tem",
                nomes[i]
            );
        }
    }
}

/// ⭐⭐ **Nenhum nome de ESCOPO da borracha sai cortado, e eles são DISTINTOS.**
///
/// A mesma régua da coluna da operação, sobre a população do
/// [`ph2d_tool_painter::N_COMPOSITE_ERASE_SCOPES`] — ⛔ *derivar a largura de um número escolhido à
/// mão é exactamente o que a foto do dono reprovou uma wave antes*.
#[test]
fn nenhum_nome_de_escopo_sai_cortado_nem_se_repete() {
    let mut ts = TextSystem::new();
    let fonte = Button::label_font_px();
    let col = super::largura_do_chip_do_escopo(&mut ts);
    let orcamento = label_budget(col);
    let n = ph2d_tool_painter::N_COMPOSITE_ERASE_SCOPES;
    assert!(n >= 2, "piso de população: o chip cicla ao menos dois escopos");
    let nomes: Vec<&str> = (0..n).map(|e| super::escopo_name(e as u8)).collect();
    for (i, nome) in nomes.iter().enumerate() {
        let w = ts.prefix_width(nome, fonte);
        assert!(
            w <= orcamento,
            "o escopo {i} ({nome}) pede {w:.1} de {orcamento:.1} na coluna medida ({col:.1})"
        );
    }
    for i in 0..nomes.len() {
        for j in (i + 1)..nomes.len() {
            assert_ne!(
                nomes[i], nomes[j],
                "os escopos {i} e {j} mostram o mesmo nome — ou falta um braço no `escopo_name`, \
                 ou o `N_COMPOSITE_ERASE_SCOPES` conta mais do que a lei tem"
            );
        }
    }
}

/// Pinta o CARTÃO com a posição `0` na operação `op` e devolve os ids que registaram hit rect.
///
/// ⚠️ **É o PINTOR que corre, nunca a porta `apaga`:** *uma tabela de leis é um resumo do produto,
/// e um resumo não tem de conter tudo* — e um chip pintado sem hit rect é exactamente o «morto sob
/// o dedo» que esta casa já pagou sete vezes na escultura.
fn ids_pintados_com(op: u8) -> Vec<ph2d_a11y::NodeId> {
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<crate::PainterLayersPanel>();
    let mut scene = ph2d_vector::VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    let viewport = ph2d_editor_core::zones::Rect::new(0.0, 0.0, 360.0, 4000.0);
    let layout = ph2d_editor_core::screens::HeroLayout::for_viewport(viewport);
    let mut brush = crate::paint_brush::FALLBACK_BRUSH;
    brush.composite_enabled = true;
    brush.composite_ops[0] = op;
    brush.composite_strength[0] = 1.0;
    {
        let mut ctx = ph2d_editor_core::panel::PaintCtx {
            host: &mut host,
            layout: &layout,
            slot: layout
                .slot_rects(ph2d_editor_core::screens::slot::SlotSet::ANY_DOCK)
                .get(ph2d_editor_core::screens::slot::Slot::RightTop),
            viewport,
            scene: &mut scene,
            text_system: &mut text,
        };
        super::paint_composite_card(&mut ctx, ph2d_tokens::Theme::default(), 0.0, 320.0, 0.0, brush);
    }
    use ph2d_editor_core::panel::PanelHostInternal;
    host.hit_index_mut()
        .iter_registrations()
        .map(|(id, _)| id)
        .collect()
}

/// ⛔⛔ **O chip do escopo é pintado — e ALCANÇÁVEL — só numa camada de borracha.**
///
/// ⚠️ **As duas metades, porque cada uma sozinha mente:** *está lá quando a operação é `Erase`*
/// (senão a ordem do dono é inalcançável) **e** *não está nas outras* (senão é um controlo morto
/// numa fileira onde a pergunta não tem sujeito — a espécie que o §5.0 do `CLAUDE.md` nomeia).
#[test]
fn o_chip_do_escopo_so_aparece_numa_camada_de_borracha() {
    let alvo = ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_ERASE_SCOPE[0];
    for op in 0..ph2d_tool_painter::N_COMPOSITE_OPS as u8 {
        let pintado = ids_pintados_com(op).contains(&alvo);
        assert_eq!(
            pintado,
            super::apaga(op),
            "a operação {op} ({}) {} o chip do escopo",
            op_name(op),
            if pintado { "PINTA" } else { "não pinta" }
        );
    }
}
