//! ⭐⭐⭐ **O MAPA imagem-px → ecrã que a ARTE DOBRADA manda** — a porta de quem desenha CHROME por
//! cima do canvas do Painter.
//!
//! ⛔⛔ **O ponteiro foi curado em 2026-09-14 e o chrome não** (medido 2026-09-15): a entrega do
//! ponteiro passou a resolver o clique pela malha posada, e o editor de curva continuou a pintar os
//! pontos de controlo pelo afim do **quad de repouso**. ⇒ o artista clicava num sítio e o ponto
//! aparecia noutro, longe da tinta que ele próprio acabara de pousar.
//!
//! ⚠️⚠️ **E a metade que torna isto obrigatório é a do DEDO, não a do olho:** as alças do editor são
//! agarradas pelo `deliver_canvas_pointer`, que hoje resolve pela MALHA. Curar só uma das duas
//! direcções deixa um controlo **desenhado por um mapa e agarrado por outro** — a espécie de
//! controlo morto que o §5.0 diz que nenhuma sonda deste repo apanha. *As duas viajam juntas ou
//! nenhuma viaja.*
//!
//! ⚠️ **O afim FICA, e não é dívida:** ele é a lei de uma sprite que se desenha como QUAD, e carrega
//! a grelha da folha desdobrada que esta porta não conhece. A malha só responde onde ela existe.
//!
//! ⛔ **O que ele NÃO faz: SUBDIVIDIR.** Um ponto atravessa exactamente; um SEGMENTO entre dois
//! pontos é desenhado recto, e sobre uma dobra o recto certo seria partido nas arestas dos
//! triângulos. Para a espinha da curva isso é inofensivo **por construção** — ela já chega
//! achatada em muitos pontos, pela mesma `flatten_spine` que a tinta percorre —, e para a CAIXA do
//! gizmo de transformação é uma aproximação **declarada**: quatro cantos mapeados, arestas rectas.

use ph2d_render::Camera2d;
use ph2d_vector::{Affine, Point};

/// Ver o cabeçalho do módulo.
#[derive(Clone, Copy)]
pub struct CanvasMap<'a> {
    /// A lei do QUAD DE REPOUSO: imagem-px → ecrã (tamanho · escala · rotação · âncora · câmera).
    afim: Affine,
    iw: f64,
    ih: f64,
    /// A malha posada desta sprite, quando ela é desenhada como malha.
    malha: Option<(ph2d_render::DrawnMesh<'a>, Affine)>,
}

impl<'a> CanvasMap<'a> {
    /// O mapa desta sprite. `iw`/`ih` são o tamanho do canvas do Painter em pixels — é neles que os
    /// pontos autorados (as alças da curva) vivem.
    #[must_use]
    pub fn new(
        present: &'a ph2d_ecs::World,
        sim_entity_bits: u64,
        iw: u32,
        ih: u32,
        afim: Affine,
        camera: &Camera2d,
        window: ph2d_host::WindowSize,
    ) -> Self {
        let malha = (iw > 0 && ih > 0)
            .then(|| ph2d_render::drawn_mesh_of(present, sim_entity_bits))
            .flatten()
            .map(|m| (m, camera.world_to_screen_affine(window)));
        Self {
            afim,
            iw: f64::from(iw.max(1)),
            ih: f64::from(ih.max(1)),
            malha,
        }
    }

    /// O mapa de uma sprite **sem malha** — o afim de sempre, e nada mais. Para os chamadores que
    /// ainda não têm o mundo de apresentação à mão.
    #[must_use]
    pub fn rest(afim: Affine) -> Self {
        Self {
            afim,
            iw: 1.0,
            ih: 1.0,
            malha: None,
        }
    }

    /// **Um ponto autorado (imagem-px) no ECRÃ.** Pela malha onde ela o desenha; pelo afim do quad
    /// onde ela não o desenha (fora dela) ou não existe.
    #[must_use]
    pub fn point(&self, p: [f32; 2]) -> Point {
        if let Some((malha, mundo_para_ecra)) = &self.malha {
            let uv = [
                (f64::from(p[0]) / self.iw) as f32,
                (f64::from(p[1]) / self.ih) as f32,
            ];
            if let Some(w) = malha.world_at_uv(uv) {
                return *mundo_para_ecra * Point::new(f64::from(w[0]), f64::from(w[1]));
            }
        }
        self.afim * Point::new(f64::from(p[0]), f64::from(p[1]))
    }

    /// O afim do quad de repouso — para quem precisa de uma ESCALA (o raio de uma alça em px de
    /// imagem) ou de desenhar uma IMAGEM, que não é um ponto e não atravessa esta porta.
    #[must_use]
    pub fn affine(&self) -> Affine {
        self.afim
    }

    /// **O mesmo mapa, deslocado no ECRÃ** — o que o *Repeat Image* precisa: o mesmo desenho
    /// autorado repetido nos ladrilhos vizinhos.
    ///
    /// ⚠️ **O deslocamento entra nas DUAS metades** (o afim do quad *e* a projecção da malha), senão
    /// um ladrilho desenharia as alças da malha por cima do ladrilho central enquanto a linha que as
    /// liga viajava — *meia lei aplicada é pior que nenhuma, porque as duas concordam no centro*.
    #[must_use]
    pub fn deslocado(&self, dx: f64, dy: f64) -> Self {
        let t = Affine::translate((dx, dy));
        Self {
            afim: t * self.afim,
            malha: self.malha.map(|(m, mundo)| (m, t * mundo)),
            ..*self
        }
    }

    /// `true` quando a arte por baixo é desenhada como MALHA — o que separa *«o afim está certo»* de
    /// *«o afim é o que sobra»*.
    #[must_use]
    pub fn is_mesh(&self) -> bool {
        self.malha.is_some()
    }
}
