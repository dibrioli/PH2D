//! **A tabela de COR autorada viaja no arquivo** (plano UI/UX W6, degrau 1) — irmão do
//! [`crate::project`] pelo teto de LOC (HR-18), e o corte é por assunto: aqui mora *como uma
//! re-vestida do design system sobrevive a um save*.
//!
//! # Ela fica FORA do `ProjectState`, e não é arrumação
//!
//! O `ProjectState` é a unidade do undo GLOBAL, e um Ctrl+Z do canvas não deve rebobinar a cara do
//! editor — o mesmo motivo que mantém `physics`, `motion` e `timeline` fora dele. O preço honesto
//! é que **editar um token não entra na fila do Ctrl+Z**; quem desfaz é o *Reset* da linha, que é
//! o que o painel de física também faz.
//!
//! # A chave, nunca o índice
//!
//! O que viaja é a **chave do token no `tokens.json`** (`"accent"`, `"bg-0"`). Guardar o índice do
//! variant amarraria todo projeto salvo à ORDEM da lista, e acrescentar um token no meio da tabela
//! re-pintaria o app com as cores trocadas — a mesma lei que a W4a aplicou ao binding.

use ph2d_tokens::color::Color;
use ph2d_tokens::num_overrides::{NumOverride, NumValue, num_overrides, set_num_overrides};
use ph2d_tokens::overrides::{ColorOverride, TokenValue, color_overrides, set_color_overrides};
use ph2d_tokens::{ColorToken, NumToken, Theme};

/// **O que um token autorado vale, no arquivo** — as duas espécies do [`TokenValue`].
///
/// ⚠️ Um ENUM, e não um `rgba` com um `alias: Option<String>` ao lado: os dois campos seriam
/// mutuamente exclusivos e nada no formato o diria, então um arquivo poderia trazer os dois e o
/// leitor teria de escolher um vencedor que ninguém especificou. A representação apaga o caso.
#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) enum SavedValue {
    /// Uma cor literal.
    Literal([u8; 4]),
    /// **A CHAVE do token seguido** — nunca o índice, a mesma lei do campo `key` abaixo.
    Alias(String),
    /// Um comprimento em px (plano UI/UX W4c.1).
    ///
    /// ⚠️ **Apendado**, e postcard indexa variants por posição: `Literal`(0) e `Alias`(1) não se
    /// movem, então **todo arquivo já salvo continua a ler**. O `PROJECT_SCHEMA` sobe pelo caminho
    /// INVERSO — um build antigo a ler um arquivo novo bateria num índice de variant que ele não
    /// tem —, que é o mesmo raciocínio do `JointKind::Weld` e do `Cap::Square`.
    Number(f32),
    /// Uma FÓRMULA, como TEXTO (plano UI/UX W4c.3) — `{spacing.md} * 2`.
    ///
    /// ⚠️ Texto, e não uma árvore parseada: é o texto que o artista reabre e edita, e serializar o
    /// IR faria o formato do arquivo depender da forma de um tipo do parser — a mesma decisão que
    /// a `motion.expression` tomou, e a razão de o `NumValue::Expr` também guardar texto.
    ///
    /// ⚠️ Apendada DEPOIS do `Number` pelo mesmo raciocínio posicional dele: `Literal`(0),
    /// `Alias`(1) e `Number`(2) não se movem, então todo arquivo já salvo continua a ler.
    Formula(String),
}

/// Um token de cor autorado: **que modo, que token, valendo o quê**.
///
/// ⚠️ Tipo PRÓPRIO do arquivo, e não o [`ColorOverride`] da `ph2d-tokens` — a crate de tokens não
/// depende de `serde` para o valor de runtime dela, e fazer o formato do arquivo herdar o layout
/// de um tipo de runtime é o que torna um refactor interno numa quebra de save.
#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct SavedToken {
    /// O modo (`Theme as u8`) — o discriminante é estável porque o enum é append-only.
    theme: u8,
    /// A chave do token no `tokens.json` (`"accent"`, `"bg-0"`, …).
    key: String,
    value: SavedValue,
}

/// O modo a partir do byte guardado. **Porta única** da direção inversa do `theme as u8`.
///
/// ⚠️ Um byte que o enum não tem cai no default (`Forge`) em vez de recusar o arquivo inteiro: um
/// modo desconhecido é um override que não se sabe mostrar, e jogar fora o projeto por causa dele
/// seria a resposta errada — o `PROJECT_SCHEMA` é quem recusa formato, não um campo.
const fn theme_from_u8(b: u8) -> Theme {
    match b {
        1 => Theme::Workshop,
        2 => Theme::Sunstone,
        3 => Theme::Blueprint,
        _ => Theme::Forge,
    }
}

/// O que o save grava.
///
/// ⚠️ Sai da PORTA (`color_overrides()`), que já devolve a lista em ordem canônica: dois projetos
/// logicamente iguais dão os mesmos bytes, seja qual for a ordem dos cliques.
pub(crate) fn collect() -> Vec<SavedToken> {
    // ⚠️ **UMA lista para as duas famílias**, e não um campo novo ao lado. O que o arquivo guarda é
    // *"que tokens o artista autorou"*, e a chave já é a identidade de um token — `"accent"` para
    // uma cor, `"spacing.md"` para um comprimento. Duas listas seriam duas respostas à mesma
    // pergunta, e o import/export DTCG (W4c.5) teria de as juntar de novo.
    //
    // ⚠️ Isto só é seguro porque as duas famílias são **provavelmente disjuntas** nas chaves — há
    // gate a afirmá-lo (`no_key_is_claimed_by_both_families`), porque sem ele o load teria de
    // escolher um dono e a escolha seria silenciosa.
    color_overrides()
        .into_iter()
        .map(|e| SavedToken {
            theme: e.theme as u8,
            key: e.token.key().to_string(),
            value: match e.value {
                TokenValue::Literal(c) => SavedValue::Literal([c.r, c.g, c.b, c.a]),
                TokenValue::Alias(t) => SavedValue::Alias(t.key().to_string()),
            },
        })
        .chain(num_overrides().into_iter().map(|e| SavedToken {
            theme: e.theme as u8,
            key: e.token.key().to_string(),
            value: match e.value {
                NumValue::Literal(px) => SavedValue::Number(px),
                NumValue::Alias(t) => SavedValue::Alias(t.key().to_string()),
                NumValue::Expr(src) => SavedValue::Formula(src),
            },
        }))
        .collect()
}

/// **A tabela do documento anterior morre aqui, e a do arquivo entra.**
///
/// ⚠️ Instalar a lista INTEIRA (em vez de acrescentar) é o que faz o load ESQUECER: sem isto,
/// abrir um projeto de fábrica depois de um re-vestido deixaria o app com as cores do documento
/// ANTERIOR, e nada na tela diria porquê — a mesma classe da timeline.
///
/// Um token que o `tokens.json` já não tem é **DESCARTADO** (o `from_key` devolve `None`): a
/// tabela de fábrica é a autoridade sobre quais tokens existem. E o que cai é **DITO** — uma
/// tabela que encolhe em silêncio lê-se como *"eu nunca autorei isto"*, e o artista procuraria a
/// cor onde ela não está.
///
/// Devolve **quantas entradas do arquivo não puderam entrar** — o mesmo número que a mensagem diz.
/// ⚠️ Duas representações do MESMO fato, para dois públicos: a linha de log é para a pessoa que
/// abriu o projeto, o retorno é para o gate. Sem ele, um descarte que deixasse de ser contado
/// passaria com a suíte verde, que é precisamente o *encolher em silêncio* que este doc proíbe.
/// ⚠️ **A CHAVE decide de que família é a entrada**, e é a única coisa que decide. Uma entrada cuja
/// chave é de cor mas cujo valor é um número (ou o contrário) é um par que nenhuma porta emite —
/// só um arquivo editado à mão o produz — e ela **cai**, em vez de ser dobrada para o que der: uma
/// cor que aparecesse por um `spacing` mal-emparelhado seria a re-vestida a inventar-se sozinha.
pub(crate) fn install(saved: &[SavedToken]) -> usize {
    let mut dropped = 0usize;
    let mut colours: Vec<ColorOverride> = Vec::new();
    let mut nums: Vec<NumOverride> = Vec::new();

    for t in saved {
        let theme = theme_from_u8(t.theme);
        if let Some(token) = ColorToken::from_key(&t.key) {
            // ⚠️ Um alias cujo ALVO já não existe cai pelo mesmo motivo que o token desconhecido:
            // a tabela de fábrica é a autoridade sobre quais tokens existem, e um elo pendurado no
            // vazio não tem valor a devolver.
            match &t.value {
                SavedValue::Literal([r, g, b, a]) => colours.push(ColorOverride {
                    theme,
                    token,
                    value: TokenValue::Literal(Color {
                        r: *r,
                        g: *g,
                        b: *b,
                        a: *a,
                    }),
                }),
                SavedValue::Alias(key) => match ColorToken::from_key(key) {
                    Some(target) => colours.push(ColorOverride {
                        theme,
                        token,
                        value: TokenValue::Alias(target),
                    }),
                    None => dropped += 1,
                },
                SavedValue::Number(_) | SavedValue::Formula(_) => dropped += 1,
            }
        } else if let Some(token) = NumToken::from_key(&t.key) {
            match &t.value {
                SavedValue::Number(px) => nums.push(NumOverride {
                    theme,
                    token,
                    value: NumValue::Literal(*px),
                }),
                SavedValue::Alias(key) => match NumToken::from_key(key) {
                    Some(target) => nums.push(NumOverride {
                        theme,
                        token,
                        value: NumValue::Alias(target),
                    }),
                    None => dropped += 1,
                },
                // ⚠️ A fórmula entra CRUA e quem a admite é a porta (`set_num_overrides`), que
                // confere sintaxe, laço e comprimento — a MESMA admissão do gesto do artista. Um
                // 2º validador aqui seria a segunda resposta a *"esta fórmula serve?"*, e a que
                // diverge no dia em que a linguagem ganhar um operador.
                SavedValue::Formula(src) => nums.push(NumOverride {
                    theme,
                    token,
                    value: NumValue::Expr(src.clone()),
                }),
                SavedValue::Literal(_) => dropped += 1,
            }
        } else {
            dropped += 1;
        }
    }

    // ⚠️ As DUAS portas correm sempre, mesmo com a lista vazia: instalar é o que faz o load
    // ESQUECER, e pular a família que este arquivo não usa deixaria a escala do documento ANTERIOR
    // de pé sob as cores do novo.
    //
    // ⚠️ Cada porta descarta o que não pode entrar (um laço, um número que não é comprimento) e DIZ
    // quantos — um arquivo editado à mão pode trazer um, e recusar o projeto inteiro por causa dele
    // seria jogar fora a re-vestida.
    dropped += set_color_overrides(colours);
    dropped += set_num_overrides(nums);
    if dropped > 0 {
        eprintln!(
            "[proj] {dropped} token(s) do arquivo nao existem mais no design system (ou fechavam \
             um laco de alias, ou nao eram um valor daquela familia) e foram descartados"
        );
    }
    dropped
}

#[cfg(test)]
#[path = "project_tokens_tests.rs"]
mod tests;
