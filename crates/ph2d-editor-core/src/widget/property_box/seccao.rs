//! ⭐⭐⭐ **O que uma SECÇÃO declara sobre as linhas dela** — e as colunas que daí saem.
//!
//! ⚠️ **Irmão por RESPONSABILIDADE do [`super::row`]** (2026-09-15): aquele ficheiro responde *como
//! uma faixa se reparte entre um nome e um controlo*; este responde *o que a secção declara antes
//! de a repartição poder acontecer*. Os dois crescem por motivos diferentes.
//!
//! ⛔⛔ **E ele vive no `widget` e não no [`crate::property_row`], por um CICLO:** o `property_row`
//! já depende do `widget` (ele chama a `property_row_columns_for`, o `paint_property_label`, o
//! `NumberInput`), logo pôr a declaração do outro lado faria o `widget` depender dele — e o
//! `architecture_the_foundation_modules_form_a_dag` recusa a aresta. *A declaração de uma linha
//! pertence a quem sabe repartir a linha.* O `property_row` re-exporta-a, e nenhum chamador muda.

use super::PropertyRow;
use ph2d_text::TextSystem;
use ph2d_tokens::TypeToken;

/// ⭐⭐⭐ **O QUE UMA SECÇÃO DECLARA SOBRE AS LINHAS DELA — e é UMA declaração, não uma por linha.**
///
/// ⛔⛔ **Report do dono, 2026-09-15, com foto do painel da Grelha e uma seta na linha
/// *Major every (px)*:** *«a caixa recua quando na verdade o nome deveria criar as colunas»*.
///
/// Medido nesse dia com o sistema de texto REAL (`Sm`), painel a `220`:
///
/// | linha | o nome quer | a coluna que ela recebia | a caixa |
/// |---|---|---|---|
/// | `Cell size (px)` | `70,1` | `90,0` (a metade) | `x = 98,0` · `w = 84,0` |
/// | **`Major every (px)`** | **`92,2`** | **`92,2`** | **`x = 100,2` · `w = 81,8`** ⛔ |
/// | `Origin X (px)` | `70,7` | `90,0` | `x = 98,0` · `w = 84,0` |
/// | `Origin Y (px)` | `70,4` | `90,0` | `x = 98,0` · `w = 84,0` |
///
/// ⇒ *a linha com o nome mais comprido empurrava a caixa DELA e mais nenhuma*, e a coluna que o
/// dono mandou alinhar (*«as labels alinhadas todas à direita»*) saía esfarrapada **por
/// construção**.
///
/// # ⚠️⚠️ A causa é uma ASSIMETRIA que o doc da porta já denunciava — na outra metade
///
/// A largura da coluna do nome sai de **duas** grandezas, e até hoje elas tinham granularidades
/// diferentes:
///
/// - o **`control_need`** sempre foi da SECÇÃO, com a razão escrita na
///   [`super::property_label_col_w_for`]: *«se cada linha cedesse pelo que ELA precisa, a
///   coluna saía esfarrapada»*;
/// - o **nome** era medido POR LINHA, dentro da [`crate::property_row`].
///
/// ⛔ E ele entra na conta **duas** vezes: como o que a coluna pode pedir emprestado ao controlo
/// (§6) e como o **piso da cedência** (§6-ter, *«nunca abaixo do que o rótulo precisa»*). Medido a
/// `273,3` na secção *Transform* do Inspector, os dois papéis produziam `104,6` para
/// `Position X / Y` e **`56,3`** para `Rotation` — `48 px` de desalinhamento **dentro da mesma
/// secção**, que é a mesma doença da foto num regime mais largo.
///
/// ⇒ as duas grandezas passam a viajar **juntas**, numa declaração que a secção faz uma vez e
/// entrega a todas as linhas dela. *Uma coluna é uma resposta da SECÇÃO; uma resposta por linha é
/// uma coluna por linha.*
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Seccao {
    campos: usize,
    nome_w: Option<f32>,
}

impl Seccao {
    /// ⭐ **Mede o rótulo mais largo da secção, na fonte E NO PESO em que ele vai ser pintado.**
    ///
    /// ⚠️ **A medição é da PORTA, nunca do chamador:** a fonte é a [`TypeToken::Sm`] e o peso é o
    /// `Medium` do [`TextSystem::prefix_width`] — *medir num peso e pintar noutro corta curto*
    /// (defeito que esta casa já pagou duas vezes), e um painel não tem por que saber disso.
    ///
    /// ⚠️ **`nomes` são os da SECÇÃO inteira, não os desta linha** — incluindo os das linhas que
    /// este quadro não vai pintar, se elas partilham a coluna: *uma coluna que muda quando uma
    /// linha aparece é uma coluna que salta debaixo do olho do artista.*
    #[must_use]
    pub fn medida(text_system: &mut TextSystem, campos: usize, nomes: &[&str]) -> Self {
        debug_assert!(
            !nomes.is_empty(),
            "uma seccao sem nomes nao tem coluna para medir — use `Seccao::apenas_campos`"
        );
        let fonte = TypeToken::Sm.px();
        let nome_w = nomes
            .iter()
            .map(|n| text_system.prefix_width(n, fonte))
            .fold(None::<f32>, |acc, w| Some(acc.map_or(w, |a| a.max(w))));
        Self {
            campos: if campos == 0 { 1 } else { campos },
            nome_w,
        }
    }

    /// ⭐ **A secção que só declara quantas componentes tem.**
    ///
    /// A coluna fica na **metade** da linha e não há cedência nenhuma — é o comportamento de quem
    /// não sabe que nomes vai pintar. ⚠️ Sem cedência, uma linha de várias componentes **reflui**
    /// em vez de pedir espaço ao nome (§6-bis).
    #[must_use]
    pub const fn apenas_campos(campos: usize) -> Self {
        Self {
            campos: if campos == 0 { 1 } else { campos },
            nome_w: None,
        }
    }

    /// ⭐ **A mesma secção, com pelo menos `n` componentes** — para a linha que traz MAIS campos do
    /// que a secção declarou.
    ///
    /// ⚠️ **A COLUNA não muda**: ela é da secção. O que muda é só quanto o controlo desta linha
    /// precisa para não quebrar, que é uma pergunta da LINHA.
    #[must_use]
    pub fn com_pelo_menos(self, n: usize) -> Self {
        Self {
            campos: self.campos.max(n).max(1),
            nome_w: self.nome_w,
        }
    }

    /// Quantas componentes tem **a linha que a secção não quer ver quebrar**.
    #[must_use]
    pub const fn campos(self) -> usize {
        self.campos
    }

    /// O rótulo mais largo da secção, já medido — `None` quando ela não os enumerou.
    #[must_use]
    pub const fn nome_w(self) -> Option<f32> {
        self.nome_w
    }
}

/// ⭐⭐⭐ **AS COLUNAS de uma linha, derivadas do que a SECÇÃO declarou — uma vez, para todas.**
///
/// ⚠️ **Ela existe porque as duas famílias de linha (a que traz campos e a que só traz o nome) têm
/// de cair no MESMO `x`.** Antes de 2026-09-15 a primeira media o nome dela e a segunda nem isso —
/// e uma secção que misturasse as duas (amostragem · 9-slice · visibilidade) desalinhava sem que
/// nenhuma das duas estivesse «errada» sozinha. *Duas derivações da mesma coluna são duas colunas.*
#[must_use]
pub fn colunas_da_linha(x: f32, w: f32, y: f32, h: f32, seccao: Seccao) -> PropertyRow {
    let gap = ph2d_tokens::control_gap_px();
    // ⭐⭐ **O que o CONTROLO precisa para não quebrar** — `n` caixas ao piso, com os vãos.
    let n = seccao.campos() as f32;
    let precisa = n * super::super::NUMBER_INPUT_MIN_W_PX + (n - 1.0) * gap;
    super::property_row_columns_for(x, w, y, h, seccao.nome_w(), Some(precisa))
}
