//! **A MÁQUINA DE MORPH A CORRER** (plano 32 W5) — quem faz a forma virar a outra.
//!
//! # ⚠️ Ela só corre num MODO, e isso não é conservadorismo
//!
//! A condição de uma seta é uma **acção do Input Map**, isto é, uma tecla. Se a máquina escutasse
//! enquanto o artista edita, carregar em `Z` faria a forma mudar **e** o que quer que o `Z` faça no
//! editor — os dois, sem que nada na tela explicasse. É o argumento do [`crate::render_loop::ui_preview`]
//! (*"um hover que animasse a forma enquanto o artista trabalha tornaria o editor inutilizável"*)
//! com outro dispositivo de entrada, e a resposta é a mesma: **um modo**.
//!
//! # ⛔⛔ O PLAYHEAD ERA A PORTA E DEIXOU DE SER — e a diferença é uma medição, não um gosto
//!
//! A W5 escreveu que *"o modo já existe: neste editor, o jogo a correr é o playhead a andar"*, e
//! era um argumento bom sobre a coisa errada. **O playhead não tranca o teclado do editor.** Com
//! ele a andar, as teclas continuam a chegar aos atalhos — então a mesma tecla morfa a forma **e**
//! faz o que ela faz no editor, que é exactamente o que a nota dizia estar a evitar.
//!
//! Enio, 2026-08-25, depois do smoke: *"precisamos de um modo preview (com botão) como o de states
//! de animação pois senão temos conflitos de atalhos (como setas do teclado movendo as formas)"*.
//!
//! ⇒ a porta é o **interruptor `Preview`** da seção *Morph States*, e ele **toma o teclado**
//! (`input_dispatch::keyboard`, logo depois do retrato dos dispositivos e antes de todo atalho).
//! ⛔ **Uma porta, não duas:** deixar o playhead a dirigir também manteria o conflito viva na porta
//! que não tranca nada — e *duas portas para o mesmo modo divergem em silêncio*.
//!
//! ⚠️ *Um modo cuja entrada não exclui os outros consumidores não é um modo — é mais um produtor.*
//!
//! # ⚠️ O que ela escreve é PRÉ-VISUALIZAÇÃO, e o undo não a vê
//!
//! A máquina escreve **dois** campos do `VecMorph` — o par e o `t` —, e os dois passam pelo ledger
//! ([`crate::preview_drive`]). ⛔ **O `Driver::MorphT` sozinho não bastava:** ele cobre o `t` e
//! **só** o `t`, e sem o `MorphPair` uma transição durante a reprodução entraria no undo como se o
//! artista tivesse re-ligado as fontes à mão.
//!
//! # ⚠️⚠️ E parar o relógio NÃO devolve a forma autorada — a nota anterior estava ERRADA
//!
//! Ela dizia *"ao largar as máquinas a cena volta ao que o artista desenhou"*. **Não volta**, e a
//! lei que manda é a da [`crate::preview_drive::PreviewDrive::settle`]: o ledger repõe o autorado
//! **dentro da fotografia** enquanto o motor conduz, e no primeiro quadro em que ele **para** a
//! entrada morre ⇒ a captura seguinte vê o vivo, difere do baseline e regista **UM** passo. É o
//! *«desfaz a corrida»*, e aqui ele significa: *sair do modo COMPROMETE a forma em que se ficou*.
//!
//! ⚠️ **Isso é o comportamento certo** (o artista carrega em ▶, sai, e o objecto ficou onde ele o
//! pôs — desfazível num Ctrl+Z) **e tem uma consequência que custou o report de 2026-08-26**: a
//! máquina seguinte nasce **depois** de o componente já estar noutra forma. Por isso ela é
//! **semeada pelo mundo** ([`open`]) e não por `graph.start()`.

use std::collections::BTreeMap;

use ph2d_ecs::{Entity, SimWorld, VecMorph, VecMorphMachine};
use ph2d_input::Input;
use ph2d_morph_machine::MorphMachine;

use crate::preview_drive::{Driven, PreviewDrive};

/// As máquinas VIVAS, por entidade de Morph.
///
/// ⚠️ **Não são serializadas, e não podem ser:** uma máquina é *onde a forma está agora*, e o
/// documento guarda *quais são as setas*. Salvá-la faria um projecto reabrir a meio de uma
/// transição. Mesma lei, palavra por palavra, das `UiMachines`.
///
/// ⚠️ `BTreeMap` e não `HashMap` — a espinha do determinismo deste repo (lint estrutural).
pub(crate) type MorphMachines = BTreeMap<u64, MorphMachine>;

/// **Um quadro da máquina.** Devolve quantas máquinas correram.
///
/// `active` é o **modo de pré-visualização**: falso ⇒ as máquinas são **largadas** e nada é escrito
/// (o ledger devolve o autorado sozinho, na próxima captura).
///
/// ⚠️ **O nome é `active` e não `playing` de propósito** — ele deixou de ser o playhead na W9, e um
/// parâmetro que continuasse a chamar-se `playing` faria a próxima leitura procurar o transporte.
pub(crate) fn tick(
    machines: &mut MorphMachines,
    sim: &mut SimWorld,
    map_paths: &crate::vec_entities::VecEntityMap,
    // ⚠️ **O par `InputMap` + `ActionState` viaja JUNTO**, e não como dois argumentos: eles só
    // existem para construir o `Input`, e separá-los deixaria um chamador livre para passar o
    // estado de um mapa com o mapa de outro. (Também é o que traz a assinatura de volta ao teto
    // de 7 do clippy — a cura certa era a que já estava certa por outra razão.)
    input: &Input<'_>,
    active: bool,
    dt: f64,
    drive: &mut PreviewDrive,
) -> usize {
    if !active {
        // ⭐ **Largar COMPROMETE.** Não há «voltar ao estado inicial» aqui — e também não há volta
        // ao autorado: a `settle` promove o vivo a documento no quadro seguinte, e a forma em que o
        // artista ficou vira **um** passo de undo. É por isso que a máquina seguinte é semeada pelo
        // MUNDO (`open`) e não pela primeira forma da lista.
        machines.clear();
        return 0;
    }
    // ⭐ **O grafo é DERIVADO dos filhos, a cada quadro** (W11) — `morph_set::graph_of`. É por
    // isso que arrastar uma forma para dentro do conjunto na Hierarquia a faz participar sem que
    // uma linha de código reaja ao gesto: no quadro seguinte ela simplesmente **está na lista**.
    let bits: Vec<u64> = sim
        .world_mut()
        .query::<(Entity, &VecMorphMachine)>()
        .iter(sim.world())
        .map(|(e, _)| e.to_bits())
        .collect();
    let hosts: Vec<(u64, ph2d_morph_machine::MorphGraph)> = bits
        .into_iter()
        .map(|b| {
            (
                b,
                crate::morph_set::graph_of(sim, map_paths, Entity::from_bits(b)),
            )
        })
        .collect();
    // Uma máquina cuja entidade morreu (ou perdeu as setas) some junto — senão ela sobreviveria ao
    // objecto e o mapa cresceria para sempre. Mesma varredura das `UiMachines`.
    machines.retain(|k, _| hosts.iter().any(|(h, _)| h == k));

    let mut ran = 0;
    for (bits, graph) in hosts {
        let e = Entity::from_bits(bits);
        let m = open(machines, sim, e, &graph);
        // ⚠️ **Só o que ACABOU de ser carregado dispara.** Com `pressed` uma tecla segurada
        // re-disparava a cada quadro e a máquina saltaria a cadeia inteira num piscar de olhos.
        for a in m.live_actions(&graph) {
            if input.just_pressed(a) {
                m.fire(&graph, a);
                break;
            }
        }
        m.advance(&graph, dt);
        let (pair, t) = ([m.pair().0, m.pair().1], m.t());
        // ⚠️ **O ledger primeiro, a escrita depois** — ele precisa do valor ANTES para saber o que
        // repor. Escrever e só então registar guardaria o valor do motor como se fosse o autorado.
        write_driven(sim, e, drive, Driven::MorphPair(pair));
        write_driven(sim, e, drive, Driven::MorphT(t));
        ran += 1;
    }
    ran
}

/// Regista o valor ANTES no ledger e escreve o novo — a porta única das duas metades.
fn write_driven(sim: &mut SimWorld, e: Entity, drive: &mut PreviewDrive, after: Driven) {
    let Some(before) = Driven::read(after.driver(), sim, e) else {
        return;
    };
    if before == after {
        return;
    }
    drive.driven(e, before, after);
    let Some(mut m) = sim.world_mut().get_mut::<VecMorph>(e) else {
        return;
    };
    match after {
        Driven::MorphPair(p) => m.sources = p,
        Driven::MorphT(v) => m.t = v,
        _ => {}
    }
}

/// ⭐⭐⭐ **O CONJUNTO DE ESTADOS ANIMADO POR UMA TRANSIÇÃO DE UI** (plano 32 W11c).
///
/// Enio, 2026-08-26: *"Assegure-se que esse sistema de states em morph seja integrado e
/// completamente compatível com o sistema de States previamente existente, ou seja, que eu possa
/// usar o state morph nas animações criadas em States."*
///
/// # A costura, e por que ela é UMA função de dez linhas
///
/// O trabalho verdadeiro está feito de outro lado: a pose grava **que forma** (`ObjectPose::
/// morph_shape`), e a `Transition` diz **de que forma para que forma, e a que altura**
/// (`morph_steps`). Aqui só se escreve o que isso significa no mundo — o par e o `t` do
/// [`VecMorph`], exactamente os dois campos que a máquina do Morph já escreve.
///
/// ⚠️ **Pelo LEDGER**, como tudo o que um motor escreve: isto é pré-visualização — vê-se, não se
/// guarda nem se desfaz. Sem ele, passar o rato por um botão registaria um passo de undo.
///
/// ⛔ **Só sobre quem TEM máquina.** Um morph autorado à mão tem o `t` conduzido pela timeline;
/// escrever nele a partir de uma pose mataria a curva dela.
///
/// ⚠️ **Ele corre DEPOIS do [`tick`] no quadro**, e a ordem é uma decisão: se as duas coisas
/// escrevem o mesmo objecto, quem manda é a **transição de UI** — ela é o gesto que o artista
/// acabou de fazer (um hover), e a máquina de teclas é o estado de fundo.
pub(crate) fn apply_ui_steps(
    sim: &mut SimWorld,
    map: &crate::vec_entities::VecEntityMap,
    steps: &[ph2d_ui_state::MorphStep],
    drive: &mut PreviewDrive,
) -> usize {
    let mut n = 0;
    for st in steps {
        let Some(&bits) = map.get(&st.id) else {
            continue;
        };
        let e = Entity::from_bits(bits);
        if sim.world().get::<VecMorphMachine>(e).is_none() {
            continue;
        }
        #[allow(clippy::cast_possible_truncation)]
        let t = st.t as f32;
        write_driven(sim, e, drive, Driven::MorphPair([st.from, st.to]));
        write_driven(sim, e, drive, Driven::MorphT(t));
        n += 1;
    }
    n
}

/// ⭐⭐⭐ **O BOTÃO ▶ DE UMA FORMA** — a porta do verbo de mundo (plano 32 W11d).
///
/// Enio, 2026-08-26: *"na animação de States, o morph não consegue segurar os estados atribuidos no
/// momento do Rec. (…) para animações de states eventos atribuidos para Morph states não devem ser
/// necessários, pois os estados morph são mudados com play"*.
///
/// # ⛔ Por que ela EXISTE, em vez de duas linhas no braço do despacho
///
/// O mapa de máquinas é **propriedade do [`tick`]**, e o `tick` **esvazia-o** em todo quadro fora do
/// modo. O verbo corre DEPOIS do `tick` no mesmo quadro ⇒ escrito como um `get_mut`, ele encontrava
/// o mapa **vazio** vindo de fora da pré-visualização, ligava o modo e **não viajava**: a forma só
/// mudava ao segundo clique, e o **Rec** seguinte fotografava a forma errada.
///
/// ⇒ ela **abre a máquina** pela mesma porta que o `tick` (`open`), e é isso que a torna igual em
/// qualquer dos dois quadros.
///
/// ⚠️ **Ela não olha para tecla nenhuma**, e é o pedido do Enio palavra por palavra: `travel` é a
/// porta *sem condição*, então um conjunto sem uma única acção atribuída é integralmente
/// conduzível — que é o que uma animação de **States** precisa.
///
/// ⚠️ **Ela não liga o modo**, de propósito: quem o liga é o despacho, e pôr o interruptor aqui
/// dentro daria duas respostas a *"quem decide o modo"*.
pub(crate) fn play(
    machines: &mut MorphMachines,
    sim: &SimWorld,
    map: &crate::vec_entities::VecEntityMap,
    host: Entity,
    row: usize,
) -> bool {
    let graph = crate::morph_set::graph_of(sim, map, host);
    open(machines, sim, host, &graph).travel(&graph, row)
}

/// ⭐⭐⭐ **O PAR DESENHADO TEM DE NOMEAR MEMBROS** — a reconciliação, todo quadro (plano 32 W11g).
///
/// Enio, 2026-08-26, 3.º report: *"desconectar muda correctamente na hierarquia e painel, mas deixa
/// a imagem de resquício no canvas e o nome de resquício no painel"*.
///
/// # ⛔⛔ O mecanismo, medido
///
/// A lista de estados é **derivada** dos filhos (W11) — mas o par que a cena DESENHA
/// (`VecMorph::sources`) é **guardado**, e nada o reconciliava. Medido: um conjunto a mostrar a
/// forma `0`, o artista carrega no ⊘ dessa forma, e o `sources` fica em **`[0, 0]`** com a lista já
/// em `[1, 2, 3]`. ⇒ **dois** resquícios, um mecanismo:
///
/// - o `morph_live::recook` continua a cozer a forma que saiu ⇒ ela aparece **duas vezes** no
///   canvas (solta, no sítio dela, e clonada dentro do conjunto);
/// - o `vec_morph_edit::publish` lê `sources[1]` para o readout ⇒ o painel **nomeia** a forma que
///   já não é estado.
///
/// ⚠️ **É a MESMA família da W11f**, um valor depois: *a lista passou a ser derivada e dois valores
/// guardados não a acompanharam* — a visibilidade ontem, o par hoje. ⛔ O terceiro candidato está
/// coberto pela mesma varredura: uma forma **apagada** também sai dos `Children`.
///
/// # A lei
///
/// ⇒ **se um lado do par não é membro, o par colapsa num que seja** — preferindo o **destino**
/// (é o que a cena mostra), depois a origem, depois o primeiro estado. *Uma forma desenhada tem de
/// ser um estado; um estado que saiu não desenha.*
///
/// ⭐ **E a máquina viva é LARGADA junto**, em vez de corrigida: ela renasce **semeada pelo mundo**
/// ([`open`]) — que a varredura acabou de arrumar. Sem isso, o `tick` seguinte reescreveria a forma
/// que saiu, porque o `current` dela ainda a nomeia. *A cura da W11d é o que torna esta barata.*
///
/// ⛔⛔ **Esta metade sobreviveu a uma mutação** (2026-08-26): nenhum gate corria o `tick` DEPOIS da
/// varredura, então apagar o `machines.remove` deixava a suíte inteira verde — e o resquício
/// voltava no quadro seguinte, **só dentro do modo de pré-visualização**, que é onde o artista
/// acabou de estar (o ▶ liga-o). Hoje há
/// `the_ghost_does_not_come_back_on_the_next_tick`.
///
/// ⚠️ **Escrita DIRECTA, não pelo ledger:** isto não é pré-visualização — é a consequência
/// documental de um gesto do artista (o ⊘), e o `post_frame_undo` regista-a **junto** com ele.
///
/// Devolve quantos conjuntos foram arrumados (diagnóstico e gate).
pub(crate) fn reconcile(
    machines: &mut MorphMachines,
    sim: &mut SimWorld,
    map: &crate::vec_entities::VecEntityMap,
) -> usize {
    let hosts: Vec<u64> = sim
        .world_mut()
        .query::<(Entity, &VecMorphMachine)>()
        .iter(sim.world())
        .map(|(e, _)| e.to_bits())
        .collect();
    let mut fixed = 0;
    for bits in hosts {
        let e = Entity::from_bits(bits);
        let shapes = crate::morph_set::graph_of(sim, map, e).shapes();
        let Some(m) = sim.world().get::<VecMorph>(e) else {
            continue;
        };
        let (a, b) = (m.sources[0], m.sources[1]);
        if shapes.contains(&a) && shapes.contains(&b) {
            continue;
        }
        // ⚠️ **O que SOBREVIVE dos dois é o que fica** — e a ordem entre eles é **indiferente**,
        // medido: a guarda acima já saiu cedo quando os dois são membros, então no máximo **um**
        // deles passa este `find`. ⛔ A 1.ª redacção afirmava que o destino tinha precedência
        // *"porque é ele que a cena mostra"* — verdade sobre o produto, e **uma afirmação sobre
        // nada** aqui: trocar a ordem não muda uma única resposta, e a mutação que a trocou
        // sobreviveu à suíte inteira. *Uma afirmação que mutação nenhuma mata é uma afirmação
        // sobre nada.*
        //
        // ⇒ o que resta a dizer é o caso em que **nenhum** sobrevive: aí é o primeiro estado, que
        // é onde uma máquina nova nasceria de qualquer modo.
        let Some(keep) = [b, a]
            .into_iter()
            .find(|s| shapes.contains(s))
            .or_else(|| shapes.first().copied())
        else {
            continue; // conjunto sem estado nenhum: o `disconnect_row` dissolve na fronteira
        };
        if let Some(mut m) = sim.world_mut().get_mut::<VecMorph>(e) {
            m.sources = [keep, keep];
            m.t = 0.0;
        }
        machines.remove(&bits);
        fixed += 1;
    }
    fixed
}

/// ⭐⭐⭐ **QUEM MANDA NA FORMA QUANDO OS DOIS MOTORES ESTÃO LIGADOS** (plano 32 W11e).
///
/// Enio, 2026-08-26, 2.º report: *"Ao ligar o preview Default não segurou wide e está em tall. No
/// hover há uma transição tall - wide - tall. Ao sair de hover o mesmo acontece."*
///
/// # ⛔⛔ O mecanismo: dois motores a escrever o MESMO `VecMorph`, e a ordem por quadro não bastava
///
/// A W11c ordenou-os **dentro** do quadro (a transição de UI corre depois do [`tick`], logo ganha).
/// Mas ela só ganha nos instantes em que **fala** — e o `morph_steps` cala-se nas pontas, de
/// propósito. Em todo instante de REPOUSO, e no quadro da CHEGADA, quem escrevia era a máquina de
/// teclas, parada na forma onde o `▶ Play` a deixou. Daí o retrato exacto do report:
///
/// - o `ui_preview::enter` instala o `Default` (**wide**) e o `tick` do quadro seguinte repõe
///   **tall** ⇒ *"Default não segurou wide"*;
/// - o hover morfa `wide -> tall`, mas o quadro anterior mostrava tall ⇒ *"tall - wide - tall"*;
/// - a saída morfa `tall -> wide`, chega, o `apply_ui_steps` cala-se e o `tick` repõe **tall** ⇒
///   *"ao sair de hover o mesmo acontece"*.
///
/// # A lei
///
/// ⇒ **o sistema de States tem precedência, e a máquina de teclas LARGA enquanto ele age.**
/// `ui_state_live` é verdade com o modo de pré-visualização ligado **ou** com alguma transição no
/// ar — as duas situações em que a forma é função do estado de UI, não da tecla.
///
/// ⭐ **Largar é a resposta certa, e não «não escrever»:** o `!active` do [`tick`] **apaga** as
/// máquinas, e a seguinte nasce **semeada pelo mundo** ([`open`]) — ou seja no sítio onde os States
/// a deixaram. É por isso que sair da pré-visualização não dá salto nenhum. *A cura da W11d é o que
/// torna esta possível.*
///
/// ⚠️ **Uma função e não um `&&` no braço do despacho**: é a quinta vez nesta linha que uma lei
/// escrita dentro do laço de render fica fora do alcance de todo gate.
#[must_use]
pub(crate) fn drives(morph_preview: bool, ui_state_live: bool) -> bool {
    morph_preview && !ui_state_live
}

/// **A máquina deste hospedeiro, criando-a se preciso** — a porta única das duas metades.
///
/// ⚠️ **A semente é o que a CENA MOSTRA** (`VecMorph::sources[1]`), e não `graph.start()`: a
/// máquina morre ao sair do modo e o componente **fica**, então uma máquina nova que se julgasse na
/// primeira forma discordaria do desenho — e a regra *«chegar onde já se está não é chegar»*
/// recusaria justamente o voo que o artista pediu. Ver [`MorphMachine::seeded`].
fn open<'a>(
    machines: &'a mut MorphMachines,
    sim: &SimWorld,
    host: Entity,
    graph: &ph2d_morph_machine::MorphGraph,
) -> &'a mut MorphMachine {
    let showing = sim.world().get::<VecMorph>(host).map(|m| m.sources[1]);
    machines
        .entry(host.to_bits())
        .or_insert_with(|| MorphMachine::seeded(graph, showing))
}

#[cfg(test)]
#[path = "morph_machine_drive_tests.rs"]
mod tests;
