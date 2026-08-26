//! **A cena da MÁQUINA DE ESTADOS DO MORPH** — `PH2D_BUILD_SMOKE=75` (plano 32 W6).
//!
//! ⚠️ **Ela nasceu em `=74` e mudou para `=75` no dia em que nasceu.** O `74` já era da
//! `ui_states_bool_smoke`, e o roteador é uma lista de `if`: **o primeiro vence**, então esta cena
//! teria ficado **inalcançável em silêncio**. Eu escolhi o número lendo o MAIOR `if level ==` do
//! ficheiro — e o dono do 74 está **acima** dele, na ordem do texto. *O número da próxima cena
//! CONTA-SE varrendo o ficheiro inteiro, nunca lendo o fim dele* — e quem o contou foi o gate
//! `no_two_smoke_scenes_claim_the_same_level`, que existe exactamente por isto ter acontecido
//! antes (a cena dos tokens, 2026-08-02).
//!
//! # O que a cena arma, e o que ela deixa para o artista
//!
//! Arma o **material**: três formas bem diferentes umas das outras, e um objecto de Morph entre as
//! duas primeiras. ⛔ **Nada nasce ligado** — nenhuma seta, nenhuma condição. Quem desenha as setas
//! e escolhe o que as dispara é o artista, e é **exactamente essa** a costura que a wave existe
//! para provar. *Um smoke que arma o gesto por baixo do pano pula a costura que devia testar.*
//!
//! ⚠️ **As acções são as de FÁBRICA** (`jump` no `Z`, `dash` no `Q`) — nenhuma acção nova é criada.
//! É deliberado: a lista que o menu da condição mostra é a do **projecto**, e usar a que já lá está
//! prova que o vínculo com o Input Map é real, em vez de o simular.
//!
//! # Três formas, e a diferença entre elas é o instrumento
//!
//! Um morph entre dois rectângulos parecidos é indistinguível de *nada a acontecer*. A cena dá um
//! **quadrado largo**, um **losango alto** e uma **barra fina** — se a transição correr, dá para
//! ver da outra ponta da sala; se ela não correr, também.

use ph2d_ecs::{Entity, Name};
use ph2d_vec_scene::{Paint, Rgba8, VecPath, VecPathId, rectangle};

/// Onde as três formas ficam, em mundo — separadas o suficiente para o arrasto de uma para a outra
/// não ser ambíguo, e juntas o suficiente para caberem no ecrã de abertura.
const GAP: f64 = 3.0;

const BLUE: [u8; 3] = [86, 132, 214];
const GREEN: [u8; 3] = [110, 190, 130];
const AMBER: [u8; 3] = [214, 150, 70];

fn tint(mut p: VecPath, rgb: [u8; 3]) -> VecPath {
    p.fill = Some(Paint::Solid(Rgba8::new(rgb[0], rgb[1], rgb[2], 255)));
    p
}

pub(crate) fn frame(app: &mut crate::App, f: u32) {
    match f {
        3 => build(app),
        8 => announce(app),
        _ => {}
    }
}

fn build(app: &mut crate::App) {
    // As TRÊS formas — larga, alta e fina. A diferença é o instrumento.
    let (a, b, c) = {
        let Some(gfx) = app.gfx.as_mut() else {
            return;
        };
        let a = gfx
            .vec_scene
            .push_path(tint(rectangle([-GAP - 1.2, -0.8], [-GAP + 1.2, 0.8]), BLUE));
        let b = gfx
            .vec_scene
            .push_path(tint(rectangle([-0.6, -1.6], [0.6, 1.6]), GREEN));
        let c = gfx.vec_scene.push_path(tint(
            rectangle([GAP - 1.4, -0.25], [GAP + 1.4, 0.25]),
            AMBER,
        ));
        (a, b, c)
    };
    name_shapes(app, [a, b, c]);

    // O MORPH entre as duas primeiras. ⚠️ A dança do `sync` é a do `morph_fade_smoke`: o morph
    // nasce vazio (a geometria é DERIVADA pelo recook) e só ganha entidade no `sync`.
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    crate::vec_entities::sync(&mut gfx.sim, &mut gfx.vec_scene, &mut app.vec_entities);
    let (id, morph) = crate::morph_live::create(&mut gfx.vec_scene, a, b);
    crate::vec_entities::sync(&mut gfx.sim, &mut gfx.vec_scene, &mut app.vec_entities);
    let attached = crate::morph_live::attach(&mut gfx.sim, &app.vec_entities, id, &morph);
    assert!(attached, "[morph-states-smoke] o morph nao pendurou");
    if let Some(&bits) = app.vec_entities.get(&id) {
        gfx.sim
            .world_mut()
            .entity_mut(Entity::from_bits(bits))
            .insert(Name::new("Morpher"));
    }
    // ⛔ **E PARA POR AQUI.** Nenhuma seta, nenhuma condição, nenhuma máquina: são elas que o
    // roteiro manda o artista construir.
}

/// Dá nome às três formas — o painel mostra o NOME de cada ponta da seta, e `#123` não diz nada.
fn name_shapes(app: &mut crate::App, ids: [VecPathId; 3]) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    crate::vec_entities::sync(&mut gfx.sim, &mut gfx.vec_scene, &mut app.vec_entities);
    for (id, n) in ids.into_iter().zip(["Wide", "Tall", "Thin"]) {
        if let Some(&bits) = app.vec_entities.get(&id) {
            gfx.sim
                .world_mut()
                .entity_mut(Entity::from_bits(bits))
                .insert(Name::new(n));
        }
    }
}

/// A mensagem — com os números MEDIDOS da cena viva e do mapa vivo, nunca de memória.
fn announce(app: &crate::App) {
    let Some(gfx) = app.gfx.as_ref() else {
        return;
    };
    let n = gfx.vec_scene.paths().len();
    // ⚠️ A contagem sai de uma QUERY, e não de um `iter_entities` — a shell não tem acesso mutável
    // aqui, e a query é a porta que o resto do frame já usa.
    let morphs = app
        .vec_entities
        .iter()
        .filter(|(_, bits)| {
            gfx.sim
                .world()
                .get::<ph2d_ecs::VecMorph>(Entity::from_bits(**bits))
                .is_some()
        })
        .count();
    // ⚠️ As acções saem do MAPA VIVO, e a tecla de cada uma sai da LIGAÇÃO viva — o roteiro nunca
    // nomeia uma tecla de memória. Se o artista já remapeou, o texto acompanha.
    let keys = |name: &str| -> String {
        gfx.hero_screen
            .as_ref()
            .and_then(|h| h.input_map.id(name).and_then(|i| h.input_map.get(i)))
            .map(|a| {
                a.bindings
                    .iter()
                    .map(|b| ph2d_editor::screens::hero::chrome::binding_label(*b).0)
                    .collect::<Vec<_>>()
                    .join(" ou ")
            })
            .unwrap_or_else(|| "(sem tecla)".to_string())
    };
    let jump = keys(ph2d_input::PLAYER_JUMP);
    let dash = keys(ph2d_input::PLAYER_DASH);

    eprintln!("[morph-states-smoke] cena montada: {n} formas ({morphs} morph).");
    eprintln!(
        "[morph-states-smoke] (!) se nao forem 4 formas e 1 morph, PARE: a cena perdeu a \
         premissa e o resto do roteiro nao mede nada."
    );
    eprintln!(
        "[morph-states-smoke] as tres formas chamam-se **Wide**, **Tall** e **Thin**; o objecto \
         entre as duas primeiras chama-se **Morpher**."
    );
    eprintln!("[morph-states-smoke] o roteiro:");
    eprintln!(
        "  1. Pegue a ferramenta VECTOR e clique no **Morpher** (a forma do meio, entre a azul e \
         a verde). O painel ganha a seccao **States**, a dizer que ainda nao ha' setas."
    );
    eprintln!(
        "  2. Na fileira de modos do painel, pegue o pill **States**. Agora arraste **de dentro \
         da forma AZUL para dentro da VERDE**: nasce uma seta curva, ambar, com ponta."
    );
    eprintln!(
        "     (!) Se nada aparecer, PARE. A seta e' desenhada no canvas, entre as duas formas."
    );
    eprintln!(
        "  3. Arraste tambem **da VERDE para a AMARELA**, e depois **da AMARELA para a AZUL**. \
         Sao tres setas, e a lista do painel tem tres linhas."
    );
    eprintln!(
        "     (!) Tente arrastar da AZUL para a AZUL: nao pode nascer nada -- uma forma virar ela \
         mesma nao e' uma transicao."
    );
    eprintln!(
        "  4. Em cada linha do painel ha' um menu **When**. Ponha **jump** nas tres. (O menu so' \
         oferece accoes que existem no projecto -- e' de proposito.)"
    );
    eprintln!(
        "  5. Aperte **Play** na barra de transporte. Agora carregue em **{jump}**: a forma vira \
         a seguinte. Carregue outra vez: vira a terceira. E outra vez: volta a' primeira."
    );
    eprintln!(
        "     (!) SEGURE a tecla em vez de a bater: ela tem de disparar **uma** vez, nao percorrer \
         a cadeia inteira num piscar de olhos."
    );
    eprintln!(
        "  6. **PARE o transporte.** A forma volta a ser a que voce' desenhou -- e o **Ctrl+Z** \
         NAO tem nenhum passo das transicoes para desfazer. Se tiver, PARE: o que o motor mostra \
         e' pre-visualizacao, nunca documento."
    );
    eprintln!(
        "  7. O CONTROLE: com o transporte PARADO, carregue em **{jump}** outra vez. A forma nao \
         pode mexer-se. (Se mexer, a maquina esta' a escutar durante a edicao -- e ai' toda tecla \
         faz duas coisas.)"
    );
    eprintln!(
        "  8. Volte ao painel e ponha **dash** ({dash}) numa das setas. Com o Play ligado, as duas \
         teclas passam a levar a sitios diferentes a partir da mesma forma."
    );
    eprintln!(
        "  9. Apague uma seta pela lixeira da linha. Ela some do canvas no mesmo quadro -- a lista \
         e o desenho leem o MESMO grafo."
    );
}
