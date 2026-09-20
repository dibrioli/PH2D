//! **Os cartões da aquarela respondem ao ponteiro — e cada linha ocupa o LUGAR dela.**
//!
//! ⛔⛔ Estes gates nascem de um report do dono (2026-09-20, com foto: *«label embolada e valor não
//! sai de 0»*) sobre a linha **Self Pickup**, acrescentada no dia anterior. Eram **três** defeitos
//! numa edição de quatro linhas, e o que interessa é que **nenhum instrumento desta casa os podia
//! ver**:
//!
//! 1. o cartão BRUSH continuou dimensionado para `3` linhas com `4` dentro;
//! 2. a linha do `Pull` não avançava o `y` (`let _ = card_row(…)` era a ÚLTIMA quando foi escrita)
//!    ⇒ as duas linhas foram pintadas **uma por cima da outra** — a «label embolada» da foto;
//! 3. o id novo nunca entrou em [`ph2d_tool_painter::ids::PAINTER_WATERCOLOR_FIELDS`] ⇒ o
//!    `is_param_field` do `event.rs` responde `false`, o `ValueChanged` **morre dentro do painel**,
//!    e o espelho do quadro seguinte repõe o número autorado: *o valor não sai de 0*.
//!
//! ⚠️⚠️ **O gate que devia ter apanhado o (3) varre a LISTA que esqueceu o membro** — o
//! `seam::watercolor_sliders_forward_setvalue` itera o próprio `PAINTER_WATERCOLOR_FIELDS`, logo
//! fica verde por construção sobre um campo que não está lá. ⇒ o censo daqui é **DERIVADO DA TELA**:
//! ele pinta o painel em dois meios e pergunta pelo que a aquarela ACRESCENTA, que é uma população
//! que ninguém tem de se lembrar de estender.

use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::EventOutcome;
use ph2d_editor_core::tool::Tool;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_painter_layers::PainterLayersPanel;
use ph2d_panel_painter_layers::state::{PainterLayersPanelState, set_current_brush};
use ph2d_tool_painter::{PaintMedia, PainterTool};
use ph2d_ui_testkit::MockPanelHost;

fn viewport() -> Rect {
    Rect::new(0.0, 0.0, 1600.0, 900.0)
}

fn tool_em(media: PaintMedia) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_paint_media(media);
    t
}

fn painted(tool: &PainterTool) -> (MockPanelHost, PainterLayersPanelState, Vec<(NodeId, Rect)>) {
    set_current_brush(Some(tool.brush_settings()));
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let rects = host.paint::<PainterLayersPanel>(&mut st, viewport());
    (host, st, rects)
}

fn rect_of(rects: &[(NodeId, Rect)], id: NodeId) -> Option<Rect> {
    rects
        .iter()
        .find(|(w, r)| *w == id && r.w > 0.0 && r.h > 0.0)
        .map(|(_, r)| *r)
}

/// ⭐⭐⭐ **TODO CAMPO QUE A AQUARELA ACRESCENTA À TELA CHEGA À FERRAMENTA.**
///
/// A população é a DIFERENÇA entre o que o painel pinta com o meio aquarela escolhido e o que ele
/// pinta com o meio digital — *o que esta secção põe no ecrã*. Um campo novo entra nela no dia em
/// que é pintado, sem ninguém o acrescentar a uma lista.
///
/// **Mutações que sangram:** tirar um id de `PAINTER_WATERCOLOR_FIELDS` (o campo pinta, arrasta e
/// morre dentro do painel) · tirar o braço `is_param_field` do `event.rs` (nenhum chega ao tool).
#[test]
fn cada_campo_que_a_aquarela_acrescenta_a_tela_chega_a_ferramenta() {
    let (_, _, secos) = painted(&tool_em(PaintMedia::Digital));
    let base: Vec<NodeId> = secos.iter().map(|(id, _)| *id).collect();

    let molhado = tool_em(PaintMedia::Watercolor);
    let (mut host, mut st, rects) = painted(&molhado);

    let mut medidos = 0usize;
    let mut mudos: Vec<NodeId> = Vec::new();
    for (id, r) in &rects {
        if r.w <= 0.0 || r.h <= 0.0 || base.contains(id) {
            continue;
        }
        // Só os CAMPOS numéricos: um chip é o que o `mirror_value` registou como `NumberInput`.
        if host.store().number_value(*id).is_none() {
            continue;
        }
        medidos += 1;
        let outcome =
            host.apply_panel_event::<PainterLayersPanel>(&mut st, WidgetEvent::ValueChanged(*id));
        let chegou = host.drained_actions().iter().any(|a| {
            matches!(
                a,
                EditorAction::ToolPanelEvent(ph2d_editor_core::tool::PanelEvent::SetValue(i, _))
                    if i == id
            )
        });
        if outcome != EventOutcome::Consumed || !chegou {
            mudos.push(*id);
        }
    }
    assert!(
        mudos.is_empty(),
        "{} campo(s) da aquarela são pintados e NÃO chegam à ferramenta — o id falta em \
         PAINTER_WATERCOLOR_FIELDS (o `is_param_field` do event.rs responde false e o arrasto morre \
         dentro do painel): {mudos:?}",
        mudos.len()
    );
    // Piso de população: se a diferença entre os dois meios encolher, este censo passou a medir o nada.
    assert!(
        medidos >= 12,
        "só {medidos} campos medidos — a varredura partiu-se ou a secção encolheu"
    );
}

/// ⭐⭐ **NENHUMA LINHA É PINTADA POR CIMA DE OUTRA.**
///
/// ⚠️ A régua é o retângulo de HIT, que é o que o dedo encontra: dois widgets no MESMO retângulo são
/// dois rótulos sobrepostos na tela (a foto do dono) **e** um clique que vai para o último
/// registado, porque o `HitIndex::hit` resolve de trás para a frente. *Um cartão dimensionado para
/// `n` linhas com `n+1` dentro não reprova em lado nenhum; duas linhas no mesmo `y`, sim.*
#[test]
fn nenhuma_linha_dos_cartoes_e_pintada_por_cima_de_outra() {
    for media in [PaintMedia::Watercolor, PaintMedia::Digital] {
        let (_, _, rects) = painted(&tool_em(media));
        let vivos: Vec<(NodeId, Rect)> = rects
            .iter()
            .filter(|(_, r)| r.w > 0.0 && r.h > 0.0)
            .copied()
            .collect();
        let mut colisoes: Vec<(NodeId, NodeId)> = Vec::new();
        for (i, (a, ra)) in vivos.iter().enumerate() {
            for (b, rb) in &vivos[i + 1..] {
                if a != b
                    && (ra.x - rb.x).abs() < 0.5
                    && (ra.y - rb.y).abs() < 0.5
                    && (ra.w - rb.w).abs() < 0.5
                    && (ra.h - rb.h).abs() < 0.5
                {
                    colisoes.push((*a, *b));
                }
            }
        }
        assert!(
            colisoes.is_empty(),
            "{media:?}: {} par(es) de widgets partilham o MESMO retângulo — duas linhas pintadas no \
             mesmo `y` (uma `card_row` cujo `y` de retorno foi deitado fora): {colisoes:?}",
            colisoes.len()
        );
        assert!(
            vivos.len() >= 30,
            "{media:?}: só {} widgets pintados — a varredura partiu-se",
            vivos.len()
        );
    }
}

/// ⭐⭐⭐ **O `Self Pickup` É UMA LINHA PRÓPRIA, E A EDIÇÃO DELE CHEGA AO PINCEL** — com o `Pull`,
/// que ele estava a tapar, como CONTROLO do outro lado.
///
/// ⚠️ **As duas metades são precisas.** Sem a primeira, um `Self Pickup` registado por cima do
/// `Pull` ganharia o clique (o hit resolve de trás para a frente) e o gate passaria a medir o
/// vizinho; sem a segunda, mover os dois ao mesmo tempo leria-se como sucesso.
#[test]
fn o_self_pickup_e_uma_linha_propria_e_a_edicao_chega_ao_pincel() {
    let mut tool = tool_em(PaintMedia::Watercolor);
    let antes = tool.brush_settings();
    let (mut host, mut st, rects) = painted(&tool);

    let pickup = rect_of(
        &rects,
        ph2d_tool_painter::ids::PAINTER_WATERCOLOR_SELF_PICKUP,
    )
    .expect("o campo Self Pickup não é pintado");
    let pull = rect_of(&rects, ph2d_tool_painter::ids::PAINTER_WATERCOLOR_PULL)
        .expect("o campo Pull não é pintado");
    assert!(
        (pickup.y - pull.y).abs() > pickup.h * 0.5,
        "o Self Pickup é pintado em cima do Pull (y {} contra {}) — as duas labels sobrepõem-se e o \
         dedo só alcança uma delas",
        pickup.y,
        pull.y
    );

    // ⚠️ O gesto é a EDIÇÃO do número e não um arrasto: medido nesta mesma família (`Dilution` e
    //    `Pull`, que shipam há meses), um `drag_at` sobre um chip deste painel produz cinco eventos
    //    e autora ZERO — o chip é um `NumberInput`, não um `Slider`, e é `type_into_number` que o
    //    testkit conduz. *Escolher o gesto errado aqui daria um gate vermelho sobre produto certo
    //    nas três linhas, o que se lê como defeito de lei.*
    for ev in host.type_into_number(
        ph2d_tool_painter::ids::PAINTER_WATERCOLOR_SELF_PICKUP,
        "0.62",
    ) {
        host.apply_panel_event::<PainterLayersPanel>(&mut st, ev);
    }
    for action in host.drained_actions() {
        if let EditorAction::ToolPanelEvent(pe) = action {
            tool.handle_panel_event(pe);
        }
    }
    let depois = tool.brush_settings();
    assert!(
        (depois.wet_self_pickup - 0.62).abs() < 1e-3,
        "escrever 0.62 no Self Pickup não chegou ao pincel (ficou em {})",
        depois.wet_self_pickup
    );
    assert!(
        (depois.wet_pull - antes.wet_pull).abs() < 1e-6,
        "escrever no Self Pickup mexeu no Pull — a edição caiu na linha errada"
    );
}

// ─────────────────────────────────────────────────────────────────────────────────────────────
// O terceiro defeito do report: o cartão dimensionado para MENOS linhas do que tem.
// ─────────────────────────────────────────────────────────────────────────────────────────────

/// Os dois ficheiros que DECLARAM cartões com o [`crate::card::card_frame`] partilhado. ⚠️ O
/// `paint_mask.rs` tem um `card_frame` PRIVADO cujo último argumento é uma ALTURA e não uma
/// contagem, e o `paint_wetpaint.rs`/`paint_impasto_rig.rs` usam `card_row` **fora** de um cartão —
/// *uma `card_row` é uma linha `rótulo · número`, e nem toda linha dessas vive num cartão*.
const FICHEIROS_COM_CARTAO: [(&str, &str); 2] = [
    (
        "paint_watercolor.rs",
        include_str!("../../src/paint_watercolor.rs"),
    ),
    (
        "paint_impasto.rs",
        include_str!("../../src/paint_impasto.rs"),
    ),
];

/// O último argumento de uma chamada `card_frame(`, lido por contagem de parênteses (o argumento é
/// `n_rows`). `None` quando ele não é um literal inteiro.
fn n_rows_declarado(corpo: &str, inicio: usize) -> Option<usize> {
    // ⚠️ O último argumento NÃO é o texto depois da última vírgula: o Rust admite vírgula final, e
    //    com ela esse texto é espaço em branco. A leitura honesta é a última FATIA não vazia.
    let bytes = corpo.as_bytes();
    let mut nivel = 0i32;
    let mut corte = inicio;
    let mut fatias: Vec<&str> = Vec::new();
    for i in inicio..bytes.len() {
        match bytes[i] {
            b'(' => {
                nivel += 1;
                if nivel == 1 {
                    corte = i;
                }
            }
            b')' => {
                nivel -= 1;
                if nivel == 0 {
                    fatias.push(&corpo[corte + 1..i]);
                    break;
                }
            }
            b',' if nivel == 1 => {
                fatias.push(&corpo[corte + 1..i]);
                corte = i;
            }
            _ => {}
        }
    }
    // Tira comentários de linha e espaços — o argumento pode vir precedido de prosa.
    let limpo = |s: &str| -> String {
        s.lines()
            .map(|l| l.split("//").next().unwrap_or("").trim())
            .collect::<Vec<_>>()
            .join("")
    };
    fatias
        .iter()
        .rev()
        .map(|s| limpo(s))
        .find(|s| !s.is_empty())
        .and_then(|s| s.parse().ok())
}

/// ⭐⭐⭐ **O NÚMERO DE LINHAS QUE UM CARTÃO DECLARA É O NÚMERO DE LINHAS QUE ELE PINTA.**
///
/// ⛔ O [`crate::card::card_frame`] desenha a moldura **antes** das linhas, logo ele tem de as
/// CONTAR de antemão — e uma linha acrescentada sem mexer nesse número é pintada **FORA** do
/// cartão. Foi o terceiro defeito do report de 2026-09-20 (o cartão BRUSH ficou com `3` e passou a
/// ter `4`), e ele **não move um único retângulo de hit**: nenhum gate de costura o pode ver.
///
/// ⚠️ **A população é DERIVADA, nunca uma lista:** um cartão que DELEGA as linhas a um irmão
/// (`paint_sculpt_rows`, `paint_light_rows`) não pode ser contado aqui, e a delegação é detectada
/// pela CHAMADA e não por um nome escrito à mão — *uma lista de excepções é onde um cartão novo se
/// esconde*. Os dois pisos existem para o classificador não colapsar num lado só.
#[test]
fn o_numero_de_linhas_que_um_cartao_declara_e_o_que_ele_pinta() {
    let mut medidos = 0usize;
    let mut delegados = 0usize;
    let mut erros: Vec<String> = Vec::new();

    for (nome, fonte) in FICHEIROS_COM_CARTAO {
        // Um "corpo" é o texto de uma função: do `fn ` até ao `fn ` seguinte na coluna zero.
        for corpo in fonte
            .split("\nfn ")
            .flat_map(|c| c.split("\npub(crate) fn "))
        {
            let Some(pos) = corpo.find("card_frame(") else {
                continue;
            };
            let titulo = corpo.lines().next().unwrap_or("?").trim();
            // Delegação: uma chamada a um irmão que pinta linhas por nós.
            let delega = corpo
                .match_indices("_rows(")
                .any(|(i, _)| !corpo[..i].ends_with("card"));
            if delega {
                delegados += 1;
                continue;
            }
            let Some(declarado) = n_rows_declarado(corpo, pos) else {
                erros.push(format!("{nome} :: {titulo}: o `n_rows` não é um literal"));
                continue;
            };
            // ⚠️ `card_row` NÃO é a única espécie de linha de um cartão: há botões
            //    (`wetness_button_row`), dropdowns e caixas, cada um a ocupar UMA fileira. A
            //    população é toda chamada `…_row(` — *contar só a `card_row` acusaria três cartões
            //    correctos de declararem a mais.* O plural (`…_rows(`) é a delegação, tratada acima.
            let pintadas = corpo.matches("_row(").count();
            medidos += 1;
            if declarado != pintadas {
                erros.push(format!(
                    "{nome} :: {titulo}: o cartão declara {declarado} linha(s) e pinta {pintadas} \
                     — a(s) que sobra(m) é(são) desenhada(s) FORA da moldura"
                ));
            }
        }
    }

    assert!(erros.is_empty(), "{}", erros.join("\n"));
    assert!(
        medidos >= 6,
        "só {medidos} cartões contados — a varredura partiu-se (delegados: {delegados})"
    );
    assert!(
        delegados >= 1,
        "nenhum cartão foi classificado como delegado — o classificador colapsou, e um cartão que \
         delega passaria a ser acusado (ou, pior, todos passariam a ser saltados)"
    );
}
