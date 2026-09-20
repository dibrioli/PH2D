//! Gates do atlas.
//!
//! ⚠️ **A fixtura é SINTÉTICA de propósito**, e ela contém o fenómeno por construção: três
//! cartas planas com translações CONHECIDAS entre elas, e um interruptor que FECHA a fita
//! num anel. *Uma fixtura vinda da cadeia mede a cadeia; estas leis são sobre o que se faz
//! DEPOIS dela* — e a corrida sobre uma peça de verdade vive no `atlas_probe`, que é onde
//! o controlo de produto mora.
//!
//! ⛔⛔ **Ela tem TRÊS cartas e não duas, e a razão é uma prova de mutação:** com duas, o
//! braço `(Some, None)` do assentamento e o braço `position(…)` da rotulagem **nunca
//! correm**, e duas mutações sobreviveram por não serem alcançadas — o que num relatório
//! se lê exactamente como *«o gate não vê o defeito»*. *Uma fixtura pequena demais não
//! testa menos: ela deixa código inteiro fora do alcance da prova.*

use super::{Atlas, bases_dos_cantos, build};
use ph2d_gridmap::cut::SeamSide;
use ph2d_gridmap::{CutMesh, GridMap, Seam};
use ph2d_mesh::{Face, Mesh};

/// Uma fita de três cartas sobre um prisma triangular.
///
/// ```text
///   0---1---2---0      patch 0 = {0,3,4,1}   patch 1 = {1,4,5,2}   patch 2 = {2,5,3,0}
///   | \ | \ | \ |      costuras em (1,4) · (2,5) · e (0,3) SE a fita fechar
///   3---4---5---3
/// ```
///
/// ⭐ `anel = true` fecha a fita, e o que isso produz é **holonomia de verdade**: dar a
/// volta acumula `3` células de translação, e nenhum plano recebe a fita inteira sem um
/// corte. *É o controlo positivo da régua da holonomia* — sem ele, uma régua que nunca
/// mede nada lê `0` e passa.
fn fita(salto: i32, anel: bool) -> (Mesh, CutMesh, GridMap, Vec<Option<i32>>) {
    let pos = vec![
        [0.0, 1.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.5, 1.0, 1.0],
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.5, 0.0, 1.0],
    ];
    let faces = vec![
        Face::tri(0, 3, 4),
        Face::tri(0, 4, 1),
        Face::tri(1, 4, 5),
        Face::tri(1, 5, 2),
        Face::tri(2, 5, 3),
        Face::tri(2, 3, 0),
    ];
    let mesh = Mesh::from_parts(pos, faces).expect("a fixtura e' valida");
    let lado = |a: u32, l: [u32; 2]| SeamSide {
        patch: a,
        local: vec![Some(l[0]), Some(l[1])],
    };
    let mut seams = vec![
        Seam {
            arc: Some(0),
            chain: vec![1, 4],
            side: [lado(0, [3, 2]), lado(1, [0, 1])],
        },
        Seam {
            arc: Some(1),
            chain: vec![2, 5],
            side: [lado(1, [3, 2]), lado(2, [0, 1])],
        },
    ];
    let mut jumps = vec![Some(salto), Some(0)];
    if anel {
        seams.push(Seam {
            arc: Some(2),
            chain: vec![0, 3],
            side: [lado(2, [3, 2]), lado(0, [0, 1])],
        });
        jumps.push(Some(0));
    }
    let cut = CutMesh {
        origin: vec![vec![0, 3, 4, 1], vec![1, 4, 5, 2], vec![2, 5, 3, 0]],
        tris: vec![
            vec![[0, 1, 2], [0, 2, 3]],
            vec![[0, 1, 2], [0, 2, 3]],
            vec![[0, 1, 2], [0, 2, 3]],
        ],
        tri_face: vec![vec![0, 1], vec![2, 3], vec![4, 5]],
        seams,
    };
    // ⛔⛔ **CADA CARTA NASCE NUMA ORIGEM PRÓPRIA, e isso não é decoração.** A 1.ª
    // redacção punha-as contíguas (`x`, `x+1`, `x+2`), o que dá translação de costura
    // **ZERO** — e duas mutações que trocavam o SINAL do assentamento sobreviveram, porque
    // `+0` e `−0` são a mesma coisa. *Um corpus no ponto NEUTRO de um parâmetro não testa
    // esse parâmetro*, a lei que o L-System desta casa já pagou.
    let carta = |x: f32, o: [f32; 2]| {
        vec![
            [x + o[0], o[1]],
            [x + o[0], 1.0 + o[1]],
            [x + 1.0 + o[0], 1.0 + o[1]],
            [x + 1.0 + o[0], o[1]],
        ]
    };
    let map = GridMap {
        uv: vec![
            carta(0.0, [0.0, 0.0]),
            carta(1.0, [10.0, 3.0]),
            carta(2.0, [20.0, -7.0]),
        ],
        shift: vec![[0.0, 0.0]; 3],
    };
    (mesh, cut, map, jumps)
}

fn corrida(salto: i32, anel: bool) -> (Mesh, Atlas) {
    let (mesh, cut, map, jumps) = fita(salto, anel);
    let atlas = build(&mesh, &cut, &map, &jumps);
    (mesh, atlas)
}

/// ⭐⭐⭐ **A LEI DA WAVE:** uma costura que não roda NÃO é um corte, e as cartas que ela
/// liga saem numa ilha só.
#[test]
fn uma_costura_que_nao_roda_junta_as_cartas_numa_ilha() {
    let (_, a) = corrida(0, false);
    assert_eq!(a.relatorio.ilhas, 1, "tres cartas coladas sao UMA ilha");
    assert_eq!(a.relatorio.coladas, 2);
    assert_eq!(a.relatorio.rodadas, 0);
    // ⛔ O CONTROLO, e sem ele a asserção de cima passa com um `build` que junta tudo:
    // a MESMA fixtura com a primeira transição a rodar tem de dar DUAS.
    let (_, b) = corrida(1, false);
    assert_eq!(b.relatorio.ilhas, 2, "uma costura que roda e' um corte");
    assert_eq!(b.relatorio.rodadas, 1);
    assert_eq!(b.relatorio.coladas, 1);
}

/// ⭐⭐ **O assentamento fecha** numa fita aberta: os dois lados de cada costura colada
/// caem no mesmo ponto do plano da ilha.
#[test]
fn as_cartas_assentam_no_mesmo_plano_sem_holonomia() {
    let (_, a) = corrida(0, false);
    assert!(
        a.relatorio.cola_max < 1.0e-5,
        "a translacao de cada costura tem de ser constante: {}",
        a.relatorio.cola_max
    );
    assert!(
        a.relatorio.holonomia_max < 1.0e-5,
        "numa fita aberta o assentamento fecha: {}",
        a.relatorio.holonomia_max
    );
    // ⭐ O CONTROLO da coluna nova: uma fita ABERTA não obriga corte nenhum.
    assert_eq!(a.relatorio.ciclos, 0);
}

/// ⭐⭐⭐ **O CONTROLO POSITIVO DA HOLONOMIA: um ANEL não assenta, e o atlas DIZ.**
///
/// ⛔ Este gate nasceu de uma mutação SOBREVIVENTE: com a régua da holonomia a medir só as
/// costuras que ROdam, ela lia `0` na fita aberta e ninguém reparava. *Uma régua que nunca
/// vê o fenómeno acontecer não prova que ele não aconteceu.*
#[test]
fn um_anel_tem_holonomia_e_o_relatorio_acusa() {
    let (_, a) = corrida(0, true);
    assert_eq!(a.relatorio.ilhas, 1, "o anel e' uma ilha so'");
    assert_eq!(a.relatorio.coladas, 3);
    // ⭐ Três cartas em anel: a árvore usa duas costuras e a terceira FECHA o ciclo.
    assert_eq!(
        a.relatorio.ciclos, 1,
        "o anel tem de declarar o corte que ele proprio obrigou"
    );
    assert!(
        a.relatorio.holonomia_max > 1.0,
        "dar a volta ao anel acumula 3 celulas — o relatorio tem de as ver: {}",
        a.relatorio.holonomia_max
    );
}

/// ⭐⭐⭐ **O PISO DE POPULAÇÃO:** todo canto da malha recebe `(u, v)`.
///
/// ⛔ Sem ele um atlas vazio lê-se como um atlas perfeito — o defeito que a sonda do
/// doc 26 apanhou em si mesma na 1.ª corrida (`dobras 0/0`).
#[test]
fn todo_canto_recebe_uv_e_o_numero_e_o_da_malha() {
    let (mesh, a) = corrida(0, false);
    let (_, n) = bases_dos_cantos(&mesh);
    assert_eq!(n, 18, "seis triangulos sao dezoito cantos");
    assert_eq!(a.relatorio.cantos, n, "todo canto tem de ser posto");
    assert_eq!(a.relatorio.orfaos, 0);
    assert_eq!(a.uv.len(), n);
}

/// ⭐⭐ **O atlas cabe no quadrado**, que é o que um sampler pede.
#[test]
fn todo_uv_cai_dentro_do_quadrado_unitario() {
    for salto in [0, 1] {
        for anel in [false, true] {
            let (_, a) = corrida(salto, anel);
            for (c, z) in a.uv.iter().enumerate() {
                assert!(
                    (0.0..=1.0).contains(&z[0]) && (0.0..=1.0).contains(&z[1]),
                    "canto {c} caiu fora do quadrado: {z:?} (salto {salto}, anel {anel})"
                );
            }
        }
    }
}

/// ⭐⭐⭐ **DUAS ILHAS NÃO SE SOBREPÕEM** — se se sobrepusessem, dois sítios da peça
/// leriam o mesmo texel e o artista pintaria os dois de uma vez.
///
/// ⚠️ A régua é a caixa de cada ilha no atlas, reconstruída **do `uv` que saiu**, nunca
/// das caixas internas: *medir a arrumação pela variável que a produziu não a mede*.
#[test]
fn as_ilhas_nao_se_sobrepoem_no_atlas() {
    let (_, a) = corrida(1, false);
    assert_eq!(a.relatorio.ilhas, 2, "a fixtura tem de conter o fenomeno");
    let (lo, hi) = caixas(&a);
    for i in 0..a.relatorio.ilhas {
        for j in (i + 1)..a.relatorio.ilhas {
            let separadas = hi[i][0] <= lo[j][0]
                || hi[j][0] <= lo[i][0]
                || hi[i][1] <= lo[j][1]
                || hi[j][1] <= lo[i][1];
            assert!(
                separadas,
                "ilhas {i} e {j} sobrepoem-se: {:?}..{:?} contra {:?}..{:?}",
                lo[i], hi[i], lo[j], hi[j]
            );
        }
    }
}

/// A caixa de cada ilha, lida do `uv` entregue.
fn caixas(a: &Atlas) -> (Vec<[f32; 2]>, Vec<[f32; 2]>) {
    let mut lo = vec![[f32::MAX; 2]; a.relatorio.ilhas];
    let mut hi = vec![[f32::MIN; 2]; a.relatorio.ilhas];
    for (c, z) in a.uv.iter().enumerate() {
        let i = a.ilha[c] as usize;
        lo[i][0] = lo[i][0].min(z[0]);
        lo[i][1] = lo[i][1].min(z[1]);
        hi[i][0] = hi[i][0].max(z[0]);
        hi[i][1] = hi[i][1].max(z[1]);
    }
    (lo, hi)
}

/// ⭐⭐⭐ **TODA ILHA DECLARADA TEM CANTOS** — e nenhum canto fica sem ilha.
///
/// ⛔ Este gate nasceu de uma **mutação SOBREVIVENTE** (`ilha_de[p] = 0`): com todos os
/// patches rotulados com a mesma ilha, o relatório continuava a dizer `2`, o gate da
/// sobreposição comparava uma ilha cheia com uma VAZIA (e duas caixas vazias nunca se
/// sobrepõem) e tudo ficava verde. *Um rótulo errado e um rótulo certo leem-se iguais
/// enquanto ninguém contar a população de cada um.*
#[test]
fn cada_ilha_declarada_tem_cantos_seus() {
    for salto in [0, 1] {
        let (_, a) = corrida(salto, false);
        let mut quantos = vec![0usize; a.relatorio.ilhas];
        for &i in &a.ilha {
            let k = i as usize;
            assert!(
                k < a.relatorio.ilhas,
                "rotulo {k} fora das {} ilhas",
                a.relatorio.ilhas
            );
            quantos[k] += 1;
        }
        for (k, n) in quantos.iter().enumerate() {
            assert!(
                *n > 0,
                "a ilha {k} foi declarada e nao tem um canto (salto {salto})"
            );
        }
        assert_eq!(
            quantos.iter().sum::<usize>(),
            a.uv.len(),
            "todo canto tem de pertencer a uma ilha"
        );
    }
}

/// ⭐⭐⭐⭐ **UMA COSTURA COLADA NÃO É RASGADA PELO ATLAS** — os dois lados dela têm de
/// aterrar no MESMO texel, e uma que roda tem de aterrar em texels diferentes.
///
/// ⛔⛔ Este é o gate que faltava, e ele nasceu de **duas famílias de mutação
/// sobrevivente**: o sinal do assentamento e o rótulo da ilha. As duas produzem um atlas
/// em que cada ilha é internamente consistente — logo passam em toda régua de caixa — e
/// em que a costura que o motor colou aparece **partida na textura**. *Uma régua que só
/// olha para dentro de cada ilha nunca vê o que acontece ENTRE elas.*
#[test]
fn uma_costura_colada_nao_e_rasgada_pelo_atlas() {
    for (salto, rasga) in [(0, false), (1, true)] {
        let (mesh, cut, map, jumps) = fita(salto, false);
        let a = build(&mesh, &cut, &map, &jumps);
        let (base, _) = bases_dos_cantos(&mesh);
        // Por vértice global, os `(u, v)` que ele recebeu em todos os cantos.
        let mut vistos: Vec<Vec<[f32; 2]>> = vec![Vec::new(); mesh.positions().len()];
        for (f, face) in mesh.faces().iter().enumerate() {
            for (k, &g) in face.verts().iter().enumerate() {
                vistos[g as usize].push(a.uv[base[f] as usize + k]);
            }
        }
        let espalhamento = |g: u32| -> f32 {
            let v = &vistos[g as usize];
            let mut pior = 0.0f32;
            for x in v {
                for y in v {
                    let d = [x[0] - y[0], x[1] - y[1]];
                    pior = pior.max(d[0].mul_add(d[0], d[1] * d[1]).sqrt());
                }
            }
            pior
        };
        // A costura `0` é a que o `salto` controla; a `1` está sempre colada.
        let s0 = espalhamento(cut.seams[0].chain[0]).max(espalhamento(cut.seams[0].chain[1]));
        let s1 = espalhamento(cut.seams[1].chain[0]).max(espalhamento(cut.seams[1].chain[1]));
        assert!(
            s1 < 1.0e-6,
            "a costura colada foi RASGADA pelo atlas: {s1} (salto {salto})"
        );
        if rasga {
            assert!(
                s0 > 1.0e-3,
                "uma costura que RODA tem de aterrar em texels diferentes: {s0}"
            );
        } else {
            assert!(
                s0 < 1.0e-6,
                "a costura colada foi RASGADA pelo atlas: {s0} (salto {salto})"
            );
        }
    }
}

/// ⭐⭐⭐⭐ **NUM ANEL, O RASGO É UM SÓ, E VALE A HOLONOMIA.**
///
/// Um anel não assenta num plano, logo o atlas TEM de o partir algures — e a pergunta que
/// separa um atlas honesto de um estragado é *quantas vezes*. Uma só, e do tamanho que a
/// volta acumula.
///
/// ⛔⛔ Este gate nasceu da última mutação sobrevivente: o braço `(Some, None)` do
/// assentamento **só é alcançado quando a fita FECHA** (numa fita aberta a travessia corre
/// sempre no outro sentido), logo trocar-lhe o sinal passava despercebido. *Um braço que
/// nenhuma fixtura alcança não é código testado — é código que ninguém correu.*
#[test]
fn num_anel_o_rasgo_e_um_so_e_vale_a_holonomia() {
    let (mesh, cut, map, jumps) = fita(0, true);
    let a = build(&mesh, &cut, &map, &jumps);
    let (base, _) = bases_dos_cantos(&mesh);
    let mut vistos: Vec<Vec<[f32; 2]>> = vec![Vec::new(); mesh.positions().len()];
    for (f, face) in mesh.faces().iter().enumerate() {
        for (k, &g) in face.verts().iter().enumerate() {
            vistos[g as usize].push(a.uv[base[f] as usize + k]);
        }
    }
    let espalhamento = |g: u32| -> f32 {
        let v = &vistos[g as usize];
        let mut pior = 0.0f32;
        for x in v {
            for y in v {
                let d = [x[0] - y[0], x[1] - y[1]];
                pior = pior.max(d[0].mul_add(d[0], d[1] * d[1]).sqrt());
            }
        }
        pior
    };
    let mut rasgos = Vec::new();
    for s in &cut.seams {
        let r = espalhamento(s.chain[0]).max(espalhamento(s.chain[1]));
        if r > 1.0e-6 {
            rasgos.push(r);
        }
    }
    assert_eq!(
        rasgos.len(),
        1,
        "o anel tem de partir UMA vez e nao {}: {rasgos:?}",
        rasgos.len()
    );
    // ⚠️ O rasgo sai em `[0,1]` (o atlas é normalizado) e a holonomia em células de grade;
    // a razão entre os dois é o LADO do quadrado, que é o mesmo para os dois.
    assert!(
        rasgos[0] > 0.0 && a.relatorio.holonomia_max > 1.0,
        "o rasgo e a holonomia sao o mesmo facto: {rasgos:?} contra {}",
        a.relatorio.holonomia_max
    );
}

/// ⛔⛔ **A DISPERSÃO DA COLA é medida, e a fixtura torta prova-o.**
///
/// A premissa do assentamento é que a translação de uma costura colada é **constante**.
/// Aqui um dos dois vértices da costura é deslocado à mão: o relatório tem de acusar.
#[test]
fn uma_costura_torta_e_acusada_em_vez_de_colada_em_silencio() {
    let (mesh, cut, mut map, jumps) = fita(0, false);
    // O vértice global `4` é o local `1` do patch 1 — move-se meia célula.
    map.uv[1][1][1] += 0.5;
    let a = build(&mesh, &cut, &map, &jumps);
    assert!(
        a.relatorio.cola_max > 0.2,
        "uma costura torta tem de aparecer no relatorio: {}",
        a.relatorio.cola_max
    );
    // ⚠️ E ela continua a colar, de propósito: *acusar não é recusar* — quem decide o que
    // fazer com uma peça torta é quem a mostra ao artista, e calar o número seria pior.
    assert_eq!(a.relatorio.ilhas, 1);
}

/// ⭐ **O VÃO existe** — duas ilhas encostadas seriam misturadas pelo primeiro mip.
#[test]
fn ha_um_vao_entre_as_ilhas_e_ele_e_o_do_mip() {
    let (_, a) = corrida(1, false);
    let (lo, hi) = caixas(&a);
    for i in 0..a.relatorio.ilhas {
        assert!(
            lo[i][0] > 0.0 && lo[i][1] > 0.0,
            "ilha {i} encosta na borda"
        );
        assert!(hi[i][0] < 1.0 && hi[i][1] < 1.0, "ilha {i} sai do quadrado");
    }
    let vao = super::VAO_EM_TEXELS / super::TEXTURA_DE_REFERENCIA;
    for (i, l) in lo.iter().enumerate() {
        assert!(
            l[0] >= vao * 0.5 && l[1] >= vao * 0.5,
            "o vao da ilha {i} tem de ser da ordem do que a const declara"
        );
    }
}

/// ⭐⭐ **Um patch que nenhuma costura alcança é uma ilha própria** — e não um patch
/// perdido sem `(u, v)`.
#[test]
fn um_patch_sozinho_vira_ilha_e_nao_um_orfao() {
    let (mesh, mut cut, map, _) = fita(0, false);
    // Tira-se toda costura: os três patches deixam de se conhecer.
    cut.seams.clear();
    let a = build(&mesh, &cut, &map, &[]);
    assert_eq!(a.relatorio.ilhas, 3, "sem costura sao tres ilhas");
    assert_eq!(a.relatorio.orfaos, 0, "e nenhum canto fica sem uv");
}
