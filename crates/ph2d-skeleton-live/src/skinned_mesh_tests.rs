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
    let pele = |dobra: f64| {
        let osso = |x0: f64, rot: f64| {
            let (c, s) = (rot.cos(), rot.sin());
            SkinBone::new(
                Xform([1.0, 0.0, 0.0, 1.0, x0, H * 0.5]),
                W * 0.5,
                1.0,
                Xform([c, s, -s, c, x0, H * 0.5]),
                Xform::IDENTITY,
            )
            .expect("repouso")
        };
        Skin::new(vec![osso(0.0, 0.0), osso(W * 0.5, dobra)]).expect("2 ossos")
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
