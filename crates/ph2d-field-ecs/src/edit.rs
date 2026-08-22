//! **O que se edita num nó da cena**, e as perguntas que o painel faz sobre ele.
//!
//! ⚠️ **Este arquivo é um ROTEADOR.** Ele chegou a 1003 linhas e foi cortado na W34 por
//! **responsabilidade**, não por tamanho: as três metades respondem a perguntas diferentes, e cada
//! uma leva a sua lei escrita no topo do próprio arquivo.
//!
//! | módulo | a pergunta que ele responde |
//! |---|---|
//! | [`params`](self::params) | *o que este nó **é** e que números ele tem* — enumerar, ler, escrever |
//! | [`pose`](self::pose) | *onde ele **está*** — e a conversão mundo↔local que o gizmo exige |
//! | [`tree`](self::tree) | *que **forma** a peça tem* — nascer, agrupar, duplicar, apagar |
//!
//! ⚠️ **O re-export é a fronteira da crate, e é explícito de propósito:** um `pub use tree::*`
//! deixaria escapar para a superfície pública qualquer helper privado que um dia deixe de o ser,
//! sem ninguém decidir.

#[path = "edit_params.rs"]
mod params;
#[path = "edit_pose.rs"]
mod pose;
#[path = "edit_tree.rs"]
mod tree;

pub use params::{
    add_mod, dims_of, mods_of, params_of, radius_bound, radius_of, remove_mod, set_dim, set_param,
    walk,
};
pub use pose::{
    rotate_world, rotate_world_about, scale_about, scale_by, top_level, translate_world,
};
pub use tree::{
    add_leaf, add_sampled, can_detach, can_wrap, duplicate, promote_leaf_hosts, remove, set_op,
    set_radius, wrap_in_op,
};
