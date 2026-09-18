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

use ph2d_sculpt3d::medida_do_pente::{lascas, pior_angulo, q_da_faixa};
use ph2d_sculpt3d::{Brush, Dab, SculptStroke, Symmetry, Verb};

/// O raio do pincel com que a cena é medida, em unidades da peça.
///
/// ⚠️ **Ele NÃO é escolhido: é o do corpus do oráculo** (`0,35` sobre uma peça
/// de raio `1`). Nesta malha a aresta mede `≈ 0,049`, logo ele cobre `≈ 7`
/// arestas de raio — a vizinhança de `8,3` em que a lei foi medida.
const RAIO: f32 = 0.35;

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
/// ⛔ **ZERO, e o número é MEDIDO e não escolhido:** sobre a peça desta cena a
/// contagem lê `0` em `2 192`–`2 260` triângulos nos **quatro** rumos varridos.
/// ⚠️ *Na esfera UV que esta cena quase abriu ela lia `3` a `45°`*, e é essa a
/// diferença que este número guarda.
const LASCAS_TOLERADAS: usize = 0;

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
    let mut malha = peca_uma_vez();
    // ⛔⛔ **TRIANGULAR, como o interruptor faz.** A `=49` abre com a topologia
    // dinâmica ARMADA, e armá-la **tritura os quads** — os dois motores do passe
    // recusam-nos por geometria. ⚠️ Sem esta linha o arnês media OUTRO programa:
    // o refino não partia nada (`288` arestas na faixa contra milhares) e a
    // segunda coluna lia **`0` triângulos**, ou seja `180°`, que se lê
    // exactamente como *«a malha está perfeita»*.
    malha.triangulate();
    let brush = Brush {
        verb: Verb::Draw,
        radius: RAIO,
        strength: 0.25,
        pente,
        ..Brush::default()
    };
    let mut stroke = SculptStroke::default();
    stroke.begin(&malha);
    let mut births = Vec::new();
    let mut region = ph2d_mesh::RegionScratch::default();
    let mut centros = Vec::new();
    for k in 0..24 {
        // Sobre a superfície: um arco de círculo máximo na frente da bola.
        let u = -0.55 + 0.05 * k as f32;
        let centro = [u.sin() * e[0], u.sin() * e[1], u.cos()];
        centros.push(centro);
        let _ = ph2d_mesh::refine_in_sphere(
            &mut malha,
            centro,
            brush.radius,
            0.035,
            &mut births,
            &mut region,
        );
        stroke.grow_with(&malha, &births);
        stroke.dab(
            &mut malha,
            &brush,
            &Dab::at(centro, brush.radius, [0.0, 0.0, -1.0]),
            Symmetry::default(),
        );
    }
    (malha, centros)
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
        let (q0, n0) = q_da_faixa(&m0, &c0, RAIO);
        let (q1, n1) = q_da_faixa(&m1, &c1, RAIO);
        let (finas, total) = lascas(&m1, &c1, RAIO, LIMIAR_DA_LASCA);
        let (pior, _) = pior_angulo(&m1, &c1, RAIO);

        assert!(
            n0 > 200 && n1 > 200 && total > 100,
            "{nome}: a faixa tem {n0}/{n1} arestas e {total} triangulos — a peca \
             da cena deixou de conter o fenomeno, e um `Q` de faixa vazia le'-se \
             igual a uma malha sem direccao"
        );
        assert!(
            q0 < BARRA,
            "{nome}: a malha da =49 ja' nasce alinhada com o traco (Q desligado = \
             {q0:+.4}, barra {BARRA:+.4}) — a cena aprovaria um pente INERTE, e o \
             passo (2) do roteiro nao seria comparacao nenhuma"
        );
        assert!(
            q1 - q0 > BARRA,
            "{nome}: o pente no maximo move o Q de {q0:+.4} para {q1:+.4} \
             (Delta {:+.4}, medido +0,074 no pior rumo) contra a barra do corpus \
             {BARRA:+.4} — o artista nao vai ver as linhas alinharem-se, e o \
             passo (3) do roteiro promete que ele ve'",
            q1 - q0
        );
        // ⭐ **E o SINAL vira**, que é a metade que separa *«alinhou»* de
        // *«mexeu»*: `Q < 0` é uma malha a CRUZAR o traço, `Q > 0` é uma a
        // correr COM ele. ⚠️ Um Δ grande sobre dois valores negativos seria a
        // esfera UV outra vez (`−0,188 → −0,008`, Δ `+0,18` e nenhuma grade).
        assert!(
            q0 < 0.0 && q1 > 0.0,
            "{nome}: o Q foi de {q0:+.4} para {q1:+.4} e nao trocou de sinal — \
             a malha nao passou a correr COM o traco, so' se mexeu"
        );
        assert!(
            finas == LASCAS_TOLERADAS,
            "{nome}: a faixa ficou com {finas} triangulo(s) abaixo de \
             {LIMIAR_DA_LASCA}° em {total} (o pior mede {pior:.2}°; medido ZERO \
             nos quatro rumos) — sao as ESTRIAS que o `DEU ERRADO SE` do roteiro \
             nomeia, e o `Q` sozinho nao as ve'"
        );
        eprintln!(
            "[=49] {nome:<16} Q {q0:+.4} -> {q1:+.4} · lascas {finas}/{total} · pior {pior:.2}°"
        );
    }
}

/// **SONDA — que peça tem GRÃO, e quanto?** (a régua que escolheu a da cena)
#[test]
#[ignore = "sonda"]
fn diag_o_grao_das_pecas_candidatas() {
    type Candidata = (&'static str, fn() -> ph2d_mesh::Mesh);
    let candidatas: Vec<Candidata> = vec![
        ("a peca da cena (uv 12k)", super::peca as fn() -> _),
        ("uv + ruido", || {
            ph2d_mesh::shapes::uv_sphere_noisy(55, 82, 1.0, 0.01)
        }),
        ("uv + remesh isotropico", || {
            let mut m = ph2d_mesh::shapes::sphere_with_triangles(12_000, 1.0);
            m.triangulate();
            let _ = ph2d_remesh_iso::remesh_isotropic(&mut m, 0.0142);
            m
        }),
        ("sculpt_sphere (a de fabrica)", || {
            ph2d_mesh::shapes::sculpt_sphere(1.0)
        }),
    ];
    for (nome, faz) in candidatas {
        let mut m = faz();
        m.triangulate();
        let v = m.positions().len();
        for (rotulo, eixo) in [("equador x", 0usize), ("meridiano y", 1)] {
            let mut centros = Vec::new();
            for k in 0..24 {
                let u = -0.55 + 0.05 * k as f32;
                let mut c = [0.0, 0.0, u.cos()];
                c[eixo] = u.sin();
                centros.push(c);
            }
            let (q, n) = q_da_faixa(&m, &centros, RAIO);
            eprintln!("{nome:<30} {rotulo:<12} v={v:<7} Q={q:+.4} (n={n})");
        }
    }
}

/// **SONDA — a escada do pente por RUMO contra a grade**, de onde saem a tabela
/// do cabeçalho da cena e a linha de água dos 45°.
#[test]
#[ignore = "sonda"]
fn diag_a_escada_por_rumo() {
    for (nome, e) in [
        ("ao longo (x)", [1.0f32, 0.0]),
        ("30 graus", [0.866_025_4, 0.5]),
        ("45 graus", RUMOS[2].1),
        ("60 graus", [0.5, 0.866_025_4]),
        ("atravessado (y)", [0.0, 1.0]),
    ] {
        for pente in [0.0f32, 0.25, 0.375, 0.5, 0.625, 0.75, 1.0] {
            let (m, c) = traco(pente, e);
            let (q, _) = q_da_faixa(&m, &c, RAIO);
            let (ang, _) = pior_angulo(&m, &c, RAIO);
            let (finas, total) = lascas(&m, &c, RAIO, LIMIAR_DA_LASCA);
            eprintln!(
                "{nome:<18} pente {pente:.3}  Q={q:+.4}  pior={ang:6.2}°  \
                 <{LIMIAR_DA_LASCA}°: {finas:3} de {total}"
            );
        }
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
    const SCRIPTS: &str = include_str!("scripts.rs");

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
    ] {
        assert!(
            fonte.contains(agulha),
            "{o_que}: `{agulha}` desapareceu. A =49 passa a abrir noutro estado e \
             o gate que mede a lei fica VERDE, porque ele chama a porta em vez de \
             percorrer a rota"
        );
    }
}
