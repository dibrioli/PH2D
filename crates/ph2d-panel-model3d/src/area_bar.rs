//! ⭐⭐⭐ **O QUE SAIU DO PAINEL E FOI PARA A FILA** — a metade 2 da **D2**, sem faixa nova.
//!
//! # ⛔⛔ O defeito que isto cura, medido
//!
//! O painel deste módulo tinha **74 entradas**, e só **8** são propriedades do objecto
//! (`docs/UI_New_and_Simple/00_DECISOES_DO_ENIO.md` §D2). As outras **66 têm outro dono** — ele é
//! *o depósito por omissão* de tudo o que não tinha sítio. O sintoma é o próprio ficheiro do
//! painel: ele precisou de **barra de rolagem** (report do Enio, 2026-08-27), porque não cabia.
//!
//! ⇒ as **nove** primeiras entradas mudam-se: as seis **vistas nomeadas** e os três gestos de
//! **câmera**. Elas nunca foram propriedades de nada — são sobre *olhar*, e o dono delas é a área
//! que tem o canvas.
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
//! ⛔⛔ **E ele é UM pulldown, não nove chips — medido.** Com as nove entradas cruas a fila precisa
//! de **2 linhas até no iPad 12,9"**, o maior dos três alvos. *Poupar altura gastando largura não
//! poupa nada.*
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
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{ButtonState, ToolRailEntry};
use ph2d_i18n::tr;

use crate::populate::MAX_MODES;
use crate::state::{self, ModeChip};

/// As duas fileiras que deixaram o painel, e a família de id de cada uma.
///
/// ⚠️ **Uma lista, e não dois blocos copiados**: acrescentar uma terceira fileira à fila é
/// acrescentar uma linha aqui, e o `publish` segue — foi a fileira esquecida no `CHIP_FAMILIES`
/// que deu *"nenhum botão funcionou"* (ver [`crate::populate`]).
type Family = fn(u32) -> ph2d_a11y::NodeId;

fn rows(snap: &state::ModelSnapshot) -> [(&[ModeChip], Family); 2] {
    [
        (&snap.views, ids::model3d_view_button),
        (&snap.camera, ids::model3d_camera_button),
    ]
}

/// ⭐ **Publica os chips da área na fila, e o estado de cada um.**
///
/// `armed` é *«o módulo tem o canvas»* — a mesma pergunta que o shell já faz para armar a cena
/// (`field3d_smoke::set_armed_by_panel`).
///
/// ⚠️ **Chamado em TODO quadro, desarmado incluído.** Escrever só quando o módulo está aberto
/// deixaria os nove chips na fila depois de ele fechar: pintados, a despachar para um painel que já
/// não está lá, e a roubar lugar às ferramentas. *Um mapa que o tique apaga não envelhece.*
pub fn publish(store: &mut WidgetStore, armed: bool) {
    if !armed {
        store.set_area_commands("", Vec::new());
        return;
    }
    let snap = state::current();
    let mut out: Vec<ToolRailEntry> = Vec::new();
    for (chips, id_of) in rows(&snap) {
        // ⚠️ **O mesmo tecto do painel** (`MAX_MODES`): o `populate` cunha os ids às cegas, então
        // uma fileira mais longa do que a família teria chips sem registo — pintados e mortos sob
        // o dedo, que é o modo de falha mais caro desta casa.
        for (slot, chip) in chips.iter().take(MAX_MODES as usize).enumerate() {
            let id = id_of(slot as u32);
            let label = tr(chip.key);
            out.push(ToolRailEntry::compound(id, label, label, ""));
            // ⭐ **A segunda metade** — ver o cabeçalho do módulo.
            if let Some(InteractiveState::Button { state }) = store.get_mut(id) {
                *state = if chip.active {
                    ButtonState::Pressed
                } else {
                    ButtonState::Normal
                };
            }
        }
    }
    // ⭐ **A FACE é uma leitura** — qual é a vista agora, derivada da câmera pelo shell. ⚠️ Um
    // retrato que ninguém publicou traz a chave vazia, e `tr("")` não é uma pergunta com resposta.
    let face = if snap.view_label.is_empty() {
        tr(FALLBACK_FACE)
    } else {
        tr(snap.view_label)
    };
    store.set_area_commands(face, out);
}

/// O que a face diz quando ainda não há retrato — o nome da vista livre, que é o que uma câmera
/// acabada de nascer de facto é.
const FALLBACK_FACE: &str = "viewport.model3d.view.user";
