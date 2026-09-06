//! ⭐⭐⭐ **UMA ESCRITA DEIXA A PEÇA NUM ESTADO QUE O DOCUMENTO ACEITA** — a porta do produto.
//!
//! # ⛔⛔⛔ O report que a obrigou (Enio, 2026-09-06)
//!
//! *«se reduzir muito o raio de Rounded Cylinder, todas as formas na tela somem»*.
//!
//! O mecanismo não é da forma que ele apontou: o `FieldDoc` é validado **como um todo** e cozido da
//! hierarquia a cada quadro, então **uma** primitiva que o documento recusa não some sozinha — ela
//! faz a derivação inteira falhar, e o artista fica com o ecrã vazio, sem mensagem, sobre peças que
//! não têm defeito nenhum.
//!
//! ⚠️ **E o censo diz que ele apontou o único membro da família que ele calhou de tocar:** varrendo
//! `PrimitiveKind::ALL` × cada linha do painel × uma escada de valores, **31 pares (forma, linha)**
//! em ~20 formas aceitavam uma escrita que apagava a cena. *O exemplo que o dono aponta pode ser a
//! excepção da família — aqui foi a regra.*
//!
//! # As DUAS espécies, e por que a cura é uma só
//!
//! | espécie | o que se passa | a cura |
//! |---|---|---|
//! | **A** — a faixa **sabe** a parede (`Span::Wall`, `WallFromZero`, `Walls`) e ninguém a repõe depois de mexer na irmã | o bojo do cilindro, a espessura da moldura, o filete de tudo | [`clamp_dims`], que **relê a tabela** e coage cada linha para dentro da própria faixa |
//! | **B** — a faixa **não** sabe: ela diz `Positive` e o documento tem uma regra entre dimensões | o `RoundCone`, cuja altura tem de passar de `\|Δraio\|/2` | declarar a faixa honesta ([`Span::Floor`]) — e aí a espécie A cura-a |
//!
//! ⭐ **A cura é DERIVADA da tabela, e não uma lista escrita à mão:** a [`Span`] de cada linha já é
//! calculada a partir dos valores vivos da peça, então repô-la é reler a tabela. Uma forma nova
//! ganha a lei sem uma linha aqui — que é a mesma razão de o `round_index` perguntar à [`dims`].
//!
//! ⚠️ **Isto NÃO substitui o [`super::clamp_round`]**: ele continua a ser a porta de quem mexe na
//! *pose* (escalar uma peça também encolhe o filete), e essa não passa por aqui.

use super::dims_write::{keep_below, write_dim};
use super::{Span, dims};
use crate::{FieldError, Primitive};

/// ⭐ **Escreve uma dimensão e repõe as invariantes da peça** — a porta do produto.
///
/// Ver [`super::dims_write::write_dim`] para o que a escrita crua coage e recusa, e [`clamp_dims`]
/// para o que a reposição faz.
///
/// # Errors
/// O que a escrita crua devolver. ⚠️ **Uma escrita recusada não mexe na peça** — a reposição só
/// corre depois de um `Ok`, senão ela «curaria» um estado que o gesto nem chegou a criar.
pub fn set_dim(p: &mut Primitive, node: u32, index: usize, value: f32) -> Result<(), FieldError> {
    write_dim(p, node, index, value)?;
    clamp_dims(p);
    Ok(())
}

/// Quantas vezes a tabela é relida. ⚠️ **Não é um número de conforto:** coagir uma linha muda a
/// parede das outras (o bojo depende do raio *e* da altura), então uma passagem só deixaria a
/// segunda dependência por resolver. Duas bastam para tudo o que existe hoje; a terceira é a
/// margem, e o gate `no_dim_write_can_produce_a_piece_the_document_refuses` é quem responde se
/// chega.
const PASSAGENS: usize = 3;

/// ⭐⭐ **Coage cada linha para dentro da própria faixa declarada** — devolve `true` se mexeu.
///
/// ⚠️ **A coerção é sempre para DENTRO**, nunca para fora: uma linha que já cabe não é tocada, e é
/// isso que a torna idempotente e segura de correr depois de toda escrita.
pub fn clamp_dims(p: &mut Primitive) -> bool {
    let mut mexeu = false;
    for _ in 0..PASSAGENS {
        let mut nesta = false;
        for (i, d) in dims(p).iter().enumerate() {
            let Some(alvo) = coagido(d.value, d.span) else {
                continue;
            };
            // ⚠️ **A escrita CRUA, e não a porta** — a porta chama isto, e chamá-la de volta seria
            // recursão sem fundo.
            if write_dim(p, 0, i, alvo).is_ok() {
                nesta = true;
            }
        }
        mexeu |= nesta;
        if !nesta {
            break;
        }
    }
    mexeu
}

/// Para onde esta linha tem de ir, ou `None` se ela já cabe.
///
/// ⚠️ **As faixas sem parede não são tocadas**, e a lista é exaustiva de propósito: uma [`Span`]
/// nova é **erro de compilação** aqui até alguém dizer se ela tem fronteira a repor.
fn coagido(value: f32, span: Span) -> Option<f32> {
    match span {
        // ⚠️ A `Wall` recusa o zero e a `WallFromZero` aceita-o — mas **as duas** recusam passar da
        // parede, e é só isso que se repõe. O piso delas é assunto da escrita crua.
        Span::Wall(w) | Span::WallFromZero(w) => (value >= w).then(|| keep_below(value, w)),
        // ⭐ Simétrica: o que estoura é o módulo, e o sinal fica.
        // ⭐ Simétrica: o que estoura é o módulo, e o sinal fica. ⚠️ **Estritamente dentro** — a
        // validação do corte recusa `|corte| ≥ raio`, e parar em cima da parede seria parar onde a
        // peça deixa de existir.
        Span::Walls(w) => (value.abs() >= w).then(|| value.signum() * keep_below(value.abs(), w)),
        // ⭐⭐ **O PISO** — a faixa da espécie B. Ver o cabeçalho.
        Span::Floor(f) => (value <= f).then(|| super::dims_write::keep_above(value, f)),
        Span::Count { min, max } => {
            let alvo = value.round().clamp(min as f32, max as f32);
            (alvo != value).then_some(alvo)
        }
        // ⚠️ **A `Turn` não tem fronteira que se reponha** — um ângulo além da meia-volta não é
        // recusado, é renomeado (ver o doc dela). E as outras não têm parede nenhuma.
        Span::Positive
        | Span::Free
        | Span::Along
        | Span::Choice(_)
        | Span::Turn(_)
        | Span::Locked
        | Span::FromZero => None,
    }
}
