//! **QUANTO a história pode pesar** — o orçamento em bytes e a poda.
//!
//! Filho (`#[path]`) de [`super`] pelo motivo do `sculpt3d_undo.rs`: o corte é
//! de responsabilidade. O pai responde *o que a história GUARDA e como se
//! desfaz*; aqui mora *quanto disso cabe*, que é uma pergunta de RECURSO e tem
//! precedente próprio nesta casa.
//!
//! ⚠️ **A entrada mais cara guarda a malha INTEIRA** ([`StrokeUndo::Remeshed`]),
//! e é isso que torna este arquivo necessário: medido pela residência do
//! processo (`ph2d-sdf/tests/probe_repeat_remesh.rs`), um remesh a 512 empilha
//! **146 MB** e o campo transiente que o produziu pede **922 MB** no pico —
//! contra os 3500 MB que o HR-13 declara para o app inteiro. Sem teto, *fazer
//! remesh algumas vezes* é uma escada até o fim da memória.

use super::{Entry, SceneObject, Sculpt3dScene, StrokeUndo};

impl StrokeUndo {
    /// **Quantos bytes esta entrada segura.**
    ///
    /// ⚠️ **O `match` é EXAUSTIVO de propósito**, e é o que torna o teto à prova
    /// da wave seguinte: uma variante nova não compila até responder quanto
    /// pesa. Um `_ => 0` deixaria a próxima carga grande escapar do teto em
    /// silêncio — e a carga grande de hoje ([`Self::Remeshed`]) mede **168 MB**
    /// numa malha de 512.
    pub(super) fn footprint_bytes(&self) -> usize {
        let plane =
            |v: &Option<Vec<f32>>| v.as_ref().map_or(0, |x| x.capacity() * size_of::<f32>());
        match self {
            Self::Stroke {
                verts,
                positions,
                masks,
                ..
            } => {
                verts.capacity() * size_of::<u32>()
                    + positions.capacity() * size_of::<[f32; 3]>()
                    + plane(masks)
            }
            Self::Mask { before, .. } => plane(before),
            Self::DroppedLevel(level) => level.bytes(),
            Self::Descended { stamped, .. } => stamped.bytes(),
            Self::ReversedLevel(r) => r.bytes(),
            Self::Remeshed(mesh) => mesh.footprint_bytes(),
            Self::Flattened(stack) => stack.footprint_bytes(),
            Self::RemovedObject(o) => o.footprint_bytes(),
            Self::Merged(objects) => objects.iter().map(SceneObject::footprint_bytes).sum(),
            // As entradas que não CARREGAM estado: a inversa delas é uma
            // operação, não um valor. Ver os doc-comments de cada uma.
            Self::AddedLevel
            | Self::Ascended { .. }
            | Self::FilledHoles { .. }
            | Self::AddedObject
            | Self::Unmerged
            | Self::UnfilledHoles
            | Self::UnreversedLevel => 0,
        }
    }
}

/// **Descarta as entradas mais VELHAS até `undo` caber em `budget`.** Devolve
/// `(quantas caíram, quantos bytes sobraram)`.
///
/// ⚠️ É função SOLTA pelo motivo do `swap_window` do pai: o resto do caminho
/// vive numa [`Sculpt3dScene`], que precisa de um `wgpu::Device`, e a política
/// de um teto só é conferível se alguém puder empurrar entradas caras para
/// dentro dela sem abrir uma janela.
pub(super) fn trim_to_budget(undo: &mut Vec<Entry>, budget: usize) -> (usize, usize) {
    let mut held: usize = undo.iter().map(|e| e.undo.footprint_bytes()).sum();
    let mut dropped = 0usize;
    // ⚠️ `len() > 1` e não `> 0`: o passo do TOPO é o que o próximo Ctrl+Z
    // desfaz, e um remesh de peça grande é irredutivelmente uma malha.
    while held > budget && undo.len() > 1 {
        let gone = undo.remove(0);
        held -= gone.undo.footprint_bytes();
        dropped += 1;
    }
    (dropped, held)
}

impl Sculpt3dScene {
    /// **O ORÇAMENTO da história, em BYTES.**
    ///
    /// ⚠️ **Função do DOCUMENTO, e a lei é emprestada de dois precedentes desta
    /// casa** — o histórico do editor de áudio (ADR-0117) e a U1 do Painter:
    /// `2 × documento + 256 MB`. Uma peça barata ganha história funda; uma cara
    /// ganha história curta, que é a troca certa quando o orçamento do app
    /// inteiro (HR-13) é 3500 MB.
    ///
    /// ⚠️ **Um teto por CONTAGEM seria multiplicador, não limite** — e é a
    /// medição que decide: um remesh a 512 empilha uma malha de **168 MB**,
    /// então dez deles seriam 1,7 GB de história sobre um campo transiente que
    /// já pede 922 MB no pico.
    pub(super) fn history_budget_bytes(&self) -> usize {
        const FLOOR: usize = 256 << 20;
        let doc: usize = self.objects.iter().map(SceneObject::footprint_bytes).sum();
        FLOOR + 2 * doc
    }

    /// Descarta as entradas MAIS VELHAS até a história caber no orçamento.
    ///
    /// ⚠️ **Um passo é sempre preservado**, por mais caro que seja: um remesh de
    /// peça grande é irredutivelmente uma malha, e recusar-lhe o desfazer para
    /// honrar um número seria a ferramenta destruindo trabalho — o precedente é
    /// o cap do editor de áudio, que guarda o clipe inteiro quando a edição é do
    /// clipe inteiro.
    ///
    /// ⚠️ **E este é o ÚNICO ponto de crescimento**, o que é o que torna um
    /// choke point suficiente: desfazer MOVE uma entrada para a fila oposta e
    /// refazer a traz de volta, então o total das duas filas é conservado; só um
    /// `record_*` acrescenta bytes.
    pub(super) fn trim_history(&mut self) {
        let budget = self.history_budget_bytes();
        let (dropped, held) = trim_to_budget(&mut self.undo, budget);
        if dropped > 0 {
            // Uma história que encolhe em silêncio é o artista descobrindo o
            // teto por acidente, três Ctrl+Z depois.
            eprintln!(
                "[sculpt3d] historico no teto: {dropped} passo(s) antigo(s) descartado(s) \
                 ({} MB retidos, orcamento {} MB)",
                held / (1024 * 1024),
                budget / (1024 * 1024),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{super::ObjectId, *};

    /// Uma entrada de remesh com o peso de uma malha real.
    fn remesh_entry(rings: usize) -> Entry {
        let mesh = ph2d_mesh::shapes::uv_sphere(rings, rings * 2, 1.0);
        Entry {
            object: ObjectId(0),
            undo: StrokeUndo::Remeshed(Box::new(mesh)),
        }
    }

    /// **O teto é em BYTES e ele MORDE.**
    ///
    /// Report do Enio (2026-08-10): *"após algumas vezes fazendo remesh a 512, a
    /// esfera sumiu do nada"*. O motor está limpo sob repetição (medido: cinco
    /// remeshes encadeados a 512 convergem), e o que crescia sem limite era esta
    /// fila — cada entrada uma malha inteira.
    #[test]
    fn a_history_of_whole_meshes_stops_growing_at_its_budget() {
        let one = remesh_entry(24).undo.footprint_bytes();
        assert!(one > 0, "uma malha tem de pesar alguma coisa");
        let budget = one * 3 + one / 2;

        let mut undo: Vec<Entry> = Vec::new();
        for _ in 0..20 {
            undo.push(remesh_entry(24));
            trim_to_budget(&mut undo, budget);
        }

        let held: usize = undo.iter().map(|e| e.undo.footprint_bytes()).sum();
        assert!(
            held <= budget,
            "a história ficou em {held} bytes contra um orçamento de {budget}"
        );
        // E ela guarda o que CABE, não o mínimo: um teto que deixasse um passo
        // só seria um teto que apaga o desfazer.
        assert!(
            undo.len() >= 3,
            "o orçamento comporta ~3 passos e a poda deixou {}",
            undo.len()
        );
    }

    /// **O passo do topo sobrevive mesmo sozinho maior que o orçamento** — o
    /// precedente do editor de áudio: uma edição irredutivelmente cara continua
    /// desfazível.
    #[test]
    fn the_newest_step_survives_even_when_it_alone_blows_the_budget() {
        let mut undo = vec![remesh_entry(16), remesh_entry(16)];
        let (dropped, _) = trim_to_budget(&mut undo, 1);
        assert_eq!(dropped, 1, "a mais velha tem de cair");
        assert_eq!(undo.len(), 1, "e exatamente uma sobrevive");
    }

    /// **A entrada de remesh pesa a MALHA** — se este número for zero, o teto
    /// nunca morde e o gate acima passa por vácuo.
    #[test]
    fn a_remesh_entry_weighs_what_the_mesh_weighs() {
        let mesh = ph2d_mesh::shapes::uv_sphere(24, 48, 1.0);
        let want = mesh.footprint_bytes();
        let got = StrokeUndo::Remeshed(Box::new(mesh)).footprint_bytes();
        assert_eq!(got, want);
        assert!(
            want > 100_000,
            "uma esfera de 24×48 tem de pesar mais que cem mil bytes, e mediu {want}"
        );
    }

    /// **A entrada de ACHATAR pesa a PILHA INTEIRA**, e é a mais cara que
    /// existe: ela carrega TODOS os níveis, não uma malha.
    ///
    /// ⚠️ E ela pesa **mais** que a de remesh sobre a mesma malha — é o número
    /// que justifica ela passar pelo teto como todas as outras. Um `0` aqui
    /// deixaria a fila crescer sem limite no gesto que mais carrega.
    #[test]
    fn a_flatten_entry_weighs_the_whole_stack() {
        let mut stack = ph2d_mesh::Multires::new(ph2d_mesh::shapes::uv_sphere(12, 24, 1.0));
        assert!(stack.add_level());
        let one_mesh = stack.mesh().footprint_bytes();
        let want = stack.footprint_bytes();
        let got = StrokeUndo::Flattened(Box::new(stack)).footprint_bytes();
        assert_eq!(got, want);
        assert!(
            want > one_mesh,
            "a pilha ({want}) tem de pesar mais que a malha do topo ({one_mesh}) -- \
             ela carrega os DOIS níveis e o detalhe"
        );
    }

    /// **Uma entrada sem estado pesa ZERO** — o controle: se tudo pesasse algo,
    /// o `match` exaustivo estaria a contar o que não existe.
    #[test]
    fn a_stateless_entry_weighs_nothing() {
        assert_eq!(StrokeUndo::AddedObject.footprint_bytes(), 0);
        assert_eq!(StrokeUndo::Unmerged.footprint_bytes(), 0);
    }
}
