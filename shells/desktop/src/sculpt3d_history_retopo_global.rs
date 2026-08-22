//! **A PORTA DA CADEIA GLOBAL** — o botão a chamar o motor do pivô (ADR-0161).
//!
//! Irmão (`#[path]`) do [`super`] e do [`crate::sculpt3d::history_remesh`], e o
//! corte é de ASSUNTO: lá moram o voxel remesh e a retopologia **local** (o porte
//! BSD do Instant Meshes, ADR-0160); aqui a cadeia **global** — remalha isotrópica
//! · campo cruzado com decisão inteira · traçado dos patches · quantização Bi-MDF
//! · quadrangulação por patch.
//!
//! # Por que ela substitui o botão, com o número ao lado
//!
//! ⭐ **A grandeza que decide é a contagem de vértices irregulares**, e uma grade
//! numa esfera admite **oito**. Medido no corpus da bancada, mesmo alvo de
//! densidade:
//!
//! | malha | motor LOCAL | **cadeia GLOBAL** | oráculo |
//! |---|---|---|---|
//! | esfera 96×144 | 68,7 % quads · ~1 800 irreg. | **100 % · 14** | 100 % · ~7 |
//! | toro 64×32 | 64,9 % · ~2 200 | **100 % · 24** | 100 % · ~5 |
//! | esfera 98 k | 82,7 % · ~1 000 | **100 % · 21** | 100 % · ~9 |
//!
//! ⚠️ **E ela é LENTA em comparação**: o motor local responde em sub-segundo e
//! este leva ~8,5 s numa escultura de 98 k. É a troca declarada do ADR-0161 — o
//! local fica como *preview*, e é por isso que ele não foi removido.
//!
//! ⛔ **`PH2D_RETOPO_LEGACY=1` volta ao motor local**, e ele existe para bissecar:
//! um resultado mau só se atribui a esta cadeia depois de se ver o que o outro faz
//! com a mesma peça.

use super::remesh::QuadRemeshReport;
use super::{RemeshRefusal, Sculpt3dScene, SculptStroke, StrokeUndo};

/// **O ORÇAMENTO da busca da quantização.**
///
/// ⚠️ **Ele é um teto de ESPERA, e não de qualidade.** A busca do F4 é exacta e
/// prova o ótimo; o que este número limita é quanto tempo o artista fica à espera
/// antes de o gesto desistir com um nome. Medido no corpus (2026-08-21): as seis
/// malhas fecham **com prova** dentro dele, e a mais cara gasta 66 das 256
/// expansões. *Um teto que nenhuma fixtura toca é folga, não política.*
const QUANTIZE_BUDGET: (usize, usize) = (256, 512);

impl Sculpt3dScene {
    /// **A RETOPOLOGIA GLOBAL** — a cadeia inteira do ADR-0161. Devolve o
    /// [`QuadRemeshReport`].
    ///
    /// ⭐⭐ **Ela RECEBE o `adaptive` desde 2026-08-21, e ele tem consumidor.**
    ///
    /// ⛔ Antes disso não recebia, o painel avisava em voz alta que o knob não
    /// fazia nada nesta cadeia, e o artista lia *"o Follow Curvature não
    /// funciona"* — **que é a leitura correcta**: o motor por omissão é este.
    /// *Um aviso no terminal não é uma feature; é a confissão de um controlo
    /// morto.*
    ///
    /// A densidade entra pela [`ph2d_trace::PatchLayout::grade`], com o **mesmo**
    /// campo de tamanho que o motor local usa
    /// ([`ph2d_quadflow::ScaleField::adaptive`]) — então o knob significa a mesma
    /// coisa nos dois, que é o que o `detail` já fazia.
    ///
    /// ⚠️ **O `detail` atravessa pela MESMA lei do irmão local**
    /// (`ph2d_quadflow::edge_for_detail`), e isso é o que faz o slider significar
    /// a mesma coisa nos dois motores. *Duas leis para o mesmo knob é como dois
    /// botões passam a precisar de duas explicações.*
    ///
    /// ⚠️ **A mesma recusa de PILHA dos irmãos**, e pelo mesmo motivo: a saída é
    /// uma malha com outra contagem de vértices, e um nível de multires é uma
    /// subdivisão da base.
    pub(in crate::sculpt3d) fn quad_remesh_global(
        &mut self,
        detail: f32,
        adaptive: f32,
    ) -> Result<QuadRemeshReport, RemeshRefusal> {
        if self.level_count() != 1 {
            return Err(RemeshRefusal::MultiresStack);
        }
        let t = std::time::Instant::now();

        // ── F1. A remalha isotrópica, sobre uma CÓPIA.
        //
        // ⚠️ **Ela é o passe que torna a saída independente da entrada**, e sem
        // ela um cubo de oito vértices devolve malha vazia. A cópia existe porque
        // a peça na cena só é substituída no fim: uma recusa a meio da cadeia tem
        // de deixar o artista com o que ele tinha.
        let reference = self.mesh().clone();
        let mut work = reference.clone();
        ph2d_remesh_iso::remesh_isotropic(&mut work, ph2d_remesh_iso::ALPHA);
        work.triangulate();

        // ⭐⭐ **O ALVO SAI DA MALHA QUE O ARTISTA TROUXE**, e a alternativa foi
        // construída, medida e REJEITADA.
        //
        // ⛔ Derivá-lo da malha REMALHADA parecia mais coerente — é nela que o
        // layout vive — e **mata o slider**: medido em 2026-08-21 na esfera
        // amassada, os cinco pontos do curso deram `405 · 405 · 406 · 406 · 451`
        // quads, contra `405 · … · 1 336` com esta lei. A razão é que o F1 remalha
        // para `ALPHA × diagonal`, que é uma constante: o extremo fino do curso
        // ancorava-se num número que **não depende do que o artista pediu**.
        //
        // ⭐⭐ **E o PISO é o da cadeia GLOBAL, quatro vezes mais fino que o do
        // motor local** — ver [`ph2d_quadflow::GLOBAL_FLOOR_IN_INPUT_EDGES`]. O
        // `3,0` do local foi medido para a extração por retícula dele; esta cadeia
        // reamostra arcos e amostra dentro de um triângulo achatado, e a medição diz
        // que ela resolve **20 039 quads** onde antes parava em 1 336, com as dobras
        // em 0,03 % e a mediana em 1,03× o alvo.
        //
        // ⚠️ **E ela deixa um defeito ABERTO, com a causa medida:** quando o alvo é
        // maior que o comprimento típico de um arco, o piso `ArcSpec::min = 1` passa
        // a escolher o passo da grade e a densidade da saída é a do **layout**, não
        // a do slider. É o que se vê no 3.º clique seguido (mediana `0,16×`), e a
        // cura é grosseirar o layout — não trocar esta linha. Ver `PLAN.md`
        // §4-septdecies.
        let target = ph2d_quadflow::edge_for_detail_with(
            &reference,
            detail,
            ph2d_quadflow::GLOBAL_FLOOR_IN_INPUT_EDGES,
        );

        // ── F2. O campo cruzado com decisão inteira global.
        let dual = ph2d_crossfield::Dual::build(&work);

        // ⭐⭐ **DUAS TENTATIVAS, e a ordem é a lei do produto.** A primeira corre o
        // campo **alinhado ao relevo** (`ALIGN_WEIGHT`, hoje `0,03`), que é o que o
        // artista pediu três vezes seguidas — *"sem nenhuma obediência ao relevo"*.
        // A segunda, e só se a primeira **recusar**, corre o campo só-suavidade.
        //
        // ⚠️ **Ela existe porque o arredondamento é caótico**, não porque o
        // alinhamento seja duvidoso: uma célula da varredura do `ALIGN_WEIGHT`
        // move-se com a malha, e uma peça pode ser aquela em que o layout alinhado
        // não fecha. ⛔ **O liso é a REDE, nunca o produto** (CLAUDE.md §0.0): ele
        // só apara quando a outra não fecha, e o relatório diz qual correu — senão
        // uma regressão do alinhamento leria como sucesso.
        let attempt = |aligned: bool| {
            let policy = ph2d_crossfield::Rounding::default();
            // ⚠️ O `solve_miq` **é** o alinhado: o peso vive na crate do campo
            // (`ALIGN_WEIGHT`), e a rede pede explicitamente `0,0`.
            let (field, _) = if aligned {
                ph2d_crossfield::solve_miq(&dual)
            } else {
                ph2d_crossfield::solve_miq_aligned(&dual, policy, 0.0)
            };
            Self::global_chain(&work, &reference, &field, &dual, target, adaptive)
        };
        let (aligned, (out, r)) = match attempt(true) {
            Ok(ok) => (true, ok),
            // ⚠️ **Com o peso a ZERO as duas tentativas são a MESMA conta**, e
            // repeti-la seria pagar a cadeia inteira por nada. Hoje o
            // `ALIGN_WEIGHT` está a zero (o traçado perde asas num toro — ver o doc
            // dele), então esta guarda é o que impede a rede de existir enquanto não
            // há nada de que salvar.
            Err(e) if ph2d_crossfield::ALIGN_WEIGHT == 0.0 => return Err(e),
            // ⚠️ **A recusa que o artista vê é a da REDE**, e não a da primeira
            // tentativa: se nem o campo liso fecha, o problema não é o alinhamento e
            // apontar para ele mandaria o diagnóstico para o sítio errado.
            Err(_) => (false, attempt(false)?),
        };
        if out.faces().is_empty() {
            return Err(RemeshRefusal::TooCoarseToResolve);
        }
        let report = QuadRemeshReport {
            verts: r.verts,
            quads: r.quads,
            non_quads: r.non_quads,
            edge: target,
            ms: t.elapsed().as_secs_f64() * 1000.0,
            // ⚠️ **`boundary_edges` e não uma contagem de buracos por inundação.**
            // Uma aresta com uma face só é a assinatura da casca aberta, e é a
            // mesma grandeza que o irmão local reporta na coluna `holes`.
            holes: r.boundary_edges,
            irregular: r.irregular,
            edge_max_ratio: r.edge_max / target,
            edge_median_ratio: r.edge_median / target,
            // ⚠️ **Da malha ORIGINAL e não da de saída**: a caixa da saída pode ser
            // ligeiramente menor, e a régua tem de ser a mesma antes e depois.
            edge_max_span: r.edge_max / span(&reference),
            folded: r.folded,
            aligned,
        };
        let previous = core::mem::replace(self.mesh_mut().ok_or(RemeshRefusal::EmptyScene)?, out);
        self.record(StrokeUndo::Remeshed(Box::new(previous)));
        // A malha é OUTRA: o traço em voo fala de vértices que não existem mais.
        self.stroke = SculptStroke::default();
        self.mesh_rebuilt();
        Ok(report)
    }

    /// **F3 → F5 sobre um campo já resolvido** — a metade que a tentativa repete.
    ///
    /// ⚠️ **Ela recebe o campo e não o peso**, de propósito: assim a rede de
    /// recurso do [`Self::quad_remesh_global`] é *literalmente o mesmo caminho* com
    /// outro campo, e nenhuma divergência pode entrar entre as duas tentativas.
    fn global_chain(
        work: &ph2d_mesh::Mesh,
        reference: &ph2d_mesh::Mesh,
        field: &ph2d_crossfield::CrossField,
        dual: &ph2d_crossfield::Dual,
        target: f32,
        adaptive: f32,
    ) -> Result<(ph2d_mesh::Mesh, ph2d_quadfill::FillReport), RemeshRefusal> {
        // ── F3. As paredes e os patches.
        let mut layout = ph2d_trace::trace_patches(work, dual, field);

        // ⭐⭐ **O `Follow Curvature`, e ele entra AQUI e não na extração.** A
        // cadeia global não extrai de retícula nenhuma: quem decide a densidade é
        // quantos segmentos cada arco leva, e isso sai do `τ`. Graduar o `τ` por um
        // campo de tamanho faz o adensamento atravessar a quantização **e** a
        // amostragem de uma vez — ver [`ph2d_trace::PatchLayout::grade`].
        //
        // ⚠️ **O campo é o MESMO do motor local** (`ScaleField::adaptive`), então o
        // knob significa a mesma coisa nos dois. *Duas leis para o mesmo knob é
        // como dois botões passam a precisar de duas explicações.*
        //
        // ⚠️ **Ele é calculado sobre a `work` e não sobre a original**, e é
        // obrigatório: o `grade` indexa `size` pelos vértices do `arc_chain`, que
        // são índices da `work`. *A mesma família do parâmetro que servia dois
        // papéis, e aqui a assinatura já não a deixa exprimir.*
        if adaptive > 0.0 {
            // ⚠️ **Com o PISO desta cadeia**, e não com o do motor local: sem isso
            // o campo colapsa numa constante e o knob passa a **grosseirar** a peça
            // em vez de a adaptar. Ver [`ph2d_quadflow::ScaleField::adaptive_with`].
            let sizing = ph2d_quadflow::ScaleField::adaptive_with(
                work,
                target,
                adaptive,
                ph2d_quadflow::GLOBAL_FLOOR_IN_INPUT_EDGES,
            );
            let sizes: Vec<f32> = (0..sizing.len()).map(|v| sizing.at(v)).collect();
            layout.grade(work, &sizes, target);
        }
        let spec = layout.to_layout(target).map_err(RemeshRefusal::Layout)?;

        // ── F4. A quantização Bi-MDF.
        let budget = ph2d_quantize::Budget::new(QUANTIZE_BUDGET.0, QUANTIZE_BUDGET.1);
        let (quant, _) =
            ph2d_quantize::quantize_within(&spec, budget).map_err(RemeshRefusal::Quantize)?;

        // ── F5. A malha.
        //
        // ⚠️ **A superfície da reprojeção é a malha ORIGINAL e não a remalhada.**
        // Reprojetar sobre a saída do F1 seria alisar contra uma superfície que já
        // é uma aproximação — o erro das duas somaria, e a silhueta que o artista
        // esculpiu perderia o que o F1 já tinha arredondado.
        //
        // ⛔ **Este raciocínio está CERTO e foi ele que escreveu o bug**, porque a
        // função tinha um parâmetro só para dois papéis: a mesma malha servia de
        // superfície **e** de tabela de posições dos índices do layout. *Um
        // argumento correto para metade dos usos do mesmo argumento.*
        let (out, r) = ph2d_quadfill::fill(
            // ⭐ **`work` INDEXA, `reference` recebe.** As duas malhas são
            // diferentes aqui, e passá-las trocadas foi o defeito que destruiu o
            // produto em 2026-08-21 — com todos os gates verdes, porque o dano é
            // só geométrico. A assinatura de duas portas é a cura; ver o doc do
            // `ph2d_quadfill::fill`.
            work,
            reference,
            &layout,
            &quant,
            ph2d_quadfill::SMOOTHING_ROUNDS,
        )
        .map_err(RemeshRefusal::Fill)?;
        if out.faces().is_empty() {
            return Err(RemeshRefusal::TooCoarseToResolve);
        }

        Ok((out, r))
    }
}

/// **A DIAGONAL da caixa da peça** — o denominador da fração absoluta.
///
/// ⚠️ **A diagonal e não o maior lado**: a régua tem de ser a mesma que a foto do
/// defeito de 2026-08-21 usou (`2,01` numa esfera de raio `1,0`, diagonal `3,46`),
/// senão a margem de onze vezes registada no doc do `edge_max_span` deixa de ser
/// comparável.
fn span(mesh: &ph2d_mesh::Mesh) -> f32 {
    let b = mesh.bounds();
    let d = [
        b.max[0] - b.min[0],
        b.max[1] - b.min[1],
        b.max[2] - b.min[2],
    ];
    d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt()
}

/// **O motor LOCAL foi pedido explicitamente?** — `PH2D_RETOPO_LEGACY=1`.
///
/// ⚠️ **Ele existe para BISSECAR e não para escolher qualidade.** Um resultado mau
/// só se atribui à cadeia global depois de se ver o que o porte local faz com a
/// mesma peça — e sem esta porta a comparação exigiria recompilar.
#[must_use]
pub(in crate::sculpt3d) fn legacy_requested() -> bool {
    legacy_from(std::env::var("PH2D_RETOPO_LEGACY").ok().as_deref())
}

/// **A DECISÃO, sem tocar no ambiente** — a metade que se pode gatear.
///
/// ⚠️ **A separação não é estética: é o que torna esta lei testável.** Esta crate
/// proíbe `unsafe`, e desde a edição 2024 `std::env::set_var` é `unsafe` — um gate
/// que quisesse mexer na variável não compilava. Com a decisão pura, o gate lê a
/// lei e a leitura do ambiente fica numa linha que não tem o que decidir.
///
/// ⚠️ **O `"0"` DESLIGA**, e a regra não é decoração: sem ela um
/// `PH2D_RETOPO_LEGACY=0` esquecido numa sessão ligaria o motor antigo — o oposto
/// do que a linha diz. É a mesma lei do `PH2D_GPU_COOK=0` e do
/// `PH2D_FLIP_NEW_ENGINE=0`.
#[must_use]
fn legacy_from(value: Option<&str>) -> bool {
    value.is_some_and(|v| v != "0")
}

/// **Uma razão de aresta, para o log** — `?` quando o backend não a mede.
///
/// ⚠️ **`NAN` não é `0`, e a diferença importa.** O porte local não mede as
/// arestas da saída; escrever `0,0×` ali leria como uma grade perfeita, que é o
/// oposto do que ele entrega.
pub(in crate::sculpt3d) fn ratio(v: f32) -> String {
    if v.is_finite() {
        format!("{v:.2}x")
    } else {
        String::from("?")
    }
}

/// **A LINHA DE LOG da retopologia** — a única leitura que o smoke tem.
///
/// ⚠️ **Ela NOMEIA os buracos.** Enquanto o log não o fazia, a única forma de
/// detectar uma casca furada era o artista fotografar a tela — e foi o que
/// aconteceu três vezes em 2026-08-19. Um `0` aqui é a afirmação de que a peça
/// fechou.
///
/// ⭐ **E nomeia os IRREGULARES**, que é a grandeza que o pivô do ADR-0161 existiu
/// para derrubar e a que o artista de facto vê. Uma esfera admite **oito**; o motor
/// local não os conta e diz `?`, que é diferente de dizer zero.
///
/// ⭐⭐ **E as faces DOBRADAS** — a fenda escura fotografada em 2026-08-21, e a
/// única grandeza de defeito **geométrico** desta linha: uma peça pode sair com
/// 100 % de quads, casca fechada e a contagem certa de irregulares, e mesmo assim
/// estar cheia delas.
///
/// ⚠️ **Ela mora aqui e não no painel** por causa do teto de 600 LOC por arquivo da
/// shell (HR-18) — e o corte calhou no sítio certo: quem sabe o que cada coluna
/// significa é o módulo que a mediu.
pub(in crate::sculpt3d) fn retopo_line(r: &QuadRemeshReport) -> String {
    format!(
        "[sculpt3d] retopologia: {} vertices, {} quads e {} nao-quads ({:.1}% quads), \
         {} irregulares, aresta mediana {} do alvo e a mais longa {}, com quad de {:.4} \
         em {:.0} ms{}{}{}",
        r.verts,
        r.quads,
        r.non_quads,
        100.0 * r.quads as f64 / (r.quads + r.non_quads).max(1) as f64,
        if r.irregular == usize::MAX {
            String::from("?")
        } else {
            r.irregular.to_string()
        },
        ratio(r.edge_median_ratio),
        // ⚠️ **A MÁXIMA vai nas DUAS réguas, e a que decide é a
        // fração.** Ver `QuadRemeshReport::edge_max_span`: a razão
        // ao alvo triplica com o slider **sem defeito nenhum** (o
        // denominador é que encolhe), e é a fração da peça que
        // responde *"alguma coisa atravessa a peça?"*. Imprimir só
        // uma delas deixaria o leitor a comparar números que não são
        // comparáveis entre duas corridas do slider.
        if r.edge_max_span.is_finite() {
            format!(
                "{} do alvo = {:.1}% da peca",
                ratio(r.edge_max_ratio),
                100.0 * r.edge_max_span
            )
        } else {
            String::from("?")
        },
        r.edge,
        r.ms,
        if r.holes == 0 {
            String::from(" -- casca FECHADA")
        } else {
            format!(" -- ⚠️ {} BURACO(S) na casca", r.holes)
        },
        if r.folded == 0 {
            String::new()
        } else {
            format!(
                " -- ⚠️ {} face(s) DOBRADA(S) ({:.1}%)",
                r.folded,
                100.0 * r.folded as f64 / (r.quads + r.non_quads).max(1) as f64
            )
        },
        // ⭐⭐ **QUAL CAMPO correu.** A cadeia global tenta o campo
        // ALINHADO ao relevo e cai para o só-suavidade quando o
        // layout dele não fecha — e a queda é invisível em todas as
        // outras colunas desta linha. ⛔ Sem esta palavra, uma
        // regressão do alinhamento lê-se como uma corrida boa.
        if r.aligned {
            String::new()
        } else {
            String::from(" -- ⚠️ campo SO'-SUAVIDADE (o alinhado nao fechou)")
        }
    )
}

#[cfg(test)]
mod tests {
    /// ⭐ **A PORTA DE BISSECAÇÃO, e a decisão dela é pura de propósito.**
    ///
    /// ⚠️ **O gesto em si precisa de GPU** (a cena segura buffers de device), então
    /// um gate sobre ele é `skip` gracioso na máquina sem adapter — e *skip
    /// gracioso não é verde*. A **decisão** de qual motor correr não precisa de
    /// nada disso, e é ela que este gate pina.
    #[test]
    fn the_bisect_door_only_opens_when_it_is_asked_to() {
        for (value, want) in [
            // ⭐ O caso por omissão, e é o que decide qual motor o Enio recebe
            // sem configurar nada: a cadeia GLOBAL.
            (None, false),
            // ⚠️ E o `"0"` DESLIGA — sem esta linha, um `=0` esquecido ligaria o
            // motor antigo em silêncio.
            (Some("0"), false),
            (Some("1"), true),
            (Some("sim"), true),
            (Some(""), true),
        ] {
            assert_eq!(
                super::legacy_from(value),
                want,
                "PH2D_RETOPO_LEGACY={value:?} tinha de dar {want}"
            );
        }
    }
}
