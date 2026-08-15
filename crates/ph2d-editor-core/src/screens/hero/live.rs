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
/// ⚠️ **O resultado do `animate` é PUBLICADO no store**, e não descartado como era: é o gêmeo
/// exacto do que o `tick_panel_scroll` faz logo abaixo, e a razão é a mesma medida — os pintores já
/// perguntam ao `store`, e uma corrente de `motion` a par dela custaria **56 assinaturas só no
/// `ph2d-panel-inspector`**, para 20 botões. Ver [`crate::interaction::WidgetStore::button_visual`].
///
/// ⚠️ **A ordem é load-bearing:** avançar PRIMEIRO (o tempo que passou desde o último quadro
/// aplica-se ao alvo que estava em vigor) e só então ler os alvos novos do store, que os eventos de
/// ponteiro deste quadro acabaram de mexer. Ao contrário, o primeiro quadro de um hover andaria com
/// o alvo novo por um `dt` que decorreu **antes** de o rato lá chegar.
pub(super) fn tick(hero: &mut HeroScreen, dt: f64) {
    hero.motion.advance(dt);
    for (id, target) in hero.store.hover_targets().collect::<Vec<_>>() {
        let live = hero.motion.animate(id, target, crate::motion::Role::Fade);
        hero.store.set_hover_live(id, live);
    }
    tick_palette_cascade(hero, dt);
    tick_panel_scroll(hero);
    tick_section_fold(hero);
    tick_fill_tether(hero, dt);
}

/// **A DOBRA de uma secção** — o chevron gira em vez de trocar de glifo.
///
/// ⚠️ **`Role::Surface`, e não `Travel`** — pela razão exacta que o `Role` nomeia: uma dobra
/// obedece ao dedo. Ultrapassar mostraria o chevron a passar do ângulo que a secção de facto tem
/// e a voltar, que é a régua a mentir sobre um estado binário; e num carácter Expressivo o
/// `Travel` ultrapassa 15,5%.
///
/// ⚠️ **A PRIMEIRA vista parte da PARTIDA, não do alvo.** O `toggle_collapsed` grava onde a
/// secção estava; aqui isso é lido e alvejado na mesma passagem — semear e re-alvejar no mesmo
/// tique é o que faz a mola nascer em voo, e é o mesmo movimento que a cascata da paleta faz.
/// Sem ele a estreia de cada secção saltaria (a lei: *a primeira vista CHEGA ao alvo*).
fn tick_section_fold(hero: &mut HeroScreen) {
    let states: Vec<(ph2d_a11y::NodeId, bool)> = hero.store.collapse_states().collect();
    for (id, collapsed) in states {
        let target = if collapsed { 0.0 } else { 1.0 };
        let track = fold_track(id);
        if hero.motion.get(track).is_none() {
            let from = hero.store.section_open_live(id);
            hero.motion
                .animate(track, from, crate::motion::Role::Surface);
        }
        let live = hero
            .motion
            .animate(track, target, crate::motion::Role::Surface);
        hero.store.set_section_open_live(id, live);
    }
}

/// O id de motion da dobra de uma secção. ⚠️ **Não é o id da secção** — aquele já é alvo de hit
/// (o cabeçalho é clicável) e o `hover_targets` pode animá-lo; partilhá-lo poria *quanto do hover
/// está presente* e *quanto da secção está aberta* no MESMO track. O mesmo argumento do
/// `scroll_track`, e a constante de mistura é outra para os dois nunca colidirem.
fn fold_track(section: ph2d_a11y::NodeId) -> ph2d_a11y::NodeId {
    const FNV_PRIME_64: u64 = 0x0000_0100_0000_01b3;
    let mut h = section.0 ^ 0x666f_6c64_5f74_7261; // "fold_tra"
    for b in section.0.to_le_bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(FNV_PRIME_64);
    }
    ph2d_a11y::NodeId(if h == 0 { 1 } else { h })
}

/// **A ROLAGEM SUAVE** — a roda mexe um ALVO, e a superfície desliza até lá.
///
/// ⚠️ **É `Role::Surface`, e o `Travel` que aqui shipou primeiro foi reprovado no smoke** (*«o
/// balanço das labels ficou bem artificial e pouco suave»*): em Expressivo o `Travel` ultrapassa
/// 15,5%, e uma superfície que passa do sítio para onde a roda a mandou está a contradizer o gesto.
/// O *reduced motion* continua a levá-la — e leva-a bem: uma superfície que desliza é exactamente a
/// classe de movimento que o interruptor existe para matar.
///
/// ⚠️ **O tique é o ÚNICO escritor do vivo**, e é por isso que isto não é uma segunda cópia do
/// mesmo facto: o alvo é *para onde a roda mandou*, o vivo é *onde a superfície está*. Os ~130
/// sítios que pintam a partir de `panel_scroll` herdaram a suavidade sem uma linha de mudança,
/// porque aquela porta passou a devolver o vivo.
///
/// ⚠️ E **não anda o relógio**: quem o andou foi o `motion.advance` no topo do `tick`. Uma segunda
/// chamada aqui daria a esta família o dobro do `dt` das outras.
fn tick_panel_scroll(hero: &mut HeroScreen) {
    let targets: Vec<(ph2d_a11y::NodeId, f32)> = hero.store.scrolled_panels().collect();
    for (panel, target) in targets {
        let live = hero
            .motion
            .animate(scroll_track(panel), target, crate::motion::Role::Surface);
        // ⚠️ **A superfície pousa na GRADE DE PIXELS, e é aqui — no PUBLICAR — que ela pousa.**
        //    O relógio guarda o valor contínuo (uma mola alimentada com entrada quantizada pode
        //    estagnar perto do alvo) e o ALVO guarda o valor exato (é ele que soma os deltas
        //    fraccionários de um trackpad). Quantizar o número que os ~130 pintores leem é o que
        //    põe a linha, o filete e a LABEL no mesmo passo — ver `motion::on_pixel_grid`, que
        //    traz a medição do tremor que isto remove.
        hero.store
            .set_panel_scroll_live(panel, crate::motion::on_pixel_grid(live));
    }
}

/// O id de motion da rolagem de um painel. ⚠️ **Não é o id do painel** — aquele já é o alvo de hit
/// do chrome e do `hover_targets`, e partilhá-lo poria duas grandezas (*quanto do hover está
/// presente* e *onde a superfície está*) no MESMO track.
fn scroll_track(panel: ph2d_a11y::NodeId) -> ph2d_a11y::NodeId {
    const FNV_PRIME_64: u64 = 0x0000_0100_0000_01b3;
    let mut h = panel.0 ^ 0x7363_726f_6c6c_5f74; // "scroll_t"
    for b in panel.0.to_le_bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(FNV_PRIME_64);
    }
    ph2d_a11y::NodeId(if h == 0 { 1 } else { h })
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
