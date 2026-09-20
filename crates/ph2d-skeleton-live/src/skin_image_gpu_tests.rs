//! ⭐⭐⭐ **OS GATES DA PELE NO FORMATO DA PLACA** (F9 W2).
//!
//! ⚠️ **O gate que é a wave inteira numa asserção é o primeiro:** os dois motores — a CPU, que
//! continua a ser a REFERÊNCIA, e a tabela que sobe ao dispositivo — põem cada vértice no MESMO
//! sítio. *Dois motores, uma lei* (o molde é o do Flip).

use super::*;
use ph2d_poly2d::Mesh2d;
use ph2d_skeleton::{Correccao, Especie, Skin, SkinBone, Xform};

/// A barra de repouso, `0..40 × 0..10` px, partida em duas colunas de dois triângulos.
fn malha() -> Mesh2d {
    Mesh2d {
        rest: vec![
            [0.0, 0.0],
            [20.0, 0.0],
            [40.0, 0.0],
            [0.0, 10.0],
            [20.0, 10.0],
            [40.0, 10.0],
        ],
        tris: vec![[0, 1, 4], [0, 4, 3], [1, 2, 5], [1, 5, 4]],
        size: [40, 10],
    }
}

/// Dois ossos ao longo da barra, o segundo **RODADO** — uma corrente recta não distingue lei
/// nenhuma, e é a correcção que a fixtura irmã da `ph2d-skeleton` já pagou.
fn pele() -> Skin {
    pele_a(0.30)
}

/// A mesma corrente com o 2.º osso a `rad` — *a pose é PARÂMETRO porque um gate desta família tem
/// de a variar, e a `Skin` não expõe os ossos para escrita*.
fn pele_a(rad: f64) -> Skin {
    let bone = |x: f64, pose: Xform, tendon: u32| SkinBone {
        rest_a: [x, 0.0],
        rest_b: [x + 0.2, 0.0],
        radius: 0.25,
        pose,
        sub: (0, 1),
        tendon,
    };
    // Uma rotação em torno de `(0,2, 0)`, que é a junta.
    let (c, s) = (rad.cos(), rad.sin());
    let girado = Xform([c, s, -s, c, 0.2 - 0.2 * c, -0.2 * s]);
    Skin::new(vec![bone(0.0, Xform::IDENTITY, 0), bone(0.2, girado, 1)]).expect("a pele nasce")
}

/// O mapa `pixel da imagem → local da sprite`, com a arte a ocupar `0,4 × 0,1` m.
fn p2l() -> Xform {
    Xform([0.01, 0.0, 0.0, -0.01, -0.2, 0.05])
}

const ANCHOR: [f32; 2] = [0.0, 0.0];
const SIZE: [f32; 2] = [0.4, 0.1];

/// A tabela do padrão-ouro: o peso de cada vértice em cada TENDÃO, por linha.
fn tabela() -> Vec<f64> {
    let mut t = Vec::new();
    for p in malha().rest {
        let u = (p[0] / 40.0).clamp(0.0, 1.0);
        t.extend_from_slice(&[1.0 - u, u]);
    }
    t
}

/// ⭐⭐⭐ **OS DOIS MOTORES PÕEM CADA VÉRTICE NO MESMO SÍTIO** — o gate que É a wave.
///
/// ⚠️ **A barra é derivada e não escolhida:** o payload viaja em `f32` e a CPU calcula em `f64`,
/// logo o desvio esperado é o do arredondamento de uma mistura de dois afins — poucos ULP sobre
/// uma arte de `0,4 m`. `2e-6` são **~17 ULP de `f32`** nessa magnitude, a mesma barra que a
/// paridade dos pincéis tangenciais desta casa já usa.
///
/// ⛔ **E a fixtura tem de CONTER o fenómeno:** com a corrente recta as duas leis concordariam
/// por construção (as poses seriam a identidade), e o gate mediria o nada.
#[test]
fn a_placa_e_a_cpu_posam_cada_vertice_no_mesmo_sitio() {
    let (pele, pesos, p2l) = (pele(), tabela(), p2l());
    let cpu = crate::skin_image::posed_sprite_mesh_corrigida(
        malha(),
        p2l,
        &pele,
        &pesos,
        ANCHOR,
        SIZE,
        &[],
    )
    .expect("a CPU posa");
    let placa = sprite_mesh_para_a_placa(malha(), p2l, &pele, &pesos, ANCHOR, SIZE, &[])
        .expect("a placa recebe");

    // ⛔ O CONTROLO: a malha que sobe está em REPOUSO, e é diferente da posada.
    let movimento = cpu
        .local
        .iter()
        .zip(&placa.local)
        .map(|(a, b)| (a[0] - b[0]).hypot(a[1] - b[1]))
        .fold(0.0f32, f32::max);
    assert!(
        movimento > 0.01,
        "a fixtura nao dobra ({movimento:.6} m): as duas leis concordariam por construcao"
    );

    let posada = placa.posado();
    let pior = cpu
        .local
        .iter()
        .zip(&posada.local)
        .map(|(a, b)| (a[0] - b[0]).hypot(a[1] - b[1]))
        .fold(0.0f32, f32::max);
    assert!(
        pior < 2e-6,
        "a lei da placa e a da CPU discordam em {pior:.3e} m — a arte desenha num sitio e o \
         ponteiro aponta noutro"
    );
    // ⚠️ E a UV **não** é tocada pela pele: a tinta está pintada na forma de repouso.
    assert_eq!(cpu.uv, placa.uv, "a UV mudou com a rota da placa");
    assert_eq!(
        cpu.tris, placa.tris,
        "a topologia mudou com a rota da placa"
    );
}

/// ⭐⭐⭐ **AS CORRECÇÕES À MÃO CHEGAM À TABELA DA PLACA** — e isto fecha uma dívida NOMEADA.
///
/// ⛔⛔ O cabeçalho do [`crate::skin_gpu`] avisa por escrito que a mancha do artista *«vira um
/// DEFEITO MUDO no dia em que a outra metade shipar»*: a arte desenharia pela placa **sem** as
/// correcções e pela CPU **com** elas, e o sintoma seria *«a correcção funciona e depois some»* —
/// sem um erro.
///
/// ⚠️ **As duas metades:** a mancha MOVE a malha que sobe (senão ela não está lá), e o que sobe
/// concorda com a CPU **que também a aplicou** (senão está lá outra coisa).
#[test]
fn as_correccoes_a_mao_chegam_a_tabela_da_placa() {
    let (pele, pesos, p2l) = (pele(), tabela(), p2l());
    // Uma mancha ABSOLUTA (F29) no meio da barra, sobre o 1.º tendão.
    let mancha = [Correccao {
        tendon: 0,
        centro: [0.0, 0.0],
        raio: 0.15,
        especie: Especie::Alvo(1.0),
    }];
    let sem = sprite_mesh_para_a_placa(malha(), p2l, &pele, &pesos, ANCHOR, SIZE, &[])
        .expect("sem mancha");
    let com = sprite_mesh_para_a_placa(malha(), p2l, &pele, &pesos, ANCHOR, SIZE, &mancha)
        .expect("com mancha");
    let mexeu = sem
        .posado()
        .local
        .iter()
        .zip(&com.posado().local)
        .map(|(a, b)| (a[0] - b[0]).hypot(a[1] - b[1]))
        .fold(0.0f32, f32::max);
    assert!(
        mexeu > 1e-4,
        "a mancha do artista nao chegou a' tabela que sobe ({mexeu:.3e} m) — a arte desenharia \
         pela placa SEM a correccao e pela CPU COM ela, e o report seria «a correccao some»"
    );

    let cpu = crate::skin_image::posed_sprite_mesh_corrigida(
        malha(),
        p2l,
        &pele,
        &pesos,
        ANCHOR,
        SIZE,
        &mancha,
    )
    .expect("a CPU posa");
    let pior = cpu
        .local
        .iter()
        .zip(&com.posado().local)
        .map(|(a, b)| (a[0] - b[0]).hypot(a[1] - b[1]))
        .fold(0.0f32, f32::max);
    assert!(
        pior < 2e-6,
        "com a mancha aplicada os dois motores discordam em {pior:.3e} m"
    );
}

/// ⭐⭐ **A TRUNCAGEM A `K` RENORMALIZA** — senão a mistura encolhe o ponto para a origem.
///
/// ⚠️ **As três metades:** os `K` maiores sobrevivem, a soma volta a `1`, e um ponto sem dono
/// nenhum sai com [`ph2d_render::SEM_PELE`] — *«ninguém manda aqui»* é uma resposta, não um zero.
#[test]
fn a_truncagem_a_quatro_ossos_renormaliza() {
    let (w, b) = maiores_k(&[0.05, 0.30, 0.10, 0.25, 0.20, 0.10]);
    let soma: f32 = w.iter().sum();
    assert!(
        (soma - 1.0).abs() < 1e-6,
        "a truncagem deixou a soma em {soma} — a mistura encolheria o ponto para a origem"
    );
    // Os quatro maiores são `0,30 · 0,25 · 0,20 · 0,10`, por ordem decrescente.
    assert_eq!(
        &b[..3],
        &[1, 3, 4],
        "os maiores nao sao os que ficaram: {b:?}"
    );
    assert!(
        w[0] > w[1] && w[1] > w[2],
        "a lista nao saiu por ordem decrescente: {w:?}"
    );
    // ⛔ O `0,05` ficou de fora, e a proporção entre os que ficaram é a de antes.
    assert!(
        (w[0] / w[1] - 0.30 / 0.25).abs() < 1e-6,
        "a renormalizacao mudou a PROPORCAO entre os ossos que ficaram"
    );

    let (w0, b0) = maiores_k(&[0.0, 0.0, 0.0]);
    assert_eq!(w0, [0.0; 4]);
    assert_eq!(
        b0,
        [ph2d_render::SEM_PELE; 4],
        "um ponto sem dono devia sair com SEM_PELE — um indice 0 poria a pose do 1.º osso nele"
    );
}

/// ⭐⭐ **A PORTA NASCE ABERTA, E O ZERO BISSECTA** — a decisão de produto desta wave, num gate.
///
/// ⚠️ **A lei da casa é *«tudo o que é novo shipa desligado»*, e ela vale enquanto o que existe
/// funciona.** Aqui não funciona: o caminho da CPU custa `35,9 %` de um quadro a 8 imagens presas e
/// é linear nelas. *Uma porta que nasce fechada sobre um tecto medido é a decisão a ser tomada por
/// inércia.*
#[test]
fn a_porta_nasce_aberta_e_o_zero_bissecta() {
    // ⛔ O ambiente é global ao processo: este gate lê o estado de fábrica e não o escreve.
    let antes = std::env::var("PH2D_SKIN_GPU").ok();
    assert_eq!(
        a_placa_posa(),
        !matches!(antes.as_deref(), Some("0") | Some("false")),
        "a porta deixou de seguir a env"
    );
    assert!(
        antes.is_some() || a_placa_posa(),
        "sem a env a porta devia estar ABERTA — o caminho da CPU e' o tecto do produto"
    );
}

/// ⭐⭐⭐ **MOVER UM OSSO NÃO MUDA A TABELA DO BIND** — a propriedade que torna esta wave barata, e
/// que é herdada do módulo `skin_gpu`, apagado nesta jornada por ser um payload ÓRFÃO com a lei
/// antiga lá dentro.
///
/// ⭐ **A quota que reparte o peso de um tendão pelos sub-ossos é função da posição de REPOUSO**
/// ([`ph2d_skeleton::bend`]), logo a tabela de pesos por OSSO — e a das JUNTAS, que sai dos eixos de
/// repouso — só mudam quando a TOPOLOGIA do rig muda. ⇒ o que custa por quadro são `N` afins e `N`
/// ângulos, nunca `V` pesos.
///
/// ⚠️ **Esta metade é mais forte que a do módulo apagado**, que só media os pesos: aqui a tabela de
/// juntas entra, e ela é a estrutura NOVA — *uma propriedade que se herda tem de cobrir o que foi
/// acrescentado depois dela, senão ela é uma promessa sobre o passado*.
#[test]
fn mover_um_osso_nao_muda_a_tabela_do_bind() {
    let (pesos, p2l) = (tabela(), p2l());
    let a = sprite_mesh_para_a_placa(malha(), p2l, &pele(), &pesos, ANCHOR, SIZE, &[])
        .expect("a pele de repouso");
    // A MESMA corrente com o 2.º osso noutra pose.
    let b = sprite_mesh_para_a_placa(malha(), p2l, &pele_a(1.10), &pesos, ANCHOR, SIZE, &[])
        .expect("a pele posada");
    let (sa, sb) = (a.skin.as_ref().expect("a"), b.skin.as_ref().expect("b"));
    assert_eq!(sa.pesos, sb.pesos, "a tabela de PESOS seguiu a pose");
    assert_eq!(sa.ossos, sb.ossos, "os ÍNDICES seguiram a pose");
    assert_eq!(sa.juntas, sb.juntas, "a tabela de JUNTAS seguiu a pose");
    assert_eq!(a.local, b.local, "o REPOUSO que sobe seguiu a pose");
    // ⛔ O CONTROLO: o que É do quadro tem de ter mudado, senão isto mede um no-op.
    assert_ne!(
        sa.afins, sb.afins,
        "os AFINS nao mudaram — a fixtura nao posa"
    );
    assert_ne!(sa.angulos, sb.angulos, "os ANGULOS nao mudaram");
}

/// ⭐⭐ **AS DUAS MÍDIAS VIVAS CHEGAM PELA PORTA CORRIGIDA** — herdado do módulo apagado, e é a
/// metade dele que continua a afirmar sobre PRODUTO.
///
/// ⚠️ **A agulha nomeia um endereço de fiação**, que é a espécie de gate que um refactor parte — ela
/// falha ALTO, que é a sorte desta família. ⛔ A metade da imagem passou a ter DOIS construtores
/// (a CPU e a placa) e os dois têm de receber as manchas: o comportamento está no
/// [`as_correccoes_a_mao_chegam_a_tabela_da_placa`], e aqui fica a rota.
#[test]
fn as_duas_midias_vivas_chegam_pela_porta_corrigida() {
    let recook = include_str!("skin_live.rs");
    assert!(
        recook.contains("aplica_corrigido_com(") || recook.contains("aplica_pela_curva_com("),
        "o recook do vector deixou de passar pela porta corrigida"
    );
    let imagem = include_str!("skin_image.rs");
    for agulha in ["posed_sprite_mesh_corrigida", "sprite_mesh_para_a_placa"] {
        assert!(
            imagem.contains(agulha),
            "o desenho da imagem deixou de nomear `{agulha}` — um dos dois construtores saiu da \
             porta, e a arte passa a desenhar sem as manchas do artista num deles"
        );
    }
}
