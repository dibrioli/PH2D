//! ⭐⭐⭐ **UMA CAIXA DE TEXTO DE UM FORMULÁRIO TEM NOME — e ele fica quando ela é usada.**
//!
//! Report do dono, 2026-09-22, com foto da secção `FACTORY` (cinco caixas seguidas e cinco setas
//! vermelhas): *«campos de texto difíceis de saber para que servem. Como resolver isso?»*
//!
//! # ⛔⛔ A causa, medida
//!
//! | | antes | |
//! |---|---:|---|
//! | caixas de texto do app com rótulo VAZIO | **`53`** | o sentido vivia só no espaço reservado |
//! | com rótulo | `3` | |
//!
//! E o espaço reservado é pintado **apenas enquanto a caixa está vazia**
//! (`text_input/mod.rs`: `if displayed.is_empty() && !input.placeholder.is_empty()`) ⇒ *um campo
//! de texto deste app dizia para que servia exactamente até alguém o usar.*
//!
//! ⛔⛔⛔ **E o diagnóstico já estava ESCRITO no repo, com a cura aplicada a UMA secção.** O
//! `sections/hud.rs` trazia, desde a wave dele: *«o nome vai POR CIMA, e não no `TextInput`** — a
//! foto apanhou três campos seguidos sem um único nome à vista. O `text_row` pinta o controlo na
//! largura toda e **não desenha o rótulo**; e pô-lo no `placeholder` seria pior do que nada,
//! porque um placeholder desaparece exactamente quando o campo tem valor»*. ⇒ *a porta ficou como
//! estava e as outras trinta linhas continuaram mudas* — e o remendo local (o nome POR CIMA)
//! contrariava o próprio dono (*«Label acima do campo numérico! Muito ruim!»*, 2026-09-14).
//!
//! # ⭐ A régua
//!
//! **Uma caixa de texto COMEÇA onde as caixas de número do mesmo painel começam.** Ela
//! auto-calibra-se: não há aqui nenhum número escolhido — a coluna do controlo é a que o painel
//! já usa para todas as outras linhas, e uma caixa à largura inteira começa à esquerda dela.
//!
//! ⛔⛔ **O QUE ESTA RÉGUA NÃO VÊ, e está medido:** ela prova que a caixa **tem coluna de nome**,
//! não que a coluna seja a da SECÇÃO. A mutação que troca a `Seccao` de uma secção pela de omissão
//! (`apenas_campos(1)`) **sobrevive** — o nome continua a ser pintado, só que noutro `x`. Essa é
//! outra propriedade e tem outra régua: o `diag_que_linhas_o_nome_espreme`, que mede em que `x` a
//! coluna de cada linha começa. *Uma régua afirma uma frase; duas frases pedem duas réguas.*
//!
//! ⚠️ **A população é «painéis que TÊM formulário»** — um painel só com caixas de rename
//! (`hierarchy`, `asset_browser`, `audio_editor`) não tem coluna de número contra que medir, e a
//! caixa dele é nomeada pelo sítio onde vive. *Medir ali seria inventar uma barra.*

use super::a_marca_tem_a_altura_da_linha::{
    FONTES_DOS_IDS, PISO_DOS_SLUGS, caixas_de_texto_do_inspector, censo_das_colunas,
};

/// ⭐⭐ **O que NÃO é uma linha de propriedade — com o nome e o porquê.**
///
/// ⛔ Uma lista destas sem censo de obsolescência é uma LICENÇA: a 2.ª metade do gate exige que
/// cada entrada continue a descrever uma caixa que existe e que continua à largura inteira.
const FORA_COM_NOME: &[(ph2d_editor_core::NodeId, &str)] = &[
    // O nome do OBJECTO, no topo do Inspector. Ele não é uma propriedade entre outras: é o título
    // do que está seleccionado, e o sítio onde vive di-lo. ⚠️ É a mesma razão pela qual um campo
    // de rename dentro de uma janela própria não leva nome.
    (
        ph2d_editor_core::ids::INSP_ENTITY_NAME,
        "o nome do objecto é o título do painel, não uma propriedade",
    ),
];

/// ⛔ Pisos de população, MEDIDOS em 2026-09-22: `50` caixas de texto no Inspector armado.
const PISO_DE_CAIXAS: usize = 45;

#[test]
fn uma_caixa_de_texto_comeca_onde_uma_caixa_de_numero_comeca() {
    let colunas = censo_das_colunas();
    let com_formulario: Vec<_> = colunas.iter().filter(|(_, _, n)| !n.is_empty()).collect();
    assert!(
        !com_formulario.is_empty(),
        "nenhum painel com caixas de texto E de número — a régua ficou sem sujeito, e um censo \
         sem população lê-se como aprovação"
    );

    let caixas = caixas_de_texto_do_inspector();
    assert!(
        caixas.len() >= PISO_DE_CAIXAS,
        "o censo viu {} caixas de texto no Inspector (piso {PISO_DE_CAIXAS}) — uma varredura que \
         lê pouco devolve ZERO acusações.\n⚠️ SE CORREU COM `-p`: corra `--workspace`.",
        caixas.len()
    );

    // ⭐ A coluna do controlo sai do PRODUTO: é onde as caixas de número deste painel começam.
    let (_, _, numeros) = com_formulario
        .iter()
        .find(|(p, _, _)| *p == "inspector")
        .expect("o inspector é o painel de formulário desta casa");
    let coluna = *numeros
        .iter()
        .min()
        .expect("um painel de formulário tem pelo menos uma caixa de número");

    let nomes = super::o_que_o_artista_nao_alcanca::nomes_de(FONTES_DOS_IDS, PISO_DOS_SLUGS);
    let mudas: Vec<String> = caixas
        .iter()
        .filter(|(id, x)| (*x as i32) < coluna && !FORA_COM_NOME.iter().any(|(fid, _)| fid == id))
        .map(|(id, x)| {
            format!(
                "{} começa em x={x:.0}, à ESQUERDA da coluna do controlo (x={coluna})",
                nomes
                    .get(id)
                    .map_or_else(|| format!("{id:?}"), Clone::clone)
            )
        })
        .collect();
    assert!(
        mudas.is_empty(),
        "estas caixas de texto pintam-se à LARGURA INTEIRA, logo não têm nome:\n  {}\n\n\
         ⚠️ O sentido delas fica só no espaço reservado, e ele desaparece assim que a caixa tem \
         valor — que é exactamente quando o artista precisa de saber o que ela é.\n\
         ⇒ a cura é a porta `ph2d_editor_core::property_row::paint_text_row`, que EXIGE um nome e \
         uma `Seccao` — ⛔ nunca uma cópia do `paint_text_input_with_buffer` no sítio da pintura.",
        mudas.join("\n  ")
    );

    // ── 2.ª metade: a isenção ainda descreve alguma coisa ─────────────────────
    let obsoletas: Vec<String> = FORA_COM_NOME
        .iter()
        .filter(|(fid, _)| {
            !caixas
                .iter()
                .any(|(id, x)| id == fid && (*x as i32) < coluna)
        })
        .map(|(fid, porque)| {
            format!(
                "{} — {porque}",
                nomes
                    .get(fid)
                    .map_or_else(|| format!("{fid:?}"), Clone::clone)
            )
        })
        .collect();
    assert!(
        obsoletas.is_empty(),
        "estas isenções já não descrevem nada — ou a caixa saiu, ou passou a ter nome, e uma \
         isenção que não descreve nada é uma licença:\n  {}",
        obsoletas.join("\n  ")
    );
}
