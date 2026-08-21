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

/// **O QUE A RETOPOLOGIA FEZ** — o relatório que o log e o smoke lêem.
///
/// ⚠️ **O `edge` viaja no relatório, e não só o `detail` que o artista pediu.**
/// O knob é uma fração do curso, e o lado do quad sai da MALHA — então sem este
/// campo ninguém consegue relacionar o que se pediu com o que saiu, nem numa
/// sessão de smoke nem num bug daqui a um mês. *Um número que só existe dentro
/// da função não é comparável entre duas corridas.*
#[derive(Clone, Copy, Debug)]
pub(in crate::sculpt3d) struct QuadRemeshReport {
    /// Quantos vértices a malha nova tem.
    pub verts: usize,
    /// Quantas faces saíram com quatro lados.
    pub quads: usize,
    /// Quantas não saíram.
    pub non_quads: usize,
    /// O lado do quad que o `detail` pedido virou nesta malha.
    pub edge: f32,
    /// O relógio do passe.
    pub ms: f64,
    /// **Quantos buracos ficaram na casca** — `0` é o que se espera.
    pub holes: usize,
    /// ⭐ **Quantos vértices IRREGULARES** (valência ≠ 4) — a grandeza que o pivô
    /// do ADR-0161 existiu para derrubar, e a que o artista de facto vê.
    ///
    /// ⚠️ **É a CONTAGEM e não uma percentagem**, e a diferença é medida: a mesma
    /// peça com o dobro da densidade tem os mesmos irregulares e metade da
    /// percentagem. Uma esfera admite **oito**; o motor local entregava milhares.
    ///
    /// ⚠️ O backend LOCAL não a mede — ele reporta `usize::MAX` como *"não sei"*,
    /// que é diferente de zero e não pode ser lido como um resultado bom.
    pub irregular: usize,
}

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
    /// da forma. Devolve o [`QuadRemeshReport`].
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
        detail: f32,
        adaptive: f32,
    ) -> Result<QuadRemeshReport, RemeshRefusal> {
        if self.level_count() != 1 {
            return Err(RemeshRefusal::MultiresStack);
        }
        let t = std::time::Instant::now();
        let mesh = self.mesh();
        // ⚠️ **O LADO DO QUAD SAI DA MALHA, e não do slider** — ver
        // `ph2d_quadflow::edge_for_detail`. Um tamanho absoluto vindo do painel é
        // destrutivo numa malha grossa e conservador numa fina, e foi o defeito
        // que o smoke do Enio fotografou (2026-08-19).
        let edge = ph2d_quadflow::edge_for_detail(mesh, detail);
        let scale = ph2d_quadflow::ScaleField::adaptive(mesh, edge, adaptive);
        let (orient, pos) = ph2d_quadflow::solve_fields(mesh, &scale);
        // ⚠️ **NÃO HÁ passe de relaxação aqui, e a ausência é MEDIDA.** Eu
        // construí um (Laplaciano tangente + reprojeção) enquanto a extração
        // ainda era invenção minha, e ele comprava pouco. Com a extração PORTADA
        // da referência ele passou a **piorar as três fixturas em todas as
        // grandezas** — o Hausdorff da malha da cena vai de 0,60 para 1,49 quad
        // com 16 passadas. A grade que sai do campo cruzado já está alinhada; um
        // Laplaciano por cima briga com ela. Ver ADR-0160 §5-octies.
        let q = ph2d_quadflow::extract(mesh, &orient, &pos, &scale).map_err(RemeshRefusal::Quad)?;
        // ⚠️ **A recusa é NOMEADA, e não uma malha vazia.** Com o `detail` toda
        // corrida cai dentro da faixa legal, então este braço é uma rede — mas a
        // diferença entre *"a peça sumiu"* e *"ele disse por quê"* é a razão de
        // ele existir, e o custo é uma comparação.
        if q.mesh.faces().is_empty() {
            return Err(RemeshRefusal::TooCoarseToResolve);
        }
        let out = QuadRemeshReport {
            verts: q.mesh.vert_count(),
            quads: q.quads,
            non_quads: q.non_quads,
            edge,
            ms: t.elapsed().as_secs_f64() * 1000.0,
            holes: q.holes,
            // ⚠️ **`MAX` é *"não sei"*, e é a resposta honesta.** Este backend não
            // conta valências; escrever `0` seria afirmar uma grade perfeita
            // sobre um motor que entrega 21 a 49 % de irregulares.
            irregular: usize::MAX,
        };
        let previous =
            core::mem::replace(self.mesh_mut().ok_or(RemeshRefusal::EmptyScene)?, q.mesh);
        self.record(StrokeUndo::Remeshed(Box::new(previous)));
        // A malha é OUTRA: o traço em voo fala de vértices que não existem mais.
        self.stroke = SculptStroke::default();
        self.mesh_rebuilt();
        Ok(out)
    }
}
