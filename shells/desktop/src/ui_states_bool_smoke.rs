//! **A BOOLEANA VIVA DENTRO DE UM ESTADO DE UI** — `PH2D_BUILD_SMOKE=74` (Enio, 2026-08-23).
//!
//! # A pergunta desta cena é de olho, e ela é sobre o MEIO de uma troca de OPERAÇÃO
//!
//! *Eu autorei a mesma peça com DUAS operações booleanas diferentes, e o motor descobriu o caminho
//! entre elas — o buraco não pisca, ele nasce de um ponto e cresce; e a peça continua a seguir o
//! movimento das formas enquanto isso acontece.*
//!
//! ⚠️ **Nenhuma das quatro referências faz isto.** Blender, After Effects e Rive interpolam um
//! enum em modo CONSTANTE — o valor salta; o Figma, quando não sabe casar duas formas, faz
//! *crossfade* e mostra as duas ao mesmo tempo. Medido nesta linha: o salto move **64,0** de tinta
//! num quadro com a peça parada, contra **3,1** do morph.
//!
//! # A cena vem PRONTA, e a razão é o pedido
//!
//! O irmão [`crate::bool_smoke`] (`=48`) dá o material e deixa o artista armar tudo — porque o que
//! ele prova é o GESTO. Aqui o que se prova é a **animação**, e uma animação que exige quinze
//! cliques antes de aparecer não é smokável: a lei da casa é *feature nova = auto-play*. Então o
//! rig 1 nasce com as duas poses gravadas (pela porta do produto, nunca escrevendo a tabela à mão)
//! e o artista só liga a **preview** e passa o rato.
//!
//! O rig 2 é o **CONTROLE**, com o material idêntico e **sem pose nenhuma**: ele não se pode mexer
//! — e é ele que o artista usa para autorar a mesma coisa com as próprias mãos.

use ph2d_ui_state::StateRole;
use ph2d_vec_scene::{Paint, Rgba8, VecPath, VecPathId, rectangle};

/// O `x` de cada rig. Folgados o bastante para os dois caberem na foto sem se tocarem.
const RIG_X: [f64; 2] = [-3.4, 2.6];

/// O deslocamento e a escala que o operando de dentro ganha no HOVER.
///
/// ⚠️ **Ele MOVE-SE de propósito** (Enio, 2026-08-23: *"as formas além de mudar o modo do boolean
/// também podem estar animadas em pos, scl e rot"*): sem isso a cena provaria a troca de verbo
/// sobre uma peça parada, que é o caso fácil — e o difícil é o único que decide.
const HOVER_SHIFT: f32 = 0.34;
const HOVER_SCALE: f32 = 1.3;

fn tint(mut p: VecPath, rgb: [u8; 3]) -> VecPath {
    p.fill = Some(Paint::Solid(Rgba8::new(rgb[0], rgb[1], rgb[2], 255)));
    p
}

pub(crate) fn frame(app: &mut crate::App, f: u32) {
    match f {
        3 => build(app),
        // ⚠️ Num frame POSTERIOR ao `build`: a entidade de uma forma nasce no
        // `vec_entities::sync`, que corre no frame do desenho. Armar antes seria escrever num
        // objeto que ainda não existe.
        5 => name_and_arm(app),
        // ⭐ **O HOVER é autorado ANTES do Default**, e a ordem é o que deixa a cena em REPOUSO no
        // fim. Gravar o Default primeiro obrigaria a pedir um *Show* de volta — e um `go_to` para
        // a pose que a máquina já julga viva é um no-op, então a cena abriria no Hover sem que
        // nada dissesse porquê.
        7 => pose_hover(app),
        9 => record(app, StateRole::Hover),
        11 => pose_default(app),
        13 => record(app, StateRole::Default),
        15 => announce(app),
        _ => {}
    }
}

/// Cada rig: o CHIP (o fundo, que é o hospedeiro), o de FORA e o de DENTRO.
///
/// ⚠️ **O chip é maior que o de fora de propósito.** Ele é o hospedeiro dos estados, e um
/// hospedeiro é *"a forma ÚNICA selecionada"* — clicar num operando seleciona o grupo booleano
/// inteiro, e a seção States não é oferecida. A borda do chip que sobra é onde o artista clica
/// para o pegar sozinho.
fn build(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    for (i, x) in RIG_X.iter().enumerate() {
        let dim = if i == 0 { 0 } else { 40 };
        gfx.vec_scene.push_path(tint(
            rectangle([x - 1.7, 0.2], [x + 1.7, 2.6]),
            [58 + dim, 66 + dim, 92 + dim],
        ));
        gfx.vec_scene.push_path(tint(
            rectangle([x - 1.15, 0.7], [x + 1.15, 2.1]),
            [120 + dim, 170 + dim, 225],
        ));
        // ⚠️ INTEIRAMENTE dentro do de fora: é a única disposição em que `Union` e `Subtract`
        // diferem em TOPOLOGIA (1 contorno contra 2), e portanto a única que mostra um buraco a
        // nascer. Duas formas que apenas se cruzam dão um contorno nos quatro verbos.
        gfx.vec_scene.push_path(tint(
            rectangle([x - 0.5, 1.05], [x + 0.5, 1.75]),
            [235, 200 - dim, 120],
        ));
    }
}

fn path_ids(app: &crate::App) -> Vec<VecPathId> {
    app.gfx
        .as_ref()
        .map(|g| g.vec_scene.paths().iter().map(|p| p.id).collect())
        .unwrap_or_default()
}

fn entity(app: &crate::App, id: VecPathId) -> Option<ph2d_ecs::Entity> {
    app.vec_entities
        .get(&id)
        .map(|&bits| ph2d_ecs::Entity::from_bits(bits))
}

/// Os índices das três formas do rig `r`.
const fn rig(r: usize) -> (usize, usize, usize) {
    (r * 3, r * 3 + 1, r * 3 + 2)
}

/// Dá nome às formas, arma a booleana de cada rig em `Union`, e pendura o grupo no CHIP.
///
/// ⚠️ **A booleana é armada pela porta do PRODUTO** (`bool_gesture::arm`), e o grupo é pendurado
/// pela porta do reparent que a Hierarquia usa: uma cena que escrevesse os componentes à mão
/// pularia exactamente a costura que ela existe para provar.
fn name_and_arm(app: &mut crate::App) {
    let ids = path_ids(app);
    if ids.len() < 6 {
        return;
    }
    let ents: Vec<_> = ids.iter().map(|&id| entity(app, id)).collect();
    let map = app.vec_entities.clone();
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    for (i, name) in ["Ready", "Big", "Hole", "Control", "Big", "Hole"]
        .iter()
        .enumerate()
    {
        if let Some(e) = ents[i]
            && let Ok(mut ent) = gfx.sim.world_mut().get_entity_mut(e)
        {
            ent.insert(ph2d_ecs::Name::new(*name));
        }
    }
    for r in 0..2 {
        let (chip, outer, inner) = rig(r);
        let operands = [ids[outer], ids[inner]];
        if !crate::bool_gesture::arm(&mut gfx.sim, &gfx.vec_scene, &map, &operands, 0) {
            continue;
        }
        // O grupo vira FILHO do chip: é isso que põe os operandos na sub-árvore do hospedeiro, e
        // portanto dentro do que um estado dele grava.
        if let (Some(g), Some(c)) = (
            crate::bool_gesture::group_of_selection(&gfx.sim, &map, &operands),
            ents[chip],
        ) {
            crate::vec_transform::reparent_keeping_world(&mut gfx.sim, g, c);
        }
    }
}

/// Põe o rig 1 na pose de HOVER — exactamente o que o artista faria com a mão: trocar a operação
/// do grupo **e** mexer no operando de dentro.
fn pose_hover(app: &mut crate::App) {
    let ids = path_ids(app);
    if ids.len() < 6 {
        return;
    }
    let (_, outer, inner) = rig(0);
    let (ge, ie) = (
        entity(app, ids[outer]).and_then(|e| group_of(app, e)),
        entity(app, ids[inner]),
    );
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    if let Some(g) = ge {
        gfx.sim
            .world_mut()
            .entity_mut(g)
            .insert(ph2d_ecs::VecBoolGroup { op: 1 }); // Subtract: o buraco.
    }
    if let Some(e) = ie
        && let Some(mut t) = gfx.sim.world_mut().get_mut::<ph2d_ecs::Transform>(e)
    {
        t.translation.x += HOVER_SHIFT;
        t.scale.x = HOVER_SCALE;
        t.scale.y = HOVER_SCALE;
    }
}

/// Devolve o rig 1 ao repouso: o grupo em `Union` e o operando onde estava.
fn pose_default(app: &mut crate::App) {
    let ids = path_ids(app);
    if ids.len() < 6 {
        return;
    }
    let (_, outer, inner) = rig(0);
    let (ge, ie) = (
        entity(app, ids[outer]).and_then(|e| group_of(app, e)),
        entity(app, ids[inner]),
    );
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    if let Some(g) = ge {
        gfx.sim
            .world_mut()
            .entity_mut(g)
            .insert(ph2d_ecs::VecBoolGroup { op: 0 });
    }
    if let Some(e) = ie
        && let Some(mut t) = gfx.sim.world_mut().get_mut::<ph2d_ecs::Transform>(e)
    {
        t.translation.x -= HOVER_SHIFT;
        t.scale.x = 1.0;
        t.scale.y = 1.0;
    }
}

/// O grupo booleano acima de `e`, pela porta única.
fn group_of(app: &crate::App, e: ph2d_ecs::Entity) -> Option<ph2d_ecs::Entity> {
    let gfx = app.gfx.as_ref()?;
    let id = gfx.sim.world().get::<ph2d_ecs::VecPathRef>(e)?.0;
    crate::bool_live::group_above(&gfx.sim, &app.vec_entities, id).map(|(g, _)| g)
}

/// Grava a pose do rig 1 no papel `role`, pela porta do produto.
fn record(app: &mut crate::App, role: StateRole) {
    let ids = path_ids(app);
    if ids.len() < 6 {
        return;
    }
    let map = &app.vec_entities;
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    crate::vec_ui_state_edit::apply(
        &mut gfx.sim,
        &mut gfx.vec_scene,
        map,
        &[ids[rig(0).0]],
        &mut gfx.ui_states,
        crate::vec_ui_state_edit::UiStateEdit::Record(role),
    );
}

fn announce(app: &mut crate::App) {
    let ids = path_ids(app);
    let Some(gfx) = app.gfx.as_ref() else {
        return;
    };
    if ids.len() < 6 {
        eprintln!("[bool-states] ⚠️ a cena nao montou — PARE");
        return;
    }
    let host = ids[rig(0).0];
    let poses: usize = gfx.ui_states.get(host).len();
    // ⭐ **O NÚMERO QUE TORNA A CENA VÁLIDA**: quantas formas o motor vê a trocar de verbo entre as
    // duas poses. Se ele for ZERO, a autoria não gravou o canal e o resto do roteiro não diz nada
    // — o Hover trocaria a operação de uma vez, no fim, como o Blender faz.
    let changing = match (
        gfx.ui_states.role(host, StateRole::Default),
        gfx.ui_states.role(host, StateRole::Hover),
    ) {
        (Some(a), Some(b)) => ph2d_ui_state::Transition::new(&a.objects, &b.objects)
            .bool_morphs(0.5)
            .len(),
        _ => 0,
    };
    eprintln!(
        "[bool-states] {poses} poses gravadas no chip 'Ready'; {changing} forma(s) trocam de \
         operacao booleana entre elas."
    );
    if poses < 2 || changing == 0 {
        eprintln!(
            "[bool-states] ⚠️ **PARE**: eram para ser 2 poses e pelo menos 1 forma a trocar de \
             operacao. A autoria nao correu."
        );
        return;
    }
    eprintln!("[bool-states] o roteiro (pegue a ferramenta VECTOR primeiro):");
    eprintln!("  1. Clique na BORDA do chip da esquerda (o fundo escuro, fora do azul). ⚠️ Tem de");
    eprintln!("     acender so' ELE — clicar no azul pega o grupo inteiro, e a' seccao States");
    eprintln!("     precisa de uma forma so'.");
    eprintln!("  2. Na seccao **States**: Default e Hover ja' tem pose. Aperte **Show** no Hover.");
    eprintln!("     ⭐ **A PROVA**: o buraco NAO pisca. Ele nasce de um ponto no meio da peca e");
    eprintln!("     cresce — e ao mesmo tempo a forma que o abre desliza e aumenta. Aperte Show");
    eprintln!("     no Default para voltar: o buraco encolhe ate' desaparecer.");
    eprintln!("  3. Aperte **Preview** e passe o rato por cima do chip. A mesma coisa, agora sem");
    eprintln!("     clicar em nada. Saia com o rato e ele volta.");
    eprintln!("  4. ⚠️ O chip da DIREITA e' o CONTROLE: material identico, sem pose nenhuma.");
    eprintln!("     Ele NAO se pode mexer, com preview ligada ou desligada.");
    eprintln!("  5. Agora faca voce: com a preview DESLIGADA, clique na borda do chip da direita,");
    eprintln!("     **Rec** no Default. Depois clique no azul dele (o grupo acende), va' a'");
    eprintln!("     seccao Boolean e clique **Subtract**. Volte a clicar na borda do chip e");
    eprintln!("     **Rec** no Hover. Ligue a Preview e passe o rato: o da direita passa a fazer");
    eprintln!("     o mesmo que o da esquerda.");
    eprintln!("  6. ⚠️ Se em vez de crescer o buraco APARECER de uma vez, pare e diga: e' isso");
    eprintln!("     que as outras ferramentas fazem, e e' o que esta wave existe para nao fazer.");
}

#[cfg(test)]
#[path = "ui_states_bool_smoke_tests.rs"]
mod tests;
