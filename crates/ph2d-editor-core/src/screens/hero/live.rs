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
    tick_palette_cascade(hero, dt);
    tick_fill_tether(hero, dt);
}

/// A CASCATA da paleta de comandos: os cartões chegam **um a seguir ao outro**, para que `N` deles
/// se leiam como um gesto só em vez de um bloco que pisca.
///
/// ⚠️ **`Role::Travel` — e este é o PRIMEIRO consumidor dele no produto.** Até aqui o substrato
/// tinha quatro papéis e o eixo que o *reduced motion* existe para matar **não era usado por
/// ninguém**: o interruptor estava ligado a um cabo sem lâmpada. A subida dos cartões é a lâmpada.
///
/// ⚠️ **O alvo é escalonado; a MOLA não sabe disso.** Ver [`crate::motion::cascade_target`].
///
/// ⚠️ **Fechada, o horário ZERA e nenhum alvo é escrito.** Os tracks são podados sozinhos quando
/// param de ser pintados (`ids_that_stop_being_painted_are_pruned`), então a reabertura encontra o
/// mapa limpo e a primeira vista de cada cartão chega a `0` — que é o que a torna uma entrada.
fn tick_palette_cascade(hero: &mut HeroScreen, dt: f64) {
    let Some(model) = hero.store.command_palette_model() else {
        hero.palette_open_secs = 0.0;
        return;
    };
    let n = model.groups.len();
    // ⚠️ **ALVEJA com o horário de AGORA e só então anda o relógio — e a sonda provou que a ordem
    //    inversa apaga a wave inteira.** Somando primeiro, o quadro da abertura já traz `secs = dt`,
    //    logo `dt > 0` e o cartão 0 nasce com alvo `1.0`; pela lei do substrato a **primeira vista
    //    CHEGA ao alvo**, então ele apareceria assente e a cascata começaria no segundo cartão. Com
    //    esta ordem todo cartão nasce em `0` no quadro da abertura, que é a interrupção que a mola
    //    sabe integrar. Medido: `n=1` dava **0,02 s** (um quadro — o salto) e dá 0,37 s.
    let secs = hero.palette_open_secs;
    hero.palette_open_secs += dt;
    for i in 0..n {
        hero.motion.animate(
            crate::widget::command_palette::cascade_id(i),
            crate::motion::cascade_target(secs, i),
            crate::motion::Role::Travel,
        );
    }
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
