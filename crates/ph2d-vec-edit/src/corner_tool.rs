//! O gesto das ferramentas **Fillet / Chamfer** (`impl PenTool` irmão, teto de LOC).
//!
//! Clicar-e-arrastar sobre uma quina para arredondá-la (arco) ou chanfrá-la (reta). Só o
//! PRESS é próprio (agarra a quina e semeia um `Part::Radius` com o SINAL da ferramenta);
//! move e release reusam o caminho do pen (`on_drag` / `on_release`). É a consolidação da
//! antiga alça de raio do Node + o toggle Chamfer da seção Vertex numa dupla explícita.

use crate::{Grab, Part, PenTool, corner_handle, dist2};
use ph2d_vec_scene::{VecPathId, VecScene};

/// Raio de captura da quina, em PIXELS de tela. Um pouco mais generoso que o do nó (10 px): o
/// alvo aqui é o PONTO da quina e o gesto começa nele. **Uma constante só** — o `on_press_corner`
/// e o `corner_hit_at` têm de concordar, senão o shell congelaria a receita de uma forma viva
/// num clique que o press depois recusa.
const CORNER_HIT_PX: f64 = 12.0;

impl PenTool {
    /// **Há uma quina agarrável sob `p`?** — a MESMA busca do [`Self::on_press_corner`], sem
    /// efeito colateral.
    ///
    /// O shell a consulta para decidir se congela a receita de uma forma VIVA antes do gesto:
    /// congelar num clique que ERRA a quina expandiria a forma sem o artista pedir.
    #[must_use]
    pub fn corner_hit_at(&self, scene: &VecScene, p: [f64; 2], px_to_world: f64) -> bool {
        self.selected
            .and_then(|id| self.nearest_anchor_on(scene, id, p, CORNER_HIT_PX * px_to_world))
            .is_some()
    }

    /// Pressão primária nas ferramentas **Fillet / Chamfer**: agarra a QUINA do path
    /// selecionado sob o cursor e arma o arrasto de raio — o dedo dita a MAGNITUDE, a
    /// ferramenta dita o ESTILO (`chamfer`). É a mesma máquina do arrasto da alça de raio
    /// (`Part::Radius`), só semeada pela ÂNCORA em vez de uma bolinha na bissetriz; move e
    /// release reusam o caminho do pen (`on_drag` / `on_release`) sem mudança.
    ///
    /// **A metade "avançada" que o Enio pediu:** se a âncora sob o cursor não é uma quina
    /// arredondável (um ponto `Smooth` tem os handles colineares — não há ângulo), ela é
    /// primeiro transformada em quina afiada ([`ph2d_vec_scene::make_sharp_corner`]). Quem
    /// clica um ponto suave quer arredondá-lo, e para isso ele precisa virar quina antes.
    ///
    /// Opera só no path SELECIONADO: o shell (re)seleciona o path sob o cursor antes de
    /// chamar e barra as formas VIVAS (cujo `corner_radius` o recook varreria). Devolve
    /// `true` se agarrou uma quina (o arrasto está armado, `is_dragging` passa a valer).
    pub fn on_press_corner(
        &mut self,
        scene: &mut VecScene,
        p: [f64; 2],
        px_to_world: f64,
        chamfer: bool,
    ) -> bool {
        let hit_r = CORNER_HIT_PX * px_to_world;
        let Some(id) = self.selected else {
            return false;
        };
        let Some(vert) = self.nearest_anchor_on(scene, id, p, hit_r) else {
            return false;
        };
        self.selected_paths = vec![id];
        self.selected_verts = vec![vert];
        // "Primeiro transforma em quina" quando o ponto é suave (sem ângulo a arredondar).
        let is_corner = scene
            .paths()
            .iter()
            .find(|pp| pp.id == id)
            .and_then(|path| corner_handle::frame_at_flat(path, vert))
            .is_some();
        if !is_corner && let Some(path) = scene.path_mut(id) {
            let _ = ph2d_vec_scene::make_sharp_corner(path, vert);
        }
        // O frame DEPOIS da conversão. Se ainda não há quina (vizinhos colineares), desiste.
        let Some(frame) = scene
            .paths()
            .iter()
            .find(|pp| pp.id == id)
            .and_then(|path| corner_handle::frame_at_flat(path, vert))
        else {
            return false;
        };
        // O offset que torna o arrasto RELATIVO (ver `Grab::radius_offset`), medido AGORA.
        let pl = self.to_local(id, p);
        let d = [pl[0] - frame.anchor[0], pl[1] - frame.anchor[1]];
        let proj = d[0] * frame.bisector[0] + d[1] * frame.bisector[1];
        self.grab = Some(Grab {
            path: id,
            vert,
            part: Part::Radius,
            radius_offset: frame.setback - proj,
            chamfer: Some(chamfer),
        });
        true
    }

    /// A ÂNCORA do path `id` mais próxima de `p` (mundo), dentro de `hit_r` (mundo). `None`
    /// se o path não é agarrável ou nenhuma âncora está perto. Diferente do `hit_test`: só
    /// ÂNCORA (as ferramentas de quina nunca agarram um handle Bézier) e só o path dado (o
    /// shell já escolheu o alvo pelo `path_at`).
    fn nearest_anchor_on(
        &self,
        scene: &VecScene,
        id: VecPathId,
        p: [f64; 2],
        hit_r: f64,
    ) -> Option<usize> {
        if !self.view.is_pickable(id) {
            return None;
        }
        let path = scene.paths().iter().find(|pp| pp.id == id)?;
        let xf = self.xf(id);
        let r2 = hit_r * hit_r;
        let mut best: Option<(usize, f64)> = None;
        for (i, v) in path.verts_all().enumerate() {
            let d2 = dist2(p, xf.apply(v.anchor));
            if d2 <= r2 && best.is_none_or(|(_, b)| d2 < b) {
                best = Some((i, d2));
            }
        }
        best.map(|(i, _)| i)
    }
}
