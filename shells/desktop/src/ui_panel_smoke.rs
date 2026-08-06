//! **A cena do PAINEL GERADO** — `PH2D_BUILD_SMOKE=62` (plano UI/UX W8b).
//!
//! # A pergunta desta cena
//!
//! *Eu desenhei um painel, e o app me devolveu o CÓDIGO dele.*
//!
//! Uma moldura chamada **Color** com cinco filhos: quatro vestidos (o cabeçalho, um slider, um
//! toggle, um botão) e **um que é só desenho**. No fim ela imprime, no stderr, o código-fonte que
//! o gerador escreveu — que é o artefato desta wave.
//!
//! ⚠️ **O filho de desenho puro é o CONTROLE da cena**, e ele não é decoração: ele prova que *só
//! quem VESTE vira row*. Sem ele, um gerador que transformasse todo filho em linha passaria
//! despercebido, e o painel gerado teria uma row que não faz nada — o item-de-menu-morto na sua
//! forma mais cara.
//!
//! ⚠️ **E a cena imprime o número que a torna válida:** quantas rows o plano tem. Se não forem
//! quatro, PARE — o resto não diz nada.
//!
//! # A W8b.3 põe a ARTE do outro lado do fio
//!
//! Um retângulo laranja ao lado do painel, **fora** da moldura, com a row *Opacity* já presa a
//! ele. Arrastar o slider desvanece a arte; prender o *Visible* à mão é o gesto que a fatia
//! acrescenta. ⚠️ E o `Reset` é o CONTROLE: um `Button` não dirige, e a linha *Drives* nem
//! aparece nele.
//!
//! # A W8b.2 abre o painel, e a cena diz a verdade sobre ele
//!
//! ⚠️ **O painel na tela mostra a tabela COMMITADA, não a que o log acabou de imprimir.** É o que
//! codegen é: o app escreve o código, alguém o commita, o `cargo` o compila. Esconder isso faria o
//! artista renomear um filho, ver o log mudar, olhar o painel intacto e concluir que ele está
//! quebrado — então o roteiro diz na cara, e o gate de staleness é quem garante que os dois estão
//! sincronizados no `main`.

use ph2d_editor::widget::WidgetKind;
use ph2d_vec_scene::{Paint, Rgba8, VecPath, VecPathId, rectangle};

/// A moldura, e os filhos que ela contém: `(caixa, nome, o que ele veste)`.
///
/// ⚠️ **UMA TABELA, DOIS CONSUMIDORES:** a cena constrói a árvore a partir dela, e o gate de
/// staleness (`the_generated_panel_is_not_stale`) constrói o mundo a partir dela para emitir o
/// código e comparar com o arquivo commitado. Uma segunda lista escrita à mão no gate divergiria
/// desta no dia em que uma row entrasse — e o gate ficaria verde sobre o painel errado.
pub(crate) const AUTHORED: [([f64; 4], &str, Option<WidgetKind>); 6] = [
    ([-2.0, -2.4, 2.0, 2.4], "Color", None),
    (
        [-1.8, 1.4, 1.8, 2.2],
        "Appearance",
        Some(WidgetKind::SectionHeader),
    ),
    ([-1.8, 0.4, 1.8, 1.1], "Opacity", Some(WidgetKind::Slider)),
    ([-1.8, -0.6, 1.8, 0.1], "Visible", Some(WidgetKind::Toggle)),
    ([-1.8, -1.6, 1.8, -0.9], "Reset", Some(WidgetKind::Button)),
    ([-1.7, -2.2, 1.7, -1.9], "Backdrop", None),
];

/// A moldura é a primeira linha; os filhos são o resto.
const FRAME: usize = 0;

/// **A ARTE** — a forma que as rows vão DIRIGIR (W8b.3), e ela não é filha da moldura.
///
/// ⚠️ Fora da tabela `AUTHORED` de propósito: aquela lista descreve *o painel*, e o gate de
/// staleness constrói o mundo a partir dela para emitir o código. Uma estrela ali dentro entraria
/// na conta do painel; aqui ela é o que o painel MEXE — os dois lados do fio, e cada um no seu
/// lugar.
const STAR: usize = AUTHORED.len();

pub(crate) fn frame(app: &mut crate::App, f: u32) {
    match f {
        3 => build(app),
        5 => name_and_parent(app),
        7 => bind_the_slider(app),
        9 => announce(app),
        11 => open_the_panel(app),
        _ => {}
    }
}

fn build(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    for (i, (r, _, kind)) in AUTHORED.iter().enumerate() {
        let mut p: VecPath = rectangle([r[0], r[1]], [r[2], r[3]]);
        // A moldura é escura; quem veste fica visível; o desenho puro fica apagado, para a foto
        // dizer qual é qual.
        let c = match (i == FRAME, kind) {
            (true, _) => [30, 34, 46],
            (_, Some(_)) => [64, 84, 128],
            (_, None) => [44, 48, 58],
        };
        p.fill = Some(Paint::Solid(Rgba8::new(c[0], c[1], c[2], 255)));
        gfx.vec_scene.push_path(p);
    }
    // A arte, ao lado do painel: um quadrado grande e quente, para o desvanecer ser óbvio.
    let mut star: VecPath = rectangle([2.6, -1.0], [5.0, 1.4]);
    star.fill = Some(Paint::Solid(Rgba8::new(232, 150, 60, 255)));
    gfx.vec_scene.push_path(star);
}

fn path_ids(app: &crate::App) -> Vec<VecPathId> {
    app.gfx
        .as_ref()
        .map(|g| g.vec_scene.paths().iter().map(|p| p.id).collect())
        .unwrap_or_default()
}

/// Nomeia, pendura os filhos na moldura e veste quem veste.
///
/// ⚠️ Num frame POSTERIOR ao `build`, e é obrigatório: a entidade de uma forma nasce no
/// `vec_entities::sync`, que corre no frame do desenho.
fn name_and_parent(app: &mut crate::App) {
    let ids = path_ids(app);
    if ids.len() < AUTHORED.len() {
        return;
    }
    let ents: Vec<_> = ids
        .iter()
        .map(|&id| {
            app.vec_entities
                .get(&id)
                .map(|&bits| ph2d_ecs::Entity::from_bits(bits))
        })
        .collect();
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let Some(frame_e) = ents[FRAME] else { return };
    gfx.sim
        .world_mut()
        .entity_mut(frame_e)
        .insert(ph2d_ecs::VecFrame { clip: false });
    for (i, (_, name, kind)) in AUTHORED.iter().enumerate() {
        let Some(e) = ents[i] else { continue };
        let Ok(mut ent) = gfx.sim.world_mut().get_entity_mut(e) else {
            continue;
        };
        ent.insert(ph2d_ecs::Name::new(*name));
        if let Some(k) = kind {
            ent.insert(ph2d_ecs::VecWidget { kind: k.code() });
        }
        if i != FRAME {
            ent.insert(ph2d_ecs::ChildOf(frame_e));
        }
    }
    // A arte ganha nome e NÃO ganha pai — ela vive na cena, não no painel.
    if let Some(e) = ents.get(STAR).copied().flatten()
        && let Ok(mut ent) = gfx.sim.world_mut().get_entity_mut(e)
    {
        ent.insert(ph2d_ecs::Name::new("Star"));
    }
}

/// Prende a row **Opacity** à estrela — o que o artista faria com o conta-gotas.
///
/// ⚠️ Um lado do fio vem pronto e o outro **não**: com os dois prontos o smoke provaria o
/// resolvedor e não o GESTO, e é o gesto que esta fatia acrescenta. O roteiro manda o artista
/// prender o Toggle com a mão.
fn bind_the_slider(app: &mut crate::App) {
    let ids = path_ids(app);
    if ids.len() <= STAR {
        return;
    }
    let (slider, star) = (ids[2], ids[STAR]);
    let Some(gfx) = app.gfx.as_mut() else { return };
    crate::vec_widget_edit::bind(&mut gfx.sim, &app.vec_entities, slider, star);
}

/// Abre o painel autorado, do mesmo jeito que o interruptor da seção Frame o abre.
///
/// ⚠️ Escreve a MESMA chave que o chip escreve (`visibility_key`), e não um literal: se a moldura
/// for renomeada, o painel muda de identidade e um literal aqui abriria um painel que não existe —
/// em silêncio, que é a cicatriz do painel de física do W2b.
fn open_the_panel(app: &mut crate::App) {
    if let Some(hero) = app.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
        hero.panel_visibility
            .insert(ph2d_panel_authored::visibility_key(), true);
    }
}

fn announce(app: &mut crate::App) {
    let ids = path_ids(app);
    let Some(gfx) = app.gfx.as_ref() else { return };
    if ids.len() <= STAR {
        eprintln!("[ui-panel] ⚠️ a cena nao montou — PARE");
        return;
    }
    let Some(frame_e) = app
        .vec_entities
        .get(&ids[FRAME])
        .map(|&b| ph2d_ecs::Entity::from_bits(b))
    else {
        eprintln!("[ui-panel] ⚠️ a moldura nao tem entidade — PARE");
        return;
    };
    let spec = crate::ui_panel_spec::of(&gfx.sim, frame_e);
    eprintln!(
        "[ui-panel] a moldura '{}' descreve um painel de {} row(s).",
        spec.title,
        spec.rows.len()
    );
    if spec.rows.len() != 4 {
        eprintln!("[ui-panel] ⚠️ **PARE**: eram para ser 4 rows (o 'Backdrop' e' desenho puro).");
        return;
    }
    eprintln!(
        "[ui-panel] o codigo gerado:\n{}",
        ph2d_ui_codegen::emit(&spec)
    );
    eprintln!("[ui-panel] o roteiro:");
    eprintln!("  1. ⚠️ **A PROVA DA WAVE** esta' acima: o bloco de codigo. Cada row dele e' um");
    eprintln!("     filho VESTIDO da moldura, na ordem em que voce os ve na Hierarquia.");
    eprintln!("  2. ⚠️ **O CONTROLE**: o 'Backdrop' e' filho da moldura e NAO aparece no codigo.");
    eprintln!("     Ele e' desenho — um fundo —, e desenho nao vira controle. Um gerador que");
    eprintln!("     transformasse todo filho em row daria um painel com uma linha morta.");
    eprintln!("  3. Renomeie um filho na Hierarquia e re-rode: o rotulo e a chave acompanham.");
    eprintln!("     O nome que o artista digita e' o que o painel mostra.");
    eprintln!("  4. Reordene os filhos: as rows sairem noutra ordem. A ordem da arvore E' a");
    eprintln!("     decisao — e' ela que o auto layout flui.");
    eprintln!("  5. Tire o `VecWidget` de um filho (secao Widget -> nenhum): ele sai do codigo e");
    eprintln!("     volta a ser desenho.");
    eprintln!("  6. ⚠️ **O PAINEL ESTA' NA TELA** (W8b.2), docado a' ESQUERDA do inspector. Cada");
    eprintln!("     row dele e' uma linha do bloco de codigo acima, compilada. Arraste o slider,");
    eprintln!("     aperte o toggle, clique o botao: eles RESPONDEM — o comportamento e' o dos");
    eprintln!("     widgets do catalogo, nao um interpretador do canvas.");
    eprintln!("  7. ⚠️ **O 'Backdrop' nao esta' no painel** — o CONTROLE, outra vez: quem so'");
    eprintln!("     desenha nao vira row, e quem nao RESPONDE (o cabecalho de secao) nao acende");
    eprintln!("     sob o rato. Um controle que acende e nao faz nada e' pior que um que falta.");
    eprintln!("  8. Feche pelo X do painel e reabra pelo pill **UI** no topbar (ao lado do TOK).");
    eprintln!("     Ha' TRES abridores — o X, o pill, e o chip **Show as Panel** da secao Frame");
    eprintln!("     (com a moldura 'Color' selecionada) — e os tres escrevem o MESMO fato: feche");
    eprintln!("     por um e os outros dois apagam sozinhos. ⚠️ O pill e' o unico que nao exige");
    eprintln!("     a ferramenta Vector em maos nem a moldura selecionada.");
    eprintln!("  9. ⚠️ **O QUE O PAINEL MOSTRA E' A TABELA COMMITADA, nao a do log.** Renomeie um");
    eprintln!("     filho e re-rode: o CODIGO acima muda, o painel NAO — ate' alguem colar o");
    eprintln!("     codigo em crates/ph2d-panel-authored/src/generated/panel.rs e recompilar.");
    eprintln!("     E' o que codegen e'; esconder isso faria o painel parecer quebrado.");
    eprintln!(" 10. ⚠️ **A ROW MEXE NA ARTE** (W8b.3): arraste o slider **Opacity** e olhe o");
    eprintln!("     retangulo laranja a' direita — ele DESVANECE. A tinta autorada nao e'");
    eprintln!("     tocada: leve a zero e volte, e a cor volta exatamente como estava.");
    eprintln!(" 11. **O GESTO e' seu:** selecione o filho 'Visible' na Hierarquia, na secao");
    eprintln!("     Widget Skin aperte **Bind Shape...** e clique no retangulo laranja. Agora o");
    eprintln!("     toggle do painel APAGA e ACENDE a arte. (O slider ja' vinha preso — o");
    eprintln!("     roteiro traz um lado pronto e deixa o outro para voce fazer.)");
    eprintln!(" 12. ⚠️ **O CONTROLE:** selecione 'Reset' (um Button) — a linha *Drives* nao");
    eprintln!("     aparece. Um botao produz um EVENTO, nao um valor; oferecer-lhe o vinculo");
    eprintln!("     daria um conta-gotas que resolve e nao faz nada.");
    eprintln!(" 13. ⚠️ **A POSICAO do controle SOBREVIVE ao arquivo** (W8b.4): ponha o slider em");
    eprintln!("     ~30%, **Ctrl+S**, feche o app, reabra e **Ctrl+O** — a estrela volta a 30%.");
    eprintln!("     A TINTA nao foi tocada: o que o arquivo guarda e' onde o CONTROLE esta'.");
    eprintln!(" 14. E o **Ctrl+Z** move o slider de volta junto com a arte — mover um controle e'");
    eprintln!("     uma edicao, e um passo por GESTO (nao por frame). ⚠️ Abrir a cena, ao");
    eprintln!("     contrario, nao autora nada: a fila de undo nasce vazia.");
}

#[cfg(test)]
#[path = "ui_panel_smoke_tests.rs"]
mod tests;
