//! ⭐⭐ **O RELEVO de cada camada do Composite Brush** — a metade que a pilha perdia (fila 44,
//! item 8).
//!
//! # O defeito, medido
//!
//! A pilha que ACUMULA ([`super::composite_acumulado`]) deposita cada camada num PLANO de medição, e
//! força ali `Draw To = Color`: um plano é RGBA, e o relevo é da TELA. A frase estava certa e a
//! consequência não tinha dono — **nenhuma camada chegava ao depósito de altura**. Sonda
//! `diag_o_relevo_da_pilha.rs` (ao lado), Impasto, o mesmo traço:
//!
//! | pilha | corpo depositado | com `Draw To = Depth`, tinta |
//! |---|---|---|
//! | nenhuma | `974,24` | `0` |
//! | 1 Brush (a rota de sempre) | `974,24` | `0` |
//! | **2 Brush** | **`0,00`** | **`393 834`** |
//!
//! ⇒ com duas camadas ou mais **o Impasto pintava chapado**, e um pincel de só-relevo pintava COR. E a
//! camada `Erase` não mordia o relevo que a borracha avulsa morde (`0,840 → 0,633` no centro de um
//! dab a força `0,5`). O mesmo vale para o FILME do papel no Digital, que passa pela mesma porta.
//!
//! # A lei
//!
//! **O relevo é da TELA e a cor é do PLANO, e cada um tem uma porta.** Os planos ficam com o
//! `Draw To` do ARTISTA — é ele que corta o pigmento de um pincel que deposita corpo num FILME, e
//! forçá-lo a `Color` pintava a camada com a tinta cheia do Digital (pior `178` contra o avulso), e
//! com `Depth` punha cor onde não devia haver nenhuma. O que impede o corpo de ser depositado dentro
//! de um plano é o `acumulando_no_plano`, lido pelo despacho ANTES do depósito de altura.
//!
//! O relevo corre à parte: cada camada viva chama o **depósito de altura de sempre**
//! ([`PainterTool::stamp_dabs_height`]) sobre a lista dela, com a força e a dureza dela —
//! directamente, e **não** pelas rotas de cor (o ramo «só relevo» do despacho cala a cor no
//! Impasto, mas no Digital com filme de papel ele a pintaria).
//!
//! **Resultado, medido:** cor e relevo da pilha IDÊNTICOS AO BIT aos do pincel avulso, no Digital e
//! no Impasto, para o Brush e para a borracha (gates em `composite_relevo_tests.rs`).
//!
//! * **Brush** deposita o corpo no envelope do traço, que só assenta na camada ao soltar. O envelope
//!   é um MÁXIMO (uma passagem deixa uma espessura), logo a ordem entre camadas não o muda.
//! * **Erase de escopo `Tudo`** morde o relevo ASSENTE, pela lei da borracha avulsa.
//! * **Erase de escopo `Traco`** não o toca: ela devolve o `pre`, e o `pre` do relevo assente é ele
//!   próprio.
//! * **Blur** não mexe no relevo, como o Blur avulso (medido: `974,24` antes e depois).
//! * **Smear** já o arrasta pela sessão de deformação, como o avulso (medido: `929,24` nos dois).
//!
//! ⏳ **O que fica ABERTO, nomeado:** uma Erase **acima** de um Brush apaga a COR que o Brush pôs
//! neste traço e **não** o CORPO dele, porque o corpo ainda vive no envelope, que só assenta no fim.
//! Curá-lo pede compor o envelope pela mesma lei dos planos de cor (a cobertura acumulada da borracha
//! a multiplicar a tinta do envelope), e é trabalho próprio.
//!
//! # E o TILING, que a mesma régua apanhou
//!
//! A região da pilha era medida com os dabs SEM embrulhar ([`super::region::caixa_das_camadas`]):
//! um lote com o cursor já para lá da borda dava `None` e a pilha saía cedo, e as cópias do outro
//! lado da costura nunca chegavam à tela — `0` px de cor contra `738` do avulso, no Digital
//! também. E o depósito da pilha não publicava os GRUPOS do Tiling, logo cada cópia embrulhada
//! tirava a sua própria moldura aleatória (régua: o traço deslocado de uma largura inteira tem de
//! pintar a mesma imagem, porque a tela é um toro).
//!
//! # O estado por traço é POR CAMADA
//!
//! O depósito de altura lembra onde o lote anterior acabou (`last_height_center`, para o corpo de um
//! dab se ligar ao anterior em vez de ficar uma conta de rosário) e a onda do Push. Partilhados, a
//! 1.ª camada de um lote ligar-se-ia à ÚLTIMA camada do lote anterior — outro tamanho, outra força —,
//! logo cada camada guarda os seus e troca-os à volta da passagem, como o `rng_camada` e a máscara.

use super::composite::{CompositeOp, EscopoDaBorracha};
use super::relief_state::WaveTip;
use crate::tool::PainterTool;
use ph2d_painter_brush::Dab;

/// O estado do depósito de altura que é de UMA camada ao longo do traço.
#[derive(Default)]
pub(super) struct RelevoDaCamada {
    cadeia: Vec<Option<([f32; 2], f32)>>,
    onda: Vec<(f32, Option<WaveTip>)>,
}

/// Que relevo a camada escreve — a porta ÚNICA da pergunta.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(super) enum RelevoDaOp {
    /// Deposita o corpo no envelope do traço.
    Corpo,
    /// Morde o relevo assente (a borracha avulsa).
    Morde,
    /// Não o toca.
    Nada,
}

impl CompositeOp {
    /// O relevo que esta operação escreve, dado o escopo da borracha.
    pub(super) fn relevo(self, escopo: EscopoDaBorracha) -> RelevoDaOp {
        match self {
            Self::Brush => RelevoDaOp::Corpo,
            Self::Erase if matches!(escopo, EscopoDaBorracha::Tudo) => RelevoDaOp::Morde,
            Self::Erase | Self::Blur | Self::Smear => RelevoDaOp::Nada,
        }
    }
}

impl PainterTool {
    /// **O relevo da camada `pos`** sobre os dabs novos dela. Corre ANTES da acumulação da cor: o
    /// depósito de altura lê uma CÓPIA do fluxo aleatório, e os dois têm de partir do mesmo ponto.
    pub(super) fn relevo_da_camada(&mut self, pos: usize, dabs: &[Dab]) {
        let layer = self.paint.composite[pos];
        let morde = match layer.op.relevo(layer.erase_scope) {
            RelevoDaOp::Corpo => false,
            RelevoDaOp::Morde => true,
            RelevoDaOp::Nada => return,
        };
        // A lista com o Tiling, e os GRUPOS dela: uma cópia embrulhada liga-se ao antecessor do
        // ORIGINAL dela, senão o corpo seria uma barra atravessada na tela.
        let tiling = self.paint.tiling;
        let (lista, grupos) = if tiling[0] || tiling[1] {
            super::tiling::tiled_dabs_grouped(dabs, self.source_size, tiling)
        } else {
            (dabs.to_vec(), Vec::new())
        };
        let grupos_de_antes = std::mem::replace(&mut self.paint.dab_groups, grupos);
        let saved_strength = self.paint.brush.strength;
        let saved_hardness = self.paint.brush.hardness;
        let saved_eraser = self.paint.eraser;
        let saved_rng = self.paint.tex_rng;
        self.paint.brush.strength = layer.strength;
        self.paint.brush.hardness = layer.hardness.unwrap_or(saved_hardness);
        self.paint.eraser = morde;
        self.paint.tex_rng = self.paint.rng_camada[pos];
        std::mem::swap(
            &mut self.paint.stroke_mask,
            &mut self.paint.composite_mask[pos],
        );
        let estado = &mut self.paint.pilha.relevo[pos];
        std::mem::swap(
            &mut self.paint.relief.last_height_center,
            &mut estado.cadeia,
        );
        std::mem::swap(&mut self.paint.relief.stroke_wave, &mut estado.onda);

        let spec = self.stroke_spec();
        self.stamp_dabs_height(&lista, &spec);

        let estado = &mut self.paint.pilha.relevo[pos];
        std::mem::swap(
            &mut self.paint.relief.last_height_center,
            &mut estado.cadeia,
        );
        std::mem::swap(&mut self.paint.relief.stroke_wave, &mut estado.onda);
        std::mem::swap(
            &mut self.paint.stroke_mask,
            &mut self.paint.composite_mask[pos],
        );
        self.paint.tex_rng = saved_rng;
        self.paint.eraser = saved_eraser;
        self.paint.brush.hardness = saved_hardness;
        self.paint.brush.strength = saved_strength;
        self.paint.dab_groups = grupos_de_antes;
    }
}
