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
//! # ⛔⛔ A LUZ DESTA LEI É ABSOLUTA, e sem a VISTA ela chega ao ecrã crua
//!
//! O passe da TINTA é **relativo** — ele divide pelo que uma superfície plana do mesmo material
//! devolveria, e é por isso que tinta plana sai byte-idêntica nele. Esta lei devolve **radiância**,
//! e escrevê-la direito em bytes dá um objecto visivelmente mais escuro: medido na placa sobre a
//! mesma peça, média `59` contra `105` da tinta. *Não é um defeito da lei — é a metade que faltava.*
//!
//! ⇒ ela sai pela [`Look`] da [`ph2d_view_transform`], que é a lei desta casa para *cena linear →
//! ecrã*, e não por uma exposição escrita aqui. ⚠️ O [`Look::default`] é a **identidade** (`0`
//! stops, `Standard`), logo quem não pedir vista nenhuma recebe a radiância crua — e é isso que
//! torna a exposição uma ESCOLHA visível em vez de uma constante escondida.
//!
//! # ⚠️ O albedo é lido LINEAR, e a escolha é de CONSISTÊNCIA e não de física
//!
//! O passe da tinta sobe o mesmo `base` como **`Rgba8Unorm`** (nunca `…Srgb`), logo ele já trata
//! estes bytes como valores lineares. ⛔ Decodificar sRGB aqui faria as duas leis discordarem sobre
//! o que os MESMOS bytes significam, e a diferença apareceria como *"o PBR está mais escuro"* —
//! um defeito de ponte lido como um defeito de lei. *Se esta convenção estiver errada, ela está
//! errada nas duas, e é uma pergunta sobre o `base` — não sobre o OpenPBR.*

pub use ph2d_view_transform::{Look, ViewTransform};

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
            albedo: [
                f32::from(px[0]) / 255.0,
                f32::from(px[1]) / 255.0,
                f32::from(px[2]) / 255.0,
            ],
            cobertura: p.form[(i0 + j) * 4 + 3],
            oclusao: p.form_occ[i0 + j],
        };
        let c = acende_texel(s, &t, lampadas, ceu, olhar);
        for k in 0..3 {
            // ⚠️ `clamp` ANTES do `as u8`: um `as` satura, mas um `NaN` vira `0` em silêncio — e é
            // a cerca do meio-vector degenerado que garante que ele não chega aqui. Esta é a
            // segunda linha de defesa, não a primeira.
            px[k] = (c[k].clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
        }
    }
}

#[cfg(test)]
#[path = "imagem_tests.rs"]
mod tests;
