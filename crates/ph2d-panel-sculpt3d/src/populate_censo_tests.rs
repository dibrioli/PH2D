//! ⛔⛔⛔ **O CENSO QUE FAZ A OITAVA OCORRÊNCIA SER IMPOSSÍVEL.**
//!
//! # O defeito, sete vezes
//!
//! Uma fileira de chips vive em **três** sítios: o `paint` desenha-a e
//! hit-indexa-a, o `event` dá-lhe um braço, e o `populate` **regista-a**. Quem
//! esquece o terceiro entrega um controlo pintado, com moldura, com realce ao
//! passar o rato — e cujo clique é **descartado em silêncio**.
//!
//! ⚠️⚠️ **E o sintoma que o artista vê é o PIOR possível:** ele não lê *«o botão
//! não responde»*, ele lê *«a ferramenta não funciona»* — porque o pincel fica
//! no modo de omissão e as outras leis parecem partidas. Foi exactamente esse o
//! report do dono em 2026-09-15 (*«os outros 2 botões ainda não funcionam»*),
//! sobre `13` chips mortos em duas famílias.
//!
//! # Porque nenhum gate o via
//!
//! Os gates de costura desta crate **clicam de verdade** — e cada um arma **um**
//! pincel. Com o `Crease` na mão a fileira do tecido nem é desenhada; com o
//! tecido na mão a da pose nem é desenhada. *Uma fixtura que não contém o
//! fenómeno não afirma nada sobre ele*, e a cura por «escrever mais um gate de
//! costura» depende de alguém se lembrar — que é precisamente o que falhou sete
//! vezes.
//!
//! # A régua
//!
//! Este censo não arma pincel nenhum: ele compara **as duas listas**, extraídas
//! do código dos dois ficheiros. Toda fileira que o `event` sabe DESPACHAR tem
//! de ser registada pelo `populate`, e toda a que o `populate` regista tem de
//! ter braço — *as duas metades, porque as curas são OPOSTAS* (§5.0: um morto
//! liga-se, um órfão apaga-se).
//!
//! ⚠️ **`include_str!` e não `read_to_string`:** o gémeo em runtime só falha
//! quando o teste corre, e um filtro deixa-o mudo para sempre; assim, se um dos
//! dois mudar de sítio isto **deixa de compilar**.
//!
//! ⚠️ **As linhas de COMENTÁRIO são peneiradas**, e não é cerimónia: os dois
//! ficheiros citam nomes de `SCULPT3D_*` na prosa — este cabeçalho inclusive.
//! *Um censo textual que não separa prosa de código mente nos DOIS sentidos.*

const EVENT: &str = include_str!("event.rs");
const POPULATE: &str = include_str!("populate.rs");

/// As linhas de CÓDIGO de um ficheiro — a prosa fora.
///
/// ⚠️ Ela peneira `//` e `//!`, que é o que os dois ficheiros usam; um bloco
/// `/* … */` passaria, e **nenhum dos dois os usa** (há gate abaixo a exigir que
/// a extracção encontre a população inteira, que é o que acusaria a mudança).
fn codigo(fonte: &str) -> impl Iterator<Item = &str> {
    fonte
        .lines()
        .map(str::trim)
        .filter(|l| !l.starts_with("//"))
}

/// Os grupos que o **despacho** conhece: `index_of(&crate::ids::NOME`.
fn despachados() -> std::collections::BTreeSet<&'static str> {
    codigo(EVENT)
        .filter_map(|l| l.split_once("index_of(&crate::ids::"))
        .filter_map(|(_, r)| {
            r.split(|c: char| !(c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_'))
                .next()
        })
        .filter(|n| !n.is_empty())
        .collect()
}

/// Os grupos que o **registo** percorre: `&crate::ids::NOME[..]`.
fn registados() -> std::collections::BTreeSet<&'static str> {
    codigo(POPULATE)
        .filter_map(|l| l.split_once("&crate::ids::"))
        .filter_map(|(_, r)| r.split_once("[..]").map(|(n, _)| n))
        .filter(|n| !n.is_empty())
        .collect()
}

/// ⛔⛔⛔ **GATE — toda fileira DESPACHADA é REGISTADA**, que é a metade do
/// controlo morto sob o dedo.
#[test]
fn toda_fileira_com_braco_no_event_e_registada_pelo_populate() {
    let mortos: Vec<&str> = despachados().difference(&registados()).copied().collect();
    assert!(
        mortos.is_empty(),
        "estas fileiras tem braco no `event.rs` e NAO sao registadas pelo \
         `populate.rs`: {mortos:?} — elas sao pintadas, hit-indexadas e MORTAS \
         sob o ponteiro, e o artista le' isso como «a ferramenta nao funciona»"
    );
}

/// ⛔⛔ **GATE — toda fileira REGISTADA tem BRAÇO**, que é a metade do id órfão,
/// e a cura dela é a OPOSTA (apagar, nunca ligar).
///
/// ⚠️ *A sonda vê os dois iguais*, e tratar um órfão como morto leva alguém a
/// construir um consumidor para um widget que não existe (§5.0).
#[test]
fn toda_fileira_registada_tem_braco_no_event() {
    let orfaos: Vec<&str> = registados().difference(&despachados()).copied().collect();
    assert!(
        orfaos.is_empty(),
        "estas fileiras sao registadas e NINGUEM as despacha: {orfaos:?} — um id \
         registado sem braco e' lixo, e a cura e' APAGAR e nao ligar"
    );
}

/// ⭐ **O PISO DE POPULAÇÃO** — sem ele os dois gates acima ficam verdes sobre o
/// VÁCUO no dia em que a extracção deixar de casar (um `for` reescrito, um
/// bloco `/* */`, um caminho de `use` diferente).
///
/// ⚠️ **É a forma que o `CLAUDE.md` §5.0 nomeia**: *um censo que varre zero e
/// fica verde*. Aqui ele varreria zero nos dois lados e a diferença de dois
/// conjuntos vazios é vazia — **trivialmente** verde.
#[test]
fn o_censo_das_fileiras_varre_a_populacao_toda() {
    let (d, r) = (despachados(), registados());
    assert!(
        d.len() >= 20 && r.len() >= 20,
        "a extracção achou {} fileiras despachadas e {} registadas — este painel \
         tem mais de vinte, logo uma das duas deixou de casar e os gates irmãos \
         passaram a medir o vácuo",
        d.len(),
        r.len()
    );
    // ⚠️ **E o controlo POSITIVO da própria extracção:** três nomes que têm de
    // estar lá, um por família — o verbo (que existe desde o princípio), a
    // deformação da pose (o report do dono) e o modo do contorno (o irmão que
    // caiu no mesmo buraco no mesmo dia). *Uma contagem sozinha não prova que a
    // extracção lê o que promete ler.*
    for nome in [
        "SCULPT3D_VERB",
        "SCULPT3D_POSE_MODE",
        "SCULPT3D_BOUNDARY_MODE",
    ] {
        assert!(
            d.contains(nome),
            "a extracção do despacho não achou `{nome}` — ela deixou de ler o \
             `event.rs` como ele está escrito"
        );
        assert!(
            r.contains(nome),
            "a extracção do registo não achou `{nome}` — ela deixou de ler o \
             `populate.rs` como ele está escrito"
        );
    }
}

const TOGGLES_FONTE: &str = include_str!("event_toggles.rs");

/// ⛔⛔⛔ **GATE — NENHUM INTERRUPTOR DA TABELA É NOMEADO À MÃO NO `populate`.**
///
/// A tabela [`crate::event::toggles::TOGGLES`] já sabe **quais** existem, e o
/// registo percorre-a. Uma segunda lista escrita ao lado dela é a segunda
/// resposta à mesma pergunta — e quem envelhece é a que o artista toca.
///
/// ⚠️⚠️ **Isto não é higiene: era um defeito vivo.** O `Pin far end` e o `Scale
/// without rotating` da pose estavam na tabela e **fora** da lista à mão, logo
/// pintados, hit-indexados e **mortos sob o dedo** — e com eles morria metade da
/// espec §5.1, que é o que separa uma rotação em torno do pivô de um arrasto
/// rígido. Quem os apanhou foi o
/// `every_pose_control_is_clickable_where_it_is_drawn`, num clique real.
///
/// ⚠️ **O doc do módulo dizia *«os SEIS interruptores»* e eram DEZASSETE** — uma
/// contagem escrita à mão ao lado de uma tabela envelhece na primeira adição, e
/// esta envelheceu onze vezes.
#[test]
fn nenhum_interruptor_da_tabela_e_nomeado_a_mao_no_populate() {
    let da_tabela: std::collections::BTreeSet<&str> = codigo(TOGGLES_FONTE)
        .filter_map(|l| l.split_once("crate::ids::"))
        .filter_map(|(_, r)| {
            r.split(|c: char| !(c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_'))
                .next()
        })
        .filter(|n| !n.is_empty())
        .collect();
    assert!(
        da_tabela.len() >= 15,
        "a extracção achou só {} interruptores na tabela — ela deixou de ler o \
         `event_toggles.rs` como ele está escrito, e o gate passaria a medir o \
         vácuo",
        da_tabela.len()
    );
    let a_mao: Vec<&str> = codigo(POPULATE)
        .filter(|l| !l.contains("[..]"))
        .filter_map(|l| l.split_once("crate::ids::"))
        .filter_map(|(_, r)| {
            r.split(|c: char| !(c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_'))
                .next()
        })
        .filter(|n| da_tabela.contains(n))
        .collect();
    assert!(
        a_mao.is_empty(),
        "o `populate` nomeia à mão interruptores que a tabela já declara: \
         {a_mao:?} — ele percorre a `TOGGLES`, e uma segunda lista ao lado dela \
         diverge na primeira adição (foi assim que o `Pin far end` da pose \
         nasceu morto sob o dedo)"
    );
}
