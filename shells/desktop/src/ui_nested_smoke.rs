//! **A cena da HIERARQUIA** — `PH2D_BUILD_SMOKE=64` (plano UI/UX W7, a metade que faltava).
//!
//! # A pergunta desta cena é de olho, e é uma só
//!
//! *Passo o cursor pelo menu, ele abre; desço para um item, o item acende — **e o menu continua
//! aberto**.*
//!
//! Um menu com dois itens, e os **três** são hospedeiros: o menu tem o seu par
//! Default/Hover (fechado/aberto) e cada item tem o seu. Os itens são **filhos** do menu na
//! árvore, que é o que faz deles descendentes para o `host_under`.
//!
//! ⚠️ **O defeito que ela existe para mostrar era visível a olho:** o `point` mandava *o
//! hospedeiro anterior* para o `Default`, e um ANCESTRAL do novo alvo contava como anterior. O
//! menu fechava com o cursor dentro dele — e não havia como usar a feature.
//!
//! ⚠️ **E ela imprime o número que a torna válida:** quantos hospedeiros ela autorou e se o
//! aninhamento pegou. Se disser que o menu não governa os itens, PARE — a cena não contém o
//! fenômeno e o resto do roteiro não diz nada.

use ph2d_ui_state::StateRole;
use ph2d_vec_scene::{Paint, Rgba8, rectangle};

use crate::smoke_script::Step;

/// O menu (o fundo) e os dois itens, em unidades de mundo.
///
/// ⚠️ Os itens ficam **por cima** do fundo do menu, e é isso que faz o `host_under` receber os
/// dois no mesmo pick — a situação exacta em que a versão antiga escolhia pelo `VecPathId` menor
/// em vez de pelo mais interno.
const MENU_BOX: [f64; 4] = [-2.6, -1.9, 0.6, 1.9];
const ITEMS: [[f64; 4]; 2] = [[-2.4, 0.2, 0.4, 1.6], [-2.4, -1.6, 0.4, -0.2]];

/// Repouso e hover, na ordem: menu, item 0, item 1.
const REST: [[u8; 3]; 3] = [[38, 40, 48], [58, 62, 74], [58, 62, 74]];
const HOVER: [[u8; 3]; 3] = [[52, 56, 68], [92, 150, 220], [92, 150, 220]];

pub(crate) fn frame(app: &mut crate::App, f: u32) {
    match f {
        3 => build(app),
        // O `sync` do render loop é que dá entidade a cada caminho; sem entidade não há árvore
        // para pendurar, e sem árvore não há hierarquia (a razão do `text_fx_smoke`).
        4 => nest_and_record(app),
        5 => announce(app),
        _ => {}
    }
}

fn build(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("vector"));
    for (i, r) in std::iter::once(&MENU_BOX).chain(ITEMS.iter()).enumerate() {
        let id = gfx
            .vec_scene
            .push_path(rectangle([r[0], r[1]], [r[2], r[3]]));
        if let Some(p) = gfx.vec_scene.path_mut(id) {
            let c = REST[i];
            p.fill = Some(Paint::Solid(Rgba8::new(c[0], c[1], c[2], 255)));
        }
    }
}

/// Os ids desta cena, na ordem em que ela os empurrou.
fn ids(app: &crate::App) -> Vec<ph2d_vec_scene::VecPathId> {
    app.gfx
        .as_ref()
        .map(|g| g.vec_scene.paths().iter().map(|p| p.id).collect())
        .unwrap_or_default()
}

/// Pendura os itens no menu e grava os dois papéis dos três hospedeiros.
fn nest_and_record(app: &mut crate::App) {
    let ids = ids(app);
    if ids.len() < 3 {
        return;
    }
    // ⚠️ O aninhamento vai pela árvore ECS, que é onde `members` o lê.
    let Some(menu_e) = app.vec_entities.get(&ids[0]).copied() else {
        return;
    };
    for id in &ids[1..3] {
        let Some(bits) = app.vec_entities.get(id).copied() else {
            continue;
        };
        if let Some(gfx) = app.gfx.as_mut() {
            gfx.sim
                .world_mut()
                .entity_mut(ph2d_ecs::Entity::from_bits(bits))
                .remove::<ph2d_ecs::RootOrder>()
                .insert(ph2d_ecs::ChildOf(ph2d_ecs::Entity::from_bits(menu_e)));
        }
    }
    record(app, &ids, StateRole::Default);
    // A pose de HOVER: o menu clareia e cresce um pouco; o item acende.
    if let Some(gfx) = app.gfx.as_mut() {
        for (i, id) in ids.iter().take(3).enumerate() {
            if let Some(p) = gfx.vec_scene.path_mut(*id) {
                let c = HOVER[i];
                p.fill = Some(Paint::Solid(Rgba8::new(c[0], c[1], c[2], 255)));
            }
        }
    }
    record(app, &ids, StateRole::Hover);
    // Devolve a tinta de repouso — o artista abre a cena no Default.
    if let Some(gfx) = app.gfx.as_mut() {
        for (i, id) in ids.iter().take(3).enumerate() {
            if let Some(p) = gfx.vec_scene.path_mut(*id) {
                let c = REST[i];
                p.fill = Some(Paint::Solid(Rgba8::new(c[0], c[1], c[2], 255)));
            }
        }
    }
}

/// ⚠️ Pela porta do PRODUTO (`vec_ui_state_edit::apply`), e não escrevendo a tabela à mão: uma
/// cena que semeia estado por baixo pula exactamente a costura que ela existe para provar.
fn record(app: &mut crate::App, ids: &[ph2d_vec_scene::VecPathId], role: StateRole) {
    let map = &app.vec_entities;
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    for id in ids.iter().take(3) {
        crate::vec_ui_state_edit::apply(
            &mut gfx.sim,
            &mut gfx.vec_scene,
            map,
            &[*id],
            &mut gfx.ui_states,
            crate::vec_ui_state_edit::UiStateEdit::Record(role),
        );
    }
}

fn announce(app: &mut crate::App) {
    let ids = ids(app);
    let (hosts, governs) = app.gfx.as_ref().map_or((0, 0), |g| {
        let hosts = g.ui_states.hosts().count();
        let governs = ids.first().map_or(0, |m| {
            crate::vec_ui_state_edit::members(&g.sim, &g.vec_scene, &app.vec_entities, *m).len()
        });
        (hosts, governs)
    });
    eprintln!(
        "[nested] menu com 2 itens: {hosts} hospedeiro(s) autorado(s); o menu governa \
         {governs} caminho(s) (ele proprio + os itens)."
    );
    if hosts < 3 || governs < 3 {
        eprintln!(
            "[nested] !! a cena NAO contem o fenomeno (esperado: 3 hospedeiros, o menu a \
             governar 3 caminhos). PARE e reporte -- o resto do roteiro nao significa nada."
        );
    }
    crate::smoke_script::script("nested", "os três hospedeiros já estão gravados", STEPS);
}

const STEPS: &[Step] = &[
    Step {
        verb: "LIGUE A PREVIEW",
        lines: &[
            "Na seção UI States do painel, marque Preview.",
            "É o modo em que o rato dirige — e só nele.",
        ],
    },
    Step {
        verb: "⭐ O MENU NÃO FECHA",
        lines: &[
            "Passe o cursor pelo FUNDO do menu: ele clareia (abriu).",
            "Agora DESÇA para um dos itens, sem sair do menu.",
            "O item acende — e o menu TEM DE CONTINUAR CLARO.",
            "Se ele escurecer no instante em que você entra no item,",
            "é o defeito que esta cena existe para pegar: PARE.",
        ],
    },
    Step {
        verb: "O ITEM É QUEM RESPONDE AO CLIQUE",
        lines: &[
            "Aperte o botão sobre um item. Só ele vai para Pressed;",
            "o menu segura o Hover — o cursor está dentro dele, e",
            "acender o menu inteiro ao clicar num item seria errado.",
            "(Sem Pressed gravado, o item recua para o Default: é a lei",
            "dos papéis opcionais, não um defeito.)",
        ],
    },
    Step {
        verb: "SAIR APAGA OS DOIS",
        lines: &[
            "Leve o cursor para fora do menu inteiro.",
            "O item E o menu voltam ao repouso, juntos.",
        ],
    },
    Step {
        verb: "O CONTROLE — desligue a Preview",
        lines: &[
            "Desmarque Preview: a cena volta EXACTAMENTE ao que era,",
            "e o rato deixa de dirigir. Passar por cima não faz nada.",
        ],
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    /// **Os itens ficam por cima do fundo do menu** — sem sobreposição o pick nunca traz os dois,
    /// e a cena não conteria a situação que a wave resolve.
    #[test]
    fn the_items_sit_inside_the_menu_box() {
        for it in ITEMS {
            assert!(
                it[0] >= MENU_BOX[0] && it[1] >= MENU_BOX[1],
                "o item {it:?} sai da caixa do menu {MENU_BOX:?}"
            );
            assert!(it[2] <= MENU_BOX[2] && it[3] <= MENU_BOX[3]);
        }
    }

    /// **Repouso e hover são distinguíveis a olho** em cada um dos três.
    #[test]
    fn every_host_changes_visibly_between_the_two_roles() {
        for i in 0..3 {
            assert_ne!(
                REST[i], HOVER[i],
                "o hospedeiro {i} pinta a mesma cor nos dois papeis — o smoke nao mostraria nada"
            );
        }
    }

    /// **O roteiro cabe no terminal** — a mesma régua dos irmãos.
    #[test]
    fn the_script_fits_the_terminal() {
        crate::smoke_script::assert_fits("nested", STEPS);
    }
}
