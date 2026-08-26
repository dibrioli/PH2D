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
//! Arma o **material**: três formas bem diferentes umas das outras, soltas. ⛔ **Não há Morph
//! nenhum, nenhuma máquina, nenhuma condição** — quem carrega no botão que faz o conjunto e quem
//! escolhe o que dispara cada transição é o artista, e é **exactamente essa** a costura que a wave
//! existe para provar. *Um smoke que arma o gesto por baixo do pano pula a costura que devia
//! testar.*
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
    // ⛔ **E PARA POR AQUI.** Nenhum Morph, nenhuma máquina, nenhuma condição: são elas que o
    // roteiro manda o artista construir, e é a construção que a wave existe para provar.
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
        "[morph-states-smoke] (!) se nao forem 3 formas e ZERO morph, PARE: a cena perdeu a \
         premissa e o resto do roteiro nao mede nada."
    );
    eprintln!(
        "[morph-states-smoke] as tres formas chamam-se **Wide** (azul), **Tall** (verde) e \
         **Thin** (amarela)."
    );
    eprintln!("[morph-states-smoke] o roteiro:");
    eprintln!(
        "  1. Pegue a ferramenta VECTOR e escolha as TRES formas: clique na azul e depois \
         Shift+clique na verde e na amarela."
    );
    eprintln!(
        "  2. No painel, abra a seccao **Morph States**. Ela diz «3 shapes, one key each» e \
         traz o botao **Make Morph States**. Carregue nele."
    );
    eprintln!(
        "     (!) Se a seccao nao aparecer com as tres escolhidas, PARE -- ela e' a unica porta \
         para esta feature."
    );
    eprintln!(
        "  3. O que tem de acontecer, tudo de uma vez: na arvore nasce **Morph States 3**, com as \
         tres formas por baixo dele; **no canvas fica UMA forma so'** (a azul, que era a primeira \
         escolhida) **no MEIO de onde as tres estavam**; e o painel lista **TRES** linhas, uma \
         por forma -- nao seis, nao doze."
    );
    eprintln!(
        "     (!) Se continuar a ver as tres formas espalhadas, PARE: as outras duas deviam ter \
         ficado ocultas, e as tres deviam ter-se juntado no mesmo ponto -- e' isso que faz a \
         transicao acontecer EM LUGAR, sem a peca atravessar o ecra."
    );
    eprintln!(
        "  4. **Ctrl+Z** agora. As tres formas voltam, soltas, como estavam -- **num passo so'**. \
         Se precisar de varios Ctrl+Z, PARE. Depois **Ctrl+Shift+Z** para refazer e continuar."
    );
    eprintln!(
        "  5. Cada linha e' uma FORMA. Carregue no botao da coluna **Key** (ele mostra um tracinho \
         quando nao ha' tecla): abre a lista dos eventos. Ponha **jump** na linha **Tall** e \
         **dash** na linha **Thin**. Deixe a **Wide** no tracinho."
    );
    eprintln!(
        "     (!) O menu so' oferece accoes que existem no projecto -- e' de proposito, e e' por \
         isso que nao da' para escrever um nome que nao existe."
    );
    eprintln!(
        "  6. No TOPO da seccao ha' o botao **Preview**. Carregue nele: ele acende, e por baixo \
         aparece «Preview on -- the keyboard drives the machine. Esc exits.»"
    );
    eprintln!(
        "  7. Agora carregue em **{jump}**: vira a **Tall**. Carregue em **{dash}**: vira a \
         **Thin**. E **{jump}** outra vez: volta a' Tall -- a MESMA tecla leva sempre a' MESMA \
         forma, venha-se de onde se vier. E' esta a mudanca."
    );
    eprintln!(
        "     (!) SEGURE o **{jump}** e carregue no **{dash}**: tem de ir para a Thin e FICAR la'. \
         Se voltar sozinha para a Tall, PARE -- a tecla segurada esta' a pinar a maquina."
    );
    eprintln!(
        "  8. O CONTROLE QUE IMPORTA: com o **Preview** ligado, carregue nas **setas do teclado**. \
         Nada pode mexer-se de sitio. (Era isto que estava errado: as setas morfavam a forma E \
         moviam as formas ao mesmo tempo.)"
    );
    eprintln!(
        "  9. **Esc** (ou o botao **Preview** outra vez) para sair. A forma FICA na que voce' \
         deixou -- sair COMPROMETE --, e um **Ctrl+Z** so' desfaz isso: UM passo para a sessao \
         inteira de transicoes, nunca um por troca."
    );
    eprintln!(
        " 10. Com o Preview DESLIGADO, carregue em **{jump}** outra vez. A forma nao pode mexer-se \
         -- fora do modo o teclado e' do editor."
    );
    eprintln!(
        " 11. **ARRASTE o objecto** «Morph States 3» pelo canvas, como qualquer forma. Ele anda, e \
         os estados vao junto: ligue o Preview outra vez e troque de forma -- a nova aparece no \
         sitio NOVO, nao no antigo."
    );
    eprintln!(
        " 12. Com o Preview ligado e estando na **Tall**, carregue em **{jump}** (que e' a tecla \
         da Tall). Nada pode mexer-se -- chegar onde ja' se esta' nao e' chegar."
    );
    eprintln!(
        " 13. **O BOTAO ▶ de cada linha, com o Preview DESLIGADO** (desligue-o antes): carregue no \
         da **Thin**. Ele liga o modo **e** a forma viaja, ao PRIMEIRO clique. (!) Se precisar de \
         carregar duas vezes, PARE -- era esse o defeito de 26/08. Depois carregue no ▶ da **Wide**: \
         ela tem de voltar, mesmo tendo sido a forma inicial."
    );
    eprintln!(
        " 14. **ARRASTE uma forma NOVA para dentro do «Morph States 3» na Hierarquia.** Ela passa \
         a fazer parte do sistema **sozinha**: aparece na lista, some do canvas, e ja' da' para \
         lhe dar uma tecla. Arraste-a para FORA e ela sai -- e volta a aparecer."
    );
    eprintln!(
        "     (!) Arraste-a para dentro OUTRA vez: a tecla que voce' tinha dado tem de VOLTAR com \
         ela. Se vier em branco, PARE -- desconectar nao pode destruir o que voce' escreveu."
    );
    eprintln!(
        " 15. **O botao ⊘ de uma linha** (Desconectar): a forma sai do conjunto e volta a \
         aparecer no canvas, solta. (!) Se ela sumir em vez de aparecer, PARE. (!) Depois volte ao \
         passo 19 e passe o rato: a forma que voce' soltou nao pode SALTAR para o meio do \
         conjunto -- ela ja' nao faz parte da animacao."
    );
    eprintln!(
        " 16. **Undo Morph States**, no fim da seccao: desfaz tudo. As tres formas voltam soltas e \
         visiveis, onde estavam, e o objecto some da arvore."
    );
    eprintln!(
        " 17. **O MORPH DENTRO DE UMA ANIMACAO DE STATES** (refaca o conjunto se desfez): com o \
         **Morph States 3** escolhido, abra a seccao **States** (a das poses) e carregue em **Rec** \
         no papel **Default**."
    );
    eprintln!(
        " 18. Agora use o **▶** de outra linha para por o conjunto NOUTRA forma, e carregue em \
         **Rec** no papel **Hover**. Os dois papeis passam a guardar formas DIFERENTES. (!) Nao \
         precisa de atribuir tecla nenhuma para isto -- se precisar, PARE."
    );
    eprintln!(
        " 19. Na seccao States, ligue o **Preview** (o das poses, nao o do Morph) e passe o rato \
         por cima da forma: ela **MORFA** de uma para a outra, animada, e volta ao sair. (!) Nao \
         precisa de desligar o Preview do Morph antes -- enquanto os States mandam, a maquina de \
         teclas LARGA. Se a forma piscar (tall-wide-tall) ou nao segurar a do Default, PARE."
    );
    eprintln!(
        "     (!) Se ela SALTAR em vez de morfar, PARE. (!) Se nao mexer, PARE -- e' a \
         compatibilidade que nao esta' a chegar."
    );
}
