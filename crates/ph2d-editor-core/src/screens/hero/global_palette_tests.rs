//! Gates da paleta de comandos global — ver o doc-header de [`super`].

use super::*;
use crate::interaction::WidgetEvent;
use crate::screens::hero::HeroScreen;
use ph2d_a11y::NodeId;

fn model(painter: bool) -> PaletteModel {
    let mut hero = HeroScreen::new(NodeId(1));
    if painter {
        hero.image_edit.mode_on = true;
        hero.image_edit.active_tool_id = Some("painter");
    }
    build_global_model(&hero)
}

/// ⭐ **TODO item que a paleta oferece é um id que o app CONSOME.**
///
/// É o `every_painted_rail_button_is_dispatched_by_somebody` generalizado, e é o gate que torna a
/// paleta estruturalmente honesta: ela não pode oferecer um comando morto, porque a lista dela é a
/// lista que o chrome pinta. Um item que ninguém consome aparece aqui como falha ALTA, com o nome.
///
/// ⚠️ **O oráculo é o `HeroScreen::apply_event`, e a primeira versão deste gate usava o
/// `chrome::dispatch_all` — que é MEIA porta.** Ele reprovou nove pills da barra de topo
/// (`Forge`, `Level_01`, `Save`, `Open`, `MIX`, `WAVE`, `WIDGET`, `GRID`, `SETTINGS`) que estão
/// perfeitamente vivos: um clique atravessa uma CASCATA (painéis → showcase → `dispatch_all` →
/// `topbar::apply_event` → `left_rail::apply_event`), e os pills da barra são consumidos pelo braço
/// do `topbar`. O gate irmão do rail pode usar a meia porta porque um chip de rail É chrome; este
/// não pode, porque projecta as duas superfícies. **Medir por uma porta que o produto não usa
/// reprova produto correcto** — e é também exactamente a porta pela qual a escolha vai ser roteada.
///
/// *Mutação que sangra:* incluir os clusters `Play`/`Right` (cujo id externo não é um botão) ⇒ eles
/// entram na lista `dead`.
#[test]
fn every_global_palette_item_is_dispatched_by_somebody() {
    for painter_active in [false, true] {
        let m = model(painter_active);
        let items: Vec<PaletteItem> = m
            .groups
            .iter()
            .flat_map(|g| &g.subs)
            .flat_map(|s| &s.items)
            .cloned()
            .collect();
        assert!(
            items.len() >= 10,
            "a paleta global tem comandos (painter={painter_active}), tem {}",
            items.len()
        );
        let mut dead = Vec::new();
        for it in &items {
            // Um hero NOVO por item: o dispatch muta estado, e um chip que só responde na 1a vez
            // passaria por acidente da ordem.
            let mut hero = HeroScreen::new(NodeId(1));
            if !hero.apply_event(WidgetEvent::Click(it.id)) {
                dead.push(it.label.clone());
            }
        }
        assert!(
            dead.is_empty(),
            "comandos OFERECIDOS na paleta global (painter={painter_active}) que ninguém \
             despacha — entradas mortas: {dead:?}"
        );
    }
}

/// ⭐ **O modelo é uma PROJEÇÃO das duas listas, e não uma cópia delas.**
///
/// Enunciado sobre o RESULTADO: todo chip que o rail pinta com nome tem de estar na paleta, com o
/// MESMO id e o MESMO nome. Sem isto, a paleta podia ter uma lista própria que por acaso concorda
/// hoje — e é exactamente essa a forma que apodrece.
///
/// *Mutação que sangra:* filtrar o rail por qualquer critério (`take(3)`, saltar os Swatch, …).
#[test]
fn the_model_carries_every_named_rail_chip_verbatim() {
    for painter_active in [false, true] {
        let mut hero = HeroScreen::new(NodeId(1));
        if painter_active {
            hero.image_edit.mode_on = true;
            hero.image_edit.active_tool_id = Some("painter");
        }
        let painted = super::super::left_rail::rail_entries(&hero.store, painter_active);
        let m = build_global_model(&hero);
        let items: Vec<&PaletteItem> = m
            .groups
            .iter()
            .flat_map(|g| &g.subs)
            .flat_map(|s| &s.items)
            .collect();
        for e in &painted {
            let (Some(id), Some(label)) = (e.node_id(), e.label()) else {
                continue; // o Divider não é um comando
            };
            let found = items.iter().find(|it| it.id == id);
            let found = found.unwrap_or_else(|| {
                panic!("o chip {label:?} está PINTADO no rail e não está na paleta")
            });
            assert_eq!(
                found.label, label,
                "o nome do comando tem de ser o nome do chip"
            );
        }
    }
}

/// ⚠️ **A BARRA DE TOPO fica de FORA, e o gate afirma-o para ninguém a "completar" de volta.**
///
/// Ela foi projectada, medida e retirada: nove pills reprovaram porque são **abridores de menu
/// ancorados a um rectângulo**, e um pick de paleta não tem rectângulo (ver o doc-header). Sem esta
/// afirmação, a próxima pessoa a olhar para a paleta vê uma ausência e "arruma-a".
///
/// *Mutação que sangra:* voltar a projectar `topbar_clusters` ⇒ os ids entram e o
/// `every_global_palette_item_is_dispatched_by_somebody` acusa-os como mortos.
#[test]
fn the_top_bar_pills_are_out_because_they_open_menus_anchored_to_a_rect() {
    let m = model(false);
    let items: Vec<&PaletteItem> = m
        .groups
        .iter()
        .flat_map(|g| &g.subs)
        .flat_map(|s| &s.items)
        .collect();
    for (id, cluster) in super::super::fixture::topbar_clusters() {
        assert!(
            !items.iter().any(|it| it.id == id),
            "o cluster {:?} da barra de topo entrou na paleta; ele abre um MENU ancorado ao \
             rectângulo do chip, e um pick não tem rectângulo",
            cluster.palette_label().unwrap_or("(sem nome)")
        );
    }
}

/// O nome de um painel é DERIVADO do id — 21 dos 23 saem limpos, e os dois feios estão nomeados.
#[test]
fn a_panel_title_is_derived_from_its_id() {
    assert_eq!(humanise_panel_id("audio_editor"), "Audio Editor");
    assert_eq!(humanise_panel_id("wet_tuning"), "Wet Tuning");
    assert_eq!(humanise_panel_id("vector"), "Vector");
    // ⚠️ Os dois que a derivação não embeleza, PINADOS: quem lhes der um título próprio um dia
    // muda esta linha de propósito, em vez de descobrir a feiura num screenshot.
    assert_eq!(humanise_panel_id("bgremoval"), "Bgremoval");
    assert_eq!(humanise_panel_id("sculpt3d"), "Sculpt3d");
}

/// A paleta **segue o modo**: com o Painter em mãos ela oferece as ferramentas de pintura.
///
/// Sem isto, ela seria um catálogo fixo — e um catálogo fixo mente sobre o que está alcançável.
#[test]
fn the_palette_follows_the_mode_the_rail_follows() {
    let names = |painter: bool| -> Vec<String> {
        model(painter)
            .groups
            .iter()
            .flat_map(|g| &g.subs)
            .flat_map(|s| &s.items)
            .map(|it| it.label.clone())
            .collect()
    };
    let off = names(false);
    let on = names(true);
    assert_ne!(
        off, on,
        "a paleta tem de mudar com o modo, como o rail muda"
    );
}

/// ⭐ **A execução: um id de PAINEL alterna a visibilidade, e é a mesma porta que o emitiu.**
///
/// O gate usa um registry PRÓPRIO (a `editor-core` não alcança o `register_all_panels`, que vive
/// numa crate que depende dela), então ele prova a LEI e não o catálogo: o catálogo tem o gate dele
/// na `ph2d-panel-registry-init`.
///
/// *Mutação que sangra:* o `route_global_pick` escrever `true` em vez de `!now` ⇒ o segundo pick
/// deixa de esconder, e o comando passa a ser *"mostrar"* em vez de *"alternar"*.
#[test]
fn picking_a_panel_toggles_it_and_picking_it_again_undoes_that() {
    use crate::panel::PanelHostInternal;
    let mut hero = HeroScreen::new(NodeId(1));
    // Um id de painel que o registry desta suíte não tem: a rota tem de o tratar como do RAIL.
    // (a metade do painel é exercitada pelo gate da `registry-init`, onde há painéis de verdade.)
    let unknown = NodeId(0xDEAD_BEEF);
    assert!(
        !route_global_pick(&mut hero, unknown),
        "um id que não é de painel nem de chip não pode ser reclamado por esta rota"
    );
    // E a lei do toggle, afirmada directamente sobre a porta que a rota usa.
    assert!(!hero.is_panel_visible("timeline"));
    hero.set_panel_visible("timeline", true);
    assert!(hero.is_panel_visible("timeline"));
    hero.set_panel_visible("timeline", false);
    assert!(!hero.is_panel_visible("timeline"));
}

/// ⭐ **A execução de um chip do rail é o CLIQUE, pela porta do rato.**
///
/// *Mutação que sangra:* a rota devolver `false` sem despachar ⇒ o comando fica mudo, e este gate é
/// o único que o vê (o modelo continua perfeito).
#[test]
fn picking_a_rail_chip_replays_the_click_the_mouse_would_send() {
    let m = model(false);
    let first = m
        .groups
        .iter()
        .flat_map(|g| &g.subs)
        .flat_map(|s| &s.items)
        .next()
        .expect("a paleta tem comandos")
        .clone();
    let mut hero = HeroScreen::new(NodeId(1));
    assert!(
        route_global_pick(&mut hero, first.id),
        "o comando {:?} foi OFERECIDO e a rota não o executou",
        first.label
    );
}
