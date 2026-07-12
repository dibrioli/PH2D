//! **O gesto do conector** (`DrawMode::Connect`): pressiona numa forma, arrasta, solta noutra.
//!
//! - Down **sobre uma forma** → a ponta nasce presa a ela. Down no vazio → a ponta nasce
//!   solta, ali.
//! - Up **sobre outra forma** → a linha liga as duas, e as segue para sempre.
//! - Up **no vazio** → a ponta fica solta naquele ponto (o conector meio-ligado do draw.io:
//!   arrasta-se a ponta depois, para prendê-la).
//! - Down e Up **na MESMA forma** → um laço (a forma apontando para si mesma).
//!
//! # O preview é o conector, não um desenho dele
//!
//! O path é empurrado na cena já no **Down**, e o componente [`VecConnector`] é pendurado na
//! entidade no mesmo frame. A partir daí quem desenha a linha é o re-cook de todo frame
//! ([`crate::connector_live`]) — a MESMA chamada de `route` que desenhará o conector final.
//! Um preview desenhado à parte seria um segundo caminho de código, com um segundo jeito de
//! errar: o que se vê durante o arrasto **é** o que se obtém ao soltar, por construção.
//!
//! O alvo sob o cursor entra como `Bound` já **durante** o arrasto, e é por isso que a linha
//! "gruda" visivelmente na forma quando se passa por cima dela — o feedback é o resultado.

use ph2d_ecs::{Anchor, ConnectorEnd, Entity, VecConnector};
use ph2d_vec_scene::{Marker, VecPathId};

use crate::app_state::App;

/// O conector em construção (Down..Up). O `path` já existe na cena; o `conn` é a relação que
/// o `connector_live` re-cozinha.
#[derive(Clone, Debug)]
pub(crate) struct ConnectorDrag {
    pub(crate) path: VecPathId,
    pub(crate) conn: VecConnector,
    /// Onde o gesto começou (mundo) — para descartar o clique-sem-arrasto que não prendeu
    /// nada (uma linha de comprimento zero solta no vazio não é um objeto, é um acidente).
    start_world: [f64; 2],
}

/// Distância mínima (world) que o cursor precisa percorrer para um Up **no vazio** valer um
/// conector. Abaixo disso é um clique perdido, e o gesto é desfeito.
const MIN_DRAG: f64 = 0.02;

impl App {
    /// O ponto de MUNDO sob um ponto de tela. `None` antes do primeiro frame (sem `gfx`).
    pub(crate) fn vec_world_at(&self, screen: (f32, f32)) -> Option<[f64; 2]> {
        let gfx = self.gfx.as_ref()?;
        let w = gfx.camera.screen_to_world(screen, gfx.surface.size());
        Some([f64::from(w[0]), f64::from(w[1])])
    }

    /// A forma sob o cursor — **ignorando os conectores** (inclusive o que está sendo
    /// arrastado, cuja ponta está exatamente sob o cursor e roubaria todo hit-test).
    ///
    /// Ligar um conector a outro conector é possível no draw.io; aqui não é: o alvo de uma
    /// ponta é uma FORMA. (Nada no motor impede — é uma decisão de gesto, e reverter é tirar
    /// o filtro.)
    pub(crate) fn shape_under_cursor(&self, world: [f64; 2]) -> Option<VecPathId> {
        let gfx = self.gfx.as_ref()?;
        let window_size = gfx.surface.size();
        let view = crate::vec_entities::view_state(&gfx.sim, &self.vec_entities);
        let hits = crate::vec_gizmo_view::pick_all_at_world(
            &gfx.sim,
            &gfx.vec_scene,
            &view,
            &self.vec_entities,
            [world[0] as f32, world[1] as f32],
            crate::vec_gizmo_view::stroke_hit_r(&gfx.camera, window_size),
        );
        // `pick_all_at_world` devolve do TOPO para o fundo: a primeira forma que não é um
        // conector é a que o olho vê sob o cursor.
        hits.into_iter()
            .filter(|&bits| {
                gfx.sim
                    .world()
                    .get::<VecConnector>(Entity::from_bits(bits))
                    .is_none()
            })
            .find_map(|bits| {
                self.vec_entities
                    .iter()
                    .find(|&(_, &b)| b == bits)
                    .map(|(&id, _)| id)
            })
    }

    /// A ponta que um ponto do canvas produz: presa à forma sob ele, ou solta ali.
    fn end_at(&self, world: [f64; 2]) -> ConnectorEnd {
        match self.shape_under_cursor(world) {
            Some(target) => ConnectorEnd::Bound {
                target,
                anchor: Anchor::Floating,
            },
            None => ConnectorEnd::Free { at: world },
        }
    }

    /// **Down.** Abre o gesto: empurra a linha na cena (com o Style da tool + a **ponta de
    /// seta** no fim) e guarda a relação. `false` se não há cena (nada a fazer).
    ///
    /// A seta não é enfeite: um conector sem ponta é só uma linha, e o diagrama perde a
    /// direção. É `marker_end` por default — e continua sendo Style (o painel a troca).
    pub(crate) fn connector_down(&mut self, world: [f64; 2]) -> bool {
        let px_to_world = self.vec_px_to_world();
        let style = self.vec_pen.style();
        let start = self.end_at(world);
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        // Snapshot pré-gesto (o undo global também cobre, mas o histórico vetorial é o que o
        // resto do módulo usa — mantém a simetria com pen/shape).
        self.vec_history.begin(&gfx.vec_scene);
        let mut path = ph2d_vec_scene::line(world, world);
        path.stroke = Some(ph2d_vec_scene::StrokeSpec {
            color: style.stroke,
            width: style.stroke_w_px * px_to_world,
            cap: style.cap,
            join: style.join,
            dash: style.dash,
            marker_start: style.marker_start,
            // A ponta de seta é o que faz a linha ser um conector, e não um traço.
            marker_end: Marker::Triangle,
            marker_scale: style.marker_scale,
            marker_round: style.marker_round,
        });
        let id = gfx.vec_scene.push_path(path);
        self.vec_connect = Some(ConnectorDrag {
            path: id,
            conn: VecConnector {
                start,
                end: ConnectorEnd::Free { at: world },
                route: ph2d_ecs::RouteKind::default(),
                parallel_index: 0,
                // Nascem AUTOMÁTICOS: o jetty acompanha o tamanho das caixas e o spread, o
                // índice do feixe. Só o painel os fixa.
                jetty: None,
                spread: None,
                corner_radius: 0.0,
                waypoints: Vec::new(),
            },
            start_world: world,
        });
        true
    }

    /// **Move.** A ponta livre segue o cursor — e **gruda** na forma sob ele (a linha já se
    /// encosta no contorno dela, porque é o re-cook de verdade que a desenha). `false` se não
    /// há gesto (o chamador segue o caminho normal do Move).
    pub(crate) fn connector_drag_move(&mut self, x: f32, y: f32) -> bool {
        if self.vec_connect.is_none() {
            return false;
        }
        let Some(w) = self
            .gfx
            .as_ref()
            .map(|gfx| gfx.camera.screen_to_world((x, y), gfx.surface.size()))
        else {
            return false;
        };
        let world = [f64::from(w[0]), f64::from(w[1])];
        let end = self.end_at(world);
        if let Some(drag) = self.vec_connect.as_mut() {
            drag.conn.end = end;
        }
        true
    }

    /// **Up.** Fecha o gesto. Solta no vazio sem ter arrastado nada e sem ter prendido nada ⇒
    /// era um clique perdido: o path some (e o passo de undo é cancelado).
    ///
    /// `true` se o Up foi consumido (havia gesto).
    pub(crate) fn connector_up(&mut self, world: [f64; 2]) -> bool {
        let Some(mut drag) = self.vec_connect.take() else {
            return false;
        };
        drag.conn.end = self.end_at(world);
        let bound = VecConnector::target_of(&drag.conn.start).is_some()
            || VecConnector::target_of(&drag.conn.end).is_some();
        let moved = (world[0] - drag.start_world[0]).hypot(world[1] - drag.start_world[1]);
        if !bound && moved < MIN_DRAG {
            if let Some(gfx) = self.gfx.as_mut() {
                gfx.vec_scene.remove_path(drag.path);
            }
            self.vec_history.cancel();
            return true;
        }
        // Dois conectores no MESMO par de formas não podem se sobrepor — o segundo sumiria
        // por baixo do primeiro, e o usuário juraria que ele não foi criado.
        drag.conn.parallel_index = self.next_parallel_index(&drag.conn, drag.path);
        if let Some(gfx) = self.gfx.as_mut() {
            // O componente é (re)escrito no frame do render, quando a entidade já existe.
            // Aqui só resta selecionar a linha nova — como a shape-tool faz.
            self.vec_pen.select(Some(drag.path));
            self.vec_history.commit_if_changed(&gfx.vec_scene);
        }
        self.vec_connect_pending = Some((drag.path, drag.conn));
        true
    }

    /// **Cancela** o gesto (botão direito / Esc): a linha em construção some.
    pub(crate) fn connector_cancel(&mut self) -> bool {
        let Some(drag) = self.vec_connect.take() else {
            return false;
        };
        if let Some(gfx) = self.gfx.as_mut() {
            gfx.vec_scene.remove_path(drag.path);
        }
        self.vec_history.cancel();
        true
    }

    /// Quantos conectores JÁ ligam este mesmo par de formas (em qualquer ordem) — o índice do
    /// novo. `0` = o único.
    ///
    /// **`me` é o path do próprio conector, e excluí-lo não é higiene: é correção.** O preview
    /// do arrasto É o conector de verdade — o componente dele já está pendurado na entidade,
    /// com este mesmo par, desde o primeiro frame do gesto. Sem esta exclusão ele contaria a si
    /// mesmo, e o PRIMEIRO conector entre duas formas já nasceria deslocado como se fosse o
    /// segundo.
    fn next_parallel_index(&self, conn: &VecConnector, me: VecPathId) -> u8 {
        let (Some(gfx), Some(a)) = (self.gfx.as_ref(), VecConnector::target_of(&conn.start)) else {
            return 0;
        };
        let Some(b) = VecConnector::target_of(&conn.end) else {
            return 0;
        };
        let pair = |c: &VecConnector| {
            let (x, y) = (
                VecConnector::target_of(&c.start),
                VecConnector::target_of(&c.end),
            );
            (x == Some(a) && y == Some(b)) || (x == Some(b) && y == Some(a))
        };
        let n = self
            .vec_entities
            .iter()
            .filter(|&(&id, _)| id != me)
            .filter_map(|(_, &bits)| gfx.sim.world().get::<VecConnector>(Entity::from_bits(bits)))
            .filter(|c| pair(c))
            .count();
        u8::try_from(n).unwrap_or(u8::MAX)
    }
}
