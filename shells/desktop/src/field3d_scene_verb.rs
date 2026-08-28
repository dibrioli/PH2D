//! ⭐⭐⭐ **A FILEIRA DO VERBO e o SELO da Hierarquia** (W97) — as duas superfícies em que a receita
//! de uma peça se vê.
//!
//! ⚠️ **Elas respondem a perguntas DIFERENTES, e é de propósito:**
//!
//! | superfície | pergunta | mostra |
//! |---|---|---|
//! | o selo da linha | *«o que esta forma FAZ à peça?»* | o verbo **efectivo** — quem herda sela na mesma |
//! | a fileira do painel | *«quem escolheu?»* | o `Inherit` aceso quando ninguém se pronunciou |
//!
//! ⚠️ **Arquivo irmão por LOC** (HR-18): o `field3d_scene_panel.rs` passou dos 600 com esta wave.
//! ⛔ *Split, nunca allowlist* — e o corte é por assunto, que é o que torna o irmão legível sozinho.

use super::*;

/// ⭐⭐⭐ **O seletor do VERBO de uma forma** — e a posição `0` é a **herança**.
///
/// ⚠️ **Quatro e não três**: sem o `Inherit` não haveria gesto que devolvesse a forma ao padrão do
/// grupo, e escolher um verbo uma vez seria irreversível. *Um modo em que só se entra é um modo
/// errado.*
pub(crate) const VERBS: [&str; 4] = [
    "panel.model3d.verb.inherit",
    "panel.model3d.verb.add",
    "panel.model3d.verb.cut",
    "panel.model3d.verb.common",
];

/// A escolha que cada posição do seletor faz — `None` é *«volta a herdar»*.
///
/// ⭐ **A MISTURA em vigor viaja com o verbo**, e não é conforto: uma forma que herdava a subtração
/// de um grupo com filete `0,12` e passasse a subtrair com aresta **viva** mudaria de forma ao
/// clique, sem ninguém ter mexido num raio. É a mesma lei que o [`ph2d_field_ecs::set_op`] já
/// escreve para o grupo — *o raio é do nó, não da operação*.
pub(crate) fn verb_at(slot: usize, blend: Blend) -> Option<Option<Op>> {
    Some(match slot {
        0 => None,
        1 => Some(Op::Union(blend)),
        2 => Some(Op::Difference(blend)),
        3 => Some(Op::Intersection(blend)),
        _ => return None,
    })
}

/// A posição de um verbo no seletor — o inverso do [`verb_at`], para o `active`.
fn verb_slot(op: Op) -> usize {
    match op {
        Op::Union(_) => 1,
        Op::Difference(_) => 2,
        Op::Intersection(_) => 3,
    }
}

/// ⭐⭐⭐ **A fileira do verbo da forma escolhida**, e o NOME dela — vazio quando não há verbo a
/// escolher.
///
/// # ⚠️ Quem é o sujeito, e por que ele é NOMEADO
///
/// É o **primeiro** da seleção, e a fileira diz o nome dele. Tocar um filho pode acender o grupo
/// inteiro no canvas, e sem o nome o artista escolhe o verbo sem saber de qual das formas o painel
/// fala — foi exactamente esse o defeito que o vetorial pagou em 2026-08-22, e o report dele não
/// bastava para localizar a causa.
///
/// # ⚠️ A BASE não recebe fileira, e a raiz também não
///
/// A base **semeia** o acumulado: ela não dobra sobre nada, e o verbo dela não é perguntado por
/// ninguém ([`ph2d_field::fold_verb`]). Pintar quatro chips ali seria a affordance que mente — a
/// lei da W34 aplicada a esta fileira. Quem responde *«sou a base?»* é o
/// [`ph2d_field_ecs::verb_role`], que deriva a resposta da **mesma** função que o cozimento usa.
///
/// # ⚠️ Herdar acende o `Inherit`, e não o verbo herdado
///
/// O `active` diz *o que foi escolhido*, e o que foi escolhido é «seguir o grupo». Acender o verbo
/// herdado faria um clique nele parecer inerte e no entanto mudar o estado (de herdado para
/// próprio). *O que de facto acontece* lê-se no selo da Hierarquia, que mostra o verbo **efectivo**
/// — as duas superfícies respondem a duas perguntas, e é de propósito.
pub(crate) fn verbs_for(
    world: &bevy_ecs::world::World,
    selected: &[bevy_ecs::entity::Entity],
) -> (Vec<ph2d_panel_model3d::ModeChip>, Option<String>) {
    let nothing = (Vec::new(), None);
    let Some(&e) = selected.first() else {
        return nothing;
    };
    let active = match ph2d_field_ecs::verb_role(world, e) {
        // A base não dobra sobre nada — ver acima.
        None | Some(ph2d_field_ecs::VerbRole::Base) => return nothing,
        Some(ph2d_field_ecs::VerbRole::Inherited(_)) => 0,
        Some(ph2d_field_ecs::VerbRole::Own(op)) => verb_slot(op),
    };
    let chips = VERBS
        .iter()
        .enumerate()
        .map(|(i, key)| ph2d_panel_model3d::ModeChip {
            key,
            active: i == active,
        })
        .collect();
    let name = world
        .get::<ph2d_ecs::Name>(e)
        .map_or_else(|| "?".to_string(), |n| n.as_str().to_string());
    (chips, Some(name))
}

/// ⭐ **O selo da BASE** — a forma que semeia o acumulado.
///
/// ⚠️ **É o ÚNICO verbo que cede ao `LNK`**, e a regra que o decide vale para a fileira toda: *o selo
/// diz o que a linha não consegue dizer sozinha*. `BSE` é **derivável da posição** (é a primeira
/// linha que conta); `SUB`/`INT`/`UNI` não são deriváveis de nada, e o `LNK` também não.
///
/// ⛔ E é por isso que o `ISO` continua a ganhar aos dois: ele é a única que explica **uma ausência**
/// (por que todo o resto desapareceu).
pub(crate) const BASE_BADGE: &str = "BSE";

/// ⭐⭐ **O selo do VERBO na linha da Hierarquia** — `None` para quem não participa de receita.
///
/// ⚠️ **É a metade que faz o desenho funcionar** (a lição do vetorial, 2026-08-22): com o verbo só
/// no painel lateral, entender uma peça de cinco formas custa cinco cliques e memória. A Hierarquia
/// já mostra a **ordem**; o selo acrescenta o **verbo**; *ordem + verbo **são** a receita.*
///
/// ⚠️ **Ele mostra o verbo EFECTIVO** — quem herda sela `UNI` na mesma. A pergunta desta lista é
/// *«o que acontece?»*; *«quem escolheu?»* é a do painel, e as duas têm superfícies separadas.
///
/// ⚠️ **Os códigos são os do vetorial**, e a tabela de tons de `paint_hierarchy_row` já os conhece
/// (`SUB` avisa, `INT` acentua, `BSE` fica neutro) — foi ela que os estreou. Um código novo aqui
/// nasceria sem tom, e a consistência entre as duas metades do app é metade do valor desta wave.
pub(crate) fn verb_badge(
    world: &bevy_ecs::world::World,
    e: bevy_ecs::entity::Entity,
) -> Option<&'static str> {
    Some(match ph2d_field_ecs::verb_role(world, e)? {
        ph2d_field_ecs::VerbRole::Base => BASE_BADGE,
        ph2d_field_ecs::VerbRole::Inherited(op) | ph2d_field_ecs::VerbRole::Own(op) => match op {
            Op::Union(_) => "UNI",
            Op::Difference(_) => "SUB",
            Op::Intersection(_) => "INT",
        },
    })
}

// ───────── W99: o CARÁTER da mistura ─────────

/// ⭐⭐⭐ **A fileira do CARÁTER** — a forma da transição, ao lado do número que diz o tamanho.
///
/// ⚠️ **Ela é DERIVADA de [`ph2d_field::Character::ALL`]**, que é a fonte da contagem: um carácter
/// novo aparece na UI sem uma linha de mudança aqui. É a mesma lei do `Mode::ALL` e do
/// `ExportLevel::ALL`.
///
/// ⚠️ **Três chips e não quatro:** a aresta **viva** não é um carácter, é o **raio zero**, e o
/// slider já o exprime. Um quarto seria uma segunda porta para o mesmo facto, e as duas podiam
/// discordar.
///
/// ⚠️ **A pergunta é a MESMA que a do raio** ([`ph2d_field_ecs::character_of`]): a fileira aparece
/// onde há mistura — numa operação (o carácter do filete dela, que é o padrão dos filhos calados) e
/// numa forma que se junta ao resto. Vazio na base e na raiz, que não têm junta nenhuma.
pub(crate) fn characters_for(
    world: &bevy_ecs::world::World,
    selected: &[bevy_ecs::entity::Entity],
) -> Vec<ph2d_panel_model3d::ModeChip> {
    let Some(&e) = selected.first() else {
        return Vec::new();
    };
    let Some(actual) = ph2d_field_ecs::character_of(world, e) else {
        return Vec::new();
    };
    ph2d_field::Character::ALL
        .iter()
        .map(|c| ph2d_panel_model3d::ModeChip {
            key: character_key(*c),
            active: *c == actual,
        })
        .collect()
}

/// A chave i18n de cada carácter. ⚠️ **Um `match` exaustivo**, e é ele que fecha a corrente: um
/// carácter novo no documento é **erro de compilação** aqui, e não um chip sem rótulo.
pub(crate) fn character_key(c: ph2d_field::Character) -> &'static str {
    match c {
        ph2d_field::Character::Fillet => "panel.model3d.character.fillet",
        ph2d_field::Character::Chamfer => "panel.model3d.character.chamfer",
        ph2d_field::Character::Organic => "panel.model3d.character.organic",
    }
}
