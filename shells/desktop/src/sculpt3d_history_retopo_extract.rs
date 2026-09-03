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
//! # ⚠️ A COSTURA entra por ELIMINAÇÃO, e foi isso que fechou a casca (2026-08-24)
//!
//! O G3 **pesava** a costura; hoje ela é uma restrição eliminada
//! ([`ph2d_gridmap::round_welded`]) — o resíduo deixa de ser uma célula inteira e passa a ser
//! **zero**. Medido A/B em 5 peças fechadas: bordo `30`–`78` → **`0`**, `χ` `−4`..`−13` →
//! **`+2`**, e `3`–`4×` mais rápido. ⚠️ **A regressão que fica tem nome e cura publicada** (as
//! faces com canto pior que `60°` sobem; *local stiffening*, §5.4 do mesmo *paper*) e ficou
//! **fora daquela wave de propósito**: com dois mecanismos dentro, uma regressão de forma fica
//! sem dono.
//!
//! ⚠️ **As tabelas, as 5 recusas medidas e a pergunta devolvida ao dono vivem no handoff**
//! (`docs/3D/handoffs/HANDOFF_INTEGRACAO_line_seamelim_2026-08-24.md` e o `§8-bis` do
//! `…_quadextract_2026-08-24.md`) — *este doc é um roteador, e cada linha dele é paga por todo
//! agente que abrir o ficheiro.*
//!
//! ⚠️ **`PH2D_GRIDMAP_WELD=0` volta ao G3 penalizado**, dentro deste caminho — é a
//! forma de bissecar.

use super::remesh::QuadRemeshReport;
use super::{RemeshRefusal, Sculpt3dScene, SculptStroke, StrokeUndo};

/// **O ALVO da grade** — irmão pelo teto de LOC da shell (HR-18, 600), cortado por
/// RESPONSABILIDADE: ver [`target`].
/// ⚠️ **Visível ao módulo** desde 03/09: a sonda da porta tinha uma CÓPIA da fase zero, que envelheceu.
#[path = "sculpt3d_retopo_target.rs"]
pub(in crate::sculpt3d) mod target;

/// **AS RÉGUAS da tentativa** — irmão pela mesma razão: ver [`rulers`].
#[path = "sculpt3d_retopo_rulers.rs"]
mod rulers;

/// ⭐⭐ **UMA TENTATIVA, de ponta a ponta** — irmão pela mesma razão: ver [`one`].
#[path = "sculpt3d_retopo_one.rs"]
mod one;

/// ⭐⭐⭐ **A ESCOLHA entre duas candidatas** — irmão pela mesma razão: ver [`decide`].
#[path = "sculpt3d_retopo_decide.rs"]
mod decide;

// ⚠️ **Os `use` são o que mantém `super::worse` e `super::boundary_edges` a resolver**
// no `mod tests` irmão: ele chama tudo pelo prefixo, de propósito, e um nome trazido para
// cá por `use` continua a ser alcançável por `super::`.
// ⚠️ **`open_edges` SAIU daqui em 2026-08-30, e não é limpeza cosmética:** este ficheiro deixou
// de o chamar quando as duas condições de armar passaram pela porta [`still_broken`]. ⛔ O `mod
// tests` irmão chamava-o por `super::` e **parte** — a cura é apontá-lo ao dono
// (`super::rulers::open_edges`), nunca reter aqui um `use` morto nem calar o aviso. *Um import
// que só existe para um teste o alcançar é uma dependência invisível entre dois ficheiros.*
use decide::worse;
use rulers::{boundary_edges, edges, irregular, span, still_broken};

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

        // ⭐⭐⭐ **A FASE ZERO** — as duas decisões dela (seguir o alvo, e graduar a
        // densidade) vivem no [`target`], com as tabelas medidas ao lado.
        let work = target::phase_zero(&reference, target);

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
        // ⭐⭐⭐ **UMA TENTATIVA corre no IRMÃO** — ver [`one`]. ⚠️ A `attempt` fica aqui
        // como closure de propósito: é ela que o `catch_unwind` embrulha, e o que ela
        // fecha é exactamente o que uma tentativa pode ler sem escrever.
        let cx = one::Ctx {
            work: &work,
            reference: &reference,
            dual: &dual,
            target,
        };
        let attempt = |w: f32, features: bool, adaptive: f32, travel: f32, density: f32| {
            one::one(&cx, w, features, adaptive, travel, density)
        };

        // ⭐⭐ **O PREÇO, e a nota anterior MENTIA:** ela dizia *«a cadeia corre duas vezes»*
        // e escrevia `~9 s`. Hoje a ronda de abertura são **quatro** candidatas (o campo
        // alinhado e o liso, cada um com e sem a correcção de densidade) e as tentativas de
        // socorro acrescentam até mais quatro. Medido na peça do dono: `57`–`71 s` a
        // `Detail 0,75`–`1,00`. *Um comentário que descreve uma versão antiga do laço é lido
        // como se descrevesse esta.*
        //
        // ⛔ **A saída barata continua RECUSADA por medição:** sair cedo quando a primeira
        // candidata já é perfeita nas chaves da frente manteria o caso comum a `1×`, e
        // perderia a melhoria da mediana onde ela existe (`sculpt_eared`, `7,8° → 5,1°`, com
        // as chaves da frente a zero nas duas). *É uma troca de qualidade por espera, e a
        // escolha é do dono do produto* — o número está aqui para ele a poder fazer.
        //
        // ⚠️ **O custo é o MÁXIMO e não a soma** — as candidatas partilham só leituras
        // (`work`, `reference`, `dual`) e não escrevem nada em comum. Ver [`one::par`].
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
        let guarded = |w: f32, features: bool, adaptive: f32, travel: f32, density: f32| {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                attempt(w, features, adaptive, travel, density)
            }))
            // ⛔ **`Panicked` e não `TooCoarseToResolve`** — ver o doc da variante. A frase da
            // outra manda o artista **subdividir a escultura**, que é a cura de um problema
            // que ele não tem. *Um estouro e uma malha grossa demais não se parecem em nada,
            // e liam-se iguais.*
            .unwrap_or(Err(RemeshRefusal::Panicked))
        };
        // ⭐⭐⭐ **UMA CORRIDA são DUAS candidatas** — o campo alinhado ao relevo e o liso —, e
        // ⛔ nenhum dos dois ganha sempre: medido em 6 peças, o liso ganha em 5 e o alinhado
        // em 1. *A escolha não é uma constante: é uma medição por peça.* ⚠️ Escrita UMA vez
        // porque os quatro sítios que a pediam divergiam de cada vez que um argumento novo
        // entrava — e entraram três em dois dias (cerca de viagem, densidade, graduação).
        let corrida = |adaptive: f32, travel: f32, density: f32| {
            one::par(
                || {
                    guarded(
                        ph2d_crossfield::ALIGN_WEIGHT,
                        false,
                        adaptive,
                        travel,
                        density,
                    )
                },
                || guarded(0.0, false, adaptive, travel, density),
            )
        };
        let (aligned, smooth) = corrida(adaptive, ph2d_quadfill::EXTRACT_TRAVEL, 0.0);
        // ⭐⭐⭐ **AS DUAS CANDIDATAS COM A DENSIDADE NO CAMPO** — a cura do report de
        // 2026-09-01 (a tampa chata no bico), ver [`ph2d_crossfield::Dual::scale_by_density`].
        //
        // ⛔⛔⛔ **Ela é CANDIDATA e não interruptor, e a medição é a razão.** Na escultura do
        // dono, com a força exacta da teoria (`1`):
        //
        // **A tabela por densidade vive em [`one::FIELD_DENSITY`]**, ao lado do número. ⚠️ A
        // `Detail 0,75` ela COME uma ponta inteira — o espinho mede ali `1,2` quads de largura
        // e a grade não contrai tão depressa. *Serve onde a resolução chega para o espinho e
        // destrói onde não chega*, que é a forma de coisa que o [`worse`] existe para decidir
        // e ⛔ nunca a de uma constante.
        //
        // ⚠️ **A força é `1` e NÃO tem curso**: a correcção conforme é `α = −∗ds` exactamente.
        // Medido, `1,5` e `2` sobre-conduzem (a `1,5` o mapa vai a `105` dobras) — *escolher
        // uma força seria inventar um número onde a teoria já deu um.*
        let densas = corrida(adaptive, ph2d_quadfill::EXTRACT_TRAVEL, one::FIELD_DENSITY);
        let (relief_won, (out, e, _shift_frac_max, shape, dev, den)) = match (aligned, smooth) {
            (Ok(a), Ok(b)) => {
                if worse(
                    &a.0,
                    a.3.skew_over_60,
                    a.3.skew_p50,
                    a.4,
                    a.5,
                    &b.0,
                    b.3.skew_over_60,
                    b.3.skew_p50,
                    b.4,
                    b.5,
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

        // ⭐⭐⭐ **E AS COM DENSIDADE NÃO PODEM COMPRAR UMA PONTA COM UM FURO.**
        //
        // ⛔⛔⛔ **Medido a `Detail 0,75`:** a curada sai `0` furos e come **duas** pontas; a
        // de omissão traz `1` aresta não-manifold e come **uma**. Os furos são a chave da
        // FRENTE — por medição, e depois de três queixas do dono —, então a curada vencia
        // **com toda a razão do critério** e o artista perdia um espinho.
        //
        // ⚠️ **A cura não é reordenar o critério** — essa é decisão de produto que ninguém
        // mediu. É fazer valer a promessa que a correcção traz escrita: *ela só entra onde é
        // melhor*. ⚠️ `tips == 0` é «não medido» e nunca desqualifica.
        let sem_cura = dev;
        let so_se_nao_amputar = |c: Option<decide::Candidata>| {
            c.filter(|x| sem_cura.tips == 0 || x.4.tips == 0 || x.4.over <= sem_cura.over)
        };
        // ⭐ E as duas com densidade entram pela MESMA porta — elas só vencem onde são
        // melhores, que é a garantia que faz a cura poder ser agressiva sem risco.
        let (relief_won, (out, e, _shift_frac_max, shape, dev, den)) = decide::melhor(
            relief_won,
            (out, e, _shift_frac_max, shape, dev, den),
            so_se_nao_amputar(densas.0.ok()),
            true,
        );
        let (relief_won, (out, e, _shift_frac_max, shape, dev, den)) = decide::melhor(
            relief_won,
            (out, e, _shift_frac_max, shape, dev, den),
            so_se_nao_amputar(densas.1.ok()),
            false,
        );

        // ⭐⭐⭐ **A TERCEIRA TENTATIVA — e ela corre SÓ SE A SAÍDA AINDA ESTÁ PARTIDA.**
        //
        // ⚠️ **«Partida» são DUAS coisas desde 30/08** ([`still_broken`]): furo **ou** face
        // cruzada sobre si própria. *A condição sempre foi «a chave da frente do critério ainda
        // não está satisfeita» — o que mudou foi quantas chaves há à frente.*
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
        let feicoes = still_broken(&out, dev, den)
            .then(|| {
                guarded(
                    ph2d_crossfield::ALIGN_WEIGHT,
                    true,
                    adaptive,
                    ph2d_quadfill::EXTRACT_TRAVEL,
                    0.0,
                )
                .ok()
            })
            .flatten();
        let (relief_won, (out, e, _shift_frac_max, shape, dev, den)) = decide::melhor(
            relief_won,
            (out, e, _shift_frac_max, shape, dev, den),
            feicoes,
            relief_won,
        );

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
        // ⇒ **A cura tem a forma da terceira tentativa acima, e a mesma garantia:** se a saída
        // ainda está partida ([`still_broken`]: furo **ou** face cruzada) e o knob estava
        // ligado, corre-se mais uma vez **sem** o campo, e a decisão
        // passa pelo mesmo [`worse`]. *A adaptação não pode piorar a escolha; só pode não ser
        // escolhida.* ⭐ Ela é **de graça** quando o knob está desligado (a condição exige
        // `adaptive > 0`) e quando a saída já sai sã.
        // ⚠️⚠️ **A recaída corre a CORRIDA INTEIRA, não uma variante.** A 1.ª versão
        // desta guarda pediu **uma** candidata sem campo (a do `w` que tinha vencido) e a
        // peça continuou com `4` bordo: *a linha de base não é uma corrida, são duas — a
        // alinhada e a suave — e é o [`worse`] entre elas que dá a malha limpa.* Pedir só
        // metade do caminho de omissão devolve algo que não é o caminho de omissão.
        let uniforme = if adaptive > 0.0 && still_broken(&out, dev, den) {
            let (a, b) = corrida(0.0, ph2d_quadfill::EXTRACT_TRAVEL, 0.0);
            match (a, b) {
                (Ok(a), Ok(b)) => {
                    if worse(
                        &a.0,
                        a.3.skew_over_60,
                        a.3.skew_p50,
                        a.4,
                        a.5,
                        &b.0,
                        b.3.skew_over_60,
                        b.3.skew_p50,
                        b.4,
                        b.5,
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
        let uniforme_won = uniforme.as_ref().is_some_and(|(rw, _)| *rw);
        let (relief_won, (out, e, _shift_frac_max, shape, dev, den)) = decide::melhor(
            relief_won,
            (out, e, _shift_frac_max, shape, dev, den),
            uniforme.map(|(_, u)| u),
            uniforme_won,
        );

        // ⭐⭐⭐ **A QUINTA TENTATIVA — A CERCA DE VIAGEM DO ACABAMENTO.**
        //
        // ⛔⛔⛔ **A cerca EXISTE, o doc dela intitula-se *«a porta do PRODUTO»*, e o produto
        // passava `f32::INFINITY`** ([`ph2d_quadfill::EXTRACT_TRAVEL`]). O acabamento corre até
        // `1 200` rondas a deslizar cada vértice **ao longo** da superfície, e é assim que um
        // espinho encolhe: a componente *«escorregar ponta abaixo»* é tangencial, logo sobrevive
        // ao passo, e a reprojecção repõe o vértice **mais em baixo**.
        //
        // ⚠️ **E a aceitação do acabamento não podia apanhá-lo:** ela lê enviesamento e aspecto,
        // que é exactamente o que a relaxação sem cerca **melhora** enquanto desmancha a ponta.
        // *Uma ronda que come o espinho e endireita os quads é aceite por unanimidade.*
        //
        // ⭐⭐⭐ **MEDIDO na peça do dono, na configuração em que o defeito EXISTE**
        // (`_base_sculpt` recentrada, `Detail 0,75`, `Follow Curvature 1` — a que ele usa):
        //
        // | cerca | ponta (quads) | pontas acima da barra | enviesamento p50 | `>60°` |
        // |---|---|---|---|---|
        // | ⛔ `∞` (o que shipava) | **`2,39`** | `1` | `3,2°` | `2` |
        // | `2` | `1,69` | `1` | `3,8°` | `2` |
        // | `1` | `1,06` | `1` | `5,2°` | `4` |
        // | ⭐ **`0,5`** | **`0,67`** | **`0`** | `6,3°` | `5` |
        // | *(acabamento desligado)* | `0,74` | `0` | `9,4°` | `48` |
        //
        // ⭐⭐ **A cerca é estritamente melhor que o interruptor** — cura a ponta **mais** que
        // desligar o acabamento (`0,67` contra `0,74`) e paga **um quinto** das faces `>60°`.
        // *Desligar o acabamento nunca foi a alternativa certa; a cerca é.*
        //
        // ⚠️ **Ela é uma TENTATIVA e não uma constante nova, e a razão é medida:** a `Detail 1,00`
        // a mesma peça não tem ponta partida, e ali a cerca **piora** a forma de graça
        // (`4,22° → 6,7°`). ⇒ o preço só se paga onde há defeito, e quem decide continua a ser o
        // [`worse`] — *ela não pode piorar a escolha; só pode não ser escolhida.*
        //
        // ⛔ **A condição é a MESMA porta das outras duas** ([`still_broken`], que desde
        // 2026-09-01 conta a amputação). *Foi a falta dessa terceira condição que fazia esta peça
        // — topologia impecável, uma ponta comida — nunca armar tentativa nenhuma.*
        // ⚠️ **E ela corre DUAS VEZES — com e sem a correcção de densidade** — porque a
        // `Detail 0,75` uma salva as pontas e a outra a topologia (tabela no bloco acima):
        // *a saída não é reordenar o critério, é produzir a candidata que tem as duas coisas.*
        let apertadas = if still_broken(&out, dev, den) {
            let (a, b) = one::par(
                || {
                    guarded(
                        ph2d_crossfield::ALIGN_WEIGHT,
                        false,
                        adaptive,
                        ph2d_quadfill::EXTRACT_TRAVEL_RESCUE,
                        0.0,
                    )
                },
                || {
                    guarded(
                        ph2d_crossfield::ALIGN_WEIGHT,
                        false,
                        adaptive,
                        ph2d_quadfill::EXTRACT_TRAVEL_RESCUE,
                        one::FIELD_DENSITY,
                    )
                },
            );
            (a.ok(), b.ok())
        } else {
            (None, None)
        };
        let (relief_won, (out, e, _shift_frac_max, shape, dev, den)) = decide::melhor(
            relief_won,
            (out, e, _shift_frac_max, shape, dev, den),
            apertadas.0,
            true,
        );
        let (relief_won, (out, e, _shift_frac_max, shape, _dev, _den)) = decide::melhor(
            relief_won,
            (out, e, _shift_frac_max, shape, dev, den),
            so_se_nao_amputar(apertadas.1),
            false,
        );

        // ⭐⭐⭐ **O VETO — a peça não pode sair PARTIDA.** Ver [`rulers::shattered`].
        //
        // ⛔⛔⛔ **Report do artista com foto, 2026-08-30** (*«péssimo»*): um quad a flutuar solto
        // ao lado de uma ponta. ⚠️ **Reproduzido ao carregar no botão uma SEGUNDA vez** sobre a
        // saída da primeira (`Detail 0,85` + `Follow Curvature 1`, a configuração que a janela
        // anterior lhe recomendou): `2` peças — um pedaço solto de `22` faces —, `χ` de `2` para
        // `4`, e a ponta mais longa cortada de `−0,2 %` para **`−35,0 %`**. Um clique só sai
        // `1` peça e `χ = 2`; *é a re-entrada que parte.*
        //
        // ⛔ **O veto é ABSOLUTO e vem DEPOIS da escada**, e não é uma quinta candidata: o
        // [`worse`] sabe dizer qual das tentativas é a melhor e **nunca** compara com a malha que
        // o artista já tinha. Quando todas partem a peça, a melhor delas ainda é uma peça
        // partida — *e o mínimo do produto é o artista não perder a escultura porque a
        // retopologia falhou*, que é a mesma lei que pôs o `catch_unwind` acima.
        //
        // ⚠️ **Ele não fecha a porta ao caso legítimo:** é RELATIVO (`saiu > entrou`), então uma
        // cena que já tem dois objectos soltos continua a poder sair com dois.
        if let Some((pieces, was)) = rulers::shattered(&out, &reference) {
            return Err(RemeshRefusal::Shattered { pieces, was });
        }

        // ⭐⭐⭐ **A PONTA, uma a uma** — ver [`ph2d_quadfill::tip_survival`]. Contra a
        // `reference` (a escultura que entrou), **não** a `work`: o artista compara com o
        // que ele esculpiu, e a fase zero já é parte da cadeia que se está a julgar.
        let tips = ph2d_quadfill::tip_survival(&reference, &out);
        // ⭐⭐⭐ **E QUANTO DA ESCULTURA FICOU PARA TRÁS** — ver [`ph2d_quadfill::coverage`].
        //
        // ⚠️ **A direcção é `entrada → saída` e é a lei inteira:** a inversa — *«a malha nova
        // está pousada na escultura?»* — é a que os dois lados já medem, e ela dá **zero** sobre
        // uma peça com a ponta comida. *Eu medi a errada primeiro, no mesmo dia.*
        //
        // ⚠️ **Contra a `reference` pela mesma razão que a [`ph2d_quadfill::tip_survival`]:** o
        // artista compara com o que esculpiu, e a fase zero é parte da cadeia que se julga.
        let cover = ph2d_quadfill::coverage(&reference, &out);
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
            tips_cut: tips.cut,
            tips_total: tips.total,
            tips_worst_pct: tips.worst_pct,
            coverage_shell_p50: cover.shell_p50,
            coverage_shell_worst: cover.shell_worst,
            coverage_samples: cover.samples,
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
