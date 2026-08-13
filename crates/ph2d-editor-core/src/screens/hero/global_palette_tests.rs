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

/// ⚠️ **Os PILLS da barra ficam de fora, e o gate afirma-o para ninguém os "completar" de volta.**
///
/// Eles foram projectados, medidos e retirados: nove reprovaram porque são **abridores de menu
/// ancorados a um rectângulo**, e um pick de paleta não tem rectângulo (ver o doc-header). Sem esta
/// afirmação, a próxima pessoa a olhar para a paleta vê uma ausência e "arruma-a".
///
/// ⚠️ Isto **não** diz que a barra de topo fica de fora — as **rows** dos menus folha dela estão na
/// paleta desde a wave das rows, e o gate irmão `a_leaf_menu_row_lands_its_effect_with_no_menu_open`
/// é quem prova que elas pousam. O que fica de fora é o gesto de ABRIR.
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

/// ⭐ **UMA ROW DE MENU FOLHA POUSA O EFEITO DELA COM MENU NENHUM ABERTO.**
///
/// Este é o gate que carrega a wave das rows, e a distinção que ele faz é a que importa: o
/// `every_global_palette_item_is_dispatched_by_somebody` prova que a row é **consumida**, e
/// consumir não é **fazer**. Aqui o oráculo é o EFEITO — o tema muda, a escala do projecto muda —
/// e ele é medido num `HeroScreen` que **nunca abriu menu nenhum**, que é precisamente a condição
/// que um pick de paleta cria e que o gesto do rato nunca cria.
///
/// ⚠️ E ele afirma a premissa que torna a projecção legítima, em vez de a assumir: os handlers
/// destas rows só tocam o contexto de menu por `close_context_menu()`, que **é um no-op sem nada
/// aberto** — lido handler a handler, não por grep sobre um nome de função.
///
/// *Mutação que sangra:* fazer o `theme.rs` ler o menu aberto antes de escrever (`let Some(_) =
/// hero.store.context_menu() else { return false }`) ⇒ o tema deixa de mudar e este gate acusa,
/// enquanto o gate de despacho continua VERDE (ele só pergunta se alguém devolveu `true`).
#[test]
fn a_leaf_menu_row_lands_its_effect_with_no_menu_open() {
    use crate::ids;
    let mut hero = HeroScreen::new(NodeId(1));
    assert!(
        hero.store.context_menu().is_none(),
        "a premissa do gate é que NÃO há menu aberto — é a condição que um pick de paleta cria"
    );

    let before = hero.theme;
    assert!(route_global_pick(&mut hero, ids::CTX_MENU_THEME_BLUEPRINT));
    assert_ne!(
        hero.theme, before,
        "a row de tema foi consumida e não FEZ nada — consumir não é fazer"
    );
    assert_eq!(hero.theme, ph2d_tokens::Theme::Blueprint);

    // E uma row de SUBMENU, que é a outra metade do que a projecção oferece: ela vive dois níveis
    // fundo no gesto do rato (pill -> cascata -> row) e mesmo assim é endereçável por id.
    assert!(route_global_pick(&mut hero, ids::CTX_MENU_PPM_256));
    assert!(
        (hero.project.pixels_per_meter - 256.0).abs() < f32::EPSILON,
        "a row de submenu não pousou: ppm = {}",
        hero.project.pixels_per_meter
    );
}

/// ⚠️ **As rows de CASCATA ficam de fora, e o motivo é o mesmo dos pills — um RECTÂNGULO.**
///
/// O `SettingsMenu` parece uma folha (seis rows com nome), e não é: cada uma chama
/// `cascade_anchor(hero, id)` e abre um submenu **ancorado ao rectângulo da row clicada**. Servida
/// pela paleta ela seria consumida (devolve `true`) e abriria um menu numa posição derivada de uma
/// row que ninguém pintou — um menu no canto, sem gesto que o tenha pedido.
///
/// ⇒ *ser consumido não basta para ser servível*: o gate de despacho aprovaria estas seis.
///
/// *Mutação que sangra:* acrescentar `ContextMenuKind::SettingsMenu` ao `TOPBAR_LEAF_MENUS`.
#[test]
fn the_cascade_rows_are_out_because_they_anchor_a_submenu_to_a_rect() {
    use crate::interaction::ContextMenuKind;
    let m = model(false);
    let ids_offered: Vec<NodeId> = m
        .groups
        .iter()
        .flat_map(|g| &g.subs)
        .flat_map(|s| &s.items)
        .map(|it| it.id)
        .collect();
    for (id, label, _) in super::super::menu_rows::menu_rows(ContextMenuKind::SettingsMenu) {
        assert!(
            !ids_offered.contains(id),
            "a row de cascata {label:?} entrou na paleta; ela ABRE um submenu ancorado ao \
             rectângulo dela, e um pick não tem rectângulo"
        );
    }
    // …e o CONTROLE: a cascata está fora, mas o que ela alcança está DENTRO — senão a exclusão
    // teria levado embora o cluster de Settings inteiro.
    assert!(
        ids_offered.contains(&crate::ids::CTX_MENU_PPM_256),
        "os presets ATRÁS da cascata têm de continuar alcançáveis pela paleta"
    );
}

/// ⭐ **O grupo dos menus é uma PROJEÇÃO da tabela do pintor, não uma cópia dela.**
///
/// Enunciado sobre o RESULTADO, como o irmão do rail: toda row que o pintor de menus desenha para
/// um menu folha está na paleta, com o MESMO id. Uma lista própria que por acaso concorda hoje é
/// exactamente a forma que apodrece na próxima row.
///
/// *Mutação que sangra:* um `.take(2)` na projecção, ou tirar um `ContextMenuKind` da lista.
#[test]
fn the_menu_group_carries_every_leaf_row_verbatim() {
    let m = model(false);
    let items: Vec<&PaletteItem> = m
        .groups
        .iter()
        .flat_map(|g| &g.subs)
        .flat_map(|s| &s.items)
        .collect();
    let mut seen = 0usize;
    for kind in super::super::menu_rows::TOPBAR_LEAF_MENUS {
        for (id, label, _) in super::super::menu_rows::menu_rows(*kind) {
            let found = items.iter().find(|it| it.id == *id).unwrap_or_else(|| {
                panic!("a row {label:?} de {kind:?} é PINTADA no menu e não está na paleta")
            });
            assert_eq!(found.label, palette_label(label));
            seen += 1;
        }
    }
    // O número é DERIVADO da tabela, não escrito à mão: um literal só sabe dizer *"mudou"*, e a
    // pergunta é *a projecção continua completa?*. O piso existe para a varredura não passar vazia.
    assert!(
        seen >= 20,
        "a projecção dos menus ficou pequena demais para ser o que se afirma: {seen} rows"
    );
}

/// O rótulo perde o **recuo codificado como caractere** e mais nada — o atalho fica.
///
/// *Mutação que sangra:* cortar também no `·` ⇒ o atalho desaparece do rótulo.
///
/// ⚠️ E a **primeira versão dessa mutação SOBREVIVEU, por ser inválida e não por buraco de gate**:
/// eu encadeei o `split` ANTES do `strip_prefix`, e o `.unwrap_or(label)` do strip — que cai de
/// volta no rótulo ORIGINAL — desfazia o corte sempre que não havia travessão, que é exactamente o
/// caso do `Save · Cmd+S`. *A mutação cancelava-se a si própria.* Cortada DEPOIS do travessão, onde
/// o fallback não a alcança, ela sangra.
#[test]
fn the_palette_label_drops_the_indent_dash_and_nothing_else() {
    assert_eq!(palette_label("\u{2014} Corners: Sharp"), "Corners: Sharp");
    assert_eq!(palette_label("Forge (dark)"), "Forge (dark)");
    // O atalho é informação que um command palette MOSTRA — não é ruído a limpar.
    assert_eq!(palette_label("Save \u{00b7} Cmd+S"), "Save \u{00b7} Cmd+S");
}
