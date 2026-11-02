//! O ciclo do clique no canvas: onde COMEÇA (a seleção, salvo um ícone por cima) e o degrau
//! de cada clique. Puro, sobre os bits da pilha sob o dedo, que a shell monta e ordena
//! (`pick_order::descendants_first`).

/// ⭐⭐⭐ **ONDE O CICLO COMEÇA: o primeiro clique é de quem JÁ ESTÁ SELECIONADO** (Enio,
/// 2026-08-26).
///
/// > *«Como os objetos filhos ficam com um z-index relativamente maior que o pai, quando tentamos
/// > arrastar o pai (objeto previamente vazio) selecionamos um filho. Precisamos que a preferência
/// > do primeiro clique seja do objeto que está selecionado na hierarquia e depois a cada clique a
/// > seleção passa a ciclar.»*
///
/// # ⚠️ Ela NÃO revoga a lei do contêiner — escolhe outra coisa
///
/// [`descendants_first`] responde *«em que ORDEM os candidatos ficam»*, e continua a valer: um
/// clique dentro de um grupo que ainda não está selecionado pega o filho, como no Figma. Esta
/// função responde *«por onde o ciclo COMEÇA»*, e a resposta é o objeto que o artista já escolheu
/// — porque o gesto seguinte a escolher um objeto é **mexer nele**, e pedir a um artista que
/// descubra uma cadência de cliques para voltar ao que ele acabou de selecionar é o mesmo defeito
/// que a lei do contêiner curou, do outro lado.
///
/// ⚠️ **É só o PRIMEIRO clique.** A lista devolvida é a mesma, e o ciclo dos cliques seguintes
/// anda a partir daqui — quem quer o filho por baixo continua a chegar lá.
///
/// # ⚠️ `inside_its_gizmo` — o caso em que o selecionado não está nos hits
///
/// A caixa de um objeto vazio é um quadrado e o anel dele é o disco **inscrito**: premir numa quina
/// da caixa é premir o gizmo dele e **não** o corpo dele, e sem esta metade o clique caía no filho
/// por baixo. ⇒ quando o press é um *Translate* no gizmo primário, o selecionado entra na lista
/// (no FIM, para a ordem de camada dos outros ficar intacta) e o ciclo começa nele. *É a mesma lei
/// que já dizia «sem nada sob o cursor, cai na seleção atual» (Enio, 2026-07-09), com algo sob o
/// cursor.*
///
/// ⛔ **NUNCA num clique com modificador**: `Shift`/`Cmd` estão a curar a seleção, e preferir o
/// primário faria o `Shift`+clique num filho alternar o PAI. Quem chama passa `false` aí.
pub fn start_on_selection(
    hits: &mut Vec<u64>,
    selected: Option<u64>,
    inside_its_gizmo: bool,
) -> usize {
    let Some(sel) = selected else {
        return 0;
    };
    if let Some(i) = hits.iter().position(|&b| b == sel) {
        return i;
    }
    if inside_its_gizmo && !hits.is_empty() {
        hits.push(sel);
        return hits.len() - 1;
    }
    0
}

/// ⭐ **O que se VÊ por cima ganha da seleção** (dono 01/11): a seleção só fica com o press se o que
/// está acima dela na pilha é dela (`is_inside`: um filho que a desenha por cima — a lei de 26/08,
/// arrastar o grupo); qualquer outra peça por cima, e todo ícone, leva o press. Fora da pilha (o
/// press no gizmo dela) a seleção fica.
pub fn top_beats_selection(
    hits: &[u64],
    selected: Option<u64>,
    icons: &[u64],
    is_inside: impl Fn(u64) -> bool,
) -> Option<u64> {
    let sel = selected?;
    let Some(i) = hits.iter().position(|&h| h == sel) else {
        return Some(sel);
    };
    hits[..i]
        .iter()
        .all(|&h| !icons.contains(&h) && is_inside(h))
        .then_some(sel)
}

/// O que um press nu faz com a pilha sob o dedo.
#[derive(Debug, PartialEq, Eq)]
pub struct CycleStep {
    /// A seleção no PRESS — é ela que o arrasto leva.
    pub now: Option<u64>,
    /// A seleção se o press acabar em CLIQUE (soltar sem arrastar): o degrau seguinte da pilha.
    pub on_click: Option<u64>,
}

/// ⭐⭐ **UM CLIQUE POR DEGRAU** (dono 01/11) — e o arrasto leva sempre o que já está escolhido.
///
/// - O selecionado está na pilha ⇒ o press fica com ele (arrastar move-o, a lei de 26/08 de
///   [`start_on_selection`]); se o ciclo está `armed` (o clique anterior foi nesta MESMA pilha, com
///   esta seleção), soltar sem arrastar desce para o seguinte. Desarmado (veio da Hierarchy), o
///   primeiro clique fica nele.
/// - Fora da pilha ⇒ o de cima.
///
/// ⛔ Até 01/11 o press contava e só os ímpares desciam: dois cliques por degrau.
pub fn cycle_step(hits: &[u64], selected: Option<u64>, armed: bool) -> CycleStep {
    let Some(&top) = hits.first() else {
        return CycleStep {
            now: None,
            on_click: None,
        };
    };
    match selected.and_then(|s| hits.iter().position(|&b| b == s)) {
        Some(i) => CycleStep {
            now: Some(hits[i]),
            on_click: armed.then(|| hits[(i + 1) % hits.len()]),
        },
        None => CycleStep {
            now: Some(top),
            on_click: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// O ícone e a «Pile» sobre o Card selecionado levam o press; o filho do grupo selecionado
    /// não; com o de cima já escolhido, o ciclo desce. *Mutação: devolver `selected` sempre.*
    #[test]
    fn what_is_seen_on_top_beats_the_selection_under_it() {
        let (cam, pile, card, filho) = (7, 8, 9, 10);
        let dentro = |h| h == filho;
        let hits = [cam, pile, card];
        assert_eq!(top_beats_selection(&hits, Some(card), &[cam], dentro), None);
        assert_eq!(
            top_beats_selection(&[pile, card], Some(card), &[], dentro),
            None
        );
        assert_eq!(cycle_step(&hits, None, true).now, Some(cam));
        assert_eq!(
            top_beats_selection(&hits, Some(cam), &[cam], dentro),
            Some(cam)
        );
        assert_eq!(
            top_beats_selection(&[filho, card], Some(card), &[], dentro),
            Some(card)
        );
        assert_eq!(
            top_beats_selection(&[filho, card], Some(card), &[filho], dentro),
            None
        );
        assert_eq!(
            top_beats_selection(&[pile], Some(card), &[], dentro),
            Some(card)
        );
    }

    /// ⭐⭐⭐ **O primeiro clique é de quem já está selecionado** (report do Enio, 2026-08-26).
    ///
    /// ⚠️ E a lista **não** é reordenada: o ciclo dos cliques seguintes continua a alcançar o
    /// filho, que é a lei do contêiner intacta.
    ///
    /// (Mutação: devolver sempre `0` ⇒ RED.)
    #[test]
    fn the_first_click_starts_on_what_is_already_selected() {
        let (parent, child, _) = (1_u64, 2_u64, 3_u64);
        let mut hits = vec![child, parent];
        assert_eq!(
            start_on_selection(&mut hits, Some(parent), false),
            1,
            "o ciclo comecou no filho — arrastar o pai selecionado seleciona um filho"
        );
        assert_eq!(hits, vec![child, parent], "a lista foi reordenada");
    }

    /// **Sem seleção, ou com uma seleção que não está sob o cursor, o ciclo começa no topo.**
    ///
    /// (Mutação: devolver `hits.len() - 1` no caso de fora ⇒ RED.)
    #[test]
    fn a_selection_that_is_not_under_the_cursor_changes_nothing() {
        let (parent, child, grand) = (1_u64, 2_u64, 3_u64);
        let mut hits = vec![child, parent];
        assert_eq!(start_on_selection(&mut hits, None, false), 0);
        assert_eq!(start_on_selection(&mut hits, Some(grand), false), 0);
        assert_eq!(
            hits,
            vec![child, parent],
            "a lista mudou sem o gizmo primario"
        );
    }

    /// ⚠️ **Premir a QUINA da caixa de um objeto vazio** — o gizmo é dele, o corpo (o disco
    /// inscrito) não está sob o cursor, e sem esta metade o clique caía no filho.
    ///
    /// (Mutação: ignorar `inside_its_gizmo` ⇒ RED.)
    #[test]
    fn pressing_inside_its_own_gizmo_brings_the_selection_into_the_cycle() {
        let (parent, child, _) = (1_u64, 2_u64, 3_u64);
        let mut hits = vec![child];
        assert_eq!(start_on_selection(&mut hits, Some(parent), true), 1);
        assert_eq!(
            hits,
            vec![child, parent],
            "o selecionado tem de entrar no FIM — a ordem de camada dos outros fica intacta"
        );
    }

    /// ⛔ **Uma lista VAZIA continua vazia** — o caminho do clique no nada, que limpa a seleção.
    /// Empurrar o selecionado aqui faria um clique fora de tudo re-selecionar o que já estava.
    #[test]
    fn an_empty_list_is_left_alone() {
        let (parent, _, _) = (1_u64, 2_u64, 3_u64);
        let mut hits: Vec<u64> = Vec::new();
        assert_eq!(start_on_selection(&mut hits, Some(parent), true), 0);
        assert!(hits.is_empty(), "o clique no nada passou a selecionar algo");
    }

    /// ⭐⭐ **Um clique por degrau, a pilha inteira e de volta ao topo** (dono 01/11) — e o press de
    /// cada clique fica com o já escolhido, para o arrasto o levar.
    #[test]
    fn each_click_goes_one_step_down_the_stack() {
        let stack = [1u64, 2, 3];
        let mut sel = None;
        let mut armed = false;
        let mut seen = Vec::new();
        for _ in 0..4 {
            let step = cycle_step(&stack, sel, armed);
            let pressed = step.now;
            sel = step.on_click.or(pressed);
            armed = true;
            seen.push(sel.unwrap());
        }
        assert_eq!(
            seen,
            vec![1, 2, 3, 1],
            "o ciclo nao desce um degrau por clique"
        );
        // O press de um clique armado fica no selecionado — é ele que o arrasto leva.
        assert_eq!(cycle_step(&stack, Some(2), true).now, Some(2));
    }

    /// **Escolhido na Hierarchy (desarmado), o 1.º clique fica nele** (lei de 26/08); fora da pilha, o
    /// de cima.
    #[test]
    fn a_disarmed_click_keeps_the_selection_and_outside_picks_the_top() {
        let stack = [1u64, 2, 3];
        assert_eq!(
            cycle_step(&stack, Some(3), false),
            CycleStep {
                now: Some(3),
                on_click: None
            }
        );
        assert_eq!(cycle_step(&stack, Some(9), true).now, Some(1));
        assert_eq!(cycle_step(&[], Some(1), true).now, None);
    }
}
