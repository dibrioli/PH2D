//! **A PORTA DAS FITAS** — o vocabulário do modo `Branches` e as duas funções que a shell
//! chama (HR-18, irmão de [`super`] pelo tecto de LOC; o corte é por responsabilidade: lá fica
//! *o que o nó é*, aqui *o contrato pelo qual a geometria dele nasce noutro sítio*).
//!
//! Report do Enio (2026-08-30): *"não quero eliminar o modo atual, quero uma opção nova. Comece
//! e coloque como a opção padrão."*

use crate::params::Params;
use crate::{MANIFEST, build};

/// **O modo SEGMENTOS** — um elemento por osso, que é o que este nó sempre emitiu.
///
/// ⚠️ **Continua a existir por decisão do Enio** (*"não quero eliminar o modo atual, quero uma
/// opção nova"*, 2026-08-30), e não é só compatibilidade: é o modo que publica o esqueleto que
/// os cinco `rig.*` consomem, com o contrato de colunas que um gate mede ao bit.
pub const GEOMETRY_SEGMENTS: i32 = 0;
/// **O modo RAMOS** — uma fita contínua por ramo, o que as quatro referências fazem.
pub const GEOMETRY_BRANCHES: i32 = 1;

/// Os dois modos de geometria. ⚠️ A ordem É o valor gravado — `Segments` fica em `0` para
/// sempre, mesmo sendo o `Branches` o **default** (ver o `ParamSpec` de [`param::GEOMETRY`]).
pub const GEOMETRY_LABELS: &[&str] = &["Segments", "Branches"];

/// **A chave do canal externo** por onde a shell entrega as fitas já construídas — o nome que o
/// `eval` lê e sob o qual a shell publica.
///
/// ⚠️ **Uma porta, dois lados.** É a mesma lei do `source.shape` (ADR-0154): o nó descreve, a
/// shell constrói, e **quem nomeia a chave é uma função só** — dois nomes divergiriam no dia em
/// que alguém mudasse um, e a planta desapareceria sem erro nenhum.
///
/// ⚠️ **Endereçada pelo CONTEÚDO, e não pelo id do nó** (a decisão do `shape_key`): duas plantas
/// com exactamente os mesmos números e a mesma gramática partilham as fitas em vez de as
/// construir duas vezes — e, mais importante, um nó que não mudou não republica.
///
/// ⚠️ **A lista de params sai do [`MANIFEST`]**, nunca de uma segunda lista escrita à mão: um
/// param novo entra na chave sozinho, e uma chave que ignorasse um param faria a shell servir a
/// geometria ANTIGA depois de o artista mexer nele.
///
/// ⛔⛔ **E a lista de TEXTO tem de vir junto — foi o achado §3.3 da auditoria de seis lentes.**
/// A 1.ª redacção iterava `MANIFEST.params` (f32) e acrescentava o axioma e as regras **à mão**,
/// deixando de fora os **três nomes de objecto de folha**: duas plantas com os mesmos números e
/// a mesma gramática, uma com *Leaf (J) = folha* e outra *= flor*, partilhavam a chave e a
/// segunda **sobrescrevia** a primeira. ⚠️ *A invariante que este doc declara — «um param novo
/// entra na chave sozinho» — era verdadeira para o `f32` e falsa para o texto*, e o manifesto é
/// `f32`-only por contrato congelado (§6). ⇒ os dois lados saem agora de listas: `MANIFEST.params`
/// e [`crate::TEXT_PARAMS`].
#[must_use]
pub fn ribbon_key(get: impl Fn(&str) -> f32, text: impl Fn(&str) -> String) -> String {
    // ⛔⛔⛔ **O `$` NÃO É DECORAÇÃO — ele é a cerca que mantém isto FORA do selector.**
    //
    // Auditoria de seis lentes, doc 96 §5.5. Esta chave é publicada na MESMA tabela de externos
    // de que o picker de objectos tira as opções (`source_options`), e o filtro dele é
    // `!is_reserved(k)`. Sem o prefixo, cada planta/forma/texto/tabela derivada aparecia ao
    // artista como uma *"Drawn shape"* escolhível — na cena `=108` são **cinco chips de lixo**,
    // com a gramática crua lá dentro, e clicar num planta a PRÓPRIA planta como folha dela.
    //
    // ⚠️ O doc do `RESERVED_PREFIX` já dizia as duas metades: *«o editor publica DENTRO do
    // namespace, e recusa publicar um nome do artista que já esteja nele»*. A primeira metade é
    // que não estava a ser cumprida por quem cunha chaves de CONTEÚDO.
    //
    // ⚠️ **Mudar o prefixo é seguro porque a chave é opaca:** quem a cunha e quem a lê chamam
    // esta mesma função, e ela não é persistida em lado nenhum (é derivada a cada quadro).
    let mut k = String::from("$lsysrib");
    for spec in MANIFEST.params {
        k.push(':');
        // Os BITS, não o decimal: `to_string` de um `f32` arredonda, e duas larguras que se
        // imprimem iguais dariam a mesma chave para geometrias diferentes.
        k.push_str(&get(spec.name).to_bits().to_string());
    }
    // ⚠️ O texto entra CRU — a gramática é o que decide a forma, e um resumo dela (um hash
    // curto) trocaria uma colisão improvável por uma planta errada sem sintoma.
    for name in crate::TEXT_PARAMS {
        k.push('\u{1}');
        k.push_str(&text(name));
    }
    k
}

/// **O ESQUELETO, por GETTER** — a porta pela qual a shell constrói as fitas com exactamente os
/// números que o nó ia cozinhar.
///
/// ⚠️ **A shell deriva UMA vez e o nó não deriva nenhuma** (em `Branches` ele só clona o
/// externo), então isto não é trabalho a dobrar: é o mesmo trabalho, do outro lado da cerca que
/// impede um nó de alcançar a biblioteca vetorial.
///
/// ⚠️ **O `get` tem de resolver a escada inteira** (conduzido → override → default): um param
/// conduzido por fio só tem valor DURANTE o cozimento, e uma chave cunhada do valor estático não
/// encontra a que o nó procura — a planta desaparece em silêncio. É o modo de falha mais caro
/// desta casa, e já mordeu o `source.shape` (ver `motion_externals::driven_params`).
#[must_use]
pub fn skeleton(
    axiom: &str,
    rules: &str,
    get: impl Fn(&str) -> f32,
) -> ph2d_nodegraph::attr::Stream {
    build(axiom, rules, &Params::read_with(get))
}
