//! Os gates da malha que leva os pesos dentro.

use super::SkinnedMesh;
use ph2d_poly2d::Mesh2d;

fn malha(verts: usize) -> Mesh2d {
    Mesh2d {
        #[expect(
            clippy::cast_precision_loss,
            reason = "fixtura de meia dúzia de vértices"
        )]
        rest: (0..verts).map(|i| [i as f64, 0.0]).collect(),
        tris: vec![[0, 1, 2]],
        size: [10, 10],
    }
}

/// ⭐⭐⭐ **A CONTAGEM DE OSSOS É DERIVADA, e a fatia de cada vértice sai certa.**
#[test]
fn a_contagem_de_ossos_deriva_se_e_cada_vertice_le_a_fatia_dele() {
    let s = SkinnedMesh {
        mesh: malha(4),
        pesos: vec![
            1.0, 0.0, 0.0, // v0
            0.5, 0.5, 0.0, // v1
            0.0, 1.0, 0.0, // v2
            0.0, 0.0, 1.0, // v3
        ],
    };
    assert_eq!(s.ossos(), 3, "4 vertices x 3 ossos = 12 pesos");
    assert!(s.valida());
    assert_eq!(s.pesos_de(1), &[0.5, 0.5, 0.0]);
    assert_eq!(s.pesos_de(3), &[0.0, 0.0, 1.0]);
    assert!(s.pesos_de(4).is_empty(), "fora da malha nao inventa fatia");
}

/// ⛔⛔ **UMA TABELA QUE NÃO FECHA É RECUSADA, e não lida deslocada.**
///
/// ⚠️ **É o modo de falha que importa:** uma tabela a que falte um vértice ainda entrega pesos
/// *plausíveis* a todo vértice — só que os do vizinho. A arte sai deformada de um jeito que passa
/// por todo gate de geometria (a soma é 1, nada é negativo, nada é órfão).
///
/// (Mutação: trocar `% n == 0` por `>= n` ⇒ RED.)
#[test]
fn uma_tabela_que_nao_fecha_e_recusada() {
    let s = SkinnedMesh {
        mesh: malha(4),
        // 11 pesos para 4 vértices — não é múltiplo de nada.
        pesos: vec![0.25; 11],
    };
    assert!(
        !s.valida(),
        "11 nao e' multiplo de 4 e a porta tem de o dizer"
    );
}

/// ⭐ **SEM PESOS É UM ESTADO LEGAL** — a leitura de *«resolve pela lei derivada»*.
#[test]
fn sem_pesos_e_um_estado_legal_e_diz_zero_ossos() {
    let s = SkinnedMesh::sem_pesos(malha(4));
    assert_eq!(s.ossos(), 0);
    assert!(s.valida(), "uma malha sem tabela e' coerente");
    assert!(s.pesos_de(0).is_empty());
}

/// ⭐⭐⭐ **A MALHA E OS PESOS ATRAVESSAM O ARQUIVO JUNTOS** — não há como gravar uma sem a outra.
///
/// ⚠️ Este é o gate que substitui a objecção do *vector paralelo*: a ida-e-volta é de **um**
/// objecto, logo uma edição não tem por onde dessincronizar as duas listas.
#[test]
fn a_malha_e_os_pesos_atravessam_o_arquivo_juntos() {
    let s = SkinnedMesh {
        mesh: malha(3),
        pesos: vec![1.0, 0.0, 0.25, 0.75, 0.5, 0.5],
    };
    let bytes = postcard::to_allocvec(&s).expect("serializa");
    let volta: SkinnedMesh = postcard::from_bytes(&bytes).expect("desserializa");
    assert_eq!(volta, s, "a ida-e-volta perdeu alguma coisa");
    assert_eq!(volta.ossos(), 2);
}

/// ⭐⭐⭐ **UM BIND GRAVADO ANTES DO CAMPO CONTINUA A LER-SE, COM OS PESOS INTACTOS.**
///
/// # ⛔⛔ O defeito que este gate impede
///
/// O postcard é **posicional**: apendar o `campo` ao [`super::SkinnedPath`] faz os bytes de um bind
/// já gravado acabarem cedo. Sem a leitura versionada o `from_bytes` devolve `Err`, a
/// [`super::le`] devolve `None`, e quem chama lê isso como *«esta fonte não se lê»* e resolve pela
/// **lei derivada** — ou seja, **toda forma já presa perderia a tabela do padrão-ouro em
/// silêncio**, e o desenho dela mudava sem ninguém lhe ter tocado.
///
/// ⚠️ **A fixtura são os bytes da forma ANTIGA**, montados aqui pela struct que a descreve — ⛔ não
/// os bytes da nova com o campo a `None`, que seriam outro teste (esses têm o byte da tag e leem-se
/// pelo caminho novo).
#[test]
fn um_bind_gravado_antes_do_campo_ainda_le_os_pesos() {
    #[derive(serde::Serialize)]
    struct ComoEraAntes {
        path: ph2d_vec_scene::VecPath,
        pesos: Vec<f64>,
    }
    let antes = ComoEraAntes {
        path: ph2d_vec_scene::cook(
            ph2d_vec_scene::ShapeKind::Rectangle,
            [0.0, 0.0],
            [4.0, 2.0],
            &[],
        ),
        pesos: vec![0.25; 4 * 3 * 2],
    };
    let bytes = postcard::to_allocvec(&antes).expect("serializa a forma antiga");

    let lido = super::le(&bytes).expect("a forma ANTIGA tem de continuar a ler-se");
    assert_eq!(
        lido.pesos, antes.pesos,
        "os pesos do padrão-ouro de um bind já gravado não se podem perder"
    );
    assert!(
        lido.campo.is_none(),
        "um bind anterior não tem campo, e isso é a resposta — não um erro"
    );
    assert!(
        lido.valida(),
        "o par caminho/tabela continua a fechar depois da leitura versionada"
    );
}

/// ⭐⭐ **O CONTROLO do gate acima** — a forma NOVA passa pelo caminho novo, com o campo dentro.
///
/// ⛔ Sem ele, uma [`super::le`] que ignorasse o campo e caísse SEMPRE na forma antiga ficaria
/// verde no irmão, e o campo nunca chegaria ao quadro.
#[test]
fn a_forma_nova_atravessa_o_arquivo_com_o_campo_dentro() {
    let path = ph2d_vec_scene::cook(
        ph2d_vec_scene::ShapeKind::Rectangle,
        [0.0, 0.0],
        [40.0, 10.0],
        &[],
    );
    let eixos = vec![
        ph2d_skin_weights::Handle {
            a: [0.0, 5.0],
            b: [20.0, 5.0],
        },
        ph2d_skin_weights::Handle {
            a: [20.0, 5.0],
            b: [40.0, 5.0],
        },
    ];
    let campo = ph2d_vec_skin::pesos::campo_do_caminho(&path, &eixos).expect("campo");
    let pesos = ph2d_vec_skin::pesos::pesos_dos_pontos(&path, &campo);
    let g = super::SkinnedPath {
        path,
        pesos,
        campo: Some(campo),
    };
    let bytes = super::grava(&g).expect("grava");
    let volta = super::le(&bytes).expect("lê");
    assert_eq!(volta, g, "a ida-e-volta do campo perdeu alguma coisa");
    let c = volta.campo.expect("o campo tem de atravessar");
    assert!(c.valida(), "o campo chegou por fechar");
    assert!(
        c.malha.rest.len() > 100,
        "a malha do domínio tem de atravessar inteira: {} vértices",
        c.malha.rest.len()
    );
}

/// ⭐ **A SONDA QUE RESPONDE À PERGUNTA HONESTA: com o bind a SUBDIVIDIR, o campo ainda importa?**
///
/// A F32 pôs a subdivisão no bind, e segmentos curtos aproximam a recta do campo por construção —
/// ⇒ *a wave do campo podia ser teórica*. Esta sonda mede o erro da recta nos DOIS estados, pela
/// mesma lei do produto. Ela IMPRIME; quem afirma são os gates da `ph2d-vec-skin`.
#[test]
fn diag_a_subdivisao_do_bind_ja_mata_o_erro_da_recta() {
    use ph2d_skin_weights::Handle;
    const W: f64 = 40.0;
    const H: f64 = 10.0;
    let eixos = vec![
        Handle {
            a: [0.0, H * 0.5],
            b: [W * 0.5, H * 0.5],
        },
        Handle {
            a: [W * 0.5, H * 0.5],
            b: [W, H * 0.5],
        },
    ];

    let erro_da_recta = |subdividir: bool| -> (usize, f64) {
        let mut src = ph2d_vec_scene::cook(
            ph2d_vec_scene::ShapeKind::Rectangle,
            [0.0, 0.0],
            [W, H],
            &[],
        );
        if subdividir && let Some(alvo) = crate::subdivisao::alvo_dos_eixos(&eixos) {
            crate::subdivisao::subdivide(&mut src, alvo, crate::subdivisao::VERTICES_MAX);
        }
        let campo = ph2d_vec_skin::pesos::campo_do_caminho(&src, &eixos).expect("campo");
        let tabela = ph2d_vec_skin::pesos::pesos_dos_pontos(&src, &campo);
        let n = campo.ossos();
        let cozido = src.cooked();
        let (verts, _) = cozido.contour(0).expect("contorno");
        let nv = verts.len();
        let mut pior = 0.0_f64;
        for i in 0..nv {
            let (a, b) = (&verts[i], &verts[(i + 1) % nv]);
            let (wa, wb) = (
                &tabela[i * 3 * n..i * 3 * n + n],
                &tabela[((i + 1) % nv) * 3 * n..((i + 1) % nv) * 3 * n + n],
            );
            for k in 0..=64 {
                let t = f64::from(k) / 64.0;
                let u = 1.0 - t;
                let (c0, c1, c2, c3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
                let p = [
                    c0 * a.anchor[0]
                        + c1 * a.out_handle[0]
                        + c2 * b.in_handle[0]
                        + c3 * b.anchor[0],
                    c0 * a.anchor[1]
                        + c1 * a.out_handle[1]
                        + c2 * b.in_handle[1]
                        + c3 * b.anchor[1],
                ];
                let Some(verdade) = campo.linha(p) else {
                    continue;
                };
                let d = (0..n)
                    .map(|j| (wb[j].mul_add(t, wa[j] * (1.0 - t)) - verdade[j]).abs())
                    .fold(0.0_f64, f64::max);
                pior = pior.max(d);
            }
        }
        (nv, pior)
    };

    let (nos_cru, erro_cru) = erro_da_recta(false);
    let (nos_sub, erro_sub) = erro_da_recta(true);
    println!("\n{:-<64}", "");
    println!("{:<26} {:>8} {:>14}", "estado", "nós", "erro da recta");
    println!("{:-<64}", "");
    println!(
        "{:<26} {nos_cru:>8} {erro_cru:>14.4}",
        "como o artista desenha"
    );
    println!(
        "{:<26} {nos_sub:>8} {erro_sub:>14.4}",
        "como o BIND entrega"
    );
    println!("{:-<64}", "");
    println!(
        "a subdivisão do bind corta o erro em {:.1}× — e o que SOBRA é o que o campo cura",
        if erro_sub > 0.0 {
            erro_cru / erro_sub
        } else {
            f64::INFINITY
        }
    );
}

/// ⭐⭐⭐ **A SONDA QUE DECIDE SE O DONO VÊ ALGUMA COISA** — o DESENHO, na configuração do produto.
///
/// ⚠️ Ela mede a arte e não os pesos: um erro de peso de `3 %` pode valer muito ou nada consoante
/// o quanto o osso roda. *A pergunta do dono é sobre o que ele VÊ.*
#[test]
fn diag_o_desenho_com_e_sem_campo_na_configuracao_do_produto() {
    use ph2d_skeleton::{Correccao, Especie, Skin, SkinBone, Xform};
    use ph2d_skin_weights::Handle;
    const W: f64 = 40.0;
    const H: f64 = 10.0;
    let eixos = vec![
        Handle {
            a: [0.0, H * 0.5],
            b: [W * 0.5, H * 0.5],
        },
        Handle {
            a: [W * 0.5, H * 0.5],
            b: [W, H * 0.5],
        },
    ];
    // ⛔⛔⛔ **Esta fixtura escreveu `tendon: [0, 0]` até 2026-09-20 e mediu OUTRO PROGRAMA.**
    // A [`SkinBone::new`] crava `tendon: 0` («o neutro honesto de um osso SOZINHO»), logo com dois
    // ossos as duas colunas do campo colapsam **numa** e a sonda lia o colapso como se fosse a lei.
    // A porta que atribui tendão é a [`ph2d_skeleton::SkinBone::bent`], e é por ela que o produto
    // passa; uma fixtura multi-osso montada à mão tem de o dizer em voz alta.
    let pele = |dobra: f64| {
        let osso = |x0: f64, rot: f64, t: u32| {
            let (c, s) = (rot.cos(), rot.sin());
            let b = SkinBone::new(
                Xform([1.0, 0.0, 0.0, 1.0, x0, H * 0.5]),
                W * 0.5,
                1.0,
                Xform([c, s, -s, c, x0, H * 0.5]),
                Xform::IDENTITY,
            )
            .expect("repouso");
            SkinBone { tendon: t, ..b }
        };
        let k = Skin::new(vec![osso(0.0, 0.0, 0), osso(W * 0.5, dobra, 1)]).expect("2 ossos");
        // ⚠️ O guarda vive AQUI porque a fixtura é uma closure: não há onde um censo a alcançar.
        let t: Vec<u32> = k.bones().iter().map(|b| b.tendon).collect();
        assert_eq!(t, vec![0, 1], "os tendões desta fixtura NÃO podem colapsar");
        k
    };

    let desvio = |forma: ph2d_vec_scene::ShapeKind,
                  subdividir: bool,
                  dobra: f64,
                  manchas: &[Correccao]|
     -> (usize, f64) {
        let mut src = ph2d_vec_scene::cook(forma, [0.0, 0.0], [W, H], &[]);
        if subdividir && let Some(alvo) = crate::subdivisao::alvo_dos_eixos(&eixos) {
            crate::subdivisao::subdivide(&mut src, alvo, crate::subdivisao::VERTICES_MAX);
        }
        let campo = ph2d_vec_skin::pesos::campo_do_caminho(&src, &eixos).expect("campo");
        let tabela = ph2d_vec_skin::pesos::pesos_dos_pontos(&src, &campo);
        let k = pele(dobra);
        let (mut a, mut b) = (src.clone(), src.clone());
        ph2d_vec_skin::curva::aplica_pela_curva_com(&k, &mut a, &tabela, manchas, true, None);
        ph2d_vec_skin::curva::aplica_pela_curva_com(
            &k,
            &mut b,
            &tabela,
            manchas,
            true,
            Some(&campo),
        );
        (
            src.verts_all().count(),
            crate::test_support::pior_desvio_do_desenho(&a, &b),
        )
    };

    // Uma mancha no MEIO de uma aresta longa — o sítio da queixa do dono.
    let mancha = [Correccao {
        tendon: 0,
        centro: [W * 0.5, 0.0],
        raio: W * 0.25,
        especie: Especie::Soma(0.5),
    }];

    println!("\n{:-<78}", "");
    println!(
        "{:<34} {:>6} {:>12} {:>18}",
        "caso", "nós", "desvio", "% da peça (40 u)"
    );
    println!("{:-<78}", "");
    for (nome, forma, sub, dobra, m) in [
        (
            "rect cru · dobra 0,8",
            ph2d_vec_scene::ShapeKind::Rectangle,
            false,
            0.8,
            &[][..],
        ),
        (
            "rect BIND · dobra 0,8",
            ph2d_vec_scene::ShapeKind::Rectangle,
            true,
            0.8,
            &[][..],
        ),
        (
            "rect BIND · 0,8 · PINTADO",
            ph2d_vec_scene::ShapeKind::Rectangle,
            true,
            0.8,
            &mancha[..],
        ),
        (
            "rect BIND · 1,5 · PINTADO",
            ph2d_vec_scene::ShapeKind::Rectangle,
            true,
            1.5,
            &mancha[..],
        ),
        // ⚠️ Uma forma CURVA é mais perto da arte real que um rectângulo — as alças deixam de ser
        // degeneradas, e é a alça que a lei da curva corrige.
        (
            "elipse BIND · dobra 0,8",
            ph2d_vec_scene::ShapeKind::Ellipse,
            true,
            0.8,
            &[][..],
        ),
        (
            "elipse BIND · 1,5 · PINTADO",
            ph2d_vec_scene::ShapeKind::Ellipse,
            true,
            1.5,
            &mancha[..],
        ),
    ] {
        let (n, d) = desvio(forma, sub, dobra, m);
        println!("{nome:<34} {n:>6} {d:>12.5} {:>17.3} %", d / W * 100.0);
    }
    println!("{:-<78}", "");
    println!(
        "loadavg: {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
}

/// ⚠️ **SONDA — o BRAÇO DA CENA DO DONO, com e sem o campo, pelo caminho do PRODUTO.**
///
/// ⛔⛔ Ela existe porque o dono reportou *«parece sem mudanças»* (2026-09-20) sobre a cena
/// `PH2D_VEC_BONE_SMOKE=1`, e as tabelas da wave saíram de um rectângulo `40 × 10` e de uma elipse
/// — *a barra da cena é um `7 × 1` com as pontas redondas*, que é outra forma. **A fixtura tem de
/// ser a da cena que o dono usou.**
///
/// ⛔⛔⛔ **O DENOMINADOR desta sonda estava ERRADO por `7×`, e eu reportei o número ao dono.**
/// Ela dividia por `7,0` — o COMPRIMENTO da barra — um desvio que vive **ATRAVÉS** dela: o campo
/// corrige a arte junto do cotovelo, e a grandeza que ali diz *«isto vê-se?»* é a **ESPESSURA**
/// (`1,0`), que é o tamanho da feição. Dividir pelo comprimento faz toda barra comprida parecer
/// sete vezes mais fiel do que é, e a razão é que *um número grande no denominador não é a peça: é
/// a direcção em que nada acontece.*
///
/// ⇒ ela imprime as **duas** colunas de propósito. A que a prosa cita é a da ESPESSURA, e a do
/// comprimento fica ao lado para a troca ser visível a quem ler a tabela antiga.
#[test]
fn diag_o_braco_da_cena_do_dono() {
    use crate::barra_da_cena_tests_support::barra_da_cena_com;
    use ph2d_ecs::{Entity, Transform};
    use ph2d_skeleton_ecs::SkinBind;

    /// A barra da cena é um `7 × 1`. O desvio do campo é TRANSVERSAL ao eixo dos ossos.
    const ESPESSURA: f64 = 1.0;
    const COMPRIMENTO: f64 = 7.0;

    println!("\n{:-<103}", "");
    println!(
        "{:<24} {:>5} {:>7} {:>7} {:>11} {:>16} {:>15}",
        "caso", "nós", "malha", "graus", "desvio", "% ESPESSURA (1 u)", "% compr. (7 u)"
    );
    println!("{:-<103}", "");
    for subdividir in [false, true] {
        let (mut sim, scene, map, id, ossos) = barra_da_cena_com(subdividir);
        let e = Entity::from_bits(map[&id]);
        let skin = sim.world().get::<SkinBind>(e).expect("pele").clone();
        let g = crate::skinned_mesh::le(&skin.source).expect("a fonte lê-se");
        let nos = g.path.verts_all().count();
        let malha = g.campo.as_ref().map_or(0, |c| c.malha.rest.len());
        for graus in [20.0f32, 45.0, 70.0] {
            for o in &ossos[1..] {
                sim.world_mut()
                    .get_mut::<Transform>(*o)
                    .expect("Transform")
                    .rotation = graus.to_radians();
            }
            let (mut sem, mut com) = (scene.clone(), scene.clone());
            crate::skin_live::recook_com_mistura(&sim, &mut sem, true, true, false);
            crate::skin_live::recook_com_mistura(&sim, &mut com, true, true, true);
            let pega = |s: &ph2d_vec_scene::VecScene| {
                s.paths().iter().find(|p| p.id == id).expect("path").clone()
            };
            let d = crate::test_support::pior_desvio_do_desenho(&pega(&sem), &pega(&com));
            let cru = pega(&scene);
            let dobrou = crate::test_support::pior_desvio_do_desenho(&cru, &pega(&sem));
            println!(
                "{:<24} {nos:>5} {malha:>7} {graus:>7.0} {d:>11.5} {:>15.2} % {:>13.2} %  (a dobra move {dobrou:.4})",
                if subdividir { "BIND (produto)" } else { "cru" },
                d / ESPESSURA * 100.0,
                d / COMPRIMENTO * 100.0
            );
        }
    }
    println!("{:-<103}", "");
}

/// ⚠️ **SONDA — a mesma forma em TAMANHOS diferentes.** A lei é geométrica, logo a coluna do
/// `% da peça` TEM de ser a mesma; se não for, quem manda é um número ABSOLUTO escondido.
#[test]
fn diag_o_campo_e_invariante_a_escala() {
    use ph2d_skeleton::{Skin, SkinBone, Xform};
    use ph2d_skin_weights::Handle;

    println!("\n{:-<72}", "");
    println!(
        "{:<22} {:>6} {:>8} {:>12} {:>14}",
        "forma", "nós", "malha", "desvio", "% da peça"
    );
    println!("{:-<72}", "");
    for (rotulo, w, h, forma) in [
        (
            "rect 40x10",
            40.0_f64,
            10.0_f64,
            ph2d_vec_scene::ShapeKind::Rectangle,
        ),
        ("rect 7x1", 7.0, 1.0, ph2d_vec_scene::ShapeKind::Rectangle),
        (
            "rect 4x1 (mesma razão)",
            40.0,
            10.0,
            ph2d_vec_scene::ShapeKind::Rectangle,
        ),
        (
            "elipse 40x10",
            40.0,
            10.0,
            ph2d_vec_scene::ShapeKind::Ellipse,
        ),
        ("elipse 7x1", 7.0, 1.0, ph2d_vec_scene::ShapeKind::Ellipse),
        ("elipse 6x2", 6.0, 2.0, ph2d_vec_scene::ShapeKind::Ellipse),
    ] {
        let eixos = vec![
            Handle {
                a: [0.0, h * 0.5],
                b: [w * 0.5, h * 0.5],
            },
            Handle {
                a: [w * 0.5, h * 0.5],
                b: [w, h * 0.5],
            },
        ];
        let osso = |x0: f64, rot: f64| {
            let (c, s) = (rot.cos(), rot.sin());
            SkinBone::new(
                Xform([1.0, 0.0, 0.0, 1.0, x0, h * 0.5]),
                w * 0.5,
                1.0,
                Xform([c, s, -s, c, x0, h * 0.5]),
                Xform::IDENTITY,
            )
            .expect("repouso")
        };
        let k = Skin::new(vec![osso(0.0, 0.0), osso(w * 0.5, 0.8)]).expect("2 ossos");
        let mut src = ph2d_vec_scene::cook(forma, [0.0, 0.0], [w, h], &[]);
        if let Some(alvo) = crate::subdivisao::alvo_dos_eixos(&eixos) {
            crate::subdivisao::subdivide(&mut src, alvo, crate::subdivisao::VERTICES_MAX);
        }
        let campo = ph2d_vec_skin::pesos::campo_do_caminho(&src, &eixos).expect("campo");
        let tabela = ph2d_vec_skin::pesos::pesos_dos_pontos(&src, &campo);
        let (mut a, mut b) = (src.clone(), src.clone());
        ph2d_vec_skin::curva::aplica_pela_curva_com(&k, &mut a, &tabela, &[], true, None);
        ph2d_vec_skin::curva::aplica_pela_curva_com(&k, &mut b, &tabela, &[], true, Some(&campo));
        let d = crate::test_support::pior_desvio_do_desenho(&a, &b);
        println!(
            "{rotulo:<22} {:>6} {:>8} {d:>12.6} {:>13.3} %",
            src.verts_all().count(),
            campo.malha.rest.len(),
            d / w * 100.0
        );
    }
    println!("{:-<72}", "");
}

// ═══════════════════════════════════════════════════════════════════════════════════════════
// LENTE B — AS RÉGUAS **LOCAIS** DA DEFORMAÇÃO VECTORIAL (auditoria ordenada em 2026-09-20)
//
// ⛔⛔ A casa mede sobretudo `pior_desvio_do_desenho`, que é um MÁXIMO GLOBAL entre DUAS SAÍDAS
// NOSSAS. Ele responde *«a lei A e a lei B concordam?»* e é cego a tudo o que um artista chama de
// irregularidade: área perdida, contorno a cruzar-se, vinco, braço a afinar.
//
// ⭐ Estas sondas medem contra um PADRÃO-OURO — a mesma forma deformada como a mídia IMAGEM faz
// (cada ponto pelo peso DELE, lido do `CampoDoDominio` guardado no bind) — e são todas p50/p90/max.
// ═══════════════════════════════════════════════════════════════════════════════════════════

/// Amostras por segmento de cúbica. ⚠️ Fixo, e a parametrização `(segmento, t)` é a MESMA na fonte
/// e no produto — é isso que deixa comparar ponto a ponto além de curva a curva.
const B_N: usize = 32;

fn b_cub(v: &[ph2d_vec_scene::VecVertex], k: usize) -> [[f64; 2]; 4] {
    let n = v.len();
    let (a, b) = (&v[k], &v[(k + 1) % n]);
    [a.anchor, a.out_handle, b.in_handle, b.anchor]
}

fn b_eval(c: &[[f64; 2]; 4], t: f64) -> [f64; 2] {
    let u = 1.0 - t;
    let (w0, w1, w2, w3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
    [
        w0 * c[0][0] + w1 * c[1][0] + w2 * c[2][0] + w3 * c[3][0],
        w0 * c[0][1] + w1 * c[1][1] + w2 * c[2][1] + w3 * c[3][1],
    ]
}

/// O contorno `0` amostrado na parametrização `(segmento, t)`, `t ∈ [0,1)`.
fn b_amostra(p: &ph2d_vec_scene::VecPath) -> Vec<[f64; 2]> {
    let cozido = p.cooked();
    let Some((v, _)) = cozido.contour(0) else {
        return Vec::new();
    };
    let n = v.len();
    let mut out = Vec::with_capacity(n * B_N);
    for k in 0..n {
        let c = b_cub(v, k);
        for i in 0..B_N {
            #[expect(clippy::cast_precision_loss, reason = "i < B_N")]
            out.push(b_eval(&c, i as f64 / B_N as f64));
        }
    }
    out
}

/// As âncoras do contorno `0`, na ordem dos nós.
fn b_ancoras(p: &ph2d_vec_scene::VecPath) -> Vec<[f64; 2]> {
    let cozido = p.cooked();
    cozido
        .contour(0)
        .map(|(v, _)| v.iter().map(|x| x.anchor).collect())
        .unwrap_or_default()
}

fn b_pct(v: &mut [f64]) -> (f64, f64, f64) {
    if v.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    v.sort_by(|a, b| a.total_cmp(b));
    #[expect(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "índice de percentil"
    )]
    let q = |f: f64| v[(((v.len() - 1) as f64) * f).round() as usize];
    (q(0.5), q(0.9), v[v.len() - 1])
}

/// A distância de `p` à POLILINHA FECHADA `poli` — *«o desenho passa por aqui?»*.
fn b_dist(p: [f64; 2], poli: &[[f64; 2]]) -> f64 {
    let n = poli.len();
    let mut melhor = f64::INFINITY;
    for i in 0..n {
        let (a, b) = (poli[i], poli[(i + 1) % n]);
        let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
        let l2 = dx.mul_add(dx, dy * dy);
        let t = if l2 > 0.0 {
            (((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / l2).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let d = (p[0] - t.mul_add(dx, a[0])).hypot(p[1] - t.mul_add(dy, a[1]));
        if d < melhor {
            melhor = d;
        }
    }
    melhor
}

/// O perfil de afastamento de `a` a `b` — p50 / p90 / max.
fn b_perfil(a: &[[f64; 2]], b: &[[f64; 2]]) -> (f64, f64, f64) {
    let mut d: Vec<f64> = a.iter().map(|&p| b_dist(p, b)).collect();
    b_pct(&mut d)
}

fn b_area(poli: &[[f64; 2]]) -> f64 {
    ph2d_poly2d::signed_area(poli).abs()
}

/// Quantos pares de segmentos NÃO adjacentes se cruzam de verdade.
fn b_auto(poli: &[[f64; 2]], eps: f64) -> usize {
    let n = poli.len();
    let curto = |a: [f64; 2], b: [f64; 2]| (a[0] - b[0]).hypot(a[1] - b[1]) <= eps;
    let sinal = |o: [f64; 2], u: [f64; 2], v: [f64; 2]| {
        ((u[0] - o[0]) * (v[1] - o[1]) - (v[0] - o[0]) * (u[1] - o[1])).signum()
    };
    let mut c = 0usize;
    for i in 0..n {
        let (a1, a2) = (poli[i], poli[(i + 1) % n]);
        if curto(a1, a2) {
            continue;
        }
        for j in (i + 2)..n {
            if i == 0 && j == n - 1 {
                continue;
            }
            let (b1, b2) = (poli[j], poli[(j + 1) % n]);
            if curto(b1, b2) {
                continue;
            }
            if a1[0].max(a2[0]) < b1[0].min(b2[0])
                || b1[0].max(b2[0]) < a1[0].min(a2[0])
                || a1[1].max(a2[1]) < b1[1].min(b2[1])
                || b1[1].max(b2[1]) < a1[1].min(a2[1])
            {
                continue;
            }
            if sinal(a1, a2, b1) != sinal(a1, a2, b2) && sinal(b1, b2, a1) != sinal(b1, b2, a2) {
                c += 1;
            }
        }
    }
    c
}

/// Os pesos do vértice da malha do domínio mais próximo — a resposta que a mídia imagem daria a um
/// ponto sem triângulo (a malha do bind é uma GRELHA, logo a curva pode sair dela).
fn b_mais_proximo(campo: &ph2d_vec_skin::pesos::CampoDoDominio, p: [f64; 2]) -> Vec<f64> {
    let mut melhor = (f64::INFINITY, 0usize);
    for i in 0..campo.malha.rest.len() {
        if let Some(q) = campo.local_do_vertice(i) {
            let d = (q[0] - p[0]).hypot(q[1] - p[1]);
            if d < melhor.0 {
                melhor = (d, i);
            }
        }
    }
    campo
        .linha_do_vertice(melhor.1)
        .map(<[f64]>::to_vec)
        .unwrap_or_default()
}

/// ⭐⭐⭐ **O PADRÃO-OURO NUM PONTO** — a lei da mídia IMAGEM aplicada ao contorno: cada ponto pelo
/// peso DELE, lido do campo, misturado pela mesma [`ph2d_skeleton::Skin::blend`] do produto.
///
/// Devolve `(imagem, veio_do_campo)`.
fn b_ouro_pt(
    pele: &ph2d_skeleton::Skin,
    campo: &ph2d_vec_skin::pesos::CampoDoDominio,
    correcoes: &[ph2d_skeleton::Correccao],
    p: [f64; 2],
) -> ([f64; 2], bool) {
    let mut w = pele.scratch();
    let dentro = campo.linha(p);
    let ok = dentro.is_some();
    let linha = dentro.unwrap_or_else(|| b_mais_proximo(campo, p));
    pele.weights_corrected(p, Some(&linha), &mut w, correcoes);
    (pele.blend(p, &w), ok)
}

/// ⭐ **A LARGURA DO BRAÇO ao longo dele** — o par de pontos de repouso que estão um por cima do
/// outro, medido DEPOIS da deformação. Em repouso a barra tem `1,0` de espessura em todo o troço
/// recto (`x ∈ [-8, -2]`), logo toda leitura diferente de `1,0` é encolhimento ou inchaço.
fn b_larguras(rest: &[[f64; 2]], def: &[[f64; 2]]) -> Vec<f64> {
    let mut out = Vec::new();
    for s in 0..=12 {
        let x = f64::from(s).mul_add(6.0 / 12.0, -8.0);
        let (mut cima, mut baixo) = ((f64::INFINITY, usize::MAX), (f64::INFINITY, usize::MAX));
        for (i, r) in rest.iter().enumerate() {
            let d = (r[0] - x).abs();
            if (r[1] - 3.0).abs() < 1e-6 {
                if d < cima.0 {
                    cima = (d, i);
                }
            } else if (r[1] - 2.0).abs() < 1e-6 && d < baixo.0 {
                baixo = (d, i);
            }
        }
        if cima.1 == usize::MAX || baixo.1 == usize::MAX || cima.0 > 0.25 || baixo.0 > 0.25 {
            continue;
        }
        let (a, b) = (def[cima.1], def[baixo.1]);
        out.push((a[0] - b[0]).hypot(a[1] - b[1]));
    }
    out
}

/// A QUEBRA DA TANGENTE em cada nó, em graus — `0` num nó liso.
fn b_quebra_nos(p: &ph2d_vec_scene::VecPath) -> Vec<f64> {
    let cozido = p.cooked();
    let Some((v, _)) = cozido.contour(0) else {
        return Vec::new();
    };
    v.iter()
        .filter_map(|x| {
            let ent = [x.anchor[0] - x.in_handle[0], x.anchor[1] - x.in_handle[1]];
            let sai = [x.out_handle[0] - x.anchor[0], x.out_handle[1] - x.anchor[1]];
            let (le, ls) = (ent[0].hypot(ent[1]), sai[0].hypot(sai[1]));
            if le <= 1e-12 || ls <= 1e-12 {
                return None;
            }
            let cruz = ent[0].mul_add(sai[1], -(ent[1] * sai[0]));
            let esc = ent[0].mul_add(sai[0], ent[1] * sai[1]);
            Some(cruz.atan2(esc).abs().to_degrees())
        })
        .collect()
}

/// A MELHOR cúbica possível para um segmento — pontas presas no padrão-ouro, as duas alças livres,
/// mínimos quadrados sobre `amostras` pontos. ⭐ **É o CHÃO do modelo**: nenhum ajuste de alças pode
/// fazer melhor do que isto com esta contagem de nós.
fn b_melhor_cubica(ouro: &[[f64; 2]], ts: &[f64]) -> [[f64; 2]; 4] {
    let (p0, p3) = (ouro[0], ouro[ouro.len() - 1]);
    let (mut a11, mut a12, mut a22) = (0.0_f64, 0.0_f64, 0.0_f64);
    let (mut b1, mut b2) = ([0.0_f64; 2], [0.0_f64; 2]);
    for (i, &t) in ts.iter().enumerate() {
        let u = 1.0 - t;
        let (w0, w1, w2, w3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
        let d = [
            w3.mul_add(-p3[0], w0.mul_add(-p0[0], ouro[i][0])),
            w3.mul_add(-p3[1], w0.mul_add(-p0[1], ouro[i][1])),
        ];
        a11 = w1.mul_add(w1, a11);
        a12 = w1.mul_add(w2, a12);
        a22 = w2.mul_add(w2, a22);
        b1 = [w1.mul_add(d[0], b1[0]), w1.mul_add(d[1], b1[1])];
        b2 = [w2.mul_add(d[0], b2[0]), w2.mul_add(d[1], b2[1])];
    }
    let det = a12.mul_add(-a12, a11 * a22);
    let p1 = [
        (b1[0] * a22 - b2[0] * a12) / det,
        (b1[1] * a22 - b2[1] * a12) / det,
    ];
    let p2 = [
        (b2[0] * a11 - b1[0] * a12) / det,
        (b2[1] * a11 - b1[1] * a12) / det,
    ];
    [p0, p1, p2, p3]
}

/// A escala a que o VINCO é medido — `5 %` da espessura da barra.
///
/// ⚠️ **Ela é a régua, e tem de ser dita:** a curvatura de um canto depende da escala a que se
/// olha. A esta, uma quina de `180°` satura em `κ = 2/h = 40`, a tampa da cápsula (raio `0,5`) lê
/// `2,0` e uma aresta recta lê `0`.
const B_H: f64 = 0.05;

/// O comprimento de arco acumulado de uma polilinha FECHADA (`len + 1` casas; a última é o
/// perímetro).
fn b_cum(poli: &[[f64; 2]]) -> Vec<f64> {
    let n = poli.len();
    let mut c = Vec::with_capacity(n + 1);
    c.push(0.0);
    for i in 0..n {
        let (a, b) = (poli[i], poli[(i + 1) % n]);
        c.push(c[i] + (a[0] - b[0]).hypot(a[1] - b[1]));
    }
    c
}

/// O ponto da polilinha ao comprimento de arco `s` (com volta).
fn b_em(poli: &[[f64; 2]], cum: &[f64], s: f64) -> [f64; 2] {
    let per = cum[cum.len() - 1];
    if per <= 0.0 {
        return poli[0];
    }
    let s = s.rem_euclid(per);
    let (mut lo, mut hi) = (0usize, cum.len() - 1);
    while hi - lo > 1 {
        let m = (lo + hi) / 2;
        if cum[m] <= s { lo = m } else { hi = m }
    }
    let seg = cum[lo + 1] - cum[lo];
    let u = if seg > 0.0 { (s - cum[lo]) / seg } else { 0.0 };
    let n = poli.len();
    let (a, b) = (poli[lo % n], poli[(lo + 1) % n]);
    [u.mul_add(b[0] - a[0], a[0]), u.mul_add(b[1] - a[1], a[1])]
}

/// ⭐⭐⭐ **O VINCO — curvatura de MENGER à escala `h`**, uma por amostra, na MESMA ordem do
/// repouso.
///
/// ⛔⛔ **Ela substitui duas réguas que MENTIRAM nesta auditoria.** A curvatura tirada da
/// amostragem em `t` divide o ângulo por um comprimento que a própria deformação COMPRIME e leu
/// `7,07e15` sobre a cápsula (segmentos de comprimento zero) e `752` sobre uma aresta sã; a versão
/// que reamostra o arco e mede vizinhos consecutivos **ALIASA** — dois pontos dentro do mesmo
/// segmento da polilinha dão viragem `0` e dois que o atravessam dão a viragem inteira, o que
/// concentra o sinal e inflaciona o máximo (leu `124,75` onde esta lê muito menos).
///
/// ⭐ Esta pega os pontos a **`±h` de ARCO** e lê o círculo que passa pelos três: ela não conhece a
/// amostragem, e uma aresta recta lê `0` mesmo com amostras irregulares.
fn b_menger(poli: &[[f64; 2]], h: f64) -> Vec<f64> {
    let cum = b_cum(poli);
    let n = poli.len();
    (0..n)
        .map(|i| {
            let s = cum[i];
            let (a, b, c) = (b_em(poli, &cum, s - h), poli[i], b_em(poli, &cum, s + h));
            let ab = (a[0] - b[0]).hypot(a[1] - b[1]);
            let bc = (b[0] - c[0]).hypot(b[1] - c[1]);
            let ca = (c[0] - a[0]).hypot(c[1] - a[1]);
            if ab <= 0.0 || bc <= 0.0 || ca <= 0.0 {
                return 0.0;
            }
            let area2 = (b[0] - a[0])
                .mul_add(c[1] - a[1], -((c[0] - a[0]) * (b[1] - a[1])))
                .abs();
            2.0 * area2 / (ab * bc * ca)
        })
        .collect()
}

/// O vinco em GRAUS DE QUINA — o número que o artista lê. Satura em `180°`.
fn b_quina(k: f64, h: f64) -> f64 {
    2.0 * (k * h * 0.5).min(1.0).asin().to_degrees()
}

/// As amostras cujo REPOUSO está no troço RECTO da barra (`y = 2` ou `y = 3`) — ⛔ a tampa da
/// cápsula lê `κ = 2` por construção e afogaria o sinal.
fn b_rectas(rest: &[[f64; 2]]) -> Vec<usize> {
    (0..rest.len())
        .filter(|&i| (rest[i][1] - 2.0).abs() < 1e-9 || (rest[i][1] - 3.0).abs() < 1e-9)
        .collect()
}

/// O palco das sondas da lente B — a barra da cena do dono, montada UMA vez.
struct BPalco {
    sim: ph2d_ecs::SimWorld,
    scene: ph2d_vec_scene::VecScene,
    id: ph2d_vec_scene::VecPathId,
    alvo: ph2d_ecs::Entity,
    ossos: Vec<ph2d_ecs::Entity>,
    campo: ph2d_vec_skin::pesos::CampoDoDominio,
    correcoes: Vec<ph2d_skeleton::Correccao>,
    fonte: ph2d_vec_scene::VecPath,
}

fn b_palco(subdividir: bool) -> BPalco {
    use crate::barra_da_cena_tests_support::barra_da_cena_com;
    use ph2d_ecs::Entity;
    use ph2d_skeleton_ecs::SkinBind;
    let (sim, scene, map, id, ossos) = barra_da_cena_com(subdividir);
    let alvo = Entity::from_bits(map[&id]);
    let skin = sim.world().get::<SkinBind>(alvo).expect("pele").clone();
    let g = crate::skinned_mesh::le(&skin.source).expect("a fonte lê-se");
    BPalco {
        sim,
        scene,
        id,
        alvo,
        ossos,
        campo: g.campo.clone().expect("o bind desta wave guarda o campo"),
        correcoes: skin.correcoes_resolvidas(),
        fonte: g.path.clone(),
    }
}

impl BPalco {
    fn dobra(&mut self, graus: f32) {
        use ph2d_ecs::Transform;
        for o in &self.ossos[1..] {
            self.sim
                .world_mut()
                .get_mut::<Transform>(*o)
                .expect("Transform")
                .rotation = graus.to_radians();
        }
    }
    fn pele(&self) -> ph2d_skeleton::Skin {
        crate::skin_live::skin_of(&self.sim, self.alvo).expect("pele resolvida")
    }
    /// O PADRÃO-OURO sobre uma amostragem de repouso — a lei da mídia IMAGEM, ponto a ponto.
    fn ouro(&self, pele: &ph2d_skeleton::Skin, rest: &[[f64; 2]]) -> Vec<[f64; 2]> {
        rest.iter()
            .map(|&x| b_ouro_pt(pele, &self.campo, &self.correcoes, x).0)
            .collect()
    }
    /// A saída do PRODUTO, pela porta do produto.
    fn produto(&self, curva: bool, campo: bool) -> ph2d_vec_scene::VecPath {
        let mut sc = self.scene.clone();
        crate::skin_live::recook_com_mistura(&self.sim, &mut sc, curva, true, campo);
        sc.paths()
            .iter()
            .find(|p| p.id == self.id)
            .expect("path")
            .clone()
    }
}

/// ⭐⭐⭐ **SONDA B1 — A ESCADA DE DOBRAS na barra REAL da cena, contra o PADRÃO-OURO.**
///
/// A **tabela A** responde *«quão longe do padrão-ouro está o que o produto desenha?»*; a
/// **tabela B** responde a pergunta que decide a wave seguinte: *«e o padrão-ouro, ele PRÓPRIO é
/// bom?»* — área, largura do braço, vinco e auto-intersecção, com o REPOUSO como CONTROLO.
#[test]
fn diag_b_escada_de_dobras() {
    let mut p = b_palco(true);
    let rest = b_amostra(&p.fonte);
    let rest_anc = b_ancoras(&p.fonte);
    let area_rest = b_area(&rest);
    const EPS: f64 = 1e-7;

    let comp: Vec<f64> = {
        let n = rest.len();
        (0..n)
            .map(|i| {
                let (a, b) = (rest[i], rest[(i + 1) % n]);
                (a[0] - b[0]).hypot(a[1] - b[1])
            })
            .collect()
    };
    let n_zero = comp.iter().filter(|c| **c <= EPS).count();
    let vivo_min = comp
        .iter()
        .copied()
        .filter(|c| *c > EPS)
        .fold(f64::MAX, f64::min);
    let rectas = b_rectas(&rest);

    println!("\n{:=<128}", "");
    println!(
        "SONDA B1 · a escada de dobras na barra da cena, contra o PADRÃO-OURO (a lei da mídia IMAGEM)"
    );
    println!("{:=<128}", "");
    println!(
        "nós={} · amostras do contorno={} ({} no troço RECTO) · malha do campo={} vértices / {} tendões · ossos resolvidos={} · área de repouso={area_rest:.5}",
        p.fonte.verts_all().count(),
        rest.len(),
        rectas.len(),
        p.campo.malha.rest.len(),
        p.campo.ossos(),
        p.pele().len(),
    );
    println!(
        "segmentos de repouso DEGENERADOS (≤ {EPS:.0e}): {n_zero} de {} — a barra é uma CÁPSULA \
         (raio 0,5 = meia altura), logo as duas paredes dos topos têm comprimento ZERO; o menor \
         segmento VIVO mede {vivo_min:.6}.",
        rest.len()
    );
    let fora = rest.iter().filter(|q| p.campo.linha(**q).is_none()).count();
    #[expect(clippy::cast_precision_loss, reason = "contagens pequenas")]
    let pct_fora = fora as f64 / rest.len() as f64 * 100.0;
    println!(
        "amostras do contorno SEM triângulo no campo: {fora}/{} ({pct_fora:.2} %)",
        rest.len()
    );
    println!(
        "loadavg: {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );

    let mut a = Vec::new();
    let mut b = Vec::new();
    // O CONTROLO: o REPOUSO, pelas mesmas réguas.
    let kr: Vec<f64> = {
        let k = b_menger(&rest, B_H);
        rectas.iter().map(|&i| k[i]).collect()
    };
    let (_, kr90, krmax) = b_pct(&mut kr.clone());
    let xr = b_auto(&rest, EPS);

    for graus in [0.0_f32, 20.0, 45.0, 70.0, 90.0, 120.0, 150.0] {
        p.dobra(graus);
        let pele = p.pele();
        let ouro = p.ouro(&pele, &rest);
        let ouro_anc = p.ouro(&pele, &rest_anc);
        let prod_path = p.produto(true, true);
        let prod = b_amostra(&prod_path);
        let prod_anc = b_ancoras(&prod_path);

        let (d50, d90, dmax) = {
            let (x, y, z) = b_perfil(&prod, &ouro);
            let (x2, y2, z2) = b_perfil(&ouro, &prod);
            (x.max(x2), y.max(y2), z.max(z2))
        };
        let nomax = ouro_anc
            .iter()
            .zip(&prod_anc)
            .map(|(u, v)| (u[0] - v[0]).hypot(u[1] - v[1]))
            .fold(0.0_f64, f64::max);
        let (_, q90, qmax) = b_pct(&mut b_quebra_nos(&prod_path));
        let move_se = ouro
            .iter()
            .zip(&rest)
            .map(|(u, v)| (u[0] - v[0]).hypot(u[1] - v[1]))
            .fold(0.0_f64, f64::max);
        a.push((graus, d50, d90, dmax, nomax, q90, qmax, move_se));

        let ko_t = b_menger(&ouro, B_H);
        let kp_t = b_menger(&prod, B_H);
        let mut kos: Vec<f64> = rectas.iter().map(|&i| ko_t[i]).collect();
        let mut kps: Vec<f64> = rectas.iter().map(|&i| kp_t[i]).collect();
        let (_, ko90, komax) = b_pct(&mut kos);
        let (_, kp90, kpmax) = b_pct(&mut kps);
        let mut lo = b_larguras(&rest, &ouro);
        let mut lp = b_larguras(&rest, &prod);
        let lomin = lo.iter().copied().fold(f64::MAX, f64::min);
        let lpmin = lp.iter().copied().fold(f64::MAX, f64::min);
        let (lo50, ..) = b_pct(&mut lo);
        let (lp50, ..) = b_pct(&mut lp);
        b.push((
            graus,
            b_area(&ouro) / area_rest,
            b_area(&prod) / area_rest,
            lo50,
            lomin,
            lp50,
            lpmin,
            ko90,
            komax,
            kp90,
            kpmax,
            b_auto(&ouro, EPS),
            b_auto(&prod, EPS),
        ));
    }

    println!(
        "\n── TABELA A · FIDELIDADE AO PADRÃO-OURO ── a barra tem 1,0 u de ESPESSURA e 7,0 de comprimento {:─<30}",
        ""
    );
    println!(
        "{:>6} {:>10} {:>10} {:>10} {:>10} {:>13} {:>11} {:>11}",
        "graus",
        "desv p50",
        "desv p90",
        "desv max",
        "% da esp",
        "erro nos NÓS",
        "quebra p90",
        "quebra max"
    );
    for (g, x, y, z, no, q90, qmax, mv) in &a {
        println!(
            "{g:>6.0} {x:>10.5} {y:>10.5} {z:>10.5} {:>9.2} % {no:>13.2e} {q90:>10.4}° {qmax:>10.4}°   (a dobra move a arte {mv:.3})",
            z * 100.0
        );
    }

    println!(
        "\n── TABELA B · QUALIDADE INTRÍNSECA ── o PADRÃO-OURO é bom? (o REPOUSO é o CONTROLO) {:─<43}",
        ""
    );
    println!(
        "{:>6} | {:>8} {:>8} | {:>8} {:>8} | {:>8} {:>8} | {:>7} {:>7} {:>8} | {:>7} {:>7} {:>8} | {:>4} {:>4}",
        "graus",
        "áreaOURO",
        "áreaPRD",
        "largOURO",
        "min",
        "largPRD",
        "min",
        "κ OU90",
        "κ máx",
        "quina°",
        "κ PR90",
        "κ máx",
        "quina°",
        "Xou",
        "Xpr"
    );
    println!(
        "{:>6} | {:>8} {:>8} | {:>8} {:>8} | {:>8} {:>8} | {:>7.3} {:>7.3} {:>8.2} | {:>7} {:>7} {:>8} | {:>4} {:>4}",
        "REST",
        "100.00%",
        "—",
        "1.0000",
        "1.0000",
        "—",
        "—",
        kr90,
        krmax,
        b_quina(krmax, B_H),
        "—",
        "—",
        "—",
        xr,
        "—"
    );
    for (g, ao, ap, lo50, lomin, lp50, lpmin, ko90, komax, kp90, kpmax, xo, xp) in &b {
        println!(
            "{g:>6.0} | {:>7.2}% {:>7.2}% | {lo50:>8.4} {lomin:>8.4} | {lp50:>8.4} {lpmin:>8.4} | \
             {ko90:>7.3} {komax:>7.3} {:>8.2} | {kp90:>7.3} {kpmax:>7.3} {:>8.2} | {xo:>4} {xp:>4}",
            ao * 100.0,
            ap * 100.0,
            b_quina(*komax, B_H),
            b_quina(*kpmax, B_H)
        );
    }
    println!(
        "\nκ = curvatura de MENGER à escala h={B_H} (rad/u), SÓ nas {} amostras cujo repouso está no \
         troço RECTO — a tampa da cápsula (raio 0,5) leria 2,0 e ficou de FORA. `quina°` é o mesmo \
         κ máx lido como ÂNGULO DE QUINA, que satura em 180°. X = pares de segmentos do contorno \
         que se CRUZAM.",
        rectas.len()
    );
    println!("{:=<128}", "");
}

/// ⭐ **O CHÃO DO MODELO** — a melhor cúbica possível em CADA segmento desta fonte, com as pontas
/// presas no padrão-ouro. Devolve `(polilinha do chão, pior erro mesmo-`t`)`.
///
/// ⛔ Nenhum ajuste de alças pode fazer melhor do que isto com esta contagem de nós: o que sobra
/// abaixo desta linha é **modelo**, e o que está acima dela é **procedimento**.
fn b_chao(
    fonte: &ph2d_vec_scene::VecPath,
    pele: &ph2d_skeleton::Skin,
    campo: &ph2d_vec_skin::pesos::CampoDoDominio,
    correcoes: &[ph2d_skeleton::Correccao],
) -> (Vec<[f64; 2]>, f64, Vec<f64>) {
    #[expect(clippy::cast_precision_loss, reason = "i <= B_N")]
    let ts: Vec<f64> = (0..=B_N).map(|i| i as f64 / B_N as f64).collect();
    let cozido = fonte.cooked();
    let Some((v, _)) = cozido.contour(0) else {
        return (Vec::new(), 0.0, Vec::new());
    };
    let n = v.len();
    let mut poli = Vec::new();
    let mut pior = 0.0_f64;
    let mut por_seg = Vec::with_capacity(n);
    for k in 0..n {
        let c = b_cub(v, k);
        let ouro: Vec<[f64; 2]> = ts
            .iter()
            .map(|&t| b_ouro_pt(pele, campo, correcoes, b_eval(&c, t)).0)
            .collect();
        let best = b_melhor_cubica(&ouro, &ts);
        let mut e = 0.0_f64;
        for (i, &t) in ts.iter().enumerate() {
            let q = b_eval(&best, t);
            e = e.max((q[0] - ouro[i][0]).hypot(q[1] - ouro[i][1]));
        }
        por_seg.push(e);
        pior = pior.max(e);
        for &t in &ts[..B_N] {
            poli.push(b_eval(&best, t));
        }
    }
    (poli, pior, por_seg)
}

/// ⭐⭐⭐ **SONDA B2 — A ATRIBUIÇÃO.** Parte o erro em parcelas e diz a percentagem de cada uma.
///
/// Cada linha é uma ABLAÇÃO pela porta do produto, medida contra o MESMO padrão-ouro.
#[test]
fn diag_b_atribuicao() {
    let mut p = b_palco(true);
    let mut q = b_palco(false); // o mundo de ANTES de 2026-09-19 — oito nós
    let rest = b_amostra(&p.fonte);

    println!("\n{:=<122}", "");
    println!(
        "SONDA B2 · a ATRIBUIÇÃO — quanto do erro é MODELO (nós/cúbica) e quanto é PROCEDIMENTO (o ajuste)"
    );
    println!("{:=<122}", "");
    println!(
        "nós: BIND={} · sem subdivisão={} · o padrão-ouro é SEMPRE o mesmo (o campo do bind subdividido)",
        p.fonte.verts_all().count(),
        q.fonte.verts_all().count()
    );
    println!(
        "{:>6} {:<34} {:>10} {:>10} {:>10} {:>10}",
        "graus", "configuração", "desv p50", "desv p90", "desv max", "% da esp"
    );
    println!("{:-<122}", "");

    for graus in [45.0_f32, 90.0, 120.0, 150.0] {
        p.dobra(graus);
        q.dobra(graus);
        let pele = p.pele();
        let ouro: Vec<[f64; 2]> = rest
            .iter()
            .map(|&x| b_ouro_pt(&pele, &p.campo, &p.correcoes, x).0)
            .collect();

        let (chao34, mesmo_t34, por_seg) = b_chao(&p.fonte, &pele, &p.campo, &p.correcoes);
        let (chao8, mesmo_t8, _) = b_chao(&q.fonte, &pele, &p.campo, &p.correcoes);

        let linhas: Vec<(String, Vec<[f64; 2]>)> = vec![
            (
                "PRODUTO (34 nós · curva · campo)".into(),
                b_amostra(&p.produto(true, true)),
            ),
            (
                "  ablação: PH2D_SKIN_CAMPO=0".into(),
                b_amostra(&p.produto(true, false)),
            ),
            (
                "  ablação: PH2D_SKIN_CURVE=0 (ingénua)".into(),
                b_amostra(&p.produto(false, true)),
            ),
            (
                "  8 nós · curva · campo".into(),
                b_amostra(&q.produto(true, true)),
            ),
            (
                "  8 nós · ingénua (antes de 19/09)".into(),
                b_amostra(&q.produto(false, true)),
            ),
            ("CHÃO do modelo · 34 nós".into(), chao34),
            ("CHÃO do modelo ·  8 nós".into(), chao8),
        ];

        let mut prod_max = 0.0_f64;
        let mut chao_max = 0.0_f64;
        for (i, (nome, poli)) in linhas.iter().enumerate() {
            let (x, y, z) = b_perfil(poli, &ouro);
            let (x2, y2, z2) = b_perfil(&ouro, poli);
            let (d50, d90, dmax) = (x.max(x2), y.max(y2), z.max(z2));
            if i == 0 {
                prod_max = dmax;
            }
            if i == 5 {
                chao_max = dmax;
            }
            println!(
                "{:>6} {nome:<34} {d50:>10.5} {d90:>10.5} {dmax:>10.5} {:>9.2} %",
                if i == 0 {
                    format!("{graus:.0}")
                } else {
                    String::new()
                },
                dmax * 100.0
            );
        }
        let fit = (prod_max - chao_max).max(0.0);
        println!(
            "{:>6} {:<34} MODELO {:>5.1} %  ·  PROCEDIMENTO {:>5.1} %   (chão mesmo-t: 34 nós {mesmo_t34:.5} · 8 nós {mesmo_t8:.5})",
            "",
            "→ atribuição do PRODUTO:",
            chao_max / prod_max * 100.0,
            fit / prod_max * 100.0
        );
        // Os segmentos onde o MODELO não chega — é lá que faltam nós.
        let mut ord: Vec<(usize, f64)> = por_seg.iter().copied().enumerate().collect();
        ord.sort_by(|a, b| b.1.total_cmp(&a.1));
        let cozido = p.fonte.cooked();
        let (v, _) = cozido.contour(0).expect("contorno");
        let piores: Vec<String> = ord
            .iter()
            .take(3)
            .map(|(k, e)| {
                let a = v[*k].anchor;
                format!("#{k} em ({:.2},{:.2}) → {e:.5}", a[0], a[1])
            })
            .collect();
        println!(
            "{:>6} {:<34} {}",
            "",
            "  piores segmentos do CHÃO:",
            piores.join("  ·  ")
        );
        println!("{:-<122}", "");
    }
    println!(
        "loadavg: {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    println!("{:=<122}", "");
}

/// ⭐⭐⭐ **SONDA B3 — O MECANISMO.** Onde nasce o defeito, porquê, e qual é o TECTO.
#[test]
fn diag_b_onde_nasce() {
    let mut p = b_palco(true);
    let rest = b_amostra(&p.fonte);

    println!("\n{:=<126}", "");
    println!("SONDA B3 · o MECANISMO — onde nasce o vinco, e qual é o tecto");
    println!("{:=<126}", "");
    println!(
        "juntas dos ossos (repouso, x): {:?}",
        [-8.2_f64, -8.2 + 6.4 / 3.0, -8.2 + 2.0 * 6.4 / 3.0, -1.8]
            .map(|x| (x * 100.0).round() / 100.0)
    );

    // ── (1) O VINCO (Menger, troço RECTO) e a COERÊNCIA da média em círculo ────────────
    let rectas = b_rectas(&rest);
    println!(
        "\n── (1) VINCO por MENGER (h={B_H}, só no troço RECTO: {} de {} amostras) + a DEGENERESCÊNCIA da média em círculo {:─<8}",
        rectas.len(),
        rest.len(),
        ""
    );
    println!(
        "{:>6} | {:>7} {:>7} {:>7} {:>8} | {:>7} {:>7} {:>7} {:>8} | {:>8} {:>8} | {:>7} {:>7}",
        "graus",
        "κ OURO",
        "p90",
        "máx",
        "quina°",
        "κ PRD",
        "p90",
        "máx",
        "quina°",
        "coer p50",
        "coer MIN",
        "κ>2 ou",
        "κ>2 pr"
    );
    let mut piores = Vec::new();
    for graus in [20.0_f32, 45.0, 70.0, 90.0, 120.0, 150.0] {
        p.dobra(graus);
        let pele = p.pele();
        let ouro = p.ouro(&pele, &rest);
        let prod = b_amostra(&p.produto(true, true));
        let kot = b_menger(&ouro, B_H);
        let kpt = b_menger(&prod, B_H);
        let ko: Vec<f64> = rectas.iter().map(|&i| kot[i]).collect();
        let kp: Vec<f64> = rectas.iter().map(|&i| kpt[i]).collect();
        let (ko50, ko90, komax) = b_pct(&mut ko.clone());
        let (kp50, kp90, kpmax) = b_pct(&mut kp.clone());
        let no = ko.iter().filter(|x| **x > 2.0).count();
        let np = kp.iter().filter(|x| **x > 2.0).count();

        // A coerência |Σ w·(cos θ, sin θ)| — `1` = todos os ossos concordam, `0` = a média em
        // círculo DEGENERA e a lei cai na mistura linear (o candy-wrapper).
        let mut coer: Vec<f64> = Vec::with_capacity(rest.len());
        for &x in &rest {
            let mut w = pele.scratch();
            let linha = p
                .campo
                .linha(x)
                .unwrap_or_else(|| b_mais_proximo(&p.campo, x));
            pele.weights_corrected(x, Some(&linha), &mut w, &p.correcoes);
            let (mut sx, mut sy) = (0.0_f64, 0.0_f64);
            for (bn, &pw) in pele.bones().iter().zip(w.iter()) {
                let t = bn.angulo_da_pose();
                sx = pw.mul_add(t.cos(), sx);
                sy = pw.mul_add(t.sin(), sy);
            }
            coer.push(sx.hypot(sy));
        }
        let cmin = coer.iter().copied().fold(f64::MAX, f64::min);
        let (c50, ..) = b_pct(&mut coer.clone());

        println!(
            "{graus:>6.0} | {ko50:>7.3} {ko90:>7.3} {komax:>7.3} {:>8.2} | {kp50:>7.3} {kp90:>7.3} {kpmax:>7.3} {:>8.2} | {c50:>8.4} {cmin:>8.4} | {no:>7} {np:>7}",
            b_quina(komax, B_H),
            b_quina(kpmax, B_H)
        );

        let iko = ko
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .map_or(0, |(i, _)| i);
        let ikp = kp
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .map_or(0, |(i, _)| i);
        let ic = coer
            .iter()
            .enumerate()
            .min_by(|a, b| a.1.total_cmp(b.1))
            .map_or(0, |(i, _)| i);
        piores.push((
            graus,
            rest[rectas[iko]],
            komax,
            rest[rectas[ikp]],
            kpmax,
            rest[ic],
            cmin,
        ));
    }
    println!("\n  onde (posição de REPOUSO; as juntas estão em x = -6,07 e x = -3,93):");
    for (g, ro, k, rp, kpv, rc, c) in &piores {
        println!(
            "  {g:>5.0}°  pior vinco OURO κ={k:>7.3} em ({:>6.2},{:>5.2})  ·  pior vinco PRODUTO κ={kpv:>7.3} em ({:>6.2},{:>5.2})  ·  pior coerência {c:.4} em ({:>6.2},{:>5.2})",
            ro[0], ro[1], rp[0], rp[1], rc[0], rc[1]
        );
    }

    // ── (2) O TECTO: o CHÃO do modelo em função da contagem de nós ──────────────────────
    println!(
        "\n── (2) O TECTO — o CHÃO do modelo se cada segmento for partido em M {:─<50}",
        ""
    );
    println!(
        "{:>6} | {:>12} {:>12} {:>12} {:>12} {:>12}",
        "graus", "M=1 (34 nós)", "M=2 (68)", "M=4 (136)", "M=8 (272)", "PRODUTO hoje"
    );
    #[expect(clippy::cast_precision_loss, reason = "índices pequenos")]
    for graus in [45.0_f32, 90.0, 120.0, 150.0] {
        p.dobra(graus);
        let pele = p.pele();
        let cozido = p.fonte.cooked();
        let (v, _) = cozido.contour(0).expect("contorno");
        let n = v.len();
        let mut col = Vec::new();
        for m in [1usize, 2, 4, 8] {
            let mut pior = 0.0_f64;
            for k in 0..n {
                let c = b_cub(v, k);
                for j in 0..m {
                    let (t0, t1) = (j as f64 / m as f64, (j + 1) as f64 / m as f64);
                    let ts: Vec<f64> = (0..=B_N)
                        .map(|i| (i as f64 / B_N as f64).mul_add(t1 - t0, t0))
                        .collect();
                    let ouro: Vec<[f64; 2]> = ts
                        .iter()
                        .map(|&t| b_ouro_pt(&pele, &p.campo, &p.correcoes, b_eval(&c, t)).0)
                        .collect();
                    // Reparametrizado em [0,1] no sub-intervalo.
                    let loc: Vec<f64> = (0..=B_N).map(|i| i as f64 / B_N as f64).collect();
                    let best = b_melhor_cubica(&ouro, &loc);
                    for (i, &t) in loc.iter().enumerate() {
                        let q = b_eval(&best, t);
                        pior = pior.max((q[0] - ouro[i][0]).hypot(q[1] - ouro[i][1]));
                    }
                }
            }
            col.push(pior);
        }
        let ouro_full = p.ouro(&pele, &rest);
        let prod = b_amostra(&p.produto(true, true));
        let (_, _, dprod) = b_perfil(&prod, &ouro_full);
        println!(
            "{graus:>6.0} | {:>12.5} {:>12.5} {:>12.5} {:>12.5} {:>12.5}",
            col[0], col[1], col[2], col[3], dprod
        );
    }

    // ── (3) A MÍDIA IMAGEM na MESMA cena — a malha densa, triângulo a triângulo ─────────
    println!(
        "\n── (3) A MÍDIA IMAGEM na mesma cena — distorção de área POR TRIÂNGULO {:─<48}",
        ""
    );
    println!(
        "{:>6} | {:>10} {:>10} {:>10} {:>10} {:>10}",
        "graus", "área total", "p50 tri", "p10 tri", "min tri", "invertidos"
    );
    let malha = p.campo.malha.clone();
    let locais: Vec<[f64; 2]> = (0..malha.rest.len())
        .map(|i| p.campo.local_do_vertice(i).expect("régua"))
        .collect();
    let tri_area = |a: [f64; 2], b: [f64; 2], c: [f64; 2]| {
        ((b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1])) * 0.5
    };
    let rest_tri: Vec<f64> = malha
        .tris
        .iter()
        .map(|t| {
            tri_area(
                locais[t[0] as usize],
                locais[t[1] as usize],
                locais[t[2] as usize],
            )
        })
        .collect();
    let soma_rest: f64 = rest_tri.iter().map(|x| x.abs()).sum();
    for graus in [45.0_f32, 90.0, 120.0, 150.0] {
        p.dobra(graus);
        let pele = p.pele();
        let pos: Vec<[f64; 2]> = (0..malha.rest.len())
            .map(|i| {
                let q = locais[i];
                let mut w = pele.scratch();
                let linha = p.campo.linha_do_vertice(i).map(<[f64]>::to_vec);
                pele.weights_corrected(q, linha.as_deref(), &mut w, &p.correcoes);
                pele.blend(q, &w)
            })
            .collect();
        let mut razoes = Vec::new();
        let mut invertidos = 0usize;
        let mut soma = 0.0_f64;
        for (t, r0) in malha.tris.iter().zip(&rest_tri) {
            let a = tri_area(pos[t[0] as usize], pos[t[1] as usize], pos[t[2] as usize]);
            soma += a.abs();
            if r0.abs() > 1e-12 {
                razoes.push(a / *r0);
                if (a / *r0) < 0.0 {
                    invertidos += 1;
                }
            }
        }
        razoes.sort_by(f64::total_cmp);
        #[expect(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "percentil"
        )]
        let q = |f: f64| razoes[(((razoes.len() - 1) as f64) * f).round() as usize];
        println!(
            "{graus:>6.0} | {:>9.2} % {:>10.4} {:>10.4} {:>10.4} {:>10}",
            soma / soma_rest * 100.0,
            q(0.5),
            q(0.1),
            razoes[0],
            invertidos
        );
    }
    println!(
        "\nloadavg: {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    println!("{:=<126}", "");
}

/// O ÂNGULO MÉDIO `θ̄` que a [`ph2d_skeleton::Skin::blend`] aplica num ponto, e a COERÊNCIA da
/// média em círculo.
fn b_theta(
    pele: &ph2d_skeleton::Skin,
    campo: &ph2d_vec_skin::pesos::CampoDoDominio,
    correcoes: &[ph2d_skeleton::Correccao],
    p: [f64; 2],
) -> (f64, f64) {
    let mut w = pele.scratch();
    let linha = campo.linha(p).unwrap_or_else(|| b_mais_proximo(campo, p));
    pele.weights_corrected(p, Some(&linha), &mut w, correcoes);
    let (mut sx, mut sy) = (0.0_f64, 0.0_f64);
    for (bn, &pw) in pele.bones().iter().zip(w.iter()) {
        let t = bn.angulo_da_pose();
        sx = pw.mul_add(t.cos(), sx);
        sy = pw.mul_add(t.sin(), sy);
    }
    (sy.atan2(sx), sx.hypot(sy))
}

/// ⭐⭐⭐ **SONDA B4 — A LEI DO VINCO.** A previsão geométrica, medida.
///
/// # A previsão
///
/// A [`ph2d_skeleton::Skin::blend`] roda **em torno da JUNTA** (`c`) e translada pela mistura
/// linear DELA. Num par pai→filho os dois ossos partilham a junta, logo `M₁(c) = M₂(c)` e a
/// translação é a MESMA para todo peso: perto da junta a lei é *«rodar em torno de `J` por um
/// ângulo `θ̄(p)` que varia com o peso»*.
///
/// Derivando ao longo do contorno (parâmetro de arco `s`), a tangente da imagem é
/// `R(θ̄)·[t̂(s) + θ̄′(s)·perp(p − c)]`. Na aresta INTERIOR do cotovelo `perp(p − c)` aponta **contra**
/// a marcha e tem módulo `r` (a meia-espessura) ⇒ **o esticão é `|1 − θ̄′·r|`, e ele chega a ZERO
/// quando `θ̄′ = 1/r`.** Num ponto de esticão zero a curva tem uma CÚSPIDE; acima dele, ela dobra
/// sobre si mesma e o contorno passa a CRUZAR-SE.
#[test]
fn diag_b_a_lei_do_vinco() {
    let mut p = b_palco(true);
    let rest = b_amostra(&p.fonte);
    let n = rest.len();
    // O arco de repouso acumulado — o `s` da derivada.
    let cum = b_cum(&rest);
    const R: f64 = 0.5; // meia-espessura da barra

    println!("\n{:=<126}", "");
    println!("SONDA B4 · A LEI DO VINCO — a cúspide prevista, medida");
    println!("{:=<126}", "");
    println!(
        "previsão: esticão = |1 − θ̄′·r| com r = {R} ⇒ CÚSPIDE quando θ̄′ = {:.3} rad/u",
        1.0 / R
    );
    println!(
        "{:>6} | {:>9} {:>9} | {:>9} {:>9} | {:>10} {:>10} | {:>8} {:>6} | {:>5}",
        "graus",
        "estic p50",
        "estic MIN",
        "θ̄′ máx",
        "θ̄′·r máx",
        "prev. estic",
        "medido",
        "κ máx",
        "quina°",
        "X"
    );
    for graus in [
        20.0_f32, 45.0, 60.0, 70.0, 80.0, 85.0, 90.0, 95.0, 100.0, 110.0, 120.0, 150.0,
    ] {
        p.dobra(graus);
        let pele = p.pele();
        let ouro = p.ouro(&pele, &rest);
        let thetas: Vec<f64> = rest
            .iter()
            .map(|&x| b_theta(&pele, &p.campo, &p.correcoes, x).0)
            .collect();

        let mut estica = Vec::new();
        let mut dtheta = Vec::new();
        let mut min_e = (f64::MAX, 0usize);
        let mut max_d = (0.0_f64, 0usize);
        for i in 0..n {
            let j = (i + 1) % n;
            let dr = cum[i + 1] - cum[i];
            if dr <= 1e-9 {
                continue;
            }
            let dd = (ouro[i][0] - ouro[j][0]).hypot(ouro[i][1] - ouro[j][1]);
            let e = dd / dr;
            estica.push(e);
            if e < min_e.0 {
                min_e = (e, i);
            }
            // A derivada do ângulo médio ao longo do arco de REPOUSO — com a volta em ±π tratada.
            let mut dt = thetas[j] - thetas[i];
            while dt > std::f64::consts::PI {
                dt -= std::f64::consts::TAU;
            }
            while dt < -std::f64::consts::PI {
                dt += std::f64::consts::TAU;
            }
            let d = (dt / dr).abs();
            dtheta.push(d);
            if d > max_d.0 {
                max_d = (d, i);
            }
        }
        let (e50, ..) = b_pct(&mut estica.clone());
        let kt = b_menger(&ouro, B_H);
        let rectas = b_rectas(&rest);
        let (_, _, kmax) = b_pct(&mut rectas.iter().map(|&i| kt[i]).collect::<Vec<_>>());
        println!(
            "{graus:>6.0} | {e50:>9.4} {:>9.4} | {:>9.4} {:>9.4} | {:>10.4} {:>10.4} | {kmax:>8.3} {:>6.1} | {:>5}",
            min_e.0,
            max_d.0,
            max_d.0 * R,
            (1.0 - max_d.0 * R).abs(),
            min_e.0,
            b_quina(kmax, B_H),
            b_auto(&ouro, 1e-7)
        );
    }
    println!(
        "\n  (a coluna `prev. esticão` é |1 − θ̄′·r| com o θ̄′ MÁXIMO; a coluna `medido` é o esticão \
         MÍNIMO do contorno. Elas concordam ⇒ a lei está identificada.)"
    );

    // ── ONDE, e o retrato local ────────────────────────────────────────────────────────
    println!(
        "\n── O RETRATO LOCAL a 90° — a aresta INTERIOR do cotovelo 2 (junta em x = -3.93) {:─<45}",
        ""
    );
    p.dobra(90.0);
    let pele = p.pele();
    let ouro = p.ouro(&pele, &rest);
    let mut alvo: Vec<(f64, usize)> = (0..n)
        .filter(|&i| (rest[i][1] - 3.0).abs() < 1e-9 && (rest[i][0] + 3.93).abs() < 0.7)
        .map(|i| (rest[i][0], i))
        .collect();
    alvo.sort_by(|a, b| a.0.total_cmp(&b.0));
    println!(
        "{:>9} {:>10} {:>10} {:>10} {:>10} {:>10}",
        "rest x", "def x", "def y", "θ̄ (graus)", "coer", "passo"
    );
    let mut ant: Option<[f64; 2]> = None;
    for (x, i) in alvo.iter().step_by(2) {
        let (t, c) = b_theta(&pele, &p.campo, &p.correcoes, rest[*i]);
        let passo = ant.map_or(0.0, |a| (a[0] - ouro[*i][0]).hypot(a[1] - ouro[*i][1]));
        println!(
            "{x:>9.3} {:>10.4} {:>10.4} {:>10.2} {c:>10.4} {passo:>10.5}",
            ouro[*i][0],
            ouro[*i][1],
            t.to_degrees()
        );
        ant = Some(ouro[*i]);
    }
    // Qual aresta é a de DENTRO?
    for (rot, nome) in [(3.0_f64, "y = 3 (topo)"), (2.0, "y = 2 (base)")] {
        let idx: Vec<usize> = (0..n)
            .filter(|&i| (rest[i][1] - rot).abs() < 1e-9)
            .collect();
        let mut e: Vec<f64> = idx
            .iter()
            .filter_map(|&i| {
                let j = (i + 1) % n;
                let dr = cum[i + 1] - cum[i];
                (dr > 1e-9).then(|| (ouro[i][0] - ouro[j][0]).hypot(ouro[i][1] - ouro[j][1]) / dr)
            })
            .collect();
        let mn = e.iter().copied().fold(f64::MAX, f64::min);
        let (e50, ..) = b_pct(&mut e);
        println!(
            "  {nome}: esticão p50 = {e50:.4} · MIN = {mn:.4}  ⇒  {}",
            if mn < 0.5 {
                "é a de DENTRO (comprime)"
            } else {
                "é a de FORA (estica)"
            }
        );
    }
    println!(
        "\nloadavg: {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    println!("{:=<126}", "");
}

/// Os ângulos das poses DESDOBRADOS ao longo da cadeia — cada um escolhido na volta mais próxima
/// do anterior.
///
/// ⚠️ **`angulo_da_pose` é um `atan2` e vive em `(−π, π]`**: numa cadeia que dobra `90°` por junta,
/// o terceiro osso lê `±180°` e o sinal é um sorteio de último bit.
fn b_desdobra(pele: &ph2d_skeleton::Skin) -> Vec<f64> {
    let mut out: Vec<f64> = Vec::with_capacity(pele.len());
    let mut ant = 0.0_f64;
    for b in pele.bones() {
        let mut t = b.angulo_da_pose();
        while t - ant > std::f64::consts::PI {
            t -= std::f64::consts::TAU;
        }
        while t - ant < -std::f64::consts::PI {
            t += std::f64::consts::TAU;
        }
        out.push(t);
        ant = t;
    }
    out
}

/// A MESMA mistura da [`ph2d_skeleton::Skin::blend`] — mesmo centro, mesma translação — só que o
/// ângulo é a **média LINEAR dos ângulos DESDOBRADOS** em vez da média em círculo.
fn b_blend_ang_linear(
    pele: &ph2d_skeleton::Skin,
    p: [f64; 2],
    w: &[f64],
    desdobrados: &[f64],
) -> [f64; 2] {
    let Some(c) = pele.centro_de_rotacao(w) else {
        return pele.blend_linear(p, w);
    };
    let (mut ang, mut soma) = (0.0_f64, 0.0_f64);
    for (i, &pw) in w.iter().enumerate() {
        if pw == 0.0 {
            continue;
        }
        ang = pw.mul_add(desdobrados[i], ang);
        soma += pw;
    }
    if soma == 0.0 {
        return pele.blend_linear(p, w);
    }
    let (si, co) = (ang / soma).sin_cos();
    let base = pele.blend_linear(c, w);
    let d = [p[0] - c[0], p[1] - c[1]];
    [
        si.mul_add(-d[1], co.mul_add(d[0], base[0])),
        si.mul_add(d[0], co.mul_add(d[1], base[1])),
    ]
}

/// ⭐⭐⭐ **SONDA B5 — O TECTO: as TRÊS leis de mistura, medidas lado a lado.**
///
/// `LINEAR` = a mistura de posições (`blend_linear`, o *candy-wrapper* que a [`ph2d_skeleton`]
/// guarda como CONTROLO) · `CÍRCULO` = o que o produto ship hoje ([`ph2d_skeleton::Skin::blend`],
/// média em círculo) · `ÂNGULO` = a mesma lei com o ângulo médio LINEAR sobre ângulos desdobrados.
#[test]
fn diag_b_o_tecto_das_tres_leis() {
    let mut p = b_palco(true);
    let rest = b_amostra(&p.fonte);
    let n = rest.len();
    let cum = b_cum(&rest);
    let rectas = b_rectas(&rest);
    let area_rest = b_area(&rest);
    const R: f64 = 0.5;

    println!("\n{:=<128}", "");
    println!("SONDA B5 · O TECTO — as três leis de mistura na MESMA barra, com os MESMOS pesos");
    println!("{:=<128}", "");
    println!(
        "{:>6} {:<9} | {:>9} {:>9} | {:>9} | {:>8} {:>7} | {:>8} {:>8} | {:>4}",
        "graus",
        "lei",
        "estic p50",
        "estic MIN",
        "θ̄′·r máx",
        "κ máx",
        "quina°",
        "área %",
        "larg min",
        "X"
    );
    for graus in [45.0_f32, 70.0, 90.0, 110.0, 120.0, 150.0] {
        p.dobra(graus);
        let pele = p.pele();
        let desd = b_desdobra(&pele);
        if (graus - 90.0).abs() < 0.5 {
            println!(
                "         (a 90° os ângulos CRUS das poses são {:?} e os DESDOBRADOS {:?})",
                pele.bones()
                    .iter()
                    .map(|b| (b.angulo_da_pose().to_degrees() * 10.0).round() / 10.0)
                    .collect::<Vec<_>>(),
                desd.iter()
                    .map(|t| (t.to_degrees() * 10.0).round() / 10.0)
                    .collect::<Vec<_>>()
            );
        }
        for (nome, modo) in [("LINEAR", 0u8), ("CÍRCULO", 1), ("ÂNGULO", 2)] {
            let mut thetas = vec![0.0_f64; n];
            let saida: Vec<[f64; 2]> = rest
                .iter()
                .enumerate()
                .map(|(i, &x)| {
                    let mut w = pele.scratch();
                    let linha = p
                        .campo
                        .linha(x)
                        .unwrap_or_else(|| b_mais_proximo(&p.campo, x));
                    pele.weights_corrected(x, Some(&linha), &mut w, &p.correcoes);
                    let (mut sx, mut sy, mut soma) = (0.0_f64, 0.0_f64, 0.0_f64);
                    for (k, &pw) in w.iter().enumerate() {
                        sx = pw.mul_add(desd[k].cos(), sx);
                        sy = pw.mul_add(desd[k].sin(), sy);
                        soma = pw.mul_add(desd[k], soma);
                    }
                    thetas[i] = if modo == 2 { soma } else { sy.atan2(sx) };
                    match modo {
                        0 => pele.blend_linear(x, &w),
                        1 => pele.blend(x, &w),
                        _ => b_blend_ang_linear(&pele, x, &w, &desd),
                    }
                })
                .collect();
            let mut estica = Vec::new();
            let mut maxd = 0.0_f64;
            for i in 0..n {
                let j = (i + 1) % n;
                let dr = cum[i + 1] - cum[i];
                if dr <= 1e-9 {
                    continue;
                }
                estica.push((saida[i][0] - saida[j][0]).hypot(saida[i][1] - saida[j][1]) / dr);
                let mut dt = thetas[j] - thetas[i];
                while dt > std::f64::consts::PI {
                    dt -= std::f64::consts::TAU;
                }
                while dt < -std::f64::consts::PI {
                    dt += std::f64::consts::TAU;
                }
                maxd = maxd.max((dt / dr).abs());
            }
            let emin = estica.iter().copied().fold(f64::MAX, f64::min);
            let (e50, ..) = b_pct(&mut estica);
            let kt = b_menger(&saida, B_H);
            let (_, _, kmax) = b_pct(&mut rectas.iter().map(|&i| kt[i]).collect::<Vec<_>>());
            let mut l = b_larguras(&rest, &saida);
            let lmin = l.iter().copied().fold(f64::MAX, f64::min);
            l.clear();
            println!(
                "{:>6} {nome:<9} | {e50:>9.4} {emin:>9.4} | {:>9.4} | {kmax:>8.3} {:>7.1} | {:>8.2} {lmin:>8.4} | {:>4}",
                if modo == 0 {
                    format!("{graus:.0}")
                } else {
                    String::new()
                },
                maxd * R,
                b_quina(kmax, B_H),
                b_area(&saida) / area_rest * 100.0,
                b_auto(&saida, 1e-7)
            );
        }
        println!("{:-<128}", "");
    }
    println!(
        "loadavg: {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    println!("{:=<128}", "");
}

/// ⭐⭐ **SONDA B6 — ONDE A SUBDIVISÃO PÕE OS NÓS, e onde o erro MORA.**
///
/// A [`crate::subdivisao`] parte por um passo UNIFORME (o osso mais curto a dividir por `3`). Esta
/// sonda cruza o espaçamento dos nós com o CHÃO do modelo por segmento e com a distância à junta
/// mais próxima — é a régua da hipótese *«a subdivisão entrega nós a menos nos sítios errados»*.
#[test]
fn diag_b_onde_a_subdivisao_poe_os_nos() {
    let mut p = b_palco(true);
    let juntas = [-8.2_f64, -8.2 + 6.4 / 3.0, -8.2 + 2.0 * 6.4 / 3.0, -1.8];
    println!("\n{:=<110}", "");
    println!("SONDA B6 · onde a subdivisão põe os nós — e onde o erro mora");
    println!("{:=<110}", "");
    println!(
        "juntas em x = {:?}",
        juntas.map(|x| (x * 100.0).round() / 100.0)
    );

    let v: Vec<ph2d_vec_scene::VecVertex> = p
        .fonte
        .cooked()
        .contour(0)
        .map(|(x, _)| x.to_vec())
        .expect("contorno");
    let dist_junta = |x: f64| {
        juntas
            .iter()
            .map(|j| (j - x).abs())
            .fold(f64::MAX, f64::min)
    };

    for alvo_y in [3.0_f64, 2.0] {
        let mut xs: Vec<f64> = v
            .iter()
            .filter(|w| (w.anchor[1] - alvo_y).abs() < 1e-9)
            .map(|w| w.anchor[0])
            .collect();
        xs.sort_by(f64::total_cmp);
        let passos: Vec<f64> = xs.windows(2).map(|w| w[1] - w[0]).collect();
        let pmin = passos.iter().copied().fold(f64::MAX, f64::min);
        let pmax = passos.iter().copied().fold(0.0_f64, f64::max);
        println!(
            "\n  aresta y={alvo_y}: {} nós, passo min={pmin:.4} máx={pmax:.4} (razão {:.2}× ⇒ {})",
            xs.len(),
            pmax / pmin,
            if pmax / pmin < 1.5 {
                "UNIFORME"
            } else {
                "graduado"
            }
        );
        println!(
            "    x dos nós: {}",
            xs.iter()
                .map(|x| format!("{x:.2}"))
                .collect::<Vec<_>>()
                .join(" ")
        );
    }

    // O CHÃO por segmento contra a distância à junta.
    println!("\n  o CHÃO do modelo por segmento, contra a distância à junta mais próxima:");
    println!(
        "  {:>6} | {:>26} | {:>26}",
        "graus", "segs a ≤0,4 da junta", "segs a >0,4 da junta"
    );
    for graus in [70.0_f32, 90.0, 120.0, 150.0] {
        p.dobra(graus);
        let pele = p.pele();
        let (_, _, por_seg) = b_chao(&p.fonte, &pele, &p.campo, &p.correcoes);
        let (mut perto, mut longe) = (Vec::new(), Vec::new());
        for (k, e) in por_seg.iter().enumerate() {
            let a = v[k].anchor;
            let b = v[(k + 1) % v.len()].anchor;
            let d = dist_junta((a[0] + b[0]) * 0.5);
            if d <= 0.4 {
                perto.push(*e)
            } else {
                longe.push(*e)
            }
        }
        let (p50a, _, maxa) = b_pct(&mut perto.clone());
        let (p50b, _, maxb) = b_pct(&mut longe.clone());
        println!(
            "  {graus:>6.0} | n={:>3} p50={p50a:.5} máx={maxa:.5} | n={:>3} p50={p50b:.5} máx={maxb:.5}",
            perto.len(),
            longe.len()
        );
    }
    println!(
        "\nloadavg: {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    println!("{:=<110}", "");
}

/// ⭐⭐ **SONDA B7 — O QUE CADA WAVE COMPROU, dobra a dobra, contra o PADRÃO-OURO.**
///
/// ⛔ Ela existe porque o dono reportou *«não houve nenhuma melhora na deformação do vetor»* sobre
/// a wave do CAMPO (2026-09-20), e a sonda que a casa tinha media a diferença entre DUAS SAÍDAS
/// NOSSAS — nunca a distância ao padrão-ouro.
#[test]
fn diag_b_o_que_cada_wave_comprou() {
    let mut p = b_palco(true);
    let mut q = b_palco(false);
    let rest = b_amostra(&p.fonte);
    println!("\n{:=<118}", "");
    println!(
        "SONDA B7 · o que cada wave comprou — desvio MÁXIMO ao padrão-ouro, em % da ESPESSURA da barra"
    );
    println!("{:=<118}", "");
    println!(
        "{:>6} | {:>12} {:>12} {:>12} | {:>10} {:>10} {:>10}",
        "graus", "8nós ingénua", "8nós curva", "34nós curva", "+SUBDIV", "+CURVA", "+CAMPO"
    );
    println!(
        "{:>6} | {:>12} {:>12} {:>12} | {:>10} {:>10} {:>10}",
        "", "(pré-19/09)", "", "sem campo", "×", "×", "×"
    );
    for graus in [
        20.0_f32, 45.0, 70.0, 90.0, 110.0, 120.0, 130.0, 140.0, 150.0, 160.0,
    ] {
        p.dobra(graus);
        q.dobra(graus);
        let pele = p.pele();
        let ouro = p.ouro(&pele, &rest);
        let d = |path: &ph2d_vec_scene::VecPath| -> f64 {
            let poli = b_amostra(path);
            let (_, _, a) = b_perfil(&poli, &ouro);
            let (_, _, b) = b_perfil(&ouro, &poli);
            a.max(b)
        };
        let v8n = d(&q.produto(false, true));
        let v8c = d(&q.produto(true, true));
        let v34s = d(&p.produto(true, false));
        let v34 = d(&p.produto(true, true));
        println!(
            "{graus:>6.0} | {:>11.2}% {:>11.2}% {:>11.2}% | {:>9.1}× {:>9.1}× {:>9.2}×",
            v8n * 100.0,
            v8c * 100.0,
            v34s * 100.0,
            v8c / v34s.max(1e-12),
            v8n / v8c.max(1e-12),
            v34s / v34.max(1e-12)
        );
    }
    println!(
        "\n  a coluna `+SUBDIV` é (8 nós curva) ÷ (34 nós curva sem campo); `+CURVA` é (8 ingénua) ÷ \
         (8 curva); `+CAMPO` é (34 sem campo) ÷ (34 com campo). Um número ABAIXO de 1,0 quer dizer \
         que a wave PIOROU o desenho."
    );
    println!("\n  PRODUTO de hoje (34 nós · curva · campo), em % da espessura:");
    for graus in [
        20.0_f32, 45.0, 70.0, 90.0, 110.0, 120.0, 130.0, 140.0, 150.0, 160.0,
    ] {
        p.dobra(graus);
        let pele = p.pele();
        let ouro = p.ouro(&pele, &rest);
        let poli = b_amostra(&p.produto(true, true));
        let (_, _, a) = b_perfil(&poli, &ouro);
        let (_, _, b) = b_perfil(&ouro, &poli);
        let sc = b_amostra(&p.produto(true, false));
        let (_, _, c) = b_perfil(&sc, &ouro);
        let (_, _, e) = b_perfil(&ouro, &sc);
        println!(
            "    {graus:>5.0}°  com campo {:>7.2}%   ·   sem campo {:>7.2}%   ⇒  {}",
            a.max(b) * 100.0,
            c.max(e) * 100.0,
            if a.max(b) <= c.max(e) {
                "o campo AJUDA"
            } else {
                "o campo PIORA"
            }
        );
    }
    println!(
        "\nloadavg: {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    println!("{:=<118}", "");
}
