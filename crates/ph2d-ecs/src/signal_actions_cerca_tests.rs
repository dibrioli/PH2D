//! Os gates da **CERCA** e dos **DOIS LADOS** do [`super`] (suplente #24, 2026-09-19) — irmão do
//! [`super::tests`] por ASSUNTO, e não por tecto: o que se mede aqui é *quem reage* e *a quem*,
//! depois de o sinal passar a trazer **quem o disse**.
//!
//! ⚠️ **A fixtura é a da sonda** (`mede_o_que_a_composicao_ja_da_ao_golpe`): dez inimigos iguais,
//! com a MESMA tabela, porque é exactamente aí que a cegueira aparece. Um gate com um reactor só
//! fica verde com a cerca apagada — *uma fixtura com um sujeito não testa a escolha de sujeito*.

use super::*;
use crate::{StableId, Transform, assign_missing_stable_ids};

/// Uma linha com a cerca escolhida.
fn linha(on: &str, verb: SignalVerb, from: SignalFrom, target_by: SignalTarget) -> SignalAction {
    SignalAction {
        on: on.into(),
        target: String::new(),
        verb,
        arg: String::new(),
        target_by,
        from,
    }
}

/// **`n` inimigos IGUAIS**, cada um com a mesma tabela. Devolve o mundo e as entidades, em ordem.
fn arena(n: usize, tabela: Vec<SignalAction>) -> (World, Vec<Entity>) {
    let mut w = World::new();
    let mut es = Vec::new();
    for i in 0..n {
        es.push(
            w.spawn((
                Transform::default(),
                Name::new(format!("Inimigo {i}")),
                SignalActions(tabela.clone()),
            ))
            .id(),
        );
    }
    assign_missing_stable_ids(&mut w);
    (w, es)
}

/// Um disparo com sujeito.
fn de(nome: &str, quem: Entity) -> Disparo<'_> {
    Disparo {
        nome,
        quem: Some(quem),
        outro: None,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// G1 — A CERCA
// ─────────────────────────────────────────────────────────────────────────────

/// ⭐⭐⭐ **Dez inimigos, um golpe, UM reagente** — e o CONTROLO ao lado, que é o que o produto
/// fazia até 2026-09-19.
///
/// ⚠️ **As duas metades no mesmo gate de propósito:** a positiva sozinha ficaria verde num mundo em
/// que nada mais reagisse, e é o `10` da negativa que prova que a fixtura CONTÉM o fenómeno. *Uma
/// régua que não vê o defeito acontecer não prova que ele deixou de acontecer.*
///
/// **Mutação que deve sangrar:** apagar a chamada a [`SignalFrom::deixa_passar`] do laço da
/// [`resolve`].
#[test]
fn a_cerca_faz_um_golpe_atingir_um_inimigo_e_nao_os_dez() {
    // ⛔ O CONTROLO: a MESMA arena, a MESMA tabela, sem a cerca.
    let (mut w, es) = arena(
        10,
        vec![linha(
            "golpe",
            SignalVerb::AddToCounter,
            SignalFrom::Anyone,
            SignalTarget::Named,
        )],
    );
    assert_eq!(
        resolve(&mut w, &TagTree::new(), &[de("golpe", es[3])]).len(),
        10,
        "o CONTROLO tem de reproduzir a cegueira: sem cerca, um tiro atinge os dez"
    );

    // ⭐ E com a cerca, só quem gritou.
    let (mut w, es) = arena(
        10,
        vec![linha(
            "golpe",
            SignalVerb::AddToCounter,
            SignalFrom::Myself,
            SignalTarget::Named,
        )],
    );
    let out = resolve(&mut w, &TagTree::new(), &[de("golpe", es[3])]);
    assert_eq!(out.len(), 1, "a cerca deixou passar mais do que um");
    assert_eq!(out[0].source, es[3], "reagiu quem NAO gritou");
    assert_eq!(
        out[0].target, es[3],
        "o alvo vazio tem de cair em quem reagiu"
    );
}

/// ⭐⭐ **Um sinal SEM SUJEITO nunca passa a cerca** — a timeline, um botão do painel e o Motion
/// publicam sem `source`.
///
/// ⚠️ **É a metade mais fácil de escrever ao contrário:** tratar `None` como *«qualquer um»* faria
/// uma cerca FECHADA deixar passar tudo, que é o modo de falha mais caro que uma cerca pode ter — e
/// ele é **mudo**, porque a linha continua a disparar.
///
/// **Mutação que deve sangrar:** `Myself => quem_falou.is_none() || quem_falou == Some(reactor)`.
#[test]
fn um_sinal_sem_sujeito_nao_passa_a_cerca_e_o_anyone_deixa_passar() {
    let (mut w, _) = arena(
        3,
        vec![linha(
            "sino",
            SignalVerb::Show,
            SignalFrom::Myself,
            SignalTarget::Named,
        )],
    );
    assert!(
        resolve(&mut w, &TagTree::new(), &[Disparo::anonimo("sino")]).is_empty(),
        "um sinal sem sujeito passou uma cerca `Myself`"
    );
    // ⛔ O CONTROLO: a mesma tabela sem cerca ouve o mesmo sino.
    let (mut w, _) = arena(
        3,
        vec![linha(
            "sino",
            SignalVerb::Show,
            SignalFrom::Anyone,
            SignalTarget::Named,
        )],
    );
    assert_eq!(
        resolve(&mut w, &TagTree::new(), &[Disparo::anonimo("sino")]).len(),
        3,
        "sem cerca, um sinal sem sujeito tem de continuar a chegar aos tres"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// G2 — OS DOIS LADOS
// ─────────────────────────────────────────────────────────────────────────────

/// ⭐⭐⭐ **`Other` é quem BATEU, e não quem falou.**
///
/// ⚠️ **A fixtura tem os dois sujeitos DIFERENTES de propósito**: com `quem == outro` a troca seria
/// invisível. É a mesma cerca que o gate da porta do barramento põe do outro lado da fronteira.
///
/// **Mutação que deve sangrar:** o braço `Other` a ler `disparo.quem`.
#[test]
fn o_outro_lado_e_quem_bateu_e_nao_quem_falou() {
    let (mut w, es) = arena(
        1,
        vec![
            // ⛔ O CONTROLO: a MESMA linha com o alvo vazio cai em QUEM REAGE.
            linha(
                "toque",
                SignalVerb::Show,
                SignalFrom::Anyone,
                SignalTarget::Named,
            ),
            linha(
                "toque",
                SignalVerb::Hide,
                SignalFrom::Anyone,
                SignalTarget::Other,
            ),
        ],
    );
    let heroi = w.spawn((Transform::default(), Name::new("Heroi"))).id();
    let bala = w.spawn((Transform::default(), Name::new("Bala"))).id();
    assign_missing_stable_ids(&mut w);

    let out = resolve(
        &mut w,
        &TagTree::new(),
        &[Disparo {
            nome: "toque",
            quem: Some(heroi),
            outro: Some(bala),
        }],
    );
    assert_eq!(out.len(), 2, "as duas linhas tinham de resolver");
    assert_eq!(out[0].verb, SignalVerb::Show);
    assert_eq!(out[0].target, es[0], "o alvo vazio nao caiu em quem reagiu");
    assert_eq!(out[1].verb, SignalVerb::Hide);
    assert_eq!(
        out[1].target, bala,
        "`Other` nao caiu no outro lado — leu `quem` em vez de `outro`?"
    );
    assert_ne!(out[1].target, heroi, "`Other` caiu em QUEM FALOU");
    // ⚠️ E quem reagiu continua a ser o dono da tabela, nos dois efeitos.
    assert_eq!(out[0].source, es[0]);
    assert_eq!(out[1].source, es[0]);
}

/// ⛔ **Sem o outro lado, o alvo é NINGUÉM** — `None` não é um curinga, é a lei do alvo que não
/// existe (a mesma que este enum já escreve para uma tag apagada).
///
/// ⚠️ **E o CONTROLO é metade do gate:** sem a linha vazia a disparar no mesmo mundo, um `resolve`
/// que devolvesse zero por qualquer outra razão leria como aprovação.
///
/// **Mutação que deve sangrar:** o braço `Other` a cair em quem reagiu quando não há outro lado.
#[test]
fn sem_o_outro_lado_o_alvo_e_ninguem() {
    let (mut w, _) = arena(
        2,
        vec![
            linha(
                "sino",
                SignalVerb::Hide,
                SignalFrom::Anyone,
                SignalTarget::Other,
            ),
            // ⛔ O CONTROLO: a linha vizinha, com alvo vazio, DISPARA no mesmo mundo.
            linha(
                "sino",
                SignalVerb::Show,
                SignalFrom::Anyone,
                SignalTarget::Named,
            ),
        ],
    );
    let out = resolve(&mut w, &TagTree::new(), &[Disparo::anonimo("sino")]);
    assert_eq!(
        out.len(),
        2,
        "o CONTROLO tinha de disparar nos dois reactores — sem ele este gate mede o nada"
    );
    assert!(
        out.iter().all(|e| e.verb == SignalVerb::Show),
        "a linha com alvo `Other` produziu efeito sem haver outro lado"
    );
}

/// ⭐⭐ **Um lado que JÁ SAIU da cena não produz efeito.**
///
/// ⚠️ **Não é defesa teórica:** o dreno da morte corre no mesmo quadro, depois desta tabela, e os
/// bits de uma entidade despawnada são RECICLADOS pelo bevy — um efeito sobre eles escreveria no
/// objecto errado, calado.
///
/// **Mutação que deve sangrar:** apagar a conferência `get_entity` do `vivo`.
#[test]
fn um_lado_que_ja_saiu_da_cena_nao_produz_efeito() {
    let (mut w, _) = arena(
        1,
        vec![linha(
            "toque",
            SignalVerb::Hide,
            SignalFrom::Anyone,
            SignalTarget::Other,
        )],
    );
    let fantasma = w.spawn((Transform::default(), Name::new("Bala"))).id();
    assign_missing_stable_ids(&mut w);
    // ⛔ O CONTROLO: com ela VIVA, a linha resolve.
    assert_eq!(
        resolve(
            &mut w,
            &TagTree::new(),
            &[Disparo {
                nome: "toque",
                quem: None,
                outro: Some(fantasma),
            }],
        )
        .len(),
        1,
        "o controlo tem de resolver com o alvo vivo"
    );
    w.entity_mut(fantasma).despawn();
    assert!(
        resolve(
            &mut w,
            &TagTree::new(),
            &[Disparo {
                nome: "toque",
                quem: None,
                outro: Some(fantasma),
            }],
        )
        .is_empty(),
        "um alvo ja' despawnado produziu efeito"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// G3 — UM EFEITO POR DISPARO
// ─────────────────────────────────────────────────────────────────────────────

/// ⭐⭐⭐ **Uma linha reage a um DISPARO, não a um NOME** — e isto é uma MUDANÇA declarada.
///
/// Até 2026-09-19 a entrada era `&[&str]` e a pergunta era *«este nome soou?»*: dois eventos com o
/// mesmo nome no mesmo quadro davam **um** efeito. ⚠️ Isso era um defeito latente — *duas moedas
/// apanhadas no mesmo quadro somavam um ponto*.
///
/// ⛔ **Não é o colapso que as origens declaram** (`fires`/`cycles`/`rows`/`count`): esse é dentro
/// de UM produtor, e dois inimigos atingidos são dois produtores.
///
/// **Mutação que deve sangrar:** deduplicar os disparos pelo nome antes do laço.
#[test]
fn uma_linha_reage_por_disparo_e_nao_por_nome() {
    let (mut w, es) = arena(
        2,
        vec![linha(
            "golpe",
            SignalVerb::AddToCounter,
            SignalFrom::Anyone,
            SignalTarget::Named,
        )],
    );
    // Dois golpes no mesmo quadro, de dois sujeitos diferentes.
    let dois = [de("golpe", es[0]), de("golpe", es[1])];
    assert_eq!(
        resolve(&mut w, &TagTree::new(), &dois).len(),
        4,
        "dois disparos sobre dois reactores tinham de dar quatro efeitos"
    );

    // ⭐ E com a cerca, cada um conta o SEU: dois efeitos, um por inimigo.
    let (mut w, es) = arena(
        2,
        vec![linha(
            "golpe",
            SignalVerb::AddToCounter,
            SignalFrom::Myself,
            SignalTarget::Named,
        )],
    );
    let dois = [de("golpe", es[0]), de("golpe", es[1])];
    let out = resolve(&mut w, &TagTree::new(), &dois);
    assert_eq!(out.len(), 2, "cada inimigo tinha de contar o seu golpe");
    assert_eq!(out[0].target, es[0]);
    assert_eq!(out[1].target, es[1]);
}

// ─────────────────────────────────────────────────────────────────────────────
// G4 — O VERBO NOVO
// ─────────────────────────────────────────────────────────────────────────────

/// ⭐⭐ **O verbo que tira da cena existe, é o ÚLTIMO da lista, e não lê o `arg`.**
///
/// ⚠️ **A posição é a TAG e ela viaja no ficheiro** — um verbo no meio reescreveria o sentido de
/// todas as linhas já gravadas, em silêncio. O gate prende as duas metades: a contagem e o fim.
///
/// **Mutações que devem sangrar:** pôr o `Destroy` antes do `AddToCounter` no `ALL` · dar-lhe
/// `uses_arg = true`.
#[test]
fn o_verbo_que_apaga_e_o_ultimo_e_nao_le_o_arg() {
    assert_eq!(
        SignalVerb::ALL.len(),
        9,
        "a lista de verbos mudou de tamanho"
    );
    assert_eq!(
        *SignalVerb::ALL.last().expect("nao vazia"),
        SignalVerb::Destroy,
        "o verbo novo tem de entrar no FIM — a posicao e' a tag do ficheiro"
    );
    assert!(
        !SignalVerb::Destroy.uses_arg(),
        "«tira este» nao tem parametro"
    );
    // ⚠️ A ida-e-volta da tag: um clique escreve a posição, e a posição tem de voltar ao verbo.
    for v in SignalVerb::ALL {
        assert_eq!(SignalVerb::from_tag(v.tag()), v, "{v:?} nao volta da tag");
    }
}

/// ⭐ **A cerca também vai e volta pela tag** — o segmentado do painel escreve a posição.
///
/// **Mutação que deve sangrar:** `from_tag` a devolver sempre o default.
#[test]
fn a_cerca_vai_e_volta_pela_tag() {
    assert_eq!(SignalFrom::ALL.len(), 2);
    assert_eq!(
        SignalFrom::ALL[0],
        SignalFrom::Anyone,
        "o default tem de ser o primeiro — e' o que um ficheiro antigo le'"
    );
    for f in SignalFrom::ALL {
        assert_eq!(SignalFrom::from_tag(f.tag()), f, "{f:?} nao volta da tag");
    }
}
