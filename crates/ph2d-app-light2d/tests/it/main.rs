//! Os gates da família da luz 2D (plano `docs/Luz2D/01_plano.md` §5). Um binário por crate; os de
//! placa precisam de `PH2D_GPU=1` (exclusão na placa) e saltam sem adaptador.

mod a_escada_da_sombra;
mod a_face_acesa_da_parede;
mod a_grade_fina;
mod a_luz_nao_muda_com_a_vista;
mod a_nitidez_da_luz;
mod captura_do_vetor;
mod captura_por_texel;
mod censo_wgsl_partilhado;
mod comum;
mod every_word_this_family_shows_comes_from_the_string_table;
mod g11_pan_sem_tremor;
mod g6_ponto_neutro;
mod medida_luz2d_w3;
mod medida_luz2d_w4;
mod medida_luz2d_w4b;
mod medida_luz2d_w5a;
mod o_gizmo_da_luz;
mod o_oclusor_fino;
mod paridade_luz_recebida;
mod smoke_cena_1;
mod smoke_cenas_w4;
mod w4_artista;
