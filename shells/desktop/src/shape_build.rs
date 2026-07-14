//! **Shape Builder** — o cursor pinta REGIÕES, e o que ele pinta vira forma.
//!
//! Com 2+ formas fechadas selecionadas, o plano fica dividido nas faces do arranjo
//! ([`ph2d_vec_boolean::Arrangement`]): "dentro da A e fora da B", "dentro das duas", e
//! assim por diante. O Shape Builder deixa o dedo passar por cima dessas faces:
//!
//! - **Arrastar** (ou clicar): as faces tocadas viram **UMA** forma.
//! - **Alt + arrastar**: as faces tocadas **somem**.
//!
//! É a diferença entre pedir uma operação e **desenhar o resultado**. Um Pathfinder obriga
//! o artista a traduzir o que ele quer ("a lua crescente") numa sequência de ops ("subtrai
//! o círculo B do A"); aqui ele passa o dedo na parte que quer e pronto.
//!
//! ## A sobra é POR FORMA — e é isto que o smoke do Enio derrubou
//!
//! A 1ª versão devolvia a sobra como `união(todas as fontes) − o que foi levado`: **uma**
//! forma só. O efeito no app real (medido, [`crate::build_smoke`]): um clique numa região
//! fundia o pentágono, a estrela e o retângulo num BLOB único, com um estilo só — as
//! silhuetas que o artista tinha acabado de desenhar simplesmente sumiam, e as fronteiras
//! entre elas (que são justamente o que define as faces seguintes) deixavam de existir.
//! Um clique destruía a arte.
//!
//! Agora cada forma de origem sobrevive como **ela mesma menos o que foi levado**, com o
//! ESTILO dela e no z dela; a forma que ninguém tocou **não é sequer tocada** (mantém id,
//! entidade, `Transform`, raio de quina e os params de Live Shape — uma booleana que passa
//! por perto não pode assar uma forma paramétrica que o artista não escolheu). É o que o
//! Illustrator faz: o Shape Builder divide o que você percorre e deixa o resto em paz.
//!
//! ## A regra do estilo (uma só, e é a que a booleana já usava)
//!
//! A forma NOVA (as faces pintadas) herda o estilo da forma do **TOPO entre as tocadas** — a
//! convenção do `apply_many` e do Illustrator. Sem essa regra, a face herdaria o estilo do
//! último argumento de uma SUBTRAÇÃO, que é justamente uma forma que ela **não** contém.

use ph2d_vec_boolean::{Arrangement, BoolOp, FaceId, MAX_BUILD_SHAPES, Membership, apply_many};
use ph2d_vec_scene::{VecPath, VecPathId, VecScene, VecXforms};

/// O que um gesto de Shape Builder produz. Tudo em MUNDO.
#[derive(Default)]
pub(crate) struct BuildResult {
    /// A forma nova: a união das faces pintadas (vazia no modo subtrair, ou se nada foi
    /// pintado). Mais de uma entrada quando as faces pintadas são desconexas.
    pub(crate) merged: Vec<VecPath>,
    /// O que sobra de cada forma **tocada**, por índice de fonte (fundo → topo). Uma lista
    /// vazia = a fonte foi inteiramente levada e deixa de existir. As fontes que o gesto
    /// não tocou **não aparecem aqui** — o path delas não é mexido.
    pub(crate) remainder: Vec<(usize, Vec<VecPath>)>,
}

impl BuildResult {
    pub(crate) fn is_empty(&self) -> bool {
        self.merged.is_empty() && self.remainder.is_empty()
    }
}

/// O gesto de Shape Builder em curso.
pub(crate) struct BuildSession {
    /// As faces, em MUNDO (as formas entram assadas — a booleana precisa de um frame só).
    pub(crate) arr: Arrangement,
    /// Os ids das formas de origem, **alinhados com `arr.sources()`** (mesma ordem, mesmo
    /// comprimento: `open` descarta as abertas e as que sumiram). São eles que o `build_up`
    /// consome.
    pub(crate) sources: Vec<VecPathId>,
    /// A seleção (em z) para a qual esta sessão foi aberta, e a impressão digital da
    /// geometria+pose dela. O `upkeep` reabre a sessão quando qualquer um dos dois muda —
    /// senão o arranjo seguiria descrevendo formas que já não estão ali.
    pub(crate) opened_for: SourceKey,
    /// As faces que o dedo já pintou, na ordem em que foram tocadas.
    pub(crate) marked: Vec<FaceId>,
    /// A face sob o cursor agora (o realce que segue o mouse mesmo sem botão apertado).
    pub(crate) hover: Option<FaceId>,
    /// Alt: o gesto SUBTRAI em vez de unir. Fixado no press — trocar de modo no meio do
    /// arrasto faria o mesmo gesto significar duas coisas.
    pub(crate) subtract: bool,
    /// Entre o press e o release.
    pub(crate) dragging: bool,
}

/// A identidade da entrada do arranjo: quem, com que pose, e com que geometria.
///
/// **Detector de mudança, não hash criptográfico:** id, nº de vértices, a soma das âncoras e
/// o afim. Reabrir o arranjo é o que impede o véu de descrever a forma onde ela *estava* (o
/// arranjo é assado em MUNDO; se a pose muda, ele mente).
pub(crate) type SourceKey = Vec<(VecPathId, usize, [f64; 2], ph2d_vec_scene::Xform)>;

/// A impressão digital da seleção `ids` (na ordem dada).
pub(crate) fn source_key(scene: &VecScene, xforms: &VecXforms, ids: &[VecPathId]) -> SourceKey {
    ids.iter()
        .filter_map(|id| {
            let p = scene.paths().iter().find(|p| p.id == *id)?;
            let mut n = 0usize;
            let mut sum = [0.0f64; 2];
            for v in p.verts_all() {
                n += 1;
                sum[0] += v.anchor[0];
                sum[1] += v.anchor[1];
            }
            Some((*id, n, sum, ph2d_vec_scene::xform_of(xforms, *id)))
        })
        .collect()
}

impl BuildSession {
    /// Abre a sessão para as formas `ids` (ordem de z, fundo → topo), assando cada uma no
    /// MUNDO. `None` com menos de 2 formas fechadas — não há região para pintar.
    pub(crate) fn open(
        scene: &VecScene,
        xforms: &VecXforms,
        ids: &[VecPathId],
    ) -> Option<BuildSession> {
        // `sources` e `arr.sources()` têm de ficar ALINHADOS: uma forma aberta (ou que
        // sumiu) sai da lista de ids também, senão o índice `i` do arranjo apontaria para o
        // id errado — e o `build_up` consumiria a forma errada.
        let mut kept: Vec<VecPathId> = Vec::new();
        let mut world: Vec<VecPath> = Vec::new();
        for id in ids {
            let Some(p) = scene.paths().iter().find(|p| p.id == *id) else {
                continue;
            };
            if !p.closed {
                continue;
            }
            let mut w = p.clone();
            ph2d_vec_scene::bake_xform(&mut w, &ph2d_vec_scene::xform_of(xforms, p.id));
            kept.push(*id);
            world.push(w);
            if kept.len() == MAX_BUILD_SHAPES {
                break; // o mesmo teto que o `Arrangement::new` aplica
            }
        }
        if world.len() < 2 {
            return None;
        }
        Some(BuildSession {
            arr: Arrangement::new(world),
            opened_for: source_key(scene, xforms, &kept),
            sources: kept,
            marked: Vec::new(),
            hover: None,
            subtract: false,
            dragging: false,
        })
    }

    /// O cursor andou (mundo): atualiza o realce e, se estiver arrastando, PINTA a face.
    pub(crate) fn touch(&mut self, world: [f64; 2]) {
        let face = self.arr.face_at(world);
        self.hover = face;
        if let (true, Some(f)) = (self.dragging, face)
            && !self.marked.contains(&f)
        {
            self.marked.push(f);
        }
    }

    /// O que este gesto produz. Vazio = nada a fazer.
    ///
    /// - **Unir:** as faces pintadas viram uma forma nova; cada fonte tocada perde o que foi
    ///   levado.
    /// - **Subtrair:** o mesmo, sem a forma nova — as faces pintadas simplesmente somem.
    ///
    /// As duas são a MESMA conta (a única diferença é se o que foi pintado é entregue ou
    /// jogado fora), e é isso que garante que "pintar tudo e unir" e "pintar tudo e
    /// subtrair" sejam exatamente complementares.
    pub(crate) fn resolve(&mut self) -> BuildResult {
        if self.marked.is_empty() {
            return BuildResult::default();
        }
        let faces: Vec<VecPath> = self
            .marked
            .clone()
            .into_iter()
            .filter_map(|f| self.arr.face_path(f).cloned())
            .collect();
        if faces.is_empty() {
            return BuildResult::default();
        }
        let taken = union_of(&faces);
        // As fontes que o gesto tocou: o bit `i` ligado em ALGUMA face pintada. Uma fonte
        // que não aparece em nenhuma delas não perde área nenhuma — e não é tocada.
        let touched: Membership = self.marked.iter().fold(0, |a, f| a | f.membership);

        let mut remainder = Vec::new();
        for i in 0..self.arr.len() {
            if touched & (1 << i) == 0 {
                continue;
            }
            let src = self.arr.sources()[i].clone();
            let mut rest = subtract_all(&src, &taken);
            // O `apply_many` tira o estilo do ÚLTIMO argumento — que aqui é uma FACE, não a
            // fonte. O que sobra de uma forma continua sendo aquela forma: o estilo é dela.
            for p in &mut rest {
                p.fill = src.fill.clone();
                p.stroke = src.stroke;
            }
            drop_slivers(
                &mut rest,
                ph2d_vec_boolean::area(&src),
                &format!("sobra da fonte {i}"),
            );
            remainder.push((i, rest));
        }

        let mut merged = if self.subtract { Vec::new() } else { taken };
        // A forma NOVA também passa pelo filtro: a união das faces é geometria re-derivada
        // como qualquer outra. A referência é a área dela mesma (a soma das peças) — se tudo
        // degenerar, o `>` estrito não deixa nada passar.
        let merged_area: f64 = merged.iter().map(ph2d_vec_boolean::area).sum();
        drop_slivers(&mut merged, merged_area, "forma nova");
        // O estilo da forma nova: o da fonte do TOPO entre as tocadas (Illustrator).
        if let Some(top) = (0..self.arr.len())
            .rev()
            .find(|i| touched & (1 << i) != 0)
            .map(|i| &self.arr.sources()[i])
        {
            for p in &mut merged {
                p.fill = top.fill.clone();
                p.stroke = top.stroke;
            }
        }
        BuildResult { merged, remainder }
    }
}

/// Aplica o resultado do gesto na cena, e devolve a **seleção** que fica (o que sobrou + o
/// que nasceu + as fontes intactas) — é ela que reabre o arranjo no frame seguinte, e é o
/// que deixa o artista continuar construindo em cima do resultado.
///
/// Mora aqui, e não no gesto, porque é a metade da regra que se pode PROVAR sem `App`: é
/// esta função que decide o que é destruído. O `build_up` é só a ponte com o frame.
pub(crate) fn commit(
    scene: &mut VecScene,
    sources: &[VecPathId],
    result: BuildResult,
) -> Vec<VecPathId> {
    let touched: Vec<VecPathId> = result
        .remainder
        .iter()
        .filter_map(|(i, _)| sources.get(*i).copied())
        .collect();
    let mut sel: Vec<VecPathId> = sources
        .iter()
        .filter(|id| !touched.contains(id))
        .copied()
        .collect();
    // A fatia de z da tocada mais de trás — o grupo re-empilha ali (a forma não salta para
    // o topo do documento), com a forma NOVA por cima do que sobrou.
    let at = touched
        .iter()
        .filter_map(|id| scene.paths().iter().position(|p| p.id == *id))
        .min()
        .unwrap_or(0);
    for id in &touched {
        scene.remove_path(*id);
    }
    let mut k = at;
    for (_, rest) in result.remainder {
        for r in rest {
            sel.push(scene.insert_path(k, r));
            k += 1;
        }
    }
    for m in result.merged {
        sel.push(scene.insert_path(k, m));
        k += 1;
    }
    sel
}

/// Fração da área da FONTE abaixo da qual uma peça **não é região** — é resíduo de borda.
///
/// **De onde vem a lasca** (medido, não suposto): toda sobra é `fonte − união(faces levadas)`,
/// e as faces vêm do **arranjo**. A borda que elas devolvem é uma *re-derivação* da borda da
/// fonte, não a mesma sequência de bytes — então subtrair uma curva **dela mesma**, depois de
/// ela ter dado a volta pelo arranjo, deixa resíduo. A booleana comum (os botões do painel) é
/// **limpa**: 2 operandos, zero lasca. Isto é do Build.
///
/// **E é por isso que ela aparece como LINHA, não como forma pequena:** sem área, não há
/// preenchimento para pintar — o que sobra na tela é o *traço* da fonte, percorrendo a borda
/// e voltando. Medir bbox ou comprimento não a distingue de arte nenhuma. Só a área.
///
/// As duas populações, medidas no produto (`PH2D_BUILD_LOG=1`), não se tocam:
///
/// | | área (fração da fonte) | verts | densidade (área/bbox) |
/// |---|---|---|---|
/// | resíduo de borda **curva** | **0,0000%** (área exatamente 0) | 144 | 0,00 |
/// | resíduo de **quina** | 0,07% – 0,30% | 3–4 | 0,44 |
/// | **arte** | **6,5% – 81%** | 3–21 | 0,41 – 0,79 |
///
/// O piso fica em **0,5%**: 13× abaixo da menor peça de arte medida e 1,7× acima do maior
/// resíduo. É o único número aqui, e ele sai da tabela — não de um chute.
const SLIVER_AREA_FRACTION: f64 = 0.005;

/// Tira as lascas: o que não tem área não é região.
///
/// O `>` é estrito de propósito — com uma referência degenerada (área 0) o piso vira 0, e uma
/// peça de área 0 continua caindo fora. `PH2D_BUILD_LOG=1` diz o que foi descartado e por quê:
/// geometria que some em silêncio é pior que uma lasca, então ela nunca some em silêncio.
fn drop_slivers(pieces: &mut Vec<VecPath>, reference_area: f64, what: &str) {
    let floor = SLIVER_AREA_FRACTION * reference_area;
    let log = std::env::var_os("PH2D_BUILD_LOG").is_some();
    pieces.retain(|p| {
        let a = ph2d_vec_boolean::area(p);
        let keep = a > floor;
        if log {
            let pct = 100.0 * a / reference_area.max(f64::MIN_POSITIVE);
            let verdict = if keep { "ARTE" } else { "lasca (descartada)" };
            let nverts = p.verts.len();
            eprintln!(
                "[build] {what}: area={a:.8} ({pct:.4}% de {reference_area:.4}) · \
                 verts={nverts} · {verdict}"
            );
        }
        keep
    });
}

/// A união de um punhado de formas. Uma só é ela mesma (o `apply_many` exige duas).
fn union_of(paths: &[VecPath]) -> Vec<VecPath> {
    if paths.len() == 1 {
        return paths.to_vec();
    }
    apply_many(&paths.iter().collect::<Vec<_>>(), BoolOp::Union)
}

/// `base − (∪ cutters)`. O `Subtract` do `apply_many` já é `base − (c1 ∪ c2 ∪ …)` (o
/// acumulador dobra), então uma chamada basta.
fn subtract_all(base: &VecPath, cutters: &[VecPath]) -> Vec<VecPath> {
    if cutters.is_empty() {
        return vec![base.clone()];
    }
    let mut args: Vec<&VecPath> = Vec::with_capacity(1 + cutters.len());
    args.push(base);
    args.extend(cutters.iter());
    apply_many(&args, BoolOp::Subtract)
}

#[cfg(test)]
#[path = "shape_build_tests.rs"]
mod tests;
