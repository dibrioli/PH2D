//! ⭐⭐⭐ **A LEI SOBRE UM SPRITE INTEIRO** — o caminho de REFERÊNCIA, e é isso que ele é.
//!
//! O [`super::acende_texel`] é a lei num ponto; aqui ela corre sobre os três planos que o
//! `baked_form` guarda (`base`, `form`, `form_occ`) e devolve os pixels acesos.
//!
//! # ⛔ Porque isto NÃO é o caminho rápido, com o número
//!
//! Medido pelo [`super::tests`] (`load 3,3`, `--release`, 32 núcleos): a `1024²` este corredor
//! custa `11,1 ms` com **uma** lâmpada e `34,1 ms` com **quatro**, contra um orçamento de quadro de
//! `16,7 ms` — ⇒ ele atravessa o orçamento à **segunda** lâmpada, e o rig permite quatro.
//!
//! *Quem manda no tecto é o dispositivo; este caminho só precisa de computar a mesma resposta.*
//! ⚠️ E o inverso também: ele é **rápido o suficiente para o dono JULGAR a aparência** a uma
//! lâmpada, que é a pergunta que vem antes de qualquer optimização.
//!
//! # ⛔⛔ A LUZ DESTA LEI É ABSOLUTA, e chegar ao ecrã custa DUAS coisas
//!
//! O passe da TINTA é **relativo** — ele divide pelo que uma superfície plana do mesmo material
//! devolveria, e é por isso que tinta plana sai byte-idêntica nele: *ele multiplica códigos por uma
//! razão e nunca sai do espaço de códigos*. Esta lei devolve **radiância**, logo tem de atravessar
//! as duas metades que aquele nunca atravessa:
//!
//! 1. **A VISTA** — a [`Look`] da [`ph2d_view_transform`], *cena linear → ecrã linear em `0..=1`*.
//!    ⚠️ O [`Look::default`] é a **identidade** (`0` stops, `Standard`), logo quem não pedir vista
//!    nenhuma recebe a radiância crua — é isso que torna a exposição uma ESCOLHA visível em vez de
//!    uma constante escondida.
//! 2. **A CURVA** — `linear → sRGB`, no [`acende_faixa`], porque a ranhura para onde estes bytes
//!    vão é `Rgba8UnormSrgb` e o hardware **descodifica** ao amostrar. Ela **não** mora na
//!    [`ph2d_view_transform`], que o declara por escrito (*«⛔ a codificação sRGB NÃO mora aqui — ela
//!    é da `ph2d-color` e de quem escreve bytes»*), e quem escreve bytes é este ficheiro.
//!
//! ⛔⛔⛔ **E até 2026-09-20 a (2) NÃO EXISTIA, com a (1) a pagar por ela.** A redacção anterior desta
//! secção dizia *«escrevê-la direito em bytes dá um objecto visivelmente mais escuro: média `59`
//! contra `105` da tinta — não é um defeito da lei, é a metade que faltava»*, e nomeava **a vista**
//! como essa metade. ⚠️ **A medição estava certa e a atribuição não:** `srgb⁻¹(0,5) = 0,214`
//! reproduz aquele `128 → ~55 ≈ 59` — *o que faltava era a CURVA, e a exposição estava a fazer o
//! trabalho dela*. O preço disso está medido no [`ph2d_form_donation::lei_da_luz::OLHAR_DA_FORMA`]:
//! uma exposição **multiplica tudo** (queima o alto, não salva o escuro) e uma curva **levanta o
//! escuro preservando o alto**, e é por isso que o assado saía ao mesmo tempo estourado e esmagado.
//!
//! # ⛔⛔⛔ O albedo é DESCODIFICADO, e as duas pontas são UMA lei
//!
//! Os bytes de uma ranhura `Individual` são **códigos sRGB** — a convenção está declarada no
//! `individual.rs::copy_from_texture` (*«straight-sRGB8 … copiado byte-a-byte»*) e é o que a
//! importação de PNG e o compositor do Painter honram. A lei do OpenPBR quer **LUZ** ⇒ a entrada
//! descodifica e a saída codifica, e as duas são a mesma decisão tomada uma vez.
//!
//! ⭐⭐ **O que as prende é o texel FORA da silhueta**, onde a lei devolve o albedo **verbatim**:
//! ali o byte só sai intacto se as duas pontas forem uma curva e a inversa dela. Com a codificação
//! escrita sozinha (2026-09-20) um `3` saía **`28`** — e quem o apanhou foi o
//! `fora_da_silhueta_o_byte_sai_intacto_e_o_alfa_atravessa`, sobre uma metade de lei. ⚠️ *Uma lei
//! de duas pontas escrita numa ponta só estraga exactamente os pixels que ela não toca.*
//!
//! ⚠️ **A redacção anterior desta secção defendia o contrário** (*«o passe da tinta sobe o mesmo
//! `base` como `Rgba8Unorm`, logo decodificar aqui faria as duas leis discordarem»*) e ela já
//! trazia a saída escrita: *«se esta convenção estiver errada, ela está errada nas duas»*. ⛔ **E
//! não está errada nas duas, porque as duas leis NÃO fazem a mesma coisa:** a da tinta é
//! **RELATIVA** — ela multiplica códigos por uma razão e nunca sai do espaço de códigos, e é por
//! isso que tinta plana sai byte-idêntica nela — e esta é **ABSOLUTA**. *Só quem sai do espaço de
//! códigos tem de pagar a viagem de volta.*
//!
//! ⚠️ **Custo declarado:** numa tela BRANCA isto vale exactamente zero (`255` é ponto fixo da
//! curva), e é por isso que a foto do dono não o mostrava. Em arte colorida ele vale até `2,3×`
//! no albedo de um meio-tom.

pub use ph2d_view_transform::{Look, ViewTransform};

/// ⭐⭐⭐ **A CONVERSÃO, e ela é UMA decisão com DUAS pontas** — código sRGB ↔ luz.
///
/// Ver a secção *«O albedo é DESCODIFICADO»* no cabeçalho: os bytes de uma ranhura `Individual` são
/// códigos e a lei quer luz, logo a entrada desce a curva e a saída sobe-a.
///
/// ⛔⛔ **Isto é uma PORTA e não duas linhas, porque o segundo consumidor já existe e já divergiu:**
/// o `cada_pixel_e_o_que_a_lei_por_texel_da` reconstruía as duas conversões à mão para comparar a lei
/// por texel com o corredor — e no dia em que a curva entrou ele passou a medir **outro programa**,
/// reprovando sobre produto correcto (`12` contra `1`). *Um arnês que reimplementa a conversão do
/// produto afirma sobre uma lei que o produto não corre.*
pub mod codigo {
    /// Código sRGB → luz. O que o [`super::acende_faixa`] entrega à lei.
    #[must_use]
    pub fn para_luz(byte: u8) -> f32 {
        ph2d_color::srgb::srgb_to_linear_byte(byte)
    }

    /// Luz → código sRGB, com a quantização dentro. O que o [`super::acende_faixa`] escreve.
    ///
    /// ⚠️ **A lei do `NaN` não muda com a curva:** um `NaN` atravessa-a e vira `0` na saturação do
    /// `as u8`, como antes — a cerca do meio-vector degenerado continua a ser a primeira linha de
    /// defesa, e esta a segunda.
    #[must_use]
    pub fn de_luz(linear: f32) -> u8 {
        ph2d_color::srgb::linear_to_srgb_byte(linear)
    }
}

use super::{Lampada, Surface, Texel, acende_texel};

/// Os três planos que o objecto assado guarda, emprestados.
pub struct Planos<'a> {
    /// `(largura, altura)` em texels.
    pub size: (u32, u32),
    /// Os pixels **antes** da luz, `RGBA8`. Ver a nota do módulo sobre o espaço de cor.
    pub base: &'a [u8],
    /// O G-buffer da malha: `[nx, ny, nz, cobertura]` por texel.
    ///
    /// ⚠️ O neutro de *"não há forma aqui"* é `[0, 0, 1]` com cobertura `0` — **não** um zero em
    /// todo o lado: um `z` zero é uma superfície de pé, que é outra coisa.
    pub form: &'a [f32],
    /// A oclusão de forma, um escalar por texel. O neutro é `1`.
    pub form_occ: &'a [f32],
}

impl Planos<'_> {
    /// Quantos texels. ⚠️ Em `usize` de propósito: `w * h` num `u32` estoura a `65 536²`.
    fn texels(&self) -> usize {
        self.size.0 as usize * self.size.1 as usize
    }

    /// ⛔ **Um plano curto RECUSA em voz alta, dizendo QUAL.**
    ///
    /// Sem isto, o laço leria o que coubesse e devolveria uma imagem plausível com a metade de
    /// baixo a preto — *um defeito de tamanho lido como um defeito de luz*.
    ///
    /// ⭐ **`pub` desde que o passe de dispositivo existe, e a razão é uma lei desta casa:** os dois
    /// caminhos correm o MESMO predicado. Um segundo *«este pedido está bem formado»* escrito do
    /// lado da placa continuaria a passar depois de este ficar torto — e o sintoma de um pedido
    /// recusado é o sprite **não mudar nada**, indistinguível de a tecla não ter chegado.
    ///
    /// # Errors
    /// Se algum plano não medir o que o [`Planos::size`] pede, dizendo **qual**.
    pub fn confere(&self) -> Result<usize, String> {
        let n = self.texels();
        for (nome, tem, quer) in [
            ("base", self.base.len(), n * 4),
            ("form", self.form.len(), n * 4),
            ("form_occ", self.form_occ.len(), n),
        ] {
            if tem != quer {
                return Err(format!(
                    "o plano `{nome}` mede {tem} e o sprite de {}x{} pede {quer}",
                    self.size.0, self.size.1
                ));
            }
        }
        Ok(n)
    }
}

/// **ACENDE o sprite inteiro**, repartido pelos núcleos que a máquina tem.
///
/// ⚠️ A repartição é a única coisa que esta porta decide — a LEI é a [`acende_faixa`], e as duas
/// portas chamam-na. *Um corredor que reescrevesse o laço seria a segunda redacção da lei.*
///
/// # Errors
/// Se algum plano não medir o que o `size` pede — ver [`Planos::confere`].
pub fn acende_imagem(
    s: &Surface,
    p: &Planos,
    lampadas: &[Lampada],
    ceu: super::Ceu,
    olhar: Look,
) -> Result<Vec<u8>, String> {
    let faixas = std::thread::available_parallelism().map_or(1, std::num::NonZero::get);
    acende_imagem_com(s, p, lampadas, ceu, olhar, faixas)
}

/// O mesmo, com o número de faixas **dito** em vez de lido da máquina.
///
/// ⭐⭐ **Ele existe para o gate**, e a razão é uma lei desta casa: *um gate que lê o ambiente mede a
/// máquina*. Com a repartição como PARÂMETRO, a igualdade ao bit entre `1` faixa e `N` faixas é uma
/// propriedade que se afirma; lida de `available_parallelism` ela seria uma corrida com sorte.
///
/// # Errors
/// Ver [`acende_imagem`].
pub fn acende_imagem_com(
    s: &Surface,
    p: &Planos,
    lampadas: &[Lampada],
    ceu: super::Ceu,
    olhar: Look,
    faixas: usize,
) -> Result<Vec<u8>, String> {
    let n = p.confere()?;
    let mut out = p.base.to_vec();
    if n == 0 {
        return Ok(out);
    }
    // ⚠️ O passo é em TEXELS e arredonda para cima: com `faixas > n` sobram fatias vazias, que é
    // inofensivo, e com `faixas = 0` o `max(1)` impede uma divisão por zero.
    let passo = n.div_ceil(faixas.max(1));
    std::thread::scope(|sc| {
        for (k, fatia) in out.chunks_mut(passo * 4).enumerate() {
            sc.spawn(move || acende_faixa(s, p, lampadas, ceu, olhar, k * passo, fatia));
        }
    });
    Ok(out)
}

/// ⭐ **A LEI, sobre uma faixa** — e a única cópia dela neste ficheiro.
///
/// `i0` é o índice do primeiro texel da faixa, e `fatia` são os bytes `RGBA` dela, que chegam com o
/// [`Planos::base`] dentro e saem acesos.
///
/// ⚠️ **O ALFA atravessa intacto**, e não é um detalhe: ele é a silhueta do sprite. Uma lei de luz
/// que lhe tocasse mudaria o RECORTE do objecto ao mover a lâmpada.
fn acende_faixa(
    s: &Surface,
    p: &Planos,
    lampadas: &[Lampada],
    ceu: super::Ceu,
    olhar: Look,
    i0: usize,
    fatia: &mut [u8],
) {
    // A `fatia` é um empréstimo do `out`, que nasceu do `base` — logo o albedo lê-se DELA, e é isso
    // que torna a repartição irrelevante para o resultado.
    // ⚠️ `as_chunks_mut` e não `chunks_exact_mut(4)`: com um tamanho constante o clippy pede o
    // primeiro, e ele dá um `&mut [u8; 4]` — o índice deixa de poder sair da casa.
    for (j, px) in fatia.as_chunks_mut::<4>().0.iter_mut().enumerate() {
        let t = Texel {
            normal: {
                let f = (i0 + j) * 4;
                [p.form[f], p.form[f + 1], p.form[f + 2]]
            },
            // ⭐⭐⭐ **DESCODIFICA — e esta metade é OBRIGATÓRIA assim que a outra existe.**
            //
            // A lei do OpenPBR quer o albedo em **LUZ**, e estes bytes são **códigos sRGB** (a
            // ranhura é `Rgba8UnormSrgb`). ⚠️ E a prova de que as duas metades são UMA lei é o
            // texel FORA da silhueta: ali a lei devolve o albedo **verbatim**, logo o byte só sai
            // intacto se a entrada e a saída forem a curva e a INVERSA dela — com a codificação
            // sozinha, um `3` saía `28` (medido; foi um gate que o apanhou).
            //
            // ⭐ O ida-e-volta é exacto nos **256** bytes (varrido), logo o no-op é byte-idêntico.
            albedo: [
                codigo::para_luz(px[0]),
                codigo::para_luz(px[1]),
                codigo::para_luz(px[2]),
            ],
            cobertura: p.form[(i0 + j) * 4 + 3],
            oclusao: p.form_occ[i0 + j],
        };
        let c = acende_texel(s, &t, lampadas, ceu, olhar);
        for k in 0..3 {
            // ⭐⭐⭐ **A CURVA ENTRA AQUI, e é ela que faz o assado ser o que se vê em 3D.**
            //
            // O `c` é display-referred LINEAR (a [`Look`] deixa-o em `0..=1` e diz por escrito que
            // a codificação não mora lá). A ranhura para onde estes bytes vão é `Rgba8UnormSrgb`,
            // que o hardware DESCODIFICA ao amostrar ⇒ escrever `v * 255` põe no ecrã `v` onde a
            // malha põe `srgb(v)`. É a diferença inteira entre as duas esferas da foto do dono:
            // **até `+73` códigos no meio-tom**.
            //
            // ⚠️ **A quantização continua a ser NOSSA e a lei de `NaN` não muda:** o
            // [`ph2d_color::srgb::linear_to_srgb_byte`] faz `encode → ×255 → +0.5 → clamp → as u8`, e um
            // `NaN` atravessa a curva e vira `0` na saturação do `as`, como antes — a cerca do
            // meio-vector degenerado continua a ser a primeira linha de defesa, não esta.
            px[k] = codigo::de_luz(c[k]);
        }
    }
}

#[cfg(test)]
#[path = "imagem_tests.rs"]
mod tests;
