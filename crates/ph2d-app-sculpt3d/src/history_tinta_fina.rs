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
    /// ⭐ **O RELEVO `[altura, corpo]` de cada uma ANTES do traço**, na mesma
    /// ordem — o impasto do Painter na peça (`docs/3D/29`). `None` quando o
    /// traço não mudou o relevo: uma pincelada de cor sobre uma peça com relevo
    /// não paga o canal.
    relevo: Option<Vec<[f32; 2]>>,
    /// ⭐ **A CAMADA pintada e os bytes dela de antes** (W2, `docs/3D/30`
    /// §11) — com ela o desfazer troca a camada e recompõe a janela; sem ela
    /// (`None`) troca o plano da peça como antes.
    camada: Option<(ph2d_tool_painter::LayerId, Vec<[u8; 4]>)>,
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
            relevo: t.relevo_mudou().then(|| t.base_relevo().to_vec()),
            camada: None,
        })
    }

    /// ⭐⭐⭐ **A janela de um traço sobre a CAMADA `id`** — os bytes de antes
    /// saem da cópia `f32` de antes (`para_bytes`, cuja ida e volta é a
    /// identidade): são EXACTAMENTE os que a camada tinha.
    pub(crate) fn do_traco_na_camada(
        t: &TintaDoTraco,
        id: ph2d_tool_painter::LayerId,
    ) -> Option<Self> {
        let mut j = Self::do_traco(t)?;
        let rgba = t
            .base()
            .iter()
            .zip(t.base_alfa())
            .map(|(&c, &a)| crate::pilha_da_peca::para_bytes(c, a))
            .collect();
        j.cores = Vec::new();
        j.camada = Some((id, rgba));
        Some(j)
    }

    /// ⭐⭐⭐ **Desfaz/refaz na PEÇA** — pela camada se a janela é de uma, pelo
    /// plano se não. `None` = a largada (o plano já não é aquele).
    pub(crate) fn troca_na_peca(self, obj: &mut crate::SceneObject) -> Option<Self> {
        let Some((id, rgba)) = self.camada else {
            return self.troca(obj.tinta.as_mut());
        };
        let crate::objects::SceneObject { tinta, pilha, .. } = obj;
        let (Some(peca), Some(pilha)) = (tinta.as_mut(), pilha.as_mut()) else {
            return None;
        };
        if IdDoPlano::de(peca) != self.plano {
            return None;
        }
        let rgba = pilha.troca_janela(id, &self.amostras, &rgba)?;
        let relevo = match &self.relevo {
            Some(r) => Some(pilha.troca_relevo(id, &self.amostras, r)?),
            None => None,
        };
        crate::tinta_da_peca::pilha::recompoe_sujas(obj, &self.amostras);
        Some(Self {
            relevo,
            camada: Some((id, rgba)),
            ..self
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
        // ⚠️ O relevo troca-se com a cor, na mesma janela. `relevo_mut` CRIA o
        // relevo se o plano de agora não o tiver — que é o caso de refazer um
        // traço de impasto depois de o plano ter sido reconstruído sem ele.
        let relevo = self
            .relevo
            .as_ref()
            .map(|a| super::swap_window(t.relevo_mut(), &self.amostras, a));
        Some(Self {
            cores,
            relevo,
            ..self
        })
    }

    /// Quantas amostras esta janela nomeia — a régua dos gates.
    #[cfg(test)]
    pub(super) fn len(&self) -> usize {
        self.amostras.len()
    }
}

/// ⭐⭐ **Um passo do painel de camadas** (`docs/3D/30` §4, W3) — a troca
/// estrutural da pilha, com a cerca de [`IdDoPlano`]: um plano reconstruído
/// tem outra pilha, e a troca de antes já não a descreve.
pub(crate) struct CamadasDaPeca {
    plano: IdDoPlano,
    troca: crate::pilha_da_peca::TrocaDaPilha,
}

impl CamadasDaPeca {
    /// O passo de antes de uma operação do painel sobre o plano `t`.
    pub(crate) fn da_peca(t: &Tinta, troca: crate::pilha_da_peca::TrocaDaPilha) -> Self {
        Self {
            plano: IdDoPlano::de(t),
            troca,
        }
    }

    /// ⭐⭐ **Desfaz/refaz na PEÇA**: a troca na pilha e a peça recomposta.
    /// `None` = a largada (o plano já não é aquele).
    pub(crate) fn troca_na_peca(self, obj: &mut crate::SceneObject) -> Option<Self> {
        let peca = obj.tinta.as_ref()?;
        if IdDoPlano::de(peca) != self.plano {
            return None;
        }
        let troca = obj.pilha.as_mut()?.troca_estrutura(self.troca)?;
        crate::tinta_da_peca::pilha::recompoe(obj);
        Some(Self { troca, ..self })
    }

    /// Quanto ele segura — a régua do tecto da história.
    pub(crate) fn bytes(&self) -> usize {
        self.troca.bytes()
    }
}

/// ⭐⭐ **O PLANO INTEIRO de antes de um `Fill`** — o irmão de peça inteira da
/// [`JanelaFina`], como a [`super::StrokeUndo::Mask`] é da janela de máscara.
///
/// ⚠️ **Sem índices, e é o que o separa da janela:** o `Fill` escreve toda
/// amostra, e uma janela de `0..n` pagaria `4 B` por amostra só para dizer
/// *«todas»*. A cerca é a mesma [`IdDoPlano`] — a régua do produto, nem mais
/// apertada nem mais frouxa — e mais a CONTAGEM, que aqui não é redundante: a
/// troca é de fatias inteiras, e fatias de tamanhos diferentes não se trocam.
pub(crate) struct PlanoInteiro {
    plano: IdDoPlano,
    cores: Vec<[f32; 3]>,
    /// ⭐ A CAMADA preenchida e o plano dela de antes (W2) — ver
    /// [`JanelaFina::camada`].
    camada: Option<(ph2d_tool_painter::LayerId, Vec<u8>)>,
}

impl PlanoInteiro {
    /// ⚠️ O produto guarda o plano da CAMADA ([`Self::da_camada`], W2); esta
    /// — o plano da peça — fica para os gates da lei.
    #[cfg(test)]
    /// O plano inteiro, tal como está.
    pub(crate) fn de(t: &Tinta) -> Self {
        Self {
            plano: IdDoPlano::de(t),
            cores: t.amostras().to_vec(),
            camada: None,
        }
    }

    /// ⭐⭐ **O balde sobre a CAMADA `id`** — o plano dela de antes, em RGBA8.
    pub(crate) fn da_camada(t: &Tinta, id: ph2d_tool_painter::LayerId, rgba8: Vec<u8>) -> Self {
        Self {
            plano: IdDoPlano::de(t),
            cores: Vec::new(),
            camada: Some((id, rgba8)),
        }
    }

    /// ⭐⭐ **Desfaz/refaz na PEÇA** — ver [`JanelaFina::troca_na_peca`].
    pub(crate) fn troca_na_peca(self, obj: &mut crate::SceneObject) -> Option<Self> {
        let Some((id, rgba8)) = self.camada else {
            return self.troca(obj.tinta.as_mut());
        };
        let crate::objects::SceneObject { tinta, pilha, .. } = obj;
        let (Some(peca), Some(pilha)) = (tinta.as_mut(), pilha.as_mut()) else {
            return None;
        };
        if IdDoPlano::de(peca) != self.plano {
            return None;
        }
        let rgba8 = pilha.troca_plano(id, rgba8)?;
        crate::tinta_da_peca::pilha::recompoe_o_plano(obj);
        Some(Self {
            camada: Some((id, rgba8)),
            ..self
        })
    }

    /// ⭐ **A TROCA**, com a mesma lei da [`JanelaFina::troca`]: devolve a
    /// inversa, ou `None` = largada (o plano já não existe, ou é outro).
    ///
    /// ⚠️ **Troca as fatias no sítio** (`swap_with_slice`) em vez de copiar:
    /// a `16x` isto são ~`300 MB`, e uma cópia pediria outro tanto de pico.
    pub(crate) fn troca(mut self, tinta: Option<&mut Tinta>) -> Option<Self> {
        let t = tinta?;
        if IdDoPlano::de(t) != self.plano || t.amostras().len() != self.cores.len() {
            return None;
        }
        t.amostras_mut().swap_with_slice(&mut self.cores);
        Some(self)
    }

    /// Quanto ela segura — a régua do tecto da história.
    pub(crate) fn bytes(&self) -> usize {
        self.cores.capacity() * size_of::<[f32; 3]>()
            + self.camada.as_ref().map_or(0, |(_, p)| p.capacity())
    }
}

impl JanelaFina {
    /// Quanto ela segura — a régua do tecto da história.
    ///
    /// ⛔ **Ela não existia, e o tecto não a contava:** o braço do traço no
    /// [`super::StrokeUndo::footprint_bytes`] terminava num `..`, que engolia o
    /// campo `finas` em silêncio — um traço fino punha na fila `16 B` por
    /// amostra tocada que a poda não via.
    pub(crate) fn bytes(&self) -> usize {
        self.amostras.capacity() * size_of::<u32>()
            + self.cores.capacity() * size_of::<[f32; 3]>()
            + self
                .relevo
                .as_ref()
                .map_or(0, |a| a.capacity() * size_of::<[f32; 2]>())
            + self.camada.as_ref().map_or(0, |(_, p)| p.capacity() * 4)
    }
}

#[cfg(test)]
#[path = "history_relevo_tests.rs"]
mod relevo_tests;
#[cfg(test)]
#[path = "history_tinta_fina_tests.rs"]
mod tests;
