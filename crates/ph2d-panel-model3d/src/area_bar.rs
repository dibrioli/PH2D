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

/// ⭐⭐ **O PULLDOWN DA ÁREA — um, e é o da VISTA.**
///
/// | pulldown | rótulo | face (a leitura) | fileiras |
/// |---|---|---|---|
/// | 0 | *View* | a vista actual (`Front`…) | as 6 vistas nomeadas + os 3 gestos de câmera |
///
/// ⛔⛔ **E o GIZMO não é um segundo pulldown — ele já tem chips na fila.** Enio, 2026-09-01 (com
/// foto): *«esses botões de mover, rot e scale já existiam. só não estavam ligados a cada modo.»*
/// Um pulldown *Gizmo* foi construído e **apagado no mesmo dia**: os `MOVE`/`ROT`/`SCALE` e o
/// `SPACE` do trilho estavam ali, pintados e a acender-se — e o que lhes faltava era um
/// **consumidor**. Ver [`rail_verb_key`].
///
/// ⚠️ *Um controlo morto e um controlo ausente dão o mesmo report, e as curas são opostas:*
/// construir a segunda porta faz o app ter **dois** sítios para o mesmo verbo, e o que apodrece é
/// o que ninguém relê.
///
/// ⚠️ **O orçamento medido é `3` chips** ([`ids::area_menu_button`]) e é usado `1`.
fn menus(snap: &state::ModelSnapshot) -> [MenuPlan<'_>; 1] {
    [MenuPlan {
        label: "panel.model3d.area.view",
        face: view_face(snap),
        rows: vec![
            (&snap.views[..], ids::model3d_view_button as Family),
            (&snap.camera[..], ids::model3d_camera_button as Family),
        ],
    }]
}

/// ⭐⭐⭐ **OS CHIPS DO TRILHO QUE JÁ EXISTIAM, e a chave do verbo de cada um.**
///
/// ⛔⛔ **Eles eram CONTROLOS MORTOS, e a espécie é a 2.ª do `CLAUDE.md` §5.0:** o clique chegava
/// (o `chrome::rail_tools` fazia deles um rádio exclusivo), a luz acendia — e o valor **não chegava
/// a consumidor nenhum**. Censo de 2026-09-01, sobre a árvore inteira e **sem `head`**: fora o
/// próprio pintor, `TOOL_TRANSLATE`/`ROTATE`/`SCALE` e o `tool_space_local` do `SPACE` não têm um
/// único leitor. *Nenhuma sonda deste repo pergunta se o VALOR de um controlo chega a um efeito* —
/// o `the_rail_names_a_consumer_for_every_chip` é a primeira que o faz, e nasceu deste erro.
///
/// ⇒ com o módulo 3D no canvas, **estes chips SÃO o selector do gizmo**. Um id, um sítio.
///
/// # ⚠️ A ligação é pela CHAVE i18n, nunca pelo índice
///
/// O `slot` do intent é a posição em `ModelSnapshot::modes`, que é derivada de `Mode::ALL` no
/// shell. Uma tabela `[TOOL_TRANSLATE → 0, …]` aqui seria uma **segunda contagem**: acrescentar um
/// verbo lá dentro faria o `SCALE` mandar rodar, e nada acusaria.
///
/// ⛔⛔ **O `PIVOT` fica de fora — e ele NÃO é um dos mortos.** ⚠️ *Correcção de 2026-09-01: a 1.ª
/// redacção desta nota dizia que ele também não tinha leitor, e o `grep` que o devia ter mostrado
/// foi cortado por um `head -20`.* Ele tem **dois** consumidores no shell: o `input_dispatch` lê o
/// `ButtonState` dele para armar o arrasto **MovePivot**, e o `render_loop::snapshots` para realçar
/// o ponto do pivô.
///
/// ⇒ o gizmo deste módulo tem três verbos (`Mode::ALL`) e nenhum é *mover o pivô*, logo ele não
/// entra na tabela — mas **apaga-se** enquanto o módulo conduz o trilho (ver
/// [`publish_rail_state`]): dois chips acesos ao mesmo tempo seria a promessa do rádio quebrada, e
/// pior, a ferramenta de pivô do editor 2D ficaria **armada** por baixo de um canvas que é do 3D.
fn rail_verb_key(id: ph2d_a11y::NodeId) -> Option<&'static str> {
    RAIL_VERBS
        .iter()
        .find(|(chip, _)| *chip == id)
        .map(|(_, key)| *key)
}

/// A tabela do [`rail_verb_key`] — chip do trilho ⟷ chave do verbo.
///
/// ⚠️ `LazyLock` e não `const`: os ids do trilho são `hash_node_id` em tempo de compilação, mas a
/// chave é `&'static str` e o par tem de viver num sítio só. ⛔ Duas listas (uma para a luz, outra
/// para o clique) divergiriam no dia em que um verbo mudasse de nome.
static RAIL_VERBS: std::sync::LazyLock<[(ph2d_a11y::NodeId, &'static str); 3]> =
    std::sync::LazyLock::new(|| {
        [
            (ids::TOOL_TRANSLATE, "panel.model3d.mode.move"),
            (ids::TOOL_ROTATE, "panel.model3d.mode.rotate"),
            (ids::TOOL_SCALE, "panel.model3d.mode.scale"),
        ]
    });

/// A posição, no retrato, do verbo que este chip do trilho pede — ou `None` se o chip não é um
/// verbo ou o retrato não o tem.
pub(crate) fn rail_verb_slot(id: ph2d_a11y::NodeId, snap: &state::ModelSnapshot) -> Option<usize> {
    let key = rail_verb_key(id)?;
    snap.modes.iter().position(|c| c.key == key)
}

/// ⭐ **A luz dos chips do trilho vem da CENA, não do clique.**
///
/// ⚠️ Enquanto o módulo está armado, o `chrome::rail_tools` nunca corre para estes ids (o painel
/// consome-os antes) — logo o rádio que os acendia está desligado, e quem os acende é isto. *Fiar o
/// clique não é fiar o ESTADO.*
///
/// ⚠️ E o `SPACE` é uma **face**, não um botão de estado: ele lê `store.tool_space_local()`, então
/// publicar o referencial é escrever esse campo. ⛔ Sem isto o chip diria *Global* para sempre com
/// o gizmo em local, porque o clique deixou de passar por quem escrevia o campo.
fn publish_rail_state(store: &mut WidgetStore, snap: &state::ModelSnapshot) {
    let active = active_key(&snap.modes);
    for (chip, key) in RAIL_VERBS.iter() {
        let (chip, on) = (*chip, Some(*key) == active);
        if let Some(InteractiveState::Button { state }) = store.get_mut(chip) {
            *state = if on {
                ButtonState::Pressed
            } else {
                ButtonState::Normal
            };
        }
    }
    // ⛔⛔ **E o `PIVOT` apaga-se** — ver [`rail_verb_key`]. Com o módulo armado o painel consome os
    // verbos ANTES do `chrome::rail_tools`, logo o rádio exclusivo dele não corre e um `PIVOT` aceso
    // de antes ficaria aceso. *Quem toma o rádio herda a promessa dele: exactamente um aceso.*
    if let Some(InteractiveState::Button { state }) = store.get_mut(ids::TOOL_PIVOT) {
        *state = ButtonState::Normal;
    }
    if let Some(key) = active_key(&snap.frames) {
        store.set_tool_space_local(key == LOCAL_FRAME_KEY);
    }
}

/// ⚠️ A chave do referencial LOCAL — a face do `SPACE` é booleana (`Global`/`Local`) e o retrato é
/// uma fileira, então a ponte entre os dois é esta chave e não um índice.
const LOCAL_FRAME_KEY: &str = "panel.model3d.frame.local";

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
    // ⭐⭐ **Quem responde *«o gizmo é meu?»* é ESTA chamada, e é a única que sabe.** O `event.rs`
    // consome os ids do TRILHO (`MOVE`/`ROT`/`SCALE`/`SPACE`), e um painel do registry vê todo
    // evento mesmo fechado — sem esta bandeira ele roubaria aqueles cliques ao editor 2D. ⚠️ Ela
    // é escrita em todo quadro pela mesma porta que já recebe a verdade.
    state::set_armed(armed);
    if !armed {
        store.set_area_commands(Vec::new(), Vec::new());
        return;
    }
    let snap = state::current();
    publish_rail_state(store, &snap);
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
