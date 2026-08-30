//! **§14 Platform Player — o `populate` dele.**
//!
//! Irmão do [`super::populate_physics`] pelo cap de 600 LOC do arquivo, e o corte é por
//! RESPONSABILIDADE: lá ficam o corpo rígido (§11), a junta (§12) e a roda (§13) — todos
//! propriedades do SOLVER; aqui fica o player de plataforma, que é uma LEI PURA
//! (`ph2d-platformer`) montada sobre eles. As duas metades registam pelo mesmo
//! `register_button_ids`, então nenhuma regra se duplica no corte.
//!
//! ⚠️ **Registrar aqui não é opcional:** um id que o painel PINTA e o `populate` não regista
//! nasce hit-registrado e **morto sob o mouse** — o `architecture_panel_wiring_parity` é quem
//! cobra, e as células/chips que registam em LAÇO são o ponto cego dele (por isso os arrays são
//! `const`).
//!
//! ⚠️ **Ao mover isto para cá (2026-08-30) corrigiu-se um doc-comment com o DONO errado:** o
//! cabeçalho *«§14 Platform Player (W5) — os três botões e os oito números»* estava colado ao
//! `populate_player_chips`, que não regista número nenhum. Ele descreve o ASSUNTO, e o assunto
//! agora é o módulo. *Um doc separado do item que descreve muda de dono em silêncio.*

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore, format_number};
use ph2d_editor_core::widget::TextInputState;

use crate::populate::register_button_ids;

/// **Os CHIPS da §14** — os grupos segmentados e as dicas deles.
///
/// ⚠️ **Irmão de [`populate_player`] por TETO DE LOC, cortado por
/// RESPONSABILIDADE:** aqui moram os controles que ESCOLHEM entre opções
/// nomeadas, lá os que carregam um NÚMERO. As duas metades registam pelo mesmo
/// `register_button_ids`, então nenhuma regra se duplica no corte.
fn populate_player_chips(store: &mut WidgetStore) {
    // ⚠️ **O grupo do chip, registrado como os quinze da §11** — sem isto os dois
    // botões nascem pintados, no hit-index, e MORTOS sob o mouse (a cicatriz das
    // 36 células do W2c).
    register_button_ids(store, &ids::INSP_PLAYER_MODE_IDS);
    store.set_tooltip(
        ids::INSP_PLAYER_MODE_IDS[0],
        "The floating capsule: impulses, a spring leg, the solver owns the pose.",
    );
    store.set_tooltip(
        ids::INSP_PLAYER_MODE_IDS[1],
        "The controller: the pose is written, the world only says how much fit.",
    );
    store.set_tooltip(
        ids::INSP_PLAYER_MODE_IDS[2],
        "Classic platformer: the same controller, but the physical world is \
         scenery. Everything stops him and he moves nothing.",
    );
    // O mesmo, para o chip da saída de sinais (A3).
    register_button_ids(store, &ids::INSP_PLAYER_EMIT_IDS);
    store.set_tooltip(
        ids::INSP_PLAYER_EMIT_IDS[0],
        "Nobody hears him: he lands and jumps in silence.",
    );
    store.set_tooltip(
        ids::INSP_PLAYER_EMIT_IDS[1],
        "Publish what he does as signals (player.landed, player.jumped.wall, ...).",
    );
    // O mesmo, para o chip da política de plataforma (`W-Leave`).
    register_button_ids(store, &ids::INSP_PLAYER_LIFT_POLICY_IDS);
    store.set_tooltip(
        ids::INSP_PLAYER_LIFT_POLICY_IDS[0],
        "Jump height is measured against the PLATFORM: a rising lift launches \
         him higher, a descending one almost cancels the jump.",
    );
    store.set_tooltip(
        ids::INSP_PLAYER_LIFT_POLICY_IDS[1],
        "A rising platform still launches him; a descending one stops stealing \
         the jump. The authored height is delivered in the world.",
    );
    store.set_tooltip(
        ids::INSP_PLAYER_LIFT_POLICY_IDS[2],
        "The platform never changes the jump: the authored height is always \
         measured against the world.",
    );
    // O mesmo, para os dois chips da trava de beirada (`W-Brink`).
    register_button_ids(store, &ids::INSP_PLAYER_WALK_OFF_IDS);
    store.set_tooltip(
        ids::INSP_PLAYER_WALK_OFF_IDS[0],
        "He walks off ledges, like every character before this option existed.",
    );
    store.set_tooltip(
        ids::INSP_PLAYER_WALK_OFF_IDS[1],
        "He stops at the edge instead of walking off it. Jumping off still \
         works, and so does being carried off by a platform or a belt. A gap \
         wider than his leg can span reads as a ledge, so he stops there too.",
    );
    register_button_ids(store, &ids::INSP_PLAYER_CROUCH_WALK_OFF_IDS);
    store.set_tooltip(
        ids::INSP_PLAYER_CROUCH_WALK_OFF_IDS[0],
        "Crouching does not change it: he walks off ledges if standing does.",
    );
    store.set_tooltip(
        ids::INSP_PLAYER_CROUCH_WALK_OFF_IDS[1],
        "Crouched, he stops at the edge -- the sneak-to-the-brink move. It only \
         tightens: it cannot give back what standing already refuses.",
    );
}

pub(super) fn populate_player(store: &mut WidgetStore) {
    populate_player_chips(store);
    // ⛔⛔ **O `INSP_PLAYER_ADD` saiu desta lista em 2026-08-30 — era um REGISTO ÓRFÃO.**
    //
    // Ele era o botão «Make Platform Player» da face vazia, e a face **morreu na F3**
    // (ADR-0166: quem anexa passou a ser o `+` do cabeçalho do Inspector). O `sections/player.rs`
    // já dizia *«eram cinco até a F3»* na tabela de dicas — e o registo ficou para trás, sozinho:
    // pintado por ninguém, despachado por ninguém, e a ocupar espaço no `WidgetStore` de toda
    // sessão.
    //
    // ⚠️ **Um id registado sem pintura nem consumo não é inofensivo:** ele faz toda sonda futura de
    // controlos mortos mentir — a régua vê «registado» e conta-o como controlo, e a acusação que
    // ela produz aponta para um botão que já não existe. Gate:
    // `tests/seam_player.rs::the_dead_empty_face_leaves_no_registration_behind`.
    register_button_ids(
        store,
        &[
            ids::INSP_PLAYER_REMOVE,
            ids::INSP_PLAYER_FIT,
            ids::INSP_PLAYER_CLEAR_RUN,
            ids::INSP_PLAYER_RESTORE_RUN,
            ids::INSP_PLAYER_FIT_CROUCH,
        ],
    );
    // ⚠️ **As DICAS saem da MESMA tabela que o pintor lê** (W9, Enio: *"precisamos
    // de dicas no mouse hover"*). O `set_tooltip` é a infra que o app já tem — o
    // hover publica o `hot_id`, e o passe de tooltip do `hero` procura o texto
    // dele —, então uma seção que não registra nada é uma seção sem dica, em
    // silêncio. Registrar aqui, num laço sobre a tabela, é o que faz um controle
    // novo nascer explicado.
    for (_, _, rows) in crate::sections::player::PLAYER_CARDS {
        for (_, id, tip) in rows {
            store.set_tooltip(*id, *tip);
        }
    }
    for (id, tip) in crate::sections::player::PLAYER_BUTTON_TIPS {
        store.set_tooltip(id, tip);
    }
    for (id, value, min, max, step) in [
        // Metros. O piso NÃO é zero por acaso: uma perna de comprimento zero é
        // um personagem que não paira, e o piso geométrico real é maior ainda
        // (o botão Fit to Collider o diz).
        (ids::INSP_PLAYER_FLOAT, 0.5, 0.01, 100.0, 0.01), // LITERAL-PX-OK: meters
        (ids::INSP_PLAYER_CLING, 0.25, 0.0, 100.0, 0.01), // LITERAL-PX-OK: meters
        // Aceleração-por-metro. ⚠️ **TETO MEDIDO em 3600** (`1/dt²` para o tique
        // de 60 Hz, `RideConfig::MAX_SPRING_STRENGTH`): ali a mola chega ao alvo
        // em UM passo (*deadbeat*), e **acima disso ela passa do alvo** — 4000
        // afunda 2,5 cm no pouso, 5000 afunda 8,9.
        //
        // ⚠️ **Esta linha dizia *"sem teto medido"* e a frase envelheceu** — era
        // verdade até a `W-Landing` medir o teto e pôr o clamp na LEI, e ninguém
        // reconferiu a nota. O slider seguiu a oferecer **100 000**, vinte e
        // sete vezes o que o kernel honra: o artista arrastava até ao fim e não
        // via nada mudar. O gate `a_typable_ceiling_never_passes_what_the_law_honours`
        // lê a constante VIVA, então mover a medição move a faixa.
        //
        // ⚠️ E o default é o do produto (`RideConfig::STARTING_POINT`, 2000), não
        // o `400` do `bevy-tnua` que shipava aqui: o `Add` escreve 2000, então
        // este número era a segunda cópia de um default — e a errada.
        (ids::INSP_PLAYER_STIFFNESS, 2000.0, 0.0, 3600.0, 1.0), // LITERAL-PX-OK: accel per metre
        // ⚠️ TETO MEDIDO em 1.0 (`RideConfig::MAX_DAMPING`): acima dele o boost
        // INVERTE a velocidade em vez de matá-la, e o personagem pipoca. É o
        // único teto desta seção que descreve um limite de estabilidade em vez
        // de conveniência de stepper.
        //
        // ⚠️ E o default é o do produto (`1,0`, o teto): a `W-Landing` pôs o
        // amortecimento no teto para o resíduo de rampa ser **0,0000 m exato**
        // em toda inclinação, e este `0,5` era a segunda cópia — a errada.
        (ids::INSP_PLAYER_DAMPING, 1.0, 0.0, 1.0, 0.05), // LITERAL-PX-OK: fraction per tick
        // m/s, relativa ao chão.
        (ids::INSP_PLAYER_SPEED, 6.0, 0.0, 1000.0, 0.1), // LITERAL-PX-OK: m/s
        (ids::INSP_PLAYER_ACCEL, 60.0, 0.0, 10_000.0, 1.0), // LITERAL-PX-OK: m/s^2
        (ids::INSP_PLAYER_AIR_ACCEL, 20.0, 0.0, 10_000.0, 1.0), // LITERAL-PX-OK: m/s^2
        // ⚠️ **O teto de 100 é CONVENIÊNCIA, e o §0 exige que isto esteja escrito:**
        // a lei é auto-limitada (a sobra que cabe num tique é escrita EXATA), então
        // não há limite de recurso nenhum — o que existe é um ponto de SATURAÇÃO,
        // `speed / (turn·accel·dt)`, que vale **4,0** no perfil de partida e é
        // FUNÇÃO DA CONFIG. Um `MAX_*` não caberia; 100 dá folga de 25x sobre ele.
        (ids::INSP_PLAYER_BRAKE, 1.0, 0.0, 100.0, 0.1), // LITERAL-PX-OK: fraction of the budget
        // Graus. O teto é 90 porque acima disso a superfície aponta para BAIXO
        // e a pergunta deixa de ter sentido — recurso, não conveniência.
        (ids::INSP_PLAYER_MAX_SLOPE, 45.0, 0.0, 90.0, 1.0), // LITERAL-PX-OK: degrees
        // O PULO (W4). A altura em METROS, que é o que o artista pensa; os seis
        // multiplicadores em fração de gravidade, onde `1.0` é a do mundo.
        (ids::INSP_PLAYER_JUMP_HEIGHT, 2.0, 0.0, 1000.0, 0.1), // LITERAL-PX-OK: metres
        // ⚠️ **O teto de 8 NÃO é limite de recurso, e o §0 exige que isso esteja
        // escrito:** um contador de pulos custa uma comparação por tique e mora
        // num `u32` — nada se esgota. 8 é folga larga sobre o catálogo inteiro
        // (Celeste 1, Hollow Knight 1, Rayman 1, os "infinitos" são cheat), e a
        // caixa de texto continua aceitando o que o artista digitar.
        (ids::INSP_PLAYER_AIR_JUMPS, 0.0, 0.0, 8.0, 1.0), // LITERAL-PX-OK: count
        (ids::INSP_PLAYER_AIR_JUMP_H, 2.0, 0.0, 1000.0, 0.1), // LITERAL-PX-OK: metres
        // ⚠️ Os multiplicadores NÃO têm teto de recurso: o piso é 0 (gravidade
        // nenhuma naquela fase) e o topo é onde o desenho deixa de ser um pulo,
        // que é decisão de LOOK e não um limite físico. 20 é folga de sobra
        // sobre os defaults de 0,5..4 e é declarado como tal.
        (ids::INSP_PLAYER_TAKEOFF_G, 1.0, 0.0, 20.0, 0.1), // LITERAL-PX-OK: gravity multiple
        (ids::INSP_PLAYER_TAKEOFF_SPEED, 0.0, 0.0, 1000.0, 0.1), // LITERAL-PX-OK: m/s
        (ids::INSP_PLAYER_PEAK_G, 0.5, 0.0, 20.0, 0.1),    // LITERAL-PX-OK: gravity multiple
        (ids::INSP_PLAYER_PEAK_SPEED, 1.5, 0.0, 1000.0, 0.1), // LITERAL-PX-OK: m/s
        (ids::INSP_PLAYER_FALL_G, 2.0, 0.0, 20.0, 0.1),    // LITERAL-PX-OK: gravity multiple
        (ids::INSP_PLAYER_CUT_G, 4.0, 0.0, 20.0, 0.1),     // LITERAL-PX-OK: gravity multiple
        // ⚠️ **O TETO de 0,5 s é MEDIDO, e o recurso dele é a QUEDA:** a 0,5 s
        // o personagem já desceu `½·g·t² = 1,23 m` — mais de uma altura de
        // corpo (a cápsula tem 0,9 m) —, e a janela deixa de ler como *"eu
        // ainda estava na borda"* para ler como *"pulei do ar"*. A 0,1 s do
        // perfil de partida a queda é de 4,9 cm.
        (ids::INSP_PLAYER_COYOTE, 0.1, 0.0, 0.5, 0.01), // LITERAL-PX-OK: seconds
        (ids::INSP_PLAYER_BUFFER, 0.1, 0.0, 0.5, 0.01), // LITERAL-PX-OK: seconds
        // ⚠️ METROS, e o teto sai da MEDIÇÃO: acima de ~⅓ da largura do corpo a
        // assistência começa a salvar pulos que visivelmente bateram (a cápsula
        // das cenas tem 0,4 m). 0,3 é folga generosa para um corpo maior.
        (ids::INSP_PLAYER_CORNER, 0.12, 0.0, 0.3, 0.01), // LITERAL-PX-OK: metres
        // ⚠️ **O teto de 257 é MEDIDO e o recurso NÃO é tempo**
        // (`measure_player_probes::measure_what_a_sample_costs`): 18 ns por raio,
        // PLANO em N, então 257 custam 4,55 us = 0,027% de um quadro. O que se
        // esgota é a PRECISAO — o passo cai a 2,5 mm, e o solver assenta com
        // ~1,3 mm. O passo 2 é o clamp para ÍMPAR feito visível.
        (ids::INSP_PLAYER_CORNER_SAMPLES, 65.0, 1.0, 257.0, 2.0), // LITERAL-PX-OK: count
        // ⚠️ TIQUES, e 8 é folga larga: o default 2 é literalmente *"o boost no
        // tique ANTERIOR ao contato"*, e o leque se escala sozinho com a
        // velocidade (`rel_up · dt · N`), então um número grande aqui só antecipa
        // mais cedo — nunca alonga um sensor parado.
        (ids::INSP_PLAYER_CORNER_AHEAD, 2.0, 0.0, 8.0, 0.5), // LITERAL-PX-OK: ticks
        // ⚠️ Segundos, e o teto cobre o pulo mais longo da config de partida
        // (1,45 s no ar, medido) com folga para um `jump_height` maior.
        (ids::INSP_PLAYER_LIFT, 1.5, 0.0, 4.0, 0.05), // LITERAL-PX-OK: seconds
        // AS PAREDES (W13) — ⚠️ as duas primeiras nascem em ZERO: a capacidade
        // e' opt-in, e ligá-la por default mudaria todo player já autorado.
        (ids::INSP_PLAYER_WALL_SLIDE, 0.0, 0.0, 12.0, 0.25), // LITERAL-PX-OK: m/s
        (ids::INSP_PLAYER_WALL_JUMP, 0.0, 0.0, 6.0, 0.1),    // LITERAL-PX-OK: metres
        (ids::INSP_PLAYER_WALL_PUSH, 6.0, 0.0, 16.0, 0.25),  // LITERAL-PX-OK: m/s
        (ids::INSP_PLAYER_WALL_LOCK, 0.2, 0.0, 0.6, 0.02),   // LITERAL-PX-OK: seconds
        (ids::INSP_PLAYER_WALL_REACH, 0.08, 0.0, 0.4, 0.01), // LITERAL-PX-OK: metres
        // ⚠️ O MESMO teto do irmão da quina, e pela MESMA medição — um número, um
        // argumento. O que N compra aqui é COBERTURA DE FRESTA (3 num corpo de
        // 1 m cegam-se com 0,5 m; 9, com 12,5 cm), não precisão de beirada.
        (ids::INSP_PLAYER_FOOT_SAMPLES, 3.0, 1.0, 257.0, 2.0), // LITERAL-PX-OK: count
        (ids::INSP_PLAYER_FOOT_SPREAD, 1.0, 0.0, 1.0, 0.05),   // LITERAL-PX-OK: fraction
        (ids::INSP_PLAYER_WALL_SAMPLES, 3.0, 1.0, 257.0, 2.0), // LITERAL-PX-OK: count
        // ⚠️ FRAÇÃO da meia-altura: 1 põe as amostras de fora na borda exata da
        // caixa (o mundo de sempre) e baixá-lo afasta-as das PONTAS, onde uma
        // cápsula é um ponto e um raio rasante vê parede onde o corpo mal encosta.
        (ids::INSP_PLAYER_WALL_SPREAD, 1.0, 0.0, 1.0, 0.05), // LITERAL-PX-OK: fraction
        // ⚠️ O teto de 10 s NAO e' um limite de recurso — nao ha' recurso, e' um
        // `f32`. E' a FAIXA da UI, e o numero vem da referencia: a reserva do
        // Celeste da' ~11 s de pendura pura. Acima disso o artista nao quer um
        // numero maior, quer INFINITO, que e' outra feature (uma habilidade que
        // se ganha, nao um recurso que se gasta).
        (ids::INSP_PLAYER_WALL_GRAB, 0.0, 0.0, 10.0, 0.25), // LITERAL-PX-OK: seconds
        (ids::INSP_PLAYER_DASH_SPEED, 0.0, 0.0, 40.0, 0.5), // LITERAL-PX-OK: m/s
        (ids::INSP_PLAYER_DASH_TIME, 0.15, 0.0, 1.0, 0.01), // LITERAL-PX-OK: seconds
        (ids::INSP_PLAYER_DASH_COOL, 0.2, 0.0, 2.0, 0.02),  // LITERAL-PX-OK: seconds
        (ids::INSP_PLAYER_CROUCH_HEIGHT, 0.0, 0.0, 3.0, 0.05), // LITERAL-PX-OK: metres
        (ids::INSP_PLAYER_CROUCH_SPEED, 2.0, 0.0, 20.0, 0.25), // LITERAL-PX-OK: m/s
        // O NADO (W-Swim). ⚠️ O teto do LIMIAR é `4`, e ele é MEDIDO: numa poça
        // quatro vezes mais densa que o corpo — a fixture destas waves — a razão
        // satura em `3,99` com o corpo todo submerso (`measure_the_swim_threshold`),
        // então acima disso o número deixaria de ser alcançável e a capacidade
        // ficaria desligada com o slider a dizer o contrário.
        (ids::INSP_PLAYER_SWIM_SPEED, 0.0, 0.0, 20.0, 0.25), // LITERAL-PX-OK: m/s
        (ids::INSP_PLAYER_SWIM_ACCEL, 12.0, 0.0, 60.0, 0.5), // LITERAL-PX-OK: m/s2
        (ids::INSP_PLAYER_SWIM_ENTER, 1.0, 0.0, 4.0, 0.05),  // LITERAL-PX-OK: weights
        // A BEIRADA (W-Ledge). ⚠️ O teto do ALCANCE é `2`, e o que ele mede é o
        // BRAÇO: acima de uma altura de corpo ele deixa de ser um alcance e vira
        // teletransporte para patamares que o personagem não vê. O da velocidade
        // acompanha o do arranque — é o mesmo tipo de gesto.
        (ids::INSP_PLAYER_LEDGE_GRAB, 0.0, 0.0, 2.0, 0.05), // LITERAL-PX-OK: m
        (ids::INSP_PLAYER_LEDGE_REACH_Y, 0.6, 0.0, 2.0, 0.05), // LITERAL-PX-OK: m
        (ids::INSP_PLAYER_LEDGE_SPAN, 0.0, 0.0, 2.0, 0.05), // LITERAL-PX-OK: m
        (ids::INSP_PLAYER_LEDGE_OFFSET_Y, 0.0, -2.0, 2.0, 0.05), // LITERAL-PX-OK: m
        (ids::INSP_PLAYER_LEDGE_SPEED, 3.0, 0.0, 20.0, 0.25), // LITERAL-PX-OK: m/s
        (ids::INSP_PLAYER_GLIDE_FALL, 0.0, 0.0, 20.0, 0.25), // LITERAL-PX-OK: m/s
        (ids::INSP_PLAYER_MAX_FALL, 0.0, 0.0, 150.0, 0.5),  // LITERAL-PX-OK: m/s
        // A REAÇÃO (W6), em FRAÇÃO da força que o personagem faz. ⚠️ O piso é 0
        // (nada volta) e o teto é 1 (volta inteira) porque **acima de 1 o
        // personagem devolveria mais do que recebeu** — inventar energia, e o
        // único teto desta seção que é de RECURSO e não de gosto.
        (ids::INSP_PLAYER_REACT_SUPPORT, 1.0, 0.0, 1.0, 0.05), // LITERAL-PX-OK: fraction
        (ids::INSP_PLAYER_REACT_MOVEMENT, 0.0, 0.0, 1.0, 0.05), // LITERAL-PX-OK: fraction
        (ids::INSP_PLAYER_REACT_PUSH, 1.0, 0.0, 1.0, 0.05),    // LITERAL-PX-OK: fraction
    ] {
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value,
                buffer: format_number(value),
                caret: 0,
                last_committed: value,
                selection_anchor: None,
            },
        );
        store.set_number_range(id, min, max, step);
    }
}
