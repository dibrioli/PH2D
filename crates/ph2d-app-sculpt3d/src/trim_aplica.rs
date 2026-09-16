//! **O CORTE, aplicado à peça** — a costura entre o gesto, a lei e o motor.
//!
//! Filho (`#[path]`) de [`super`]. O que é do gesto vive em
//! [`super::trim_gesto`], a lei na [`ph2d_trim`], e o motor atrás da
//! [`ph2d_mesh_bool`]. Aqui fica o que precisa da CENA: a câmara que faz os
//! raios, a peça, o desfazer e a fala das recusas.

use super::trim_gesto::{Gesto, Orientacao};
use super::{Sculpt3dScene, SculptStroke, StrokeUndo};

/// Porque é que o corte não aconteceu — **em voz alta, sempre**.
///
/// ⚠️ **Três origens, e é de propósito que elas não se fundem num `Option`:** a
/// recusa tem de dirigir o conserto, e este módulo já viu o preço de não o fazer
/// (o cabeçalho do [`super::history_remesh`] regista-o: *uma recusa que nomeia a
/// causa errada é pior que uma recusa muda*).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum TrimRecusa {
    /// A cena não tem peça.
    SemPeca,
    /// Há uma pilha de multiresolução montada.
    ///
    /// ⚠️ **A mesma recusa do remesh, e pelo mesmo motivo:** a saída é uma malha
    /// com outra contagem de vértices, e um nível de multires é uma subdivisão
    /// da base — as duas coisas não coexistem.
    PilhaMontada,
    /// O gesto não delimitou área, ou a lei não fez volume dele.
    Gesto(ph2d_trim::Recusa),
    /// O motor recusou — a peça aberta, a lâmina aberta, ou o corte que apaga
    /// tudo.
    Corte(ph2d_mesh_bool::Recusa),
}

impl TrimRecusa {
    /// A frase que o artista lê.
    pub(crate) fn porque(self) -> &'static str {
        match self {
            Self::SemPeca => "nao ha' peca nenhuma para cortar",
            Self::PilhaMontada => {
                "ha' uma pilha de multiresolucao montada -- J reverte-a e o corte volta"
            }
            Self::Gesto(r) => r.porque(),
            Self::Corte(r) => r.porque(),
        }
    }
}

impl Sculpt3dScene {
    /// **Corta a peça com o volume que o gesto desenhou.**
    ///
    /// ⚠️ **A ordem das recusas é da mais específica para a mais geral**, que é
    /// a lei que a família de recusas desta linha já aplica: *dizer «falta uma
    /// peça» a quem também tem a pilha montada é mandá-lo resolver a metade
    /// errada.*
    pub(crate) fn trim_aplica(
        &mut self,
        gesto: &Gesto,
        orientacao: Orientacao,
    ) -> Result<(), TrimRecusa> {
        if self.obj().is_none() {
            return Err(TrimRecusa::SemPeca);
        }
        if self.level_count() != 1 {
            return Err(TrimRecusa::PilhaMontada);
        }

        let anel = gesto.anel(
            self.brush.trim_suavizacao,
            self.passo_do_corte_px(gesto.inicio()),
        );
        if anel.len() < 3 {
            return Err(TrimRecusa::Gesto(ph2d_trim::Recusa::GestoDegenerado));
        }
        let raios: Vec<ph2d_mesh::Ray> = anel.iter().map(|p| self.ray_at(p[0], p[1])).collect();

        // ⚠️ **A origem SEM acerto é o centro da peça, e a escolha é
        // INOBSERVÁVEL** no regime de omissão: a faixa da §6.1 é medida em
        // distâncias COM SINAL a partir dessa origem, e o prisma é pousado de
        // volta a partir dela — deslocar a origem ao longo do eixo desloca os
        // dois na mesma medida e cancela. Há gate na `ph2d-trim` a afirmá-lo.
        let centro = {
            let b = self.mesh().bounds();
            [
                (b.min[0] + b.max[0]) * 0.5,
                (b.min[1] + b.max[1]) * 0.5,
                (b.min[2] + b.max[2]) * 0.5,
            ]
        };
        let olhar = self.camera.view_axis();
        let (plano, coagida) = gesto.plano(orientacao, [-olhar.x, -olhar.y, -olhar.z], centro);
        // ⭐⭐ **O alvo corrige isto em SILÊNCIO; nós DIZEMO-LO** (espec §3 **N**).
        // *Um knob que muda de valor sem avisar é a espécie de controlo que
        // mente* — e esta é a única correcção calada de um parâmetro autorado em
        // toda a família dele.
        if coagida {
            eprintln!(
                "[sculpt3d] o corte comecou FORA da peca: sem superficie nao ha' \
                 normal, entao a orientacao passou a ser a da VISTA"
            );
        }

        let prisma = ph2d_trim::prisma(
            &anel,
            &raios,
            &plano,
            self.mesh(),
            ph2d_trim::Profundidade::DaPeca,
            ph2d_trim::Paredes::Fixas,
            // ⭐⭐⭐ **A densidade da LÂMINA é a densidade da PEÇA** — ver
            // [`alvo_da_lamina`]. É esta linha que responde ao report do dono.
            ph2d_trim::Resolucao::Ate(alvo_da_lamina(self.mesh())),
        )
        .map_err(TrimRecusa::Gesto)?;

        let cortada = ph2d_mesh_bool::corta(self.mesh(), &prisma, ph2d_mesh_bool::Op::Subtrair)
            .map_err(TrimRecusa::Corte)?;

        let previous = core::mem::replace(self.mesh_mut().ok_or(TrimRecusa::SemPeca)?, cortada);
        // ⚠️ **A MESMA entrada do remesh** (`StrokeUndo::Remeshed`, a troca
        // simétrica): um corte muda a contagem de vértices, e a janela por
        // vértice do traço não descreve isso — os índices de depois não
        // descrevem os de antes.
        self.record(StrokeUndo::Remeshed(Box::new(previous)));
        // A malha é OUTRA: o traço em voo fala de vértices que já não existem, e
        // os buffers do device mudaram de tamanho.
        self.stroke = SculptStroke::default();
        self.mesh_rebuilt();
        Ok(())
    }
}

/// ⭐⭐⭐ **A aresta que a peça TEM** — o alvo com que a lâmina é tesselada.
///
/// # Porque ela existe (report do dono, 2026-09-15, com foto)
///
/// *«o remesh da face que você cortou fica ruim demais»*. A face que um corte
/// deixa é a **parede do prisma** recortada pela peça, logo ela herda a
/// tesselação da lâmina — e uma lâmina de duas faces deixava a face cortada com
/// **dois triângulos**, `1 385 ×` mais grossa que a peça à volta dela (medido
/// numa peça de `10 000` triângulos). Uma face plana com dois triângulos tem os
/// cantos a partilhar a normal da esfera ⇒ ela é **sombreada como se fosse
/// curva**, que é o aspecto derretido da foto, e não há onde esculpir a seguir.
///
/// ⭐ **A régua é a que a casa já tem** ([`ph2d_mesh::edge_for_tri_count`], a
/// mesma do alvo de topologia): o lado de um triângulo equilátero que ladrilha
/// esta **ÁREA** com esta **CONTAGEM**. ⚠️ E ela é ancorada na área de
/// propósito — *a densidade não é propriedade da vista nem da caixa*, que é a
/// lei que este módulo já pagou duas vezes (o `Quad Size` absoluto do botão de
/// retopologia, e o alvo de topologia que variava `4,9 ×` com o zoom).
///
/// ⚠️ **Medida contra a mediana das arestas nas fixturas da casa, ela concorda
/// a `1,03 ×`–`1,10 ×`** — e custa uma passagem sem alocação nenhuma, contra
/// uma ordenação de `3 × T` números.
pub(crate) fn alvo_da_lamina(peca: &ph2d_mesh::Mesh) -> f32 {
    // ⚠️ **Triângulos, não faces:** um quad conta DOIS, e ler `face_count` daria
    // um alvo `√2 ×` grosso numa peça que ainda não foi triangulada.
    let tris: usize = peca
        .faces()
        .iter()
        .map(|f| f.verts().len().saturating_sub(2))
        .sum();
    ph2d_mesh::edge_for_tri_count(peca.surface_area(), tris as f32)
}

#[cfg(test)]
#[path = "trim_aplica_tests.rs"]
mod tests;
