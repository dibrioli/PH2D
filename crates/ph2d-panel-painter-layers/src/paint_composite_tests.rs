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
    assert!(
        n >= 4,
        "piso de população: o catálogo tem ao menos os quatro"
    );
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
    assert!(
        n >= 2,
        "piso de população: o chip cicla ao menos dois escopos"
    );
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
    // ⛔ A pilha nasce VAZIA desde 2026-09-21, e o cartão só pinta as camadas VIVAS: sem o
    // `composite_len` a fixtura pintaria zero fileiras e o gate ficava verde a medir NADA.
    brush.composite_len = 1;
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
        super::paint_composite_card(
            &mut ctx,
            ph2d_tokens::Theme::default(),
            0.0,
            320.0,
            0.0,
            brush,
        );
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

/// ⭐ **A ALTURA DO CARTÃO DA PILHA** — o item que a §22.9 do handoff deixou por medir, e o limite
/// REAL de quantas camadas cabem (o CPU não é).
///
/// A lei é `pad + ROW_H + (vão + fileiras × camadas × pitch) + pad`, e ela é EXACTA porque nenhuma
/// fileira empilha.
#[test]
#[ignore = "sonda: corre à mão"]
fn diag_a_altura_do_cartao() {
    use ph2d_tokens::{ROW_H_PX, Spacing};
    let pad = Spacing::Sm.px();
    let gap = Spacing::Xs.px();
    let pitch = ph2d_tokens::row_pitch_px();
    println!("\n  A ALTURA DO CARTÃO DA PILHA");
    println!("  pad {pad} · ROW_H {ROW_H_PX} · pitch {pitch} · vão {gap}\n");
    println!("  camadas | fileiras | altura px  (a pilha monta-se à mão desde 21/09)");
    println!("  --------+----------+----------");
    // ⚠️ O `+ 1` é a fileira do `+`, pintada sempre que a secção está ligada.
    for n in [0usize, 1, 3, 5, 7] {
        let h = pad + ROW_H_PX + (gap + (3 * n + 1) as f32 * pitch) + pad;
        println!("  {n:7} | {:8} | {h:8.0}", 3 * n + 1);
    }
}

/// Pinta o cartão com uma pilha de `n` camadas (todas `Brush` menos as que a quota não dá) e
/// devolve `(ids com hit rect, rótulos que as caixas de valor receberam)`.
fn pinta_pilha(n: usize) -> (Vec<ph2d_a11y::NodeId>, Vec<String>) {
    super::ROTULOS_PINTADOS.with(|c| c.borrow_mut().clear());
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<crate::PainterLayersPanel>();
    let mut scene = ph2d_vector::VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    let viewport = ph2d_editor_core::zones::Rect::new(0.0, 0.0, 360.0, 4000.0);
    let layout = ph2d_editor_core::screens::HeroLayout::for_viewport(viewport);
    let mut brush = crate::paint_brush::FALLBACK_BRUSH;
    brush.composite_enabled = true;
    brush.composite_len = n;
    // A disponibilidade vem DERIVADA do motor no produto; aqui a fixtura escreve-a à mão para
    // poder pôr o cartão nos dois estados (com e sem quota).
    brush.composite_add_available = [n < ph2d_tool_painter::N_COMPOSITE_LAYERS; 4];
    for pos in 0..n {
        brush.composite_strength[pos] = 1.0;
    }
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
        super::paint_composite_card(
            &mut ctx,
            ph2d_tokens::Theme::default(),
            0.0,
            320.0,
            0.0,
            brush,
        );
    }
    use ph2d_editor_core::panel::PanelHostInternal;
    let ids = host
        .hit_index_mut()
        .iter_registrations()
        .map(|(id, _)| id)
        .collect();
    (
        ids,
        super::ROTULOS_PINTADOS.with(|c| c.borrow().iter().map(|(r, _)| r.clone()).collect()),
    )
}

/// ⭐⭐⭐ **A secção nasce VAZIA e o `+` está lá** (ordem do dono, 2026-09-21).
///
/// ⚠️ As duas metades: *não há fileira de camada nenhuma* (senão a secção não nasce vazia) **e**
/// *o `+` e o menu dele são ALCANÇÁVEIS* (senão a secção nasce vazia e sem forma de a encher —
/// que é pior do que nascer cheia).
#[test]
fn a_seccao_nasce_vazia_e_o_mais_esta_la() {
    let (ids, rotulos) = pinta_pilha(0);
    assert!(
        rotulos.is_empty(),
        "com zero camadas o cartão não pode pintar caixa de valor nenhuma: {rotulos:?}"
    );
    assert!(
        !ids.contains(&ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_REMOVE[0]),
        "com zero camadas não há `x` nenhum para registar"
    );
    for (nome, id) in [
        ("o `+`", ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_ADD),
        (
            "o menu do `+`",
            ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_ADD_KIND,
        ),
    ] {
        assert!(
            ids.contains(&id),
            "{nome} não é alcançável: uma secção que nasce vazia e sem `+` não tem como encher"
        );
    }
}

/// ⭐⭐⭐ **Com a quota toda gasta, o `+` e o menu ficam INERTES** — *«ao usar todas inativa-se o
/// dropdown e o botão +»*.
///
/// ⚠️ O CONTROLO é a metade que torna o gate honesto: com a pilha a meio eles TÊM de estar vivos.
#[test]
fn com_a_quota_gasta_o_mais_fica_inerte() {
    let (meio, _) = pinta_pilha(2);
    assert!(
        meio.contains(&ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_ADD),
        "CONTROLO: com a pilha a meio o `+` tem de estar vivo"
    );
    let (cheio, _) = pinta_pilha(ph2d_tool_painter::N_COMPOSITE_LAYERS);
    for (nome, id) in [
        ("o `+`", ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_ADD),
        (
            "o menu do `+`",
            ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_ADD_KIND,
        ),
    ] {
        assert!(
            !cheio.contains(&id),
            "{nome} continua alcançável com a pilha CHEIA"
        );
    }
}

/// ⭐ **Cada camada viva tem um `x`, e as posições que não existem não têm.**
#[test]
fn cada_camada_viva_tem_um_x() {
    let n = 3usize;
    let (ids, _) = pinta_pilha(n);
    for pos in 0..n {
        assert!(
            ids.contains(&ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_REMOVE[pos]),
            "a camada {pos} não tem `x` alcançável"
        );
    }
    for pos in n..ph2d_tool_painter::N_COMPOSITE_LAYERS {
        assert!(
            !ids.contains(&ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_REMOVE[pos]),
            "a posição {pos} não existe e mesmo assim registou um `x`"
        );
    }
}

/// ⭐⭐⭐ **Todo slider deste cartão tem NOME, o Strength incluído** (ordem do dono, 2026-09-21).
///
/// ⛔⛔ Até hoje o rótulo do Strength era a string VAZIA, e o doc do pintor defendia-o com *«o nome
/// desta linha é o chip da operação»*. Com o chip a virar um NOME de camada essa premissa morreu:
/// *o nome da CAMADA e o nome do que a barra CONTROLA são duas coisas*.
///
/// ⚠️ A régua OBSERVA o que o pintor recebeu (ver `ROTULOS_PINTADOS`) em vez de varrer o fonte —
/// *um censo textual das chamadas mede o código, não o produto*.
#[test]
fn todo_slider_do_cartao_tem_nome() {
    let (_, rotulos) = pinta_pilha(2);
    assert_eq!(
        rotulos.len(),
        6,
        "duas camadas × três caixas (força, tamanho, dureza): {rotulos:?}"
    );
    assert!(
        rotulos.iter().all(|r| !r.trim().is_empty()),
        "uma caixa de valor foi pintada SEM nome: {rotulos:?}"
    );
    assert!(
        rotulos
            .iter()
            .any(|r| r == ph2d_i18n::tr("panel.painter_layers.composite.strength")),
        "o Strength tem de estar entre os nomes: {rotulos:?}"
    );
}

/// A pilha do smoke do dono (`PH2D_COMPOSITE_SMOKE=1`): Blur · Brush × 3 · Smear · Erase.
const PILHA_DO_SMOKE: [u8; 6] = [2, 0, 0, 0, 1, 3];

/// Pinta o cartão com a [`PILHA_DO_SMOKE`] na linha `content_w` e devolve `(nome e rect de cada caixa de valor,
/// o hit rect da caixa de marcar)`.
fn pinta_o_smoke(
    text: &mut TextSystem,
    content_w: f32,
) -> (
    Vec<(String, ph2d_editor_core::zones::Rect)>,
    ph2d_editor_core::zones::Rect,
) {
    super::ROTULOS_PINTADOS.with(|c| c.borrow_mut().clear());
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<crate::PainterLayersPanel>();
    let mut scene = ph2d_vector::VectorScene::new();
    let viewport = ph2d_editor_core::zones::Rect::new(0.0, 0.0, 800.0, 4000.0);
    let layout = ph2d_editor_core::screens::HeroLayout::for_viewport(viewport);
    let mut brush = crate::paint_brush::FALLBACK_BRUSH;
    brush.composite_enabled = true;
    let n = PILHA_DO_SMOKE
        .len()
        .min(ph2d_tool_painter::N_COMPOSITE_LAYERS);
    brush.composite_len = n;
    brush.composite_ops[..n].copy_from_slice(&PILHA_DO_SMOKE[..n]);
    {
        let mut ctx = ph2d_editor_core::panel::PaintCtx {
            host: &mut host,
            layout: &layout,
            slot: layout
                .slot_rects(ph2d_editor_core::screens::slot::SlotSet::ANY_DOCK)
                .get(ph2d_editor_core::screens::slot::Slot::RightTop),
            viewport,
            scene: &mut scene,
            text_system: text,
        };
        super::paint_composite_card(
            &mut ctx,
            ph2d_tokens::Theme::default(),
            0.0,
            content_w,
            0.0,
            brush,
        );
    }
    use ph2d_editor_core::panel::PanelHostInternal;
    let marcar = host
        .hit_index_mut()
        .iter_registrations()
        .find(|(id, _)| *id == ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_ENABLE)
        .map(|(_, r)| r)
        .expect("a caixa de marcar do Composite registou hit rect");
    (super::ROTULOS_PINTADOS.with(|c| c.borrow().clone()), marcar)
}

/// O nome de uma caixa sai inteiro no rect onde o pintor a pôs? — a repartição da caixa única
/// (`label_column`) e a medida que o `fit_label` compara; o rect é o OBSERVADO.
fn cabe(text: &mut TextSystem, rotulo: &str, r: ph2d_editor_core::zones::Rect) -> bool {
    let deco = ph2d_editor_core::widget::FORM_ROWS_SHOW_DECORATOR
        && ph2d_editor_core::paint::ui_is_redesign();
    let col =
        ph2d_editor_core::widget::label_column(r, ph2d_editor_core::widget::default_chip_w(), deco);
    text.layout(rotulo, ph2d_tokens::TypeToken::Sm.px(), f32::INFINITY)
        .width()
        <= col.w
}

/// ⏳ `(painel, Strength cortados, Size/Hardness cortados)` no cartão do smoke — e só ENCOLHE. Até 08/11 a caixa
/// da fileira A dividia a linha com o número, o nome da operação e as três setas, e abaixo de ~`304` o nome dela
/// elidia até ao vazio; agora, não cabendo, ela desce à fileira própria (`forca_na_fileira_a`).
const CORTADOS_POR_LARGURA: &[(f32, usize, usize)] = &[
    // Perto do mínimo do dock a caixa recuada inteira (sem prefixo) é estreita demais para o nome e a coluna do
    // número (o piso de 72 px, ordem do dono de 2026-05-24): tudo corta.
    (245.0, 6, 12),
    // A largura do dono (14/09): a Strength e o prefixo descem às fileiras próprias e tudo sai inteiro.
    (273.3, 0, 0),
    (304.0, 0, 0),
    (720.0, 0, 0),
];

/// ⭐⭐⭐ **No cartão do Composite do painel de antes, o «Size (px)» e o «Hardness» saem INTEIROS em toda camada,
/// e o nome da secção que quebra em duas linhas não invade a 1.ª camada** (smoke 08/11: `Size…` na Erase e um
/// «Bru» à esquerda da Strength da Blur).
///
/// ⛔ O `cada_nome_deste_painel_cabe_na_coluna_da_seccao` não via nenhum dos dois: ele mede os nomes da COLUNA
/// DA SECÇÃO (o cartão só declara `Composite Brush`), nunca os nomes DENTRO de uma caixa de valor; e dá por bom o
/// nome que quebra (`Composite` / `Brush`) sem perguntar se a linha dele CRESCE — o cartão tinha altura fixa.
/// Esta régua OBSERVA o rect de cada caixa que o pintor recebeu, à largura do painel.
///
/// ⚠️ O CONTROLO: à largura do dono o chip do escopo AO LADO do tamanho cortava-o — senão a fileira própria do
/// escopo seria um custo sem causa.
#[test]
fn no_cartao_do_composite_cada_nome_de_caixa_cabe_e_o_titulo_nao_invade() {
    let mut text = TextSystem::new();
    let tamanho = ph2d_i18n::tr("panel.painter_layers.composite.size");
    let dureza = ph2d_i18n::tr("panel.painter_layers.composite.hardness");
    let forca = ph2d_i18n::tr("panel.painter_layers.composite.strength");
    let mut medido = Vec::new();
    for &(painel, tecto, tecto_outros) in CORTADOS_POR_LARGURA {
        let content_w = painel - 2.0 * ph2d_tokens::PANEL_HEAD_PAD_PX;
        let (caixas, marcar) = pinta_o_smoke(&mut text, content_w);
        assert_eq!(
            caixas.len(),
            3 * PILHA_DO_SMOKE.len(),
            "piso de população a {painel}: {caixas:?}"
        );
        let (mut forca_cortada, mut outros) = (0, Vec::new());
        for (rotulo, r) in &caixas {
            let inteiro = cabe(&mut text, rotulo, *r);
            if rotulo == forca {
                forca_cortada += usize::from(!inteiro);
            } else if !inteiro {
                assert!(
                    [tamanho, dureza].contains(&rotulo.as_str()),
                    "nome inesperado «{rotulo}»"
                );
                outros.push(format!("«{rotulo}» {r:?}"));
            }
            assert!(
                r.y >= marcar.y + marcar.h,
                "painel {painel}: a caixa «{rotulo}» ({r:?}) começa dentro do nome da secção ({marcar:?})"
            );
        }
        medido.push((painel, forca_cortada, outros.len()));
        assert!(
            forca_cortada <= tecto && outros.len() <= tecto_outros,
            "painel {painel}: {forca_cortada} Strength (tecto {tecto}) e {} Size/Hardness (tecto {tecto_outros}) \
             cortados — subiu:\n  {}",
            outros.len(),
            outros.join("\n  ")
        );
    }
    let tabela: Vec<(f32, usize, usize)> = CORTADOS_POR_LARGURA.to_vec();
    assert_eq!(
        medido, tabela,
        "melhorou (ou a tabela está larga) — aperte-a para o medido"
    );
    // O CONTROLO à largura do dono.
    let content_w = 273.3 - 2.0 * ph2d_tokens::PANEL_HEAD_PAD_PX; // LITERAL-PX-OK: a largura do dono (14/09)
    let iw = content_w - 2.0 * ph2d_tokens::Spacing::Sm.px();
    assert!(
        !super::prefixo_ao_lado_do_tamanho(&mut text, 0.0, iw, 3),
        "CONTROLO: à largura do dono o chip do escopo ao lado do tamanho tem de o cortar"
    );
    // E o título que quebra cresce a linha dele (a 2.ª linha, `Brush`, não cai na 1.ª camada): varre o curso do
    // dock, com piso de população — alguma largura TEM de o quebrar, senão a metade de cima não mediu nada.
    let mut quebrou = 0;
    for passo in 0..=100u16 {
        let painel = 220.0 + 5.0 * f32::from(passo); // LITERAL-PX-OK: varredura 220..720 do dock
        let (caixas, marcar) =
            pinta_o_smoke(&mut text, painel - 2.0 * ph2d_tokens::PANEL_HEAD_PAD_PX);
        if marcar.h > ph2d_tokens::ROW_H_PX {
            quebrou += 1;
            assert!(
                caixas.iter().all(|(_, r)| r.y >= marcar.y + marcar.h),
                "painel {painel}: o nome quebrado da secção ({marcar:?}) cai dentro da 1.ª camada"
            );
        }
    }
    assert!(
        quebrou > 0,
        "CONTROLO: nenhuma largura do dock quebrou o `Composite Brush`"
    );
}
