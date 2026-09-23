//! ⭐⭐⭐ **A COLUNA DO PAINEL — dentro de um painel, o valor arranca numa coluna SÓ.**
//!
//! ⛔⛔ Ordem do dono (2026-09-23): *«quero tudo alinhado e padronizado»* — a resposta à pergunta
//! que lhe foi devolvida com o preço ao lado: *secções diferentes do mesmo painel punham a coluna
//! do valor em sítios diferentes* (Inspector `111` / `136` / `141,8`, Grid Snap `110` / `122`,
//! medidos pela varredura `onde_comeca_o_valor`), porque cada [`super::Seccao`] mede os nomes
//! DELA e cede pelos campos DELA. Isso estava certo como lei de secção e errado como PAINEL.
//!
//! # A lei
//!
//! Enquanto um painel pinta ([`crate::panel::ErasedPanel::paint`]), cada pedido de coluna
//! (`nome mais largo`, `o que o controlo precisa`, `quanto a linha está recuada`) é LEMBRADO, e o
//! valor de toda linha arranca no **mesmo `x`**: o mais largo que os NOMES pedem, sem apertar o
//! CONTROLO de nenhuma linha de largura inteira abaixo do que ele declara (`coluna_do_painel`).
//!
//! - ⭐ **A lei de uma secção tem DUAS perguntas e o painel lê-as separadas**
//!   ([`super::row::limites_da_seccao`]): a METADE (ou o empréstimo de um nome largo) é o que o
//!   nome ACEITA, e passa a PISO; a cedência ao controlo é o que ele PRECISA, e passa a TECTO. ⛔ A
//!   1.ª redacção tomava o `min` das colunas de cada secção, e no degrau estreito o Inspector
//!   passava de `79` para `211` nomes cortados (a coluna do nome caía a `48 px`).
//! - ⭐ O preço é o que foi dito ao dono: nomes de secções que emprestavam para lá da coluna perdem
//!   essa folga, e o que não couber sai cortado **com balão** (a lei de 19/09) — medido, `24` nomes
//!   do Inspector e `2` do Painter a `300 px`.
//! - ⚠️ **Um cartão recuado alinha-se sempre que o campo dele ainda caiba** (`72 px`, ordem do dono
//!   de 24/05); quando não cabe, só ELE recua o recuo dele — e só no fim estreito do dock.
//! - ⭐ **Um CARTÃO recuado alinha com o painel:** uma linha dentro de um cartão (recuada o mesmo
//!   dos dois lados) tem a coluna do nome encurtada pelo recuo, para o VALOR cair no mesmo `x` das
//!   linhas de fora. Medido antes: os 52 números do `Platform Player` arrancavam `6 px` à esquerda
//!   do resto do Inspector, e o cartão de instância a `79,7` contra `111`. ⚠️ Uma linha que NÃO é
//!   simétrica (as duas metades de um par de
//!   parâmetros, uma célula de grelha) fica com a coluna dela — alinhá-la ao painel partiria o par.
//! - ⭐ **Guardam-se os PEDIDOS e não a coluna:** um pedido não depende da largura, logo a coluna
//!   acerta no mesmo quadro em que o dock é arrastado — guardar o `x` daria um quadro desalinhado
//!   por cada largura nova, isto é, o arrasto inteiro. E a BASE (o rectângulo das linhas de largura
//!   inteira) sai da PRIMEIRA linha do quadro mais o recuo que ela tinha no quadro anterior — a base
//!   anda com a janela, com o dock e com a barra de rolagem, sem esperar um quadro.
//! - ⚠️ **A base do quadro anterior é a geometria MAIS LARGA que se repete** — nem o envelope (uma
//!   célula de par que começa na borda arrastava-o `67 px` e punha as `392` linhas do Inspector
//!   como assimétricas no quadro seguinte) nem a moda (no degrau estreito há mais linhas DENTRO dos
//!   cartões do que fora, e a moda escolhia a de um cartão).
//! - ⭐ **A memória só CRESCE** (enquanto o painel viver): uma secção que se fecha não tira o pedido
//!   dela, senão fechar uma secção mexia a coluna das outras — *uma coluna que muda quando uma linha
//!   aparece é uma coluna que salta debaixo do olho do artista* (o doc da `Seccao::medida`).
//! - ⚠️ **Um painel converge no 3.º quadro, e não no 2.º:** o 1.º não tem base lembrada (e se a
//!   primeira linha for um cartão recuado, as de largura inteira leem-se assimétricas e não pedem
//!   nada); o 2.º já tem a base e aprende os pedidos de todas; do 3.º em diante a coluna é uma
//!   só. Medido no Inspector: `1` pedido depois do 1.º quadro, `51` depois do 2.º.
//! - ⚠️ **Fora de um painel nada muda** — a função devolve a resposta da secção, byte a byte.

use std::cell::RefCell;

/// Um pedido de coluna — o nome mais largo, o que o controlo precisa e o recuo da linha.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Pedido {
    nome: Option<f32>,
    precisa: Option<f32>,
    recuo: f32,
}

/// ⭐ **A memória de um painel** — os pedidos de coluna que ele já fez e onde a BASE estava em
/// relação à primeira linha. Mora no [`crate::panel::ErasedPanel`], um por painel.
#[derive(Clone, Debug, Default)]
pub struct ColunaDoPainel {
    pedidos: Vec<Pedido>,
    base: Option<Base>,
}

/// ⭐ **Onde a base estava, dita a partir da PRIMEIRA linha do quadro** — e não da faixa do painel.
///
/// ⚠️ A faixa (`PaintCtx::slot`) é a posição de NASCIMENTO de um painel que flutua (o Grid Snap),
/// logo um recuo medido contra ela ficaria errado a cada quadro de um arrasto; a primeira linha
/// anda com a janela, com o dock e com a rolagem, no MESMO quadro.
#[derive(Clone, Copy, Debug)]
struct Base {
    /// `base.x − primeira.x` e `primeira.direita − base.direita`, com sinal trocado: o quanto a
    /// primeira linha está RECUADA de cada lado.
    recuo: (f32, f32),
    /// A largura da primeira linha — se a deste quadro for outra, a primeira linha mudou de
    /// ESPÉCIE (outra selecção no Inspector) e o recuo antigo não a descreve.
    primeira_w: f32,
}

/// O estado de um quadro em pintura.
struct Quadro {
    pedidos: Vec<Pedido>,
    antes: Option<Base>,
    /// `(x, direita)` da base — fixada pela primeira linha.
    base: Option<(f32, f32)>,
    /// `(x, direita)` da primeira linha, e a de todas — para o próximo quadro achar a MODA.
    primeira: Option<(f32, f32)>,
    linhas: Vec<(i32, i32)>,
}

thread_local! {
    /// O quadro do painel que está a pintar AGORA (`None` = nenhum painel a pintar).
    static PINTANDO: RefCell<Option<Quadro>> = const { RefCell::new(None) };
}

/// A tolerância de «recuada o mesmo dos dois lados», em px — meio píxel de arredondamento de
/// cada lado.
const SIMETRIA: f32 = 1.0; // LITERAL-PX-OK: tolerancia de arredondamento, nao metrica de UI

impl ColunaDoPainel {
    /// Corre `pinta` com esta memória em vigor, e guarda o que ela aprendeu.
    ///
    /// ⚠️ Reentrante: um painel que pinte outro dentro dele guarda e repõe o quadro de fora.
    pub fn pintando<R>(&mut self, pinta: impl FnOnce() -> R) -> R {
        let quadro = Quadro {
            pedidos: std::mem::take(&mut self.pedidos),
            antes: self.base,
            base: None,
            primeira: None,
            linhas: Vec::new(),
        };
        let de_fora = PINTANDO.with(|p| p.replace(Some(quadro)));
        let r = pinta();
        let quadro = PINTANDO.with(|p| p.replace(de_fora));
        if let Some(q) = quadro {
            self.pedidos = q.pedidos;
            if let (Some((px, pr)), Some((ex, er))) = (q.primeira, moda(q.linhas)) {
                self.base = Some(Base {
                    recuo: (px - ex, er - pr),
                    primeira_w: pr - px,
                });
            }
        }
        r
    }

    /// Quantos pedidos esta memória guarda — para os gates.
    #[must_use]
    pub fn quantos(&self) -> usize {
        self.pedidos.len()
    }
}

/// A lei de uma secção partida em `(prefere, tecto do controlo)` — ver
/// [`super::row::limites_da_seccao`].
pub(super) type Limites = fn(f32, f32, Option<f32>, Option<f32>) -> (f32, f32);

/// A coluna de uma linha — a da secção fora de um painel; dentro de um, a que põe o VALOR no
/// `x` do painel.
pub(super) fn no_painel(
    x: f32,
    w: f32,
    desired: Option<f32>,
    control_need: Option<f32>,
    limites: Limites,
) -> f32 {
    let (prefere, tecto) = limites(x, w, desired, control_need);
    let propria = prefere.min(tecto);
    PINTANDO.with(|p| {
        let mut p = p.borrow_mut();
        let Some(q) = p.as_mut() else {
            return propria;
        };
        let direita = x + w;
        if q.primeira.is_none() {
            q.primeira = Some((x, direita));
        }
        q.linhas.push((meio_px(x), meio_px(direita)));
        let antes = q.antes;
        let (bx, br) = *q
            .base
            .get_or_insert_with(|| base_a_partir_de(antes, x, direita));
        let (re, rd) = (x - bx, br - direita);
        // ⚠️ Só a linha recuada o MESMO dos dois lados é do painel — um par de parâmetros, uma
        //    célula de grelha ou uma linha que sai da base respondem por si.
        if re < -SIMETRIA / 2.0 || rd < -SIMETRIA / 2.0 || (re - rd).abs() > SIMETRIA {
            return propria;
        }
        let recuo = (re * 2.0).round() / 2.0;
        let este = Pedido {
            nome: desired,
            precisa: control_need,
            recuo,
        };
        if !q.pedidos.contains(&este) {
            q.pedidos.push(este);
        }
        let valor = coluna_do_painel(&q.pedidos, bx, br - bx, limites);
        // ⚠️ `valor − recuo ≤ tecto` por construção (o painel respeita o tecto de TODA linha); o
        //    `min` só guarda contra o arredondamento do recuo a meio píxel.
        crate::math::safe_clamp(valor - recuo, 0.0, tecto)
    })
}

/// ⭐⭐⭐ **O `x` do valor no painel, relativo à base: a coluna que o NOME MAIS LARGO pede, sem
/// apertar o CONTROLO de nenhuma linha de LARGURA INTEIRA abaixo do que ele declara.**
///
/// `min( max(recuo + prefere), min(tecto das linhas sem recuo) )`.
///
/// ⛔⛔ **A 1.ª redacção era o `min` das colunas de cada secção, e a régua das elisões reprovou-a no
/// degrau estreito com `211` nomes cortados no Inspector contra `79`** — a coluna do rótulo caía a
/// `48 px`. O mecanismo: uma linha SEM nome largo responde a METADE, e a metade não é uma
/// necessidade, é o que ela aceita; o `min` fazia essa resposta mandar no painel inteiro. Separar as
/// duas perguntas ([`super::row::limites_da_seccao`]) dá à METADE o papel de piso e ao controlo o
/// de tecto, que é o que cada uma é.
///
/// ⚠️⚠️ **E o tecto de uma linha RECUADA (um cartão) não manda no painel — duas ordens do dono
/// colidem ali e a do campo ganha, só para ela.** O recuo encurta o controlo do cartão, logo o tecto
/// dele fica `recuo` abaixo do das outras; medido no degrau estreito (`184 px` de conteúdo), o
/// cartão do `Platform Player` punha o painel inteiro a `84` contra `90` e o Inspector armado
/// passava a cortar `197` nomes. ⇒ o painel escolhe a coluna pelas linhas de largura inteira, e um
/// cartão alinha-se com ela sempre que o campo dele ainda tenha os `72 px` que o dono mandou
/// ([`crate::widget::NUMBER_INPUT_MIN_W_PX`]); quando não tem, ELE recua o recuo dele — e só no fim
/// estreito do curso do dock (a `300 px` o tecto do cartão está `63 px` acima da coluna).
fn coluna_do_painel(pedidos: &[Pedido], bx: f32, largura: f32, limites: Limites) -> f32 {
    let mut nome = 0.0_f32;
    let mut controlo = f32::INFINITY;
    for d in pedidos {
        let (p, t) = limites(bx + d.recuo, largura - 2.0 * d.recuo, d.nome, d.precisa);
        nome = nome.max(d.recuo + p);
        if d.recuo == 0.0 {
            controlo = controlo.min(t);
        }
    }
    nome.min(controlo)
}

/// ⭐ **A base do quadro a partir da primeira linha dele.**
///
/// - Sem memória (o 1.º quadro de um painel): a primeira linha É a base — errado se ela for um
///   cartão recuado, e é por isso que o painel converge no 3.º quadro (o 2.º já tem a base e
///   aprende os pedidos das linhas que o 1.º julgou assimétricas).
/// - A primeira linha com a MESMA largura da do quadro anterior: o recuo lembrado.
/// - Outra largura (a primeira linha mudou de espécie): ela é lida como RECUADA POR IGUAL dentro da
///   largura de base lembrada — que é a forma de toda linha de cartão desta casa — em vez de
///   desalinhar o painel inteiro durante um quadro.
fn base_a_partir_de(antes: Option<Base>, x: f32, direita: f32) -> (f32, f32) {
    let w = direita - x;
    match antes {
        None => (x, direita),
        Some(b) if (w - b.primeira_w).abs() <= SIMETRIA => (x - b.recuo.0, direita + b.recuo.1),
        Some(b) => {
            let folga = (b.primeira_w + b.recuo.0 + b.recuo.1 - w) * 0.5;
            (x - folga, direita + folga)
        }
    }
}

/// Uma coordenada em MEIOS píxeis — a chave da moda (duas linhas que diferem por arredondamento
/// contam como a mesma).
fn meio_px(v: f32) -> i32 {
    (v * 2.0).round() as i32
}

/// ⭐ **A BASE do quadro: a geometria `(x, direita)` MAIS LARGA que se REPETE.**
///
/// ⚠️ **Não o envelope:** uma única célula fora da faixa (a metade esquerda de um par que começa na
/// borda) arrastava o envelope `67 px` e punha **todas** as linhas do quadro seguinte como
/// assimétricas (medido no Inspector: `392` de `392`).
///
/// ⚠️⚠️ **E não a MODA:** num Inspector armado estreito há mais linhas DENTRO dos cartões do que
/// fora deles, e a moda escolhia a geometria de um cartão — a base ficava `8 px` recuada, e as
/// linhas de largura inteira liam-se assimétricas (medido no degrau estreito). As linhas de largura
/// inteira são, por construção, as MAIS LARGAS que pedem coluna; exige-se que se repitam para que
/// uma peça solta mais larga (uma só) não mande.
fn moda(mut linhas: Vec<(i32, i32)>) -> Option<(f32, f32)> {
    linhas.sort_unstable();
    let grupos: Vec<((i32, i32), usize)> = linhas
        .chunk_by(|a, b| a == b)
        .map(|run| (run[0], run.len()))
        .collect();
    let largura = |k: (i32, i32)| k.1 - k.0;
    let repetidas = grupos.iter().filter(|(_, n)| *n >= 2);
    let escolhida = repetidas
        .max_by_key(|(k, n)| (largura(*k), *n))
        .or_else(|| grupos.iter().max_by_key(|(k, n)| (*n, largura(*k))));
    escolhida.map(|((x, r), _)| (*x as f32 / 2.0, *r as f32 / 2.0))
}

#[cfg(test)]
#[path = "coluna_do_painel_tests.rs"]
mod tests;
