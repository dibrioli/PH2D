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

use super::remesh::QuadRemeshReport;
use super::{RemeshRefusal, Sculpt3dScene, SculptStroke, StrokeUndo};

/// **O ALVO da grade** — irmão pelo teto de LOC da shell (HR-18, 600), cortado por
/// RESPONSABILIDADE: ver [`target`].
#[path = "sculpt3d_retopo_target.rs"]
mod target;

/// **AS RÉGUAS da tentativa** — irmão pela mesma razão: ver [`rulers`].
#[path = "sculpt3d_retopo_rulers.rs"]
mod rulers;

// ⚠️ **Os `use` são o que mantém `super::worse` e `super::boundary_edges` a resolver**
// no `mod tests` irmão: ele chama tudo pelo prefixo, de propósito, e um nome trazido para
// cá por `use` continua a ser alcançável por `super::`.
use rulers::{boundary_edges, edges, irregular, open_edges, span, worse};
use target::{f1_follows_target, sizing_field};

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
        let t = std::time::Instant::now();

        // ── F1. A fase zero. ⛔ **Não a salte, e não meça sem ela:** com a
        // triangulação crua a mesma cadeia dá o dobro do enviesamento.
        //
        // ⭐⭐⭐ **MAS ANTES: reparar as MORDIDAS da entrada** — ver
        // [`ph2d_quadextract::repair_doublets`]. ⛔⛔ A saída que o artista exportou em
        // 2026-08-29 tinha `19` doublets, todos em pontas finas, e ao voltar a entrar ela faz
        // a fase zero devolver `χ = 6` com aresta não-manifold — donde o estouro do
        // `ph2d-gridmap` (`assembly.rs:193`). *Fechar só o lado da saída deixaria toda peça
        // já gravada a partir este botão para sempre.*
        //
        // ⚠️ **A reparação é EXACTA e não move um vértice:** funde as duas faces que prendem
        // o vértice numa só (`V−1`, `E−2`, `F−1`, `χ` invariante).
        let mut reference = self.mesh().clone();
        let bitten = ph2d_quadextract::repair_doublets(&mut reference).unwrap_or(0);
        let reference = reference;

        // ⭐ **O alvo sai da malha que o artista trouxe** — a mesma lei do irmão, e a
        // alternativa (derivá-lo da remalhada) foi medida e mata o slider.
        // ⭐⭐⭐ **O ALVO É UMA CONTAGEM, e a contagem sai da ÁREA** — ver
        // [`ph2d_quadflow::MAX_QUADS`]. ⛔⛔ Até 2026-08-28 ele saía de
        // `edge_for_detail_with`, cujo piso é a **aresta média da malha da cena**: depois
        // de uma retopologia essa malha é a SAÍDA, então o mesmo ponto do slider pedia
        // quads cada vez maiores. Medido na peça do artista com o `Detail` parado em
        // `0,50`: `19 786 -> 1 747 -> 520 -> 281` quads em três apertos, `−98,6 %`.
        // *Foi isto que ele fotografou e chamou de «pontas com baixa resolução».*
        let target = ph2d_quadflow::edge_for_detail_by_count(&reference, detail);

        // ⭐⭐⭐ **A FASE ZERO SEGUE O ALVO** — ver [`f1_follows_target`].
        let work = if f1_follows_target() {
            ph2d_quadchain::phase_zero(&reference, target)
        } else {
            let mut w = reference.clone();
            ph2d_remesh_iso::remesh_isotropic(&mut w, ph2d_remesh_iso::ALPHA);
            w.triangulate();
            w
        };

        // ── F2 + F3 + G1 + G2.
        let mut dual = ph2d_crossfield::Dual::build(&work);

        // ⭐⭐⭐ **O BORDO É UMA LINHA DE FEIÇÃO** — a mesma lei do caminho irmão
        // (`retopo_global.rs`), e a tabela vive lá. ⚠️ **Este caminho constrói o PRÓPRIO
        // campo**, então herdar a lei do irmão não é automático: em 2026-08-26 ela foi
        // ligada no `retopo_global` e este ficheiro ficou sem ela por meia hora. *Dois
        // caminhos que constroem o mesmo objecto precisam da mesma lei escrita duas vezes,
        // ou de uma porta só — e a porta ainda não existe.*
        // ⚠️ Inerte em peça fechada por construção; `PH2D_BOUNDARY_FEATURE=0` desliga.
        if std::env::var("PH2D_BOUNDARY_FEATURE").as_deref() != Ok("0") {
            let (edges, _loops) = ph2d_mesh::boundary_feature_edges(&work);
            dual.constrain(&work, &edges);
        }
        // ⭐⭐ **AS LINHAS DE FEIÇÃO** — obra B da `SPEC_restricoes_por_eliminacao.md` §3, o
        // 1.º dos três consumidores (o campo). ⛔ **Nasce DESLIGADA**, e a medição é a razão:
        // na peça do artista ela leva as arestas de bordo — que é o que um buraco é — de
        // `14` para `6` e o enviesamento de `7,4°` para `6,7°` (dentro da barra do oráculo),
        // ⛔ **e triplica as faces com canto pior que 60°** (`4` para `12`). ⚠️ *É a MESMA
        // regressão que a obra A deixou, com a mesma cura publicada por nomear (`local
        // stiffening`, §5.4).* ⇒ o artista pode vê-la; o produto ainda não a assume.
        // ⇒ desde 2026-08-26 ela deixou de ser um interruptor e passou a ser um **MODO de
        // tentativa**: a 3.ª candidata, corrida **só quando as duas primeiras ainda deixam
        // furo**. `PH2D_FEATURE_EDGES=1` força-a na primeira.
        let dual = dual;
        let with_features = |d: &ph2d_crossfield::Dual| {
            let mut d = d.clone();
            // ⚠️ **O `h` é o `target`**, e não uma medida da malha: a lei da feição mede-se
            // em múltiplos do **passo alvo da grade**, que é exactamente o número que o G3
            // recebe três blocos abaixo. *Medi-lo outra vez daria duas respostas à mesma
            // pergunta, e a que envelhece é a que ninguém vê.*
            let (fd, _) =
                ph2d_mesh::feature_dirs(&work, target, ph2d_mesh::FeatureOptions::default());
            let (fe, _) = ph2d_mesh::feature_edges(&work, &fd, ph2d_mesh::FEATURE_EDGE_MIN_COS);
            d.constrain(&work, &fe);
            d
        };
        // ⭐⭐⭐ **AS DUAS CORREM, E A MEDIÇÃO ESCOLHE — o alinhamento ao relevo deixou de
        // ser uma aposta única.**
        //
        // ⛔⛔ Até 2026-08-26 este caminho corria **só** o campo alinhado
        // ([`ph2d_crossfield::ALIGN_WEIGHT`], `0,03`), e o irmão dele caía para o liso apenas
        // quando o alinhado **RECUSAVA**. ⚠️ *Uma rede que dispara na recusa não apanha o
        // layout que fecha e sai péssimo* — e foi exactamente isso que a `sculpt_004` do
        // artista mostrou (a orelha, a única ponta cuja malha de entrada era complicada):
        //
        // | peça | alinhado (`0,03`) | liso (`0,0`) |
        // |---|---|---|
        // | ⛔ `sculpt_004` | `23,5°` · `43` faces `>60°` · `14` bordo | ⭐ **`7,8°` · `3` · `4`** |
        // | `sculpt_eared` | `7,8°` | ⭐ `5,1°` |
        // | `sculpt_hooked` | `6,6°` · `1` não-manifold | ⭐ `6,4°` · `0` |
        // | `sculpt_ridged` | p99 `31,4°` | ⭐ p99 `22,0°` |
        // | `sculpt_t002` | `6,7°` | ⭐ `5,5°` |
        // | ⭐ `sculpt_t003` | **`6,6°` · `4` bordo** | `7,9°` · `6` bordo |
        //
        // ⭐⭐ **O liso ganha em 5 de 6 e o alinhado em 1** — e nenhum ganha sempre. ⇒ *a
        // escolha não é uma constante: é uma medição por peça.*
        //
        // ⚠️ **E o termo do relevo não entrega o que foi acrescentado para entregar:** medido
        // no mesmo dia com a régua `follows_relief`, ele compra **`0,4°`** (`22,1° → 21,7°`,
        // ambos ao lado dos `22,5°` que significam «não olhou»). *O número foi escolhido em
        // Agosto pelo campo do oráculo, quando esta régua não existia.*
        //
        // ⚠️ **A ORDEM do critério é: furos, depois faces `>60°`, depois o enviesamento
        // mediano.** Os furos vêm primeiro porque são o que o artista **vê** — foi a queixa
        // dele três vezes seguidas.
        let attempt = |w: f32,
                       features: bool,
                       adaptive: f32|
         -> Result<
            (
                ph2d_mesh::Mesh,
                ph2d_quadextract::ExtractReport,
                f32,
                ph2d_quadfill::QuadShape,
            ),
            RemeshRefusal,
        > {
            let owned;
            let dual: &ph2d_crossfield::Dual =
                if features || super::retopo_extract::features_requested() {
                    owned = with_features(&dual);
                    &owned
                } else {
                    &dual
                };
            let (field, _) = if (w - ph2d_crossfield::ALIGN_WEIGHT).abs() < f32::EPSILON {
                ph2d_crossfield::solve_miq(dual)
            } else {
                ph2d_crossfield::solve_miq_aligned(dual, ph2d_crossfield::Rounding::default(), w)
            };
            let layout = ph2d_trace::trace_patches(&work, dual, &field);
            let (cut, _) = ph2d_gridmap::cut_along_patches(&work, &layout);
            let (combed, _) = ph2d_gridmap::comb_patches(&work, &layout, &cut);

            // ⭐ As singularidades saem do CAMPO — o índice por-vértice é um facto dele, e
            // pedir à `ph2d-gridmap` que o re-derive seria reconstruir o que já existe.
            let singular: Vec<u32> = ph2d_crossfield::vertex_index(&work, dual, &field)
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
            // ⭐⭐⭐ **A DENSIDADE SEGUE A FORMA** — ver [`sizing_field`]. Com
            // `adaptive == 0` o campo é constante e o passo é o escalar de sempre.
            let sizing = sizing_field(&work, target, adaptive);
            let step = ph2d_gridmap::Step {
                h: target,
                per_vertex: &sizing,
            };
            let (map, round) = if welded {
                ph2d_gridmap::round_welded(&work, &cut, &combed, step, opts, &singular)
            } else {
                ph2d_gridmap::round_to_integers(&work, &cut, &combed, step, opts, &singular)
            };

            // ── A extracção das isolinhas.
            let (tris, uv) = ph2d_gridmap::corner_map(&cut, &map);
            let cm = ph2d_quadextract::CornerMap {
                pos: work.positions(),
                tris: &tris,
                uv: &uv,
            };
            let (mut out, e) =
                ph2d_quadextract::extract(&cm, None).map_err(RemeshRefusal::Extract)?;
            if out.faces().is_empty() {
                return Err(RemeshRefusal::TooCoarseToResolve);
            }

            // ⭐⭐⭐ **O ACABAMENTO — e este caminho não o tinha.**
            //
            // ⛔⛔ O irmão dele, o `ph2d_quadfill::fill`, corre [`ph2d_quadfill::SMOOTHING_ROUNDS`]
            // passos de Laplaciano tangencial com reprojeção **desde sempre**; a extracção
            // entregava a malha **crua**. *Dois caminhos para o mesmo botão, e só um com
            // acabamento.*
            //
            // ⚠️ **A superfície é a `reference` — a escultura — e nunca a `work`.** É a mesma lei
            // que o doc do `fill` escreve com o defeito de 2026-08-21 ao lado: reprojectar sobre a
            // remalhada somaria os dois erros.
            //
            // Medido 2026-08-26 na `sculpt_t003` do artista, na densidade fina:
            //
            // | régua | cru | **com acabamento** |
            // |---|---|---|
            // | distância à ESCULTURA p95 | `0,106 %` | ⭐ **`0,000 %`** |
            // | enviesamento p99 · `>60°` | `39,3°` · `18` | ⭐ **`29,1°` · `1`** |
            // | aspecto p99 · `>4×` | `2,05` · `7` | ⭐ **`1,63` · `0`** |
            //
            // ⚠️ **Ele NÃO alisa a superfície, e isso é o achado:** a rugosidade fica onde estava
            // (`14,2° ⇒ 14,3°`) porque a reprojecção repõe os vértices na peça. *A aspereza que o
            // artista vê é a da escultura dele — a grade fina RESOLVE-A, a cadeia não a inventa.*
            // ⭐ **O preço, medido:** `425 ms` sobre `7 750` quads numa cadeia de `7,0 s` —
            // **6 %**, na densidade mais fina medida (melhor de 3, `6 979` contra `7 404 ms`).
            // ⚠️ `PH2D_EXTRACT_FINISH=0` desliga, para bissecar.
            //
            // ⭐⭐⭐ **E DESDE 2026-08-28 O ACABAMENTO É UMA PORTA, não uma linha aqui** — a
            // mesma que a `ph2d-quadchain` chama, porque *duas ordens para o mesmo botão com
            // acabamentos diferentes é uma lei que gate nenhum defende*. Ela corre o
            // Laplaciano como **ronda zero** e depois o ajuste de quadrado **alinhado ao
            // relevo**, e entrega a MELHOR ronda — ver `ph2d_quadfill::finish_extract`.
            //
            // ⚠️ **O ganho, medido na densidade que este botão usa** (`sculpt_eared`, 524
            // quads): enviesamento mediano `10,4° → 3,8°`, aspecto `1,14 → 1,07`, faces
            // péssimas `0 → 0`, e o preço `21 ms → ~400 ms` numa cadeia de segundos.
            if std::env::var("PH2D_EXTRACT_FINISH").as_deref() != Ok("0") {
                ph2d_quadfill::finish_extracted(&mut out, &reference);
            }
            let out = out;

            let shape = ph2d_quadfill::quad_shape(&out);
            Ok((out, e, round.shift_frac_max, shape))
        };

        // ⭐ **O PREÇO, MEDIDO:** a cadeia corre duas vezes. Na `sculpt_004` uma passagem
        // custa **`4 475 ms`** (melhor de 2), logo o botão passa de ~4,5 s a **~9 s**. O F1 é
        // partilhado; o que duplica é campo + traçado + mapa + extracção.
        //
        // ⛔ **A saída barata foi considerada e NÃO tomada:** sair cedo quando a primeira
        // tentativa já é perfeita nas duas chaves da frente (`0` furos e `0` faces `>60°`)
        // manteria o caso comum a `1×` — mas mede-se que ela **perderia** a melhoria da
        // mediana onde ela existe (na `sculpt_eared`, `7,8° → 5,1°`, com as duas chaves da
        // frente a zero nas duas tentativas). ⇒ *é uma troca de qualidade por espera, e a
        // escolha é do dono do produto* — o número está aqui para ele a poder fazer.
        // ⭐⭐⭐ **AS DUAS EM PARALELO — o custo passa a ser o MÁXIMO, não a soma.**
        //
        // ⛔ A nota acima ainda vale sobre a saída barata (ela perderia qualidade), e é por
        // isso que a cura **não** é escolher uma: as duas tentativas são **independentes** —
        // partilham só leituras (`work`, `reference`, `layout`) e não escrevem nada em comum.
        // *Não havia troca nenhuma a fazer: havia duas coisas em série que podiam estar lado
        // a lado.*
        //
        // ⚠️ **`rayon::join` e não threads à mão:** ele é a lib sancionada da casa
        // (SKILL_Stack §919) e o *work-stealing* dele compõe com o paralelismo que já existe
        // **dentro** de cada passagem (o acabamento), em vez de competir com ele.
        //
        // ⚠️ `PH2D_RETOPO_SERIAL=1` volta a correr as duas em série — é o A/B, e é o que
        // permite dizer quanto o paralelo vale nesta máquina em vez de o supor.
        // ⭐⭐⭐ **UMA TENTATIVA QUE ESTOURA É UMA TENTATIVA QUE PERDE, e não o fim da peça.**
        //
        // ⛔⛔ **Reproduzido em 2026-08-29 com a peça do artista** (a bola de espinhos): a
        // fase zero devolve uma malha de trabalho **não-manifold** (`χ = 6`, uma aresta com
        // três faces) porque a remalha isotrópica *belisca* um espinho mais fino que a aresta
        // alvo — e a jusante o `ph2d-gridmap` entra em `index out of bounds`
        // (`assembly.rs:193`). *É o mesmo estouro que este repo tinha SEM ENDEREÇO desde
        // 26/08.*
        //
        // ⚠️ **A cura de fundo é a fase zero preservar a topologia que recebe**, e ela é
        // outra wave. O que esta porta tem de garantir é o mínimo do produto: **o artista
        // não perde a escultura porque a retopologia falhou.** A `ph2d-quadchain` já tinha
        // esta rede (`Verdict::Panicked`) e este caminho não — *duas portas para o mesmo
        // botão, e só uma sabia não cair.*
        //
        // ⚠️ **`AssertUnwindSafe` é honesto aqui:** a `attempt` não escreve em nada partilhado
        // — ela lê `work`/`reference`/`dual` e devolve uma malha nova.
        let guarded = |w: f32, features: bool, adaptive: f32| {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                attempt(w, features, adaptive)
            }))
            .unwrap_or(Err(RemeshRefusal::TooCoarseToResolve))
        };
        let (aligned, smooth) = if std::env::var("PH2D_RETOPO_SERIAL").as_deref() == Ok("1") {
            (
                guarded(ph2d_crossfield::ALIGN_WEIGHT, false, adaptive),
                guarded(0.0, false, adaptive),
            )
        } else {
            rayon::join(
                || guarded(ph2d_crossfield::ALIGN_WEIGHT, false, adaptive),
                || guarded(0.0, false, adaptive),
            )
        };
        let (relief_won, (out, e, _shift_frac_max, shape)) = match (aligned, smooth) {
            (Ok(a), Ok(b)) => {
                if worse(
                    &a.0,
                    a.3.skew_over_60,
                    a.3.skew_p50,
                    &b.0,
                    b.3.skew_over_60,
                    b.3.skew_p50,
                ) {
                    (false, b)
                } else {
                    (true, a)
                }
            }
            (Ok(a), Err(_)) => (true, a),
            (Err(_), Ok(b)) => (false, b),
            (Err(e), Err(_)) => return Err(e),
        };

        // ⭐⭐⭐ **A TERCEIRA TENTATIVA — e ela corre SÓ SE AINDA HÁ FURO.**
        //
        // ⛔⛔ As linhas de feição por curvatura **custam bordo** na maioria das peças
        // (`sculpt_t001` `4 → 14`, `sculpt_t002` `14 → 18`, `sculpt_hooked` `0 → 4`), e é por
        // isso que elas não são um default. ⚠️ *Mas na `sculpt_004` do artista elas levam o
        // bordo a **ZERO*** (`4 → 0`, com o enviesamento em `9,6°`).
        //
        // ⇒ A condição **não é um limiar escolhido à mão**: é *«a chave da frente do critério
        // ainda não está satisfeita»*. Uma peça que já fecha não paga nada; uma que ainda tem
        // furo paga mais uma passagem — que é exactamente onde a queixa do artista vive.
        //
        // ⚠️ **E ela é segura por CONSTRUÇÃO:** entra pelo mesmo [`worse`], logo só vence
        // onde é melhor. *A terceira candidata não pode piorar a escolha; só pode não ser
        // escolhida.*
        let (relief_won, (out, e, _shift_frac_max, shape)) = if open_edges(&out) > 0
            && let Ok(f) = guarded(ph2d_crossfield::ALIGN_WEIGHT, true, adaptive)
            && worse(
                &out,
                shape.skew_over_60,
                shape.skew_p50,
                &f.0,
                f.3.skew_over_60,
                f.3.skew_p50,
            ) {
            (relief_won, f)
        } else {
            (relief_won, (out, e, _shift_frac_max, shape))
        };

        // ⭐⭐⭐ **A QUARTA TENTATIVA — o campo adaptativo PERDE se abrir a malha.**
        //
        // ⛔⛔⛔ **Report do artista, 2026-08-30, com foto: «praticamente uma regressão».** E
        // era: no `Detail` de FÁBRICA (`0,50`) o `Follow Curvature = 1` levava a peça dele de
        // `χ = 2 · 0 bordo` para **`χ = 1 · 4 bordo`** — furos onde não havia.
        //
        // ⚠️⚠️ **E a wave que o introduziu mediu-o a `Detail 0,85`, onde fica limpo.** *Afinar
        // e validar num ponto do slider que não é o de fábrica é medir a configuração que
        // ninguém usa* — a fixtura sintética de espinhos já tinha avisado (bordo `0 → 4`) e a
        // leitura foi «na peça dele fica limpo», que era verdade só naquele ponto.
        //
        // ⇒ **A cura tem a forma da terceira tentativa acima, e a mesma garantia:** se ainda
        // há furo e o knob estava ligado, corre-se mais uma vez **sem** o campo, e a decisão
        // passa pelo mesmo [`worse`]. *A adaptação não pode piorar a escolha; só pode não ser
        // escolhida.* ⭐ Ela é **de graça** quando o knob está desligado (a condição exige
        // `adaptive > 0`) e quando a saída já fecha.
        // ⚠️⚠️ **A recaída corre a CORRIDA INTEIRA, não uma variante.** A 1.ª versão
        // desta guarda pediu **uma** candidata sem campo (a do `w` que tinha vencido) e a
        // peça continuou com `4` bordo: *a linha de base não é uma corrida, são duas — a
        // alinhada e a suave — e é o [`worse`] entre elas que dá a malha limpa.* Pedir só
        // metade do caminho de omissão devolve algo que não é o caminho de omissão.
        let uniforme = if adaptive > 0.0 && open_edges(&out) > 0 {
            let (a, b) = if std::env::var("PH2D_RETOPO_SERIAL").as_deref() == Ok("1") {
                (
                    guarded(ph2d_crossfield::ALIGN_WEIGHT, false, 0.0),
                    guarded(0.0, false, 0.0),
                )
            } else {
                rayon::join(
                    || guarded(ph2d_crossfield::ALIGN_WEIGHT, false, 0.0),
                    || guarded(0.0, false, 0.0),
                )
            };
            match (a, b) {
                (Ok(a), Ok(b)) => {
                    if worse(
                        &a.0,
                        a.3.skew_over_60,
                        a.3.skew_p50,
                        &b.0,
                        b.3.skew_over_60,
                        b.3.skew_p50,
                    ) {
                        Some((false, b))
                    } else {
                        Some((true, a))
                    }
                }
                (Ok(a), Err(_)) => Some((true, a)),
                (Err(_), Ok(b)) => Some((false, b)),
                (Err(_), Err(_)) => None,
            }
        } else {
            None
        };
        let (relief_won, (out, e, _shift_frac_max, shape)) = if let Some((rw, u)) = uniforme
            && worse(
                &out,
                shape.skew_over_60,
                shape.skew_p50,
                &u.0,
                u.3.skew_over_60,
                u.3.skew_p50,
            ) {
            (rw, u)
        } else {
            (relief_won, (out, e, _shift_frac_max, shape))
        };

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
            mirrored: e.mirrored_cells,
            // ⚠️ **A soma das DUAS metades**: as que a extracção não emitiu e as que a
            // reparação da entrada dissolveu. *O artista vê uma mordida, não duas fases.*
            doublets: e.doublets + bitten,
            // ⭐⭐⭐ **`aligned` diz QUAL CAMPO produziu esta malha** — é o sentido que o
            // `retopo_line` lhe dá. ⛔ Até 2026-08-26 este caminho punha aqui a
            // **exactidão do arredondamento** (`shift_frac_max == 0.0`), que é outra
            // grandeza: o log imprimia *«o alinhado nao fechou»* sempre que uma translação
            // saísse fraccionária. *Dois sentidos no mesmo campo, e o texto do log
            // escolhido pelo primeiro.*
            aligned: relief_won,
            measured: true,
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

/// ⭐⭐ **AS LINHAS DE FEIÇÃO entram no campo?** — obra B, e ⛔ **`false` por omissão**.
///
/// ⚠️ **Ela é o contrário do [`extract_requested`], e a diferença é a medição.** Aquele
/// virou o default porque o motor novo bate o antigo em tudo o que o artista nomeou; esta
/// fica desligada porque **compra e vende**: na peça dele as arestas de bordo caem de `14`
/// para `6` e o enviesamento entra na barra do oráculo, ⛔ e as faces com canto pior que
/// `60°` vão de `4` para `12`. *Uma troca com dois sinais é decisão do dono do produto, e
/// ele decide vendo — não lendo uma tabela.*
pub(in crate::sculpt3d) fn features_requested() -> bool {
    std::env::var("PH2D_FEATURE_EDGES").as_deref() == Ok("1")
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

#[cfg(test)]
#[path = "sculpt3d_history_retopo_extract_tests.rs"]
mod tests;
