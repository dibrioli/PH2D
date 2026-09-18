//! Os gates da CENA do gatilho — ela tem de ensinar o que diz que ensina.

use super::*;

fn monta() -> World {
    let mut w = World::new();
    assert_eq!(montar(&mut w, 1).nivel, 1);
    w
}

fn acha(w: &mut World, nome: &str) -> Entity {
    let mut q = w.query::<(Entity, &Name)>();
    q.iter(w)
        .find(|(_, n)| n.0 == nome)
        .map(|(e, _)| e)
        .unwrap_or_else(|| panic!("a cena não montou «{nome}»"))
}

/// ⭐⭐⭐ **As duas armas diferem em UMA COISA SÓ** — e é isso que faz o controlo controlar.
///
/// ⚠️ **Sem este gate a cena podia divergir em silêncio** (outra receita, outra rajada, outro
/// sinal) e o dono leria a diferença das balas como sendo da mira. *Um controlo que difere em duas
/// coisas não é um controlo.*
#[test]
fn as_duas_armas_diferem_so_na_mira() {
    let mut w = monta();
    let h = acha(&mut w, "Heroi");
    let t = acha(&mut w, "Torreta (sem mira)");
    let (fh, ft) = (
        w.get::<Factory>(h).expect("o herói tem fábrica").clone(),
        w.get::<Factory>(t).expect("a torreta tem fábrica").clone(),
    );
    assert!(fh.aim_from_spawner, "o herói tem de apontar");
    assert!(!ft.aim_from_spawner, "a torreta é o CONTROLO: não aponta");
    assert_eq!(fh.on_signal, ft.on_signal, "as duas ouvem o MESMO sinal");
    assert_eq!(fh.burst, ft.burst, "a rajada tem de ser igual");
    assert_ne!(
        fh.master, 0,
        "a receita do herói ficou por resolver — a fábrica não nasce nada"
    );
    assert_ne!(ft.master, 0, "a receita da torreta ficou por resolver");
    assert_ne!(
        fh.master, ft.master,
        "as duas balas são receitas diferentes (só a cor muda), senão a do controlo não se distingue"
    );
}

/// ⚠️⚠️ **A torreta está RODADA, e sem isso o controlo não controla:** com a rotação a zero,
/// *«herda a rotação da fábrica»* e *«fica com a do molde»* dariam a MESMA imagem.
#[test]
fn a_torreta_esta_rodada_senao_o_controlo_nao_controla() {
    let mut w = monta();
    let t = acha(&mut w, "Torreta (sem mira)");
    let r = w.get::<Transform>(t).expect("pose").rotation;
    assert!(
        r.abs() > 0.1,
        "a torreta tem de estar rodada para o controlo ser observável, e está a {r} rad"
    );
}

/// ⭐ **Os dois gatilhos ouvem a MESMA acção e dizem o MESMO sinal** — e os dois nomes saem das
/// consts que o prólogo da shell também lê.
///
/// ⚠️ Sem esta amarra, a cena liga uma tecla a uma acção e o gatilho ouve outra — e o sintoma é
/// exactamente o de uma ferramenta partida.
#[test]
fn os_dois_gatilhos_leem_a_mesma_accao_e_o_mesmo_sinal() {
    let mut w = monta();
    for nome in ["Heroi", "Torreta (sem mira)"] {
        let e = acha(&mut w, nome);
        let g = w.get::<SignalOnAction>(e).expect("tem gatilho");
        assert_eq!(g.0.len(), 1, "«{nome}» tem de ter UMA linha");
        assert_eq!(g.0[0].action, ACCAO, "«{nome}» ouve a acção errada");
        assert_eq!(g.0[0].signal, SINAL, "«{nome}» diz o sinal errado");
        assert_eq!(
            g.0[0].edge,
            ActionEdge::Press,
            "⚠️ o `Hold` daria uma rajada por quadro — a cena quer UM tiro por toque"
        );
        let f = w.get::<Factory>(e).expect("tem fábrica");
        assert_eq!(
            f.on_signal, SINAL,
            "a fábrica de «{nome}» ouve um sinal que ninguém publica"
        );
    }
}

/// ⚠️ **O CHÃO nasce PRIMEIRO** — desde a cura de 15/09 a ordem das raízes é a ordem de CRIAÇÃO,
/// logo quem nasce primeiro desenha por baixo. *Antes era o contrário, e ninguém sabia.*
#[test]
fn o_chao_nasce_antes_de_tudo() {
    let mut w = monta();
    let chao = acha(&mut w, "Ground");
    let heroi = acha(&mut w, "Heroi");
    assert!(
        chao.index() < heroi.index(),
        "o chão tem de nascer antes do herói, senão ele desenha por cima"
    );
}

/// ⭐ **A bala MORRE** — sem o `Lifetime` a cena enche-se de balas paradas no fim do alcance.
#[test]
fn a_bala_tem_higiene_de_ciclo_de_vida() {
    let mut w = monta();
    let b = acha(&mut w, "Bala");
    let l = w
        .get::<Lifetime>(b)
        .expect("a receita tem de ter `Lifetime`");
    assert!(l.duration_us > 0, "uma vida de `0` NÃO mata (a lei do #12)");
}

/// ⭐⭐⭐ **Quem nasce ESCOLHIDO é o HERÓI, e ele tem o componente que o roteiro manda ver.**
///
/// ⚠️ **A foto é que exigiu este gate:** a cena abria com o Inspector à frente e **ninguém
/// escolhido**, logo o painel dizia *«Select an entity in the Hierarchy»* e o passo que manda ver a
/// secção *Trigger* nomeava uma superfície que o dono não tinha. *Um passo que nomeia uma secção
/// AFIRMA que ela está na tela* — a lei que o `#15` pagou com um report do dono.
///
/// ⛔ E ele afirma as DUAS metades: sem a segunda, apontar a selecção ao CHÃO passaria.
#[test]
fn quem_nasce_escolhido_e_o_heroi_e_ele_tem_gatilho() {
    let mut w = World::new();
    let m = super::montar(&mut w, 1);
    let e = Entity::from_bits(m.escolhido);
    assert_eq!(
        w.get::<Name>(e).map(|n| n.0.clone()).as_deref(),
        Some("Heroi"),
        "o escolhido tem de ser o heroi"
    );
    assert!(
        w.get::<SignalOnAction>(e).is_some_and(|g| !g.0.is_empty()),
        "o escolhido tem de ter a seccao que o roteiro manda ver"
    );
}
