//! ⭐ **O solver nasce sob os dois tetos do DISPOSITIVO** (W5e): o lado de textura
//! (`max_texture_dimension_2d`, conferido em TODA textura da [`PassList`] antes de alocar — a W5d só
//! conferia a fluência, e a captura fina, 2× mais larga, dava um erro de validação a 2160p e a luz
//! saía apagada) e a MEMÓRIA, que o wgpu não diz quanta há: diz só que RECUSOU (`OutOfMemory`, do
//! driver ou do `MemoryBudgetThresholds` da instância). Recusada, a montagem sai como
//! [`ErroSolver::SemMemoria`] com os bytes que pedia — o que o governador lê.

use ph2d_gpu::GpuContext;

use super::Solver;
use crate::config::Config;
use crate::erro::ErroSolver;
use crate::passlist::PassList;

impl Config {
    /// Toda textura da lista desta configuração cabe no lado `limite` do dispositivo?
    ///
    /// # Errors
    /// [`ErroSolver::GradeGrandeDemais`] com o maior lado que não cabe.
    pub fn cabe_na_placa(&self, limite: u32) -> Result<(), ErroSolver> {
        let cfg = Self {
            grade: self.grade_efetiva(),
            ..*self
        };
        let lado = PassList::de(&cfg, false)
            .texturas
            .iter()
            .map(|t| t.tamanho[0].max(t.tamanho[1]))
            .max()
            .unwrap_or(0);
        if lado > limite {
            return Err(ErroSolver::GradeGrandeDemais { lado, limite });
        }
        Ok(())
    }
}

impl Solver {
    /// [`Self::novo`] com a [`crate::Fabrica`] partilhada: as pipelines de uma grade anterior com as
    /// mesmas constantes voltam sem recompilar (a janela redimensionada custava ~2 s por quadro).
    ///
    /// # Errors
    /// As de [`Self::novo`], e [`ErroSolver::SemMemoria`] quando o dispositivo recusa alocar.
    pub fn novo_com(
        gpu: &GpuContext,
        pedido: Config,
        fabrica: &crate::Fabrica,
    ) -> Result<Self, ErroSolver> {
        // A validação por fora: depois de uma recusa de memória a montagem continua e cria vistas das
        // texturas inválidas — erros que são CONSEQUÊNCIA da recusa (o gate
        // `a_luz_recua_quando_a_placa_recusa_a_memoria` viu-os escapar).
        let validacao = gpu.device.push_error_scope(wgpu::ErrorFilter::Validation);
        let memoria = gpu.device.push_error_scope(wgpu::ErrorFilter::OutOfMemory);
        let feito = Self::montar(gpu, pedido, fabrica);
        let sem_memoria = pollster::block_on(memoria.pop()).is_some();
        let invalido = pollster::block_on(validacao.pop());
        if let (false, Some(e)) = (sem_memoria, &invalido) {
            // Sem recusa, é um defeito: dito, e a luz apaga-se com o aviso (nunca calado).
            eprintln!("[luz2d] o dispositivo recusou a montagem do solver: {e}");
            return Err(ErroSolver::ConfigInvalida(
                "o dispositivo recusou a montagem",
            ));
        }
        match sem_memoria {
            false => feito,
            true => Err(ErroSolver::SemMemoria {
                bytes: PassList::de(
                    &Config {
                        grade: pedido.grade_efetiva(),
                        ..pedido
                    },
                    false,
                )
                .bytes_alocados(),
            }),
        }
    }
}
