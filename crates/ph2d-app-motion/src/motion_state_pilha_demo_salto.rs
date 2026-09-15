//! ⭐⭐⭐ **O SALTO DEPOIS DE ASSENTAR** — o 5.º report do dono sobre a `=114` (2026-09-15:
//! *«melhorou os tremores mas a física ficou imprecisa. Essas 4 caixas apontadas depois de se
//! assentar corretamente uma sobre a outra, dão um salto e ficam nessa angulação irreal»*, com foto
//! e quatro setas no MEIO do monte).
//!
//! ⛔⛔ **As QUATRO réguas que esta cena já tinha são cegas a isto, e a cegueira é estrutural —
//! cada uma por um motivo diferente:**
//!
//! | régua | o que mede | por que não vê um salto |
//! |---|---|---|
//! | [`super::tremor::balanco_angular`] | `|Δrot|` **MEDIANO** por tique | *uma mediana é cega a um evento raro por construção* — 1 tique em 174 não a move |
//! | [`super::tremor::giro_liquido`] | `Σ Δrot` na janela | um salto de `20°` e uma deriva de `20°` leem-se **iguais** |
//! | `y` final / `vizinho_mediano` | uma FOTOGRAFIA no fim | a peça já saltou e já parou — a foto é do depois |
//! | as três acima | **os 0,9 s do PRIMEIRO ciclo** | o salto medido cai no assentar da **segunda** queda |
//!
//! ⇒ *a grandeza deste report é um EXTREMO num intervalo curto, e as quatro que existiam são
//! médias, somas ou fotografias.* É a quinta vez que esta linha paga a mesma forma (o `edge_max`
//! global cego ao quad fino, o `χ` cego à almofada, a `ENTREGA` cega à ponta que engrossou, a
//! mediana de aspecto cega a três quads emaranhados).
//!
//! ## O que é um SALTO, escrito como o dono o descreveu
//!
//! Ele nomeia **três** coisas, e a régua tem de exigir as três, senão mede outra: *assentar
//! correctamente* (a peça estava QUIETA antes), *dar um salto* (uma rotação LÍQUIDA grande num
//! punhado de tiques) e *ficar nessa angulação* (ela fica QUIETA depois, no ângulo novo).
//! ⚠️ **Sem a terceira, um ressalto da queda conta como salto** — e a queda tem muitos.
//!
//! ## ⭐⭐⭐ O que a régua MEDIU, e porque ela desmente a leitura óbvia
//!
//! Peça `12`, tique `157`, `substeps = 8`: quieta a `0,147 °/tique`, **`23,26°` em 8 tiques**,
//! deslocando `0,52` do lado dela, e quieta outra vez a `0,337°`. A peça `17` salta `16,5°` no
//! MESMO instante — *duas peças ao mesmo tempo, que é o que as quatro setas da foto mostram.*
//!
//! ⛔⛔ **E os SUB-PASSOS não o causam — eles só o revelam.** Cinco realizações por célula
//! ([`probe_o_salto_contra_os_substeps`]):
//!
//! ```text
//!   substeps | pior salto legitimo (5 realizacoes)          | mediana
//!          1 |  8,93°   8,41°   1,73°   0,90°   8,45°       |  8,41°
//!          4 | 17,45°  17,29°   2,66°  17,32°  17,36°       | 17,32°
//!          8 |  7,78°   3,40°  16,51°   3,15°   3,46°       |  3,46°
//!         16 |  2,46°  30,32°   1,60°  21,97°  20,13°       | 20,13°
//! ```
//!
//! O salto **já existia a `substeps = 1`**, o de ontem, e as medianas **não ordenam**: o ruído
//! entre realizações (`0,90°` a `30,32°`) engole qualquer tendência. ⇒ *a cura do tremor não
//! trouxe este defeito; ela tirou o tremor que o camuflava* — antes, nada assentava, e um salto
//! não tem contraste contra ruído.
//!
//! ⛔ **Por isso NÃO há gate aqui ainda.** A barra teria de sair de um vale medido, e este corpus
//! não tem nenhum: o pior de toda a varredura é `30,32°` e um quadrado tem simetria de `90°`.
//! *Inventar um número aqui seria exactamente o que o §0.0 proíbe* — o gate nasce com a cura, que
//! é quem define o lado aprovado.

use super::*;
use crate::motion_state::MotionState;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::graph::NodeId;

/// Até que tique se marcha. ⚠️⚠️ **A 1.ª redacção parava em `174` (`2,9 s`), «dentro do primeiro
/// ciclo de propósito» — e foi assim que ela deixou passar o defeito que existe para medir.**
///
/// O reinício do laço dá um `Δrot` gigante que de facto não é salto, mas a resposta a isso é
/// **filtrar o reinício**, nunca encurtar a janela: o dono vê a cena a repetir, e o salto de `18°`
/// que ele fotografou cai no **assentar da SEGUNDA queda** (índice `164`), que a janela velha
/// nunca alcançava — e a margem de `JANELA + salto` comia ainda mais cauda.
///
/// ⇒ *uma janela que nunca atravessa o reinício não pode ver um defeito que só aparece depois
/// dele*, e as quatro réguas anteriores desta cena partilhavam essa cegueira.
const ATE: u64 = 430;

/// ⭐ **O reinício do laço reconhece-se pela ALTURA, não por um relógio.** Quando a zona recomeça,
/// as peças voltam à altura de nascimento — um salto de `y` para cima muito maior do que qualquer
/// contacto produz. Um tique assim não é um salto e é EXCLUÍDO, com a peça toda.
///
/// ⚠️ Derivar isto do `DURACAO` seria a 2.ª resposta à pergunta *«quando é que o laço virou?»*, e
/// ela discordaria da primeira no dia em que o relógio da zona mudasse de lei.
const RENASCE_DY: f32 = 0.05;

/// Quantos tiques de cada lado têm de estar QUIETOS para o evento ser «assentou → saltou → ficou».
/// `12` tiques = `0,2 s` — tempo em que o olho já considera a peça parada.
const JANELA: usize = 12;

/// ⚠️⚠️ **Quantos tiques dura o SALTO em si — e a 1.ª redacção desta régua tinha-o a `1`.**
///
/// Com `1` tique ela lia `0,89°` na cena que o dono reprovou, e `0,89°` é invisível: *eu tinha
/// escrito a régua na unidade de tempo errada.* O olho não chama salto a um tique — chama salto a
/// **tudo o que acontece mais depressa do que ele consegue acompanhar**, que são ~`0,1 s`. ⇒ a
/// grandeza é a rotação LÍQUIDA numa janela de `SALTO` tiques, e não o degrau de um tique só.
///
/// ⚠️ Ele é ARGUMENTO nas portas e este é só o valor que as sondas usam por omissão: a duração de
/// um salto é precisamente o que não se sabe de antemão, e uma constante escondida escolheria a
/// resposta. As sondas VARREM-no.
const SALTO: usize = 8;

/// **Um salto: onde, quando, quanto, e a prova de que a peça estava parada dos dois lados.**
#[derive(Clone, Copy)]
pub(super) struct Salto {
    pub peca: usize,
    pub tique: u64,
    /// A rotação LÍQUIDA ao longo dos `SALTO` tiques do salto, em graus.
    pub grau: f32,
    /// O pior `|Δrot|` por tique nos `JANELA` tiques ANTES — a prova de que ela tinha assentado.
    pub antes: f32,
    /// E nos `JANELA` tiques DEPOIS — a prova de que ela FICOU no ângulo novo.
    pub depois: f32,
    /// Quanto a peça se deslocou durante o salto, em fracção do LADO dela.
    pub desloca: f32,
}

/// ⭐⭐⭐ **A MARCHA pela porta do PUMP, com as DUAS colunas.**
///
/// ⛔⛔ Ver [`super::tremor`]: uma sonda que chame o `Cook::cook` à mão mede uma variante da cena
/// **sem sub-passos**, e foi assim que uma tabela inteira de medições ficou sem sentido.
pub(super) fn marcha(
    state: &mut MotionState,
    sink: NodeId,
    ate: u64,
) -> Vec<(Vec<f32>, Vec<[f32; 2]>)> {
    let escopos = ph2d_nodegraph::cook::TimeScopes::new();
    let mut fora = Vec::new();
    for k in 0..=ate {
        state.pump.mark_dirty();
        state.pump.advance_or_scrub_to_nodes_scoped(
            &state.doc.graph,
            &state.registry,
            &[sink],
            k,
            |t| {
                #[expect(clippy::cast_precision_loss, reason = "um indice de tique")]
                let s = t as f64 / 60.0;
                s
            },
            &escopos,
        );
        let Some((_, saida)) = state
            .pump
            .boundary_streams()
            .iter()
            .find(|(no, _)| *no == sink)
        else {
            continue;
        };
        if let (Some(Column::Scalar(r)), Some(Column::Vec2(p))) = (saida.get("rot"), saida.get("P"))
        {
            fora.push((r.clone(), p.clone()));
        }
    }
    fora
}

/// **Todos os saltos da metade da direita**, ordenados do maior para o menor. `salto` é quantos
/// tiques ele dura (ver [`SALTO`]).
pub(super) fn saltos(substeps: u32, eps: f32, salto: usize) -> Vec<Salto> {
    let (mut state, sink) = super::obra::com_substeps(eps, substeps);
    let serie = marcha(&mut state, sink, ATE);
    let n = serie.first().map_or(0, |(r, _)| r.len());
    if serie.len() < JANELA * 2 + salto + 3 || n == 0 {
        return Vec::new();
    }
    // `d[t][i]` = |Δrot| da peça `i` entre os tiques `t` e `t+1`; `dp` o mesmo para a posição.
    let (mut d, mut dp) = (Vec::new(), Vec::new());
    for w in serie.windows(2) {
        d.push(
            (0..n)
                .map(|i| (w[1].0[i] - w[0].0[i]).abs())
                .collect::<Vec<_>>(),
        );
        dp.push(
            (0..n)
                .map(|i| (w[1].1[i][0] - w[0].1[i][0]).hypot(w[1].1[i][1] - w[0].1[i][1]))
                .collect::<Vec<_>>(),
        );
    }
    let pior = |faixa: &[Vec<f32>], i: usize| {
        faixa.iter().fold(0.0_f32, |a, linha| {
            a.max(linha.get(i).copied().unwrap_or(0.0))
        })
    };
    // ⭐ Os tiques em que a peça RENASCEU (o laço virou) — ver [`RENASCE_DY`]. Um salto que toque
    // um destes não é um salto; é a cena a recomeçar.
    let renasceu: Vec<Vec<bool>> = serie
        .windows(2)
        .map(|w| {
            (0..n)
                .map(|i| w[1].1[i][1] - w[0].1[i][1] > RENASCE_DY)
                .collect()
        })
        .collect();
    let mut fora = Vec::new();
    // `t` é o 1.º tique do salto; ele ocupa `t .. t+salto`, com `JANELA` tiques quietos de cada lado.
    for t in JANELA..d.len().saturating_sub(JANELA + salto) {
        for i in 0..n {
            // ⛔ O reinício do laço não é um salto — e ele tem de ser excluído da janela INTEIRA
            // (quieto antes + salto + quieto depois), senão a queda seguinte lê-se como «assentou».
            if renasceu[t - JANELA..t + salto + JANELA]
                .iter()
                .any(|linha| linha[i])
            {
                continue;
            }
            let (antes, depois) = (
                pior(&d[t - JANELA..t], i),
                pior(&d[t + salto..t + salto + JANELA], i),
            );
            // ⭐ O LÍQUIDO ao longo do salto — e não a soma dos `|Δrot|`: uma peça que vai e volta
            // não mudou de ângulo, e o dono queixa-se de uma que FICOU noutro.
            let liquido = (serie[t + salto].0[i] - serie[t].0[i]).abs();
            let desloca = (serie[t + salto].1[i][0] - serie[t].1[i][0])
                .hypot(serie[t + salto].1[i][1] - serie[t].1[i][1]);
            fora.push(Salto {
                peca: i,
                tique: t as u64,
                grau: liquido,
                antes,
                depois,
                desloca: desloca / LADO,
            });
        }
    }
    fora.sort_by(|a, b| b.grau.total_cmp(&a.grau));
    fora
}

/// ⭐⭐ **O pior salto LEGÍTIMO** — o maior `|Δrot|` de um tique cuja peça estava quieta dos DOIS
/// lados, em graus. É a grandeza do report do dono, e a única desta cena que não é média nem soma.
///
/// `quieto` é o tecto de `|Δrot|` por tique que ainda conta como «parada». Ele é um ARGUMENTO e não
/// uma constante de propósito: a sonda varre-o, e é a varredura que mostra que o achado não
/// depende dele.
pub(super) fn pior_salto(substeps: u32, eps: f32, quieto: f32, salto: usize) -> Option<Salto> {
    saltos(substeps, eps, salto)
        .into_iter()
        .find(|s| s.antes <= quieto && s.depois <= quieto)
}

// ---------------------------------------------------------------------------------------------
// SONDAS
// ---------------------------------------------------------------------------------------------

/// **SONDA — os saltos da cena que SHIPA, um a um.**
///
/// ```text
/// cargo test -p ph2d-app-motion --lib probe_os_saltos -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medicao"]
fn probe_os_saltos() {
    for sub in [1_u32, 8] {
        eprintln!("\n=== substeps = {sub} ===");
        let todos = saltos(sub, 0.0, SALTO);
        eprintln!("  peca | tique |   |Δrot| | quieto antes | quieto depois | desloca (lado)");
        eprintln!("  -----|-------|----------|--------------|---------------|---------------");
        for s in todos.iter().take(10) {
            eprintln!(
                "  {:>4} | {:>5} | {:>7.2}° | {:>11.3}° | {:>12.3}° | {:>13.3}",
                s.peca, s.tique, s.grau, s.antes, s.depois, s.desloca
            );
        }
        // ⚠️ A DURAÇÃO do salto varre-se: ela é o que não se sabe de antemão.
        for dur in [4_usize, 8, 15, 30] {
            let t = saltos(sub, 0.0, dur);
            let linha = [0.05_f32, 0.1, 0.2, 0.5]
                .map(|q| {
                    t.iter()
                        .find(|s| s.antes <= q && s.depois <= q)
                        .map_or(0.0, |s| s.grau)
                })
                .map(|g| format!("{g:>6.2}°"))
                .join(" ");
            eprintln!("  salto de {dur:>2} tiques | quieto 0,05/0,10/0,20/0,50 ⇒ {linha}");
        }
    }
}

/// **SONDA — o PERFIL NO TEMPO, atravessando o reinício do laço.**
///
/// ⚠️⚠️ **Todas as réguas desta cena param em `2,9 s`, dentro do PRIMEIRO ciclo** — e o dono vê o
/// laço repetir-se indefinidamente. *Uma janela que nunca atravessa o reinício não pode ver um
/// defeito que só aparece na segunda queda.* Esta sonda marcha `2` ciclos e imprime, tique a tique,
/// quanto a pilha rodou — a queda, o assentar, o reinício e o que vier depois.
///
/// ```text
/// cargo test -p ph2d-app-motion --lib probe_o_perfil_no_tempo -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medicao"]
fn probe_o_perfil_no_tempo() {
    for sub in [1_u32, 8] {
        let (mut state, sink) = super::obra::com_substeps(0.0, sub);
        let serie = marcha(&mut state, sink, 430);
        let n = serie.first().map_or(0, |(r, _)| r.len());
        eprintln!(
            "\n=== substeps = {sub} · {} tiques · {n} pecas ===",
            serie.len()
        );
        eprintln!("  tique |    t(s) | pior |Δrot| do tique | peca | y mediano");
        eprintln!("  ------|---------|--------------------|------|----------");
        for (t, w) in serie.windows(2).enumerate() {
            let (mut pior, mut quem) = (0.0_f32, 0_usize);
            for i in 0..n {
                let d = (w[1].0[i] - w[0].0[i]).abs();
                if d > pior {
                    pior = d;
                    quem = i;
                }
            }
            // Imprime só o que vale a pena olhar: eventos grandes, ou uma amostra a cada 30 tiques.
            if pior > 1.0 || t % 30 == 0 {
                let mut ys: Vec<f32> = w[1].1.iter().map(|p| p[1]).collect();
                ys.sort_by(f32::total_cmp);
                let y = ys.get(ys.len() / 2).copied().unwrap_or(0.0);
                #[expect(clippy::cast_precision_loss, reason = "um indice de tique")]
                let s = t as f32 / 60.0;
                eprintln!("  {t:>5} | {s:>6.2}s | {pior:>17.2}° | {quem:>4} | {y:>8.3}");
            }
        }
    }
}

/// **SONDA — o salto contra os SUB-PASSOS, com o ruído entre realizações ao lado.**
///
/// ⚠️ 25 quadrados a cair são caóticos: cada célula corre `5` realizações perturbando o berço em
/// `±0,003` (doc 109 §8.8). *Uma célula de uma realização só não distingue cura de sorteio.*
///
/// ```text
/// cargo test -p ph2d-app-motion --lib probe_o_salto_contra_os_substeps -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medicao"]
fn probe_o_salto_contra_os_substeps() {
    /// O tecto de `|Δrot|` por tique que ainda conta como «a peça tinha assentado».
    const QUIETO: f32 = 0.2;
    eprintln!("\n  substeps | pior salto legitimo (5 realizacoes)        | mediana");
    eprintln!("  ---------|-------------------------------------------|--------");
    for sub in [1_u32, 4, 8, 16] {
        let mut v: Vec<f32> = [-0.003_f32, -0.0015, 0.0, 0.0015, 0.003]
            .into_iter()
            .map(|eps| pior_salto(sub, eps, QUIETO, SALTO).map_or(0.0, |s| s.grau))
            .collect();
        let linha = v
            .iter()
            .map(|x| format!("{x:>7.2}°"))
            .collect::<Vec<_>>()
            .join(" ");
        v.sort_by(f32::total_cmp);
        eprintln!("  {sub:>8} | {linha} | {:>6.2}°", v[v.len() / 2]);
    }
}
