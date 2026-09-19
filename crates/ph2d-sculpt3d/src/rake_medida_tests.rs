//! **A MEDIÇÃO QUE A ESPEC §14.5 EXIGE** — o `G-2` sobre a NOSSA saída.
//!
//! A pergunta que a espec deixa **em aberto** é se o pente é *uma* lei no sítio
//! certo ou *duas*. Ela não é decidível de fora do alvo: o instrumento dele
//! nunca produz uma célula em que o operador de topologia tenha trabalhado **e**
//! as arestas sejam comparáveis. ⇒ a espec manda fazer a hipótese **(b)** — a
//! relaxação dentro do laço por-carimbo — e **medir contra o `G-2`**. Se a nossa
//! malha ficar abaixo da barra com a lei já no sítio certo, a causa é a **(a)** e
//! há uma segunda lei a construir.
//!
//! ⚠️ **A régua é a do corpus e é PLANAR**: `Q = média(cos 4α)` sobre as arestas
//! cujo ponto médio cai na pegada, com a aresta e a direcção do traço projectadas
//! no plano do ecrã. As fixturas do oráculo são planas vistas de topo, logo a
//! projecção XY **é** o ecrã — e a dívida de uma régua para malha curva está
//! nomeada no `README` delas.
//!
//! ⚠️ **A região é `raio/2`**, não o raio inteiro: medido no corpus, ali a
//! separação é `2`–`3×` maior, porque o efeito concentra-se na faixa central do
//! traço e a orla dilui-o.

use ph2d_mesh::Mesh;

use crate::{Brush, Dab, Falloff, SculptStroke, Symmetry, Verb};

/// Uma chapa plana triangulada, `n × n` vértices no plano `z = 0`, com o
/// interior **sacudido**.
///
/// ⚠️ **Plana de propósito:** é o que torna a projecção XY igual ao ecrã, que é
/// a premissa da régua do corpus.
///
/// ⛔⛔ **E SACUDIDA de propósito, porque a 1.ª redacção media a FIXTURA.** Uma
/// grade regular triangulada pelos eixos já É uma grade alinhada com um traço ao
/// longo de `+x`: com o pente desligado ela lia `Q = +0,2885`, seis vezes acima
/// da barra, e o gate `G-3` reprovava sobre uma lei correcta. *O lado desligado
/// tem de conter o fenómeno tanto quanto o ligado* — o corpus do oráculo parte de
/// uma malha de topologia dinâmica, que é irregular por construção.
///
/// O sacudir é **determinístico** (um hash dos índices): ele não é ruído de
/// corrida, é a fixtura.
fn chapa(n: usize, lado: f32) -> Mesh {
    let passo = lado / (n - 1) as f32;
    let meio = lado * 0.5;
    let mut pos = Vec::with_capacity(n * n);
    let sacode = |i: usize, j: usize, sal: u32| -> f32 {
        let mut h = (i as u32).wrapping_mul(0x9E37_79B9)
            ^ (j as u32).wrapping_mul(0x85EB_CA6B)
            ^ sal.wrapping_mul(0xC2B2_AE35);
        h ^= h >> 15;
        h = h.wrapping_mul(0x2545_F491);
        h ^= h >> 13;
        (h & 0xFFFF) as f32 / 65535.0 - 0.5
    };
    for j in 0..n {
        for i in 0..n {
            // ⚠️ A borda fica QUIETA: sacudi-la deformaria o contorno da chapa,
            // e o que se quer irregular é o INTERIOR, que é onde a régua mede.
            let dentro = i > 0 && j > 0 && i < n - 1 && j < n - 1;
            let (dx, dy) = if dentro {
                (sacode(i, j, 1) * passo * 0.7, sacode(i, j, 2) * passo * 0.7)
            } else {
                (0.0, 0.0)
            };
            pos.push([
                i as f32 * passo - meio + dx,
                j as f32 * passo - meio + dy,
                0.0,
            ]);
        }
    }
    let mut faces = Vec::new();
    for j in 0..n - 1 {
        for i in 0..n - 1 {
            let (a, b, c, d) = (
                (j * n + i) as u32,
                (j * n + i + 1) as u32,
                ((j + 1) * n + i + 1) as u32,
                ((j + 1) * n + i) as u32,
            );
            faces.push(ph2d_mesh::Face::tri(a, b, c));
            faces.push(ph2d_mesh::Face::tri(a, c, d));
        }
    }
    Mesh::from_parts(pos, faces).expect("a chapa e' uma malha valida")
}

/// ⭐⭐ **AS DUAS COLUNAS vêm da PORTA** ([`crate::medida_do_pente`]) e não de
/// cópias aqui: a bancada corre sobre uma CHAPA e o gate da cena de smoke sobre
/// uma BOLA, e duas cópias divergiriam na primeira wave que mexesse numa delas.
///
/// ⚠️ **A porta é 3D e a versão que aqui viveu era PLANA** — ela media `x` e `y`
/// e deitava o `z` fora. Sobre esta chapa as duas dão o MESMO número, e é isso
/// que os valores registados na tabela do [`crate::Brush::pente`] provam: *a
/// forma velha é um caso particular da nova, não uma aproximação dela.*
use crate::medida_do_pente::{pior_angulo as ph2d_sculpt3d_ang, q_da_faixa as ph2d_sculpt3d_q};

fn pincel(pente: f32) -> Brush {
    Brush {
        verb: Verb::Draw,
        radius: 0.30,
        strength: 0.25,
        falloff: Falloff::Smooth,
        pente,
        ..Brush::default()
    }
}

/// O instrumento da varredura do viés — ver [`diag_a_varredura_do_vies`].
///
/// ⚠️ **Ele NÃO é o produto**: os dois `k` são livres aqui e no produto eles
/// saem de [`ph2d_rake::VIES_DA_GRADE`] vezes o botão. É por isso que o gate
/// chama o [`traco`], que os crava na lei — *uma sonda que escolhe os próprios
/// números não pode ser a régua de quem os escolheu.*
#[derive(Clone, Copy)]
struct Vies {
    /// ⭐⭐ **`None` é o campo DO PRODUTO** ([`ph2d_rake::campo_do_pente`]), e é
    /// com ele que o gate corre — *uma fixtura que reescreve a lei mede a cópia
    /// dela*. `Some((k, escala))` é a SONDA, com os dois números livres.
    ///
    /// ⛔⛔ A `escala` é o **CONTROLO** da varredura: sem ela a leitura fica
    /// CONFUNDIDA, porque o viés compra `Q` **e** adensa a malha — *uma malha só
    /// mais fina podia ler `Q` mais alto sem anisotropia nenhuma*.
    livre: Option<(f32, f32)>,
    /// O que a metade do COLAPSO faz. ⚠️ Ela é do PRODUTO (o `passe_nos_motores`
    /// colapsa antes de refinar) e a varredura mede as três, porque *uma metade
    /// que custa `Q` — ou que custa ÂNGULO — tem de aparecer numa coluna*.
    colapso: Colapso,
}

/// ⛔⛔⛔ **As três células, e a do meio é a que decide.** Medido em 18/09: o
/// colapso ENVIESADO leva o pior triângulo da faixa de `4,56°` para **`1,96°`**
/// — ele funde preferencialmente as diagonais, e fundir uma diagonal numa
/// configuração fina deixa uma LASCA. *O `Q` não vê isso*, que é exactamente o
/// que o gate `o_pente_nao_compra_alinhamento_com_lascas` existe para apanhar.
#[derive(Clone, Copy, PartialEq)]
enum Colapso {
    /// Nem corre — o regime da fixtura até 18/09. ⚠️ Não é o do produto.
    Fora,
    /// Corre com o alvo NU, que é o que o produto fazia antes desta wave.
    Nu,
    /// Corre com o campo do pente normalizado como o do refino (mínimo em
    /// `base`) — ⛔ ele funde arestas **mais longas** do que antes, e é essa a
    /// origem das lascas.
    Enviesado,
    /// Corre com o campo do pente normalizado pelo **MÁXIMO** (`1/(1+k)`): ele
    /// pode fundir MENOS do que o passe isotrópico e nunca mais.
    Conservador,
}

/// O campo que uma corrida entrega ao passe — o do produto, ou o da sonda.
///
/// ⛔⛔ **`pente = 0` tem de desarmar os DOIS ramos**, e a 1.ª redacção desarmava
/// só o do produto: o ramo livre ignorava o argumento, logo as células `Nu` e
/// `Enviesado` do colapso liam **exactamente o mesmo número** e a varredura
/// dizia que a metade do colapso não tinha efeito nenhum. *Uma sonda que não
/// distingue os dois lados de uma pergunta responde sempre «não há diferença».*
fn campo(
    vies: Vies,
    base: f32,
    direccao: [f32; 3],
    pente: f32,
    porta: ph2d_rake::Porta,
) -> Box<dyn Fn([f32; 3], [f32; 3]) -> f32 + Sync> {
    match vies.livre {
        None => Box::new(ph2d_rake::campo_do_pente(base, direccao, pente, porta)),
        // ⛔⛔ **As DUAS cercas do produto têm de estar aqui**, senão a sonda mede
        // outro programa: `pente = 0` desarma, e **sem TRAÇO o campo é o alvo
        // NU**. A 1.ª redacção esquecia a segunda e lia `Q 0,0598` onde o
        // produto lê `0,0435` — *um carimbo de vinte e quatro, e a sonda
        // escolhia o número da lei com ele*.
        Some(_) if pente <= 0.0 || direccao.iter().all(|c| *c == 0.0) => {
            Box::new(move |_p: [f32; 3], _u: [f32; 3]| base)
        }
        Some((k, escala)) => Box::new(move |_p: [f32; 3], u: [f32; 3]| {
            base * escala * (1.0 - k * ph2d_rake::quatro_dobras(direccao, u))
        }),
    }
}

/// Um traço recto ao longo de `+x`, com o passe de topologia a correr antes de
/// cada carimbo quando `refina`.
fn traco(pente: f32, refina: bool) -> (Mesh, Vec<[f32; 3]>) {
    traco_com(
        pente,
        refina,
        Vies {
            livre: None,
            colapso: Colapso::Enviesado,
        },
    )
}

fn traco_com(pente: f32, refina: bool, vies: Vies) -> (Mesh, Vec<[f32; 3]>) {
    let mut malha = chapa(61, 3.0);
    let brush = pincel(pente);
    let mut stroke = SculptStroke::default();
    stroke.begin(&malha);
    let mut births = Vec::new();
    let mut remap = ph2d_mesh::Remap::default();
    let mut region = ph2d_mesh::RegionScratch::default();
    let mut centros = Vec::new();
    const ALVO: f32 = 0.035;
    for k in 0..24 {
        let centro = [-1.2 + 0.1 * k as f32, 0.0, 0.0];
        centros.push(centro);
        if refina {
            // ⭐⭐⭐ **O PASSE RECEBE O CAMPO DO PENTE** — é AQUI que o
            // alinhamento nasce, e é por isso que este gate tem de o conter.
            //
            // ⚠️ **A direcção é lida ANTES do carimbo**, que é exactamente o
            // instante em que o produto a lê: o `last_center` ainda descreve o
            // carimbo anterior, e a porta é a mesma
            // ([`SculptStroke::direccao_do_traco`]) — *uma segunda escrita dela
            // poria o refino a pentear numa direcção e o deslocamento noutra,
            // com os dois gates verdes.*
            let direccao = stroke.direccao_do_traco(centro);
            // ⚠️ **A ORDEM é a do produto: o COLAPSO primeiro.** As duas metades
            // falam com o traço em voo por canais diferentes — o colapso por uma
            // renumeração, o refino por uma lista de nascimentos —, e o segundo
            // afirma que a malha cresceu exactamente o que ele partiu.
            if vies.colapso != Colapso::Fora {
                let alvo = ph2d_mesh::collapse_target(ALVO);
                // ⚠️ `Nu` passa `pente = 0`, e o campo devolve o alvo **ao bit**
                // — é o mesmo caminho, sem um segundo braço de chamada.
                let forca = if vies.colapso == Colapso::Nu {
                    0.0
                } else {
                    brush.pente
                };
                // ⚠️ A normalização CONSERVADORA põe o máximo do campo em `alvo`
                // em vez do mínimo — ver [`Colapso::Conservador`]. ⭐ No caminho
                // do PRODUTO (`livre: None`) ela é a própria
                // [`ph2d_rake::Porta::Colapso`], e esta linha é inerte: *a sonda
                // reproduz à mão o que a lei faz, para poder medir a alternativa
                // que a lei não escolheu.*
                let vies_do_colapso = if vies.colapso == Colapso::Conservador {
                    Vies {
                        livre: vies.livre.map(|(k, _)| (k, 1.0 / (1.0 + k))),
                        ..vies
                    }
                } else {
                    vies
                };
                let campo = campo(
                    vies_do_colapso,
                    alvo,
                    direccao,
                    forca,
                    ph2d_rake::Porta::Colapso,
                );
                if matches!(
                    ph2d_mesh::collapse_in_sphere_sized(
                        &mut malha,
                        centro,
                        brush.radius,
                        alvo,
                        Some(&campo),
                        &mut remap,
                        &mut region,
                    ),
                    ph2d_mesh::Collapse::Done { .. }
                ) {
                    stroke.shrink_with(&remap);
                }
            }
            let campo = campo(vies, ALVO, direccao, brush.pente, ph2d_rake::Porta::Refino);
            let _ = ph2d_mesh::refine_in_sphere_sized(
                &mut malha,
                centro,
                brush.radius,
                ALVO,
                Some(&campo),
                &mut births,
                &mut region,
            );
            stroke.grow_with(&malha, &births);
            // ⭐⭐⭐ **A terceira metade**, na ordem do produto: depois do refino.
            if brush.pente > 0.0 {
                let preferencia = ph2d_rake::preferencia_do_pente(direccao, brush.pente);
                ph2d_mesh::alinha_arestas(
                    &mut malha,
                    centro,
                    brush.radius,
                    &preferencia,
                    &mut region,
                );
            }
        }
        stroke.dab(
            &mut malha,
            &brush,
            &Dab::at(centro, brush.radius, [0.0, 0.0, -1.0]),
            Symmetry::default(),
        );
    }
    (malha, centros)
}

/// ⭐⭐⭐ **A MEDIÇÃO: o `G-2` sobre a nossa saída.**
///
/// Barra do corpus: `Q ≥ +0,0465` com o pente no máximo e `Q ≤ +0,0465` com ele
/// desligado — a **mesma** barra, o meio de um vale cujos dois lados são saída do
/// próprio alvo.
#[test]
#[ignore = "sonda"]
fn diag_o_g2_sobre_a_nossa_saida() {
    for refina in [false, true] {
        let etiqueta = if refina { "com refino" } else { "sem refino" };
        let (m0, c0) = traco(0.0, refina);
        let (m1, c1) = traco(1.0, refina);
        let (q0, n0) = ph2d_sculpt3d_q(&m0, &c0, 0.30);
        let (q1, n1) = ph2d_sculpt3d_q(&m1, &c1, 0.30);
        println!(
            "{etiqueta}:  Q desligado {q0:+.4} (n={n0})  ·  Q no maximo {q1:+.4} (n={n1})  \
             ·  ΔQ {:+.4}  ·  barra +0,0465  ⇒  {}",
            q1 - q0,
            if q1 >= 0.0465 { "PASSA" } else { "FALHA" }
        );
    }
}

/// ⭐⭐⭐ **A NOSSA MALHA PENTEIA-SE ACIMA DA BARRA DO ORÁCULO — e é isto que
/// responde à pergunta que a espec deixou EM ABERTO.**
///
/// A espec §14.5 manda: *faça a hipótese **(b)** primeiro — a relaxação dentro
/// do laço por-carimbo — e meça contra o `G-2`; se a nossa malha ficar abaixo da
/// barra com a lei já no sítio certo, a causa é a **(a)** e há uma segunda lei a
/// construir.* Medido, no regime em que a barra foi calibrada (com o passe de
/// refino, que é o da fixtura `escada/k_*`):
///
/// | | `Q` desligado | `Q` no máximo | `ΔQ` |
/// |---|---|---|---|
/// | **com refino** | `−0,0192` | **`+0,1471`** | `+0,1664` |
/// | sem refino | `+0,0541` | `+0,3322` | `+0,2781` |
///
/// ⇒ **`+0,1471` contra a barra de `+0,0465`: `3,2×`.** Pelo critério que a
/// própria espec prescreveu, **NÃO há segunda lei a construir.**
///
/// ⚠️⚠️ **E a indicação que sugeria o contrário era falível, e estava errada.** A
/// §14.3.1 extrapola a composição em série em `1/n` e aterra em `+0,0393`,
/// **abaixo** da barra — o que o §14.5.2 lê como *«(b) sozinha pode não
/// chegar»*, com quatro fraquezas declaradas. Ela chegou. *Uma indicação
/// declarada como falível que se mede e cai é a declaração a funcionar.*
///
/// ⭐⭐ **E há uma concordância INDEPENDENTE que ninguém forçou:** a espec §2.2
/// mede que no alvo o efeito é **MAIOR sem o passe de refino** (`ΔQ = +0,2061`
/// contra `+0,1835` com ele). A nossa lei, escrita sem olhar para esse número,
/// lê `+0,2781` contra `+0,1664` — *a mesma ordem, pelo mesmo mecanismo: os
/// vértices que o refino insere não sabem nada do traço.*
///
/// ⛔ **A linha «sem refino» NÃO entra no gate**, e a razão é da fixtura: ali o
/// lado desligado lê `+0,0541`, ligeiramente **acima** da barra, porque uma
/// chapa sacudida a `0,7` do passo ainda é meio alinhada. *A barra foi
/// calibrada no regime COM refino, e é nele que ela afirma.*
#[test]
fn a_nossa_malha_penteia_se_acima_da_barra_do_oraculo() {
    /// A barra do corpus: o MEIO do vale, e ela é uma só — `≥` de um lado e `≤`
    /// do outro. ⛔ Não é um número escolhido: os dois lados do vale
    /// (`+0,0298` e `+0,0632`) são saída do **próprio alvo**.
    const BARRA: f64 = 0.0465;

    let (m0, c0) = traco(0.0, true);
    let (m1, c1) = traco(1.0, true);
    let (q0, n0) = ph2d_sculpt3d_q(&m0, &c0, 0.30);
    let (q1, n1) = ph2d_sculpt3d_q(&m1, &c1, 0.30);

    // (1) — **o controlo, e sem ele as outras duas metades não afirmam nada:**
    // o lado DESLIGADO tem de estar abaixo da barra. Uma fixtura cuja malha já
    // nasce alinhada com o traço passa a metade (2) sem o pente fazer nada — foi
    // exactamente o que a 1.ª redacção desta chapa fazia (`+0,2885`).
    assert!(
        n0 > 3_000 && n1 > 3_000,
        "a populacao encolheu ({n0} e {n1} arestas na faixa) — o arranjo deixou \
         de conter o que este gate mede"
    );
    assert!(
        q0 <= BARRA,
        "com o pente DESLIGADO a nossa malha ja' le' Q {q0:+.4} contra a barra \
         de {BARRA:+.4} (medido −0,0192) — a fixtura nasce alinhada com o traco \
         e a metade (2) passa a afirmar o vazio"
    );

    // (2) — e o lado LIGADO passa a barra do oráculo. **É esta linha que diz que
    // a hipótese (b) chega**, e com ela a pergunta «uma lei ou duas» fecha.
    assert!(
        q1 >= BARRA,
        "com o pente no maximo a nossa malha le' Q {q1:+.4} contra a barra de \
         {BARRA:+.4} (medido +0,1471) — se isto reprovou, a causa e' a (a) da \
         espec §14.4 e ha' uma SEGUNDA lei a construir: o pente do alvo enviesa \
         as decisoes do passe de topologia, e a relaxacao sozinha nao chega"
    );
}

/// O PIOR ÂNGULO de triângulo da faixa, em graus — a segunda coluna, e sem ela a
/// escada do `Q` aprovaria uma malha destruída que por acaso ficou alinhada.
/// A ESCADA do nosso botão — é dela que sai a faixa, nunca do alvo.
///
/// ⚠️ **DUAS colunas de propósito:** o `Q` sozinho aprovaria uma malha destruída
/// que por acaso ficou alinhada. A segunda é o pior ângulo de triângulo da faixa.
#[test]
#[ignore = "sonda"]
fn diag_a_escada_do_pente() {
    for p in [
        0.0f32, 0.0625, 0.125, 0.25, 0.375, 0.5, 0.625, 0.75, 1.0, 1.25, 1.5, 2.0, 3.0,
    ] {
        let (m, c) = traco(p, true);
        let (q, n) = ph2d_sculpt3d_q(&m, &c, 0.30);
        let (ang, _) = ph2d_sculpt3d_ang(&m, &c, 0.30);
        println!("pente {p:.4}   Q {q:+.4}   pior angulo {ang:6.2}°   (n={n})");
    }
}

/// ⭐⭐⭐ **O PENTE NÃO COMPRA ALINHAMENTO COM LASCAS — a segunda coluna, e ela
/// apanhou um defeito que a primeira aprovava.**
///
/// A 1.ª redacção da lei rodava cada aresta mantendo o comprimento **dela**.
/// Medido nesta chapa, a régua do alinhamento subia bonito **e** o pior ângulo
/// de triângulo da faixa desabava:
///
/// | pente | `Q` | pior ângulo, lei 1.ª | pior ângulo, lei de hoje |
/// |---|---|---|---|
/// | `0,000` | `−0,019` | `7,86°` | `7,86°` |
/// | `0,250` | `+0,045` | `3,33°` | **`8,21°`** |
/// | `0,500` | `+0,085` | `0,65°` | `6,57°` |
/// | `1,000` | `+0,127` | **`0,31°`** | **`4,56°`** |
/// | `3,000` | `+0,179` | `0,10°` | `0,62°` |
///
/// ⛔ *Um triângulo de três décimos de grau não tem normal utilizável*, e a
/// régua do `Q` **aprovava**, porque ela só vê direcções. A cura foi o alvo de
/// cada aresta passar a ser o eixo vezes o raio **MÉDIO do anel**: a
/// configuração para que o vértice é puxado é uma cruz regular, logo a lei
/// alinha **e** regulariza.
///
/// ⭐⭐ **E na metade de baixo do curso ela MELHORA a malha** (`8,21°` a `0,25`
/// contra `7,86°` desligada) — o pente desfaz as lascas que o próprio refino
/// deixa.
#[test]
fn o_pente_nao_compra_alinhamento_com_lascas() {
    /// O pior ângulo que a faixa pode ter com o pente no tecto. ⛔ Não é um
    /// número escolhido: ele separa o `4,56°` que a lei de hoje entrega do
    /// `0,31°` que a lei refutada entregava, com margem dos dois lados.
    const CHAO_DO_ANGULO: f64 = 2.0;

    let (m0, c0) = traco(0.0, true);
    let (base, n_base) = ph2d_sculpt3d_ang(&m0, &c0, 0.30);

    // (1) — **o controlo:** a malha por pentear já tem lascas (o refino
    // deixa-as), senão não há o que piorar e as outras metades não afirmam nada.
    assert!(
        (4.0..12.0).contains(&base),
        "a malha por pentear le' um pior angulo de {base:.2}° (medido 7,86°) — \
         o arranjo mudou, e as barras abaixo foram calibradas contra este numero"
    );

    // ⛔ **E o PISO DA POPULAÇÃO, que é o par que a porta devolve por uma
    // razão:** uma faixa VAZIA lê `180°` — *«a malha está perfeita»* —, e as
    // três metades abaixo ficariam verdes a medir o nada.
    assert!(
        n_base > 100,
        "a faixa do traco tem {n_base} triangulo(s): a fixtura deixou de conter \
         o fenomeno, e um `180°` de faixa vazia le'-se como malha perfeita"
    );

    // (2) — no tecto do botão a faixa continua a ter triângulos com normal.
    let (m1, c1) = traco(1.0, true);
    let (no_tecto, n_tecto) = ph2d_sculpt3d_ang(&m1, &c1, 0.30);
    assert!(
        n_tecto > 100,
        "a faixa com o pente no tecto tem {n_tecto} triangulo(s)"
    );
    assert!(
        no_tecto >= CHAO_DO_ANGULO,
        "com o pente no tecto o pior triangulo da faixa mede {no_tecto:.2}° \
         (medido 4,56°; a lei refutada media 0,31°) — o pente voltou a comprar \
         alinhamento com lascas, e a regua do Q nao ve' isso"
    );

    // (3) — e na metade de baixo ele **não piora** a malha. ⚠️ Sem esta metade,
    // uma lei que degradasse tudo por igual passaria a (2) com o tecto baixo.
    let (mm, cm) = traco(0.25, true);
    let (a_um_quarto, _) = ph2d_sculpt3d_ang(&mm, &cm, 0.30);
    assert!(
        a_um_quarto >= base * 0.95,
        "a um quarto do curso o pior triangulo mede {a_um_quarto:.2}° contra \
         {base:.2}° por pentear (medido 8,21 contra 7,86) — o pente deixou de \
         desfazer as lascas que o refino deixa e passou a criar as dele"
    );
}

/// ⭐⭐⭐ **GATE — um traço PENTEADO desfaz-se INTEIRO.**
///
/// # ⛔⛔ A família que este módulo pagou DUAS vezes
///
/// O tecido em 05/09 e a pose em 14/09: um verbo que **desvia** do laço
/// por-vértice do `dab_core` move barro sem passar pelo `capture`, o
/// `close_stroke` vê a janela vazia e **devolve cedo** — *uma janela vazia e um
/// gesto que não fez nada são o mesmo byte para quem grava*, e o `Ctrl+Z` não
/// tem o que desfazer.
///
/// ⚠️⚠️ **E aqui o risco tem uma volta a mais: a pegada do pente NÃO é a do
/// verbo.** Ele corre sobre os vizinhos de quem o carimbo tocou, logo move
/// vértices que o `dab_core` nunca viu — e são exactamente esses que ficariam
/// fora da janela se o `capture` dele não existisse.
///
/// # As quatro metades
///
/// 1. **O controlo:** o pente moveu vértices que o verbo desligado não move.
///    Sem ele, o gate ficaria verde sobre um pente inerte.
/// 2. A janela não está vazia.
/// 3. O `pre` de cada vértice é o do **pen-down**, não o do evento em que ele
///    entrou — senão o desfazer devolve o meio do arrasto.
/// 4. **Nenhum vértice movido fica de fora**, que é a metade que o pente pede.
#[test]
fn um_traco_penteado_desfaz_se_inteiro() {
    let antes = chapa(61, 3.0).positions().to_vec();

    // (1) — o CONTROLO, e ele é a DIFERENÇA e não a contagem: medido, o pente
    // move `664` dos originais e o traço sem ele move os MESMOS `664` — ele
    // trabalha dentro da pegada do carimbo, logo *contar quantos mexeram não
    // distingue os dois*. O que distingue é o barro estar noutro sítio.
    let (m0, _) = traco(0.0, true);
    let diferentes = m0
        .positions()
        .iter()
        .zip(traco(1.0, true).0.positions())
        .filter(|(a, b)| a != b)
        .count();

    let mut malha = chapa(61, 3.0);
    let brush = pincel(1.0);
    let mut s = SculptStroke::default();
    s.begin(&malha);
    let mut births = Vec::new();
    let mut region = ph2d_mesh::RegionScratch::default();
    for k in 0..24 {
        let centro = [-1.2 + 0.1 * k as f32, 0.0, 0.0];
        let _ = ph2d_mesh::refine_in_sphere(
            &mut malha,
            centro,
            brush.radius,
            0.035,
            &mut births,
            &mut region,
        );
        s.grow_with(&malha, &births);
        s.dab(
            &mut malha,
            &brush,
            &Dab::at(centro, brush.radius, [0.0, 0.0, -1.0]),
            Symmetry::default(),
        );
    }

    // ⚠️ A comparação é sobre o PREFIXO: o refino acrescentou vértices, e um
    // vértice que NASCEU no traço não tem posição de antes para comparar.
    let mexidos = antes
        .iter()
        .zip(malha.positions())
        .filter(|(a, b)| a != b)
        .count();
    assert!(
        diferentes > 100,
        "o traco com o pente e o traco sem ele deixaram a malha em {diferentes} \
         posicoes diferentes — o pente esta' INERTE neste arranjo e as tres \
         metades abaixo mediriam o undo do carimbo, nao o dele"
    );
    assert!(
        mexidos > 100,
        "so' {mexidos} vertices ORIGINAIS mexeram — o arnes nao moveu nada de \
         jeito e o gate mediria vacuo"
    );
    assert!(
        !s.touched().is_empty(),
        "a janela do undo saiu VAZIA depois de mover {mexidos} vertices — o \
         `close_stroke` devolve cedo e o Ctrl+Z nao tem o que desfazer"
    );
    assert_eq!(
        s.touched().len(),
        s.base_positions().len(),
        "a janela e o `pre` tem de andar em par"
    );
    let janela: std::collections::BTreeMap<u32, [f32; 3]> = s
        .touched()
        .iter()
        .copied()
        .zip(s.base_positions().iter().copied())
        .collect();
    for (v, pre) in &janela {
        if let Some(a) = antes.get(*v as usize) {
            assert_eq!(
                pre, a,
                "o vertice {v} entrou na janela com uma pose INTERMEDIA — o \
                 Ctrl+Z devolveria o meio do traco"
            );
        }
    }
    let esquecidos = antes
        .iter()
        .zip(malha.positions())
        .enumerate()
        .filter(|(i, (a, b))| a != b && !janela.contains_key(&(*i as u32)))
        .count();
    assert_eq!(
        esquecidos, 0,
        "{esquecidos} vertices mexeram e ficaram FORA da janela — o pente corre \
         sobre os VIZINHOS da pegada do verbo, logo e' ele quem tem de os \
         fotografar, e o Ctrl+Z devolveria a peca pela metade"
    );
}

/// **A varredura que escolheu as constantes** — o instrumento, não a lei.
#[path = "rake_varredura_tests.rs"]
mod varredura;
