//! Os gates do solver de luz na GPU (plano `docs/Luz2D/01_plano.md` §5). Um binário por crate.
//! Os de placa saltam (com aviso) numa máquina sem GPU; correm com `PH2D_GPU=1`.

mod c0_o_que_fica_fora;
mod c3_grade_fina;
mod cenas;
mod comum;
mod diagnostico;
mod g10_determinismo;
mod g12_queda_wgsl;
mod g3_rc_contra_gabarito;
mod g4_custo_independe_das_luzes;
mod g5_bytes_declarados;
mod g9_rebote;
mod junto_a_face;
mod medicao;
mod medida_luz2d;
mod ultra_hrc;
