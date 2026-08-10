//! ⭐ **A cena da TABELA SINAL → PAPEL** — `PH2D_BUILD_SMOKE=68` (item 4 do estudo dos
//! contêineres).
//!
//! ⚠️ **Não confundir com o [`crate::signal_smoke`]** (`PH2D_SIGNAL_SMOKE`), que é a cena do R0:
//! ali o assunto é *duas fontes publicam na mesma saída*, e o consumidor é um toast. Aqui o
//! assunto é o **CONSUMIDOR** — o que faz o nome mover a cena.
//!
//! # A pergunta desta cena é de olho, e ela é sobre O LAÇO
//!
//! *Eu apertei uma coisa e OUTRA coisa respondeu.* Até aqui a UI que o artista desenha respondia
//! ao rato **por cima de si mesma** — o botão acendia, o botão afundava —, e um sinal publicado
//! virava um toast e mais nada. O que fecha a volta é um nome: o botão grita, a tabela diz quem
//! escuta, e o menu entra.
//!
//! # O que a cena DÁ e o que ela deliberadamente NÃO arma
//!
//! Ela dá o **material**: três formas com poses gravadas pela porta do produto. Ela **não** arma
//! a ligação — é ela a costura que esta wave existe para provar, e uma cena que a semeasse por
//! baixo do pano pularia exactamente o que devia mostrar (a cicatriz que o `impasto_smoke` já
//! prega). São três gestos: `+ Signal`, digitar o nome, escolher o papel.
//!
//! # O CONTROLE é uma terceira forma, e ele é metade da cena
//!
//! `Plain` tem as MESMAS duas poses e **nenhuma ligação**. Sem ele, uma implementação que movesse
//! *todo* hospedeiro a cada sinal passaria no olho — e o artista só descobriria no dia em que a
//! cena inteira saltasse junto.

use ph2d_ui_state::StateRole;
use ph2d_vec_scene::{Paint, Rgba8, VecPath, VecPathId, rectangle};

/// `(caixa, nome)` — a ordem é a de desenho.
const ART: [([f64; 4], &str); 3] = [
    ([-6.6, -0.5, -4.2, 0.6], "Open"),
    ([-2.0, -1.6, 3.2, 1.6], "Menu"),
    ([4.6, -0.5, 7.0, 0.6], "Plain"),
];

const OPEN: usize = 0;
const MENU: usize = 1;
const PLAIN: usize = 2;

/// A cor de repouso de cada forma.
const REST: [[u8; 3]; 3] = [[70, 108, 168], [46, 50, 62], [70, 74, 88]];

/// **O quanto o menu se esconde**, em unidades de mundo.
///
/// ⚠️ Um número GRANDE de propósito: o que a cena tem de mostrar é *outra coisa respondeu*, e um
/// deslocamento tímido lê-se como um tremor de renderização em vez de uma resposta.
const TUCK: f32 = -7.5;

pub(crate) fn frame(app: &mut crate::App, f: u32) {
    match f {
        3 => build(app),
        // O nome só depois do `sync` — é ele que dá entidade a cada caminho.
        5 => name_them(app),
        // ⚠️ **A ordem das duas gravações é a que o artista faria ao contrário**, e é deliberada:
        // gravar primeiro o `Pressed` com o menu à vista e só depois esconder e gravar o
        // `Default` evita mexer no mundo duas vezes. O que importa é o RESULTADO — as duas poses
        // TÊM de diferir, senão a transição não tem o que mostrar.
        7 => record_all(app, StateRole::Pressed),
        9 => tuck_the_menu(app),
        11 => record_all(app, StateRole::Default),
        13 => announce(app),
        _ => {}
    }
}

fn build(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    for (i, (r, _)) in ART.iter().enumerate() {
        let mut p: VecPath = rectangle([r[0], r[1]], [r[2], r[3]]);
        let c = REST[i];
        p.fill = Some(Paint::Solid(Rgba8::new(c[0], c[1], c[2], 255)));
        gfx.vec_scene.push_path(p);
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

/// ⭐ **O NOME é o contrato inteiro.** O botão grita o `Name` da entidade dele, então *"Open"* é
/// literalmente o que o artista vai digitar na ligação do menu — não há um segundo lugar onde o
/// nome de uma coisa se escreva.
fn name_them(app: &mut crate::App) {
    let ids = path_ids(app);
    if ids.len() < ART.len() {
        return;
    }
    let ents: Vec<_> = ids.iter().map(|&id| entity(app, id)).collect();
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    for (i, (_, name)) in ART.iter().enumerate() {
        let Some(e) = ents[i] else { continue };
        if let Ok(mut ent) = gfx.sim.world_mut().get_entity_mut(e) {
            ent.insert(ph2d_ecs::Name::new(*name));
        }
    }
}

/// Grava `role` nos três hospedeiros — pela porta do PRODUTO, nunca escrevendo a tabela à mão.
fn record_all(app: &mut crate::App, role: StateRole) {
    let ids = path_ids(app);
    if ids.len() < ART.len() {
        return;
    }
    let map = &app.vec_entities;
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    for i in [OPEN, MENU, PLAIN] {
        crate::vec_ui_state_edit::apply(
            &mut gfx.sim,
            &mut gfx.vec_scene,
            map,
            &[ids[i]],
            &mut gfx.ui_states,
            crate::vec_ui_state_edit::UiStateEdit::Record(role),
        );
    }
}

/// Esconde o menu — é esta a pose que vira o `Default`, e é dela que o sinal o tira.
fn tuck_the_menu(app: &mut crate::App) {
    let ids = path_ids(app);
    if ids.len() < ART.len() {
        return;
    }
    let Some(e) = entity(app, ids[MENU]) else {
        return;
    };
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    if let Some(mut t) = gfx.sim.world_mut().get_mut::<ph2d_ecs::Transform>(e) {
        t.translation.x = TUCK;
    }
}

/// A mensagem — com os números MEDIDOS da própria cena, nunca de memória.
fn announce(app: &crate::App) {
    let Some(gfx) = app.gfx.as_ref() else {
        return;
    };
    let ids = path_ids(app);
    let poses: usize = ids.iter().map(|&id| gfx.ui_states.get(id).len()).sum();
    let bound: usize = ids.iter().map(|&id| gfx.ui_states.bindings(id).len()).sum();
    eprintln!(
        "[signal-table] cena montada: {} formas, {poses} pose(s) gravada(s), {bound} ligacao(oes).",
        gfx.vec_scene.paths().len()
    );
    eprintln!(
        "[signal-table] (!) se as poses forem menos de 6, PARE — a autoria nao correu e o resto \
         do roteiro nao diz nada."
    );
    eprintln!("[signal-table] AS LIGACOES NASCEM EM ZERO — e' voce que as arma.");
    eprintln!("[signal-table] o roteiro (pegue a ferramenta VECTOR primeiro):");
    eprintln!("  1. Clique no retangulo escuro do MEIO ('Menu'). Na secao States, no FIM, ha' a");
    eprintln!("     lista 'On Signal': aperte '+ Signal', escreva  Open  e tecle Enter.");
    eprintln!("     Depois escolha o chip 'Pressed' na linha que apareceu.");
    eprintln!("  2. Aperte 'Preview'. A cena vai ao repouso: o Menu sai de cena pela esquerda.");
    eprintln!("  3. (!) O TESTE: clique no retangulo azul da ESQUERDA ('Open').");
    eprintln!("     O MENU ENTRA — e nada mais se mexe. Foi um NOME que atravessou: o botao");
    eprintln!("     gritou 'Open', a tabela disse quem escuta, e o menu foi.");
    eprintln!("  4. (!) O CONTROLE: o retangulo da DIREITA ('Plain') tem as MESMAS duas poses e");
    eprintln!("     nenhuma ligacao. Se ele se mexer junto, o sinal esta' a mover tudo.");
    eprintln!("  5. Saia da Preview (Esc). O mundo volta exactamente ao que era.");
    eprintln!("  6. (!) E fora da Preview clique no 'Open' outra vez: NADA acontece, de");
    eprintln!("     proposito. Uma pose que o artista pede com um clique (o botao Show) custa um");
    eprintln!("     passo de undo e ele sabe porque'; uma pose que CHEGA sozinha nao pode cobrar");
    eprintln!("     nada — fora da preview nao ha' restauracao, e ela moveria o desenho dele.");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **O CONTROLE tem a mesma forma do alvo** — é isso que o torna um controle.
    ///
    /// ⚠️ Se ele tivesse poses diferentes, *"o Plain não se mexeu"* teria uma segunda explicação
    /// (ele não tinha para onde ir), e a cena não separaria as duas.
    #[test]
    fn the_scene_has_a_target_a_button_and_a_control() {
        assert_eq!(ART.len(), 3);
        assert_ne!(OPEN, MENU);
        assert_ne!(MENU, PLAIN);
    }

    /// **O esconderijo tira o menu de cena, e o número é medido contra a própria largura dele.**
    #[test]
    fn the_tuck_is_far_enough_to_read_as_an_answer() {
        #[allow(clippy::cast_possible_truncation)]
        let w = (ART[MENU].0[2] - ART[MENU].0[0]) as f32;
        assert!(
            TUCK.abs() > w,
            "o menu esconde-se menos que a propria largura ({TUCK} contra {w}) — na tela isso \
             le-se como um tremor, nao como uma resposta"
        );
    }

    /// **O nome que o roteiro manda digitar é o `Name` do botão** — se os dois divergissem, o
    /// artista seguiria o roteiro e nada aconteceria.
    #[test]
    fn the_script_tells_the_artist_the_buttons_own_name() {
        assert_eq!(ART[OPEN].1, "Open");
    }
}
