//! **`ph2d-fx-cells` — o filtro da camada do Motion em CADA forma** (ciclo 20 W3b,
//! [doc 129](../../../docs/Motion%20Nodes/129_ciclo_20_o_acabamento.md) §3.4).
//!
//! A camada inteira vê a UNIÃO das formas: uma sombra interna, um bevel ou um contorno que leem a
//! vizinhança param na fronteira entre duas formas que se tocam. «Cada forma» é o idioma do
//! Illustrator (um efeito em cada objecto corre por objecto): cada instância desenhada SOZINHA numa
//! célula de um átlas, separadas por valas maiores que o alcance da pilha, a pilha UMA vez sobre o
//! átlas, e cada célula devolvida ao seu lugar na ordem de z.
//!
//! - [`plano`] — puro: as células e a translação inteira de cada uma;
//! - [`Celulas`] — na placa: as caixas no ecrã (UMA leitura pequena por quadro), a relocação, o
//!   desenho, a pilha e a volta.

#![forbid(unsafe_code)]

mod celulas;
mod passes;
pub mod plano;
mod shaders;
mod volta;

pub use celulas::{
    Celulas, Entrada, FORMATO, Formas, Imagens, LADO_MAX, Sprites, Variante, afim_das_sprites,
};
pub use plano::{Caixa, Celula, Pagina, Vala, planeia, planeia_com_par};
