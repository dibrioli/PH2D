//! ⭐⭐⭐⭐ **A JANELA DO PLANO DE TINTA FINA** — o QUARTO canal de uma entrada
//! de traço, e a lei que decide se ela ainda descreve o plano de agora.
//!
//! ⛔⛔ **Ela existe por uma medição do report de 21/09** (a sonda
//! `diag_o_ctrl_z_desfaz_a_tinta_fina`): com o plano armado, pintar `1010`
//! amostras e carregar em `Ctrl+Z` deixava **`1010`**. A entrada de desfazer de
//! um traço é a janela de **VÉRTICES tocados**, e a cor fina não escreve no
//! canal por vértice enquanto o gesto dura — o `close_stroke` até saía cedo
//! quando essa janela estava vazia, que é exactamente o caso do pincel fino
//! depois da cura do mesmo dia (*«a tinta deixa de precisar de um vértice
//! debaixo do pincel»*).
//!
//! ⭐ **Nada aqui é material novo:** a [`TintaDoTraco`] já guarda `tocadas` (as
//! amostras que o traço escreveu, cada uma uma vez) e `base` (a cor delas
//! antes) — é a janela, com a mesma forma da que o canal por vértice usa. O
//! que faltava era o canal na entrada e a **cerca** de [`IdDoPlano`].
//!
//! ⚠️⚠️ **É um QUARTO CANAL e não um variant novo**, pela lei que o
//! [`super::undo`] já escreve sobre a máscara e a cor: os canais desfazem-se
//! **cada um por si**, e a pergunta *«qual dos quatro foi?»* não existe. Um
//! gesto que mexa em dois (um pincel de cor com auto-smooth armado escreve
//! POSIÇÕES e AMOSTRAS) desfaz-se inteiro, sem ninguém escolher.
//!
//! ⚠️ **E a lei vive aqui, fora da cena, de propósito:** uma
//! [`super::Sculpt3dScene`] pede um `wgpu::Device` e todo gate que a construa
//! nasce `#[ignore]` — que é a população que nem o arnês de mutação nem o CI
//! correm. *Quando um gate precisa de um device para medir uma decisão que não
//! tem pixel nenhum, a lei está no sítio errado.*

use ph2d_mesh_colors::Tinta;
use ph2d_sculpt3d::tinta_fina::TintaDoTraco;

/// ⭐⭐⭐ **EM QUE PLANO esta janela foi escrita** — a cerca contra o defeito
/// MUDO da fronteira.
///
/// ⛔⛔ O endereço de uma amostra é `(face, sítio)`. Um plano reconstruído
/// sobre outra topologia guarda outra coisa em cada índice, e escrever a cor
/// de antes ali **não estoura e não desenha lixo óbvio** — põe a tinta de uma
/// face na face vizinha, que é o defeito que ninguém consegue atribuir
/// (o cabeçalho da [`crate::tinta_da_peca`] narra-o inteiro).
///
/// ⚠️ **Ela é exactamente tão forte quanto a régua do PRODUTO, e isso é a
/// decisão:** a [`crate::tinta_da_peca::concorda_com`] responde *«este plano
/// ainda descreve esta malha?»* por vértices **e** faces, e é ela que decide se
/// o plano vivo sobrevive ao quadro seguinte. Uma cerca mais apertada aqui
/// seria uma **segunda resposta** à mesma pergunta — e uma mais frouxa
/// escreveria onde o produto já não escreve.
///
/// ⛔⛔ **E ela NÃO responde «os meus índices cabem?», que é outra pergunta e
/// tem cerca própria na [`JanelaFina::troca`].** A 1.ª redacção metia a
/// contagem de amostras aqui, e uma MUTAÇÃO SOBREVIVENTE mostrou porque isso
/// não se pode: dado `(verts, faces, nivel)`, a contagem é **derivada** —
/// dentro deste produto ela nunca difere sozinha (se as três batem, a
/// [`crate::tinta_da_peca::concorda_com`] aceita o plano e ele **não é
/// reconstruído**) ⇒ *um campo redundante numa igualdade é um campo que
/// nenhuma fixtura consegue pôr a decidir*.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct IdDoPlano {
    verts: usize,
    faces: usize,
    nivel: u8,
}

impl IdDoPlano {
    fn de(t: &Tinta) -> Self {
        Self {
            verts: t.topologia().verts(),
            faces: t.topologia().faces(),
            nivel: t.nivel(),
        }
    }
}

/// **As amostras que um traço escreveu, e a cor que elas tinham antes.**
pub(crate) struct JanelaFina {
    plano: IdDoPlano,
    /// Os índices de amostra, na ordem em que o traço lhes tocou.
    amostras: Vec<u32>,
    /// A cor de cada uma ANTES do traço, na mesma ordem.
    cores: Vec<[f32; 3]>,
}

impl JanelaFina {
    /// A janela de um traço que fecha — `None` quando ele não tocou uma
    /// amostra.
    ///
    /// ⚠️ **O `None` é uma afirmação e não uma falha:** um traço de FORMA com
    /// o plano armado empresta-o, escreve zero amostras e devolve-o. Gravar
    /// uma janela vazia poria uma entrada de canal em toda pincelada da peça.
    pub(crate) fn do_traco(t: &TintaDoTraco) -> Option<Self> {
        if t.tocadas().is_empty() {
            return None;
        }
        Some(Self {
            plano: IdDoPlano::de(t.tinta()),
            amostras: t.tocadas().to_vec(),
            cores: t.base().to_vec(),
        })
    }

    /// ⭐⭐ **A TROCA** — instala as cores que ela carrega e devolve a inversa,
    /// que é a janela com as cores que estavam lá.
    ///
    /// `None` quer dizer **largada**: ou a peça já não tem plano (o artista
    /// voltou ao modo `Mesh`), ou o plano de agora não é aquele em que ela foi
    /// escrita.
    ///
    /// ⛔⛔ **Largar é a resposta certa, e carregá-la para a fila oposta seria
    /// um defeito de DIRECÇÃO.** Uma entrada carrega o estado de ANTES; quem a
    /// aplica devolve o de DEPOIS, e é esse que o refazer instala. Uma janela
    /// que não se pôde aplicar não tem o «depois» — devolvê-la a ela própria
    /// poria o `Ctrl+Shift+Z` a instalar as cores de ANTES outra vez, ou seja
    /// **a desfazer duas vezes**, e só no dia em que o artista voltasse a armar
    /// o mesmo degrau. *Um payload que sobrevive à recusa é pior que a recusa.*
    pub(crate) fn troca(self, tinta: Option<&mut Tinta>) -> Option<Self> {
        let t = tinta?;
        if IdDoPlano::de(t) != self.plano {
            return None;
        }
        // ⛔⛔ **A SEGUNDA cerca, e ela é uma pergunta DIFERENTE da de cima:**
        // aquela é *«é o mesmo plano?»* e esta é *«os meus índices cabem?»*. A
        // [`super::swap_window`] indexa **sem cerca nenhuma**, e um `panic` num
        // `Ctrl+Z` é o pior desfecho possível de um canal de desfazer.
        //
        // ⚠️ Hoje ela é **inalcançável pelo produto** — com a identidade a
        // bater, a contagem de amostras é derivada dela — e fica na mesma,
        // porque *a alternativa é uma afirmação sobre a malha com um `panic` à
        // espera*. Há gate que a alcança por construção.
        let n = t.amostras().len();
        if self.amostras.iter().any(|&i| i as usize >= n) {
            return None;
        }
        let cores = super::swap_window(t.amostras_mut(), &self.amostras, &self.cores);
        Some(Self { cores, ..self })
    }

    /// Quantas amostras esta janela nomeia — a régua dos gates.
    #[cfg(test)]
    pub(super) fn len(&self) -> usize {
        self.amostras.len()
    }
}

#[cfg(test)]
#[path = "history_tinta_fina_tests.rs"]
mod tests;
