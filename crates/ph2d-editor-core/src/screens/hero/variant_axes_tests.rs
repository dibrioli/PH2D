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
        parse_combo("Casa {Size=Small, State=Idle}"),
        Some(vec![
            ("Size".into(), "Small".into()),
            ("State".into(), "Idle".into())
        ]),
        "o espaço a seguir à vírgula tem de ser aparado"
    );
    // ⚠️ **As recusas são a razão de a gramática servir para decidir**: um nome meio-parseado daria
    // um eixo com um valor só, que é uma fileira que não escolhe nada.
    for bad in [
        "Hero",
        "Casa {=Small}",
        "Casa {Size=}",
        "Casa {Size==Small}",
        "Casa {}",
        // ⭐ **Sem chaves não há propriedades** — é o que faz um objecto chamado `A=B` ser um
        // objecto chamado `A=B`, e não um eixo.
        "Size=Small, State=Idle",
    ] {
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
        m(1, "Casa {Size=Small, State=Idle}"),
        m(2, "Casa {Size=Small, State=Run}"),
        m(3, "Casa {Size=Big, State=Idle}"),
        m(4, "Casa {Size=Big, State=Run}"),
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
        m(1, "Casa {Size=Small, State=Idle}"),
        m(2, "Casa {Size=Small, State=Run}"),
        m(3, "Casa {Size=Big, State=Run}"),
        m(4, "Casa {Size=Big, State=Idle}"),
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
        m(1, "Casa {Size=Small, State=Idle}"),
        m(2, "Casa {Size=Small, State=Run}"),
        m(3, "Casa {Size=Big, State=Idle}"),
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
    let fam = [
        m(1, "Casa {Size=Small}"),
        m(2, "Casa {Size=Big, State=Run}"),
    ];
    let (axes, _) = axes_for(&fam, 1);
    assert_eq!(axes.len(), 1);
    assert_eq!(axes[0].name, "");
    // ⚠️ **O rótulo é o MIOLO das chaves** (2026-08-31, ver `chip_label`): o nome comum é o mesmo
    // em toda a família, e mostrá-lo daria dois chips a dizer `Casa`. Aqui é o que DIFERE.
    assert_eq!(labels(&axes[0]), ["Size=Small", "Size=Big, State=Run"]);
}

/// ⚠️ **Menos de dois membros não é um conjunto.**
#[test]
fn a_family_of_one_offers_nothing() {
    assert_eq!(axes_for(&[m(1, "Casa {Size=Small}")], 1).0.len(), 0);
    assert_eq!(axes_for(&[], 1).0.len(), 0);
}

/// ⛔ **O excedente da tabela de ids é CONTADO, nunca truncado em silêncio.**
///
/// **Mutação que deve sangrar:** truncar sem somar ao `beyond`.
#[test]
fn what_the_id_table_cannot_address_is_counted() {
    // Nove valores num eixo, contra um teto de oito.
    let fam: Vec<(u64, String)> = (0u64..9)
        .map(|i| (i, format!("Casa {{Size=S{i}}}")))
        .collect();
    let (axes, beyond) = axes_for(&fam, 0);
    assert_eq!(axes.len(), 1);
    assert_eq!(axes[0].options.len(), crate::ids::MAX_INSTANCE_AXIS_VALUES);
    assert_eq!(beyond, 9 - crate::ids::MAX_INSTANCE_AXIS_VALUES);
}

/// ⚠️ **Sem âncora não há «alcançável daqui»** — um mestre vigente fora da família devolve nada,
/// em vez de uma fileira que mostra opções sem dizer onde se está.
#[test]
fn a_current_master_outside_the_family_offers_nothing() {
    let fam = [
        m(1, "Casa {Size=Small, State=Idle}"),
        m(2, "Casa {Size=Big, State=Idle}"),
    ];
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
    let fam = [
        m(1, "Casa {Size=Small, State=Idle}"),
        m(2, "Casa {Size=Big, State=Run}"),
    ];
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
        .map(|i| (i, format!("Casa {{Size=S{i}}}")))
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
    let mut fam = vec![m(0, "Casa {A=1, B=1, C=1, D=1, E=1}")];
    for (i, k) in ["A", "B", "C", "D", "E"].iter().enumerate() {
        let name: String = ["A", "B", "C", "D", "E"]
            .iter()
            .map(|x| format!("{x}={}", if x == k { 2 } else { 1 }))
            .collect::<Vec<_>>()
            .join(", ");
        fam.push(m(i as u64 + 1, &format!("Casa {{{name}}}")));
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
    assert_eq!(parse_combo("Casa {Size=Small, Size=Big}"), None);
    // ⚠️ O controlo: sem a repetição, a mesma forma parseia.
    assert!(parse_combo("Casa {Size=Small, State=Big}").is_some());
}

// ── As CHAVES (Enio, 2026-08-30) ───────────────────────────────────────────────────────────────

/// ⭐⭐⭐ **A hierarquia mostra o que o objecto É.**
///
/// ⛔ Report do Enio: *«criar nomes de objetos que não exprimem o que o objeto realmente é é muito
/// estranho»* e *«os nomes ficam grandes demais e nem cabem direito na hierarquia»*. As duas
/// queixas são do mesmo defeito: eu portei os nomes `k=v` do Figma e **não portei o contêiner** que
/// lá os mantém um nível abaixo.
///
/// **Mutação que deve sangrar:** devolver o nome inteiro.
#[test]
fn the_hierarchy_shows_what_the_object_is_not_its_properties() {
    use super::display_name;
    assert_eq!(display_name("Casa {Size=Small, State=Idle}"), "Casa");
    // ⚠️ **O espaço antes da chaveta é aparado** — senão a linha desenha `"Casa "` e o realce de
    // busca mede um caractere a mais.
    assert_eq!(display_name("Casa  {Size=Small}"), "Casa");
    // Sem chaves, o nome é o nome.
    assert_eq!(display_name("Casa"), "Casa");
    assert_eq!(display_name("A=B"), "A=B");
}

/// ⛔⛔ **Sem chaves NÃO há propriedades** — e é isto que as chaves compram sobre a convenção do
/// Figma: um objecto legitimamente chamado `A=B` era lido como um eixo.
#[test]
fn a_name_without_braces_declares_no_properties() {
    assert_eq!(parse_combo("Size=Small, State=Idle"), None);
    assert_eq!(parse_combo("A=B"), None);
    // ⚠️ O controlo: com chaves, a mesma forma parseia.
    assert!(parse_combo("Casa {Size=Small, State=Idle}").is_some());
}

/// ⚠️ **O nome comum não entra nos eixos** — ele é a identidade, e misturá-lo daria uma fileira
/// chamada `Casa`.
#[test]
fn the_common_name_never_becomes_an_axis() {
    let fam = [m(1, "Casa {Size=Small}"), m(2, "Casa {Size=Big}")];
    let (axes, _) = axes_for(&fam, 1);
    assert_eq!(axes.len(), 1);
    assert_eq!(axes[0].name, "Size");
    assert_eq!(labels(&axes[0]), ["Small", "Big"]);
}

/// ⛔⛔ **O sufixo de CÓPIA sobrevive ao corte** — report do Enio com foto (2026-08-30).
///
/// O app acrescenta `(1)`, `(2)` … para desempatar nomes, e esse sufixo vem **depois** das chaves.
/// A 1.ª versão cortava a partir do `{` e comia-o: duas cópias ficavam com a mesma linha, e o
/// número que as distinguia era exactamente o que se perdia.
///
/// **Mutação que deve sangrar:** voltar a `name.split_once('{').0`.
#[test]
fn the_copy_suffix_survives_the_cut() {
    use super::display_name;
    assert_eq!(
        display_name("Casa {Size=Small, State=Idle} (1)"),
        "Casa (1)"
    );
    assert_eq!(display_name("Casa {Size=Big} (12)"), "Casa (12)");
    // ⚠️ E sem sufixo nada é acrescentado — nem um espaço no fim, que o realce de busca mediria.
    assert_eq!(display_name("Casa {Size=Big}"), "Casa");
}

/// ⭐⭐ **O selo diz QUANTAS propriedades ficaram escondidas.**
///
/// ⚠️ Ele conta **definições**, não versões — é o que o pedido diz (*«sendo o número a quantidade
/// de definições»*), e é a única coisa honesta que um número sozinho pode prometer.
///
/// **Mutação que deve sangrar:** devolver `base` sem o selo.
#[test]
fn the_badge_counts_the_hidden_definitions() {
    use super::{hidden_count, row_label};
    assert_eq!(hidden_count("Casa {Size=Small, State=Idle}"), 2);
    assert_eq!(row_label("Casa {Size=Small, State=Idle}"), "Casa *²");
    assert_eq!(
        row_label("Casa {Size=Small, State=Idle, Tag=City} (1)"),
        "Casa (1) *³"
    );
    // ⛔ Sem propriedades **não há selo** — um marcador permanentemente aceso é ruído que o artista
    // aprende a ignorar.
    assert_eq!(hidden_count("Casa"), 0);
    assert_eq!(row_label("Casa"), "Casa");
    // ⛔⛔⛔ **E um nome que NÃO PARSEIA sai INTEIRO** (auditoria de 2026-08-31, achado A1).
    //
    // ⚠️ **Esta asserção estava ao contrário, e afirmava o defeito como correcto.** O corte era
    // incondicional e o selo só conta o que o `parse_combo` aceita ⇒ `Fx {glow}` desenhava-se
    // `Fx`, **sem selo**: o texto do artista sumia da linha sem nada a dizer que sumira — que é
    // exactamente o que o doc do `row_label` promete não fazer, três linhas acima.
    //
    // *Um oráculo que codifica o defeito não é uma fixtura fraca: nenhuma mutação o mata, porque
    // ele já é a mutação.*
    for unreadable in [
        "Casa {}",
        "Casa {Size=}",
        "Fx {glow}",
        "Casa {Size=Small, Size=Big}",
    ] {
        assert_eq!(
            row_label(unreadable),
            unreadable,
            "o que não é uma combinação não é uma propriedade, e não se esconde"
        );
        assert_eq!(hidden_count(unreadable), 0);
    }
    // ⚠️ E um nome a meio da digitação passa por aqui a cada tecla (o `TextChanged` do campo de
    // nome dispara por letra) — `Casa {Size=` não pode piscar o nome do artista para fora da lista.
    assert_eq!(row_label("Casa {Size="), "Casa {Size=");
}

/// ⭐⭐⭐ **UM OBJECTO SOLTO MOSTRA O QUE DECLARA** — o report do Enio de 2026-08-31.
///
/// *«quando mudo o conteúdo entre `{}` o inspector não muda»*: as chaves eram lidas pelo selo da
/// Hierarquia e pelo [`axes_for`], que exige **duas ou mais** receitas. Sem família — um objecto
/// solto, ou uma cópia de um mestre único — nada no Inspector as lia.
///
/// ⚠️ **O oráculo é o VALOR, não a contagem de fileiras:** um construtor que devolvesse duas
/// fileiras vazias passaria num gate que só contasse.
///
/// **Mutação que deve sangrar:** `rows_for` devolver só o `axes_for`.
#[test]
fn a_lone_object_shows_what_its_name_declares() {
    use super::rows_for;
    let (rows, beyond) = rows_for(&[], 0, "Casa {Size=Small, State=Idle}");
    assert_eq!(beyond, 0);
    let got: Vec<(&str, Vec<&str>)> = rows.iter().map(|a| (a.name.as_str(), labels(a))).collect();
    assert_eq!(
        got,
        vec![("Size", vec!["Small"]), ("State", vec!["Idle"])],
        "as propriedades declaradas não chegaram ao cartão"
    );
    // ⛔ **Um valor só, e ele é o VIGENTE** — o pintor lê isso para o desenhar como texto em vez de
    // um botão aceso que não leva a lado nenhum.
    assert!(
        rows.iter()
            .all(|a| a.options.len() == 1 && a.options[0].current)
    );
    // ⚠️ E `master: 0` — não há a quem pedir uma troca. O despachante honra-o.
    assert!(rows.iter().all(|a| a.options[0].master == 0));
}

/// ⛔ **Um nome sem chaves não declara nada** — e aí o cartão não existe (a lei da F3).
#[test]
fn a_name_with_no_braces_declares_no_rows() {
    use super::rows_for;
    assert!(rows_for(&[], 0, "Casa").0.is_empty());
    assert!(rows_for(&[], 0, "Casa {}").0.is_empty());
}

/// ⭐⭐ **A PERGUNTA vence a DECLARAÇÃO na mesma chave** — nunca duas fileiras com o mesmo nome.
///
/// Com uma família a oferecer `Size`, a fileira de `Size` são os chips; o `State`, que a família
/// **não** pergunta (todos os membros o têm igual), entra como o valor declarado.
///
/// ⚠️ *Duas fileiras com o mesmo nome seriam duas respostas à mesma pergunta, e a de baixo estaria
/// sempre desactualizada.*
///
/// **Mutação que deve sangrar:** apagar o `if axes.iter().any(|a| a.name == row.name)`.
#[test]
fn a_family_question_wins_over_the_same_declared_key() {
    use super::rows_for;
    let me = "Casa {Size=Small, State=Idle}";
    let members = [m(1, me), m(2, "Casa {Size=Big, State=Idle}")];
    let (rows, _) = rows_for(&members, 1, me);
    let got: Vec<(&str, Vec<&str>)> = rows.iter().map(|a| (a.name.as_str(), labels(a))).collect();
    assert_eq!(
        got,
        vec![("Size", vec!["Small", "Big"]), ("State", vec!["Idle"])],
        "a chave que a família pergunta tem de vir dos CHIPS, e a outra do nome"
    );
}

/// ⛔ **O teto é o da tabela de ids, e o que passa dele é ESCRITO** — nunca truncado em silêncio.
///
/// **Mutação que deve sangrar:** `beyond += 1` virar `continue` seco.
#[test]
fn the_declared_rows_beyond_the_id_table_are_counted() {
    use super::rows_for;
    let name = "X {A=1, B=2, C=3, D=4, E=5, F=6}";
    let (rows, beyond) = rows_for(&[], 0, name);
    assert_eq!(rows.len(), crate::ids::MAX_INSTANCE_AXES);
    assert_eq!(beyond, 2, "as fileiras que não cabem têm de ser contadas");
}

/// ⭐ **No modo PLANO o chip mostra o que DIFERE** — o miolo das chaves, não o nome comum.
///
/// ⚠️ Colapsar pelo [`super::display_name`] daria quatro chips todos a dizer `Casa`.
///
/// **Mutação que deve sangrar:** `chip_label` devolver o nome inteiro.
#[test]
fn a_flat_chip_shows_what_differs_not_the_common_name() {
    use super::{chip_label, rows_for};
    assert_eq!(chip_label("Casa {Size=Small}"), "Size=Small");
    assert_eq!(chip_label("Casa"), "Casa");
    assert_eq!(chip_label("Casa {}"), "Casa {}");
    // A família discorda das chaves ⇒ modo plano, e os chips têm de continuar distinguíveis.
    let me = "Casa {Size=Small}";
    let members = [m(1, me), m(2, "Casa {Size=Big, State=Run}")];
    let (rows, _) = rows_for(&members, 1, me);
    assert_eq!(rows[0].name, "", "isto tinha de cair no modo plano");
    assert_eq!(labels(&rows[0]), vec!["Size=Small", "Size=Big, State=Run"]);
}

/// ⛔⛔ **O chip do modo plano NÃO colapsa duas irmãs** — o defeito medido no fluxo de 2026-08-31.
///
/// Uma variante nasce com o nome da base mais um sufixo (`… Variant`, `… (1)`), e a 1.ª versão do
/// `chip_label` devolvia só o miolo das chaves ⇒ **dois botões idênticos** na fileira que existe
/// exactamente para os separar.
///
/// **Mutação que deve sangrar:** voltar a devolver só o `inner`.
#[test]
fn a_flat_chip_never_collapses_two_sisters() {
    use super::{chip_label, rows_for};
    assert_eq!(
        chip_label("Casa {Size=Small} Variant"),
        "Size=Small Variant"
    );
    assert_eq!(chip_label("Casa {Size=Small} (1)"), "Size=Small (1)");
    assert_eq!(chip_label("Casa {Size=Small}"), "Size=Small");
    // ⭐⭐ **A família que o fluxo de facto produz** — base e variante, com o MESMO miolo. Aqui o
    // nome CURTO já as separa, então é ele que o chip leva: `Casa` e `Casa Variant`.
    //
    // ⚠️ **Report do Enio com foto (2026-08-31): *«Label dos botões emboladas»*.** O rótulo longo
    // (`Size=Small Variant`) não cabia em meia largura de painel e transbordava o botão — e ele
    // era desnecessário, porque o curto distinguia.
    let me = "Casa {Size=Small}";
    let members = [m(1, me), m(2, "Casa {Size=Small} Variant")];
    let (rows, _) = rows_for(&members, 1, me);
    let flat = rows.iter().find(|a| a.name.is_empty()).expect("modo plano");
    assert_eq!(labels(flat), vec!["Casa", "Casa Variant"]);
}

/// ⛔⛔ **E quando o CURTO colide, o chip cai no longo** — a metade sem a qual a cura acima seria
/// uma regressão.
///
/// Com `Casa {A=1}` e `Casa {B=2}` os dois nomes curtos são `Casa`: o rótulo tem de crescer, não
/// colapsar. *Uma das duas regras falha no caso da outra; a escolha é do CONJUNTO.*
///
/// **Mutação que deve sangrar:** usar `short[i]` sempre.
#[test]
fn the_flat_chip_grows_only_when_the_short_name_collides() {
    use super::rows_for;
    let me = "Casa {A=1}";
    let members = [m(1, me), m(2, "Casa {B=2}")];
    let (rows, _) = rows_for(&members, 1, me);
    let flat = rows.iter().find(|a| a.name.is_empty()).expect("modo plano");
    let seen = labels(flat);
    assert_eq!(
        seen,
        vec!["A=1", "B=2"],
        "o curto colidia e o chip tinha de crescer"
    );
    assert_ne!(seen[0], seen[1]);
}

/// ⭐⭐⭐ **UMA VARIANTE NASCE COM UM VALOR PRÓPRIO** — report do Enio, 2026-08-31:
/// *«Variant deveria ser Size. Nos botões deveríamos ter Small e Big.»*
///
/// Enquanto ela herdava `<base> Variant`, as duas receitas declaravam a MESMA combinação, o eixo
/// `Size` ficava com uma resposta só e caía, e a família descia ao modo plano — a mostrar NOMES.
///
/// **Mutação que deve sangrar:** `variant_name` devolver `None` sempre.
#[test]
fn a_new_variant_is_born_with_a_value_of_its_own() {
    use super::variant_name;
    assert_eq!(
        variant_name("Casa {Size=Small}", &["Casa {Size=Small}".into()]),
        Some("Casa {Size=Small 2}".into())
    );
    // ⚠️ **Só a PRIMEIRA chave muda** — as outras são a identidade que a variante herda.
    assert_eq!(
        variant_name(
            "Casa {Size=Small, State=Idle}",
            &["Casa {Size=Small, State=Idle}".into()]
        ),
        Some("Casa {Size=Small 2, State=Idle}".into())
    );
    // ⛔ **Sem chaves não há valor a numerar** — o chamador mantém o sufixo `Variant` do Unity.
    assert_eq!(variant_name("Badge", &[]), None);
}

/// ⛔⛔ **E ela não repete uma combinação que já existe** — senão o eixo volta a colapsar, que é o
/// defeito de origem.
///
/// ⚠️ **A comparação é pela COMBINAÇÃO, não pelo nome**: duas receitas chamadas de forma diferente
/// e a declarar o mesmo `{Size=Small 2}` fariam o `Size` cair na mesma.
///
/// **Mutação que deve sangrar:** ignorar `taken`.
#[test]
fn a_new_variant_never_repeats_a_combination_that_exists() {
    use super::variant_name;
    let taken = vec![
        "Casa {Size=Small}".to_string(),
        // ⚠️ Nome DIFERENTE, combinação já usada — é isto que o gate mede.
        "Outra {Size=Small 2}".to_string(),
    ];
    assert_eq!(
        variant_name("Casa {Size=Small}", &taken),
        Some("Casa {Size=Small 3}".into())
    );
}

/// ⭐⭐ **E o resultado é uma PERGUNTA de verdade** — a metade que liga a lei ao que o artista vê.
///
/// ⚠️ Sem ela, os dois gates acima provam um `format!` e não o produto: o que interessa é que a
/// família resultante deixa de cair no modo plano.
#[test]
fn the_born_variant_makes_the_axis_a_real_question() {
    use super::{rows_for, variant_name};
    let base = "Casa {Size=Small}";
    let variant = variant_name(base, &[base.to_string()]).expect("a variante");
    let members = [m(1, base), m(2, &variant)];
    let (rows, _) = rows_for(&members, 1, base);
    assert_eq!(rows.len(), 1, "devia ser UMA fileira, a do eixo: {rows:?}");
    assert_eq!(rows[0].name, "Size", "a fileira caiu no modo plano");
    assert_eq!(labels(&rows[0]), vec!["Small", "Small 2"]);
    assert!(rows[0].options[0].current);
}

/// ⭐⭐⭐ **Reescrever UM valor não toca no resto do nome** — a porta que o campo do cartão usa.
///
/// Report do Enio (2026-08-31, a quarta vez): *«Que inferno!!!»*. Autorar o valor obrigava a
/// seleccionar a RECEITA, e ele escrevia as chaves na cópia. Hoje o cartão deixa mudar o valor
/// onde ele se lê, e o texto que se escreve vem por aqui.
///
/// **Mutação que deve sangrar:** devolver o nome sem trocar nada.
#[test]
fn one_value_is_rewritten_and_the_rest_of_the_name_survives() {
    use super::with_value;
    assert_eq!(
        with_value("Casa {Size=Small 2, State=Idle}", "Size", "Big"),
        Some("Casa {Size=Big, State=Idle}".into())
    );
    // ⚠️ O que está FORA das chaves é do artista — o nome comum e o sufixo de cópia ficam.
    assert_eq!(
        with_value("Casa {Size=Small} (1)", "Size", "Big"),
        Some("Casa {Size=Big} (1)".into())
    );
    // ⛔ Não se inventa uma propriedade a partir de uma edição.
    assert_eq!(with_value("Casa {Size=Small}", "State", "Idle"), None);
    assert_eq!(with_value("Casa", "Size", "Big"), None);
}

/// ⛔⛔ **E a gramática é defendida na PORTA** — um valor que a partiria é recusado, não gravado.
///
/// ⚠️ Sem esta metade, escrever `Big=2` no campo produziria `{Size=Big=2}`, que o `parse_combo`
/// deixa de reconhecer: *a propriedade desapareceria da linha e do cartão de uma vez, por causa de
/// uma tecla*.
///
/// **Mutação que deve sangrar:** apagar a guarda dos caracteres.
#[test]
fn a_value_that_would_break_the_grammar_is_refused() {
    use super::{parse_combo, with_value};
    for bad in ["", "   ", "Big=2", "Big,Small", "Big{", "Big}"] {
        assert_eq!(
            with_value("Casa {Size=Small}", "Size", bad),
            None,
            "«{bad}» devia ser recusado"
        );
    }
    // ⚠️ E o que passa continua a parsear — a guarda não pode deixar entrar o que ela existe para
    // barrar.
    let ok = with_value("Casa {Size=Small}", "Size", "  Big  ").expect("aparado e aceite");
    assert_eq!(ok, "Casa {Size=Big}");
    assert!(parse_combo(&ok).is_some());
}
