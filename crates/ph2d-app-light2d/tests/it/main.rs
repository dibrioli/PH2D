//! Os gates da família da luz 2D (plano `docs/Luz2D/01_plano.md` §5). Um binário por crate; os de
//! placa precisam de `PH2D_GPU=1` (exclusão na placa) e saltam sem adaptador.

mod captura_por_texel;
mod comum;
mod g11_pan_sem_tremor;
mod g6_ponto_neutro;
mod paridade_luz_recebida;
