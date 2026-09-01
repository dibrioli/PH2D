//! ⭐⭐⭐ **O QUE SAIU DO PAINEL E FOI PARA A FILA** — a metade 2 da **D2**, sem faixa nova.
//!
//! # ⛔⛔ O defeito que isto cura, medido
//!
//! O painel deste módulo tinha **74 entradas**, e só **8** são propriedades do objecto
//! (`docs/UI_New_and_Simple/00_DECISOES_DO_ENIO.md` §D2). As outras **66 têm outro dono** — ele é
//! *o depósito por omissão* de tudo o que não tinha sítio. O sintoma é o próprio ficheiro do
//! painel: ele precisou de **barra de rolagem** (report do Enio, 2026-08-27), porque não cabia.
//!
//! ⇒ **dezassete** entradas mudam-se, e para **DOIS** sítios, porque o corte da D2 é por **âmbito**:
//!
//! | fileiras | nº | destino | critério |
//! |---|---:|---|---|
//! | as 6 vistas nomeadas + os 3 gestos de câmera | 9 | pulldown **0** (*View*) | é sobre **olhar** |
//! | os 3 verbos do gizmo + os 2 referenciais | 5 | pulldown **1** (*Gizmo*) | é sobre **mover com a mão** |
//! | os 3 níveis de exportação | 3 | menu **File** do app | escrever um arquivo vale em **todo o app** |
//!
//! ⛔ **A exportação é do módulo 3D e mesmo assim não é do editor 3D.** *«Vai para onde o comando
//! VALE, não para onde ele nasceu.»*
//!
//! # ⚠️ E o dono é a FILA, não um cabeçalho próprio
//!
//! > Enio, 2026-08-31: *«esse app tem tablets e iPad como alvo. Não podemos ir perdendo espaço.»*
//!
//! O *cabeçalho por área* foi construído em 2026-08-31 e **revertido no mesmo dia**: `28 px` de
//! altura permanente, `−1,5` ponto de área de desenho no alvo declarado. ⇒ o inquilino tem de
//! caber **onde já se paga altura**, e a fila de ferramentas já é uma **região da área**, já
//! subtrai a altura que subtrai, e desde a entrega 32 já sabe **transbordar** para o `⋯`.
//!
//! ⛔⛔ **E são PULLDOWNS, não chips crus — medido.** Com as nove entradas cruas a fila precisa
//! de **2 linhas até no iPad 12,9"**, o maior dos três alvos. *Poupar altura gastando largura não
//! poupa nada.*
//!
//! ⭐ **O orçamento medido (2026-09-01) é `3` chips de área e usam-se `2`** — ver
//! [`ids::area_menu_button`] para a tabela dos três alvos. ⛔ **Foi ela que escolheu o desenho:** a
//! pergunta em aberto era se os três verbos do gizmo podiam ser chips CRUS (um clique, como no
//! Godot), e `1 + 3 = 4` põe o iPad 11 e o mini em duas linhas.
//!
//! # ⚠️ Os ids são os MESMOS, e é isso que faz o clique continuar a chegar
//!
//! `HeroScreen::apply_event` entrega todo evento a todo painel do registry e o braço decide por
//! **id**, nunca por posição — logo um chip pintado na fila despacha para o mesmo
//! `ModelIntent::SetView` de sempre, sem uma segunda porta. ⛔ Um id novo aqui seria *um comando
//! com dois sítios a apodrecer em separado* (a lei do `menu_bar`).
//!
//! # ⭐ Uma função, DUAS metades — e a segunda é a que se esquece
//!
//! [`publish`] escreve as entradas **e** o `ButtonState` de cada uma. O chip da fila resolve o
//! aspecto de *«é esta a vista actual?»* pelo `store.button_state(id)` — quem publicasse só a
//! lista teria seis chips a dizer a mesma coisa em todas as vistas. *Fiar o clique não é fiar o
//! ESTADO* (a lei que a `menu_bar::publish_toggle_state` pagou).

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{AreaMenu, ContextMenuKind, InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{ButtonState, ToolRailEntry};
use ph2d_i18n::tr;

use crate::populate::MAX_MODES;
use crate::state::{self, ModeChip};

/// A família de id de uma fileira que deixou o painel.
///
/// ⚠️ **Uma lista, e não blocos copiados**: acrescentar uma fileira à fila é acrescentar uma linha
/// numa tabela, e o `publish` segue — foi a fileira esquecida no `CHIP_FAMILIES` que deu
/// *"nenhum botão funcionou"* (ver [`crate::populate`]).
type Family = fn(u32) -> ph2d_a11y::NodeId;

/// Uma fileira do retrato e a família de id dela — o par que o [`entries`] percorre.
type Row<'a> = (&'a [ModeChip], Family);

/// **A receita de um pulldown antes de ele virar [`AreaMenu`]** — as chaves i18n em vez do texto.
///
/// ⚠️ Ela existe para a tabela abaixo caber numa assinatura legível, e a fronteira é a tradução:
/// aqui viajam **chaves** (HR-15), e quem chama `tr` é o [`publish`].
struct MenuPlan<'a> {
    label: &'static str,
    face: &'static str,
    rows: Vec<Row<'a>>,
}

/// ⭐⭐⭐ **OS DOIS PULLDOWNS, e o critério do agrupamento é a FACE.**
///
/// | pulldown | rótulo | face (a leitura) | fileiras |
/// |---|---|---|---|
/// | 0 | *View* | a vista actual (`Front`…) | as 6 vistas nomeadas + os 3 gestos de câmera |
/// | 1 | *Gizmo* | o verbo actual (`Move`…) | os 3 verbos + os 2 referenciais |
///
/// ⛔ **NÃO é um pulldown só com as cinco fileiras.** Catorze linhas atrás de uma face só é o
/// depósito da foto 3 mudado de sítio — e a face **deixaria de ser uma leitura**, porque duas
/// grandezas independentes não cabem numa palavra.
///
/// ⭐ **E o referencial fica com o VERBO, não com a vista.** *"Um referencial sem verbo não quer
/// dizer nada"* — é o que o `paint.rs` já dizia quando as duas fileiras eram vizinhas no painel, e
/// separá-las agora poria a metade num sítio e a outra noutro.
///
/// ⚠️ **O orçamento medido é `3` chips** ([`ids::area_menu_button`]) e são usados `2`: o 4.º põe o
/// iPad 11 e o mini em duas linhas.
fn menus(snap: &state::ModelSnapshot) -> [MenuPlan<'_>; 2] {
    [
        MenuPlan {
            label: "panel.model3d.area.view",
            face: view_face(snap),
            rows: vec![
                (&snap.views[..], ids::model3d_view_button as Family),
                (&snap.camera[..], ids::model3d_camera_button as Family),
            ],
        },
        MenuPlan {
            label: "panel.model3d.area.gizmo",
            face: active_key(&snap.modes).unwrap_or(FALLBACK_GIZMO_FACE),
            rows: vec![
                (&snap.modes[..], ids::model3d_mode_button as Family),
                (&snap.frames[..], ids::model3d_frame_button as Family),
            ],
        },
    ]
}

/// ⭐ **Publica os pulldowns da área e o que o módulo põe no menu *File*, com o estado de cada
/// linha.**
///
/// `armed` é *«o módulo tem o canvas»* — a mesma pergunta que o shell já faz para armar a cena
/// (`field3d_smoke::set_armed_by_panel`).
///
/// ⚠️ **Chamado em TODO quadro, desarmado incluído.** Escrever só quando o módulo está aberto
/// deixaria os chips na fila depois de ele fechar: pintados, a despachar para um painel que já
/// não está lá, e a roubar lugar às ferramentas — e uma linha *Export Draft* no menu *File* que
/// não exporta nada. *Um mapa que o tique apaga não envelhece.*
pub fn publish(store: &mut WidgetStore, armed: bool) {
    if !armed {
        store.set_area_commands(Vec::new(), Vec::new());
        return;
    }
    let snap = state::current();
    let out = menus(&snap)
        .into_iter()
        .map(|plan| AreaMenu {
            label: tr(plan.label).to_string(),
            face: tr(plan.face).to_string(),
            rows: entries(store, &plan.rows),
        })
        .collect();
    // ⭐⭐ **E a SAÍDA vai ao menu do app, não a um pulldown de área** — escrever um arquivo vale em
    // todo o app, e o corte da **D2** é por âmbito (`00_DECISOES_DO_ENIO.md` §D2, a tabela de
    // destino diz *«barra global → Arquivo»* para as três).
    let file = vec![(
        ContextMenuKind::MenuBarFile,
        entries(store, &[(&snap.exports[..], ids::model3d_export_button)]),
    )];
    store.set_area_commands(out, file);
}

/// Monta as entradas de uma lista de fileiras **e escreve o `ButtonState` de cada uma**.
///
/// ⭐ **A segunda metade é a que se esquece** — ver o cabeçalho do módulo. O chip resolve *«é este o
/// estado actual?»* pelo `store.button_state(id)`, e quem publicasse só a lista teria seis chips a
/// dizer a mesma coisa em todas as vistas.
fn entries(store: &mut WidgetStore, rows: &[Row<'_>]) -> Vec<ToolRailEntry> {
    let mut out: Vec<ToolRailEntry> = Vec::new();
    for (chips, id_of) in rows {
        // ⚠️ **O mesmo tecto do painel** (`MAX_MODES`): o `populate` cunha os ids às cegas, então
        // uma fileira mais longa do que a família teria chips sem registo — pintados e mortos sob
        // o dedo, que é o modo de falha mais caro desta casa.
        for (slot, chip) in chips.iter().take(MAX_MODES as usize).enumerate() {
            let id = id_of(slot as u32);
            let label = tr(chip.key);
            out.push(ToolRailEntry::compound(id, label, label, ""));
            if let Some(InteractiveState::Button { state }) = store.get_mut(id) {
                *state = if chip.active {
                    ButtonState::Pressed
                } else {
                    ButtonState::Normal
                };
            }
        }
    }
    out
}

/// ⭐ **A FACE da vista é uma leitura** — qual é a vista agora, derivada da câmera pelo shell.
/// ⚠️ Um retrato que ninguém publicou traz a chave vazia, e `tr("")` não é uma pergunta com
/// resposta.
fn view_face(snap: &state::ModelSnapshot) -> &'static str {
    if snap.view_label.is_empty() {
        FALLBACK_FACE
    } else {
        snap.view_label
    }
}

/// A chave do chip ACESO de uma fileira — a face de um pulldown cujo estado é um dos itens dele.
///
/// ⚠️ **Derivada da fileira e não de um campo à parte**: o `active` já é a verdade que o painel
/// pintava, e um segundo espelho divergiria dela no quadro em que alguém trocasse o verbo por uma
/// tecla.
fn active_key(chips: &[ModeChip]) -> Option<&'static str> {
    chips.iter().find(|c| c.active).map(|c| c.key)
}

/// O que a face diz quando ainda não há retrato — o nome da vista livre, que é o que uma câmera
/// acabada de nascer de facto é.
const FALLBACK_FACE: &str = "viewport.model3d.view.user";
/// E o verbo de omissão do gizmo, pela mesma razão.
const FALLBACK_GIZMO_FACE: &str = "panel.model3d.mode.move";
