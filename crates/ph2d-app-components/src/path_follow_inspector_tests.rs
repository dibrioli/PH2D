//! Os gates do instantâneo e do dreno da secção PATH FOLLOW (suplente #23).

use super::*;
use ph2d_ecs::{Name, Timer, Transform, VecPathRef};

/// Uma cena: um seguidor, e (opcionalmente) uma forma desenhada chamada `Trilho`.
fn cena(pf: PathFollow, com_forma: bool, n_timers: usize) -> (World, u64) {
    let mut w = World::new();
    if com_forma {
        w.spawn((Name::new("Trilho"), Transform::IDENTITY, VecPathRef(7)));
    }
    let cfg = Timers(
        (0..n_timers)
            .map(|_| Timer {
                duration_us: 1_000_000,
                ..Timer::default()
            })
            .collect(),
    );
    let rt = ph2d_ecs::TimerRuntime(cfg.0.iter().map(ph2d_ecs::timer::born).collect());
    let e = w.spawn((Transform::IDENTITY, pf, cfg, rt)).id();
    (w, e.to_bits())
}

fn segue(nome: &str) -> PathFollow {
    PathFollow {
        caminho: nome.into(),
        ..PathFollow::default()
    }
}

/// ⭐ **As três colunas que não vêm do componente** — e são elas que fazem a queixa.
#[test]
fn o_instantaneo_traz_o_que_o_componente_nao_sabe() {
    let (mut w, b) = cena(segue("Trilho"), true, 1);
    let i = build_path_follow_info(&mut w, b, 1, false).unwrap();
    assert!(i.nome_existe && i.nome_tem_forma);
    assert_eq!(i.duracao_us, Some(1_000_000));
    assert_eq!(i.queixa(), None);
}

/// ⛔ **Um objecto com o nome mas SEM forma desenhada** — a queixa que manda apontar a uma curva.
#[test]
fn um_nome_sem_forma_diz_que_falta_a_forma() {
    let mut w = World::new();
    w.spawn((Name::new("Trilho"), Transform::IDENTITY)); // sem `VecPathRef`
    let cfg = Timers(vec![Timer {
        duration_us: 1_000_000,
        ..Timer::default()
    }]);
    let rt = ph2d_ecs::TimerRuntime(cfg.0.iter().map(ph2d_ecs::timer::born).collect());
    let b = w
        .spawn((Transform::IDENTITY, segue("Trilho"), cfg, rt))
        .id()
        .to_bits();
    let i = build_path_follow_info(&mut w, b, 1, false).unwrap();
    assert!(i.nome_existe, "o objecto existe");
    assert!(!i.nome_tem_forma);
    assert_eq!(
        i.queixa(),
        Some(ph2d_editor_core::path_follow_edits::PathFollowQueixa::SemForma)
    );
}

/// ⛔ **Um índice sem relógio** — a queixa que manda ao índice, e o CONTROLO é o índice válido.
#[test]
fn um_indice_sem_relogio_diz_qual() {
    let (mut w, b) = cena(
        PathFollow {
            relogio: 3,
            ..segue("Trilho")
        },
        true,
        1,
    );
    let i = build_path_follow_info(&mut w, b, 1, false).unwrap();
    assert_eq!(i.duracao_us, None);
    assert_eq!(
        i.queixa(),
        Some(ph2d_editor_core::path_follow_edits::PathFollowQueixa::SemRelogio)
    );
    assert_eq!(i.relogios, 1, "e o painel sabe QUANTOS há");
}

/// ⚠️ **Sem o componente não há secção** — ADR-0166.
#[test]
fn sem_o_componente_nao_ha_seccao() {
    let mut w = World::new();
    let b = w.spawn((Transform::IDENTITY,)).id().to_bits();
    assert!(build_path_follow_info(&mut w, b, 1, false).is_none());
}

/// ⭐⭐⭐ **O ÍNDICE do relógio satura no tecto do MODELO** — *uma cerca só na UI é uma cerca que o
/// próximo chamador contorna*.
#[test]
fn o_indice_do_relogio_satura_no_tecto_do_modelo() {
    let (mut w, b) = cena(segue("Trilho"), true, 1);
    apply_path_follow_edit(&mut w, b, &PathFollowFieldEdit::Relogio(200));
    let e = Entity::from_bits(b);
    assert_eq!(
        usize::from(w.get::<PathFollow>(e).unwrap().relogio),
        ph2d_ecs::TIMERS_MAX - 1
    );
}

/// ⚠️ **A fracção de entrada satura em `0..1`** — fora dela dois pontos do slider dariam a MESMA
/// posição na pista, porque a lei dá a volta.
#[test]
fn a_fraccao_de_entrada_satura() {
    let (mut w, b) = cena(segue("Trilho"), true, 1);
    let e = Entity::from_bits(b);
    apply_path_follow_edit(&mut w, b, &PathFollowFieldEdit::Deslocamento(3.5));
    assert_eq!(w.get::<PathFollow>(e).unwrap().deslocamento, 1.0);
    apply_path_follow_edit(&mut w, b, &PathFollowFieldEdit::Deslocamento(-2.0));
    assert_eq!(w.get::<PathFollow>(e).unwrap().deslocamento, 0.0);
}

/// ⭐⭐⭐ **A POSIÇÃO no `ALL` É a tag, e o gate mede a IDA E A VOLTA** — sem a volta, uma tabela
/// que traduzisse duas tags para o mesmo valor passaria.
#[test]
fn a_tag_de_um_chip_e_a_posicao_no_all_nos_dois_sentidos() {
    let (mut w, b) = cena(segue("Trilho"), true, 1);
    let e = Entity::from_bits(b);
    for (t, esperado) in ph2d_tween::Ciclo::ALL
        .iter()
        .enumerate()
        .map(|(i, c)| (i, *c))
    {
        #[allow(clippy::cast_possible_truncation)]
        apply_path_follow_edit(&mut w, b, &PathFollowFieldEdit::Ciclo(t as u8));
        assert_eq!(w.get::<PathFollow>(e).unwrap().ciclo, esperado);
        let i = build_path_follow_info(&mut w, b, 1, false).unwrap();
        #[allow(clippy::cast_possible_truncation)]
        let voltou = t as u8;
        assert_eq!(i.ciclo, voltou, "a volta trouxe outra tag");
    }
    for (t, esperado) in ph2d_anim::EasingFamily::ALL.iter().enumerate() {
        #[allow(clippy::cast_possible_truncation)]
        apply_path_follow_edit(&mut w, b, &PathFollowFieldEdit::Familia(t as u8));
        assert_eq!(w.get::<PathFollow>(e).unwrap().easing.family, *esperado);
        let i = build_path_follow_info(&mut w, b, 1, false).unwrap();
        #[allow(clippy::cast_possible_truncation)]
        let voltou = t as u8;
        assert_eq!(i.familia, voltou);
    }
}

/// ⛔ **Uma tag fora do `ALL` é um no-op silencioso** — o painel e o motor podem estar um quadro
/// dessincronizados, e um pânico ali derrubaria o app.
#[test]
fn uma_tag_fora_do_all_nao_muda_nada_e_nao_estoura() {
    let (mut w, b) = cena(segue("Trilho"), true, 1);
    let e = Entity::from_bits(b);
    let antes = w.get::<PathFollow>(e).unwrap().clone();
    assert!(!apply_path_follow_edit(
        &mut w,
        b,
        &PathFollowFieldEdit::Ciclo(200)
    ));
    assert_eq!(*w.get::<PathFollow>(e).unwrap(), antes);
}
