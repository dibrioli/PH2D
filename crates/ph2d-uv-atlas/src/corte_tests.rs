//! Gates do corte.
//!
//! ⛔⛔ **A esfera NÃO contém o fenómeno que este módulo ataca, e isso está MEDIDO**
//! (`docs/3D/26_a_parametrizacao_como_atlas.md` §9): sobre `esfera:24` a régua lê
//! `mesma-ilha = 0,00 %`. *Um gate escrito sobre a peça de demonstração ficaria verde a
//! afirmar nada* — por isso as duas fixturas desta folha são construídas para se
//! DOBRAREM, e cada uma traz o controlo plano ao lado.
//!
//! ⚠️ Cuidado com a leitura fácil: na esfera o corte **age na mesma** (`2` → `14` peças),
//! só que pela classe `dobra`, que é o solver a inverter triângulos e não o assentamento.

use super::corte::corta;
use super::sobreposicao::{RUIDO_RELATIVO, area_de_interseccao, medir};
use super::topo;
use ph2d_mesh::{Face, Mesh};

/// ⭐ **O TAPETE AMARROTADO** — uma grelha plana cujo `u` deixa de ser monótono.
///
/// Com `amplitude · 0,9 > 1` a derivada `du/di = 1 + 0,9·A·cos(0,9 i)` troca de sinal e o
/// tapete **volta para trás sobre si mesmo**, que é exactamente a dobra que o corte
/// desfaz. ⚠️ Com `amplitude = 0` ele é uma grelha perfeita — o CONTROLO.
///
/// Devolve `(malha, plano por canto, ilha por face, tem_uv por face)`.
fn tapete(nx: usize, ny: usize, amplitude: f64) -> (Mesh, Vec<[f32; 2]>, Vec<u32>, Vec<bool>) {
    let idx = |i: usize, j: usize| u32::try_from(j * (nx + 1) + i).unwrap_or(0);
    let mut pos = Vec::new();
    for j in 0..=ny {
        for i in 0..=nx {
            pos.push([i as f32, j as f32, 0.0]);
        }
    }
    let mut faces = Vec::new();
    for j in 0..ny {
        for i in 0..nx {
            faces.push(Face::tri(idx(i, j), idx(i + 1, j), idx(i + 1, j + 1)));
            faces.push(Face::tri(idx(i, j), idx(i + 1, j + 1), idx(i, j + 1)));
        }
    }
    let mesh = Mesh::from_parts(pos, faces).expect("a grelha e' valida");
    let u_de = |i: f64| (i + amplitude * (0.9 * i).sin()) as f32;
    let vert = topo::vertices_dos_cantos(&mesh);
    let plano: Vec<[f32; 2]> = vert
        .iter()
        .map(|&v| {
            let (i, j) = (v as usize % (nx + 1), v as usize / (nx + 1));
            [u_de(i as f64), j as f32]
        })
        .collect();
    let n = mesh.faces().len();
    (mesh, plano, vec![0; n], vec![true; n])
}

/// Os triângulos de cada peça.
fn por_peca(mesh: &Mesh, peca_da_face: &[u32], pecas: usize) -> Vec<Vec<u32>> {
    let face_do_tri = topo::faces_dos_triangulos(mesh);
    let mut out = vec![Vec::new(); pecas];
    for (t, &f) in face_do_tri.iter().enumerate() {
        let p = peca_da_face[f as usize];
        if p != u32::MAX {
            out[p as usize].push(u32::try_from(t).unwrap_or(0));
        }
    }
    out
}

fn cantos(plano: &[[f32; 2]], t: [u32; 3]) -> [[f32; 2]; 3] {
    [
        plano[t[0] as usize],
        plano[t[1] as usize],
        plano[t[2] as usize],
    ]
}

fn area(z: [[f32; 2]; 3]) -> f64 {
    let (ux, uy) = (f64::from(z[1][0] - z[0][0]), f64::from(z[1][1] - z[0][1]));
    let (vx, vy) = (f64::from(z[2][0] - z[0][0]), f64::from(z[2][1] - z[0][1]));
    (ux.mul_add(vy, -(uy * vx)) * 0.5).abs()
}

/// Quantos pares de triângulos da MESMA peça se cruzam.
fn cruzamentos_internos(
    mesh: &Mesh,
    plano: &[[f32; 2]],
    peca_da_face: &[u32],
    pecas: usize,
) -> usize {
    let tris = topo::triangulos(mesh);
    let grupos = por_peca(mesh, peca_da_face, pecas);
    let mut n = 0;
    for g in &grupos {
        for a in 0..g.len() {
            for b in (a + 1)..g.len() {
                let (za, zb) = (
                    cantos(plano, tris[g[a] as usize]),
                    cantos(plano, tris[g[b] as usize]),
                );
                if area_de_interseccao(za, zb) > area(za).min(area(zb)) * RUIDO_RELATIVO {
                    n += 1;
                }
            }
        }
    }
    n
}

/// ⭐⭐⭐ **A LEI DA WAVE: depois do corte nenhuma peça se pinta duas vezes.**
///
/// ⚠️ E o CONTROLO é a mesma fixtura por amarrotar: sem ele, um corte que partisse tudo
/// em faces soltas passaria esta lei com distinção.
#[test]
fn um_tapete_amarrotado_sai_do_corte_sem_uma_peca_se_cruzar() {
    let (mesh, plano, ilha, uv) = tapete(16, 8, 2.0);
    // A fixtura CONTÉM o fenómeno — e isto é a metade que o prova.
    let inteiro = vec![0u32; mesh.faces().len()];
    assert!(
        cruzamentos_internos(&mesh, &plano, &inteiro, 1) > 100,
        "o tapete tem de estar amarrotado ANTES do corte"
    );
    let c = corta(&mesh, &plano, &ilha, &uv);
    assert_eq!(
        cruzamentos_internos(&mesh, &plano, &c.peca_da_face, c.pecas),
        0,
        "o corte deixou {} pecas e alguma delas ainda se cruza",
        c.pecas
    );
    assert!(c.recusas > 0 && c.pecas > 1, "{c:?}");

    // ⭐ O CONTROLO: o mesmo tapete ESTICADO sai numa peça só e sem uma recusa.
    let (m2, p2, i2, u2) = tapete(16, 8, 0.0);
    let d = corta(&m2, &p2, &i2, &u2);
    assert_eq!(d.pecas, 1, "um tapete plano nao se corta: {d:?}");
    assert_eq!((d.recusas, d.fusoes, d.pecas_de_uma_face), (0, 0, 0));
}

/// ⭐⭐⭐ **A FUSÃO existe, e a lei dela é a MAXIMALIDADE:** duas peças que se encostam e
/// cujo conjunto continua injectivo **têm de ser uma**.
///
/// ⛔⛔ Sem ela o crescimento ronda-a-ronda reparte a ilha entre frentes que se
/// encontraram sem se cruzarem: na escultura do dono a maior peça caía de `21 801` para
/// `1 457` faces e o total ficava em `382` em vez de `227`. *Um corte que a geometria
/// não pediu é uma costura que o artista vai ver.*
#[test]
fn nenhuma_peca_vizinha_podia_ter_sido_fundida() {
    let (mesh, plano, ilha, uv) = tapete(16, 8, 2.0);
    let c = corta(&mesh, &plano, &ilha, &uv);
    // A fixtura contém o fenómeno: houve o que fundir.
    assert!(c.fusoes > 0, "nada se fundiu — a fixtura nao testa a fusao");

    let tris = topo::triangulos(&mesh);
    let face_do_tri = topo::faces_dos_triangulos(&mesh);
    let grupos = por_peca(&mesh, &c.peca_da_face, c.pecas);
    let mut pares: Vec<(u32, u32)> = Vec::new();
    for (ta, tb, elo) in topo::elos(&mesh, &plano) {
        if !elo.no_atlas {
            continue;
        }
        let (a, b) = (
            c.peca_da_face[face_do_tri[ta as usize] as usize],
            c.peca_da_face[face_do_tri[tb as usize] as usize],
        );
        if a != u32::MAX && b != u32::MAX && a != b {
            pares.push((a.min(b), a.max(b)));
        }
    }
    pares.sort_unstable();
    pares.dedup();
    assert!(!pares.is_empty(), "as pecas tem de se encostar");
    for (a, b) in pares {
        let cruza = grupos[a as usize].iter().any(|&x| {
            grupos[b as usize].iter().any(|&y| {
                let (zx, zy) = (
                    cantos(&plano, tris[x as usize]),
                    cantos(&plano, tris[y as usize]),
                );
                area_de_interseccao(zx, zy) > area(zx).min(area(zy)) * RUIDO_RELATIVO
            })
        });
        assert!(
            cruza,
            "as pecas {a} e {b} encostam-se e NAO se cruzam — deviam ser uma so'"
        );
    }
}

/// ⭐⭐ **Toda face com `(u, v)` fica em exactamente UMA peça** — nem perdida, nem
/// repetida. ⛔ É o piso de população do corte: sem ele um corte que deixasse metade da
/// malha de fora leria zero cruzamentos e passaria a lei da wave.
#[test]
fn o_corte_coloca_cada_face_uma_vez_e_uma_so() {
    let (mesh, plano, ilha, uv) = tapete(9, 5, 2.0);
    let c = corta(&mesh, &plano, &ilha, &uv);
    assert_eq!(c.peca_da_face.len(), mesh.faces().len());
    assert_eq!(c.faces_sem_uv, 0);
    let colocadas = c.peca_da_face.iter().filter(|&&p| p != u32::MAX).count();
    assert_eq!(colocadas, mesh.faces().len(), "nenhuma face fica de fora");
    let mut vistas = vec![0usize; c.pecas];
    for &p in &c.peca_da_face {
        vistas[p as usize] += 1;
    }
    assert!(vistas.iter().all(|&n| n > 0), "nenhuma peca fica vazia");
    assert_eq!(vistas.iter().sum::<usize>(), mesh.faces().len());
}

/// ⭐⭐ **Uma peça é LIGADA no atlas** — ela cresce por vizinhança e por mais nada.
///
/// ⚠️ Sem esta lei, o empacotador poria uma caixa à volta de faces espalhadas pela ilha
/// inteira e o aproveitamento leria bem sobre uma caixa quase vazia.
#[test]
fn cada_peca_e_um_pedaco_ligado_e_nao_um_punhado_de_faces() {
    let (mesh, plano, ilha, uv) = tapete(12, 6, 2.0);
    let c = corta(&mesh, &plano, &ilha, &uv);
    let face_do_tri = topo::faces_dos_triangulos(&mesh);
    let n = mesh.faces().len();
    let mut viz: Vec<Vec<u32>> = vec![Vec::new(); n];
    for (ta, tb, elo) in topo::elos(&mesh, &plano) {
        if !elo.no_atlas {
            continue;
        }
        let (fa, fb) = (face_do_tri[ta as usize], face_do_tri[tb as usize]);
        if fa != fb {
            viz[fa as usize].push(fb);
            viz[fb as usize].push(fa);
        }
    }
    // Uma travessia por peça: se ela for ligada, a travessia alcança-a inteira.
    let mut alcancado = vec![false; n];
    for p in 0..c.pecas {
        let Some(semente) = (0..n).find(|&f| c.peca_da_face[f] == u32::try_from(p).unwrap_or(0))
        else {
            continue;
        };
        let mut pilha = vec![semente];
        alcancado[semente] = true;
        let mut conta = 1usize;
        while let Some(g) = pilha.pop() {
            for &h in &viz[g] {
                let h = h as usize;
                if !alcancado[h] && c.peca_da_face[h] == c.peca_da_face[semente] {
                    alcancado[h] = true;
                    conta += 1;
                    pilha.push(h);
                }
            }
        }
        let total = c.peca_da_face.iter().filter(|&&q| q as usize == p).count();
        assert_eq!(conta, total, "a peca {p} nao e' ligada: {conta} de {total}");
    }
}

/// ⭐⭐⭐ **O corte chega ao ATLAS, e a régua do produto passa de acusar para calar.**
///
/// ⛔ É a metade que falta às de cima: elas medem a PORTA e esta percorre o `build`
/// inteiro — *um corte com a lei certa e o atlas a não o chamar lê-se como um corte que
/// não funciona*.
#[test]
fn a_ilha_que_se_dobrava_deixa_de_pintar_duas_vezes_pelo_caminho_do_produto() {
    let (mesh, cut, map, jumps) = super::lib_tests::fita_com(0, false, true);
    let cru = super::build_com(
        &mesh,
        &cut,
        &map,
        &jumps,
        super::Opcoes {
            cortar: false,
            orientar: false,
            ..super::Opcoes::default()
        },
    );
    let antes = medir(&mesh, &cru);
    use super::sobreposicao::Classe;
    // ⭐ A fixtura contém as DUAS classes que o corte enfrenta, e a atribuição
    // distingue-as: as duas cartas em cima uma da outra dão `mesma-ilha`, e o par que
    // partilha a aresta da costura dá `dobra`.
    assert!(
        antes.area_por_classe[Classe::MesmaIlha.indice()] > 0.0
            && antes.pares_por_classe[Classe::Dobra.indice()] > 0,
        "a fixtura tem de conter o fenomeno nas duas classes: {:?}",
        antes.pares_por_classe
    );
    // ⛔ E o CONTROLO da régua: nunca há cruzamento entre ilhas — as caixas são disjuntas
    // por construção, logo um acerto aqui acusaria o empacotador e não o corte.
    assert_eq!(antes.pares_por_classe[Classe::IlhasDiferentes.indice()], 0);
    assert_eq!(cru.relatorio.ilhas, 1, "sem corte a ilha e' uma so'");

    let a = super::build(&mesh, &cut, &map, &jumps);
    let dep = medir(&mesh, &a);
    assert_eq!(a.relatorio.ilhas_antes_do_corte, 1);
    assert_eq!(a.relatorio.ilhas, 2, "a ilha dobrada parte-se em duas");
    for c in Classe::ALL {
        assert!(
            dep.area_por_classe[c.indice()] <= 0.0,
            "{} ainda cruza depois do corte: {:?}",
            c.nome(),
            dep.pares
        );
    }
    // ⭐ O CONTROLO: a MESMA fita sem a dobra não é cortada.
    let (m2, c2, p2, j2) = super::lib_tests::fita_com(0, false, false);
    let b = super::build(&m2, &c2, &p2, &j2);
    assert_eq!(
        (b.relatorio.ilhas_antes_do_corte, b.relatorio.ilhas),
        (1, 1)
    );
}

/// ⛔⛔ **Uma peça NUNCA atravessa duas ilhas.** Elas assentam cada uma na sua origem e
/// podem encostar-se por acaso — *deixar o corte deduzir a ilha da geometria seria pedir
/// à régua que adivinhasse a resposta*.
#[test]
fn uma_peca_nunca_atravessa_duas_ilhas() {
    let (mesh, plano, mut ilha, uv) = tapete(10, 6, 2.0);
    // Duas ilhas artificiais sobre a MESMA geometria: a metade de cima é outra ilha.
    let n = mesh.faces().len();
    for (f, i) in ilha.iter_mut().enumerate() {
        *i = u32::from(f >= n / 2);
    }
    let c = corta(&mesh, &plano, &ilha, &uv);
    for p in 0..c.pecas {
        let mut quais: Vec<u32> = (0..n)
            .filter(|&f| c.peca_da_face[f] as usize == p)
            .map(|f| ilha[f])
            .collect();
        quais.sort_unstable();
        quais.dedup();
        assert_eq!(quais.len(), 1, "a peca {p} mistura ilhas {quais:?}");
    }
}

/// ⛔⛔ **Um RASGO no plano parte a peça mesmo sem nada se cruzar.**
///
/// Duas metades afastadas partilham arestas da malha e **não encostam no atlas**: pô-las
/// na mesma peça daria uma caixa com o vão inteiro lá dentro, e o aproveitamento leria
/// bem sobre um rectângulo quase vazio. ⚠️ *É a única lei desta folha que não se mede
/// por cruzamento* — as duas metades são disjuntas, logo todo gate de sobreposição fica
/// verde sobre o defeito.
#[test]
fn um_rasgo_no_plano_parte_a_peca_mesmo_sem_cruzamento() {
    let (nx, ny) = (8usize, 4usize);
    let (mesh, mut plano, ilha, uv) = tapete(nx, ny, 0.0);
    // ⛔⛔ **O rasgo é POR CANTO e não por vértice, e a 1.ª redacção deste gate errou-o:**
    // deslocar um VÉRTICE move os cantos dos dois lados juntos, o que não é um rasgo — é
    // uma aresta esticada, e as duas faces continuam a encostar. *Um corte só existe onde
    // o mesmo vértice tem `(u, v)` DIFERENTE de cada lado.*
    let (base, _) = super::bases_dos_cantos(&mesh);
    for (f, face) in mesh.faces().iter().enumerate() {
        if (f / 2) % nx < nx / 2 {
            continue;
        }
        for k in 0..face.verts().len() {
            plano[base[f] as usize + k][0] += 20.0;
        }
    }
    let c = corta(&mesh, &plano, &ilha, &uv);
    assert_eq!(
        c.pecas, 2,
        "as duas metades nao encostam e nao se cruzam: tinham de ser duas pecas"
    );
    assert_eq!(c.recusas, 0, "e nenhuma face foi recusada por cruzamento");
    // ⭐ O CONTROLO: sem o rasgo, o mesmo tapete é UMA peça.
    let (m2, p2, i2, u2) = tapete(nx, ny, 0.0);
    assert_eq!(corta(&m2, &p2, &i2, &u2).pecas, 1);
}
