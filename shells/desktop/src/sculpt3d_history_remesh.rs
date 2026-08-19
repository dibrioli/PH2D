//! **OS DOIS GESTOS QUE TROCAM A MALHA INTEIRA** — o voxel remesh e a
//! retopologia por campo cruzado.
//!
//! Irmão (`#[path]`) do [`super`], e o corte é de ASSUNTO: lá moram os gestos
//! que editam a malha que existe (o traço, a máscara, a pilha de níveis); aqui
//! os dois que a **substituem**.
//!
//! ⚠️ **Eles são irmãos por natureza, e o ADR-0160 §1 é a tabela que os
//! separa:** o voxel remesh re-amostra um campo (a arrumação destrutiva, os
//! quads seguindo os eixos da grade, uma alça mais fina que o voxel some); a
//! retopologia **preserva a topologia** e alinha a grade às direções principais
//! da forma. Os dois ficam, e a UI os oferece separados — fundi-los num botão só
//! seria a pergunta *"que remesh?"* respondida por omissão.
//!
//! ⚠️ **E as duas entram na história pela MESMA entrada** (`StrokeUndo::Remeshed`,
//! que carrega a malha inteira de antes): um remesh não partilha estrutura
//! nenhuma com o que estava lá — nem a contagem de vértices, nem a
//! correspondência entre eles —, então não há representação mais barata.
//!
//! ⚠️ **O corte foi FORÇADO pelo teto de LOC do shell (HR-18, 600):** o pai
//! chegou a **651**. A cura de um teto é um corte para o IRMÃO, nunca uma
//! allowlist.

use super::{RemeshRefusal, Sculpt3dScene, SculptStroke, StrokeUndo};

impl Sculpt3dScene {
    /// lugar errado.
    /// **RECONSTRÓI a malha por voxelização** — o botão do W7.
    ///
    /// Recusa com a pilha de multires montada, e essa recusa é da MESMA família
    /// da de tapar buraco: um remesh troca a topologia inteira, e todo nível
    /// acima é `subdivide` da base — o detalhe deles passaria a descrever uma
    /// malha que não existe mais. ⚠️ **A alternativa seria achatar a pilha em
    /// silêncio**, e isso é destruir trabalho que o artista autorou sem dizer; o
    /// log nomeia a recusa e o conserto.
    ///
    /// ⚠️ **E ela devolve o MOTIVO, não um `None`.** Havia três causas de recusa
    /// entrando num `Option` só, e o chamador tinha de escolher UMA mensagem
    /// para todas — escolheu a da pilha, então um campo que vazou mandava o
    /// artista *"reverter os níveis"* que ele não tem. Uma recusa que nomeia a
    /// causa errada é pior que uma recusa muda: ela dirige o conserto para o
    pub(in crate::sculpt3d) fn remesh(
        &mut self,
        resolution: u32,
    ) -> Result<ph2d_sdf::RemeshReport, RemeshRefusal> {
        if self.level_count() != 1 {
            return Err(RemeshRefusal::MultiresStack);
        }
        let (out, report) =
            ph2d_sdf::remesh(self.mesh(), resolution).map_err(RemeshRefusal::Engine)?;
        let previous = core::mem::replace(self.mesh_mut().ok_or(RemeshRefusal::EmptyScene)?, out);
        self.record(StrokeUndo::Remeshed(Box::new(previous)));
        // A malha é OUTRA: o traço em voo fala de vértices que não existem mais,
        // e os buffers do device mudaram de tamanho.
        self.stroke = SculptStroke::default();
        self.mesh_rebuilt();
        Ok(report)
    }

    /// **A RETOPOLOGIA por campo cruzado** (ADR-0160) — a grade corre AO LONGO
    /// da forma. Devolve `(vértices, quads, não-quads, ms)`.
    ///
    /// ⚠️ **Irmã do [`Self::remesh`] e NÃO substituta**, e as duas ficam porque
    /// respondem a perguntas diferentes: o voxel remesh re-amostra um campo (a
    /// arrumação destrutiva depois de uma booleana, os quads seguindo os eixos da
    /// grade), esta **preserva a topologia** da entrada e alinha a grade às
    /// direções principais da forma. O ADR-0160 §1 traz a tabela.
    ///
    /// ⚠️ **A mesma recusa de PILHA do irmão**, e pelo mesmo motivo: a saída é
    /// uma malha com outra contagem de vértices, e um nível de multires é uma
    /// subdivisão da base — as duas coisas não coexistem.
    ///
    /// ⚠️ **E ela entra na história pela MESMA entrada** (`StrokeUndo::Remeshed`,
    /// que carrega a malha inteira de antes). Não há representação mais barata: um
    /// remesh não partilha estrutura nenhuma com o que estava lá, nem a contagem
    /// de vértices nem a correspondência entre eles.
    pub(in crate::sculpt3d) fn quad_remesh(
        &mut self,
        edge: f32,
        adaptive: f32,
    ) -> Result<(usize, usize, usize, f64), RemeshRefusal> {
        if self.level_count() != 1 {
            return Err(RemeshRefusal::MultiresStack);
        }
        let t = std::time::Instant::now();
        let mesh = self.mesh();
        let scale = ph2d_quadflow::ScaleField::adaptive(mesh, edge, adaptive);
        let (orient, pos) = ph2d_quadflow::solve_fields(mesh, &scale);
        let q = ph2d_quadflow::extract(mesh, &orient, &pos, &scale).map_err(RemeshRefusal::Quad)?;
        let out = (
            q.mesh.vert_count(),
            q.quads,
            q.non_quads,
            t.elapsed().as_secs_f64() * 1000.0,
        );
        let previous =
            core::mem::replace(self.mesh_mut().ok_or(RemeshRefusal::EmptyScene)?, q.mesh);
        self.record(StrokeUndo::Remeshed(Box::new(previous)));
        // A malha é OUTRA: o traço em voo fala de vértices que não existem mais.
        self.stroke = SculptStroke::default();
        self.mesh_rebuilt();
        Ok(out)
    }
}
