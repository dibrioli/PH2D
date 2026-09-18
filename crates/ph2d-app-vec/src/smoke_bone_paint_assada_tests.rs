//! ⏱️⏱️ **A W0 DA F9 — UMA MALHA ASSADA UMA VEZ ALISA A DOBRA COMO O REFINAMENTO POR QUADRO?**
//!
//! Filha da [`super`] (a silhueta) para herdar as fixturas e as duas réguas; a pergunta é a que a
//! fila do módulo manda medir **antes de qualquer código** (`docs/Skeleton/01_a_fila.md`, F9 W0):
//!
//! > *uma malha fixa assada em repouso alisa a dobra FORTE como o refinamento por quadro alisa?*
//!
//! # Porque esta é a pergunta DECISIVA, e não uma entre várias
//!
//! A direcção inteira da F9 — a densidade sai do QUADRO e vai para o BIND, e a deformação passa
//! para o *vertex shader* — assenta numa única propriedade: **uma topologia só tem de servir todas
//! as poses**. Se ela servir, o resto é engenharia (enviar poses, costurar os leitores da malha).
//! Se não servir, a direcção cai e as ondas W1–W4 medem outra coisa.
//!
//! ⚠️ **E ela mede-se sem inventar lei nenhuma.** Assar *no pior caso* (a dobra mais forte, no zoom
//! mais fino) usa o refinamento que já existe e responde à pergunta da topologia. O critério
//! proposto na fila — refinar onde o CAMPO DE PESOS curva — é uma forma de escolher ONDE partir com
//! menos peças, e só faz sentido medir depois de se saber que uma topologia fixa chega.
//!
//! # ⛔ O CONTROLO que torna a comparação válida
//!
//! Re-posar uma malha assada noutra cena só significa alguma coisa se o BIND das duas cenas for o
//! mesmo — mesmas posições de repouso, mesmos pesos. Se a `cena_dobrada` mexesse no bind ao dobrar,
//! a tabela compararia duas artes diferentes e leria isso como qualidade. ⇒ a sonda **afirma-o
//! primeiro**, e pára se não for verdade.
//!
//! # O que cada coluna é
//!
//! * **peças** — triângulos entregues. Na assada é um número só, o mesmo em toda a linha: é ele que
//!   vira memória de GPU (a W4 da fila).
//! * **faceta** — `skinned_deviation`, o desvio da malha contra o campo que ela segue, em pixels de
//!   ECRÃ (já multiplicado pelo zoom).
//! * **canto** — o maior canto da silhueta desenhada (`vai_e_volta`), que é a régua do OLHO e a que
//!   apanhou o report de 2026-09-16. ⚠️ Ela não sabe que campo existe, de propósito.

use super::*;

/// O CAMPO de deformação desta cena: `(repouso, pesos) → posado`.
///
/// ⚠️ Ele é um `dyn` e não um genérico porque as duas sondas o passam ADIANTE (a
/// [`posa_a_assada`] recebe-o já construído por quem tem a pele na mão), e um `impl FnMut` não
/// atravessa essa fronteira sem monomorfizar cada chamador.
type Campo<'a> = dyn FnMut([f64; 2], &[f64]) -> [f64; 2] + 'a;

/// Os pesos de um vértice DENTRO dos atributos refinados — os `ossos` primeiros do passo.
///
/// ⚠️ **O passo não é `ossos`**: a lei de Hermite guarda gradientes ao lado dos pesos, e é por isso
/// que o produto faz `w.get(..ossos)` antes de chamar o campo (`ph2d_skeleton_live::skin_refine`).
fn pesos_no_passo(attrs: &[f64], passo: usize, v: usize, ossos: usize) -> &[f64] {
    &attrs[v * passo..v * passo + ossos]
}

/// ⭐⭐⭐ **O QUE O *VERTEX SHADER* FARIA** — cada vértice da malha assada posado a partir do
/// **repouso dele e dos pesos dele**, na cena de agora.
///
/// ⚠️ É exactamente a conta da F9: por vértice, repouso + pesos (enviados quando a malha muda); por
/// quadro, só as poses dos ossos. Nada aqui olha para a pose em que a malha foi assada.
fn posa_a_assada(
    rest: &[[f64; 2]],
    attrs: &[f64],
    passo: usize,
    ossos: usize,
    campo: &mut Campo<'_>,
) -> Vec<[f64; 2]> {
    (0..rest.len())
        .map(|v| campo(rest[v], pesos_no_passo(attrs, passo, v, ossos)))
        .collect()
}

/// O maior canto da silhueta desenhada de uma malha posada.
/// ⭐⭐⭐ **A W1 DA F9 NA ARTE REAL — a malha ASSADA no bind erra o campo menos do que o `Fast`, e
/// dentro da barra que as duas leis prometem.**
///
/// Corrido sobre a arte da cena do dono (`512 × 320`) e sobre os pesos BBW que o bind dela de facto
/// resolve — ⛔ não sobre um campo sintético.
///
/// # ⛔⛔ A régua desta wave NÃO é a silhueta, e a medição é que o disse
///
/// A fila pedia *«a régua da silhueta a mesma de hoje»*, e a 1.ª redacção deste gate obedeceu à
/// letra. Medido:
///
/// | desenho | peças | nós no topo | vai-e-volta | **desvio ao CAMPO** |
/// |---|---:|---:|---:|---:|
/// | `Fast` | `2 430` | `46` | `26,60°` | `0,4143 px` |
/// | `Smooth` (do quadro, zoom `8×`) | — | `64` | `26,71°` | `0,0881 px` |
/// | **assada** | `13 996` | `85` | `29,44°` | **`0,1781 px`** |
///
/// ⚠️⚠️ **O «vai-e-volta» CRESCE COM A DENSIDADE por construção** — ele soma a viragem absoluta ao
/// longo da polilinha da silhueta, e uma polilinha com mais nós segue melhor a curva verdadeira e
/// portanto acumula mais viragem. *Uma régua cuja janela segue o número que se está a variar não
/// pode ser a testemunha da variação* — a mesma lei que o espaçamento do pincel afiado já pagou.
///
/// ⇒ a régua com UNIDADE e com barra declarada é o **desvio ao campo** (`skinned_deviation`, em
/// pixels da arte), e a barra é a que as duas leis já prometem: [`TOLERANCIA_PX`] (`0,5 px`).
///
/// # O que os números dizem
///
/// A assada erra **`2,3×` menos** que o `Fast` e **`2,0×` mais** que o `Smooth` do quadro — e as
/// duas estão **dentro** de `0,5 px`. ⭐ E a diferença é esperada e não é um defeito: o `Smooth`
/// refina onde o erro está **naquela pose e naquele zoom**, e a assada é **independente da pose**
/// por construção. *Uma aproximação que serve todas as poses nunca bate, peça a peça, uma feita
/// para uma só* — e é precisamente por servir todas que ela se paga uma vez.
///
/// ⚠️ **O CONTROLO é o `Fast`:** ele tem de errar MAIS, senão a cena não contém o fenómeno e um
/// empate a três não afirmaria nada.
#[test]
fn a_malha_assada_no_bind_desenha_como_o_smooth_do_quadro() {
    let (sim, e) = cena(super::super::super::ALTURA_PX, None);
    let (sm, p2l, pele) = campo_da_cena(&sim, e);
    let ossos = sm.ossos();
    let mut ws = pele.scratch();
    let mut campo =
        |q: [f64; 2], pesos: &[f64]| ponto_do_produto(&pele, p2l.apply(q), pesos, &mut ws);

    // A ASSADURA — a lei, não a porta (ela lê uma env var que um teste não pode fixar).
    let (assada, pesos_assados) = ph2d_skeleton_live::skin_bake::assar(&sm.mesh, &sm.pesos, ossos)
        .expect("o campo de pesos BBW desta arte curva na articulacao");
    println!(
        "  bind {} -> assada {} pecas ({:.2}x)",
        sm.mesh.tris.len(),
        assada.tris.len(),
        assada.tris.len() as f64 / sm.mesh.tris.len() as f64
    );

    let zoom = 8.0_f64;
    let posa = |m: &ph2d_poly2d::Mesh2d, pesos: &[f64], campo: &mut Campo<'_>| {
        m.rest
            .iter()
            .enumerate()
            .map(|(v, &q)| campo(q, &pesos[v * ossos..(v + 1) * ossos]))
            .collect::<Vec<_>>()
    };
    let p_fast = posa(&sm.mesh, &sm.pesos, &mut campo);
    let p_assada = posa(&assada, &pesos_assados, &mut campo);
    let smooth = ph2d_skeleton_live::skin_refine::refine_skinned(
        &sm.mesh,
        &sm.pesos,
        ossos,
        &mut campo,
        opcoes(true, zoom),
    );

    // ⭐⭐⭐ **O DISCRIMINADOR: quanto cada malha erra o CAMPO VERDADEIRO** (a régua do
    // `skinned_deviation`, em pixels da arte). A silhueta mede ONDULAÇÃO, que cresce com a
    // densidade por construção — *uma régua cuja janela segue o número que se está a variar não
    // pode ser a única testemunha da variação*, a lei que o espaçamento do pincel afiado já pagou.
    let desvio =
        |m: &ph2d_poly2d::Mesh2d, pesos: &[f64], posed: &[[f64; 2]], campo: &mut Campo<'_>| {
            let lei = ph2d_skeleton_live::skin_refine::weight_law(ossos, true);
            let attrs = ph2d_skeleton_live::skin_refine::weight_attrs(m, pesos, lei);
            ph2d_skeleton_live::skin_refine::skinned_deviation(m, posed, &attrs, lei, campo)
                * PX_POR_METRO
        };
    let d_fast = desvio(&sm.mesh, &sm.pesos, &p_fast, &mut campo);
    let d_assada = desvio(&assada, &pesos_assados, &p_assada, &mut campo);
    let d_smooth = ph2d_skeleton_live::skin_refine::skinned_deviation(
        &smooth.mesh,
        &smooth.posed,
        &smooth.attrs,
        smooth.law,
        &mut campo,
    ) * PX_POR_METRO;
    println!(
        "  desvio ao CAMPO (px da arte): Fast {d_fast:.4} | Smooth {d_smooth:.4} | ASSADA {d_assada:.4}"
    );

    let l_fast = silhueta(&sm.mesh, &p_fast, zoom);
    let l_smooth = silhueta(&smooth.mesh, &smooth.posed, zoom);
    let l_assada = silhueta(&assada, &p_assada, zoom);
    for ((nome, f), ((_, s), (_, a))) in l_fast.iter().zip(l_smooth.iter().zip(&l_assada)) {
        println!(
            "  {nome:>4} | Fast {:>6.2} ({} nos) | Smooth {:>6.2} ({} nos) | ASSADA {:>6.2} ({} nos)",
            {
                let (n, t, _) = vai_e_volta(f);
                t - n.abs()
            },
            f.len(),
            {
                let (n, t, _) = vai_e_volta(s);
                t - n.abs()
            },
            s.len(),
            {
                let (n, t, _) = vai_e_volta(a);
                t - n.abs()
            },
            a.len(),
        );
    }
    assert!(
        d_assada <= ph2d_skeleton_live::skin_bake::TOLERANCIA_PX,
        "a malha ASSADA erra {d_assada:.4} px do campo, acima da barra de {} px que a lei promete",
        ph2d_skeleton_live::skin_bake::TOLERANCIA_PX
    );
    assert!(
        d_assada < d_fast,
        "a malha ASSADA ({d_assada:.4} px) nao bate a malha do bind sem assar ({d_fast:.4} px) — \
         a assadura nao esta' a comprar nada"
    );
    // ⛔ **O CONTROLO:** sem uma malha crua que erre MAIS do que a barra, a asserção de cima
    // passaria sobre qualquer coisa.
    assert!(
        d_fast > d_assada * 2.0,
        "o `Fast` erra {d_fast:.4} px contra {d_assada:.4} da assada — a cena deixou de conter o \
         fenomeno, e um empate nao afirma nada"
    );
    // ⚠️ E o `Smooth` do quadro fica NOMEADO: ele erra menos por ser feito para ESTA pose.
    assert!(
        d_smooth <= d_assada,
        "o Smooth do quadro ({d_smooth:.4}) passou a errar MAIS que a assada ({d_assada:.4}) — \
         a leitura escrita no doc deste gate deixou de valer"
    );
}

/// A instância que o extract emite para a sprite `s` — o quad DELA, com a âncora resolvida.
///
/// ⚠️ Escrita aqui porque a sonda precisa de um `present` e a gémea dela vive na `ph2d-skeleton-live`
/// dentro de `#[cfg(test)]`, logo é invisível daqui (a armadilha §2.7 do HOWTO). As duas descrevem a
/// mesma coisa; o que esta sonda MEDE não depende de nenhum dos campos que elas pudessem divergir.
/// ⭐⭐⭐ **A CENA LOTADA CONTÉM O FENÓMENO QUE ELA EXISTE PARA MOSTRAR.**
///
/// O report que abriu a F9 é *«uma cena com muita arte presa fica com o `Smooth` igual ao `Fast`»*,
/// e a causa é a soma das malhas presas passar o orçamento de refinamento do quadro. ⚠️ **Uma cena
/// que não lá chegue ensina o contrário do que diz** — o dono carregaria em `Smooth`, veria a
/// diferença e concluiria que o defeito nunca existiu.
///
/// ⚠️ **As duas metades:** a cena de UM canvas fica ABAIXO do orçamento (é a que o dono já aprovou,
/// e ela não é o sujeito desta pergunta) · e o TECTO do roteador tem de a levar ACIMA dele.
///
/// ⚠️ A contagem por canvas sai do BIND do produto, nunca de um número escrito aqui.
#[test]
fn a_cena_lotada_passa_o_orcamento_do_quadro() {
    use ph2d_skeleton_live::skin_image::SKIN_FRAME_PIECES;
    let (sim, e) = cena(super::super::super::ALTURA_PX, None);
    let (sm, _p2l, _pele) = campo_da_cena(&sim, e);
    let por_canvas = sm.mesh.tris.len();
    assert!(
        por_canvas <= SKIN_FRAME_PIECES,
        "um canvas sozinho ({por_canvas} pecas) ja' passa o orcamento ({SKIN_FRAME_PIECES}) — a \
         cena que o dono aprovou mudou de regime, e o roteiro de 8 passos dela deixou de valer"
    );
    let tecto = por_canvas * super::super::super::NIVEIS as usize;
    assert!(
        tecto > SKIN_FRAME_PIECES,
        "com o tecto do roteador ({} canvas) a cena guarda {tecto} pecas e o orcamento e' \
         {SKIN_FRAME_PIECES} — ela NAO chega ao regime do report, e o smoke nao mostra nada",
        super::super::super::NIVEIS
    );
}

/// ⛔ **O NÍVEL É UMA CONTAGEM, e o caminho de omissão é a cena de UM canvas.**
///
/// ⚠️ **O `=` vazio é o caso que morde:** `env VAR=` **define** a variável com a string vazia (a
/// armadilha que esta casa já pagou num arnês de mutação), e ali a resposta certa é `1` e não zero
/// — *uma cena com zero canvas monta e não demonstra nada.*
#[test]
fn o_nivel_e_uma_contagem_e_o_omisso_e_um_canvas() {
    use super::super::super::{NIVEIS, quantos_de};
    assert_eq!(quantos_de(None), 1, "sem env a cena e' a de sempre");
    assert_eq!(
        quantos_de(Some("")),
        1,
        "`env VAR=` define a variavel VAZIA"
    );
    assert_eq!(quantos_de(Some("1")), 1);
    assert_eq!(quantos_de(Some(" 4 ")), 4);
    assert_eq!(quantos_de(Some("0")), 1, "zero canvas nao e' uma cena");
    assert_eq!(quantos_de(Some("99")), NIVEIS, "o tecto e' o do roteador");
    assert_eq!(
        quantos_de(Some("sim")),
        1,
        "um valor ilegivel cai no omisso"
    );
}

/// ⛔⛔ **COM UM CANVAS A CENA É A DE SEMPRE, AO BIT** — é isto que mantém o roteiro de 8 passos que
/// o dono aprovou.
///
/// ⚠️ A régua é a POSIÇÃO, que é a única coisa que a contagem podia mover: a coluna centra-se na
/// origem, logo com `n = 1` o único canvas tem de ficar exactamente onde ele ficava.
#[test]
fn com_um_canvas_a_cena_fica_onde_sempre_esteve() {
    use super::super::super::centro_do_canvas;
    assert_eq!(centro_do_canvas(0, 1, PPM), [0.0, 0.0]);
    // E o CONTROLO: com mais de um, eles separam-se — senão esta metade passaria sobre uma coluna
    // que põe os seis no mesmo sítio.
    let a = centro_do_canvas(0, 4, PPM);
    let b = centro_do_canvas(1, 4, PPM);
    assert!(
        (a[0] - b[0]).abs() > f64::from(super::super::super::LARGURA_PX) / f64::from(PPM),
        "dois canvas da cena lotada ficaram a menos de uma largura um do outro: {a:?} e {b:?}"
    );
    // ⛔ E a fileira é DEITADA: uma coluna põe a arte dobrada por cima do enquadramento (medido por
    // foto), e a régua que o diz é o `y` ficar igual.
    assert!(
        (a[1] - b[1]).abs() < 1e-9,
        "a cena lotada voltou a ser uma COLUNA: {a:?} e {b:?}"
    );
}

/// ⭐⭐⭐ **O QUE O OLHO VÊ, EM PIXELS DE ECRÃ** — filho por ASSUNTO (e pelo tecto de LOC).
///
/// ⚠️ **Ele existe porque todas as réguas deste ficheiro medem a grandeza ERRADA para a pergunta do
/// dono:** elas medem o desvio ao CAMPO em pixels da ARTE, que é uma propriedade da aproximação. O
/// dono vê pixels de ECRÃ, e foi ali que o report *«Fast e Smooth estão sempre idênticos»* se
/// mediu — e se confirmou.
/// ⏱️ **AS MEDIÇÕES (`--ignored`)** — filho por ASSUNTO: aqui vivem os gates, ali as SONDAS que
/// imprimem tabela e não têm barra. *Uma sonda não reprova nada, e misturá-las com o que reprova faz
/// o ficheiro crescer no eixo errado.*
#[path = "smoke_bone_paint_medicoes_tests.rs"]
mod medicoes;

#[path = "smoke_bone_paint_pixels_tests.rs"]
mod pixels;
