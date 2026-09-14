//! Os gates do censo do alcance (doc 110 §9).

/// **SONDA — o catálogo inteiro, e os params que ninguém alcança.**
///
/// ```text
/// cargo test -p ph2d-app-motion --lib probe_unreachable_params -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn probe_unreachable_params() {
    let c = super::censo();
    eprintln!("\n  nó                            | params que NENHUMA combinação revela");
    eprintln!("  ------------------------------|-------------------------------------");
    for (no, presos) in &c.presos {
        eprintln!("  {no:<29} | {}", presos.join(" · "));
    }
    eprintln!(
        "\n  {} acusado(s) · varridos {} nós / {} params · {} nós declaram gate.\n",
        c.presos.len(),
        c.nos,
        c.params,
        c.com_gate
    );
}

/// ⭐⭐⭐ **NENHUM PARAM DO CATÁLOGO ESTÁ ENTERRADO ATRÁS DOS PRÓPRIOS GATES.**
///
/// Um controlo que nenhuma combinação revela é a espécie de knob morto que **nenhuma** sonda deste
/// repo apanhava: a caça de 2026-08-30 seguiu o valor até ao efeito (*o painel escreve onde · quem
/// lê · o leitor decide?*) e esta pergunta vem **antes** dela — *o artista consegue sequer chegar
/// ao controlo?*
///
/// ⚠️⚠️ **O PISO DE POPULAÇÃO é metade deste gate.** Ele lê `0 acusados` quando está são, que é
/// exactamente o que lê quando está partido; sem os três números abaixo, um registry renomeado
/// deixaria este teste verde a varrer o vazio — a catraca que vira licença, um nível acima.
#[test]
fn no_param_in_the_catalogue_is_buried_behind_its_own_gates() {
    let c = super::censo();
    assert!(
        c.nos >= 100 && c.params >= 400,
        "o censo varreu {} nós / {} params -- ele mede o catálogo inteiro, e este numero diz que \
         a varredura se partiu",
        c.nos,
        c.params
    );
    assert!(
        c.com_gate >= 20,
        "só {} nós declaram gate de visibilidade -- sem sujeitos este censo não testa nada",
        c.com_gate
    );
    assert!(
        c.presos.is_empty(),
        "params que nenhuma combinação de gates revela: {:?}",
        c.presos
    );
}

/// ⭐⭐ **DENTRO DE UMA SECÇÃO, DOIS CONTROLOS NÃO PODEM TER O MESMO NOME.**
///
/// ⚠️ Ela é a fatia da lei do vocabulário que uma máquina sabe julgar — ver
/// [`super::rotulos_colididos`] para o porquê de ela não ser mais larga (a versão larga acusa ~40
/// grupos legítimos e vira licença).
///
/// ⚠️ **E o piso de população outra vez:** ela lê `0` colisões quando está sã, que é o que lê
/// quando o registry muda de nome debaixo dela.
#[test]
fn two_controls_in_one_section_never_share_a_name() {
    let m = crate::motion_state::MotionState::new();
    let rotulados: usize = m
        .registry
        .manifests()
        .map(|man| m.registry.param_ui(man.id).unwrap_or(&[]).len())
        .sum();
    assert!(
        rotulados >= 400,
        "só {rotulados} params rotulados no catálogo -- a varredura partiu-se"
    );
    let colididos = super::rotulos_colididos();
    assert!(
        colididos.is_empty(),
        "dois controlos lado a lado com o mesmo nome: {colididos:?}"
    );
}
