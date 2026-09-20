//! ⭐⭐⭐ **OS GATES DO CAMPO DO DOMÍNIO** — a malha do bind deixou de ser deitada fora.
//!
//! # O que cada um afirma, e porquê não bastava o irmão
//!
//! 1. **O defeito existe** — a recta entre dois nós erra o campo verdadeiro em `37 %` de um osso
//!    numa aresta que cruza a junta, e **`0,0000`** numa que não cruza. *É o par que separa a lei
//!    do ruído da régua: sem o segundo número, o primeiro podia ser da amostragem.*
//! 2. **A cura chega ao BARRO** — a mesma arte deformada com e sem o campo dá desenhos diferentes,
//!    e a diferença vive **entre** os nós.
//! 3. **Os NÓS não se mexem** — a linha guardada de uma âncora *é* este campo amostrado nela, logo
//!    ligar o campo é um no-op exactamente onde a tabela já respondia.
//! 4. **A tabela continua a ser a de sempre** — o [`super::pesos_do_caminho`] passou a delegar, e
//!    a saída dele tem de ser a mesma até ao bit.

use super::pesos::*;
use ph2d_skeleton::{Skin, SkinBone, Xform};
use ph2d_skin_weights::Handle;
use ph2d_vec_scene::{ShapeKind, VecPath, cook};

const W: f64 = 40.0;
const H: f64 = 10.0;

fn forma() -> VecPath {
    cook(ShapeKind::Rectangle, [0.0, 0.0], [W, H], &[])
}

/// Dois ossos ao longo do eixo maior, com a junta no MEIO — é ela que curva o campo.
fn eixos() -> Vec<Handle> {
    vec![
        Handle {
            a: [0.0, H * 0.5],
            b: [W * 0.5, H * 0.5],
        },
        Handle {
            a: [W * 0.5, H * 0.5],
            b: [W, H * 0.5],
        },
    ]
}

/// A pele correspondente, com o segundo osso dobrado.
fn pele(dobra: f64) -> Skin {
    let osso = |x0: f64, rot: f64| {
        let (c, s) = (rot.cos(), rot.sin());
        SkinBone::new(
            Xform([1.0, 0.0, 0.0, 1.0, x0, H * 0.5]),
            W * 0.5,
            1.0,
            Xform([c, s, -s, c, x0, H * 0.5]),
            Xform::IDENTITY,
        )
        .expect("repouso não-singular")
    };
    Skin::new(vec![osso(0.0, 0.0), osso(W * 0.5, dobra)]).expect("2 ossos")
}

/// Um ponto da cúbica do contorno `0`, segmento `i`, em `t`.
fn ponto_do_segmento(path: &VecPath, i: usize, t: f64) -> [f64; 2] {
    let cozido = path.cooked();
    let (verts, _) = cozido.contour(0).expect("contorno");
    let n = verts.len();
    let (a, b) = (&verts[i], &verts[(i + 1) % n]);
    let u = 1.0 - t;
    let (c0, c1, c2, c3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
    [
        c0 * a.anchor[0] + c1 * a.out_handle[0] + c2 * b.in_handle[0] + c3 * b.anchor[0],
        c0 * a.anchor[1] + c1 * a.out_handle[1] + c2 * b.in_handle[1] + c3 * b.anchor[1],
    ]
}

/// A linha guardada do nó `k` do contorno `0`, da tabela por ponto de controlo.
fn linha_do_no(tabela: &[f64], k: usize, ossos: usize) -> &[f64] {
    &tabela[k * 3 * ossos..k * 3 * ossos + ossos]
}

/// ⭐⭐⭐ **O DEFEITO QUE ESTA WAVE CURA, medido** — a recta entre dois nós contra o campo.
///
/// ⚠️ **As DUAS metades são obrigatórias.** Sozinha, a primeira (`≥ 0,30` numa aresta que cruza a
/// junta) podia ser ruído da amostragem ou um erro da régua; é a segunda (**exactamente `0`** nas
/// arestas que não cruzam junta nenhuma) que prova que a régua sabe ler um acordo perfeito e que o
/// número da primeira é a LEI.
#[test]
fn a_recta_entre_dois_nos_erra_o_campo_onde_ele_dobra() {
    let path = forma();
    let ossos = eixos();
    let campo = campo_do_caminho(&path, &ossos).expect("campo");
    let tabela = pesos_dos_pontos(&path, &campo);
    let n = campo.ossos();
    assert_eq!(n, 2, "a fixtura tem dois ossos");

    // As arestas do rectângulo: 0 e 2 atravessam a junta (são as longas), 1 e 3 não.
    let mut pior = [0.0_f64; 4];
    for (i, p) in pior.iter_mut().enumerate() {
        let (wa, wb) = (
            linha_do_no(&tabela, i, n),
            linha_do_no(&tabela, (i + 1) % 4, n),
        );
        for k in 0..=100 {
            let t = f64::from(k) / 100.0;
            let q = ponto_do_segmento(&path, i, t);
            let Some(verdade) = campo.linha(q) else {
                continue;
            };
            let d = (0..n)
                .map(|j| (wb[j].mul_add(t, wa[j] * (1.0 - t)) - verdade[j]).abs())
                .fold(0.0_f64, f64::max);
            *p = p.max(d);
        }
    }

    assert!(
        pior[0] >= 0.30 && pior[2] >= 0.30,
        "as arestas que CRUZAM a junta têm de acusar a recta: {:.4} e {:.4}",
        pior[0],
        pior[2]
    );
    // ⛔ As curtas não cruzam junta nenhuma: ali a recta é a verdade, e o `0` é o CONTROLO.
    assert!(
        pior[1] < 1e-9 && pior[3] < 1e-9,
        "as arestas que NÃO cruzam a junta têm de ler zero — a régua está a medir ruído: \
         {:.6} e {:.6}",
        pior[1],
        pior[3]
    );
}

/// ⭐⭐⭐ **A LINHA DE UM NÓ É O CAMPO AMOSTRADO NELE** — logo ligar o campo não move um nó.
///
/// ⚠️ É esta igualdade **ao bit** que torna a wave segura: a tabela por ponto de controlo não foi
/// substituída, ela é uma amostragem deste campo, e nas âncoras as duas leis dão o mesmo `f64`.
#[test]
fn nos_nos_o_campo_e_a_tabela_guardada_concordam_ao_bit() {
    let path = forma();
    let campo = campo_do_caminho(&path, &eixos()).expect("campo");
    let tabela = pesos_dos_pontos(&path, &campo);
    let n = campo.ossos();
    let cozido = path.cooked();
    let (verts, _) = cozido.contour(0).expect("contorno");
    let mut vistos = 0usize;
    for (k, v) in verts.iter().enumerate() {
        let Some(do_campo) = campo.linha(v.anchor) else {
            continue; // uma âncora fora da malha herdou o vizinho — outra lei, outro gate
        };
        let guardada = linha_do_no(&tabela, k, n);
        for j in 0..n {
            assert!(
                do_campo[j].to_bits() == guardada[j].to_bits(),
                "nó {k}, osso {j}: o campo dá {} e a tabela guardada {} — elas TÊM de ser a mesma \
                 amostragem",
                do_campo[j],
                guardada[j]
            );
        }
        vistos += 1;
    }
    assert!(
        vistos >= 4,
        "só {vistos} âncoras caíram na malha — a régua não viu o fenómeno"
    );
}

/// ⭐⭐⭐ **A CURA CHEGA AO BARRO** — a mesma arte dobrada com e sem o campo dá desenhos diferentes,
/// e a diferença vive ENTRE os nós.
///
/// ⚠️ **O controlo é a segunda metade:** as ÂNCORAS têm de ficar **onde estavam**, senão isto não
/// seria a cura da transição — seria uma lei nova a mover a forma inteira.
#[test]
fn o_campo_muda_o_desenho_entre_os_nos_e_nao_nos_nos() {
    let path = forma();
    let ossos = eixos();
    let campo = campo_do_caminho(&path, &ossos).expect("campo");
    let tabela = pesos_dos_pontos(&path, &campo);
    let k = pele(0.8);

    let mut sem = path.clone();
    crate::curva::aplica_pela_curva_com(&k, &mut sem, &tabela, &[], true, None);
    let mut com = path.clone();
    crate::curva::aplica_pela_curva_com(&k, &mut com, &tabela, &[], true, Some(&campo));

    let (a, b) = (sem.cooked(), com.cooked());
    let (va, _) = a.contour(0).expect("contorno");
    let (vb, _) = b.contour(0).expect("contorno");
    assert_eq!(
        va.len(),
        vb.len(),
        "as duas leis desenham o mesmo número de nós"
    );

    let mut pior_no = 0.0_f64;
    let mut pior_alca = 0.0_f64;
    for (x, y) in va.iter().zip(vb) {
        pior_no = pior_no.max((x.anchor[0] - y.anchor[0]).hypot(x.anchor[1] - y.anchor[1]));
        for (p, q) in [(x.in_handle, y.in_handle), (x.out_handle, y.out_handle)] {
            pior_alca = pior_alca.max((p[0] - q[0]).hypot(p[1] - q[1]));
        }
    }

    assert!(
        pior_alca > 1e-3,
        "o campo tem de mudar o desenho ENTRE os nós — maior desvio de alça: {pior_alca:.6}"
    );
    assert!(
        pior_no < 1e-9,
        "as ÂNCORAS não se podem mexer: {pior_no:.9} — a lei do nó é a mesma nas duas"
    );
}

/// ⭐ **Um caminho ABERTO continua sem campo** — não há interior, logo não há domínio.
///
/// ⛔ Sem isto, alguém leria o `Option` como *«às vezes falha»* em vez da resposta que ele é.
#[test]
fn um_caminho_aberto_nao_tem_campo() {
    let aberto = cook(ShapeKind::Line, [0.0, 0.0], [W, 0.0], &[]);
    assert!(
        campo_do_caminho(&aberto, &eixos()).is_none(),
        "uma linha de construção tem área zero — inventar-lhe um domínio seria inventar arte"
    );
}

/// ⭐ **A SONDA que mediu o preço da wave** — ela IMPRIME, e os gates acima é que afirmam.
///
/// Corra-a em `--release`: em `debug` o solver lê `~10×` mais lento e o tecto sairia dez vezes
/// menor. ⚠️ E imprima o `loadavg` ao lado — esta workstation não dá leitura de relógio fiável
/// acima de `load ~5`.
///
/// ```text
/// bash scripts/ph2d-run.sh cargo test -p ph2d-vec-skin --lib --release -- diag_o_preco --nocapture
/// ```
#[test]
fn diag_o_preco_do_campo() {
    use std::time::Instant;
    let path = forma();
    let ossos = eixos();
    let k = pele(0.8);

    let t0 = Instant::now();
    let campo = campo_do_caminho(&path, &ossos).expect("campo");
    let bind_ms = t0.elapsed().as_secs_f64() * 1e3;
    let tabela = pesos_dos_pontos(&path, &campo);

    let mede = |c: Option<&CampoDoDominio>| -> f64 {
        let mut melhor = f64::INFINITY;
        for _ in 0..40 {
            let mut p = path.clone();
            let t = Instant::now();
            crate::curva::aplica_pela_curva_com(&k, &mut p, &tabela, &[], true, c);
            melhor = melhor.min(t.elapsed().as_secs_f64() * 1e3);
        }
        melhor
    };
    let sem = mede(None);
    let com = mede(Some(&campo));

    println!("\n{:-<72}", "");
    println!(
        "malha do campo ....... {} vértices, {} triângulos",
        campo.malha.rest.len(),
        campo.malha.tris.len()
    );
    println!("bind (malha + BBW) ... {bind_ms:>8.3} ms   (UMA vez, ao prender)");
    println!(
        "recook SEM campo ..... {sem:>8.3} ms   ({:>5.2} % de um quadro)",
        sem / 16.67 * 100.0
    );
    println!(
        "recook COM campo ..... {com:>8.3} ms   ({:>5.2} % de um quadro)",
        com / 16.67 * 100.0
    );
    println!(
        "o campo acrescenta ... {:>8.3} ms   ({:>5.2} % de um quadro)",
        com - sem,
        (com - sem) / 16.67 * 100.0
    );
    println!(
        "memória do campo ..... {:>8.1} KB  ({} vértices × {} pesos + {} triângulos)",
        (campo.pesos.len() * 8 + campo.malha.rest.len() * 16 + campo.malha.tris.len() * 12) as f64
            / 1024.0,
        campo.malha.rest.len(),
        campo.ossos(),
        campo.malha.tris.len()
    );
    println!("{:-<72}", "");
    println!(
        "loadavg: {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
}

/// ⭐⭐⭐ **O CAMPO É INTERPOLADO E NÃO CONSTANTE POR PEDAÇOS** — e o discriminador é a RESOLUÇÃO.
///
/// # ⛔⛔ Este gate nasceu de uma MUTAÇÃO SOBREVIVENTE
///
/// Trocar a interpolação baricêntrica pelo **vértice mais próximo** não partia um único gate: nas
/// ÂNCORAS as duas leis dão o mesmo (o canto de um rectângulo **é** um vértice da malha, e ali a
/// baricêntrica devolve exactamente o valor dele), e o gate que mede a arte só exige que ela se
/// mexa — *um campo em degraus também move a arte*. ⇒ ele desenharia a deformação **aos saltos** e
/// toda a suíte ficava verde.
///
/// # A régua: dobrar as amostras tem de partir o salto ao meio
///
/// *Um campo contínuo amostrado com metade do passo dá metade do salto; um degrau dá o mesmo.* É a
/// mesma lei que esta casa já usou para separar um campo suave de um campo em patamares, e ela não
/// precisa de saber quanto vale o gradiente.
///
/// ⚠️ **A razão exigida é `≥ 1,7` e não `2,0`**: a fronteira de um triângulo é uma quebra de
/// derivada real (a baricêntrica é `C⁰`, não `C¹`), logo o pior salto não escala perfeitamente.
/// *Uma barra em `2,0` mediria a aritmética da malha, não a lei.*
#[test]
fn o_campo_e_interpolado_e_nao_um_degrau_por_vertice() {
    let path = forma();
    let campo = campo_do_caminho(&path, &eixos()).expect("campo");
    let n = campo.ossos();

    // Uma travessia pelo MIOLO, atravessando a junta — onde o campo de facto varia.
    let salto_maximo = |amostras: usize| -> f64 {
        let mut ant: Option<Vec<f64>> = None;
        let mut pior = 0.0_f64;
        for k in 0..=amostras {
            let x = W * 0.15 + (W * 0.70) * (k as f64 / amostras as f64);
            let Some(linha) = campo.linha([x, H * 0.5]) else {
                continue;
            };
            if let Some(a) = &ant {
                let d = (0..n).map(|j| (linha[j] - a[j]).abs()).fold(0.0, f64::max);
                pior = pior.max(d);
            }
            ant = Some(linha);
        }
        pior
    };

    let grosso = salto_maximo(200);
    let fino = salto_maximo(400);
    assert!(
        grosso > 1e-6,
        "a travessia não viu o campo variar ({grosso:.2e}) — a régua não contém o fenómeno"
    );
    let razao = grosso / fino;
    assert!(
        razao >= 1.7,
        "dobrar as amostras só cortou o salto {razao:.2}× (grosso {grosso:.5}, fino {fino:.5}) — \
         isto é um campo em DEGRAUS, não um campo interpolado"
    );
}
