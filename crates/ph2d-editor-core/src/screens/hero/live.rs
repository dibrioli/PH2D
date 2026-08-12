//! **O tique da UI VIVA** — uma chamada por quadro, com o `dt` de PAREDE.
//!
//! Irmão do `hero.rs`, e não corpo dele: aquele diz o que uma tela **É** (os campos, os painéis, a
//! selecção); isto diz o que ela **FAZ** a cada quadro. O corte nasceu no tecto de 700 LOC quando a
//! corda chegou, e a linha de corte escolheu-se sozinha — os dois inquilinos aqui (`motion` e
//! `tether`) partilham a forma inteira: são estado de APARÊNCIA, avançados de um sítio só, lidos
//! por pintores que nunca os mexem.

use super::HeroScreen;

/// **Anda o relógio da UI viva e re-alveja a partir do estado semântico.**
///
/// ⚠️ **A ordem é load-bearing:** avançar PRIMEIRO (o tempo que passou desde o último quadro
/// aplica-se ao alvo que estava em vigor) e só então ler os alvos novos do store, que os eventos de
/// ponteiro deste quadro acabaram de mexer. Ao contrário, o primeiro quadro de um hover andaria com
/// o alvo novo por um `dt` que decorreu **antes** de o rato lá chegar.
pub(super) fn tick(hero: &mut HeroScreen, dt: f64) {
    hero.motion.advance(dt);
    for (id, target) in hero.store.hover_targets().collect::<Vec<_>>() {
        hero.motion.animate(id, target, crate::motion::Role::Fade);
    }
    tick_fill_tether(hero, dt);
}

/// A CORDA do card de Fill: liga o sítio onde ele nasceu (a largada do ColorDrop) ao sítio para
/// onde o artista o arrastou.
///
/// ⚠️ Avança AQUI e não no pintor, pela mesma lei do `motion`: uma corda que avançasse ao pintar
/// seria função de *quantas vezes* foi pintada, e não do tempo.
///
/// ⚠️ E o `simulate` vem de `motion.decorates()` — **perguntado, nunca cravado**. Uma constante
/// `true` aqui simula em Discreto com a suíte inteira verde: foi uma mutação sobrevivente, e o
/// `the_seam_asks_the_character_it_does_not_hardcode_the_rope` nasceu dela.
fn tick_fill_tether(hero: &mut HeroScreen, dt: f64) {
    match (hero.store.fill_modal_pos(), hero.store.fill_modal_anchor()) {
        (Some(pos), Some(anchor)) => hero.tether.advance(
            [anchor.0, anchor.1],
            [pos.0, pos.1],
            dt as f32,
            hero.motion.decorates(),
        ),
        // Card fechado: a corda ESQUECE a pose. Sem isto, a próxima largada noutro canto do ecrã
        // faria a corda voar do sítio onde a anterior morreu — um rasto de um gesto que já acabou.
        _ => hero.tether.reset(),
    }
}
