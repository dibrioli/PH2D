//! **O MOVIMENTO CINEMÁTICO** (`W-KinMove`) — o que a ponte FAZ com os
//! deslocamentos que o laço de players colheu.
//!
//! ⚠️ **Corte por RESPONSABILIDADE, e ele é o mesmo que o do `player_marks`:** o
//! pai [`super::player`] pergunta ao mundo e COLHE (o cast toma `&self`), e este
//! APLICA (escrever a pose toma `&mut self`). A lista de `KinMove` já existia
//! por essa razão; o que muda aqui é que a metade que a consome passa a ter casa
//! própria, em vez de ser a cauda de um laço de quinhentas linhas.
//!
//! Módulo FILHO por `#[path]`, então `super::*` continua a alcançar o que o pai
//! não exporta.

use super::*;

/// **Um deslocamento cinemático devido** (W-KinMove) — colhido no laço do pai e
/// aplicado aqui, pelo motivo de sempre: perguntar ao mundo toma `&self` e
/// escrever a pose toma `&mut self`.
///
/// ⚠️ **Ela mudou-se para cá quando a `W-Ceiling` empurrou o pai contra o teto de
/// LOC** (695 → 711), e o corte é o que o doc deste módulo já declarava: a lista
/// é a fronteira entre COLHER e APLICAR, e quem a consome é este lado. ⚠️ **A
/// mudança curou um doc-comment ÓRFÃO de carona:** o texto do `GroundPush`
/// estava colado por cima desta struct sem linha em branco, então o rustdoc lia
/// os dois como UM só — o `KinMove` anunciava-se como *"um empurrão devido ao
/// chão (W6)"* e o `GroundPush` ficava sem doc nenhum.
pub(super) struct KinMove {
    pub(super) entity: Entity,
    pub(super) handle: rapier2d_handle::Handle,
    pub(super) layer: u8,
    pub(super) passing: Option<ph2d_physics::ColliderHandle>,
    pub(super) params: CharacterParams,
    /// O deslocamento que a lei PEDIU.
    pub(super) wanted: [f32; 2],
    /// A velocidade depois do avanço e **antes** do assentamento.
    pub(super) advanced: KinematicState,
    /// Os escalares da 3ª lei — o EMPURRÃO (W-KinPush) lê o `push` daqui.
    ///
    /// ⚠️ Viaja na lista porque a config do player é lida no primeiro laço (com
    /// `&self`) e o empurrão é aplicado no segundo (com `&mut self`): relê-la
    /// depois seria uma segunda consulta ao mundo ECS a meio do tique, e as
    /// duas divergiriam no frame em que o artista arrasta o slider.
    pub(super) react: ReactionConfig,
}

impl PhysicsBridge {
    /// **Aplica os deslocamentos colhidos** — mover o personagem, empurrar quem
    /// ele encostou, escrever a pose e devolver-lhe o que o mundo recusou.
    ///
    /// ⚠️ **Por último no tique, e a ordem importa:** a reação já foi enfileirada
    /// contra o chão, e o `move_shape` lê o BVH que o `step` ANTERIOR deixou (o
    /// mesmo contrato do sensor, ver o topo de [`super::player`]) — nada aqui
    /// depende dos impulsos deste tique.
    pub(super) fn apply_kin_moves(&mut self, moves: Vec<KinMove>, dt: f32) {
        // ⚠️ **UM buffer para o laço inteiro**, limpo pela porta que o preenche
        // (ver `move_character`): uma lista por personagem seria uma alocação
        // por player por tique, para carregar no máximo um punhado de contatos.
        let mut hits: Vec<CharacterHit> = Vec::new();
        let mut pushes: Vec<Push> = Vec::new();
        for m in moves {
            let got = self
                .world
                .move_character(m.handle, m.wanted, m.params, m.passing, m.layer, &mut hits);
            // ── O EMPURRÃO (W-KinPush) ───────────────────────────────────────
            //
            // ⚠️ **Um corpo cinemático tem massa INFINITA para o solver**, então
            // o `move_shape` desliza contra um caixote solto sem lhe transmitir
            // nada — medido, o dinâmico o empurra 16,55 m em 3 s e o cinemático
            // 0,0000. A 3ª lei já atravessa o modo no eixo VERTICAL (a
            // `reaction`, K6); isto é a metade lateral dela.
            //
            // ⚠️ **UM empurrão por CORPO, e é o que impede a contagem dupla:**
            // o controlador desliza em iterações e pode relatar o mesmo corpo
            // várias vezes; somar cada relatório entregaria a mesma quantidade
            // de movimento duas vezes ao mesmo caixote.
            push::push_from_hits(&m.react, m.wanted, got.translation, &hits, &mut pushes);
            for &(body, transfer, at) in &pushes {
                // ⚠️ **A conversão é a da reação vertical**: um deslocamento de
                // um tique É uma velocidade quando dividido pelo tique, que é
                // exatamente o que o canal `boost` significa (`Δv·m`).
                //
                // ⚠️ **Mas a ENTREGA é por SUB-PASSO** (§8.2, escolha do Enio),
                // e é o que separa esta metade da vertical. Entregue de uma vez,
                // o bloqueio inteiro do tique entrava como UMA martelada num
                // ponto alto do caixote e `r × F` fazia o resto — medido, um
                // caixote pequeno dava **12 voltas em 3 s** (74,29 rad) contra
                // 0,3175 do corpo dinâmico, e o giro seguia a ALAVANCA (some
                // quando o contato desce até o centro de massa).
                //
                // O dinâmico empurra com força SUSTENTADA por sub-passo, e essa
                // é literalmente a diferença medida — então a cura é entregar
                // pelo mesmo mecanismo, não capar o torque com um número novo.
                let _ = at;
                self.world
                    .apply_player_push(body, m.handle, [transfer[0] / dt, transfer[1] / dt]);
            }
            if let Some(pose) = self.world.body_pose(m.handle) {
                // ⚠️ **`set_next_kinematic_pose`, nunca uma escrita direta:** é
                // ela que faz o solver tratar o corpo como MOVENDO-SE, e é isso
                // que o faz empurrar o que toca em vez de o atravessar.
                self.world.set_next_kinematic_pose(
                    m.handle,
                    pose.translation.x + got.translation[0],
                    pose.translation.y + got.translation[1],
                    pose.rotation.angle(),
                );
            }
            // ⚠️ **E o que o mundo NÃO deixou acontecer volta como velocidade**
            // — sem isto o personagem acelera contra uma parede para sempre e
            // sai disparado quando ela acaba (ver `kinematic_settle`).
            if let Some(st) = self.player_state.get_mut(&m.entity) {
                st.kin = kinematic_settle(
                    m.advanced,
                    m.wanted,
                    got.translation,
                    // ⚠️ **A pergunta do INTEGRADOR** (ver `KinematicState::grounded`)
                    // — quem responde é quem TOCOU no mundo. A resposta da LEI
                    // sobre chão continua a `footing`, nos dois modos (K4).
                    got.grounded,
                    dt,
                );
            }
        }
    }
}
