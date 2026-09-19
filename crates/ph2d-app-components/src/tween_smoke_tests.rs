//! Os gates da cena do TWEEN — o que ela tem de CONTER para ensinar o que promete.
//!
//! ⛔ O que nenhum deles mede é o que o dono vê: o gesto é dele, e a foto é do
//! `docs/Components/ferramentas/fotografa_cena.sh`. O que estes fecham é a metade **construível**
//! — *uma cena que ensina o contrário do que acontece é pior que uma cena ausente* (§5.0).

use super::*;

fn mundo_um() -> World {
    let mut w = World::new();
    montar(&mut w, 1);
    w
}

fn mundo_dois() -> World {
    let mut w = World::new();
    montar(&mut w, 2);
    w
}

fn por_nome(w: &mut World, nome: &str) -> Entity {
    let mut q = w.query::<(Entity, &Name)>();
    q.iter(w)
        .find(|(_, n)| n.0 == nome)
        .map(|(e, _)| e)
        .unwrap_or_else(|| panic!("a cena nao montou «{nome}»"))
}

/// ⭐⭐⭐ **Toda coluna tem tantos tweens quantos relógios** — a lei do módulo, medida na CENA.
///
/// ⚠️ **É a metade que um `Tweens` sozinho não afirma:** o tween `i` corre no timer `i`, logo uma
/// coluna com dois tweens e um relógio tem o segundo **inerte e calado**. *Um tween sem relógio e um
/// tween parado leem-se exactamente igual na tela.*
#[test]
fn cada_coluna_tem_um_relogio_por_tween() {
    let mut w = mundo_um();
    let mut q = w.query::<(&Name, &Tweens, &Timers)>();
    let colunas: Vec<(String, usize, usize)> = q
        .iter(&w)
        .map(|(n, tw, ti)| (n.0.clone(), tw.0.len(), ti.0.len()))
        .collect();
    assert!(
        colunas.len() >= 4,
        "a galeria tem de ter as QUATRO colunas: {colunas:?}"
    );
    for (nome, tweens, timers) in &colunas {
        assert_eq!(
            tweens, timers,
            "«{nome}» tem {tweens} tweens e {timers} relogios"
        );
        assert!(*tweens >= 1, "«{nome}» nao tem tween nenhum");
    }
}

/// ⭐⭐ **Os quatro canais da galeria são QUATRO canais diferentes** — é isso que a torna uma
/// galeria em vez de quatro cópias do mesmo.
///
/// ⚠️ **A régua é o CONJUNTO e não uma lista escrita à mão:** ela reprova no dia em que duas colunas
/// passarem a demonstrar a mesma coisa, sem depender de alguém se lembrar de a estender.
#[test]
fn a_galeria_mostra_canais_distintos() {
    let mut w = mundo_um();
    let mut q = w.query::<&Tweens>();
    let primeiros: Vec<Canal> = q
        .iter(&w)
        .filter_map(|t| t.0.first().map(|x| x.canal))
        .collect();
    let mut vistos: Vec<Canal> = primeiros.clone();
    vistos.sort_by_key(|c| c.tag());
    vistos.dedup_by_key(|c| c.tag());
    assert_eq!(
        vistos.len(),
        primeiros.len(),
        "duas colunas demonstram o mesmo canal: {primeiros:?}"
    );
}

/// ⭐⭐⭐ **A quarta coluna tem DOIS relógios de períodos DIFERENTES** — e é ela que torna a lei do
/// índice visível.
///
/// ⛔ **Com o mesmo período o quadrado cresce por igual**, e a coluna passa a ler-se como um tween
/// só: a cena continuaria a montar e deixaria de ensinar. *É a espécie do §5.0 — ensinar o contrário
/// é pior do que não ensinar nada.*
#[test]
fn a_coluna_do_crescer_tem_dois_ritmos() {
    let mut w = mundo_um();
    let e = por_nome(&mut w, "Cresce (Scale X e Y)");
    let timers = w.get::<Timers>(e).expect("a coluna tem relogios").clone();
    assert_eq!(timers.0.len(), 2, "a coluna do crescer tem DOIS relogios");
    assert_ne!(
        timers.0[0].duration_us, timers.0[1].duration_us,
        "os dois relogios tem de ter periodos DIFERENTES, senao a lei do indice nao se ve^"
    );
    let canais: Vec<Canal> = w
        .get::<Tweens>(e)
        .expect("a coluna tem tweens")
        .0
        .iter()
        .map(|t| t.canal)
        .collect();
    assert_eq!(canais, vec![Canal::ScaleX, Canal::ScaleY]);
}

/// ⭐⭐ **As duas colunas que o passo (4) compara têm `On finish` DIFERENTES.**
///
/// ⚠️ **O roteiro manda desligar o `Repeat` e ver a diferença** — se as duas tiverem o mesmo fim, o
/// passo pede ao dono que veja uma coisa que não acontece, e ele reporta um defeito que não existe.
#[test]
fn o_passo_quatro_tem_os_dois_fins_para_comparar() {
    let mut w = mundo_um();
    let aparece = por_nome(&mut w, "Aparece (Opacity)");
    let pisca = por_nome(&mut w, "Pisca (Silhueta)");
    let fim = |w: &World, e: Entity| w.get::<Tweens>(e).expect("tem tween").0[0].ao_acabar;
    assert_eq!(fim(&w, aparece), AoAcabar::Hold);
    assert_eq!(fim(&w, pisca), AoAcabar::Rewind);
}

/// ⭐⭐ **Toda coluna NASCE a correr e em LAÇO** — senão a cena abre parada.
///
/// ⚠️ Um `autostart: false` deixa quatro quadrados imóveis com a suíte inteira verde, e o dono lê
/// *«o tween não funciona»* sobre um componente certo.
#[test]
fn a_galeria_abre_a_correr() {
    let mut w = mundo_um();
    let mut q = w.query::<(&Name, &Timers)>();
    for (n, ti) in q.iter(&w) {
        for t in &ti.0 {
            assert!(t.autostart, "«{}» / «{}» nao arranca sozinho", n.0, t.name);
            assert!(t.repeat, "«{}» / «{}» nao repete", n.0, t.name);
            assert!(
                t.signal.is_empty(),
                "«{}» / «{}» PUBLICA um sinal — o relogio de um tween e' calado",
                n.0,
                t.name
            );
        }
    }
}

/// ⭐⭐⭐ **A `=2` tem as DUAS fábricas, e a diferença entre elas é SÓ o tween.**
///
/// ⛔ Sem o controlo a cena não ensina nada: *uma cópia que entra suave, sozinha, não se distingue
/// de uma cópia que aparece*. E se as receitas diferissem noutro campo, o dono atribuiria a
/// diferença à coisa errada.
#[test]
fn a_copia_e_o_controlo_diferem_so_no_tween() {
    let mut w = mundo_dois();
    let com = por_nome(&mut w, "Copia (com tween)");
    let sem = por_nome(&mut w, "Copia (controlo)");
    assert!(w.get::<Tweens>(com).is_some_and(|t| !t.0.is_empty()));
    assert!(
        w.get::<Tweens>(sem).is_none(),
        "o CONTROLO nao pode ter tweens — e' a unica diferenca da cena"
    );
    let vida = |w: &World, e: Entity| w.get::<Lifetime>(e).expect("tem vida").duration_us;
    assert_eq!(
        vida(&w, com),
        vida(&w, sem),
        "as duas receitas tem de viver o mesmo tempo"
    );
    let tamanho = |w: &World, e: Entity| w.get::<Sprite>(e).expect("tem sprite").size;
    assert_eq!(tamanho(&w, com), tamanho(&w, sem));
}

/// ⭐⭐ **As duas fábricas apontam a receitas DIFERENTES e correm ao MESMO ritmo.**
///
/// ⛔ Se apontassem à mesma, os dois lados seriam iguais e o controlo desaparecia em silêncio — e
/// `master: 0` é o estado de MONTAGEM: uma fábrica que fique nele não produz nada.
#[test]
fn as_duas_fabricas_correm_ao_mesmo_ritmo_com_receitas_diferentes() {
    let mut w = mundo_dois();
    let mut q = w.query::<(&Name, &Factory, &Timers)>();
    let fabs: Vec<(String, u64, u64)> = q
        .iter(&w)
        .map(|(n, f, t)| (n.0.clone(), f.master, t.0[0].duration_us))
        .collect();
    assert_eq!(fabs.len(), 2, "a `=2` tem DUAS fabricas: {fabs:?}");
    assert_ne!(fabs[0].1, fabs[1].1, "as duas apontam a` MESMA receita");
    assert!(
        fabs.iter().all(|f| f.1 != 0),
        "uma fabrica ficou por resolver: {fabs:?}"
    );
    assert_eq!(fabs[0].2, fabs[1].2, "as duas tem de correr ao mesmo ritmo");
}

/// ⚠️ **O `montar` de um nível que não existe cai na `=1`** — e o gate afirma-o, porque o roteador
/// da família declara `CENAS` e um número acima dele tem de dar uma cena e não um ecrã vazio.
#[test]
fn o_roteador_declara_exactamente_as_cenas_que_existem() {
    assert_eq!(CENAS, 2);
    let mut w = World::new();
    assert_eq!(montar(&mut w, 1).nivel, 1);
    let mut w = World::new();
    assert_eq!(montar(&mut w, 2).nivel, 2);
    let mut w = World::new();
    assert_eq!(
        montar(&mut w, 99).nivel,
        1,
        "um nivel acima do teto cai na `=1`"
    );
}
