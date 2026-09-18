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
        eprintln!(
            "[=49] {nome:<16} Q {q0:+.4} -> {q1:+.4} · movidos {movidos} · \
             lascas {finas}/{total} · pior {pior:.2}°"
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
            let (q, n) = q_da_faixa(&m, &centros, raio_do_app());
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
            let (q, _) = q_da_faixa(&m, &c, raio_do_app());
            let (ang, _) = pior_angulo(&m, &c, raio_do_app());
            let (finas, total) = lascas(&m, &c, raio_do_app(), LIMIAR_DA_LASCA);
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

/// ⛔⛔⛔ **SONDA — O REGIME QUE O APP DE FACTO DÁ**, contra o que o gate da cena
/// escolheu. O report do dono foi *«não percebi diferença»*, e o gate estava
/// VERDE: a primeira coisa a medir é se ele corre no regime do artista.
#[test]
#[ignore = "sonda"]
fn diag_o_regime_do_app_contra_o_do_gate() {
    use crate::Camera3d;

    let peca = super::peca();
    let bounds = peca.bounds();
    let (vw, vh) = (1920.0f32, 1080.0f32);
    let mut cam = Camera3d {
        yaw: 0.6,
        pitch: 0.35,
        ..Camera3d::default()
    };
    cam.frame(bounds, vw / vh);

    // O raio de fábrica do pincel, em PIXELS, convertido como o `armed_brush_on`
    // converte: através da câmera, no ponto onde o cursor aterra.
    let radius_px = ph2d_panel_sculpt3d::state::Sculpt3dUi::default().radius_px;
    let no_mundo =
        cam.world_radius_for_screen_px([0.0, 0.0, 1.0], radius_px, (vw as u32, vh as u32));

    // O alvo do refino que o passe de topologia usa com o `Detail` de fábrica.
    let detalhe = ph2d_panel_sculpt3d::state::Sculpt3dUi::default().dyn_detail;
    let area = peca.surface_area();
    let tris = ph2d_mesh::tris_for_detail(detalhe);
    let alvo = ph2d_mesh::edge_for_tri_count(area, tris);

    // A aresta média da peça, para se ler quantas cabem num raio.
    let pos = peca.positions();
    let (mut soma, mut n) = (0.0f64, 0usize);
    for f in peca.faces() {
        let vs = f.verts();
        for k in 0..vs.len() {
            let (a, b) = (pos[vs[k] as usize], pos[vs[(k + 1) % vs.len()] as usize]);
            soma += f64::from((a[0] - b[0]).hypot(a[1] - b[1]).hypot(a[2] - b[2]));
            n += 1;
        }
    }
    let aresta = soma / n.max(1) as f64;

    eprintln!("--- o que o APP da' ---");
    eprintln!("  raio do pincel: {radius_px} px  ->  {no_mundo:.4} no mundo");
    eprintln!("  refino: Detail {detalhe:.2} -> {tris} triangulos -> aresta alvo {alvo:.4}");
    eprintln!("  aresta media da peca: {aresta:.4}");
    eprintln!(
        "  arestas por raio (antes do refino): {:.1}",
        f64::from(no_mundo) / aresta
    );
    eprintln!(
        "  arestas por raio (depois):          {:.1}",
        f64::from(no_mundo / alvo)
    );
    eprintln!("--- o que o GATE mede ---");
    eprintln!(
        "  raio {:.4} (derivado) · refino {:.4} (derivado do Detail da cena) \
         · arestas por raio {:.1}",
        raio_do_app(),
        alvo_do_refino(),
        f64::from(raio_do_app() / alvo_do_refino())
    );
}

/// ⛔⛔⛔ **SONDA — quantas ARESTAS POR RAIO o pente precisa para se ver.**
///
/// O report do dono (*«não percebi diferença»*) com o gate VERDE: ele corre a
/// `10` arestas por raio e o app dá `2,0`.
#[test]
#[ignore = "sonda"]
fn diag_o_pente_contra_as_arestas_por_raio() {
    for (nome, raio, alvo) in [
        ("o APP de fabrica", 0.1634f32, 0.0805f32),
        ("Detail 0,75", 0.1634, 0.0402),
        ("Detail 1,00", 0.1634, 0.0175),
        ("raio 2x, Detail 1", 0.3268, 0.0175),
        ("o GATE de hoje", 0.35, 0.035),
    ] {
        let mut q = [0.0f64; 2];
        let mut lasca = [0usize; 2];
        let (mut verts, mut n1) = (0usize, 0usize);
        let mut saidas: Vec<Vec<[f32; 3]>> = Vec::new();
        for (i, pente) in [0.0f32, 1.0].into_iter().enumerate() {
            let (m, c) = traco_com(pente, RUMOS[2].1, raio, alvo);
            let (qq, nn) = q_da_faixa(&m, &c, raio);
            q[i] = qq;
            n1 = nn;
            lasca[i] = lascas(&m, &c, raio, LIMIAR_DA_LASCA).0;
            verts = m.positions().len();
            saidas.push(m.positions().to_vec());
        }
        // ⭐ O que o OLHO lê não é a média: é quantos vértices o pente de facto
        // deslocou, e quanto.
        let movidos = saidas[0]
            .iter()
            .zip(&saidas[1])
            .filter(|(a, b)| a != b)
            .count();
        eprintln!(
            "{nome:<20} arestas/raio {:>4.1}  Q {:+.4} -> {:+.4} (D {:+.4})  \
             arestas na faixa {n1:>5}  mexidos {movidos:>5}  lascas {}  v={verts}",
            f64::from(raio / alvo),
            q[0],
            q[1],
            q[1] - q[0],
            lasca[1]
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
    let mut region = ph2d_mesh::RegionScratch::default();
    let mut centros = Vec::new();
    // ⚠️ O percurso anda **em raios de pincel**, não em unidades fixas: com um
    // pincel menor, um passo fixo seria um traço aos saltos.
    let passo = raio * 0.15;
    for k in 0..24 {
        let u = -passo * 12.0 + passo * k as f32;
        let centro = [u.sin() * e[0], u.sin() * e[1], u.cos()];
        centros.push(centro);
        let _ = ph2d_mesh::refine_in_sphere(
            &mut malha,
            centro,
            brush.radius,
            alvo,
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

/// ⛔⛔⛔ **SONDA — DESENHA o arame, porque o produto desta wave é uma IMAGEM.**
///
/// O report do dono veio com FOTO e a frase *«não sei o que é para esperar»*.
/// ⚠️ **Toda régua desta cena é um NÚMERO** (o `Q`, a contagem, as lascas), e
/// nenhuma responde *«o que é que isto parece»*. Esta escreve dois `.ppm` — o
/// mesmo traço com o pente desligado e no tecto — para se OLHAR.
///
/// ```text
/// cargo test -p ph2d-app-sculpt3d --release --lib diag_desenha_o_arame -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda"]
fn diag_desenha_o_arame() {
    let dir = std::env::var("PH2D_PENTE_DUMP").unwrap_or_else(|_| "/tmp".into());
    for (nome, raio, alvo) in [
        ("app_fabrica", 0.1634f32, 0.0805f32),
        ("cena_detail1", raio_do_app(), alvo_do_refino()),
        ("pincel_3x", raio_do_app() * 3.0, alvo_do_refino()),
    ] {
        for pente in [0.0f32, 1.0] {
            let (m, c) = traco_com(pente, RUMOS[2].1, raio, alvo);
            let alvo_png = format!(
                "{dir}/pente_{nome}_{}.ppm",
                if pente > 0.0 { "on" } else { "off" }
            );
            desenha(&m, &c, raio, &alvo_png);
        }
        eprintln!("{nome}: raio {raio:.4} alvo {alvo:.4} -> {dir}/pente_{nome}_*.ppm");
    }
}

/// Um arame ortográfico, olhando de `+z`, centrado no percurso — em `.ppm`,
/// que se converte com `magick`.
fn desenha(m: &ph2d_mesh::Mesh, percurso: &[[f32; 3]], raio: f32, caminho: &str) {
    const N: usize = 900;
    // A janela é o percurso mais dois raios de cada lado — o enquadramento que
    // o artista teria se olhasse para o traço dele.
    let (mut x0, mut x1, mut y0, mut y1) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
    for p in percurso {
        x0 = x0.min(p[0]);
        x1 = x1.max(p[0]);
        y0 = y0.min(p[1]);
        y1 = y1.max(p[1]);
    }
    let pad = raio * 2.0;
    let (cx, cy) = ((x0 + x1) * 0.5, (y0 + y1) * 0.5);
    let meia = ((x1 - x0).max(y1 - y0) * 0.5 + pad).max(1e-4);
    let para_px = |p: [f32; 3]| -> (i32, i32) {
        let u = (p[0] - cx) / (2.0 * meia) + 0.5;
        let v = 0.5 - (p[1] - cy) / (2.0 * meia);
        ((u * N as f32) as i32, (v * N as f32) as i32)
    };

    let mut buf = vec![255u8; N * N * 3];
    let pos = m.positions();
    let mut linha = |a: (i32, i32), b: (i32, i32), buf: &mut Vec<u8>| {
        let (dx, dy) = ((b.0 - a.0).abs(), -(b.1 - a.1).abs());
        let (sx, sy) = (
            if a.0 < b.0 { 1 } else { -1 },
            if a.1 < b.1 { 1 } else { -1 },
        );
        let (mut x, mut y, mut err) = (a.0, a.1, dx + dy);
        loop {
            if x >= 0 && y >= 0 && (x as usize) < N && (y as usize) < N {
                let i = ((y as usize) * N + x as usize) * 3;
                buf[i] = 40;
                buf[i + 1] = 40;
                buf[i + 2] = 60;
            }
            if x == b.0 && y == b.1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    };
    for f in m.faces() {
        let vs = f.verts();
        for k in 0..vs.len() {
            let a = pos[vs[k] as usize];
            let b = pos[vs[(k + 1) % vs.len()] as usize];
            // Só a calota virada a nós, senão o arame de trás polui a leitura.
            if a[2] < 0.3 || b[2] < 0.3 {
                continue;
            }
            linha(para_px(a), para_px(b), &mut buf);
        }
    }
    let mut ficheiro = format!("P6\n{N} {N}\n255\n").into_bytes();
    ficheiro.extend_from_slice(&buf);
    std::fs::write(caminho, ficheiro).expect("escreve o ppm");
}

/// ⛔⛔⛔ **SONDA — o alinhamento ACUMULA com as passagens?**
///
/// A espec §3.1 diz que o pente **move vértices** e não troca arestas; quem
/// muda a ligação é o passe de topologia, que parte e funde **seguindo** as
/// posições. ⇒ a hipótese é que a grade se forma ao longo de VÁRIAS passagens.
/// Esta sonda mede e **desenha** um recorte legível.
#[test]
#[ignore = "sonda"]
fn diag_o_pente_acumula_com_as_passagens() {
    let dir = std::env::var("PH2D_PENTE_DUMP").unwrap_or_else(|_| "/tmp".into());
    // ⚠️ O regime do ORÁCULO: raio `0,35` e um alvo de refino da ORDEM da aresta
    // da peça — ali o passe **mantém** a densidade em vez de a afinar a cada
    // passagem, e é isso que deixa o alinhamento acumular.
    let (raio, alvo) = (0.35f32, 0.05f32);
    for passagens in [1usize, 4, 8, 16] {
        for pente in [0.0f32, 1.0] {
            let (m, c) = traco_repetido(pente, RUMOS[2].1, raio, alvo, passagens);
            let (q, n) = q_da_faixa(&m, &c, raio);
            let (ang, _) = pior_angulo(&m, &c, raio);
            eprintln!(
                "passagens {passagens:>2}  pente {pente:.1}  Q={q:+.4} (n={n:>5})  \
                 pior={ang:5.2}°  v={}",
                m.positions().len()
            );
            if pente > 0.0 || passagens == 1 {
                desenha_recorte(
                    &m,
                    &c,
                    raio,
                    &format!(
                        "{dir}/acum_{passagens:02}_{}.ppm",
                        if pente > 0.0 { "on" } else { "off" }
                    ),
                );
            }
        }
    }
}

/// O mesmo traço, `n` vezes por cima — como a mão faz.
fn traco_repetido(
    pente: f32,
    e: [f32; 2],
    raio: f32,
    alvo: f32,
    passagens: usize,
) -> (ph2d_mesh::Mesh, Vec<[f32; 3]>) {
    let mut malha = peca_uma_vez();
    malha.triangulate();
    let brush = Brush {
        verb: Verb::Draw,
        radius: raio,
        strength: 0.05,
        pente,
        ..Brush::default()
    };
    let passo = raio * 0.15;
    let mut centros = Vec::new();
    for _ in 0..passagens {
        let mut stroke = SculptStroke::default();
        stroke.begin(&malha);
        let mut births = Vec::new();
        let mut region = ph2d_mesh::RegionScratch::default();
        centros.clear();
        for k in 0..24 {
            let u = -passo * 12.0 + passo * k as f32;
            let centro = [u.sin() * e[0], u.sin() * e[1], u.cos()];
            centros.push(centro);
            let _ = ph2d_mesh::refine_in_sphere(
                &mut malha,
                centro,
                brush.radius,
                alvo,
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
    }
    (malha, centros)
}

/// Um recorte de `3` raios à volta do meio do traço — grande o bastante para os
/// triângulos se lerem.
fn desenha_recorte(m: &ph2d_mesh::Mesh, percurso: &[[f32; 3]], raio: f32, caminho: &str) {
    const N: usize = 900;
    let meio = percurso[percurso.len() / 2];
    let meia = raio * 1.5;
    let para_px = |p: [f32; 3]| -> (i32, i32) {
        let u = (p[0] - meio[0]) / (2.0 * meia) + 0.5;
        let v = 0.5 - (p[1] - meio[1]) / (2.0 * meia);
        ((u * N as f32) as i32, (v * N as f32) as i32)
    };
    let mut buf = vec![255u8; N * N * 3];
    let pos = m.positions();
    let mut linha = |a: (i32, i32), b: (i32, i32), buf: &mut Vec<u8>| {
        let (dx, dy) = ((b.0 - a.0).abs(), -(b.1 - a.1).abs());
        let (sx, sy) = (
            if a.0 < b.0 { 1 } else { -1 },
            if a.1 < b.1 { 1 } else { -1 },
        );
        let (mut x, mut y, mut err) = (a.0, a.1, dx + dy);
        loop {
            if x >= 0 && y >= 0 && (x as usize) < N && (y as usize) < N {
                let i = ((y as usize) * N + x as usize) * 3;
                buf[i] = 30;
                buf[i + 1] = 30;
                buf[i + 2] = 50;
            }
            if x == b.0 && y == b.1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    };
    for f in m.faces() {
        let vs = f.verts();
        for k in 0..vs.len() {
            let a = pos[vs[k] as usize];
            let b = pos[vs[(k + 1) % vs.len()] as usize];
            if a[2] < 0.3 || b[2] < 0.3 {
                continue;
            }
            linha(para_px(a), para_px(b), &mut buf);
        }
    }
    let mut ficheiro = format!("P6\n{N} {N}\n255\n").into_bytes();
    ficheiro.extend_from_slice(&buf);
    std::fs::write(caminho, ficheiro).expect("escreve o ppm");
}

/// ⛔⛔⛔ **SONDA — e sobre uma peça que É uma GRADE, vê-se?**
///
/// A pergunta que sobra depois de o oráculo mostrar que a saída DELE é uma sopa
/// de triângulos: numa malha com grade (a esfera UV, que é o que o artista tem)
/// o pente **vira** linhas que existem — e isso pode ser visível.
#[test]
#[ignore = "sonda"]
fn diag_desenha_sobre_uma_grade() {
    let dir = std::env::var("PH2D_PENTE_DUMP").unwrap_or_else(|_| "/tmp".into());
    let e = RUMOS[2].1;
    for (nome, alvo) in [
        ("neutro", 0.0527f32),
        ("fino", alvo_do_refino()),
        ("grosso", 0.09f32),
    ] {
        for pente in [0.0f32, 1.0] {
            let mut malha = ph2d_mesh::shapes::sphere_with_triangles(12_000, 1.0);
            malha.triangulate();
            let raio = raio_do_app() * 2.0;
            let brush = Brush {
                verb: Verb::Draw,
                radius: raio,
                strength: 0.05,
                pente,
                ..Brush::default()
            };
            let mut stroke = SculptStroke::default();
            stroke.begin(&malha);
            let mut births = Vec::new();
            let mut region = ph2d_mesh::RegionScratch::default();
            let mut centros = Vec::new();
            let passo = raio * 0.15;
            for k in 0..24 {
                let u = -passo * 12.0 + passo * k as f32;
                let centro = [u.sin() * e[0], u.sin() * e[1], u.cos()];
                centros.push(centro);
                let _ = ph2d_mesh::refine_in_sphere(
                    &mut malha,
                    centro,
                    raio,
                    alvo,
                    &mut births,
                    &mut region,
                );
                stroke.grow_with(&malha, &births);
                stroke.dab(
                    &mut malha,
                    &brush,
                    &Dab::at(centro, raio, [0.0, 0.0, -1.0]),
                    Symmetry::default(),
                );
            }
            let (q, n) = q_da_faixa(&malha, &centros, raio);
            eprintln!("grade {nome} pente {pente:.1}  Q={q:+.4} (n={n})");
            desenha_recorte(
                &malha,
                &centros,
                raio,
                &format!(
                    "{dir}/grade_{nome}_{}.ppm",
                    if pente > 0.0 { "on" } else { "off" }
                ),
            );
        }
    }
}
