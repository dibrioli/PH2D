//! **Quando uma DESCIDA deixa de valer** (W12/W20) — irmão do `player.rs` pelo
//! teto de LOC, e cortado por RESPONSABILIDADE: o resto daquele arquivo é o fio
//! de um tique (amostrar, chamar a lei, aplicar), e isto é a **aposentadoria de
//! um estado que atravessa tiques**. A lei inteira, com as medições que a
//! produziram, viaja com ela.

use ph2d_ecs::Entity;

use super::PhysicsBridge;

/// **As descidas que já cumpriram o seu papel** (W12/W20) — rodada no topo
/// de cada tique de player, antes de o sensor perguntar qualquer coisa.
///
/// # A lei, numa frase
///
/// A descida morre quando **já passei** (a caixa do personagem está
/// inteiramente abaixo da caixa da plataforma) **E a prancha já parou de me
/// pegar** (o gancho one-way não relatou nada neste tique).
///
/// ⚠️ **As duas metades são obrigatórias, e cada uma cura um defeito que a
/// outra tem** — as duas foram medidas
/// (`ph2d-physics-ecs/tests/measure_drop_retire.rs`).
///
/// # ⚠️ Só a geometria EXPULSA o personagem
///
/// A caixa estar abaixo **não** garante que a prancha não vá agir: com o
/// corpo **0,016 m abaixo** da prancha, sem sobreposição nenhuma, a
/// re-solidificação o atirou de volta ao degrau de cima com um pico de
/// **0,3267 N·s** entre sub-passos — e o `impulse` de fim de tique lia
/// `0,0000`, que é a lição da W-ImpactForce outra vez. Faixa medida: prancha
/// de meia-espessura 0,15, vãos **1,75 a 1,85**, onde ele **não descia de
/// todo** e o botão parecia não fazer nada. É o livro-razão do gancho
/// (`PhysicsWorld::drop_is_catching`) que fecha essa borda, porque ele
/// pergunta à normal do manifold em vez de a caixas.
///
/// # ⚠️ Só a evidência REGRIDE a descida
///
/// Quando a prancha fica inteiramente DENTRO da caixa do personagem (prancha
/// fina, corpo alto) não existe *lado*, e a normal do manifold **oscila**
/// entre tiques — medido, o ponto de contato saltando de `−0,486` para
/// `+0,490` em dois tiques. Uma lei só de evidência aposenta no primeiro
/// "não" dessa oscilação e a prancha o empurra para cima: com prancha 0,10 e
/// vãos 1,10 a 1,25 ele **deixava de descer**. A geometria não oscila, e é
/// ela que segura a evidência até a travessia ter de facto acabado.
///
/// # ⚠️ O que AINDA fica fantasma, e a lei disso
///
/// Medido célula a célula, **a descida sobrevive exactamente onde a caixa de
/// repouso do personagem ainda SOBREPÕE a prancha** — nenhuma exceção nas
/// duas espessuras varridas:
///
/// | meia-espessura | vão | o que acontece |
/// |---|---|---|
/// | 0,15 | 1,60 – 1,70 | desce, e a prancha fica **fantasma** |
/// | 0,15 | 1,75 + | funciona (era **arremessado** até 1,85) |
/// | 0,10 | 1,50 – 1,60 | desce, e a prancha fica **fantasma** |
/// | 0,10 | 1,65 + | funciona |
///
/// Nessa faixa a prancha **de facto o pegaria** (o cone do gancho devolve
/// `+1,000`, medido), então as duas saídas são *fantasma* ou *cuspido* — e
/// fantasma é a menos má.
///
/// ⛔ **E o ALCANCE disso foi MEDIDO e é MENOR do que esta nota afirmava.**
/// A frase que esteve aqui dizia *"o preço continua a ser a cena inteira —
/// enquanto essa descida vive, nenhuma prancha é sólida para ele"*, e
/// prescrevia a **descida por-PLATAFORMA** como cura. Ela foi construída
/// inteira (conjunto de pares no lugar do bit, evidência por par, o gesto a
/// levar também as plataformas que o corpo já sobrepõe, o raio a ignorar a
/// lista) e **REVERTIDA**: numa cena com a escada apertada e uma prancha
/// SOLTA ao lado, a solta **segura o personagem nos DOIS mundos** — pela
/// perna, não pelo solver (`measure_whether_a_live_drop_really_dissolves_the_whole_scene`).
///
/// O bit global limpa **contatos do solver**; quem segura este personagem é
/// a **mola**, e o raio dela só ignora a plataforma da travessia. Então o
/// que a descida viva de facto custa é a prancha que ela nomeia, e não a
/// cena — e uma cura por-plataforma seria complexidade sem número.
///
/// ⚠️ A sonda **falhou o próprio controle duas vezes** antes de decidir (o
/// personagem não saía da escada; depois andava 400 tiques e atravessava a
/// prancha solta a caminho do outro lado do mundo). *Um A/B em que os dois
/// lados dão o mesmo número só vale depois de o controle dar um número
/// diferente.*
///
/// ⚠️ E a cena 91 deixou de viver dez centímetros acima de um penhasco: com
/// `RISE = 2,0` e pranchas de 0,15 a margem passou de 0,10 para **0,25**, e
/// a borda que sobrou é a honesta (ali o personagem não cabe).
pub(super) fn retire_drops(bridge: &mut PhysicsBridge) {
    // O caso comum é ninguém a descer, e ele não lê um byte.
    if bridge.player_drop.is_empty() {
        return;
    }
    let mut done: Vec<Entity> = Vec::new();
    for (&entity, &platform) in &bridge.player_drop {
        let Some(b) = bridge.bodies.get(&entity) else {
            // O corpo morreu: não há descida a manter.
            done.push(entity);
            continue;
        };
        // ── A GEOMETRIA: já passei? ──────────────────────────────────────
        let past = match (
            bridge.world.collider_aabb(platform),
            bridge.world.body_aabb(b.handle),
        ) {
            (Some((plat_mins, _)), Some((_, body_maxs))) => body_maxs[1] <= plat_mins[1],
            // A plataforma (ou o corpo) deixou de existir — o mesmo
            // veredito, pela mesma razão.
            _ => true,
        };
        // ── A INTENÇÃO: já estou a SUBIR? (W27) ──────────────────────────
        //
        // ⚠️ **Uma descida travada existe para deixar passar para BAIXO.**
        // No instante em que o corpo sobe, quem decide já é o **cone** do
        // one-way (`ALLOWED_COS`), que deixa passar por baixo por conta
        // própria — manter o bit ali não protege nada, e prende.
        //
        // ⚠️ **E o que ele prendia era o PERSONAGEM, não um contorno.**
        // Medido (`measure_what_an_armed_drop_costs`): no vão em que a
        // descida nunca se aposentava, ele descia um degrau e **ficava lá
        // para sempre** — `−0,598 → −0,598` a 1,60, em toda célula da
        // janela. O item estava registado como *"as pranchas ficam
        // fantasma"*, que é o sintoma; a armadilha é o preço.
        //
        // ⚠️ **Ela não pode reabrir a borda de CIMA**, e a razão é o sinal:
        // aquele defeito é a prancha voltar a ser sólida **com ele a CAIR
        // através dela**, e esta cláusula só dispara com a velocidade para
        // cima. Os gates daquela borda ficam verdes ao lado do desta.
        let rising = bridge
            .world
            .body_velocity(b.handle)
            .is_some_and(|v| v[1] > 0.0);
        // ── A EVIDÊNCIA: e a prancha já parou de me pegar? ───────────────
        if rising || (past && !bridge.world.drop_is_catching(b.handle)) {
            done.push(entity);
        }
    }
    for entity in done {
        bridge.player_drop.remove(&entity);
        if let Some(b) = bridge.bodies.get(&entity) {
            let handle = b.handle;
            bridge.world.set_body_drop_through(handle, false);
        }
    }
}
