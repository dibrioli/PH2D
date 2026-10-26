//! As formas GRAVADAS até ao `PROJECT_SCHEMA` 196 com o relógio POR POSIÇÃO — só para o degrau
//! `196 → 197` ([`crate::item_id::migra`], F32 §4): o `Tween` sem `relogio` e o `PathFollow` sem
//! `relogio_item`. ⛔ Congeladas: não acompanhe o tipo vivo.

use serde::Deserialize;

use crate::{ItemId, PathFollow, Tweens};

#[derive(Deserialize)]
pub(crate) struct TweenV196 {
    canal: ph2d_tween::Canal,
    de: [f32; 4],
    para: [f32; 4],
    easing: ph2d_tween::Easing,
    ao_acabar: ph2d_tween::AoAcabar,
    ciclo: ph2d_tween::Ciclo,
    item: ItemId,
}

#[derive(Deserialize)]
pub(crate) struct TweensV196(Vec<TweenV196>);

impl From<TweensV196> for Tweens {
    fn from(v: TweensV196) -> Self {
        Self(
            (v.0.into_iter())
                .map(|t| ph2d_tween::Tween {
                    canal: t.canal,
                    de: t.de,
                    para: t.para,
                    easing: t.easing,
                    ao_acabar: t.ao_acabar,
                    ciclo: t.ciclo,
                    item: t.item,
                    relogio: ItemId::NONE,
                })
                .collect(),
        )
    }
}

#[derive(Deserialize)]
pub(crate) struct PathFollowV196 {
    caminho: String,
    relogio: u8,
    ciclo: ph2d_tween::Ciclo,
    easing: ph2d_tween::Easing,
    ao_acabar: ph2d_tween::AoAcabar,
    deslocamento: f32,
    alinha: bool,
    angulo: f32,
    lado: f32,
}

impl From<PathFollowV196> for PathFollow {
    fn from(v: PathFollowV196) -> Self {
        Self {
            caminho: v.caminho,
            relogio: v.relogio,
            ciclo: v.ciclo,
            easing: v.easing,
            ao_acabar: v.ao_acabar,
            deslocamento: v.deslocamento,
            alinha: v.alinha,
            angulo: v.angulo,
            lado: v.lado,
            relogio_item: ItemId::NONE,
        }
    }
}
