//! ⭐⭐⭐ **O CAMINHO DO MAPA DE GRADE INTEIRA** — o de OMISSÃO desde 2026-08-25.
//! `PH2D_RETOPO_EXTRACT=0` volta ao de sempre.
//!
//! Irmão (`#[path]`) do [`super::retopo_global`], e o corte é de **fase**: lá a
//! cadeia que decompõe em patches, quantiza e monta cada patch (F3–F5); aqui a que
//! resolve **um mapa para a peça inteira**, arredonda-o para a grade inteira (G5) e
//! extrai a malha das **isolinhas** dele.
//!
//! ⭐⭐⭐ **Ele passou a ser o DEFAULT em 2026-08-25, por ordem do dono do produto**
//! (*«pode ligar o motor novo; o antigo não apresenta resultados úteis»*). ⚠️ **A
//! afirmação de byte-identidade INVERTE-SE, e continua a valer:** com
//! `PH2D_RETOPO_EXTRACT=0` a [`super::retopo_global::quad_remesh_global`] é
//! byte-idêntica ao que sempre foi — a bifurcação continua a ser **uma só**, na
//! primeira linha dela, e há gate a contá-la.
//!
//! # ⚠️ O que a medição diz HOJE, e é por isso que ele está desligado
//!
//! Medido em 2026-08-24, cadeia inteira com a **fase zero** honrada:
//!
//! | peça | dobras do mapa | quads | `χ` | ⭐ aspecto p50 | ⭐ enviesamento p50 |
//! |---|---|---|---|---|---|
//! | ⭐ esfera fina (96×144) | **0 %** | `2 102` | ⚠️ `−5` | ⭐ **`1,10`** | ⭐ **`6,8°`** |
//! | toro (alça) | `3,3 %` | `1 495` | ⛔ `−20` | `1,29` | `5,8°` |
//! | esfera lisa (24×36) | ⛔ `11 %` | `410` | ⛔ `−14` | `2,02` | `22,1°` |
//!
//! ⭐⭐ **A forma da esfera fina está DENTRO da barra do oráculo** (`1,08`–`1,22` de
//! aspecto, `4,8°`–`7,1°` de enviesamento). ⛔ **O que falta é a topologia**, e a
//! causa está medida e é a montante: o mapa contínuo do G3 entrega até `11 %` de
//! triângulos dobrados e uma translação de costura a meia célula de um inteiro,
//! contra `0,02 %`–`0,2 %` e `3,5e-15` dos mapas de referência. *A extracção e o
//! arredondamento não são o bloqueador; o solver contínuo é.*
//!
//! # ⭐⭐⭐ E ESSA CAUSA FOI CURADA (2026-08-24) — a costura entra por ELIMINAÇÃO
//!
//! O G3 **pesava** a costura; hoje ela é uma restrição eliminada
//! ([`ph2d_gridmap::round_welded`]). ⇒ o resíduo da costura deixa de ser uma célula
//! inteira e passa a ser **zero**, e a casca fecha. Medido na cadeia inteira, nas duas
//! peças que o artista de facto olhou:
//!
//! | peça | | arestas de bordo | células más | `χ` | aspecto p50 | enviesamento p50 | `>60°` |
//! |---|---|---|---|---|---|---|---|
//! | enrugada | penalizado | ⛔ `46` | `19 de 2 041` | ⛔ `−8` | `1,15` | `5,7°` | `4` |
//! | enrugada | ⭐ **soldado** | ⭐ **`0`** | ⭐ **`0`** | ⭐ **`+2`** | `1,15` | `6,3°` | ⚠️ `11` |
//! | orelha | penalizado | ⛔ `50` | `33 de 2 071` | ⛔ `−6` | `1,12` | `7,1°` | `7` |
//! | orelha | ⭐ **soldado** | ⭐ **`0`** | ⭐ **`0`** | ⭐ **`+2`** | `1,14` | ⚠️ `8,2°` | `7` |
//!
//! ⚠️ **A regressão que fica tem nome e uma cura publicada:** as faces com canto pior
//! que `60°` sobem de `4` para `11` na enrugada, e o enviesamento da orelha passa o
//! tecto do oráculo por `1,1°`. O mecanismo é o *local stiffening* do mesmo *paper*
//! (§5.4) — pesar por triângulo o que ficou distorcido e re-resolver. ⛔ **Não é desta
//! wave, de propósito:** com dois mecanismos dentro, uma regressão de forma fica sem
//! dono.
//!
//! ⚠️ **`PH2D_GRIDMAP_WELD=0` volta ao G3 penalizado**, dentro deste caminho — é a
//! forma de bissecar.

use ph2d_mesh::Mesh;

use super::remesh::QuadRemeshReport;
use super::{RemeshRefusal, Sculpt3dScene, SculptStroke, StrokeUndo};

impl Sculpt3dScene {
    /// **A RETOPOLOGIA POR MAPA DE GRADE INTEIRA.** Devolve o mesmo
    /// [`QuadRemeshReport`] das outras duas — é o mesmo botão.
    pub(in crate::sculpt3d) fn quad_remesh_extract(
        &mut self,
        detail: f32,
        adaptive: f32,
    ) -> Result<QuadRemeshReport, RemeshRefusal> {
        if self.level_count() != 1 {
            return Err(RemeshRefusal::MultiresStack);
        }
        let _ = adaptive;
        let t = std::time::Instant::now();

        // ── F1. A fase zero. ⛔ **Não a salte, e não meça sem ela:** com a
        // triangulação crua a mesma cadeia dá o dobro do enviesamento.
        let reference = self.mesh().clone();
        let mut work = reference.clone();
        ph2d_remesh_iso::remesh_isotropic(&mut work, ph2d_remesh_iso::ALPHA);
        work.triangulate();

        // ⭐ **O alvo sai da malha que o artista trouxe** — a mesma lei do irmão, e a
        // alternativa (derivá-lo da remalhada) foi medida e mata o slider.
        let target = ph2d_quadflow::edge_for_detail_with(
            &reference,
            detail,
            ph2d_quadflow::GLOBAL_FLOOR_IN_INPUT_EDGES,
        );

        // ── F2 + F3 + G1 + G2.
        let dual = ph2d_crossfield::Dual::build(&work);
        let (field, _) = ph2d_crossfield::solve_miq(&dual);
        let layout = ph2d_trace::trace_patches(&work, &dual, &field);
        let (cut, _) = ph2d_gridmap::cut_along_patches(&work, &layout);
        let (combed, _) = ph2d_gridmap::comb_patches(&work, &layout, &cut);

        // ⭐ As singularidades saem do CAMPO — o índice por-vértice é um facto dele, e
        // pedir à `ph2d-gridmap` que o re-derive seria reconstruir o que já existe.
        let singular: Vec<u32> = ph2d_crossfield::vertex_index(&work, &dual, &field)
            .into_iter()
            .enumerate()
            .filter(|(_, k)| *k != 0)
            .filter_map(|(v, _)| u32::try_from(v).ok())
            .collect();

        // ── G3 + G5. O mapa, e o arredondamento uma-a-uma que o torna inteiro.
        // ⭐ O G3 soldado é o default DENTRO deste caminho (que já shipa desligado);
        // `PH2D_GRIDMAP_WELD=0` volta ao penalizado, para bissecar.
        let welded = ph2d_gridmap::welded_enabled();
        let opts = ph2d_gridmap::RoundOptions::default();
        let (map, round) = if welded {
            ph2d_gridmap::round_welded(&work, &cut, &combed, target, opts, &singular)
        } else {
            ph2d_gridmap::round_to_integers(&work, &cut, &combed, target, opts, &singular)
        };

        // ── A extracção das isolinhas.
        let (tris, uv) = ph2d_gridmap::corner_map(&cut, &map);
        let cm = ph2d_quadextract::CornerMap {
            pos: work.positions(),
            tris: &tris,
            uv: &uv,
        };
        let (out, e) = ph2d_quadextract::extract(&cm, None).map_err(RemeshRefusal::Extract)?;
        if out.faces().is_empty() {
            return Err(RemeshRefusal::TooCoarseToResolve);
        }

        let shape = ph2d_quadfill::quad_shape(&out);
        let (edge_median, edge_max) = edges(&out);
        let report = QuadRemeshReport {
            verts: out.vert_count(),
            quads: e.quads,
            non_quads: out.face_count() - e.quads,
            edge: target,
            ms: t.elapsed().as_secs_f64() * 1000.0,
            holes: boundary_edges(&out),
            irregular: irregular(&out),
            edge_max_ratio: edge_max / target,
            edge_median_ratio: edge_median / target,
            edge_max_span: edge_max / span(&reference),
            shape,
            // ⚠️ **As dobras aqui são as do MAPA e não as da saída**, e é a coluna
            // que decide se a peça tinha como sair bem: a extracção tolera a dobra
            // por construção, e o que ela não pode é inventar grade onde o mapa se
            // enrola sobre si próprio.
            folded: e.folded_faces,
            aligned: round.shift_frac_max == 0.0,
        };
        let previous = core::mem::replace(self.mesh_mut().ok_or(RemeshRefusal::EmptyScene)?, out);
        self.record(StrokeUndo::Remeshed(Box::new(previous)));
        self.stroke = SculptStroke::default();
        self.mesh_rebuilt();
        Ok(report)
    }
}

/// **O CAMINHO NOVO É O DE OMISSÃO** — `PH2D_RETOPO_EXTRACT=0` volta ao de sempre.
#[must_use]
pub(in crate::sculpt3d) fn extract_requested() -> bool {
    extract_from(std::env::var("PH2D_RETOPO_EXTRACT").ok().as_deref())
}

/// **A DECISÃO, sem tocar no ambiente** — a metade que se pode gatear.
///
/// ⭐⭐⭐ **O DEFAULT VIROU em 2026-08-25, por ordem do dono do produto** — *«pode ligar
/// o motor novo; o antigo não apresenta resultados úteis»* — e a medição que o suporta
/// está no [handoff de 24/08](../../../docs/3D/handoffs/HANDOFF_INTEGRACAO_line_seamelim_2026-08-24.md):
/// em cinco peças fechadas do corpus a casca passou a fechar (`χ` de `−4`..`−13` para
/// `+2`, arestas de bordo de `30`–`78` para `0`), a forma ficou dentro da barra do
/// oráculo, e a cadeia é **3–4× mais rápida**.
///
/// ⚠️ **A LEI DA CASA INVERTE-SE AQUI, e isso é dito em voz alta:** *tudo o que é novo
/// shipa desligado* valeu enquanto o caminho novo não fechava a casca. Ele fecha. ⇒ o
/// que fica desligado passa a ser o **antigo**, e é ele que agora precisa de ser pedido.
///
/// ⚠️ **O `"0"` continua a ser a única palavra que desliga** (não `"false"`, não
/// `"off"`) — a mesma lei do `PH2D_GPU_COOK`, do `PH2D_FLIP_NEW_ENGINE` e do
/// `PH2D_GRIDMAP_WELD`. *Uma variável com dois vocabulários é duas variáveis.*
#[must_use]
pub(in crate::sculpt3d) fn extract_from(value: Option<&str>) -> bool {
    value != Some("0")
}

/// A aresta mediana e a mais longa da saída.
fn edges(mesh: &Mesh) -> (f32, f32) {
    let pos = mesh.positions();
    let mut e: Vec<f32> = Vec::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (pos[v[k] as usize], pos[v[(k + 1) % v.len()] as usize]);
            let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
            e.push(d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt());
        }
    }
    e.sort_by(f32::total_cmp);
    (
        e.get(e.len() / 2).copied().unwrap_or(0.0),
        e.last().copied().unwrap_or(0.0),
    )
}

/// Arestas com uma face só — a assinatura da casca aberta.
fn boundary_edges(mesh: &Mesh) -> usize {
    use std::collections::BTreeMap;
    let mut n: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            *n.entry(if a < b { (a, b) } else { (b, a) }).or_default() += 1;
        }
    }
    n.values().filter(|c| **c == 1).count()
}

/// Vértices com valência diferente de 4 — a grandeza que o pivô existiu para
/// derrubar. ⭐ Uma grade numa esfera admite **oito**.
fn irregular(mesh: &Mesh) -> usize {
    let mut deg = vec![0usize; mesh.vert_count()];
    use std::collections::BTreeSet;
    let mut seen: BTreeSet<(u32, u32)> = BTreeSet::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            if seen.insert(if a < b { (a, b) } else { (b, a) }) {
                deg[a as usize] += 1;
                deg[b as usize] += 1;
            }
        }
    }
    deg.iter().filter(|d| **d != 4 && **d > 0).count()
}

/// **A DIAGONAL da caixa da peça** — o denominador da fração absoluta, e a mesma
/// régua do irmão.
fn span(mesh: &Mesh) -> f32 {
    let b = mesh.bounds();
    let d = [
        b.max[0] - b.min[0],
        b.max[1] - b.min[1],
        b.max[2] - b.min[2],
    ];
    d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt()
}

#[cfg(test)]
mod tests {
    /// ⭐⭐⭐ **GATE 11 — o caminho antigo continua byte-idêntico enquanto o
    /// interruptor estiver desligado.**
    ///
    /// ⚠️ **A decisão é pura de propósito.** O gesto em si precisa de GPU (a cena
    /// segura buffers de device), então um gate sobre ele é `skip` gracioso na
    /// máquina sem adapter — e *skip gracioso não é verde*. O que se pina aqui é a
    /// **decisão**, que é a única coisa que a env acrescenta ao caminho de sempre.
    #[test]
    fn o_caminho_novo_e_o_de_omissao_e_so_o_zero_o_desliga() {
        for (value, want) in [
            // ⭐⭐ O caso por omissão VIROU em 2026-08-25 (ordem do dono do produto): é o
            // caminho NOVO que o Enio recebe sem configurar nada. *A lei «shipa
            // desligado» valeu enquanto ele não fechava a casca; ele fecha.*
            (None, true),
            // ⚠️ E o `"0"` é a ÚNICA palavra que desliga — quem quer o de sempre tem de
            // o pedir por este nome exacto.
            (Some("0"), false),
            (Some("1"), true),
            (Some("sim"), true),
            (Some(""), true),
        ] {
            assert_eq!(
                super::extract_from(value),
                want,
                "PH2D_RETOPO_EXTRACT={value:?} tinha de dar {want}"
            );
        }
    }

    /// ⭐⭐ **E A BIFURCAÇÃO É UMA SÓ** — o que faz o «byte-idêntico» ser
    /// verificável em vez de prometido.
    ///
    /// ⚠️ **O gate LÊ O FONTE**, e é de propósito: um segundo sítio a chamar
    /// [`super::extract_requested`] compilaria, passaria a suíte, e partiria a
    /// afirmação de que o caminho antigo está intocado. *Uma promessa sobre o
    /// código não é uma propriedade do código até alguém a contar.*
    #[test]
    fn a_bifurcacao_para_o_caminho_novo_e_uma_so() {
        let src = include_str!("sculpt3d_history_retopo_global.rs");
        let n = src.matches("extract_requested()").count();
        assert_eq!(
            n, 1,
            "a cadeia global chama `extract_requested()` {n} vezes; tem de ser UMA, \
             na primeira linha da porta"
        );
        assert_eq!(
            src.matches("quad_remesh_extract(").count(),
            1,
            "e chama o caminho novo uma vez so'"
        );
    }
}
