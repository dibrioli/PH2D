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
use ph2d_material::OpenPbr;

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
