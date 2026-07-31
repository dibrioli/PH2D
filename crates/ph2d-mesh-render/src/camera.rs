//! A câmera **orbital** — a única forma de olhar uma escultura.
//!
//! ⚠️ **Ela mora aqui e o GESTO mora no shell.** Este tipo é estado + matrizes,
//! sem saber o que é um ponteiro; quem traduz *arrastar* em `orbit` é a shell,
//! nunca uma `Tool` (ADR-0145: o contrato congelado não é tocado, e navegar não
//! é esculpir — o artista gira o modelo com a ferramenta de pincel na mão).
//!
//! ⚠️ **Órbita, não voo livre.** Uma câmera 3D genérica tem 6 graus de liberdade
//! e um artista de escultura usa três: girar em torno do objeto, aproximar e
//! deslocar o centro. Os outros três são maneiras de se perder. É o que
//! ZBrush, Nomad e o SculptGL entregam, e o que este tipo torna *inexprimível*
//! em vez de meramente desencorajado.

use glam::{Mat4, Vec3};

/// Quão perto do polo a órbita pode chegar. Exatamente no polo a direção da
/// vista fica paralela ao `up` e a `look_at` **degenera** (produz `NaN`); esta
/// margem torna o caso impossível em vez de tratado.
const PITCH_LIMIT: f32 = core::f32::consts::FRAC_PI_2 - 0.01;

/// Fração do enquadramento que sobra de folga em volta do modelo.
const FRAME_MARGIN: f32 = 1.15;

/// A câmera que olha uma escultura.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera3d {
    /// O ponto em torno do qual se gira.
    pub target: Vec3,
    /// Distância do olho ao alvo. Sempre > 0.
    pub distance: f32,
    /// Azimute, em radianos, em torno do `+Y` do mundo.
    pub yaw: f32,
    /// Elevação, em radianos. Clampada longe dos polos.
    pub pitch: f32,
    /// Campo de visão VERTICAL, em radianos.
    pub fov_y: f32,
}

impl Default for Camera3d {
    fn default() -> Self {
        Self {
            target: Vec3::ZERO,
            distance: 3.0,
            yaw: 0.5,
            pitch: 0.4,
            fov_y: core::f32::consts::FRAC_PI_4,
        }
    }
}

impl Camera3d {
    /// A câmera que enquadra `bounds` inteiro, com folga.
    #[must_use]
    pub fn framing(bounds: ph2d_mesh::Aabb, fov_y: f32, aspect: f32) -> Self {
        let mut cam = Self {
            fov_y,
            ..Self::default()
        };
        cam.frame(bounds, aspect);
        cam
    }

    /// Reenquadra para `bounds` preservando os ângulos — o "F" de *fit*.
    ///
    /// ⚠️ **Enquadra a CAIXA vista deste ângulo, não a esfera que a contém.** A
    /// primeira versão usava a esfera envolvente (`d ≥ r/sin(meio-fov)`), que é
    /// orientação-invariante e por isso tentadora — e **desperdiça `√3`**: para
    /// uma esfera de raio 1 a caixa é `[-1,1]³`, cuja meia-diagonal é 1,73, e o
    /// modelo saía ocupando **16,9% da tela** (medido pelo gate de GPU). A
    /// invariância só valeria a pena se o enquadramento rodasse todo frame; ele
    /// roda quando o artista aperta *fit*, uma vez.
    ///
    /// A conta exata: com o olho a `d` na direção `v`, uma quina `p` (relativa
    /// ao alvo) fica a profundidade `d − p·v` e a `p·right` de lado, então cabe
    /// quando `d ≥ p·v + |p·right| / tan(meio-fov_h)` — e o mesmo para a
    /// vertical. O `d` é o **máximo sobre as oito quinas e os dois eixos**.
    pub fn frame(&mut self, bounds: ph2d_mesh::Aabb, aspect: f32) {
        if bounds.is_empty() {
            return;
        }
        self.target = Vec3::from(bounds.center());

        let half_v = (self.fov_y * 0.5).max(0.01);
        let tan_v = half_v.tan();
        let tan_h = (tan_v * aspect.max(0.01)).max(1e-3);

        // Os eixos da câmera em mundo. `eye()` já depende de `distance`, mas só
        // a DIREÇÃO importa aqui, e ela é função de yaw/pitch apenas.
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        let v = Vec3::new(cp * sy, sp, cp * cy); // alvo → olho
        let right = Vec3::Y.cross(v).normalize_or(Vec3::X);
        let up = v.cross(right);

        let half = (Vec3::from(bounds.max) - Vec3::from(bounds.min)) * 0.5;
        let mut d: f32 = 0.0;
        for i in 0..8 {
            let p = Vec3::new(
                if i & 1 == 0 { -half.x } else { half.x },
                if i & 2 == 0 { -half.y } else { half.y },
                if i & 4 == 0 { -half.z } else { half.z },
            );
            let depth = p.dot(v);
            d = d
                .max(depth + p.dot(right).abs() / tan_h)
                .max(depth + p.dot(up).abs() / tan_v);
        }
        self.distance = (d * FRAME_MARGIN).max(1e-3);
    }

    /// Onde o olho está.
    #[must_use]
    pub fn eye(&self) -> Vec3 {
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        self.target + Vec3::new(cp * sy, sp, cp * cy) * self.distance
    }

    /// A matriz de vista (mundo → olho).
    #[must_use]
    pub fn view(&self) -> Mat4 {
        Mat4::look_at_rh(self.eye(), self.target, Vec3::Y)
    }

    /// A matriz mundo → clip.
    ///
    /// `perspective_rh` e não `perspective_rh_gl`: o clip do wgpu tem `z` em
    /// `[0,1]`, e a variante GL (`[-1,1]`) faria metade da profundidade cair
    /// atrás do plano near — o modelo apareceria cortado ao meio.
    #[must_use]
    pub fn view_proj(&self, aspect: f32) -> Mat4 {
        let (near, far) = self.clip_planes();
        Mat4::perspective_rh(self.fov_y, aspect.max(0.01), near, far) * self.view()
    }

    /// Os planos near/far, **derivados da distância** em vez de constantes.
    ///
    /// Um `near` fixo de 0,1 m destrói a precisão de profundidade ao aproximar
    /// de um modelo pequeno, e um `far` fixo corta um modelo grande. Ancorando
    /// os dois na distância do olho, a razão `far/near` fica constante — que é
    /// a grandeza de que a precisão do depth-buffer de fato depende.
    #[must_use]
    pub fn clip_planes(&self) -> (f32, f32) {
        let near = (self.distance * 0.01).max(1e-4);
        (near, self.distance * 100.0)
    }

    /// Gira. `dx`/`dy` em radianos — a shell decide quantos radianos vale um
    /// pixel, porque é ela que conhece a densidade da tela.
    pub fn orbit(&mut self, dx: f32, dy: f32) {
        self.yaw += dx;
        self.pitch = (self.pitch + dy).clamp(-PITCH_LIMIT, PITCH_LIMIT);
    }

    /// Aproxima/afasta. `steps` positivo aproxima.
    ///
    /// **Multiplicativo, nunca aditivo:** um passo aditivo dá saltos enormes
    /// longe e não chega perto; multiplicativo, o gesto tem o mesmo efeito
    /// *aparente* em qualquer distância, que é o que a mão espera.
    pub fn dolly(&mut self, steps: f32) {
        const PER_STEP: f32 = 1.1;
        self.distance = (self.distance / PER_STEP.powf(steps)).clamp(1e-3, 1e6);
    }

    /// Desloca o alvo no plano da tela. `dx`/`dy` em FRAÇÃO da altura da
    /// viewport, para que arrastar o mesmo tanto de tela mova o mesmo tanto de
    /// modelo em qualquer zoom.
    pub fn pan(&mut self, dx: f32, dy: f32) {
        let view = self.view();
        // As linhas de uma matriz de vista ortonormal são os eixos da câmera
        // em coordenadas de mundo.
        let right = Vec3::new(view.x_axis.x, view.y_axis.x, view.z_axis.x);
        let up = Vec3::new(view.x_axis.y, view.y_axis.y, view.z_axis.y);
        // A altura do mundo que a viewport cobre à distância do alvo.
        let world_per_frac = 2.0 * self.distance * (self.fov_y * 0.5).tan();
        self.target += (-right * dx + up * dy) * world_per_frac;
    }
}

#[cfg(test)]
#[path = "camera_tests.rs"]
mod tests;
