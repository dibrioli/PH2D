//! ⛔⛔⛔ **O GATE QUE FALTAVA À COSTURA DO EIXO** — uma região cujo corte só deixa a costura não
//! pode cair no degenerado.
//!
//! # O defeito, medido
//!
//! O [`crate::profile::sd_profile_in_region`] tirava as arestas do eixo com um `continue`
//! **depois** do corte, e tinha escrito ao lado que um corte vazio é *«impossível — a regra do
//! corte guarda sempre pelo menos a aresta que realiza o `dmax`»*. ⚠️ **Verdade sobre o CORTE e
//! falsa sobre a composição dos DOIS filtros**: o corte podia devolver exactamente a costura, o
//! `continue` tirava-a, e a região recaía na árvore **INTEIRA**.
//!
//! Medido no vaso da cena `5` da `line/3DModeling` (`1` das `24` arestas no eixo), numa grelha
//! `32×32` em `(u, v)`: **`5` de `1 024`** células pagavam `931` linhas de WGSL em vez de `~50`
//! (**`18×`**), e eram as células **sobre o eixo à altura da costura** — dentro do sólido, por onde
//! a marcha passa. Depois da cura: `0 de 1 024`, e o pior caso da grelha `931 → 243`.
//!
//! # ⚠️ Porque a fixtura é um QUADRADO e não o vaso
//!
//! O mecanismo não precisa do desenho do dono: precisa de **um contorno cujo fecho assente no
//! eixo**, que é o que um perfil de torno tem por construção (ele fecha sobre o eixo de rotação).
//! ⛔ Copiar o vaso para cá seria uma segunda cópia de um desenho que vive noutra crate.
//!
//! ⭐ **E o CONTROLO está dentro:** a 1.ª metade afirma que existe uma célula em que o corte,
//! **sem** o cuidado do eixo, devolve só a costura — *sem ela este gate mede um planalto*.

use crate::profile_index::ProfileIndex;
use ph2d_field::{FillRule, Profile};

/// Um contorno cujo FECHO (último → primeiro) assenta no eixo `u = 0`.
fn perfil_com_costura() -> Profile {
    Profile::new(
        vec![vec![[0.0, -1.0], [1.0, -1.0], [1.0, 1.0], [0.0, 1.0]]],
        FillRule::NonZero,
        1.0e-4,
    )
    .expect("um quadrado encostado ao eixo é um perfil válido")
}

/// A célula onde o fenómeno vive: encostada ao eixo, a meia altura da costura.
const CELULA: ([f32; 2], [f32; 2]) = ([0.0, -0.1], [0.05, 0.1]);

#[test]
fn uma_regiao_sobre_a_costura_nao_paga_a_arvore_inteira() {
    let p = perfil_com_costura();
    let idx = ProfileIndex::build(&p);
    let tol = p.tolerance();
    let (lo, hi) = CELULA;

    // ⭐ **O CONTROLO**: sem o cuidado do eixo, o corte desta célula devolve SÓ a costura — é isso
    // que fazia o `continue` a jusante esvaziar a conta.
    let cru = idx.distance_edges(lo, hi);
    assert!(
        !cru.is_empty() && cru.iter().all(|i| idx.no_eixo(*i, tol)),
        "a fixtura não contém o fenómeno: o corte cru desta célula devolveu {cru:?}, e nem todas \
         assentam no eixo — um gate sobre um planalto não afirma nada"
    );

    // ⭐⭐ **A LEI**: o corte que a região usa deixa uma aresta que NÃO é a costura.
    let fora = idx.distance_edges_fora_do_eixo(lo, hi, tol);
    assert!(
        !fora.is_empty(),
        "o corte fora-do-eixo devolveu vazio nesta célula — a região cai no degenerado e \
         reconstrói a árvore INTEIRA (medido 18× no vaso da cena 5)"
    );
    assert!(
        fora.iter().all(|i| !idx.no_eixo(*i, tol)),
        "o corte fora-do-eixo devolveu {fora:?}, e alguma assenta no eixo — a população do corte e \
         o filtro do consumidor voltaram a ser duas respostas"
    );

    // ⭐⭐⭐ **E a metade que mede o PREÇO, senão «não degenera» vira licença.**
    //
    // ⚠️⚠️ **Ela NÃO mora na célula do eixo, e isso foi MEDIDO:** a 1.ª redacção exigia `2×` ali e
    // reprovou sobre a cura CERTA (`60` linhas contra `77`). O corte é por **DIÂMETRO**, logo numa
    // célula longe de toda parede ele guarda as três (`[0, 1, 2]`) — e guarda **bem**, porque
    // qualquer delas pode ser a mais próxima de algum ponto dela. *O corte morde junto da parede, e
    // é lá que o preço se mede.*
    //
    // ⭐ **E a barra é DERIVADA, não escolhida:** uma célula encostada à parede direita
    // (`u ∈ [0,95 · 1,00]`) está a `≤ 0,05` dela e a `≥ 0,9` das outras três ⇒ o `dmax` do corte é
    // o da própria parede, e **uma** aresta sobrevive. Medido: `[1]` de `4`.
    let (plo, phi) = ([0.95, -0.1], [1.0, 0.1]);
    let na_parede = idx.distance_edges_fora_do_eixo(plo, phi, tol);
    assert_eq!(
        na_parede.len(),
        1,
        "encostado à parede o corte guardou {na_parede:?} de {} — ele concorda e não compra nada",
        idx.edge_count()
    );

    // ⭐⭐ **E a região sobre a costura não degenerou**: a árvore dela é estritamente menor que a da
    // peça inteira. ⛔ Sem esta metade, um corte que devolvesse tudo passaria as de cima.
    let linhas = |lo: [f32; 2], hi: [f32; 2]| {
        crate::Field::from_tree(&crate::profile::sd_profile_in_region(
            &p,
            &idx,
            &fidget::context::Tree::x(),
            &fidget::context::Tree::y(),
            lo,
            hi,
            true,
            None,
        ))
        .tape_shape()
        .map_or(0, |s| s.guardados)
    };
    let inteira = crate::Field::from_tree(&crate::profile::sd_profile(
        &p,
        &fidget::context::Tree::x(),
        &fidget::context::Tree::y(),
    ))
    .tape_shape()
    .map_or(0, |s| s.guardados);
    let na_costura = linhas(lo, hi);
    assert!(
        na_costura < inteira,
        "a região sobre a costura paga {na_costura} linhas contra {inteira} da peça inteira — ela \
         degenerou"
    );
}
