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

pub(crate) fn frame(app: &mut crate::App, f: u32) {
    match f {
        3 => build(app),
        5 => name_and_parent(app),
        7 => announce(app),
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
}

fn announce(app: &mut crate::App) {
    let ids = path_ids(app);
    let Some(gfx) = app.gfx.as_ref() else { return };
    if ids.len() < AUTHORED.len() {
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
    eprintln!(" 6. ⚠️ **O QUE AINDA NAO ESTA' AQUI, e e' a fatia seguinte:** este codigo nao e'");
    eprintln!("     compilado nem registrado — ele e' o ARTEFATO. Faze-lo virar um painel vivo");
    eprintln!("     (a crate, o registro, o runtime das rows) e' o W8b.2.");
}

#[cfg(test)]
#[path = "ui_panel_smoke_tests.rs"]
mod tests;
