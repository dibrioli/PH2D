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
                       features: bool|
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
        let guarded = |w: f32, features: bool| {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| attempt(w, features)))
                .unwrap_or(Err(RemeshRefusal::TooCoarseToResolve))
        };
        let (aligned, smooth) = if std::env::var("PH2D_RETOPO_SERIAL").as_deref() == Ok("1") {
            (
                guarded(ph2d_crossfield::ALIGN_WEIGHT, false),
                guarded(0.0, false),
            )
        } else {
            rayon::join(
                || guarded(ph2d_crossfield::ALIGN_WEIGHT, false),
                || guarded(0.0, false),
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
            && let Ok(f) = guarded(ph2d_crossfield::ALIGN_WEIGHT, true)
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

/// ⭐⭐⭐ **A FASE ZERO REMALHA PARA O ALVO, ou para o `ALPHA` fixo?**
///
/// ⛔⛔ **O report de 2026-08-29 (duas fotos, «o remesh amputou pontas»)**: a peça do artista
/// tem espinhos cujo **raio local** cai para `0,037`, e o F1 remalha com
/// `ALPHA × diagonal = 0,089` — **2,4× a espessura da ponta**. *A remalha isotrópica destrói
/// o espinho antes de a cadeia começar, e tudo a jusante trabalha sobre uma peça já
/// amputada.*
///
/// ⚠️ **A `ph2d-quadchain` levou esta correcção em 2026-08-25 e este caminho não** — o doc
/// do `phase_zero` diz-o com todas as letras: *«um parâmetro que metade da função ignora só
/// mente para o SEGUNDO chamador»*, e o segundo chamador é este botão.
///
/// # ⛔⛔⛔ E A HIPÓTESE FOI REFUTADA PELA MEDIÇÃO — por isso ela nasce DESLIGADA
///
/// Medido 2026-08-29 na fixtura de espinhos (`espinhos:6`), o mesmo alvo dos dois lados:
///
/// | | `Detail 0,50` | `Detail 0,85` |
/// |---|---|---|
/// | ⭐ `ALPHA` fixo (o que shipa) | `χ = 2` · `0` bordo · envies. `4,6°` · `21` dobras | `χ = 2` · `0` bordo · `4,0°` · `29` dobras |
/// | ⛔ segue o alvo | `χ = 1` · **`4` bordo** · `10,1°` · ⛔ **`123` dobras** | (não fechou a tempo) |
///
/// ⭐ **É a MESMA direcção que o varrimento de densidade da `ph2d-quadchain` deu** (§8-ter):
/// uma malha de trabalho mais fina não é mais informação — é onde a topologia se perde.
/// *A remalha grosseira é o filtro que faz o campo cruzado ver a forma e não o ruído.*
fn f1_follows_target() -> bool {
    std::env::var("PH2D_F1_TARGET").as_deref() == Ok("1")
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

/// ⭐⭐⭐ **O PASSO DA GRADE POR VÉRTICE — o `Follow Curvature` deixa de ser um knob morto.**
///
/// ⛔⛔ **Report do artista (2026-08-28):** *«as pontas finas, que deveriam ser relativamente
/// mais densas que as áreas lisas, têm menos densidade de faces e perdem detalhes»*. E a
/// medição confirma-o: na saída dele o expoente de `aresta ∼ curvatura^n` é **`−0,003`**
/// sobre uma faixa de curvatura de **`9,4×`** — *a grade é rigorosamente uniforme.*
///
/// ⚠️ **A lei já existia e não tinha consumidor nesta cadeia:**
/// [`ph2d_quadflow::ScaleField::adaptive`] dá o lado do quad por vértice a partir da
/// curvatura, com a gradação limitada pela [`ph2d_quadflow::MAX_ADAPTIVE_RATIO`] (a cerca
/// que impede a grade de rasgar em vez de transitar). Até hoje ela só era lida pelo motor
/// **local**; o de omissão fazia `let _ = adaptive;`.
///
/// # ⭐⭐ A NORMALIZAÇÃO, e por que ela não é opcional
///
/// O slider passou a pedir uma **contagem** ([`ph2d_quadflow::MAX_QUADS`]). Redistribuir os
/// quads sem renormalizar mudaria a contagem junto com a distribuição, e o slider voltava a
/// mentir. ⇒ o campo é escalado por `√(N_previsto / N_pedido)`, com
/// `N = Σ_face área/h²`. *A adaptação move os quads; ela não os cria.*
///
/// ⚠️ **Com `adaptive == 0` o campo é VAZIO** — a saída é a de sempre, e há gate.
///
/// # ⛔⛔⛔ MEDIDO E NÃO ADOPTADO — o passo no alvo do gradiente é LAVADO pela projecção
///
/// Medido 2026-08-28 na peça do artista (`Detail` fino, alvo `0,0324`, `13 289` quads):
///
/// | `Follow Curvature` | campo entregue | expoente da SAÍDA | apertada / chapada | quads | `>60°` |
/// |---|---|---|---|---|---|
/// | `0` | — | `+0,047` | `1,167` | `13 289` | `3` |
/// | `0,5` | `0,0243..0,0486` (`2×`) | `+0,024` | `1,133` | `11 963` | `3` |
/// | `1,0` | `0,0162..0,0648` (**`4×`**) | `+0,014` | `1,090` | ⚠️ `11 302` | ⛔ `6` |
///
/// ⭐⭐⭐ **Pede-se `400 %` e a saída move-se `7 %`** — e paga `15 %` da contagem e o dobro
/// das faces com canto pior que `60°`.
///
/// ⚠️ **O MECANISMO, e ele não é um defeito desta função:** o G3 resolve um mapa **escalar
/// por patch** cujo gradiente se aproxima do alvo `direcção / h`. Com `h` constante esse
/// campo alvo é integrável; **com `h` a variar ele deixa de o ser** (o rotacional deixa de
/// ser nulo), e a projecção de mínimos quadrados fica com a parte integrável — que é, quase
/// exactamente, o campo uniforme. *A adaptação não é ignorada: ela é projectada fora.*
///
/// ⭐ **A cura publicada tem nome e é outra maquinaria:** o factor de escala tem de ser
/// **conforme por construção** — resolver `Δ log h` contra a curvatura de Gauss e usar
/// `h = h₀·e^{−s}`, que é integrável por definição. É a família *«integer-grid maps with
/// prescribed sizing»*, e é uma wave com espec própria.
///
/// ⇒ **O `Follow Curvature` continua a nascer em `0`** e o caminho de omissão é
/// **byte-idêntico**. O que esta wave deixa é o **substrato** (o passo do mapa deixou de ser
/// um número — [`ph2d_gridmap::Step`]) e a medição que diz o que falta.
fn sizing_field(work: &Mesh, target: f32, adaptive: f32) -> Vec<f32> {
    if adaptive <= 0.0 {
        return Vec::new();
    }
    // ⛔⛔ **`adaptive_graded` e NÃO `adaptive_with`** — ver o doc dela. O piso da irmã é a
    // aresta média da malha de TRABALHO, que é a cerca do motor local; emprestada aqui ela
    // colapsa os dois extremos da banda no mesmo número e o campo sai constante ao bit.
    let field = ph2d_quadflow::ScaleField::adaptive_graded(work, target, adaptive);
    let mut per_vertex: Vec<f32> = (0..work.vert_count()).map(|v| field.at(v)).collect();
    // ⭐ A contagem que o campo prevê, sobre a mesma área que o alvo escalar prevê.
    let pos = work.positions();
    let (mut pred, mut area) = (0.0f64, 0.0f64);
    for f in work.faces() {
        let v = f.verts();
        for k in 1..v.len() - 1 {
            let (a, b, c) = (
                pos[v[0] as usize],
                pos[v[k] as usize],
                pos[v[k + 1] as usize],
            );
            let (u, w) = (
                [b[0] - a[0], b[1] - a[1], b[2] - a[2]],
                [c[0] - a[0], c[1] - a[1], c[2] - a[2]],
            );
            let n = [
                u[1].mul_add(w[2], -(u[2] * w[1])),
                u[2].mul_add(w[0], -(u[0] * w[2])),
                u[0].mul_add(w[1], -(u[1] * w[0])),
            ];
            let tri = f64::from(n[0].mul_add(n[0], n[1].mul_add(n[1], n[2] * n[2])).sqrt()) * 0.5;
            let h = f64::from(
                (per_vertex[v[0] as usize]
                    + per_vertex[v[k] as usize]
                    + per_vertex[v[k + 1] as usize])
                    / 3.0,
            )
            .max(1.0e-9);
            pred += tri / (h * h);
            area += tri;
        }
    }
    let want = area / f64::from(target.max(1.0e-9)).powi(2);
    // ⚠️ **A linha existe porque a 1.ª medição desta wave não distinguia «o campo é
    // constante» de «o campo não chegou»** — as três corridas do knob deram saída
    // byte-idêntica, e sem estes números não havia como dizer qual das duas era.
    {
        let mut v = per_vertex.clone();
        v.sort_by(f32::total_cmp);
        eprintln!(
            "[sculpt3d] densidade adaptativa {adaptive:.2}: passo {:.5}..{:.5} (mediana {:.5}, \
             alvo {target:.5}), previstos {pred:.0} para {want:.0} pedidos",
            v.first().copied().unwrap_or(0.0),
            v.last().copied().unwrap_or(0.0),
            v.get(v.len() / 2).copied().unwrap_or(0.0),
        );
    }
    if pred > 0.0 && want > 0.0 {
        #[allow(clippy::cast_possible_truncation)]
        let k = (pred / want).sqrt() as f32;
        if k.is_finite() && k > 0.0 {
            for h in &mut per_vertex {
                *h *= k;
            }
        }
    }
    per_vertex
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
/// ⭐⭐⭐ **A ORDEM DA ESCOLHA entre duas tentativas — `true` se `a` é PIOR que `b`.**
///
/// **Furos, depois faces `>60°`, depois o enviesamento mediano.** ⚠️ Os furos vêm primeiro
/// porque são o que o artista **vê** — foi a queixa dele três vezes seguidas
/// (*«furos nas pontas»*). *Uma ordem que pusesse o enviesamento à frente escolheria a peça
/// mais bonita com um buraco na ponta.*
///
/// ⛔⛔ **E «furo» conta as DUAS formas de a casca não fechar, desde 2026-08-28.** Até essa
/// data esta ordem via só as arestas de **bordo**; uma aresta **não-manifold** — três faces a
/// tocá-la — passava invisível, e o campo alinhado produz exactamente isso (medido:
/// `sculpt_hooked`, `1` não-manifold contra `0` do liso, com o alinhado a ganhar por
/// `0,2°` de enviesamento). ⚠️ **O artista vê o mesmo entalhe escuro nos dois casos** — e o
/// ficheiro que ele exportou em 28/08 tinha `19 786` quads impecáveis com **`2` arestas
/// não-manifold** num ponto só, três vértices de valência `2`–`3`. *Uma chave de desempate
/// que não vê metade do defeito escolhe a peça furada com toda a razão do mundo.*
///
/// ⚠️ **O desempate final é por `total_cmp`** e não por `<`: um `NaN` numa das medianas
/// tornaria a comparação não-reflexiva e a escolha dependeria da ordem dos argumentos.
fn worse(
    a_mesh: &Mesh,
    a_over60: usize,
    a_skew: f32,
    b_mesh: &Mesh,
    b_over60: usize,
    b_skew: f32,
) -> bool {
    let (a_holes, b_holes) = (open_edges(a_mesh), open_edges(b_mesh));
    if a_holes != b_holes {
        return a_holes > b_holes;
    }
    if a_over60 != b_over60 {
        return a_over60 > b_over60;
    }
    a_skew.total_cmp(&b_skew) == core::cmp::Ordering::Greater
}

fn boundary_edges(mesh: &Mesh) -> usize {
    edge_census(mesh).0
}

/// ⭐⭐⭐ **AS DUAS FORMAS DE A CASCA NÃO FECHAR, somadas** — a chave da frente de [`worse`].
///
/// ⚠️ **Uma aresta de bordo e uma não-manifold dão o MESMO report** (*«furos»*), e nenhuma
/// régua desta linha as somava: a escolha entre tentativas via só a primeira.
fn open_edges(mesh: &Mesh) -> usize {
    let (bordo, nm) = edge_census(mesh);
    bordo + nm
}

/// `(arestas de bordo, arestas não-manifold)` — uma face só, ou mais de duas.
fn edge_census(mesh: &Mesh) -> (usize, usize) {
    use std::collections::BTreeMap;
    let mut n: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            *n.entry(if a < b { (a, b) } else { (b, a) }).or_default() += 1;
        }
    }
    (
        n.values().filter(|c| **c == 1).count(),
        n.values().filter(|c| **c > 2).count(),
    )
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
#[path = "sculpt3d_history_retopo_extract_tests.rs"]
mod tests;
