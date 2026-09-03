//! ⭐⭐⭐ **O ALVO DA GRADE, e as DUAS portas medidas-e-recusadas que ele destravou.**
//!
//! Irmão de [`super::retopo_extract`] por RESPONSABILIDADE: aquele responde *«o que o
//! botão FAZ?»*, e este responde a pergunta que vem antes — *«qual é o alvo, e onde ele
//! é mais fino?»*.
//!
//! ⚠️⚠️ **As duas portas aqui nascem DESLIGADAS, cada uma com a tabela da rejeição no
//! seu doc.** *Uma fase medida sozinha pode melhorar e piorar o produto* — a lei que esta
//! linha pagou duas vezes, e que aponta as duas para a mesma obra: o **factor de escala
//! conforme por construção**.

use ph2d_mesh::Mesh;

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
pub(super) fn f1_follows_target() -> bool {
    std::env::var("PH2D_F1_TARGET").as_deref() == Ok("1")
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
pub(super) fn sizing_field(work: &Mesh, target: f32, adaptive: f32) -> Vec<f32> {
    if adaptive <= 0.0 {
        return Vec::new();
    }
    // ⛔⛔ **`adaptive_graded` e NÃO `adaptive_with`** — ver o doc dela. O piso da irmã é a
    // aresta média da malha de TRABALHO, que é a cerca do motor local; emprestada aqui ela
    // colapsa os dois extremos da banda no mesmo número e o campo sai constante ao bit.
    // ⭐⭐⭐ **A FAIXA é escolhida AQUI, no chamador** — ver
    // [`ph2d_quadflow::ScaleField::adaptive_ranged`]. ⛔ `PH2D_SIZING_RATIO=<n>` é a sonda que
    // permite MEDIR se uma faixa maior rasga a grade (o modo de falha que o tecto declara)
    // antes de alguém lhe tocar; sem a env o produto chama a lei de sempre, ao bit.
    let field = match std::env::var("PH2D_SIZING_RATIO")
        .ok()
        .and_then(|v| v.parse::<f32>().ok())
    {
        Some(r) => ph2d_quadflow::ScaleField::adaptive_ranged(work, target, adaptive, r),
        None => ph2d_quadflow::ScaleField::adaptive_graded(work, target, adaptive),
    };
    let mut per_vertex: Vec<f32> = (0..work.vert_count()).map(|v| field.at(v)).collect();
    // ⭐⭐⭐ **O ALISAMENTO CORRE ANTES DA CONTAGEM, e a ordem é load-bearing.**
    //
    // ⛔⛔ A 1.ª versão alisava DEPOIS de `pred` estar calculado, e o gate
    // `a_densidade_segue_a_curvatura_sem_mudar_a_contagem` apanhou-o: o factor de
    // renormalização descrevia o campo **anterior** ao alisamento, e a contagem
    // prevista saía `−3,1 %` fora (a barra é `2 %`). *Normalizar por um número
    // medido sobre um campo que já não existe é normalizar para nada.*
    let rounds = std::env::var("PH2D_SIZING_SMOOTH")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(SIZING_SMOOTH_ROUNDS);
    {
        let (cru, amostra) = ph2d_quadfill::tip_body_ratio(work.positions(), &per_vertex);
        // ⛔ `amostra == 0` é NÃO MEDIDO — ver o doc da porta.
        eprintln!(
            "[sculpt3d] densidade adaptativa {adaptive:.2}: PEDE CRU razao ponta/corpo \
             {cru:.3} (amostra da ponta: {amostra} vertices)"
        );
    }
    smooth_in_log(work, &mut per_vertex, rounds);
    // ⚠️ **A 2.ª leitura, DEPOIS do alisamento** — e a 1.ª versão desta varredura não a
    // tinha: o print corria **antes** de `smooth_in_log`, então as quatro corridas do
    // knob imprimiram o MESMO `0,486` e não havia como dizer se o alisamento sequer
    // mexia no pedido. *Uma sonda posta antes do passo que ela devia medir mede o passo
    // anterior.* ⭐ E com ela veio o achado: `48` rondas movem o pedido `3 %`, logo o
    // pedido **nunca foi de alta frequência** e alisar não é o que move a densidade.
    {
        let (pedido, amostra) = ph2d_quadfill::tip_body_ratio(work.positions(), &per_vertex);
        eprintln!(
            "[sculpt3d] densidade adaptativa {adaptive:.2}: apos {rounds} ronda(s) PEDE \
             {pedido:.3} (amostra da ponta: {amostra} vertices)"
        );
    }
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

/// ⭐ **Quantas rondas de alisamento o pedido leva** — ver [`smooth_in_log`].
///
/// # A varredura que escolheu o `8` (peça do artista, `Detail 0,85`, `Follow Curvature = 1`)
///
/// | rondas | quads | razão ponta/corpo | faces na ponta | `>60°` | envies. p99 |
/// |---|---|---|---|---|---|
/// | *(knob desligado)* | `9 188` | ⛔ `1,533` | `82` | `2` | `21,6` |
/// | `0` | `8 033` | `1,144` | `118` | ⛔ **`8`** | `29,2` |
/// | `4` | `7 869` | `1,057` | `139` | `2` | `25,1` |
/// | ⭐ **`8`** | `7 602` | **`1,044`** | **`134`** | ⭐ **`0`** | `22,8` |
/// | `16` | `7 598` | `1,101` | `124` | `1` | `25,0` |
/// | `48` | `7 824` | ⛔ `1,271` | `99` | `1` | `23,8` |
///
/// ⚠️ **O alisamento NÃO é o que move a densidade** — ver [`smooth_in_log`]. O que ele
/// compra é a **forma**: as faces com canto pior que `60°` caem de `8` para `0`, ficando
/// melhores que a linha de base. *A adaptação passa a ser de graça em qualidade.*
///
/// ⛔ **Acima de `16` ele começa a desfazer a própria adaptação** (`1,271` a `48`): o
/// pedido é uma prescrição sobre a peça inteira, e difundi-la demais mistura a ponta com
/// o corpo. `PH2D_SIZING_SMOOTH=<n>` bissecta.
pub(super) const SIZING_SMOOTH_ROUNDS: usize = 8;

/// ⭐⭐⭐ **ALISA O PEDIDO, em LOG** — e ele é a resposta a uma medição, não uma opção.
///
/// # ⛔⛔ O que a medição de 2026-08-30 disse
///
/// Com a mesma lei ([`ph2d_quadfill::tip_body_ratio`]) nos dois lados, na peça do
/// artista a `Detail 0,85`: o campo **PEDE** `0,486` (a ponta com metade do tamanho
/// do corpo — melhor até que o `0,59` do remalhador que ele aprovou) e a cadeia
/// **ENTREGA `1,144`** — mais grossa na ponta. *O pedido não é fraco: ele é
/// descartado, e com o sinal invertido.*
///
/// # ⭐ O mecanismo, e porque a cura é ALISAR
///
/// O G3 resolve `min ‖∇f − R/h‖²`, cuja condição de óptimo é a equação de Poisson
/// `Δf = ∇·(R/h)`. ⇒ **o que chega à saída é a divergência do alvo, passada por um
/// inverso do Laplaciano** — um operador de **passa-baixo**. Um `h` que salta de
/// vértice para vértice tem a energia toda nas frequências que esse inverso
/// atenua, e sai de lá quase plano; um `h` **suave** atravessa.
///
/// ⚠️ **Em LOG e não linear.** A grandeza que a cadeia consome é uma *razão* de
/// tamanhos (a ponta é *metade* do corpo, não *menos 0,02*), e a média aritmética
/// de razões enviesa para o maior. *Alisar `h` faria a ponta subir mais depressa do
/// que o corpo desce.*
///
/// ⚠️ **A média é UMBRELLA (vizinhos por aresta), não cotangente.** O peso
/// cotangente é o certo para difundir uma quantidade sobre a *superfície*; aqui o
/// que se difunde é uma **prescrição**, e um peso que depende da forma dos
/// triângulos faria o pedido mudar com a triangulação que o F1 calhou de dar.
///
/// ⛔ **Não confundir com a suavização do campo de DIRECÇÕES**, construída e não
/// adoptada em 28/08 (handoff §8-sexies): aquela alisa para onde a grade aponta,
/// esta aliza *quão fina* ela é. Grandezas diferentes, medições diferentes.
pub(super) fn smooth_in_log(mesh: &Mesh, per_vertex: &mut [f32], rounds: usize) {
    if rounds == 0 || per_vertex.is_empty() {
        return;
    }
    // Vizinhança por aresta, construída uma vez.
    let mut nbr: Vec<Vec<u32>> = vec![Vec::new(); per_vertex.len()];
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k] as usize, v[(k + 1) % v.len()] as usize);
            if a < nbr.len() && b < nbr.len() {
                nbr[a].push(v[(k + 1) % v.len()]);
                nbr[b].push(v[k]);
            }
        }
    }
    let mut log: Vec<f32> = per_vertex.iter().map(|h| h.max(1.0e-9).ln()).collect();
    let mut next = log.clone();
    for _ in 0..rounds {
        for (i, ns) in nbr.iter().enumerate() {
            if ns.is_empty() {
                continue;
            }
            let mut s = 0.0f32;
            for &j in ns {
                s += log[j as usize];
            }
            #[allow(clippy::cast_precision_loss)]
            let mean = s / ns.len() as f32;
            // ⚠️ **Meio passo** (`½`), como no Laplaciano do resto da casa: um passo
            // inteiro sobre uma umbrella não é contractivo e pode oscilar.
            next[i] = 0.5f32.mul_add(mean - log[i], log[i]);
        }
        std::mem::swap(&mut log, &mut next);
    }
    for (h, l) in per_vertex.iter_mut().zip(log) {
        *h = l.exp();
    }
}

/// ⭐⭐⭐ **O PASSO DA CALOTA de cada espinho afiado**, em múltiplos do passo da grade —
/// [`ph2d_remesh_iso::Cap`]. `0` desliga; `PH2D_TIP_CAP=<x>` sobrepõe-se, para bissecar.
///
/// # ⭐⭐⭐ O `1,0` é MEDIDO, e o report do dono (03/09, com foto e seta) é a razão
///
/// A fase zero entregava o bico a **`2,22 ×`** o passo da grade (p50 `1,56`), e o pólo `+1` que
/// fecha um bico precisa de `≥ 2` células de calota resolvida (plano §101). Medido de ponta a
/// ponta com `PH2D_RECENTER=1` sobre o ficheiro cru, `Detail 1` · `Curv 1`:
///
/// | peça / realização | hoje | com a calota `1,0 h` |
/// |---|---|---|
/// | ⭐⭐ `_base_sculpt` — **a realização que o dono vê** | `1/5` amputada · gap `3,00` · grade **`3,51`** | ⭐ **`0/5`** · `0,47` · **`0,79`** |
/// | `_base_sculpt` a `s = 0,7` | `0/5` · gap `0,45` · grade `1,66` (`3` acima) | `0/5` · **`0,38`** · **`1,07`** (**`2`**) |
/// | `sculpt_antes` (a agulha) | `1/4` · gap `3,00` · grade `1,15` (`1` acima) | `1/4` · **`2,57`** · **`0,98`** (**`0`**) |
///
/// ⭐ **Três de três melhoram ou empatam em todas as colunas, e nenhuma piora** — a topologia
/// fica `χ = 2`, zero bordo, zero não-manifold nas três.
///
/// ⛔⛔ **Afinar MAIS é pior, e está medido:** a `0,75` e a `0,5` a fase zero fica verde no bico
/// (`0,84` e `0,55`) e a cadeia a jusante deixa de digerir a inflação — candidatas com `7`–`48`
/// arestas de bordo e `1` não-manifold na saída. *O que a jusante não digere é a INFLAÇÃO.*
///
/// ⚠️ **E a calota sozinha não bastava:** ela **produz** a candidata verde, e quem a deixa
/// **ganhar** é [`ph2d_quadfill::untangle_bowties`] — sem ele, uma gravata a `5,7` células do
/// bico deitava-a fora na 3.ª chave do [`super::decide::worse`] (plano §105).
const TIP_CAP_STEP: f32 = 1.0;

/// **Até onde a calota alcança**, em múltiplos do passo da grade. `PH2D_TIP_CAP_R=<x>` sobrepõe.
///
/// ⚠️ **`8 h` é a mesma distância do `PH2D_TIP_ALIGN`** (plano §102) e não é coincidência: as
/// duas experiências atacam a mesma calota — uma dá-lhe PESO no campo, esta dá-lhe RESOLUÇÃO.
const TIP_CAP_RADIUS: f32 = 8.0;

fn env_f32(key: &str) -> Option<f32> {
    std::env::var(key).ok().and_then(|s| s.parse().ok())
}

/// ⭐⭐⭐ **AS CALOTAS que a fase zero recebe** — uma por espinho AFIADO da escultura.
///
/// ⛔ **A lei do ápice é a da casa** ([`ph2d_quadfill::apices`], unidade = o passo da grade):
/// ela já filtra as bossas pelo cone, e é a MESMA que as réguas por ponta usam para decidir se
/// a saída amputou. *Duas listas de bicos seriam duas respostas à mesma pergunta, e a que
/// envelhece é a que o artista vê.*
fn tip_caps(reference: &Mesh, target: f32) -> Vec<ph2d_remesh_iso::Cap> {
    let step = env_f32("PH2D_TIP_CAP").unwrap_or(TIP_CAP_STEP);
    // ⚠️ **`is_sign_positive` não serve** — ele diz `true` para `NaN` positivo e para `+0,0`; o que
    // esta porta precisa é *estritamente maior que zero e finito*, que é o que o par abaixo diz.
    if !step.is_finite() || step <= 0.0 || !target.is_finite() || target <= 0.0 {
        return Vec::new();
    }
    let radius = env_f32("PH2D_TIP_CAP_R").unwrap_or(TIP_CAP_RADIUS);
    let (_, apex) = ph2d_quadfill::apices(reference, target);
    let pos = reference.positions();
    apex.iter()
        .filter_map(|&i| pos.get(i).copied())
        .map(|at| ph2d_remesh_iso::Cap {
            at,
            radius: radius * target,
            step: step * target,
        })
        .collect()
}

/// ⭐⭐⭐ **A FASE ZERO DO BOTÃO** — as duas decisões dela, num sítio só.
///
/// Ela mora aqui e não no chamador porque as duas são sobre **o alvo da malha de trabalho**,
/// que é o assunto deste módulo: *seguir o alvo do quad* ([`f1_follows_target`], medida e
/// **recusada**) e *graduar a densidade dentro do orçamento*
/// ([`ph2d_remesh_iso::remesh_isotropic_graded`]).
///
/// # ⭐⭐⭐ A GRADUAÇÃO, medida em 2026-08-31 pela régua POR PONTA
///
/// ⛔ *O ALCANCE é um extremo global e esconde uma ponta cortada atrás de outra que
/// sobreviveu* — na `sculpt_antes` ele piora enquanto as pontas cortadas caem de `3` para `1`.
///
/// | peça (`Detail 0,85`) | pontas cortadas | furos na saída | alcance |
/// |---|---|---|---|
/// | `espinhos:6 σ=0,30` | `0/6` → `0/6` | `0` → `0` | `+2,8 %` → `+1,8 %` |
/// | ⭐ `espinhos:6 σ=0,14` | **`5/6` → `0/6`** | `0` → `0` | `+3,7 %` → `+0,3 %` |
/// | ⭐⭐⭐ `espinhos:6 σ=0,07` | pior `−20,5 %` → **`−7,6 %`** | ⛔ `4` → ⭐ **`0`** | `−15,5 %` → ⭐ **`−3,5 %`** |
/// | ⭐⭐ `_base_sculpt` | `3/4` pior `−41,2 %` → **`−8,4 %`** | `0` → `0` | ⭐⭐ `−41,8 %` → **`−11,1 %`** |
/// | ⭐ `sculpt_antes` | **`3/6` → `1/6`** | ⭐ `4` → **`0`** | ⚠️ `−13,6 %` → `−16,8 %` |
///
/// ⭐ **Cinco de cinco melhoram ou empatam nas pontas cortadas E nos furos**, e a agulha mais
/// fina — que saía com `χ = 1` e `4` arestas de bordo — passa a **fechar**. Preço: `+7 %` a
/// `+15 %` de faces na malha de trabalho, contra os `7`–`8×` da versão que só afinava.
///
/// ⛔⛔ **A escolha é DAQUI, e não de uma env lida dentro do remalhador.** A 1.ª versão desta
/// wave lia `PH2D_ISO_ADAPT` dentro do laço, logo alcançava **todos** os chamadores — e o gate
/// `the_ear_does_not_ship_an_edge_across_the_piece`, que corre o motor **legado**, reprovou.
/// *O doc do `remesh_with` já escrevia a lei que essa versão violava: «por argumento e não por
/// variável de ambiente — uma bandeira global é uma corrida escrita à mão».*
///
/// ⚠️ `PH2D_ISO_ADAPT=0` volta ao remalhador uniforme, para bissecar.
pub(in crate::sculpt3d) fn phase_zero(reference: &Mesh, target: f32) -> Mesh {
    if f1_follows_target() {
        return ph2d_quadchain::phase_zero(reference, target);
    }
    let mut w = reference.clone();
    if ph2d_remesh_iso::adaptive_on() {
        ph2d_remesh_iso::remesh_isotropic_graded_capped(
            &mut w,
            ph2d_remesh_iso::ALPHA,
            &tip_caps(reference, target),
        );
    } else {
        ph2d_remesh_iso::remesh_isotropic(&mut w, ph2d_remesh_iso::ALPHA);
    }
    w.triangulate();
    // ⭐ **`PH2D_DUMP_F1=<ficheiro>` escreve a malha de TRABALHO** — a sonda que separa as duas
    // metades da pergunta *«onde a ponta perde a resolução?»*: se a fase zero já sai grossa no
    // bico, a cura é o campo de tamanho; se ela sai fina e a saída sai grossa, é o mapa. ⛔ Sem
    // esta porta a pergunta só se responde por hipótese.
    if let Ok(path) = std::env::var("PH2D_DUMP_F1") {
        let text = ph2d_mesh::write_obj(&[ph2d_mesh::ExportPiece {
            name: Some("f1"),
            mesh: &w,
            pose: ph2d_mesh::Pose::default(),
        }]);
        let _ = std::fs::write(&path, text);
    }
    w
}

#[cfg(test)]
#[path = "sculpt3d_retopo_target_tests.rs"]
mod tests;
