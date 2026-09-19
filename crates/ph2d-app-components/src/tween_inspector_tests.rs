//! Os gates da [`super`] — o painel lê o que o objecto TEM, e um clique escreve de volta.

use super::*;
use ph2d_ecs::Timer;

fn mundo(n_tweens: usize, n_timers: usize, com_sprite: bool) -> (World, u64) {
    let mut w = World::new();
    let mut ent = w.spawn((
        Tweens((0..n_tweens).map(|_| Tween::default()).collect()),
        Timers((0..n_timers).map(|_| Timer::default()).collect()),
    ));
    if com_sprite {
        ent.insert(Sprite::atlas(0, [1.0, 1.0], [1.0, 1.0, 1.0, 1.0]));
    }
    let bits = ent.id().to_bits();
    (w, bits)
}

/// ⚠️ **`None` para quem não tem o componente** — o ADR-0166.
#[test]
fn sem_o_componente_nao_ha_seccao() {
    let mut w = World::new();
    let e = w.spawn(ph2d_ecs::Transform::default()).id().to_bits();
    assert!(build_tween_info(&w, e, 1).is_none());
}

/// ⭐⭐ **As duas colunas que NÃO vêm do componente** — *qual é a duração do relógio deste índice?* e
/// *há sprite?*
///
/// ⚠️ **O CONTROLO é metade do gate:** sem o caso com timer a mais, um `duracao_us` cravado em
/// `None` passaria; sem o caso com sprite, um cravado em `true` passaria.
#[test]
fn o_instantaneo_le_a_cena_e_nao_so_o_componente() {
    // Dois tweens, UM timer ⇒ o segundo não tem relógio.
    let (w, e) = mundo(2, 1, true);
    let i = build_tween_info(&w, e, 1).unwrap();
    assert_eq!(i.rows.len(), 2);
    assert!(i.rows[0].duracao_us.is_some(), "o primeiro TEM relogio");
    assert!(i.rows[1].duracao_us.is_none(), "o segundo NAO tem");
    assert!(i.tem_sprite);

    let (w, e) = mundo(1, 1, false);
    let i = build_tween_info(&w, e, 1).unwrap();
    assert!(!i.tem_sprite, "sem sprite, a coluna tem de o dizer");
}

/// ⭐⭐⭐ **A DURAÇÃO que o painel mostra é a do relógio do MESMO ÍNDICE** — a resposta à pergunta do
/// dono no smoke de 2026-09-19: *«onde selecciono o tempo?»*.
///
/// ⚠️ **A fixtura tem DOIS de cada, com durações DIFERENTES**, e é isso que a torna um teste de
/// índice: *uma fixtura com um elemento não pode testar um índice* — `get(i)` e `first()` devolvem
/// exactamente a mesma coisa, que é a lição que a prova de mutação desta wave já pagou.
///
/// **Mutações que devem sangrar:** ler `first()` em vez de `get(i)` · devolver `None` sempre.
#[test]
fn a_duracao_que_o_painel_mostra_e_a_do_relogio_do_mesmo_indice() {
    let mut w = World::new();
    let e = w
        .spawn((
            Tweens(vec![Tween::default(), Tween::default()]),
            Timers(vec![
                Timer {
                    duration_us: 400_000,
                    ..Timer::default()
                },
                Timer {
                    duration_us: 1_200_000,
                    ..Timer::default()
                },
            ]),
        ))
        .id()
        .to_bits();
    let i = build_tween_info(&w, e, 1).unwrap();
    assert_eq!(i.rows[0].duracao_us, Some(400_000));
    assert_eq!(i.rows[1].duracao_us, Some(1_200_000));
}

/// ⭐⭐ **Um relógio a ZERO é uma queixa PRÓPRIA** — ele existe, e nunca dispara.
///
/// ⚠️ **As duas curas ficam em sítios diferentes** (anexar um timer · escrever a duração), e é por
/// isso que não podem partilhar a frase. *Dizer «não há relógio» a quem tem um relógio a zero
/// manda-o anexar um segundo, e aí ele fica com dois tweens e um deles mudo.*
#[test]
fn um_relogio_a_zero_queixa_se_de_si_mesmo_e_nao_de_ausencia() {
    let mut w = World::new();
    let e = w
        .spawn((
            Tweens(vec![Tween::default()]),
            Timers(vec![Timer {
                duration_us: 0,
                ..Timer::default()
            }]),
            Sprite::atlas(0, [1.0, 1.0], [1.0, 1.0, 1.0, 1.0]),
        ))
        .id()
        .to_bits();
    let i = build_tween_info(&w, e, 1).unwrap();
    assert_eq!(i.rows[0].duracao_us, Some(0));
    assert_eq!(
        i.rows[0].queixa(i.tem_sprite),
        Some(ph2d_editor_core::tween_edits::TweenQueixa::RelogioSemDuracao)
    );
    // ⛔ O CONTROLO: com duração, a queixa do relógio CALA-SE — senão isto passaria por vacuidade
    // sobre um painel que se queixa sempre.
    let (w, e) = mundo(1, 1, true);
    let i = build_tween_info(&w, e, 1).unwrap();
    assert_ne!(
        i.rows[0].queixa(i.tem_sprite),
        Some(ph2d_editor_core::tween_edits::TweenQueixa::RelogioSemDuracao)
    );
}

/// **O `+` respeita o tecto — e a cerca é da PORTA, não do painel.**
///
/// ⚠️ *Uma cerca só na UI é uma cerca que o próximo chamador contorna* — o painel já esconde o
/// botão no limite, e esta é a segunda metade.
#[test]
fn o_add_respeita_o_tecto_na_porta_que_escreve() {
    let (mut w, e) = mundo(ph2d_ecs::TWEENS_MAX, 0, true);
    assert!(
        !apply_tween_edit(&mut w, e, &TweenFieldEdit::Add),
        "o `Add` passou o tecto"
    );
    assert_eq!(
        w.get::<Tweens>(Entity::from_bits(e)).unwrap().0.len(),
        ph2d_ecs::TWEENS_MAX
    );
    // O controlo: com folga, ele acrescenta.
    let (mut w, e) = mundo(1, 1, true);
    assert!(apply_tween_edit(&mut w, e, &TweenFieldEdit::Add));
    assert_eq!(w.get::<Tweens>(Entity::from_bits(e)).unwrap().0.len(), 2);
}

/// ⭐⭐⭐ **Todo campo do editor chega ao componente** — e o gate mede o BARRO, nunca a tabela.
///
/// ⚠️ **É o ponto cego que o `CLAUDE.md` §5.0 nomeia sobre si mesmo** (*«nenhum instrumento do repo
/// pergunta se o VALOR chega a um consumidor»*): cada edição é aplicada e o campo do `Tween` é
/// relido — uma variante que o dreno esquecesse passaria num gate que só contasse variantes.
#[test]
fn toda_edicao_chega_ao_componente() {
    let (mut w, e) = mundo(1, 1, true);
    let ent = Entity::from_bits(e);

    assert!(apply_tween_edit(
        &mut w,
        e,
        &TweenFieldEdit::Canal(0, Canal::ScaleX.tag())
    ));
    assert_eq!(w.get::<Tweens>(ent).unwrap().0[0].canal, Canal::ScaleX);

    assert!(apply_tween_edit(&mut w, e, &TweenFieldEdit::De(0, 0, 3.5)));
    assert!((w.get::<Tweens>(ent).unwrap().0[0].de[0] - 3.5).abs() < 1e-6);

    assert!(apply_tween_edit(
        &mut w,
        e,
        &TweenFieldEdit::Para(0, 2, -1.25)
    ));
    assert!((w.get::<Tweens>(ent).unwrap().0[0].para[2] + 1.25).abs() < 1e-6);

    let cubic = ph2d_anim::EasingFamily::ALL
        .iter()
        .position(|&f| f == ph2d_anim::EasingFamily::Cubic)
        .unwrap();
    #[allow(clippy::cast_possible_truncation)]
    let cubic = cubic as u8;
    assert!(apply_tween_edit(
        &mut w,
        e,
        &TweenFieldEdit::Familia(0, cubic)
    ));
    assert_eq!(
        w.get::<Tweens>(ent).unwrap().0[0].easing.family,
        ph2d_anim::EasingFamily::Cubic
    );

    assert!(apply_tween_edit(&mut w, e, &TweenFieldEdit::Modo(0, 1)));
    assert_eq!(
        w.get::<Tweens>(ent).unwrap().0[0].easing.mode,
        ph2d_anim::EasingMode::ALL[1]
    );

    assert!(apply_tween_edit(
        &mut w,
        e,
        &TweenFieldEdit::AoAcabar(0, AoAcabar::Rewind.tag())
    ));
    assert_eq!(
        w.get::<Tweens>(ent).unwrap().0[0].ao_acabar,
        AoAcabar::Rewind
    );

    assert!(apply_tween_edit(
        &mut w,
        e,
        &TweenFieldEdit::Ciclo(0, ph2d_tween::Ciclo::PingPong.tag())
    ));
    assert_eq!(
        w.get::<Tweens>(ent).unwrap().0[0].ciclo,
        ph2d_tween::Ciclo::PingPong
    );

    assert!(apply_tween_edit(&mut w, e, &TweenFieldEdit::Remove(0)));
    assert!(w.get::<Tweens>(ent).unwrap().0.is_empty());
}

/// ⭐⭐⭐ **O CICLO fecha nos dois sentidos, e o BARRO muda com ele** — o pedido do dono de
/// 2026-09-19 (*«onde estão as opções úteis como ping-pong?»*).
///
/// ⚠️ **A régua é o VALOR que o tween pede ao meio e no fim**, e não a tag: *uma tag lida de uma
/// posição e escrita noutra faria o chip aceso e a lei aplicada serem coisas diferentes, e as duas
/// leituras compilam.*
///
/// **Mutações que devem sangrar:** o dreno do `Ciclo` apagado · o instantâneo a ler sempre
/// `Reinicia`.
#[test]
fn o_ciclo_chega_ao_componente_e_ao_barro() {
    let (mut w, e) = mundo(1, 1, true);
    let ent = Entity::from_bits(e);
    // Um tween de subida, para a diferença ser legível.
    apply_tween_edit(&mut w, e, &TweenFieldEdit::De(0, 0, 0.0));
    apply_tween_edit(&mut w, e, &TweenFieldEdit::Para(0, 0, 1.0));

    let no_fim = |w: &World| {
        let t = w.get::<Tweens>(ent).expect("tem tweens").0[0];
        ph2d_tween::valor(&t, ph2d_tween::Relogio::a_correr(1.0)).expect("escreve")[0]
    };
    // ⛔ O CONTROLO primeiro: de fábrica ele acaba em `para`.
    assert!(
        (no_fim(&w) - 1.0).abs() < 1e-6,
        "controlo: a serra acaba em 1"
    );

    assert!(apply_tween_edit(
        &mut w,
        e,
        &TweenFieldEdit::Ciclo(0, ph2d_tween::Ciclo::PingPong.tag())
    ));
    assert!(
        (no_fim(&w) - 0.0).abs() < 1e-6,
        "com ping-pong ele tem de VOLTAR a `de` no fim do periodo"
    );
    // …e a volta: o instantâneo mostra o chip certo.
    let i = build_tween_info(&w, e, 1).unwrap();
    assert_eq!(i.rows[0].ciclo, ph2d_tween::Ciclo::PingPong.tag());
}

/// ⭐ **A ida-e-volta das TAGS das curvas** — a tradução vive numa porta só, e o gate fecha-a nos
/// dois sentidos sobre as `33`.
///
/// ⚠️ Sem isto, uma tag lida de uma posição e escrita noutra faria o chip aceso e a curva aplicada
/// serem coisas diferentes — *e as duas leituras compilam*.
#[test]
fn a_tag_de_uma_curva_fecha_nos_dois_sentidos() {
    let (mut w, e) = mundo(1, 1, true);
    let ent = Entity::from_bits(e);
    for (fi, _) in ph2d_anim::EasingFamily::ALL.iter().enumerate() {
        for (mi, _) in ph2d_anim::EasingMode::ALL.iter().enumerate() {
            #[allow(clippy::cast_possible_truncation)]
            let (fi, mi) = (fi as u8, mi as u8);
            apply_tween_edit(&mut w, e, &TweenFieldEdit::Familia(0, fi));
            apply_tween_edit(&mut w, e, &TweenFieldEdit::Modo(0, mi));
            let i = build_tween_info(&w, e, 1).unwrap();
            assert_eq!(i.rows[0].familia, fi, "a familia nao fechou");
            assert_eq!(i.rows[0].modo, mi, "o modo nao fechou");
            // …e o controlo: o componente guarda a curva, não a tag.
            let t = w.get::<Tweens>(ent).unwrap().0[0];
            assert_eq!(t.easing.family, ph2d_anim::EasingFamily::ALL[fi as usize]);
        }
    }
}

/// ⚠️ **Um índice fora da lista é um no-op silencioso** — a lista pode ter encolhido entre o quadro
/// que pintou e o que despacha.
#[test]
fn um_indice_fora_da_lista_nao_faz_nada() {
    let (mut w, e) = mundo(1, 1, true);
    for edit in [
        TweenFieldEdit::Remove(9),
        TweenFieldEdit::Canal(9, 1),
        TweenFieldEdit::De(9, 0, 1.0),
        TweenFieldEdit::De(0, 9, 1.0),
    ] {
        assert!(
            !apply_tween_edit(&mut w, e, &edit),
            "{edit:?} mexeu em alguma coisa"
        );
    }
    assert_eq!(w.get::<Tweens>(Entity::from_bits(e)).unwrap().0.len(), 1);
}

/// ⚠️ **Escrever o MESMO valor devolve `false`** — o `bevy` marca a alteração no `deref_mut`, e um
/// componente tocado a cada clique inerte é ruído para quem lê `Changed<…>`.
#[test]
fn escrever_o_mesmo_valor_nao_conta_como_mudanca() {
    let (mut w, e) = mundo(1, 1, true);
    let t = w.get::<Tweens>(Entity::from_bits(e)).unwrap().0[0];
    assert!(!apply_tween_edit(
        &mut w,
        e,
        &TweenFieldEdit::Canal(0, t.canal.tag())
    ));
    assert!(!apply_tween_edit(
        &mut w,
        e,
        &TweenFieldEdit::De(0, 0, t.de[0])
    ));
    assert!(!apply_tween_edit(
        &mut w,
        e,
        &TweenFieldEdit::AoAcabar(0, t.ao_acabar.tag())
    ));
}

/// ⭐⭐⭐ **UM CLIQUE NUM PRESET DEIXA O PAINEL SEM QUEIXA** — e o gate mede a PORTA da queixa, que
/// é o que o artista lê.
///
/// ⚠️ **É a régua certa e não «os campos batem»:** um preset que escrevesse `de == para`, ou o
/// `Flash` com `Hold`, produziria um tween que o painel acusa no mesmo instante — *e um botão que
/// deixa um aviso aceso lê-se como um botão partido*.
///
/// ⛔ E a metade que a torna honesta é o CONTROLO: o tween de fábrica **tem** queixa (ele nasce com
/// um canal de aparência e a cena do gate não tem sprite), senão isto passaria por vacuidade.
#[test]
fn um_preset_deixa_o_painel_sem_queixa() {
    for p in ph2d_tween::Preset::ALL {
        let (mut w, e) = mundo(1, 1, true);
        assert!(apply_tween_edit(
            &mut w,
            e,
            &TweenFieldEdit::Preset(0, p.tag())
        ));
        let i = build_tween_info(&w, e, 1).unwrap();
        assert_eq!(
            i.rows[0].queixa(i.tem_sprite),
            None,
            "o preset `{}` deixou uma queixa",
            p.label()
        );
    }
    // ⛔ O CONTROLO: sem sprite, o MESMO preset de aparência queixa-se — a porta está viva.
    let (mut w, e) = mundo(1, 1, false);
    apply_tween_edit(
        &mut w,
        e,
        &TweenFieldEdit::Preset(0, ph2d_tween::Preset::Flash.tag()),
    );
    let i = build_tween_info(&w, e, 1).unwrap();
    assert!(
        i.rows[0].queixa(i.tem_sprite).is_some(),
        "controlo: sem sprite a queixa TEM de existir"
    );
}

/// ⭐⭐⭐ **E ele escreve TAMBÉM a duração do relógio** — é isso que o faz um clique em vez de dois.
///
/// ⚠️ Sem esta metade o artista fica com um *flash* de **um segundo** (o valor de fábrica do
/// timer), oito vezes mais lento do que a coisa que ele pediu, e lê isso como *«o preset não
/// funcionou»*. **A mutação que apaga a escrita do timer tem de sangrar aqui.**
#[test]
fn um_preset_escreve_tambem_a_duracao_do_relogio() {
    let (mut w, e) = mundo(1, 1, true);
    let ent = Entity::from_bits(e);
    let antes = w.get::<Timers>(ent).unwrap().0[0].duration_us;
    assert_eq!(
        antes, 1_000_000,
        "controlo: o timer de fabrica dura um segundo"
    );

    apply_tween_edit(
        &mut w,
        e,
        &TweenFieldEdit::Preset(0, ph2d_tween::Preset::Flash.tag()),
    );
    assert_eq!(
        w.get::<Timers>(ent).unwrap().0[0].duration_us,
        ph2d_tween::Preset::Flash.duracao_us(),
        "o preset nao escreveu a duracao"
    );
    // …e um preset diferente escreve outra: o gate distingue as DUAS leis.
    apply_tween_edit(
        &mut w,
        e,
        &TweenFieldEdit::Preset(0, ph2d_tween::Preset::FadeOut.tag()),
    );
    assert_eq!(
        w.get::<Timers>(ent).unwrap().0[0].duration_us,
        ph2d_tween::Preset::FadeOut.duracao_us()
    );
}

/// ⚠️ **Um preset sem timer no mesmo índice escreve o TWEEN e cala-se sobre o relógio** — e ainda
/// conta como mudança, porque o tween mudou.
#[test]
fn um_preset_sem_relogio_escreve_o_que_pode() {
    let (mut w, e) = mundo(1, 0, true);
    assert!(apply_tween_edit(
        &mut w,
        e,
        &TweenFieldEdit::Preset(0, ph2d_tween::Preset::Flash.tag())
    ));
    assert_eq!(
        w.get::<Tweens>(Entity::from_bits(e)).unwrap().0[0].canal,
        Canal::Silhueta
    );
}
