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
