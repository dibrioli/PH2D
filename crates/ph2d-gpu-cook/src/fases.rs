//! **O relógio de fases de um cozimento** — quanto custou a última chamada de
//! [`GpuCook::cook_many`](crate::GpuCook::cook_many), partida em costuras · encode · submit. Só
//! `Instant` e somas de inteiros: nada aloca, e corre sempre (a sonda do custo fixo da rota
//! híbrida lê-o, doc 120 §8.3.1).

use std::time::Duration;

/// A última passagem de cada fase do cozimento.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FasesDoCozimento {
    /// O `enviar_costuras` inteiro (reaproveitar ou subir cada fronteira).
    pub costuras: Duration,
    /// Bytes que a CPU escreveu na placa nas costuras deste cozimento.
    pub bytes_enviados: u64,
    /// Fronteiras subidas de novo.
    pub enviadas: u32,
    /// Fronteiras reconhecidas como as do quadro anterior (zero bytes).
    pub reaproveitadas: u32,
    /// Do encoder criado ao `finish` (os estágios, o lowering, a tomada).
    pub encode: Duration,
    /// Dentro do `encode`: o encoder e o passeio dos estágios.
    pub estagios: Duration,
    /// Dentro do `encode`: o registo da forma, a tomada retida e o lowering das saídas.
    pub saidas: Duration,
    /// Dentro do `encode`: a tomada dos cartões no mesmo submit (o resto do `encode` é o `finish`).
    pub tap: Duration,
    /// O `queue.submit`.
    pub submit: Duration,
    /// Estágios de kernel despachados (um passe de computação cada).
    pub despachos: u32,
    /// Grupos de ligação dos kernels criados de novo × achados na cache entre quadros.
    pub grupos_criados: u32,
    pub grupos_reaproveitados: u32,
    /// Dos grupos criados (só com [`VariantesDoCozimento::diagnostica`]): com um buffer que nenhuma
    /// ligação desta viragem nem da anterior tinha × só com buffers já vistos noutra combinação.
    pub faltas_por_buffer_novo: u32,
    pub faltas_por_combinacao: u32,
    /// Os grupos dos passes estruturais (o gather do carimbo, a compactação): criados × da cache.
    pub grupos_de_stream_op_criados: u32,
    pub grupos_de_stream_op_reaproveitados: u32,
    /// Tabelas (LUT) criadas neste cozimento.
    pub luts_criadas: u32,
    /// Buffers que o pool criou neste cozimento.
    pub buffers_alocados: u32,
    /// Passes de computação abertos no codificador do cozimento.
    pub passes: u32,
}

/// **As variantes do custo fixo da rota híbrida** (doc 120 §8.3.2), escolhidas em execução para a
/// sonda intercalada as medir no MESMO processo. Todas desligadas = o código de sempre.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VariantesDoCozimento {
    /// 1a — a tabela (LUT) reaproveitada pelo conteúdo, como o uniforme.
    pub luts_por_conteudo: bool,
    /// 1b — o `n`-ésimo pedido ao pool recebe o buffer que o `n`-ésimo do cozimento anterior teve.
    pub pool_por_ordem: bool,
    /// 2 — os despachos consecutivos partilham UM passe de computação.
    pub um_passe: bool,
    /// Conta a CAUSA de cada grupo criado (guarda cada buffer ligado — custa µs, fora da régua).
    pub diagnostica: bool,
}

impl FasesDoCozimento {
    /// A soma de dois cozimentos (um quadro que coze vários sub-tiques).
    #[must_use]
    pub fn soma(self, o: Self) -> Self {
        Self {
            costuras: self.costuras + o.costuras,
            bytes_enviados: self.bytes_enviados + o.bytes_enviados,
            enviadas: self.enviadas + o.enviadas,
            reaproveitadas: self.reaproveitadas + o.reaproveitadas,
            encode: self.encode + o.encode,
            estagios: self.estagios + o.estagios,
            saidas: self.saidas + o.saidas,
            tap: self.tap + o.tap,
            submit: self.submit + o.submit,
            despachos: self.despachos + o.despachos,
            grupos_criados: self.grupos_criados + o.grupos_criados,
            grupos_reaproveitados: self.grupos_reaproveitados + o.grupos_reaproveitados,
            faltas_por_buffer_novo: self.faltas_por_buffer_novo + o.faltas_por_buffer_novo,
            faltas_por_combinacao: self.faltas_por_combinacao + o.faltas_por_combinacao,
            grupos_de_stream_op_criados: self.grupos_de_stream_op_criados
                + o.grupos_de_stream_op_criados,
            grupos_de_stream_op_reaproveitados: self.grupos_de_stream_op_reaproveitados
                + o.grupos_de_stream_op_reaproveitados,
            luts_criadas: self.luts_criadas + o.luts_criadas,
            buffers_alocados: self.buffers_alocados + o.buffers_alocados,
            passes: self.passes + o.passes,
        }
    }
}

impl crate::GpuCook {
    /// As fases do último [`Self::cook_many`].
    #[must_use]
    pub fn fases_do_ultimo_cozimento(&self) -> FasesDoCozimento {
        self.fases
    }

    /// Liga as variantes do custo fixo — só a sonda (doc 120 §8.3.2).
    #[doc(hidden)]
    pub fn usa_variantes(&mut self, v: VariantesDoCozimento) {
        self.variantes = v;
        self.pool.por_ordem = v.pool_por_ordem;
        self.ligacoes.diagnostica = v.diagnostica;
    }
}

/// Os bytes que uma costura subida escreve: `count` × a largura de cada coluna.
pub(crate) fn bytes_de(g: &crate::GpuStream) -> u64 {
    g.cols
        .values()
        .map(|c| u64::from(g.count) * crate::stream::element_stride(c.dim))
        .sum()
}
