//! ⭐⭐⭐ **A CAIXA DE CORNELL — a régua da `W5`**, a luz indirecta (`docs/Render3d/03` §W5).
//!
//! O plano nomeia-a por escrito: *«Régua: a caixa de Cornell. Ela tem resposta conhecida e não
//! deixa mentir.»* Esta é essa caixa, montada no **nosso** campo de distância — cinco paredes, duas
//! peças dentro, aberta para a câmera.
//!
//! # ⭐ Porque é ela, e não uma esfera bonita
//!
//! A propriedade que ela mede não é «parece melhor»: é o **SANGRAMENTO DE COR**. A parede da
//! esquerda é vermelha e a da direita é verde, e o chão é branco nas duas pontas — logo, com luz
//! indirecta, o chão junto da parede vermelha tem de ficar **mais vermelho que verde**, e junto da
//! verde o contrário. *O sinal é conhecido antes de medir*, e é isso que uma régua tem de ter.
//!
//! ⛔⛔ **E é por isso que ela apanha o defeito que uma imagem não apanha:** uma GI errada pela
//! metade continua a parecer plausível — mais escura aqui, mais clara ali. Trocar o sinal do
//! sangramento, não.
//!
//! # ⚠️ As divergências DECLARADAS contra a caixa de Cornell publicada
//!
//! Esta não é a caixa medida em Cornell, e as diferenças estão aqui para ninguém as ler como
//! defeito:
//!
//! - **a luz é um [`PointLamp`] e não uma área no tecto.** A caixa original tem uma luminária
//!   rectangular, e a penumbra dela faz parte da resposta publicada. O módulo tem lâmpadas pontuais
//!   ([`crate::PointLamp`]); uma luz de área é feature de produto, não uma peça desta régua.
//! - **as reflectâncias são as clássicas, não as medidas** (espectro completo em 4 nm no original);
//!   aqui são três números por parede, que é o que o nosso material recebe.
//! - **a caixa é aberta para a câmera** — o original também o é, para a fotografia.
//!
//! ⇒ *o que esta régua afirma é uma propriedade de TRANSPORTE (o sinal e a magnitude do
//! sangramento), nunca paridade fotométrica com o laboratório de Cornell.*

use crate::{Orbit, Surfaces, trace};
use ph2d_field::{Blend, FieldDoc, Node, NodeId, NodeKind, Op, Primitive, Xform};
use ph2d_field_eval::hybrid::Registry;
use ph2d_field_eval::owners::Owners;
use ph2d_material::{Environment, OpenPbr};

/// Meia-aresta do INTERIOR: a sala é um cubo de lado `1`.
const SALA: f32 = 0.5;
/// Meia-espessura de uma parede. Ela é fina de propósito — a parede é uma superfície, e o que
/// interessa é a face virada para dentro.
const PAREDE: f32 = 0.01;

/// As sete folhas, **na ordem em que os materiais têm de vir** (ver [`Surfaces::all`]).
///
/// ⛔ A ordem é contrato: uma lista de materiais com outra ordem pinta cada parede com a cor da
/// vizinha, e **sem erro nenhum**.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Face {
    Chao = 0,
    Tecto = 1,
    Fundo = 2,
    Esquerda = 3,
    Direita = 4,
    Alta = 5,
    Baixa = 6,
}

/// As reflectâncias clássicas da caixa, em linear.
const BRANCO: [f32; 3] = [0.73, 0.73, 0.73];
const VERMELHO: [f32; 3] = [0.61, 0.06, 0.06];
const VERDE: [f32; 3] = [0.12, 0.45, 0.09];

/// ⭐ **A caixa** — o documento, as peças postas (uma por folha, para o [`Owners`]) e os materiais.
///
/// ⚠️ **As três listas saem daqui juntas de propósito.** Quem constrói o documento constrói a ordem
/// das folhas, e quem constrói os materiais tem de a seguir; separá-las em duas funções é como a
/// ordem se perde.
pub(crate) fn caixa() -> (FieldDoc, Vec<FieldDoc>, Vec<OpenPbr>) {
    let folha = ph2d_field_eval::leaf;
    // As faces: meia-extensão e centro. A face de dentro de cada parede assenta em `±SALA`.
    let fora = SALA + PAREDE;
    let paredes = [
        // chão, tecto: finos em Y
        ([SALA, PAREDE, SALA], [0.0, -fora, 0.0]),
        ([SALA, PAREDE, SALA], [0.0, fora, 0.0]),
        // fundo: fino em Z
        ([SALA, SALA, PAREDE], [0.0, 0.0, -fora]),
        // esquerda, direita: finos em X
        ([PAREDE, SALA, SALA], [-fora, 0.0, 0.0]),
        ([PAREDE, SALA, SALA], [fora, 0.0, 0.0]),
    ];
    // ⚠️ As duas peças de dentro POUSAM no chão (`centro.y − meia_altura = −SALA`), como as da
    // caixa original. Uma peça a flutuar apagaria o contacto, que é onde o sangramento é mais forte.
    let alta_h = 0.30;
    let baixa_h = 0.15;
    let pecas = [
        (
            [0.14, alta_h, 0.14],
            [-0.20, -SALA + alta_h, -0.15],
            0.30f32,
        ),
        (
            [0.15, baixa_h, 0.15],
            [0.21, -SALA + baixa_h, 0.18],
            -0.28f32,
        ),
    ];

    let mut nos: Vec<Node> = Vec::new();
    let mut postas: Vec<FieldDoc> = Vec::new();
    let push = |nos: &mut Vec<Node>, postas: &mut Vec<FieldDoc>, n: Node| {
        postas.push(FieldDoc::new(vec![n.clone()], NodeId(0)).expect("a folha posta"));
        nos.push(n);
    };
    for (half, at) in paredes {
        push(
            &mut nos,
            &mut postas,
            folha(
                Primitive::Box {
                    half,
                    round: 0.0,
                    chamfer: 0.0,
                },
                Xform::at(at[0], at[1], at[2]),
            ),
        );
    }
    for (half, at, giro) in pecas {
        // Rotação em torno de Y — o quaternion `(0, sin(θ/2), 0, cos(θ/2))`.
        let (s, c) = (giro / 2.0).sin_cos();
        push(
            &mut nos,
            &mut postas,
            folha(
                Primitive::Box {
                    half,
                    round: 0.0,
                    chamfer: 0.0,
                },
                Xform {
                    translation: at,
                    rotation: [0.0, s, 0.0, c],
                    scale: 1.0,
                },
            ),
        );
    }

    let quantas = u32::try_from(nos.len()).expect("as folhas cabem");
    let filhos: Vec<NodeId> = (0..quantas).map(NodeId).collect();
    let raiz = NodeId(quantas);
    nos.push(Node {
        xform: Xform::IDENTITY,
        kind: NodeKind::Combine {
            op: Op::Union(Blend::Sharp),
            children: filhos,
        },
        mods: Vec::new(),
        verb: None,
    });

    let difusa = |base_color: [f32; 3]| OpenPbr {
        base_color,
        // ⛔ **Sem especular**, e não é preguiça: a caixa de Cornell é um teste de transporte
        // DIFUSO. Um lóbulo especular poria um segundo caminho de luz na régua, e o número do
        // sangramento deixaria de dizer de onde veio.
        specular_weight: 0.0,
        ..OpenPbr::default()
    };
    let materiais = vec![
        difusa(BRANCO),
        difusa(BRANCO),
        difusa(BRANCO),
        difusa(VERMELHO),
        difusa(VERDE),
        difusa(BRANCO),
        difusa(BRANCO),
    ];

    (
        FieldDoc::new(nos, raiz).expect("a caixa"),
        postas,
        materiais,
    )
}

/// A câmera a olhar para dentro da caixa, pelo lado aberto.
///
/// ⚠️⚠️ **Ela NÃO é a [`Orbit::default`], e a diferença é a régua:** a de omissão é uma vista de
/// **três quartos** (escolhida na `W0` para uma aresta viva e um filete se distinguirem), e daquele
/// ângulo o olho está fora da caixa — a parede do fundo **não aparece**. A caixa de Cornell vê-se
/// **de frente**, que é o único enquadramento em que as duas paredes de cor e o fundo estão no
/// mesmo quadro.
///
/// ⭐ Quem o apanhou foi o gate da fixtura, não uma imagem: a 1.ª redacção dele parava em *«90 %
/// dos pixels acertam»*, que é verdade também para o lado de FORA de uma caixa fechada.
pub(crate) fn camara() -> Orbit {
    Orbit {
        rotation: Orbit::from_yaw_pitch(0.0, 0.0).rotation,
        target: [0.0, 0.0, 0.0],
        half_extent: 0.62,
        ..Orbit::default()
    }
}

/// O [`Owners`] desta caixa, com a tolerância do quadro.
pub(crate) fn donos(postas: &[FieldDoc], reg: &Registry, cam: &Orbit, lado_px: u32) -> Owners {
    Owners::new(
        postas,
        reg,
        crate::hit_tolerance(cam.half_extent, lado_px as f32),
    )
}

// ── a fixtura morde antes de medir o produto ──────────────────────────────────────────────────

/// ⭐⭐⭐ **A caixa é a caixa** — e este gate existe porque **uma fixtura errada mede outro
/// programa**, que é o defeito mais caro que esta linha já pagou (cinco vezes, ver o `CLAUDE.md`).
///
/// Ele afirma as três coisas de que todo o resto da `W5` depende:
///
/// 1. a câmera vê a caixa **por dentro** (a esmagadora maioria dos pixels acerta);
/// 2. cada parede é **sua** — o [`Owners`] devolve a folha certa no centro de cada face de dentro;
/// 3. a esquerda é **vermelha** e a direita é **verde**, e não o contrário. *Trocar as duas é
///    exactamente o defeito que o sinal do sangramento existe para apanhar, e ele passaria
///    despercebido numa imagem.*
#[test]
fn a_caixa_de_cornell_e_a_caixa_que_a_regua_supoe() {
    let (doc, postas, materiais) = caixa();
    let reg = Registry::new();
    let cam = camara();
    let (w, h) = (96u32, 96u32);
    let g = trace(&doc, &reg, &cam, w, h);

    // (1) a câmera está dentro do enquadramento da caixa: o fundo é a boca, não o mundo.
    let acertos = g.hit.iter().filter(|h| **h).count();
    let total = (w * h) as usize;
    assert!(
        acertos * 10 >= total * 9,
        "só {acertos} de {total} pixels acertam a caixa — a câmera não está a olhar para dentro"
    );

    let donos = donos(&postas, &reg, &cam, w.min(h));

    // ⛔⛔ **(1-bis) E «90 % acertam» NÃO distingue ver por DENTRO de ver o lado de FORA de uma
    // caixa fechada** — a 1.ª redacção deste gate parava na linha acima e teria passado com a
    // câmera do lado errado, que é a fixtura a medir outro programa. O que separa as duas é
    // **QUANTAS faces aparecem**: de fora vêem-se três, e as duas peças de dentro não se vêem de
    // todo.
    let mut vistas = [false; 7];
    for i in 0..total {
        if g.hit[i]
            && let Some(k) = donos.at(g.point[i])
        {
            vistas[k] = true;
        }
    }
    let faltam: Vec<usize> = (0..7).filter(|k| !vistas[*k]).collect();
    assert!(
        faltam.is_empty(),
        "a câmera não vê as folhas {faltam:?} — de fora da caixa vêem-se três faces e nenhuma das \
         duas peças de dentro"
    );

    // (2) e (3): o dono de um ponto no MEIO de cada face de dentro.
    let meio_de = |f: Face| -> [f32; 3] {
        let d = SALA - 1.0e-3;
        match f {
            Face::Chao => [0.0, -d, 0.0],
            Face::Tecto => [0.0, d, 0.0],
            Face::Fundo => [0.0, 0.0, -d],
            Face::Esquerda => [-d, 0.0, 0.0],
            Face::Direita => [d, 0.0, 0.0],
            // as peças: um ponto no topo de cada uma
            Face::Alta => [-0.20, -SALA + 0.60 - 1.0e-3, -0.15],
            Face::Baixa => [0.21, -SALA + 0.30 - 1.0e-3, 0.18],
        }
    };
    for f in [
        Face::Chao,
        Face::Tecto,
        Face::Fundo,
        Face::Esquerda,
        Face::Direita,
        Face::Alta,
        Face::Baixa,
    ] {
        let p = meio_de(f);
        let quem = donos.at(p);
        assert_eq!(
            quem,
            Some(f as usize),
            "o ponto {p:?} devia ser da folha {f:?} ({}) e é de {quem:?}",
            f as usize
        );
    }

    // (3) as cores, lidas do material e não da memória de quem escreveu a lista.
    let esq = materiais[Face::Esquerda as usize].base_color;
    let dir = materiais[Face::Direita as usize].base_color;
    assert!(
        esq[0] > 2.0 * esq[1] && esq[0] > 2.0 * esq[2],
        "a parede da ESQUERDA tem de ser vermelha, e é {esq:?}"
    );
    assert!(
        dir[1] > 2.0 * dir[0] && dir[1] > 2.0 * dir[2],
        "a parede da DIREITA tem de ser verde, e é {dir:?}"
    );
    assert_eq!(
        materiais.len(),
        postas.len(),
        "um material por folha — a ordem é contrato"
    );
}

/// ⚠️ **O contrato da ordem, do outro lado:** o [`Surfaces`] desta caixa indexa pelo [`Owners`], e
/// um material a mais ou a menos não é erro de compilação em sítio nenhum.
#[test]
fn ha_exactamente_um_material_por_folha_e_o_surfaces_aceita_a_lista() {
    let (_doc, postas, materiais) = caixa();
    let prontos: Vec<ph2d_material::Surface> = materiais.iter().map(OpenPbr::prepare).collect();
    let reg = Registry::new();
    let cam = camara();
    let donos = donos(&postas, &reg, &cam, 96);
    let s = Surfaces {
        all: &prontos,
        owners: Some(&donos),
    };
    assert_eq!(s.all.len(), 7, "as sete faces da caixa");
    assert_eq!(donos.len(), 7, "o Owners conhece as sete");
}

// ── a REFERÊNCIA convergida ───────────────────────────────────────────────────────────────────

/// A lâmpada, no tecto e dentro da sala.
///
/// ⚠️ Ela é **pontual** — ver a divergência declarada no cabeçalho do módulo.
const LAMPADA: crate::PointLamp = crate::PointLamp {
    world: [0.0, SALA - 0.06, 0.0],
    radiance_at_one: [1.4, 1.4, 1.4],
};

/// Até onde um raio de ricochete procura: a diagonal da sala, com folga.
const ALCANCE: f32 = 2.0;

/// ⭐⭐⭐ **A radiância que SAI de um ponto por luz DIRECTA** — a mesma lei do produto, não uma
/// segunda.
///
/// ⚠️⚠️ **Ela chama o [`ph2d_material::Surface::direct`] e reproduz a conta da lâmpada do
/// [`crate::shade_render`] à letra** (`radiance_at_one · visível / d²`, com o mesmo piso). Uma
/// referência que escrevesse a sua própria lei de luz directa mediria **a diferença entre as duas
/// leis** e chamar-lhe-ia erro da GI — *um oráculo que não partilha o que NÃO está a julgar mede
/// outra coisa*.
///
/// ⭐ O que ele partilha de propósito é o MATERIAL; o que ele **não** partilha é a lei do
/// ambiente, que é exactamente a variável que a `W5` vai aproximar.
///
/// ⚠️ Os vectores vêm em MUNDO (o produto passa-os em VISTA). A BSDF só depende dos ângulos entre
/// eles, logo a resposta é a mesma desde que os três estejam no mesmo referencial — e aqui estão.
fn radiancia_directa(
    scene: &crate::march::Scene<'_>,
    q: [f32; 3],
    nq: [f32; 3],
    v: [f32; 3],
    mat: &ph2d_material::Surface,
) -> [f32; 3] {
    let d = [
        LAMPADA.world[0] - q[0],
        LAMPADA.world[1] - q[1],
        LAMPADA.world[2] - q[2],
    ];
    let cru = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
    let piso = crate::POINT_LAMP_MIN_DISTANCE * crate::POINT_LAMP_MIN_DISTANCE;
    if cru <= piso {
        return [0.0; 3];
    }
    let dist = cru.sqrt();
    let l = [d[0] / dist, d[1] / dist, d[2] / dist];
    let lift = scene.sharp.hit * crate::march::BIAS;
    let erguido = [
        q[0] + nq[0] * lift,
        q[1] + nq[1] * lift,
        q[2] + nq[2] * lift,
    ];
    let vis = crate::march::march_shadow_to(scene, &[erguido], &[l], &[dist], 64.0)[0];
    let chega = LAMPADA.radiance_at_one.map(|c| c * vis / cru.max(piso));
    mat.direct(nq, v, l, chega)
}

/// A base tangente de `n` — só a referência a usa.
///
/// ⚠️ Ela tem a descontinuidade que **toda** construção de base a partir de uma normal tem (a
/// [`crate::occlusion::cone_dir`] recusa-a por isso no produto, onde ela viraria uma costura
/// desenhada na peça). Aqui não vincula: a referência **converge**, e uma rotação do conjunto de
/// amostras à volta de `n` não muda a média.
fn base(n: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    let a = if n[0].abs() < 0.9 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    let t = [
        a[1] * n[2] - a[2] * n[1],
        a[2] * n[0] - a[0] * n[2],
        a[0] * n[1] - a[1] * n[0],
    ];
    let inv = (t[0] * t[0] + t[1] * t[1] + t[2] * t[2]).sqrt().recip();
    let t = [t[0] * inv, t[1] * inv, t[2] * inv];
    let b = [
        n[1] * t[2] - n[2] * t[1],
        n[2] * t[0] - n[0] * t[2],
        n[0] * t[1] - n[1] * t[0],
    ];
    (t, b)
}

/// ⭐⭐⭐ **A REFERÊNCIA: a irradiância INDIRECTA num ponto, por força bruta.**
///
/// `amostras` direcções cosseno-distribuídas no hemisfério de `n`; onde o raio bate, a resposta é a
/// radiância que **sai** desse ponto por luz directa ([`radiancia_directa`]); onde falha, é o céu.
///
/// ⚠️ **É UM ricochete, e isso é declarado.** A caixa de Cornell real tem infinitos; o sangramento
/// de cor — que é o que esta régua mede — nasce **no primeiro**. Um segundo ricochete acrescenta
/// magnitude e não muda o sinal.
///
/// ⭐ O valor devolvido é `E(n)/π`, que é exactamente o que o
/// [`ph2d_material::Environment::irradiance`] promete — *«um céu uniforme de radiância `L` responde
/// `L`»*. Com amostragem por cosseno, o estimador disso é a **média simples** da radiância que
/// chega, sem pesos: o `cos` do integral e o `cos/π` da densidade cancelam-se.
#[allow(clippy::too_many_arguments)]
fn irradiancia_indirecta(
    doc: &FieldDoc,
    reg: &Registry,
    cam: &Orbit,
    donos: &Owners,
    prontos: &[ph2d_material::Surface],
    ceu: &dyn ph2d_material::Environment,
    p: [f32; 3],
    n: [f32; 3],
    amostras: u32,
) -> [f32; 3] {
    let shape = ph2d_field_eval::hybrid::Hybrid::new(doc, reg);
    let scene = crate::march::Scene {
        shape: &shape,
        cam,
        basis: cam.basis(),
        sharp: crate::Sharpness::for_frame(cam.half_extent, 256),
        clip: None,
        step: ph2d_field_eval::safe_march_step(doc),
        shrink: ph2d_field_eval::field_shrink(doc, reg),
        stencil: crate::Stencil::Tetra4,
    };
    let (t, b) = base(n);
    let lift = scene.sharp.hit * crate::march::BIAS;
    let erguido = [p[0] + n[0] * lift, p[1] + n[1] * lift, p[2] + n[2] * lift];

    // As direcções: reticulado de Hammersley no disco, erguido ao hemisfério pelo cosseno.
    let mut origens = Vec::with_capacity(amostras as usize);
    let mut dirs = Vec::with_capacity(amostras as usize);
    for k in 0..amostras {
        let u1 = (f32::from(u16::try_from(k % 65536).expect("cabe")) + 0.5) / amostras as f32;
        // van der Corput base 2 — a segunda dimensão do Hammersley.
        let mut bits = k;
        bits = bits.rotate_right(16);
        bits = ((bits & 0x5555_5555) << 1) | ((bits & 0xaaaa_aaaa) >> 1);
        bits = ((bits & 0x3333_3333) << 2) | ((bits & 0xcccc_cccc) >> 2);
        bits = ((bits & 0x0f0f_0f0f) << 4) | ((bits & 0xf0f0_f0f0) >> 4);
        bits = ((bits & 0x00ff_00ff) << 8) | ((bits & 0xff00_ff00) >> 8);
        let u2 = bits as f32 * 2.328_306_4e-10;
        let r = u1.sqrt();
        let fi = std::f32::consts::TAU * u2;
        let (sf, cf) = fi.sin_cos();
        let z = (1.0 - u1).max(0.0).sqrt();
        let d = [
            r * cf * t[0] + r * sf * b[0] + z * n[0],
            r * cf * t[1] + r * sf * b[1] + z * n[1],
            r * cf * t[2] + r * sf * b[2] + z * n[2],
        ];
        origens.push(erguido);
        dirs.push(d);
    }

    let (hit, normal, ponto) =
        crate::march::march_rays(&scene, &origens, &dirs, &[0.0, ALCANCE], &mut |_| None);

    let mut soma = [0.0f32; 3];
    for k in 0..amostras as usize {
        let vinda = if hit[k] {
            let q = ponto[k];
            let nq = normal[k];
            let dono = donos.at(q).unwrap_or(0);
            let v = [-dirs[k][0], -dirs[k][1], -dirs[k][2]];
            radiancia_directa(&scene, q, nq, v, &prontos[dono.min(prontos.len() - 1)])
        } else {
            ceu.radiance(dirs[k], 1.0)
        };
        soma = [soma[0] + vinda[0], soma[1] + vinda[1], soma[2] + vinda[2]];
    }
    soma.map(|c| c / amostras as f32)
}

/// ⏱️ **A CONVERGÊNCIA da referência** — de onde a barra do gate sai, em vez de um número escrito
/// de cabeça.
#[test]
#[ignore = "sonda de calibração: imprime a escada de amostras"]
fn sonda_a_escada_da_convergencia() {
    let (doc, postas, materiais) = caixa();
    let reg = Registry::new();
    let cam = camara();
    let donos = donos(&postas, &reg, &cam, 256);
    let prontos: Vec<ph2d_material::Surface> = materiais.iter().map(OpenPbr::prepare).collect();
    let chao = -SALA + 1.0e-3;
    let cima = [0.0, 1.0, 0.0];
    let tom = |c: [f32; 3]| (c[0] - c[1]) / (c[0] + c[1]).max(1.0e-9);
    println!("  amostras ·      tom junto da VERMELHA ·        tom junto da VERDE");
    for n in [64u32, 256, 1024, 4096, 16384, 65536] {
        let e = irradiancia_indirecta(
            &doc,
            &reg,
            &cam,
            &donos,
            &prontos,
            &Escuro,
            [-SALA + 0.06, chao, 0.0],
            cima,
            n,
        );
        let d = irradiancia_indirecta(
            &doc,
            &reg,
            &cam,
            &donos,
            &prontos,
            &Escuro,
            [SALA - 0.06, chao, 0.0],
            cima,
            n,
        );
        println!(
            "  {n:>8} · {:+.5} (rgb {:.4?}) · {:+.5} (rgb {:.4?})",
            tom(e),
            e,
            tom(d),
            d
        );
    }
}

/// O céu da caixa é **preto**: a sala é iluminada só pela lâmpada do tecto, como a original.
struct Escuro;

impl ph2d_material::Environment for Escuro {
    fn radiance(&self, _dir: [f32; 3], _alpha: f32) -> [f32; 3] {
        [0.0; 3]
    }
    fn irradiance(&self, _n: [f32; 3]) -> [f32; 3] {
        [0.0; 3]
    }
}

/// ⭐⭐⭐ **O NÚMERO DA `W5`: quanto a parede tinge o chão, na referência convergida.**
///
/// Dois pontos do chão, à mesma altura e à mesma profundidade, um a `6 cm` da parede **vermelha** e
/// o outro a `6 cm` da **verde**. A irradiância indirecta em cada um tem de estar **tingida da
/// parede ao lado**, e o sinal é conhecido antes de medir.
///
/// ⛔⛔ **E o CONTROLO é o que o produto faz hoje:** sem luz indirecta a resposta do ambiente é o
/// céu, que nesta caixa é **preto** — logo o sangramento de hoje é exactamente `0`, nos dois
/// pontos. *Uma régua que não mede o lado de hoje não sabe dizer se a wave mexeu alguma coisa.*
///
/// ⚠️ A barra é o **SINAL** e uma margem folgada, nunca o valor exacto: o valor exacto depende do
/// número de ricochetes (aqui, um) e da lâmpada, e prendê-lo aqui faria esta régua reprovar no dia
/// em que a `W5` acrescentar o segundo ricochete — que é uma melhoria.
///
/// # ⭐ De onde a barra sai (a [`sonda_a_escada_da_convergencia`], `load 2`)
///
/// | amostras | tom junto da VERMELHA | tom junto da VERDE |
/// |---:|---:|---:|
/// | `64` | `+0,0628` | `−0,0979` |
/// | `256` | `+0,0711` | `−0,0458` |
/// | `1024` | `+0,0758` | `−0,0373` |
/// | `4096` | `+0,0833` | `−0,0377` |
/// | `16384` | `+0,0826` | `−0,0379` |
/// | `65536` | **`+0,0824`** | **`−0,0377`** |
///
/// ⇒ a `4096` o resíduo contra o convergido é `≤ 0,001`, e a barra é **metade do valor
/// convergido** — `40×` o ruído do estimador e metade da verdade. *Uma barra escrita de cabeça é o
/// que a 1.ª redacção deste gate tinha (`0,05`), e ela reprovava o lado VERDE sobre uma medição
/// correcta.*
///
/// ⚠️ **E as duas pontas do chão não recebem a mesma quantidade de luz** (`0,12` contra `0,45` de
/// soma): o ponto do lado vermelho está numa zona mais escura, e a escada mostra-o a convergir
/// depressa enquanto o outro lado ainda se move a `256` amostras — a assinatura de uma direcção
/// rara e muito brilhante de um lado e de nenhuma do outro. ⏳ **A causa não está medida** (a peça
/// alta fica desse lado, e é a hipótese óbvia), e por isso o tom é uma **fracção** e não uma
/// diferença absoluta: uma diferença absoluta mediria o brilho do sítio, não o tingimento.
#[test]
fn a_parede_tinge_o_chao_e_o_sinal_e_conhecido() {
    let (doc, postas, materiais) = caixa();
    let reg = Registry::new();
    let cam = camara();
    let donos = donos(&postas, &reg, &cam, 256);
    let prontos: Vec<ph2d_material::Surface> = materiais.iter().map(OpenPbr::prepare).collect();

    const AMOSTRAS: u32 = 4096;
    let chao = -SALA + 1.0e-3;
    let cima = [0.0, 1.0, 0.0];
    let perto_da_vermelha = [-SALA + 0.06, chao, 0.0];
    let perto_da_verde = [SALA - 0.06, chao, 0.0];

    let e = irradiancia_indirecta(
        &doc,
        &reg,
        &cam,
        &donos,
        &prontos,
        &Escuro,
        perto_da_vermelha,
        cima,
        AMOSTRAS,
    );
    let d = irradiancia_indirecta(
        &doc,
        &reg,
        &cam,
        &donos,
        &prontos,
        &Escuro,
        perto_da_verde,
        cima,
        AMOSTRAS,
    );

    // O sangramento: quanto o vermelho ganha ao verde, em fracção da soma. Positivo = tingido de
    // vermelho.
    let tom = |c: [f32; 3]| (c[0] - c[1]) / (c[0] + c[1]).max(1.0e-9);
    println!(
        "  REFERÊNCIA ({AMOSTRAS} amostras, 1 ricochete)\n    \
         junto da VERMELHA  rgb {:.5?}  tom {:+.4}\n    \
         junto da VERDE     rgb {:.5?}  tom {:+.4}",
        e,
        tom(e),
        d,
        tom(d)
    );

    assert!(
        e[0] > 0.0 && d[1] > 0.0,
        "a referência não trouxe luz nenhuma — a lâmpada ou a marcha não estão a chegar ao chão"
    );
    // ⭐ As barras são METADE do valor convergido — ver a escada na doc deste gate.
    const BARRA_VERMELHA: f32 = 0.041;
    const BARRA_VERDE: f32 = -0.018;
    assert!(
        tom(e) > BARRA_VERMELHA,
        "junto da parede VERMELHA o chão tinha de ficar mais vermelho que verde, e o tom é {:+.4}",
        tom(e)
    );
    assert!(
        tom(d) < BARRA_VERDE,
        "junto da parede VERDE o chão tinha de ficar mais verde que vermelho, e o tom é {:+.4}",
        tom(d)
    );

    // ⭐ O CONTROLO: o ambiente de hoje é o céu, e nesta caixa ele é preto.
    let hoje_e = Escuro.irradiance(cima);
    let hoje_d = Escuro.irradiance(cima);
    println!(
        "  HOJE (o ambiente é o céu, sem cena)\n    junto das duas    rgb {hoje_e:.5?} / {hoje_d:.5?}"
    );
    assert_eq!(
        hoje_e, hoje_d,
        "sem luz indirecta os dois pontos são indistinguíveis — é este o buraco que a W5 fecha"
    );
}
