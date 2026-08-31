//! ⭐⭐⭐ **OS EIXOS DE PROPRIEDADE de uma família de variantes** — *«esta cópia é `Size=Small`,
//! `State=Idle`»*, e cada eixo é uma fileira que escolhe.
//!
//! # O buraco que isto fecha
//!
//! O cartão geral desenha uma fileira **plana** com o `Name` cru de cada variante: `Hero Small
//! Idle`, `Hero Small Run`, `Hero Big Idle`, `Hero Big Run` — quatro chips, e o artista tem de
//! **ler os nomes** para descobrir que há duas perguntas independentes. Com seis versões são seis
//! chips e nenhuma estrutura; com doze é uma parede.
//!
//! ⚠️ **A lei existia, e só o sistema VETORIAL a tinha** (`vec_variants.rs`) — era o item que a
//! fatia F4.6c manda portar **antes** de apagar ~3 000 LOC de maquinaria duplicada. Ela é
//! puramente sobre NOMES e ids, e é por isso que se re-hospeda sem tocar em nada do vetor.
//!
//! # ⚠️ Sem componente novo: a fonte é o `Name`
//!
//! ⛔ Um `VariantAxisSet` guardado seria a **segunda** resposta a *«que versões existem?»* — e
//! divergiria no dia em que alguém renomeasse um mestre. O eixo lê-se do nome, e o gesto de o
//! autorar é renomear na Hierarquia, que já existe. *A estrutura é o que a estrutura diz.*
//!
//! # ⚠️ Duas modalidades, uma representação
//!
//! Quando os nomes **não** são combinações (ou discordam nas chaves), a família devolve **um** eixo
//! chamado `Variant` com os nomes crus — que é exactamente a fileira que o cartão já desenhava. ⇒
//! o painel tem um caminho só, e a modalidade é um facto dos dados. *Duas representações para a
//! mesma fileira seriam dois sítios a discordar sobre o que está escolhido.*

use super::inspector_model_instance::VariantChoice;
use crate::ids;

/// Uma pergunta que a família faz — `Size`, `State`, ou o `Variant` do modo plano.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct VariantAxis {
    /// O rótulo da fileira. `"Variant"` no modo plano.
    pub name: String,
    /// As respostas alcançáveis **daqui**. Ver [`axes_for`].
    pub options: Vec<VariantChoice>,
}

/// ⭐⭐⭐ **O NOME COMUM** — o que o objecto É, sem as propriedades.
///
/// `"Casa {Size=Small, State=Idle}"` → `"Casa"`. Sem chaves, o nome inteiro.
///
/// # ⛔⛔ Porque as chaves existem (Enio, 2026-08-30)
///
/// A 1.ª versão lia as propriedades do nome **inteiro** — `Size=Small, State=Idle` **era** o nome
/// do objecto, que é o que o Figma faz. O report foi imediato e está certo em dois pontos:
///
/// 1. *«criar nomes de objetos que não exprimem o que o objeto realmente é é muito estranho»* — o
///    objecto é uma **Casa**; `Size=Small` é uma propriedade dela.
/// 2. *«os nomes ficam grandes demais e nem cabem direito na hierarquia»*.
///
/// ⚠️ **E o Figma não tem o 2.º problema por uma razão que eu não portei:** lá aqueles nomes vivem
/// **dentro de um contêiner** (o *component set*), cujo nome é o comum — na lista de camadas
/// vê-se `Casa` fechado. Eu portei os nomes e **não o contêiner**, e foi isso que pôs os nomes
/// compridos no primeiro nível.
///
/// ⇒ as chaves fazem o trabalho do contêiner **sem o contêiner**: o nome comum viaja no mesmo
/// campo, e a hierarquia mostra só ele. ⭐ E resolvem uma ambiguidade que o Figma tem: um objecto
/// legitimamente chamado `A=B` era lido como eixo, e agora não é.
#[must_use]
pub fn display_name(name: &str) -> String {
    // ⛔⛔ **Tira o VÃO das chaves, e guarda os DOIS lados** (report do Enio com foto, 2026-08-30).
    //
    // A 1.ª versão cortava a partir do `{` — e o sufixo de cópia que o app acrescenta vem **depois**
    // dele: `Casa {Size=Small, State=Idle} (1)` desenhava-se `Casa`, **comendo o `(1)`**. Duas
    // cópias ficavam com a mesma linha, e o número que as distinguia era exactamente o que se
    // perdia. *Cortar por um delimitador de abertura assume que ele é o fim da linha.*
    let Some((head, rest)) = name.split_once('{') else {
        return name.to_string();
    };
    let tail = rest.split_once('}').map_or("", |(_, t)| t);
    let mut out = String::with_capacity(name.len());
    out.push_str(head.trim_end());
    let tail = tail.trim();
    if !tail.is_empty() {
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(tail);
    }
    out
}

/// ⭐⭐ **Quantas propriedades este nome esconde** — o número do selo `*²`.
///
/// ⚠️ Ele conta as **definições**, não as versões: `Casa {Size=Small, State=Idle}` tem duas coisas
/// escondidas, e é isso que o selo promete. `0` quando não há chaves ou o que está dentro delas não
/// é uma combinação.
#[must_use]
pub fn hidden_count(name: &str) -> usize {
    parse_combo(name).map_or(0, |c| c.len())
}

/// Os algarismos sobrescritos de `n`.
fn superscript(n: usize) -> String {
    const DIGITS: [char; 10] = ['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'];
    n.to_string()
        .chars()
        .filter_map(|c| c.to_digit(10))
        .map(|d| DIGITS[d as usize])
        .collect()
}

/// ⭐⭐⭐ **O RÓTULO da linha da hierarquia** — o nome curto mais o selo do que ficou escondido.
///
/// `Casa {Size=Small, State=Idle} (1)` → `Casa (1) *²`.
///
/// ⚠️ **O selo existe porque o corte esconde informação**, e esconder sem dizer é pior que mostrar
/// comprido: sem ele, duas linhas com propriedades diferentes e o mesmo nome comum leem-se como
/// duplicados de um erro. *O `*²` não diz QUAIS — diz que há duas, e onde ir ver.*
#[must_use]
pub fn row_label(name: &str) -> String {
    let base = display_name(name);
    match hidden_count(name) {
        0 => base,
        n => format!("{base} *{}", superscript(n)),
    }
}

/// **`"Casa {Size=Small, State=Idle}"` → `[("Size","Small"), ("State","Idle")]`.**
///
/// `None` quando não há chaves, ou quando o que está dentro delas não é uma combinação: sem `=`,
/// com `=` a mais, com um lado vazio, ou com uma chave repetida.
///
/// ⚠️ **Deliberadamente ESTRITA** — um nome meio-parseado daria um eixo com um valor só, que é uma
/// fileira que não escolhe nada. É a porta que decide entre as duas modalidades.
#[must_use]
pub fn parse_combo(name: &str) -> Option<Vec<(String, String)>> {
    // ⚠️ **Só o que está entre chaves.** Sem elas não há propriedades — e é isso que faz um objecto
    // chamado `A=B` ser um objecto chamado `A=B`.
    let (_, rest) = name.split_once('{')?;
    let inner = rest
        .strip_suffix('}')
        .or_else(|| rest.split_once('}').map(|(i, _)| i))?;
    let mut out = Vec::new();
    for part in inner.split(',') {
        let mut it = part.splitn(2, '=');
        let k = it.next()?.trim();
        let v = it.next()?.trim();
        if k.is_empty() || v.is_empty() || v.contains('=') {
            return None;
        }
        // ⚠️ **Uma chave REPETIDA não é uma combinação** — `"Size=Small, Size=Big"` daria duas
        // fileiras chamadas `Size` com as mesmas opções, e clicar numa mexeria na outra.
        if out.iter().any(|(existing, _)| existing == k) {
            return None;
        }
        out.push((k.to_string(), v.to_string()));
    }
    (!out.is_empty()).then_some(out)
}

/// O valor do eixo `key` nesta combinação.
fn value_of<'a>(combo: &'a [(String, String)], key: &str) -> Option<&'a str> {
    combo
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.as_str())
}

/// As chaves que **todos** os membros declaram, na ordem do primeiro — ou `None` se discordarem.
///
/// ⚠️ A exigência é de **IGUALDADE de conjunto**, e não de interseção: com `{Size}` num membro e
/// `{Size, State}` noutro, uma interseção esconderia o `State` do segundo e o artista perderia um
/// eixo sem nada a dizer porquê. Discordar cai no modo plano, onde tudo aparece.
fn shared_keys(combos: &[Option<Vec<(String, String)>>]) -> Option<Vec<String>> {
    let first = combos.first()?.as_ref()?;
    let keys: Vec<String> = first.iter().map(|(k, _)| k.clone()).collect();
    let same = |c: &Option<Vec<(String, String)>>| {
        c.as_ref()
            .is_some_and(|c| c.len() == keys.len() && keys.iter().all(|k| value_of(c, k).is_some()))
    };
    combos.iter().all(same).then_some(keys)
}

/// ⭐⭐ **As fileiras desta família, e o que a tabela de ids não endereçou.**
///
/// `members` são `(StableId do mestre, Name dele)`, e `me` é o mestre vigente desta cópia.
///
/// ⚠️ **Menos de dois membros não é um conjunto** — uma fileira com um chip só é uma escolha que
/// não escolhe, e o cartão não a pinta.
///
/// ⚠️ **O teto é de TABELA DE IDS, não do catálogo**: as variantes que passam daqui continuam a
/// existir e a ser alcançáveis; o que se perde é o chip. ⇒ o excedente é **contado e escrito**.
#[must_use]
pub fn axes_for(members: &[(u64, String)], me: u64) -> (Vec<VariantAxis>, usize) {
    if members.len() < 2 {
        return (Vec::new(), 0);
    }
    // ⛔⛔ **A ÂNCORA é a mesma pergunta nos DOIS modos** (auditoria de 2026-08-30). Ela estava só
    // no multi-eixo, e o modo plano devolvia a fileira inteira **sem nenhum chip aceso** — que é
    // exactamente *«mostra as opções e esconde a resposta»*, a frase que o pintor tem escrita.
    if !members.iter().any(|(id, _)| *id == me) {
        return (Vec::new(), 0);
    }
    let combos: Vec<Option<Vec<(String, String)>>> =
        members.iter().map(|(_, n)| parse_combo(n)).collect();
    let mut axes = match shared_keys(&combos) {
        Some(keys) => multi_axis(members, &combos, &keys, me),
        None => Vec::new(),
    };
    // ⛔⛔⛔ **O MODO PLANO É A REDE, e sem ele isto era uma REGRESSÃO** (auditoria de 2026-08-30).
    //
    // A lei da alcançabilidade pode esvaziar TODOS os eixos: com `Size=Small, State=Idle` e
    // `Size=Big, State=Run` — duas versões, nada mais — nenhuma é alcançável **num passo**, os dois
    // eixos caem por ter um valor só, e o cartão ficava **sem fileira nenhuma**. *O artista tinha
    // duas versões do componente e nenhuma superfície para trocar*, quando a fileira plana de
    // ontem as mostrava as duas.
    //
    // ⇒ quando a matriz é esparsa demais para perguntas, a família volta a ser uma **lista**. É
    // pior de ler e é **alcançável**, e essa é a ordem certa das duas.
    if axes.is_empty() {
        axes = flat_axis(members, me);
    }
    // ⚠️ **O corte é aqui e é ESCRITO** — o `populate` regista `AXES × VALUES` chips e o roteador
    // varre o mesmo intervalo.
    let mut beyond = 0;
    for (a, ax) in axes.iter_mut().enumerate() {
        if a >= ids::MAX_INSTANCE_AXES {
            // ⚠️ **O vigente não conta como perdido** — ele aparece como opção em TODO eixo, e
            // contá-lo dizia ao artista *«mais uma versão»* sobre ele próprio (auditoria de
            // 2026-08-30).
            beyond += ax.options.iter().filter(|o| !o.current).count();
            continue;
        }
        if ax.options.len() > ids::MAX_INSTANCE_AXIS_VALUES {
            // ⛔⛔ **O VIGENTE sobrevive ao corte** (auditoria de 2026-08-30). Truncar às cegas
            // deixava a fileira **sem nenhum chip aceso** quando o mestre corrente estava depois do
            // teto — e o doc do pintor diz o que isso significa: *«mostra as opções e esconde a
            // resposta»*.
            //
            // ⚠️ **A lei perdeu-se no PORTE**: o original vetorial cura o mesmo caso com
            // `ax.selected = ax.selected.min(cap - 1)`, a única correcção que aquele ficheiro
            // carregava, e ela não veio. ⭐ Aqui a cura é melhor que a de lá: o vetorial acende o
            // chip **errado**, e este mantém o certo dentro da janela.
            let cap = ids::MAX_INSTANCE_AXIS_VALUES;
            beyond += ax.options.len() - cap;
            let cur = ax.options.iter().position(|o| o.current);
            if let Some(i) = cur.filter(|i| *i >= cap) {
                let mine = ax.options.remove(i);
                ax.options.truncate(cap - 1);
                ax.options.push(mine);
            } else {
                ax.options.truncate(cap);
            }
        }
    }
    axes.truncate(ids::MAX_INSTANCE_AXES);
    (axes, beyond)
}

/// O modo de **propriedades**: uma fileira por chave, com os valores alcançáveis daqui.
fn multi_axis(
    members: &[(u64, String)],
    combos: &[Option<Vec<(String, String)>>],
    keys: &[String],
    me: u64,
) -> Vec<VariantAxis> {
    let Some(mine) = members
        .iter()
        .position(|(id, _)| *id == me)
        .and_then(|i| combos[i].as_ref())
    else {
        // O mestre vigente não está na família (ou o nome dele não parseia) — sem âncora não há
        // «alcançável daqui», e uma fileira sem vigente mostraria opções sem dizer onde se está.
        return Vec::new();
    };
    let mut axes = Vec::new();
    for key in keys {
        let mut options: Vec<VariantChoice> = Vec::new();
        for (i, (id, _)) in members.iter().enumerate() {
            let Some(c) = combos[i].as_ref() else {
                continue;
            };
            // ⚠️ **Alcançável = difere de mim SÓ neste eixo.** É o que faz de cada chip uma escolha
            // que chega a algum lado, e o que apaga os buracos da matriz sem um estado de erro.
            if !keys
                .iter()
                .all(|k| k == key || value_of(c, k) == value_of(mine, k))
            {
                continue;
            }
            let Some(v) = value_of(c, key) else { continue };
            if options.iter().any(|o| o.label == v) {
                continue;
            }
            options.push(VariantChoice {
                master: *id,
                label: v.to_string(),
                current: *id == me,
            });
        }
        // Um eixo com um valor só não é uma pergunta.
        if options.len() > 1 {
            axes.push(VariantAxis {
                name: key.clone(),
                options,
            });
        }
    }
    axes
}

/// ⭐⭐⭐ **AS FILEIRAS DO CARTÃO DE PROPRIEDADES** — o que a família PERGUNTA mais o que o nome
/// DECLARA.
///
/// # ⛔⛔ O buraco que isto fecha (report do Enio, 2026-08-31)
///
/// *«quando mudo o conteúdo entre `{}` o inspector não muda»* — e estava certo. As chaves tinham
/// **dois leitores** e só isso: o selo da Hierarquia, e o [`axes_for`], que exige uma família de
/// **duas ou mais receitas**. Fora desse caso — um objecto solto, ou uma cópia de um mestre único —
/// o Inspector não lia as chaves de lado nenhum, e reescrevê-las não mudava um pixel.
///
/// *Uma declaração sem leitor é decoração*, e esta tinha um selo na Hierarquia a prometer que
/// alguém a lia.
///
/// ⚠️ **A pergunta e a declaração são coisas diferentes**, e o cartão mostra as duas:
///
/// | de onde vem | o que é | como se pinta |
/// |---|---|---|
/// | [`axes_for`] | *«que outras versões existem?»* | chips que **trocam** |
/// | [`declared_axes`] | *«o que este objecto DIZ que é»* | o valor, em texto |
///
/// ⚠️ **A declaração NÃO substitui a pergunta** — uma chave que a família já oferece fica com os
/// chips; só as que sobram entram como texto. *Duas fileiras com o mesmo nome seriam duas respostas
/// à mesma pergunta, e a de baixo estaria sempre desactualizada.*
///
/// `declared_by` é o nome de quem DECLARA: o do mestre quando isto é uma cópia (a propriedade é do
/// componente, não do exemplar), e o próprio nome quando não há mestre nenhum.
#[must_use]
pub fn rows_for(
    members: &[(u64, String)],
    me: u64,
    declared_by: &str,
) -> (Vec<VariantAxis>, usize) {
    let (mut axes, mut beyond) = axes_for(members, me);
    for row in declared_axes(declared_by) {
        if axes.iter().any(|a| a.name == row.name) {
            continue;
        }
        // ⚠️ **O teto é o da TABELA DE IDS** e vale para as duas espécies de fileira: uma fileira de
        // texto não regista chips, mas conta para a altura do cartão e para a ordem — e um cartão
        // que cresce sem limite é a parede que os eixos existem para evitar.
        if axes.len() >= ids::MAX_INSTANCE_AXES {
            beyond += 1;
            continue;
        }
        axes.push(row);
    }
    (axes, beyond)
}

/// ⭐⭐ **As propriedades que ESTE nome declara** — uma fileira por chave, com **um** valor.
///
/// `"Casa {Size=Small, State=Idle}"` → duas fileiras, `Size` = `Small` e `State` = `Idle`.
///
/// ⚠️ **O `master` é `0` e o `current` é `true`**: não há para onde ir. O cartão pinta uma fileira
/// de um valor como TEXTO — sem botão, sem hit-rect —, que é a mesma lei do chip único (*um valor
/// que não leva a lado nenhum não é oferecido*). ⛔ Pintá-la como botão aceso convidaria ao clique
/// que não faz nada, que é a definição de um controlo morto.
#[must_use]
pub fn declared_axes(name: &str) -> Vec<VariantAxis> {
    parse_combo(name).map_or_else(Vec::new, |combo| {
        combo
            .into_iter()
            .map(|(k, v)| VariantAxis {
                name: k,
                options: vec![VariantChoice {
                    master: 0,
                    label: v,
                    current: true,
                }],
            })
            .collect()
    })
}

/// ⭐ **O rótulo de um chip do modo PLANO** — o que distingue esta versão das irmãs.
///
/// ⚠️ **Não é o [`display_name`]**: no modo plano a família discorda das chaves, e o nome comum é o
/// mesmo em todas — colapsá-lo daria quatro chips a dizer `Casa`. ⇒ o chip leva o **miolo** das
/// chaves quando ele existe (`Size=Small, State=Idle`), e o nome inteiro quando não existe.
#[must_use]
pub fn chip_label(name: &str) -> String {
    match name.split_once('{').and_then(|(_, r)| r.split_once('}')) {
        Some((inner, _)) if !inner.trim().is_empty() => inner.trim().to_string(),
        _ => name.to_string(),
    }
}

/// O modo **plano**: uma fileira `Variant` com os membros tal como se chamam.
///
/// ⚠️ É a fileira que o cartão já desenhava antes dos eixos — e continua a ser o caminho de omissão
/// para toda família cujos nomes não sejam combinações.
fn flat_axis(members: &[(u64, String)], me: u64) -> Vec<VariantAxis> {
    vec![VariantAxis {
        // ⚠️ **VAZIO, e quem o nomeia é o PAINEL** (HR-15, auditoria de 2026-08-30): um `"Variant"`
        // aqui é string de UI em inglês numa camada que o `hr15_no_hardcoded_ui_strings` **não
        // varre** — ele lê `widget/` e `ph2d-panel-*`. ⭐ É a escolha que o original vetorial já
        // tinha feito, com a chave de i18n do lado do painel, e o porte tinha-a trocado por um
        // literal. *Uma regra fora do caminho de quem executa não existe.*
        name: String::new(),
        options: members
            .iter()
            .map(|(id, name)| VariantChoice {
                master: *id,
                // ⚠️ **O miolo das chaves, não o nome cru** — ver [`chip_label`]: no modo plano o
                // nome comum é o mesmo em toda a família, e o chip tem de mostrar o que difere.
                label: chip_label(name),
                current: *id == me,
            })
            .collect(),
    }]
}

#[cfg(test)]
#[path = "variant_axes_tests.rs"]
mod tests;
