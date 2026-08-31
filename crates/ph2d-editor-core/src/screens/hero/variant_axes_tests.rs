//! Os gates dos eixos de propriedade ([`super`]).

use super::{VariantAxis, axes_for, parse_combo};

/// Atalho: `(id, "nome")`.
fn m(id: u64, name: &str) -> (u64, String) {
    (id, name.to_string())
}

/// Os rótulos de um eixo, para ler um veredito de uma linha.
fn labels(ax: &VariantAxis) -> Vec<&str> {
    ax.options.iter().map(|o| o.label.as_str()).collect()
}

/// ⭐ A gramática, e as recusas que a mantêm útil.
#[test]
fn the_grammar_is_strict_on_purpose() {
    assert_eq!(
        parse_combo("Size=Small, State=Idle"),
        Some(vec![
            ("Size".into(), "Small".into()),
            ("State".into(), "Idle".into())
        ]),
        "o espaço a seguir à vírgula tem de ser aparado"
    );
    // ⚠️ **As recusas são a razão de a gramática servir para decidir**: um nome meio-parseado daria
    // um eixo com um valor só, que é uma fileira que não escolhe nada.
    for bad in ["Hero", "=Small", "Size=", "Size==Small", ""] {
        assert_eq!(parse_combo(bad), None, "«{bad}» devia ser recusado");
    }
}

/// ⭐⭐⭐ **Duas perguntas independentes viram DUAS fileiras.**
///
/// ⚠️ É o valor inteiro desta wave: com nomes crus são quatro chips e o artista tem de os LER para
/// descobrir que há dois eixos. Com doze versões seria uma parede.
///
/// **Mutação que deve sangrar:** devolver sempre o `flat_axis`.
#[test]
fn a_matrix_of_names_becomes_one_row_per_question() {
    let fam = [
        m(1, "Size=Small, State=Idle"),
        m(2, "Size=Small, State=Run"),
        m(3, "Size=Big, State=Idle"),
        m(4, "Size=Big, State=Run"),
    ];
    let (axes, beyond) = axes_for(&fam, 1);
    assert_eq!(beyond, 0);
    assert_eq!(axes.len(), 2, "duas chaves, duas fileiras");
    assert_eq!(axes[0].name, "Size");
    assert_eq!(labels(&axes[0]), ["Small", "Big"]);
    assert_eq!(axes[1].name, "State");
    assert_eq!(labels(&axes[1]), ["Idle", "Run"]);
    // O vigente é `Size=Small` no primeiro eixo e `State=Idle` no segundo — a mesma cópia.
    assert!(axes[0].options[0].current && !axes[0].options[1].current);
    assert!(axes[1].options[0].current && !axes[1].options[1].current);
}

/// ⭐⭐⭐ **ALCANÇÁVEL = difere de mim só neste eixo** — e é isso que faz cada chip chegar a algum
/// lado.
///
/// ⚠️ De `Small/Idle`, o chip `Big` tem de apontar para `Big/Idle` — **não** para `Big/Run`, que
/// mudaria dois eixos de uma vez sem o artista o pedir.
///
/// **Mutação que deve sangrar:** aceitar qualquer membro que declare o valor (tirar o `all`).
#[test]
fn a_chip_changes_exactly_one_axis() {
    // ⛔⛔ **A ORDEM desta fixtura é load-bearing** (medido em 2026-08-30): com o `Big/Idle` antes
    // do `Big/Run`, apagar a cerca da alcançabilidade **não muda nada** — a dedupe por rótulo
    // guarda o primeiro, e o primeiro calha ser o certo. ⇒ o `Big/Run` vem PRIMEIRO, e aí a
    // mutação escolhe-o e o gate sangra. *Uma fixtura ordenada a favor da lei não a testa.*
    let fam = [
        m(1, "Size=Small, State=Idle"),
        m(2, "Size=Small, State=Run"),
        m(3, "Size=Big, State=Run"),
        m(4, "Size=Big, State=Idle"),
    ];
    let (axes, _) = axes_for(&fam, 1); // estou em Small/Idle
    let big = axes[0]
        .options
        .iter()
        .find(|o| o.label == "Big")
        .expect("o eixo Size tem de oferecer Big");
    assert_eq!(
        big.master, 4,
        "o chip Big saltou para Big/Run — mudou DOIS eixos de uma vez"
    );
}

/// ⛔ **Uma matriz com BURACOS não inventa um estado de erro** — a combinação que não existe
/// simplesmente não é oferecida.
#[test]
fn a_hole_in_the_matrix_is_an_absent_chip_not_an_error() {
    // Não existe `Big/Run`.
    let fam = [
        m(1, "Size=Small, State=Idle"),
        m(2, "Size=Small, State=Run"),
        m(3, "Size=Big, State=Idle"),
    ];
    let (axes, _) = axes_for(&fam, 2); // estou em Small/Run
    // ⚠️ **De `Small/Run` o eixo `Size` DESAPARECE** — `Big/Run` não existe, e o único valor que
    // sobrava era o meu. *Um eixo com um valor só é uma pergunta sem respostas*, e oferecê-lo seria
    // um chip que não leva a lado nenhum.
    assert!(
        !axes.iter().any(|a| a.name == "Size"),
        "o eixo Size ficou com um chip só, que é a vigente — ele não escolhe nada"
    );
    // ⭐ E o OUTRO eixo continua inteiro: o buraco tirou uma pergunta, não a fileira toda.
    let state = axes.iter().find(|a| a.name == "State").expect("eixo State");
    assert_eq!(labels(state), ["Idle", "Run"]);

    // ⚠️ **O CONTROLO**: da outra ponta da matriz o `Size` existe. Sem isto o gate ficaria verde
    // com uma lei que apagasse o eixo sempre.
    let (from_idle, _) = axes_for(&fam, 1); // estou em Small/Idle
    let size = from_idle
        .iter()
        .find(|a| a.name == "Size")
        .expect("de Small/Idle o Big/Idle existe");
    assert_eq!(labels(size), ["Small", "Big"]);
}

/// ⚠️ **Nomes que não são combinações caem no modo PLANO** — que é a fileira que o cartão já
/// desenhava. *Uma família que não se declara em eixos não é um erro; é a maioria.*
#[test]
fn plain_names_fall_back_to_one_flat_row() {
    let fam = [m(1, "Hero"), m(2, "Hero Angry"), m(3, "Hero Sad")];
    let (axes, _) = axes_for(&fam, 2);
    assert_eq!(axes.len(), 1);
    // ⚠️ **O nome é VAZIO e quem o escreve é o painel** (HR-15): um rótulo em inglês aqui é uma
    // string de UI numa camada que o portão da HR-15 não varre.
    assert_eq!(axes[0].name, "");
    assert_eq!(labels(&axes[0]), ["Hero", "Hero Angry", "Hero Sad"]);
    assert!(axes[0].options[1].current);
}

/// ⛔⛔ **Chaves DISCORDANTES caem no plano, e não numa interseção.**
///
/// ⚠️ Com `{Size}` num membro e `{Size, State}` noutro, uma interseção esconderia o `State` do
/// segundo e o artista perderia um eixo **sem nada a dizer porquê**. No plano tudo aparece.
#[test]
fn members_that_disagree_on_the_keys_fall_back_to_plain_names() {
    let fam = [m(1, "Size=Small"), m(2, "Size=Big, State=Run")];
    let (axes, _) = axes_for(&fam, 1);
    assert_eq!(axes.len(), 1);
    assert_eq!(axes[0].name, "");
    assert_eq!(labels(&axes[0]), ["Size=Small", "Size=Big, State=Run"]);
}

/// ⚠️ **Menos de dois membros não é um conjunto.**
#[test]
fn a_family_of_one_offers_nothing() {
    assert_eq!(axes_for(&[m(1, "Size=Small")], 1).0.len(), 0);
    assert_eq!(axes_for(&[], 1).0.len(), 0);
}

/// ⛔ **O excedente da tabela de ids é CONTADO, nunca truncado em silêncio.**
///
/// **Mutação que deve sangrar:** truncar sem somar ao `beyond`.
#[test]
fn what_the_id_table_cannot_address_is_counted() {
    // Nove valores num eixo, contra um teto de oito.
    let fam: Vec<(u64, String)> = (0u64..9).map(|i| (i, format!("Size=S{i}"))).collect();
    let (axes, beyond) = axes_for(&fam, 0);
    assert_eq!(axes.len(), 1);
    assert_eq!(axes[0].options.len(), crate::ids::MAX_INSTANCE_AXIS_VALUES);
    assert_eq!(beyond, 9 - crate::ids::MAX_INSTANCE_AXIS_VALUES);
}

/// ⚠️ **Sem âncora não há «alcançável daqui»** — um mestre vigente fora da família devolve nada,
/// em vez de uma fileira que mostra opções sem dizer onde se está.
#[test]
fn a_current_master_outside_the_family_offers_nothing() {
    let fam = [m(1, "Size=Small, State=Idle"), m(2, "Size=Big, State=Idle")];
    assert_eq!(axes_for(&fam, 99).0.len(), 0);
}

// ── Os achados da auditoria de 2026-08-30 ──────────────────────────────────────────────────────

/// ⛔⛔⛔ **DUAS versões «na diagonal» continuam a ter uma travessia** — e sem isto era uma
/// REGRESSÃO contra a fileira plana de antes.
///
/// ⚠️ Com `Small/Idle` e `Big/Run` nenhuma é alcançável **num passo**, os dois eixos caem por ter um
/// valor só, e o cartão ficava **sem fileira nenhuma**: o artista tinha duas versões do componente
/// e nenhuma superfície para trocar. ⇒ quando a matriz é esparsa demais para perguntas, a família
/// volta a ser uma **lista** — pior de ler, e alcançável.
///
/// **Mutação que deve sangrar:** tirar o `if axes.is_empty() { axes = flat_axis(..) }`.
#[test]
fn a_family_of_two_on_the_diagonal_still_offers_a_way_across() {
    let fam = [m(1, "Size=Small, State=Idle"), m(2, "Size=Big, State=Run")];
    let (axes, _) = axes_for(&fam, 1);
    assert_eq!(axes.len(), 1, "a família ficou sem fileira nenhuma");
    assert_eq!(axes[0].name, "", "a rede é o modo plano");
    assert_eq!(labels(&axes[0]).len(), 2, "as duas versões têm de aparecer");
    assert!(
        axes[0].options.iter().any(|o| o.current),
        "a fileira mostra as opções e esconde a resposta"
    );
}

/// ⛔⛔ **O VIGENTE sobrevive ao corte da tabela de ids.**
///
/// ⚠️ Truncar às cegas deixava a fileira **sem nenhum chip aceso** quando o mestre corrente estava
/// depois do teto. ⚠️ E o gate que media a truncagem escolhia `me = 0` — *uma fixtura ordenada a
/// favor da lei*, que é a mesma classe de defeito que esta wave já tinha corrigido uma vez.
///
/// **Mutação que deve sangrar:** voltar a `ax.options.truncate(cap)` sem tratar o vigente.
#[test]
fn the_current_option_survives_the_id_table_cap() {
    let cap = crate::ids::MAX_INSTANCE_AXIS_VALUES;
    let fam: Vec<(u64, String)> = (0u64..=(cap as u64))
        .map(|i| (i, format!("Size=S{i}")))
        .collect();
    // O vigente é o ÚLTIMO — exactamente o que cai fora do teto.
    let me = cap as u64;
    let (axes, beyond) = axes_for(&fam, me);
    assert_eq!(axes.len(), 1);
    assert_eq!(axes[0].options.len(), cap);
    assert_eq!(beyond, 1);
    let cur = axes[0]
        .options
        .iter()
        .find(|o| o.current)
        .expect("a fileira ficou SEM vigente — mostra as opções e esconde a resposta");
    assert_eq!(cur.master, me);
}

/// ⚠️ **O excedente não conta o próprio artista.**
///
/// O vigente aparece como opção em TODO eixo; contá-lo dizia *«mais uma versão»* sobre ele próprio.
///
/// **Mutação que deve sangrar:** `beyond += ax.options.len()` no ramo dos eixos.
#[test]
fn the_overflow_does_not_count_the_current_master() {
    // Cinco eixos, contra um teto de quatro. Cada um tem duas opções: eu e um vizinho.
    let mut fam = vec![m(0, "A=1, B=1, C=1, D=1, E=1")];
    for (i, k) in ["A", "B", "C", "D", "E"].iter().enumerate() {
        let name: String = ["A", "B", "C", "D", "E"]
            .iter()
            .map(|x| format!("{x}={}", if x == k { 2 } else { 1 }))
            .collect::<Vec<_>>()
            .join(", ");
        fam.push(m(i as u64 + 1, &name));
    }
    let (axes, beyond) = axes_for(&fam, 0);
    assert_eq!(axes.len(), crate::ids::MAX_INSTANCE_AXES, "o teto de eixos");
    assert_eq!(
        beyond, 1,
        "o 5.º eixo tinha DUAS opções e uma delas sou eu — só a outra se perdeu"
    );
}

/// ⛔ **Sem âncora não há fileira, TAMBÉM no modo plano.**
///
/// ⚠️ A cerca existia só no multi-eixo, e o plano devolvia a lista inteira com nenhum chip aceso.
///
/// **Mutação que deve sangrar:** tirar a guarda `members.iter().any(|(id, _)| *id == me)`.
#[test]
fn a_current_master_outside_the_family_offers_nothing_in_plain_mode_either() {
    let fam = [m(1, "Hero"), m(2, "Hero Sad")];
    assert_eq!(
        axes_for(&fam, 99).0.len(),
        0,
        "a fileira plana apareceu sem dizer onde a cópia está"
    );
}

/// ⚠️ **Uma chave REPETIDA não é uma combinação** — ela daria duas fileiras com o mesmo nome e as
/// mesmas opções, e clicar numa mexeria na outra.
#[test]
fn a_repeated_key_is_not_a_combination() {
    assert_eq!(parse_combo("Size=Small, Size=Big"), None);
    // ⚠️ O controlo: sem a repetição, a mesma forma parseia.
    assert!(parse_combo("Size=Small, State=Big").is_some());
}
