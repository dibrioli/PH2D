//! **OS GATES DA CENA DO PENTE** — irmão (`#[path]`, `cfg(test)`) do [`super`].
//!
//! ⛔⛔ **A pergunta que eles fazem é a que a `=45` custou um report para
//! ensinar:** *esta cena tem região UTILIZÁVEL?* Lá a resposta era `d/R` e a cena
//! não tinha nenhuma — *ou o barro viajava seis raios através da peça, ou nada
//! acontecia*, e a foto do dono era o único resultado possível. Aqui é a mesma
//! pergunta com outra grandeza: **na peça com que esta cena abre, e no rumo que
//! o roteiro manda riscar, subir o knob muda a malha de forma legível — e sem a
//! estragar?**
//!
//! ⚠️ **A régua é a PORTA** ([`ph2d_sculpt3d::medida_do_pente`]), a mesma que a
//! bancada da lei corre sobre a chapa. *Uma cópia aqui divergiria na primeira
//! wave que mexesse numa delas, e a que o dono vê é a que envelhece.*

use ph2d_sculpt3d::medida_do_pente::{grade_da_faixa, lascas, pior_angulo, q_da_faixa};
use ph2d_sculpt3d::{Brush, Dab, SculptStroke, Symmetry, Verb};

/// ⭐⭐⭐ **O RAIO COM QUE A CENA É MEDIDA — DERIVADO, e não escolhido.**
///
/// ⛔⛔ **A 1.ª redacção cravava `0,35` aqui, «o do corpus do oráculo», e o gate
/// ficava VERDE enquanto o dono via `não percebi diferença`:** o app dá
/// **`0,1634`** (o raio de fábrica de `50 px` convertido pela câmara que enquadra
/// a peça), ou seja **metade**. *Um gate que escolhe o seu próprio raio mede
/// outro programa.*
fn raio_do_app() -> f32 {
    use crate::Camera3d;
    let (vw, vh) = (1920.0f32, 1080.0f32);
    let mut cam = Camera3d {
        yaw: 0.6,
        pitch: 0.35,
        ..Camera3d::default()
    };
    cam.frame(peca_uma_vez().bounds(), vw / vh);
    cam.world_radius_for_screen_px(
        [0.0, 0.0, 1.0],
        ph2d_panel_sculpt3d::state::Sculpt3dUi::default().radius_px,
        (vw as u32, vh as u32),
    )
}

/// ⭐⭐⭐ **O ALVO DO REFINO — DERIVADO do `Detail` que a CENA arma**, pela mesma
/// escada ancorada em ÁREA que o passe usa.
///
/// ⚠️ **É a metade que faltava ao gate:** com o `Detail` de fábrica o alvo
/// (`0,0805`) é mais GROSSO que a aresta da peça (`0,0527`) e o passe engrossa;
/// com o que a cena arma ele é `0,0175`, e é aí que o pincel tem o que pentear.
fn alvo_do_refino() -> f32 {
    let peca = peca_uma_vez();
    ph2d_mesh::edge_for_tri_count(
        peca.surface_area(),
        ph2d_mesh::tris_for_detail(super::DETALHE_DA_CENA),
    )
}

/// A barra do corpus: o vale MEDIDO cujos dois lados são saída do próprio alvo.
const BARRA: f64 = 0.0465;

/// Abaixo de quantos graus um triângulo deixa de ter normal utilizável na tela.
///
/// ⚠️ **Ele não é escolhido:** é o degrau em que a contagem da bancada separa os
/// dois lados — `0` triângulos em todo o curso a `30°` e a `60°` da grade, e
/// `1`–`3` só a `45°` exactos.
const LIMIAR_DA_LASCA: f64 = 5.0;

/// Quantas lascas a cena pode tolerar no tecto do botão.
///
/// ⛔ **UMA, e o número é MEDIDO:** no regime do app a contagem lê `0` em quatro
/// dos cinco rumos e **`1`** a `45°` exactos, em `~2 200` triângulos — que é a
/// *linha de água* dos quatro dobras, declarada no cabeçalho da
/// [`ph2d_sculpt3d::medida_do_pente`]. ⚠️ *Na esfera UV que esta cena quase abriu
/// ela lia `3`*, e é essa a ordem de grandeza que este número guarda.
const LASCAS_TOLERADAS: usize = 1;

/// Quanto GRÃO a peça da cena pode ter antes de ela própria fazer o trabalho.
///
/// ⛔ **Medido dos dois lados:** a peça desta cena lê no máximo `+0,0516` e uma
/// esfera UV lê `+0,55`. *Uma ordem de grandeza entre os dois lados é o que uma
/// barra honesta separa.*
const GRAO_MAXIMO: f64 = 0.15;

/// **O ZERO da régua do olho** — a fracção que uma malha SEM direcção nenhuma lê.
///
/// ⭐ Ele não é escolhido: o desvio à grade de uma direcção qualquer é uniforme
/// em `[0°, 45°]`, logo as três faixas de `15°` enchem-se por igual. Medido
/// nesta peça com o pente desligado: `32,4 %` a `38,1 %` (o `45°` é o mais alto
/// — ⚠️ *o resto de direcção que sobra a um remalhador isotrópico aparece nesta
/// régua e NÃO no `Q`, que ali lê `+0,0078`: uma média cancela o que uma
/// contagem mostra*).
const GRADE_ISOTROPICA: f64 = 100.0 / 3.0;

/// **Quanto GRADE a peça pode ter ANTES de o pente lhe tocar** — o controlo.
///
/// ⛔ **A barra sai de um vale MEDIDO com esta mesma porta:** a peça desta cena
/// lê `32,4 %`–`38,1 %` nos quatro rumos (o `45°` é o mais alto — um resto de
/// direcção que o `Q` não vê, porque ele é uma média) e uma **esfera UV** lê
/// `62,2 %`–`66,6 %`. *O que esta metade separa é «a peça tem um resto» de «a
/// peça JÁ É uma grade», e isso é meia tabela de distância.*
const GRADE_DO_GRAO_MAXIMO: f64 = 45.0;

/// **Que fracção das arestas da faixa tem de correr com a grade do traço.**
///
/// ⛔⛔⛔ **É a barra que faltava no dia em que o dono reprovou a cena com foto**
/// (*«pouca ou nenhuma diferença»*, 18/09) — com as outras quatro metades deste
/// gate VERDES, e o arame dos dois lados do controlo indistinguível.
///
/// ⭐ **O número é o MEIO de um vale cujos dois lados são leis REAIS**, medido
/// nos quatro rumos com esta porta:
///
/// | chão do flip | grade, pior rumo | pior ângulo | lascas |
/// |---|---|---|---|
/// | desligado | `32,4 %` | `22,8°` | `0` |
/// | `24°` (a lei REPROVADA) | **`35,9 %`** | `22,5°` | `0` |
/// | `20°` | `37,0 %` | `17,3°` | `0` |
/// | `18°` | `38,4 %` | `5,5°` | `0` |
/// | **`16°` (a de hoje)** | **`42,1 %`** | `8,0°` | `0` |
/// | `14°` | `44,7 %` | `5,1°` | `0` |
/// | `12°` | `46,7 %` | `3,2°` | `1` ⛔ |
///
/// ⇒ o vale é `[39,8 ; 42,1]` (o melhor rumo da lei reprovada contra o pior da
/// de hoje) e `41,0` é o meio dele.
///
/// ⭐⭐ **E o lado APROVADO está medido:** a mesma régua sobre a saída do PRÓPRIO
/// alvo (`fixtures/rake/rotacao/*_p100`) lê **`43,6 %`** contra `34,1 %`
/// desligado — *a nossa lei entrega a classe dele*.
const GRADE_MINIMA: f64 = 41.0;

/// Quantos vértices um traço penteado tem de DESLOCAR para o artista ver.
///
/// ⛔⛔ **É a metade que faltava, e ela custou um report** (*«não percebi
/// diferença»*): o `Q` é uma MÉDIA e sobe com um punhado de arestas alinhadas —
/// no `Detail` de fábrica ele lia `+0,1926`, o maior de todos, movendo **`121`**
/// vértices num traço inteiro. *Uma régua que só vê a fracção não vê a
/// magnitude*, que é a mesma forma que o `Density` custou. Medido no regime que
/// a cena arma: **`2 715`**.
const MOVIDOS_MINIMOS: usize = 1_000;

/// Os rumos em que a cena é medida.
///
/// ⛔ **São QUATRO porque o roteiro promete que «todos funcionam».** Um gate com
/// um rumo só aprovaria uma peça com grão, desde que o rumo medido fosse o
/// atravessado — que é exactamente o que a 1.ª redacção desta cena fazia.
const RUMOS: [(&str, [f32; 2]); 4] = [
    ("ao longo de x", [1.0, 0.0]),
    ("30 graus", [0.866_025_4, 0.5]),
    (
        "45 graus",
        [
            std::f32::consts::FRAC_1_SQRT_2,
            std::f32::consts::FRAC_1_SQRT_2,
        ],
    ),
    ("atravessado (y)", [0.0, 1.0]),
];

/// A peça da cena, construída **uma vez** e clonada por traço.
///
/// ⚠️ **O remalhador custa `~270 ms`** e este gate corre oito traços: sem o cache
/// o gate paga `2,2 s` só a refazer a mesma bola. ⛔ Ele é um cache de ARNÊS e
/// não do produto — a cena constrói a peça uma vez, ao abrir.
fn peca_uma_vez() -> ph2d_mesh::Mesh {
    static PECA: std::sync::OnceLock<ph2d_mesh::Mesh> = std::sync::OnceLock::new();
    PECA.get_or_init(super::peca).clone()
}

/// Um traço sobre a peça da cena, no rumo `e`, com o refino a correr antes de
/// cada carimbo — que é o regime em que a `=49` abre (interruptor ARMADO, ver
/// [`super::arma`]).
fn traco(pente: f32, e: [f32; 2]) -> (ph2d_mesh::Mesh, Vec<[f32; 3]>) {
    traco_com(pente, e, raio_do_app(), alvo_do_refino())
}

/// ⭐⭐⭐ **GATE — a `=49` TEM O QUE MOSTRAR, e em QUALQUER rumo de traço.**
///
/// # As quatro metades, e porque nenhuma basta
///
/// 1. **A faixa existe.** Uma faixa vazia lê `Q = 0` e `180°` — exactamente o que
///    uma malha sem direcção e uma malha perfeita leem. *Um zero de «não medido»
///    e um de «sem direcção» são o mesmo byte.*
/// 2. **A peça NÃO tem grão:** o `Q` desligado fica **abaixo** da barra nos
///    quatro rumos. ⛔ Sem esta metade, a esfera UV de sempre passaria — ela lê
///    `+0,55` sem ninguém lhe tocar, e a cena aprovaria um pente **inerte**.
/// 3. **O pente VIRA a malha**, e isso são DUAS asserções porque são duas
///    afirmações: o `Q` sobe **mais do que a barra** (o que o artista vê é a
///    mudança), e **troca de SINAL** — de uma malha a cruzar o traço para uma a
///    correr com ele. ⚠️ Sem a segunda, a esfera UV que esta cena quase abriu
///    passaria: lá o Δ é MAIOR (`+0,18`) e o `Q` acaba em `−0,008`, ou seja sem
///    grade nenhuma. ⛔ E uma barra ABSOLUTA sobre o `Q` final seria falsa: ela
///    é medida em `3` dos `4` rumos (a `30°` um traço só chega a `+0,020`), e
///    escrevê-la faria o gate reprovar sobre produto correcto.
/// 4. ⛔ **E a SEGUNDA COLUNA**, que é o que separa este gate de uma promessa: o
///    `Q` sozinho aprova uma malha destruída que por acaso ficou alinhada —
///    medido nesta linha, a lei refutada levava o pior triângulo a `0,31°`
///    **enquanto o `Q` subia**. ⚠️ A coluna é a **CONTAGEM** e não o mínimo, pela
///    razão que o doc da [`lascas`] traz.
#[test]
fn a_cena_do_pente_tem_o_que_mostrar() {
    for (nome, e) in RUMOS {
        let (m0, c0) = traco(0.0, e);
        let (m1, c1) = traco(1.0, e);
        let raio = raio_do_app();
        let (q0, n0) = q_da_faixa(&m0, &c0, raio);
        let (q1, n1) = q_da_faixa(&m1, &c1, raio);
        let (finas, total) = lascas(&m1, &c1, raio, LIMIAR_DA_LASCA);
        let (pior, _) = pior_angulo(&m1, &c1, raio);

        assert!(
            n0 > 200 && n1 > 200 && total > 100,
            "{nome}: a faixa tem {n0}/{n1} arestas e {total} triangulos — a peca \
             da cena deixou de conter o fenomeno, e um `Q` de faixa vazia le'-se \
             igual a uma malha sem direccao"
        );
        // ⛔ **A barra do grão é `0,15` e não a do corpus, e os dois números
        // estão medidos:** a peça desta cena lê no máximo **`+0,0516`** (o rumo
        // atravessado — o remalhador não é perfeitamente isotrópico) e uma
        // esfera UV lê **`+0,55`**. *O que esta metade separa é «a peça já é uma
        // grade» de «a peça tem um resto de direcção», e isso é uma ordem de
        // grandeza, não um décimo.*
        assert!(
            q0 < GRAO_MAXIMO,
            "{nome}: a malha da =49 ja' nasce alinhada com o traco (Q desligado = \
             {q0:+.4}, barra {GRAO_MAXIMO:+.4}; medido no maximo +0,0516, e uma \
             esfera UV le' +0,55) — a cena aprovaria um pente INERTE, e o passo \
             (2) do roteiro nao seria comparacao nenhuma"
        );
        assert!(
            q1 - q0 > BARRA,
            "{nome}: o pente no maximo move o Q de {q0:+.4} para {q1:+.4} \
             (Delta {:+.4}, medido +0,074 no pior rumo) contra a barra do corpus \
             {BARRA:+.4} — o artista nao vai ver as linhas alinharem-se, e o \
             passo (3) do roteiro promete que ele ve'",
            q1 - q0
        );
        // ⭐ **E a malha deixa de CRUZAR o traço**, que é a metade que separa
        // *«alinhou»* de *«mexeu»*: `Q < 0` é uma malha a cruzar, `Q ≥ 0` é uma
        // que já não cruza. ⚠️ Um Δ grande sobre dois valores negativos seria a
        // esfera UV outra vez (`−0,188 → −0,008`, Δ `+0,18` e nenhuma grade).
        // ⛔ **A barra é `−0,01` e não `0`, e o número é MEDIDO:** a `30°` o
        // traço aterra em `−0,0001` — *ali ele neutraliza o cruzamento e não
        // constrói grade*, e escrever `> 0` reprovaria sobre produto correcto.
        assert!(
            q1 > -0.01,
            "{nome}: o Q foi de {q0:+.4} para {q1:+.4} e a malha continua a \
             CRUZAR o traco — ela so' se mexeu"
        );
        // ⛔⛔ **A MAGNITUDE**, que é a metade que o report do dono comprou.
        let movidos = m0
            .positions()
            .iter()
            .zip(m1.positions())
            .filter(|(a, b)| a != b)
            .count();
        assert!(
            movidos >= MOVIDOS_MINIMOS,
            "{nome}: o pente deslocou {movidos} vertices num traco inteiro \
             (medido 2 715; no `Detail` de fabrica eram 121, invisiveis) — o `Q` \
             pode estar alto na mesma, porque ele e' uma MEDIA"
        );
        assert!(
            finas <= LASCAS_TOLERADAS,
            "{nome}: a faixa ficou com {finas} triangulo(s) abaixo de \
             {LIMIAR_DA_LASCA}° em {total} (o pior mede {pior:.2}°; medido ZERO \
             nos quatro rumos) — sao as ESTRIAS que o `DEU ERRADO SE` do roteiro \
             nomeia, e o `Q` sozinho nao as ve'"
        );
        // ⭐⭐⭐ **A TERCEIRA COLUNA — a que o dono julga, e a que este gate não
        // tinha no dia em que ele reprovou a cena com foto.**
        //
        // ⛔⛔ As quatro metades acima ficaram TODAS verdes enquanto as duas
        // imagens do arame eram indistinguíveis: o `Q` é uma média, os
        // `movidos` contam diferenças **ao bit** (um vértice deslocado um
        // milionésimo entra na conta) e as lascas são uma cerca. *Nenhuma
        // responde «que fracção das arestas mudou de rumo», que é o que o olho
        // faz.*
        let (b0, nb0) = grade_da_faixa(&m0, &c0, raio);
        let (b1, nb1) = grade_da_faixa(&m1, &c1, raio);
        let (g0, g1) = (
            100.0 * b0[0] as f64 / nb0.max(1) as f64,
            100.0 * b1[0] as f64 / nb1.max(1) as f64,
        );
        assert!(
            g0 < GRADE_DO_GRAO_MAXIMO,
            "{nome}: a peca ja' nasce com {g0:.1} % das arestas na grade do \
             traco (isotropico e' {GRADE_ISOTROPICA:.1} %, medido 32,4–38,1 \
             nesta peca e 62,2 % numa esfera UV) — o controlo do passo (2) do \
             roteiro nao seria controlo nenhum"
        );
        assert!(
            g1 >= GRADE_MINIMA,
            "{nome}: com o pente no tecto so' {g1:.1} % das arestas da faixa \
             correm a menos de 15° da grade do traco, contra {GRADE_MINIMA:.1} % \
             (medido 42,1–45,2; o ALVO entrega 43,6 % e uma malha sem direccao \
             nenhuma le' {GRADE_ISOTROPICA:.1} %) — foi ISTO que o dono \
             reprovou com foto em 18/09, com as outras quatro metades verdes"
        );
        // ⛔⛔ **E a SUBIDA tem de acontecer em TODO rumo, que é a metade que a
        // absoluta não cobre.** Com a lei reprovada o `45°` **DESCIA** (`38,1 %`
        // desligado contra `36,0 %` no tecto): *a peça tem ali um resto de
        // direcção, e um pente fraco desarruma-o mais do que o alinha* — e o
        // `Q` daquela célula subia na mesma (`+0,0078 → +0,0657`), porque ele é
        // uma média e esta é uma contagem.
        assert!(
            g1 > g0,
            "{nome}: a grade foi de {g0:.1} % para {g1:.1} % — o pente DESARRUMOU \
             a faixa em vez de a alinhar (a lei reprovada em 18/09 fazia isto a \
             45°, e as outras metades deste gate ficavam verdes)"
        );
        eprintln!(
            "[=49] {nome:<16} Q {q0:+.4} -> {q1:+.4} · grade {g0:.1} % -> \
             {g1:.1} % · movidos {movidos} · lascas {finas}/{total} · \
             pior {pior:.2}°"
        );
    }
}

/// ⭐⭐⭐ **GATE — o FIO da `=49` está inteiro, das quatro pontas.**
///
/// ⛔⛔ **Ele existe porque o gate de cima mede a PORTA e não o FIO** — a lição
/// que o §24 desta família pagou: *um gate que chama a função em vez de percorrer
/// a rota afirma que a peça certa existe, nunca que a cena a usa*. O gate acima
/// chama [`super::peca`] directamente; se alguém trocar o braço do
/// `scenes_mesh.rs`, a cena passa a abrir na esfera de sempre — que lê
/// `Q = +0,3241`, ou seja **grão ao longo de qualquer risco** — e ele fica verde.
///
/// ⚠️ **A régua é o `include_str!` e não um `grep`**, e a diferença é o modo de
/// falha: um ficheiro que MUDA DE SÍTIO deixa isto de **compilar**, em vez de
/// varrer zero e ficar trivialmente verde.
///
/// ⚠️ **As quatro pontas são quatro defeitos diferentes:** a peça errada · o
/// prólogo que nunca corre (a cena abre com o interruptor DESLIGADO e o artista
/// vê o pente inerte no passo (2), que é o report do `Density` outra vez) · o
/// interruptor · o arame (sem ele a cena mede a sombra em vez da malha).
#[test]
fn a_cena_do_pente_esta_fiada() {
    const INPUT: &str = include_str!("input.rs");
    const SCENES: &str = include_str!("scenes.rs");
    const MESH: &str = include_str!("scenes_mesh.rs");
    const PENTE: &str = include_str!("scenes_pente.rs");
    // ⛔⛔ **E o ARNÊS deste ficheiro**, que é a ponta que faltava: uma mutação
    // que volte a cravar o raio e o alvo (`0,35` / `0,035`, os que a 1.ª
    // redacção escolheu) deixa **todas** as outras metades VERDES — o gate passa
    // a medir um regime que o artista não tem, que é exactamente o que custou o
    // report *«não percebi diferença»*.
    const ARNES: &str = include_str!("scenes_pente_tests.rs");
    const SCRIPTS: &str = include_str!("scripts.rs");

    // ⛔⛔⛔ **A AGULHA DESTA É MONTADA, e a razão é um defeito que este gate
    // teve:** escrita como literal, ela vivia **dentro do ficheiro que varre**,
    // logo `contains` encontrava-a a si própria e a mutação que cravava o raio
    // no arnês passava VERDE. *Um censo textual cuja agulha mora na fonte que
    // ele lê é satisfeito por si mesmo.*
    let deriva = format!(
        "traco_com(pente, e, {}(), {}())",
        "raio_do_app", "alvo_do_refino"
    );
    assert!(
        ARNES.contains(&deriva),
        "o arnes desta cena deixou de DERIVAR o regime do produto — com o raio e \
         o alvo cravados ele mede um regime que o artista nao tem, e todas as \
         outras metades ficam VERDES. Foi isso que custou o report «nao percebi \
         diferenca»"
    );

    for (o_que, fonte, agulha) in [
        (
            "o smoke chama o prologo",
            INPUT,
            "scenes::prologo(&mut scene)",
        ),
        ("o prologo arma esta cena", SCENES, "pente::arma(cena)"),
        ("a cena escolhe a peca dela", MESH, "scenes::pente::peca()"),
        (
            "o roteiro e' impresso",
            SCRIPTS,
            "scenes::pente::announce()",
        ),
        ("o interruptor e' armado", PENTE, "cena.toggle_dyntopo()"),
        ("o arame nasce ligado", PENTE, "cena.wireframe = true"),
        (
            "o Detail nasce no topo",
            PENTE,
            "cena.dyntopo.detail = DETALHE_DA_CENA",
        ),
    ] {
        assert!(
            fonte.contains(agulha),
            "{o_que}: `{agulha}` desapareceu. A =49 passa a abrir noutro estado e \
             o gate que mede a lei fica VERDE, porque ele chama a porta em vez de \
             percorrer a rota"
        );
    }
}

/// O mesmo traço do gate, com o raio e o alvo de refino como PARÂMETROS.
fn traco_com(pente: f32, e: [f32; 2], raio: f32, alvo: f32) -> (ph2d_mesh::Mesh, Vec<[f32; 3]>) {
    let mut malha = peca_uma_vez();
    malha.triangulate();
    let brush = Brush {
        verb: Verb::Draw,
        radius: raio,
        strength: 0.25,
        pente,
        ..Brush::default()
    };
    let mut stroke = SculptStroke::default();
    stroke.begin(&malha);
    let mut births = Vec::new();
    let mut remap = ph2d_mesh::Remap::default();
    let mut region = ph2d_mesh::RegionScratch::default();
    let mut centros = Vec::new();
    // ⚠️ O percurso anda **em raios de pincel**, não em unidades fixas: com um
    // pincel menor, um passo fixo seria um traço aos saltos.
    let passo = raio * 0.15;
    for k in 0..24 {
        let u = -passo * 12.0 + passo * k as f32;
        let centro = [u.sin() * e[0], u.sin() * e[1], u.cos()];
        centros.push(centro);
        // ⭐⭐⭐ **O PASSE RECEBE O CAMPO DO PENTE**, que é onde o alinhamento
        // mora — e sem esta linha o gate mediria a lei de deslocamento sozinha,
        // que reproduz o campo do alvo e **não produz grade nenhuma**
        // (`Q +0,0000` contra a barra). *Uma fixtura que corre a porta nua mede
        // um programa que o produto já não percorre.*
        //
        // ⚠️ **A direcção é lida ANTES do carimbo**, do `last_center` que ainda
        // descreve o dab anterior — o mesmo instante e a mesma porta que o
        // `refine_for_dab` usa.
        let direccao = stroke.direccao_do_traco(centro);
        // ⛔⛔ **O COLAPSO PRIMEIRO, como o produto** (`passe_nos_motores`), e a
        // ausência dele custava metade do efeito: sem esta metade a fixtura media
        // um passe que o app **não corre**, e o gate lia `ΔQ +0,0197` contra a
        // barra de `+0,0465` sobre uma lei que entrega `+0,074`. *Uma fixtura à
        // qual falta uma das duas metades do passe mede outro programa.*
        let alvo_do_colapso = ph2d_mesh::collapse_target(alvo);
        // ⭐⭐ **As TRÊS metades contribuem, e a do campo vale o DOBRO do flip
        // sozinho:** medido neste rumo, `Q +0,1292` só com a troca de diagonal
        // contra **`+0,2733`** com o campo de tamanho também. *Nenhuma das três
        // é decoração.*
        let campo_colapso = ph2d_sculpt3d::campo_do_pente(
            alvo_do_colapso,
            direccao,
            pente,
            ph2d_sculpt3d::Porta::Colapso,
        );
        if matches!(
            ph2d_mesh::collapse_in_sphere_sized(
                &mut malha,
                centro,
                brush.radius,
                alvo_do_colapso,
                Some(&campo_colapso),
                &mut remap,
                &mut region,
            ),
            ph2d_mesh::Collapse::Done { .. }
        ) {
            stroke.shrink_with(&remap);
        }
        let campo =
            ph2d_sculpt3d::campo_do_pente(alvo, direccao, pente, ph2d_sculpt3d::Porta::Refino);
        let _ = ph2d_mesh::refine_in_sphere_sized(
            &mut malha,
            centro,
            brush.radius,
            alvo,
            Some(&campo),
            &mut births,
            &mut region,
        );
        stroke.grow_with(&malha, &births);
        // ⭐⭐⭐ **A TERCEIRA METADE — alinhar TROCANDO diagonais**, a contagem
        // constante e sem poder piorar um triângulo.
        if pente > 0.0 {
            let preferencia = ph2d_sculpt3d::preferencia_do_pente(direccao, pente);
            let _ = ph2d_mesh::alinha_arestas(
                &mut malha,
                centro,
                brush.radius,
                &preferencia,
                &mut region,
            );
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

/// As sondas desta cena — instrumentos, não lei. Ver [`sondas`].
#[path = "scenes_pente_sondas_tests.rs"]
mod sondas;

/// Os DESENHADORES desta cena — irmão das [`sondas`], cortado por tecto de LOC.
#[path = "scenes_pente_desenhos_tests.rs"]
mod desenhos;
