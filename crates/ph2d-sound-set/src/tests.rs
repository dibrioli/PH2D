use super::*;

fn sons(pesos: &[f32]) -> Vec<Candidate> {
    pesos
        .iter()
        .map(|&weight| Candidate {
            weight,
            enabled: true,
        })
        .collect()
}

/// Com o peso quase todo num som só, um sorteio que refaz «algumas vezes» repetiria: a fixture tem o fenômeno.
#[test]
fn mil_disparos_nunca_repetem_o_ultimo() {
    let casos = [sons(&[1.0, 1.0, 1.0]), sons(&[16.0, 0.01, 0.01])];
    for c in &casos {
        for strategy in [PickStrategy::Random, PickStrategy::Shuffle] {
            let mut p = Picker::with_seed(7);
            let mut antes = None;
            for n in 0..1000 {
                let i = p.pick(c, strategy, true).expect("há sons ligados");
                assert_ne!(Some(i), antes, "{strategy:?} repetiu no disparo {n}");
                antes = Some(i);
            }
        }
    }
}

/// O controle do teste acima: sem `no_repeat`, o mesmo sorteio repete.
#[test]
fn sem_no_repeat_o_sorteio_repete() {
    let c = sons(&[16.0, 0.01, 0.01]);
    let mut p = Picker::with_seed(7);
    let mut antes = None;
    let mut repetiu = 0;
    for _ in 0..1000 {
        let i = p.pick(&c, PickStrategy::Random, false);
        repetiu += i32::from(i == antes);
        antes = i;
    }
    assert!(repetiu > 900, "repetiu {repetiu} vezes");
}

#[test]
fn shuffle_toca_todos_antes_de_repetir() {
    let c = sons(&[1.0; 5]);
    let mut p = Picker::with_seed(11);
    for volta in 0..200 {
        let mut vistos = [0u32; 5];
        for _ in 0..5 {
            vistos[p.pick(&c, PickStrategy::Shuffle, false).expect("ligados")] += 1;
        }
        assert_eq!(vistos, [1; 5], "volta {volta}");
    }
}

/// Com `no_repeat` a costura entre duas voltas pode adiar um som, mas nenhum some: em mil disparos cada um
/// dos três sai quase um terço das vezes.
#[test]
fn shuffle_sem_repetir_mantem_os_tres_equilibrados() {
    let c = sons(&[1.0; 3]);
    let mut p = Picker::with_seed(3);
    let mut n = [0i32; 3];
    for _ in 0..1000 {
        n[p.pick(&c, PickStrategy::Shuffle, true).expect("ligados")] += 1;
    }
    for x in n {
        assert!((x - 333).abs() <= 34, "contagem {n:?}");
    }
}

#[test]
fn sequence_segue_a_lista_e_pula_o_desligado() {
    let mut c = sons(&[1.0; 3]);
    let mut p = Picker::with_seed(1);
    let s: Vec<_> = (0..6)
        .map(|_| p.pick(&c, PickStrategy::Sequence, false).expect("ligados"))
        .collect();
    assert_eq!(s, [0, 1, 2, 0, 1, 2]);
    c[1].enabled = false;
    let mut p = Picker::with_seed(1);
    let s: Vec<_> = (0..4)
        .map(|_| p.pick(&c, PickStrategy::Sequence, true).expect("ligados"))
        .collect();
    assert_eq!(s, [0, 2, 0, 2]);
}

#[test]
fn um_som_so_repete_e_nada_ligado_nao_toca() {
    let mut c = sons(&[1.0]);
    let mut p = Picker::with_seed(1);
    for _ in 0..3 {
        assert_eq!(p.pick(&c, PickStrategy::Shuffle, true), Some(0));
    }
    c[0].enabled = false;
    assert_eq!(p.pick(&c, PickStrategy::Shuffle, true), None);
}

#[test]
fn desligar_um_som_a_meio_do_baralho_tira_ele_dali() {
    let mut c = sons(&[1.0; 4]);
    let mut p = Picker::with_seed(5);
    let _ = p.pick(&c, PickStrategy::Shuffle, true);
    c[2].enabled = false;
    for _ in 0..100 {
        assert_ne!(p.pick(&c, PickStrategy::Shuffle, true), Some(2));
    }
}

#[test]
fn a_mesma_semente_da_a_mesma_sequencia() {
    let c = sons(&[1.0, 2.0, 3.0]);
    let mut a = Picker::with_seed(42);
    let mut b = Picker::with_seed(42);
    for _ in 0..100 {
        assert_eq!(
            a.pick(&c, PickStrategy::Random, true),
            b.pick(&c, PickStrategy::Random, true)
        );
        assert_eq!(a.jitter(3.0, 2.0), b.jitter(3.0, 2.0));
    }
}

#[test]
fn a_variacao_fica_dentro_do_pedido_e_zero_e_identidade() {
    let mut p = Picker::with_seed(9);
    assert_eq!(p.jitter(0.0, 0.0), Jitter::NONE);
    let (mut sobe, mut desce) = (false, false);
    for _ in 0..1000 {
        let j = p.jitter(12.0, 6.0);
        assert!((0.5 - 1e-4..=2.0 + 1e-4).contains(&j.pitch), "{j:?}");
        assert!((0.5..=2.0).contains(&j.gain), "{j:?}");
        sobe |= j.pitch > 1.5;
        desce |= j.pitch < 0.7;
    }
    assert!(sobe && desce, "a variação tem de usar a faixa");
}

#[test]
fn o_formato_gravado_volta_igual() {
    let s = SoundSet {
        sounds: vec![SoundEntry::new("a.wav"), SoundEntry::new("b.wav")],
        strategy: PickStrategy::Sequence,
        no_repeat: false,
        pitch_jitter_semitones: 2.0,
        gain_jitter_db: 1.5,
    };
    let bytes = postcard::to_allocvec(&s).expect("grava");
    assert_eq!(postcard::from_bytes::<SoundSet>(&bytes).expect("lê"), s);
}

/// ⛔ O postcard guarda a posição da variante: trocar a ordem muda o modo de todo projeto gravado.
#[test]
fn a_ordem_das_estrategias_e_o_formato() {
    let tag = |s| postcard::to_allocvec(&s).expect("grava")[0];
    assert_eq!(tag(PickStrategy::Random), 0);
    assert_eq!(tag(PickStrategy::Sequence), 1);
    assert_eq!(tag(PickStrategy::Shuffle), 2);
}

#[test]
fn sanitize_cura_numero_de_arquivo() {
    let mut s = SoundSet {
        sounds: vec![SoundEntry {
            name: "  a.wav ".into(),
            weight: f32::NAN,
            enabled: true,
        }],
        pitch_jitter_semitones: 1e9,
        gain_jitter_db: f32::INFINITY,
        ..SoundSet::default()
    };
    s.sanitize();
    assert_eq!(s.sounds[0].name, "a.wav");
    assert_eq!(s.sounds[0].weight, 1.0);
    assert_eq!(s.pitch_jitter_semitones, MAX_JITTER);
    assert_eq!(s.gain_jitter_db, 0.0);
}

#[test]
fn nome_vazio_nao_conta_como_som() {
    let mut s = SoundSet::default();
    assert!(!s.has_sounds());
    s.sounds.push(SoundEntry::new("  "));
    assert!(!s.has_sounds());
    s.sounds.push(SoundEntry::new("a.wav"));
    assert!(s.has_sounds());
}
