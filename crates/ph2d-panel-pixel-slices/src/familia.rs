//! **A resposta a cada gesto do painel**: a lei sobre o sprite, sem estado da família. A família só
//! grava o op e guarda a seleção ([`Resposta`]).

use ph2d_pixel_core::slice::delete;
use ph2d_pixel_core::{Op, SliceId, Sprite};
use ph2d_pixel_slices::campos::{self, pode_remover};

use crate::canal::{Contexto, Linha, Menu, Pedido};

/// O que a família faz depois de um gesto.
#[derive(Debug, Default)]
pub struct Resposta {
    pub op: Option<Op>,
    /// A seleção nova, se muda.
    pub sel: Option<Vec<SliceId>>,
}

/// O gesto `p` sobre o sprite no quadro `frame`, com a seleção `sel`.
pub fn responde(
    p: &Pedido,
    s: &mut Sprite,
    frame: u32,
    sel: &[SliceId],
    transform: &mut bool,
) -> Resposta {
    let op = |op: Option<Op>| Resposta {
        op,
        ..Resposta::default()
    };
    match p {
        Pedido::Seleciona(id) => Resposta {
            sel: Some(vec![*id]),
            ..Resposta::default()
        },
        Pedido::Todas => Resposta {
            sel: Some(s.slices().iter().map(|c| c.id).collect()),
            ..Resposta::default()
        },
        Pedido::Nenhuma => Resposta {
            sel: Some(Vec::new()),
            ..Resposta::default()
        },
        Pedido::Transform => {
            *transform = !*transform;
            Resposta::default()
        }
        Pedido::Nome(t) => op(campos::nome(s, sel, t)),
        Pedido::Dados(t) => op(campos::dados(s, sel, t)),
        Pedido::Cor(c) => op(campos::cor(s, sel, *c)),
        // O passo é do MUNDO, não do documento: a família o faz com a cena na mão.
        Pedido::PassoFechou | Pedido::AddCollider => Resposta::default(),
        Pedido::Campo(c, t) | Pedido::Arrasta(c, t) => op(campos::edita(s, frame, sel, *c, t)),
        Pedido::Nove(on) => op(campos::nove(s, frame, sel, *on)),
        Pedido::Pivot(on) => op(campos::pivot(s, frame, sel, *on)),
        Pedido::Apaga => Resposta {
            op: delete(s, sel),
            sel: Some(Vec::new()),
        },
        Pedido::Duplica => {
            campos::duplica(s, sel).map_or_else(Resposta::default, |(op, novas)| Resposta {
                op: Some(op),
                sel: Some(novas),
            })
        }
        Pedido::RemoveMudanca => op(campos::remove_mudanca(s, frame, sel)),
        Pedido::UserData(id, e) => op(s
            .user_of(ph2d_pixel_core::Owner::Slice(*id))
            .and_then(|u| ph2d_pixel_core::user::edit::apply_edit(u, e))
            .map(|data| Op::UserData {
                owner: ph2d_pixel_core::Owner::Slice(*id),
                data: Box::new(data),
            })),
    }
}

/// O que o painel mostra do sprite, com a Slice Tool na mão.
#[must_use]
pub fn contexto(s: &Sprite, frame: u32, sel: &[SliceId], transform: bool) -> Contexto {
    let linhas = s
        .slices_by_name()
        .into_iter()
        .map(|c| Linha {
            id: c.id,
            nome: c.name.clone(),
            selecionada: sel.contains(&c.id),
        })
        .collect();
    Contexto {
        com_sprite: true,
        na_mao: true,
        linhas,
        vista: campos::vista(s, frame, sel),
        transform,
        user: match sel {
            [id] => s.slice(*id).map(|c| (*id, c.user.clone())),
            _ => None,
        },
        collider: None,
    }
}

/// O menu sobre uma slice selecionada neste quadro, em `at`, se a célula `cel` cai numa; a linha da
/// mudança só acende onde há mudança (§9.2).
#[must_use]
pub fn menu_em(
    s: &Sprite,
    frame: u32,
    sel: &[SliceId],
    cel: (i32, i32),
    at: (f32, f32),
) -> Option<Menu> {
    ph2d_pixel_slices::interior(s, frame, cel).filter(|id| sel.contains(id))?;
    Some(Menu {
        x: at.0,
        y: at.1,
        remover: pode_remover(s, frame, sel),
    })
}
