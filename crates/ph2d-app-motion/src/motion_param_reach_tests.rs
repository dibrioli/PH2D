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

/// **SONDA — que nós nenhuma folha de conferência nomeia.**
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn probe_unconferred_nodes() {
    let c = super::conferencia();
    eprintln!(
        "\n  {} folha(s) lida(s) · {} nós no registry · {} sem folha:\n",
        c.folhas,
        c.nos,
        c.ausentes.len()
    );
    for n in &c.ausentes {
        eprintln!("  ⚠️  {n}");
    }
    eprintln!();
}

/// **A DÍVIDA DA CONFERÊNCIA — nomeada, com censo de obsolescência, e só encolhe.**
///
/// ⛔⛔ Estes nós existem no registry e **nenhuma folha lhes dá uma LINHA**: ou nasceram depois da
/// folha da família deles, ou são citados só dentro da linha de outro nó (como a CURA que fechou
/// aquela célula). Nos dois casos o placar lê `0 aberto` para eles **por ausência**, e não por
/// estarem conferidos.
///
/// ⚠️ **Cada entrada diz a FOLHA a que pertence** — sem isso a lista é um lamento, e com isso é um
/// trabalho endereçado.
///
/// ⭐ **Ela já desceu uma vez: `15 → 9`.** O ciclo 6 W4 conferiu os seis do seu grupo
/// (`pulse.adsr` · `pulse.level` · `pulse.signal` · `value.cursor` · `value.number` ·
/// `value.table`) nas folhas 12 e 15, e foi a **metade de obsolescência** deste gate que exigiu
/// apagá-los daqui — em voz alta, a nomear os seis. *Uma catraca sem essa metade não desce: ela
/// vira licença* (`CLAUDE.md` §5.0).
const SEM_LINHA_DE_CONFERENCIA: &[(&str, &str)] = &[
    ("audio.bands", "07_tempo_estilisticos"),
    ("motion.bezier_warp", "04_deformers"),
    ("motion.proximity", "08_stream_utilidade"),
    ("motion.randomize", "05_transform"),
    ("motion.sub_uv", "11_fx_raster"),
    ("motion.velocity", "08_stream_utilidade"),
    ("source.lsystem", "14_source"),
    ("source.table", "14_source"),
    ("source.text", "14_source"),
];

/// ⭐⭐ **TODO NÓ DO CATÁLOGO TEM UMA LINHA DE CONFERÊNCIA — ou está nesta lista, com a folha.**
///
/// ⚠️⚠️ **As DUAS metades, senão a catraca vira licença** (`CLAUDE.md` §5.0): ela reprova quando
/// alguém nasce fora das folhas **e** quando uma entrada desta lista já não descreve nada (o nó foi
/// conferido, ou morreu) — *uma lista de dívida que ninguém pode apagar é uma licença permanente.*
///
/// ⚠️ **E o piso de população:** as folhas são lidas em RUNTIME (não há `include_str!` com glob),
/// então um caminho partido devolveria zero folhas e a lista inteira — o gate diz isso em voz alta.
#[test]
fn every_node_has_a_conference_row_or_is_named_in_the_debt() {
    let c = super::conferencia();
    assert!(
        c.folhas >= 17 && c.nos >= 100,
        "leu {} folha(s) e {} nós -- o caminho das folhas ou o registry partiu-se",
        c.folhas,
        c.nos
    );
    let devendo: Vec<&str> = SEM_LINHA_DE_CONFERENCIA.iter().map(|(n, _)| *n).collect();
    let novos: Vec<&str> = c
        .ausentes
        .iter()
        .filter(|n| !devendo.contains(n))
        .copied()
        .collect();
    assert!(
        novos.is_empty(),
        "nó(s) sem linha de conferência e fora da dívida declarada: {novos:?} -- confira-o na \
         folha da família dele, ou acrescente-o a SEM_LINHA_DE_CONFERENCIA com a folha"
    );
    // A metade que faz a catraca DESCER.
    let obsoletos: Vec<&str> = devendo
        .iter()
        .filter(|n| !c.ausentes.contains(n))
        .copied()
        .collect();
    assert!(
        obsoletos.is_empty(),
        "estas entradas de SEM_LINHA_DE_CONFERENCIA já não descrevem nada (o nó ganhou linha, ou \
         deixou de existir) -- apague-as: {obsoletos:?}"
    );
}
