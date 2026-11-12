//! ⭐ **A «outra mão» POR CAMPO num condutor persistente** (sessão «Animating», doc 17 item 9).
//!
//! Com `held`, a escrita sem chave num campo que o motor CONDUZ some no Done (o motor reescreve-o);
//! num campo que ele não conduz (a rotação de um objecto cuja curva é só `x`) é autoria e fica.

use crate::Driven;
use ph2d_ecs::Transform;

impl Driven {
    /// O autorado depois de outra mão ter escrito `before` sobre o `last` do motor, que então
    /// deixou `after`: adopta cada campo que a mão mudou (`before ≠ last`) e o motor não reescreveu
    /// (`after = before`). Um facto sem campos (um número, um `bool`) é um campo só.
    #[must_use]
    pub(crate) fn adopta_por_campo(self, last: Self, before: Self, after: Self) -> Self {
        if let (Some(a), Some(l), Some(b), Some(d)) =
            (self.pose(), last.pose(), before.pose(), after.pose())
        {
            return self.com_pose(campos(a, l, b, d));
        }
        if before != last && after == before {
            before
        } else {
            self
        }
    }

    /// A mesma variante de pose com outro `Transform`.
    fn com_pose(self, t: Transform) -> Self {
        match self {
            Self::SolverPose(_) => Self::SolverPose(t),
            Self::StagePose(_) => Self::StagePose(t),
            Self::ScriptPose(_) => Self::ScriptPose(t),
            Self::CanvasPose(_) => Self::CanvasPose(t),
            Self::ParallaxPose(_) => Self::ParallaxPose(t),
            Self::TweenPose(_) => Self::TweenPose(t),
            Self::ConstraintPose(_) => Self::ConstraintPose(t),
            Self::ColaPose(_) => Self::ColaPose(t),
            other => other,
        }
    }
}

fn campos(mut a: Transform, l: Transform, b: Transform, d: Transform) -> Transform {
    let adopta = |a: &mut f32, l: f32, b: f32, d: f32| {
        if b != l && d == b {
            *a = b;
        }
    };
    adopta(
        &mut a.translation.x,
        l.translation.x,
        b.translation.x,
        d.translation.x,
    );
    adopta(
        &mut a.translation.y,
        l.translation.y,
        b.translation.y,
        d.translation.y,
    );
    adopta(&mut a.rotation, l.rotation, b.rotation, d.rotation);
    adopta(&mut a.scale.x, l.scale.x, b.scale.x, d.scale.x);
    adopta(&mut a.scale.y, l.scale.y, b.scale.y, d.scale.y);
    adopta(&mut a.skew_x, l.skew_x, b.skew_x, d.skew_x);
    adopta(&mut a.skew_y, l.skew_y, b.skew_y, d.skew_y);
    a
}
