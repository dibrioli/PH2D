//! ⭐⭐⭐ **O PASSE AUTOMÁTICO** — doc 115 W5: *«o app separa sozinho»* (ordem do dono, 2026-09-17,
//! escolhida por ele entre as três leituras possíveis de tirar o `motion.collide` do catálogo).
//!
//! # O que ele é, e o que ele NÃO é
//!
//! Ele é um **ACABAMENTO sobre o que vai ser desenhado**: recebe a corrente cozida de um sink,
//! afasta as peças que se sobrepõem, e devolve-a. Não há nó, não há fio, não há realimentação.
//!
//! ⚠️ **A morada foi MEDIDA e não escolhida** (doc 115 §3, a W0): as duas leituras — o passe como
//! acabamento e o passe realimentado no `rope.state` — são **indistinguíveis acima de 16
//! varreduras**, e a `32` as duas entregam zero pares sobrepostos. A barata é a que encosta na
//! barra, e ela não precisa de fechar anel nenhum.
//!
//! # ⛔ Porque ele é FINO, e isso é a prova de que não há segunda cópia da lei
//!
//! As portas partilhadas desta crate **já são** a lei: [`crate::colisores`] lê o que cada linha
//! declara, [`crate::inv_inercias`] diz quanto cada peça roda, e [`crate::separate`] resolve os
//! contactos na grelha espacial. O `motion.collide` chama exactamente as mesmas três e acrescenta
//! por cima os knobs do CARTÃO dele (o `Radius` de recuo, o `Strength`, o `falloff`) — que um
//! passe automático **não tem**, porque não tem cartão.
//!
//! ⇒ *a diferença entre os dois não é a lei, são os knobs*, e é por isso que este ficheiro pode
//! nascer sem duplicar uma linha de aritmética — e sobreviver ao dia em que a W6 apagar aquele nó.
//!
//! # As três ausências, cada uma uma decisão
//!
//! - **Sem recuo para DISCO.** O nó, ao encontrar uma peça que não declara nada, dá-lhe o disco do
//!   `Radius` do cartão — *sem isso uma corrente MISTA deixaria metade das peças inertes e caladas*.
//!   Aqui não há cartão de onde tirar um raio, e **inventar um seria pior**: uma peça que não
//!   declara forma nenhuma é uma peça que ninguém disse que colide.
//! - **Sem `Strength`.** Ele é um param de CARTÃO, e um passe não tem cartão de onde o tirar.
//! - **Sem realimentação.** Ver a W0 acima.
//!
//! ⛔⛔⛔ **E havia uma TERCEIRA ausência aqui que era um ERRO DE CATEGORIA, corrigido na W6**
//! (doc 115 §15.1). Ela dizia: *«Sem `Strength` e sem `falloff`. Os dois são mistura no fim, e os
//! dois são AUTORADOS. Um passe sem cartão corre a lei inteira ou não corre.»*
//!
//! Os dois são mistura no fim — isso estava certo. **O resto não:** o `Strength` é um param do
//! CARTÃO do nó e o `falloff` é uma **COLUNA DA CORRENTE**, que ~50 nós do catálogo escrevem
//! (`motion.falloff`, a família `field.*`, …). *Um passe sem cartão não tem `Strength`; mas tem a
//! CORRENTE, logo tem o `falloff`.* Pô-los na mesma frase leu «autorado» como se fosse uma só
//! coisa, e custou à W5 a única capacidade que o caminho novo não tinha.
//!
//! ⇒ o passe honra o `falloff` **com a lei do nó, termo a termo** (`k = falloff`, e o `Strength`
//! do nó vale `1` aqui por não existir): `p + (p′ − p)·k`, `rot + giro·k`. ⭐ **Ausente lê-se `1`**,
//! logo toda cena sem aquela coluna fica **byte-idêntica**.
//!
//! ⭐⭐ **E é isto que dá a um OBJECTO a forma de NÃO colidir** (o §10.4 do doc 115, aberto desde a
//! W5): a `source.shape` tem o `Collide` do cartão dela, e um Sprite não tem cartão nenhum — mas
//! tem a corrente, e qualquer campo a montante do sink põe-lhe `falloff = 0`. ⚠️ **O que isso é, ao
//! certo:** a peça **não é movida** e as vizinhas ficam com metade da correcção que pediam, logo
//! passam *através* dela — é o «MUTAR» do par 3 da cena `=48`, e ⛔ **não** é o mesmo que
//! `inv_mass = 0` (o «PINAR»: obstáculo que não se move e empurra as outras por inteiro).

use crate::{Pecas, Saida};
use ph2d_nodegraph::attr::{Column, Stream};

/// A coluna do peso inverso de cada peça (`0` = obstáculo, que não se move).
///
/// ⚠️ **O nome é uma dívida PRÉ-EXISTENTE e nomeada:** ele é escrito como literal em **49** sítios
/// por oito crates de nó, cada uma com a cópia privada dela (`INV_MASS_COL`, `INV_MASS`, ou o
/// literal cru), e uma delas exporta-o `pub`. Unificá-lo é uma varredura que atravessa linhas e não
/// é desta wave; o que esta const faz é não acrescentar a nona cópia **anónima**.
const INV_MASS_COLUMN: &str = "inv_mass";

/// A coluna da posição.
const P_COLUMN: &str = "P";

/// A coluna do ângulo, em graus — a mesma que o [`crate::colisores`] lê para orientar uma caixa.
const ROT_COLUMN: &str = "rot";

/// **A espinha dos MOPs:** quanto este passe age sobre cada elemento (`1` = por inteiro, `0` = nada).
///
/// ⚠️ Mesmo nome e mesma semântica que o `motion.collide` lê — *a coluna é do CATÁLOGO, não de um
/// nó*, e é precisamente por isso que um passe sem cartão a pode honrar.
const FALLOFF_COLUMN: &str = "falloff";

/// **A atenuação por elemento, ou `1` para quem não a declara.**
///
/// ⚠️ **Clampada a `[0, 1]`, como no nó** — e a razão é a mesma que ele escreve: *um documento
/// editado à mão não pode INVERTER um empurrão*. Um `NaN` cai no braço de omissão (`1`), porque
/// `clamp` com `NaN` é veneno silencioso e a peça deixaria de se mexer sem ninguém saber porquê.
fn atenuacao(s: &Stream, n: usize) -> Vec<f32> {
    match s.get(FALLOFF_COLUMN) {
        Some(Column::Scalar(v)) if v.len() == n => v
            .iter()
            .map(|f| {
                if f.is_finite() {
                    f.clamp(0.0, 1.0)
                } else {
                    1.0
                }
            })
            .collect(),
        _ => vec![1.0; n],
    }
}

/// Os pesos de uma corrente, ou `1` para quem não os declara.
fn pesos(s: &Stream, n: usize) -> Vec<f32> {
    match s.get(INV_MASS_COLUMN) {
        Some(Column::Scalar(v)) if v.len() == n => v
            .iter()
            .map(|w| if w.is_finite() { w.max(0.0) } else { 0.0 })
            .collect(),
        _ => vec![1.0; n],
    }
}

/// **Separa as peças que a corrente DECLARA.** `None` quando ela não declara colisor nenhum — e é
/// isso que mantém toda cena sem colisão **byte-idêntica**: quem pergunta sai antes de tocar em
/// nada, sem clonar sequer a corrente.
///
/// `varreduras` é quantas passagens o solver faz; `0` devolve `None` (um passe que não varre nada
/// não é um passe, e devolver a corrente clonada seria trabalho por nada).
///
/// ⚠️ **O `rot` só se escreve quando alguma peça de facto RODOU** — a mesma lei estrutural do nó:
/// acrescentar a coluna a uma corrente que não a tinha muda a resposta de todo consumidor a jusante
/// que distinga *«sem rotação»* de *«rotação zero»*.
#[must_use]
pub fn separa_o_que_se_desenha(entrada: &Stream, varreduras: usize) -> Option<Stream> {
    separa_com_relatorio(entrada, varreduras).0
}

/// Como o [`separa_o_que_se_desenha`], mas diz **quantas varreduras de facto correram**.
///
/// ⚠️⚠️ **Ele existe porque o preço de um knob só aparecia no RELÓGIO DE PAREDE** (report do dono,
/// 2026-09-18: quatro rondas de smoke até o perfilador dizer que a fase do Motion eram `68 ms`, e
/// nenhuma delas podia dizer se o cursor estava em `64` ou em `1024`). ⭐ E ele não é um contador
/// novo: o [`crate::separate`] **já devolvia** o número — era a porta que o deitava fora.
///
/// `0` quer dizer *não houve separação nenhuma* (nada declarou colisor, o interruptor está
/// desarmado, ou ninguém se mexeu).
#[must_use]
pub fn separa_com_relatorio(entrada: &Stream, varreduras: usize) -> (Option<Stream>, Relatorio) {
    let mut r = Relatorio::default();
    let saida = separa_contando(entrada, varreduras, &mut r);
    (saida, r)
}

/// **O que a separação de facto FEZ** — os três números que decidem se um knob está caro, e porquê.
///
/// ⚠️⚠️ **Eles existem porque o relógio sozinho não distingue as causas.** O report de 2026-09-18
/// mediu `49 ms` na fase do Motion, e esse número é compatível com *muitas peças*, com *muitas
/// varreduras* e com *uma pilha muito apertada* — **três cenas diferentes, com três curas
/// diferentes**, e o app não sabia dizer qual. Medido: a `1000` discos e `64` varreduras, o mesmo
/// passe custa `8,7 ms` com `10` vizinhos por peça e `47,5 ms` com `72`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Relatorio {
    /// Quantas varreduras correram de facto (o laço pára no repouso visível).
    pub varreduras: usize,
    /// Quantas peças a corrente declarou como colisor.
    pub pecas: usize,
    /// Quantos CANDIDATOS a grelha entregou, somados sobre as peças — o multiplicador do custo.
    pub candidatos: usize,
    /// ⭐⭐ **Quantas peças a grelha teve de pôr numa camada à parte** por serem grandes de mais
    /// para a malha das outras — o número que separa *«uma pilha apertada»* de *«uma peça grande a
    /// inflar a grelha de todas»*, e sem o qual eu tive de INFERIR a causa da cena do dono a partir
    /// de uma contagem de vizinhos (ver o cabeçalho da [`crate::grelha`]).
    pub grandes: usize,
    /// ⛔ **`true` quando a porta de bissecção `PH2D_CONTACT_UMA_CAMADA` está armada.**
    ///
    /// ⚠️⚠️ **Ela existe porque o readout não sabia dizer QUAL motor o produziu** (report do dono,
    /// 19/09): com o corte a não armar, as duas rotas imprimem `0 grande(s)` e a linha lê-se igual
    /// nos dois lados de um A/B. *Um instrumento de bissecção que não se identifica não bissecta
    /// nada.*
    pub bisseccao: bool,
}

impl Relatorio {
    /// Candidatos por peça — a densidade que o custo de uma varredura de facto sente.
    #[must_use]
    pub fn vizinhos_por_peca(&self) -> f32 {
        if self.pecas == 0 {
            return 0.0;
        }
        #[expect(
            clippy::cast_precision_loss,
            reason = "contagens de cena, muito abaixo de 2^24"
        )]
        let v = self.candidatos as f32 / self.pecas as f32;
        v
    }
}

fn separa_contando(entrada: &Stream, varreduras: usize, r: &mut Relatorio) -> Option<Stream> {
    if varreduras == 0 {
        return None;
    }
    let colisores = crate::colisores(entrada)?;
    let n = entrada.count();
    if colisores.len() != n || !colisores.iter().any(Option::is_some) {
        return None;
    }
    let Some(Column::Vec2(p)) = entrada.get(P_COLUMN) else {
        return None; // sem posição não há o que afastar
    };
    if p.len() != n {
        return None;
    }
    let w = pesos(entrada, n);
    let inv_i = crate::inv_inercias(entrada, &colisores, &w);
    let mut pos = p.clone();
    let mut giro = vec![0.0f32; n];
    r.pecas = n;
    (r.candidatos, r.grandes) = crate::candidatos_e_grandes(&colisores, &pos, &w);
    r.bisseccao = crate::grelha::uma_camada_por_ordem();
    r.varreduras = crate::separate(
        &mut pos,
        &mut Saida { giro: &mut giro },
        &Pecas::novas(&colisores, &w, &inv_i),
        varreduras,
    );
    // ⭐⭐ **A MISTURA, no fim — a lei do nó termo a termo** (doc 115 §15.1). `k = 1` (a coluna
    // ausente) devolve o que o motor deu, **ao bit**: o `if` é o que garante que nem a aritmética
    // de vírgula flutuante corre para quem não declara atenuação nenhuma.
    //
    // ⛔ **E ela entra AQUI, antes da cerca do «nada se mexeu»** — senão uma cena inteiramente
    // atenuada (todo `falloff = 0`) devolveria uma corrente CLONADA que é igual à de entrada,
    // gastando uma cópia por quadro para não mudar um bit.
    let fall = atenuacao(entrada, n);
    for i in 0..n {
        let k = fall[i];
        if k < 1.0 {
            pos[i] = [
                p[i][0] + (pos[i][0] - p[i][0]) * k,
                p[i][1] + (pos[i][1] - p[i][1]) * k,
            ];
            giro[i] *= k;
        }
    }
    // ⭐ **Nada se mexeu ⇒ nada se escreve.** Uma cena cujas peças já estão separadas devolve a
    // corrente de entrada intacta, e o quadro é byte-idêntico ao de antes desta wave existir.
    if pos == *p && giro.iter().all(|g| *g == 0.0) {
        return None;
    }
    let mut out = Stream::new(n);
    for (name, col) in entrada.columns() {
        if name != P_COLUMN {
            out.set(name.clone(), col.clone());
        }
    }
    out.set(P_COLUMN, Column::Vec2(pos));
    if giro.iter().any(|g| *g != 0.0) {
        let antes = match entrada.get(ROT_COLUMN) {
            Some(Column::Scalar(v)) if v.len() == n => v.clone(),
            _ => vec![0.0; n],
        };
        out.set(
            ROT_COLUMN,
            Column::Scalar(antes.iter().zip(&giro).map(|(a, g)| a + g).collect()),
        );
    }
    Some(out)
}

#[cfg(test)]
#[path = "passe_tests.rs"]
mod tests;
