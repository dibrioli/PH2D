use super::*;

/// ⭐⭐ Cada escolha pede o que a caixa de antes pedia, e a escolhida não pede nada (re-armar o Loop
/// repunha a faixa dele). *Mutação: tirar o `agora == quer` ⇒ o Loop aceso pede outra vez.*
#[test]
fn cada_escolha_do_fim_pede_o_toggle_de_antes() {
    use Fim::*;
    assert_eq!(toggle_que_pede(Laco, Laco), None);
    assert_eq!(
        toggle_que_pede(Para, Laco),
        Some((ids::TIMELINE_LOOP, true))
    );
    assert_eq!(
        toggle_que_pede(Laco, PingPong),
        Some((ids::TIMELINE_PINGPONG, true))
    );
    assert_eq!(
        toggle_que_pede(Laco, Para),
        Some((ids::TIMELINE_LOOP, false))
    );
    assert_eq!(
        toggle_que_pede(PingPong, Para),
        Some((ids::TIMELINE_PINGPONG, false))
    );
}

/// ⭐ A roda (dono 13/11): com Loop escolhido, Stop à esquerda e Ping-Pong à direita; as setinhas dão
/// a volta. *Mutação: `rem_euclid` → `%` ⇒ a esquerda do Stop sai do índice.*
#[test]
fn as_vizinhas_do_fim_dao_a_volta() {
    use Fim::*;
    assert_eq!((Laco.vizinha(-1), Laco.vizinha(1)), (Para, PingPong));
    assert_eq!((Para.vizinha(-1), Para.vizinha(1)), (PingPong, Laco));
    assert_eq!((PingPong.vizinha(-1), PingPong.vizinha(1)), (Laco, Para));
    for f in RODA {
        assert_eq!(f.fenda_de(f), 0);
        assert_eq!(f.vizinha(-1).fenda_de(f), -1);
        assert_eq!(f.vizinha(1).fenda_de(f), 1);
    }
}

/// ⭐⭐ A troca anima: a meio caminho cada ícone está entre as duas fendas; no fim cada um está na
/// fenda nova, inteiro. A que dá a volta passa por transparente. *Mutação: `fenda` devolver sempre a
/// fenda nova ⇒ o meio não está entre as duas.*
#[test]
fn a_troca_desliza_cada_icone_entre_as_fendas() {
    use Fim::*;
    // Stop → Loop (a da direita): todos andam uma fenda para a esquerda.
    let de_para = (Para, Laco);
    assert_eq!(fenda(Laco, de_para, 0.0), (1.0, 1.0));
    let (s, a) = fenda(Laco, de_para, 0.5);
    assert!((s - 0.5).abs() < 1e-6 && (a - 1.0).abs() < 1e-6);
    assert_eq!(fenda(Laco, de_para, 1.0), (0.0, 1.0));
    assert_eq!(fenda(Para, de_para, 1.0), (-1.0, 1.0));
    // O Ping-Pong estava à esquerda do Stop e vai para a direita do Loop: sai e volta a entrar.
    let (s, a) = fenda(PingPong, de_para, 0.25);
    assert!(s < -1.0 && a < 1.0, "sai pela esquerda: {s} {a}");
    let (s, a) = fenda(PingPong, de_para, 0.75);
    assert!(s > 1.0 && a < 1.0, "entra pela direita: {s} {a}");
    assert_eq!(fenda(PingPong, de_para, 1.0), (1.0, 1.0));
}

/// ⭐ Movimento reduzido ⇒ a troca é seca; sem ele, começa em 0 e acaba em 1 no tempo do token.
/// *Mutação: ignorar `reduzido` ⇒ vermelho.*
#[test]
fn a_troca_e_seca_com_movimento_reduzido() {
    let t = Troca {
        de: Fim::Para,
        para: Fim::Laco,
        t0: 1_000,
    };
    assert_eq!(andamento(t, 1_000, true), 1.0);
    assert_eq!(andamento(t, 1_000, false), 0.0);
    let fim = 1_000 + ph2d_tokens::Duration::Default.ms() as u128 * 1_000_000;
    assert_eq!(andamento(t, fim, false), 1.0);
}
