//! Os gates da **TABELA SINAL → PAPEL** (item 4 do estudo dos contêineres).

use crate::binding::SignalBinding;
use crate::pose::{ObjectPose, UiState};
use crate::role::StateRole;
use crate::sets::StateSets;
use ph2d_vec_scene::VecPathId;

fn with_default(host: VecPathId) -> StateSets {
    let mut s = StateSets::default();
    let mut st = UiState::new(StateRole::Default);
    st.objects = vec![ObjectPose::new(host)];
    s.set(host, st);
    s
}

/// **O nome encontra quem responde** — a porta que o consumidor de sinais usa.
#[test]
fn a_name_finds_the_hosts_that_listen_to_it() {
    let (a, b): (VecPathId, VecPathId) = (1, 2);
    let mut s = with_default(a);
    s.set(b, UiState::new(StateRole::Default));
    s.push_binding(a);
    s.set_binding_name(a, 0, "open".into());
    s.set_binding_role(a, 0, StateRole::Pressed);
    s.push_binding(b);
    s.set_binding_name(b, 0, "open".into());
    s.set_binding_role(b, 0, StateRole::Hover);

    let mut hit: Vec<_> = s.targets("open").collect();
    hit.sort_unstable();
    assert_eq!(
        hit,
        vec![(a, StateRole::Pressed), (b, StateRole::Hover)],
        "um nome move TODOS os que o escutam — é o que faz a ligação valer mais que um campo \
         escondido dentro do botão"
    );
    assert_eq!(
        s.targets("close").count(),
        0,
        "um nome que ninguém autorou não pode mover nada"
    );
}

/// ⭐ **UMA LIGAÇÃO SEM NOME NÃO CASA COM NADA.**
///
/// ⚠️ O modo de falha que esta guarda impede não é um erro: o artista aperta *Add*, a linha nasce
/// vazia, e um produtor que publicasse um nome vazio moveria **toda ligação recém-criada do
/// documento de uma vez** — a cena inteira a saltar de pose sem ninguém ter pedido.
#[test]
fn a_binding_with_no_name_matches_nothing() {
    let host: VecPathId = 1;
    let mut s = with_default(host);
    s.push_binding(host);
    assert_eq!(s.bindings(host).len(), 1, "a fixture criou a linha vazia");
    assert_eq!(
        s.targets("").count(),
        0,
        "a linha vazia respondeu ao nome vazio — o salto de cena que a guarda existe para impedir"
    );
    assert!(!SignalBinding::empty().matches(""), "e a porta em si");
}

/// **Uma forma apagada leva as ligações dela** — pelo `retain_hosts` que já corre por frame, sem
/// uma linha a mais. É a razão inteira de a tabela morar dentro do `HostStates`.
#[test]
fn a_dead_host_takes_its_bindings_with_it() {
    let host: VecPathId = 7;
    let mut s = with_default(host);
    s.push_binding(host);
    s.set_binding_name(host, 0, "open".into());
    assert_eq!(s.targets("open").count(), 1);

    s.retain_hosts(|id| id != host);
    assert_eq!(
        s.targets("open").count(),
        0,
        "a tabela ficou a apontar para uma forma que já não existe"
    );
}

/// **A linha apagada some, e o hospedeiro vazio sai da tabela** — a mesma lei do `clear`: um
/// documento não carrega uma entrada sem nada dentro.
#[test]
fn removing_the_last_binding_evicts_an_otherwise_empty_host() {
    let host: VecPathId = 3;
    let mut s = StateSets::default();
    s.push_binding(host);
    s.set_binding_name(host, 0, "open".into());
    assert!(!s.is_empty());

    s.remove_binding(host, 0);
    assert!(
        s.is_empty(),
        "um hospedeiro sem estado, sem ligação e com o tempo de fábrica ficou no documento"
    );
}

/// **Mas ele NÃO sai se ainda tem poses** — apagar uma ligação não pode levar o trabalho ao lado.
#[test]
fn removing_a_binding_keeps_a_host_that_still_has_poses() {
    let host: VecPathId = 4;
    let mut s = with_default(host);
    s.push_binding(host);
    s.remove_binding(host, 0);
    assert!(s.bindings(host).is_empty());
    assert_eq!(
        s.get(host).len(),
        1,
        "apagar a ligação levou a pose gravada junto"
    );
}

/// **Um índice fora da lista é ignorado, nunca um pânico** — o painel publica um snapshot e o
/// clique chega um frame depois, então a lista pode ter encolhido no meio do gesto.
#[test]
fn a_stale_index_is_ignored() {
    let host: VecPathId = 5;
    let mut s = with_default(host);
    s.set_binding_name(host, 9, "open".into());
    s.set_binding_role(host, 9, StateRole::Hover);
    s.remove_binding(host, 9);
    assert_eq!(s.targets("open").count(), 0);
}
