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

/// Um osso da fixtura, no tendão `t`.
fn osso_da_fixtura(x0: f64, rot: f64, t: u32) -> SkinBone {
    let (c, s) = (rot.cos(), rot.sin());
    let b = SkinBone::new(
        Xform([1.0, 0.0, 0.0, 1.0, x0, H * 0.5]),
        W * 0.5,
        1.0,
        Xform([c, s, -s, c, x0, H * 0.5]),
        Xform::IDENTITY,
    )
    .expect("repouso não-singular");
    SkinBone { tendon: t, ..b }
}

/// ⭐⭐⭐ **A pele correspondente, com o segundo osso dobrado — e com TENDÕES DISTINTOS.**
///
/// ⛔⛔⛔ **Ela escreveu `[0, 0]` até 2026-09-20 e foi isso que publicou uma tabela errada num
/// doc-comment do produto** (ver [`crate::curva::lei_do_campo_activa`]). A causa é que
/// [`SkinBone::new`] **não recebe tendão** e crava `0` — «o neutro honesto de um osso SOZINHO», que
/// é verdade para um osso e mentira para dois: com os dois em `0` as duas colunas do campo
/// **colapsam numa**, `5` dos `20` nós ficam com `w = 0` e a sonda lia esse colapso como se fosse
/// a lei.
///
/// ⚠️ **A porta que atribui tendão é a [`SkinBone::bent`]**, e é por ela que o produto passa. Uma
/// fixtura multi-osso montada à mão tem de o dizer, e o gate
/// [`a_fixtura_deste_ficheiro_nao_colapsa_os_tendoes`] recusa a recaída.
fn pele(dobra: f64) -> Skin {
    Skin::new(vec![
        osso_da_fixtura(0.0, 0.0, 0),
        osso_da_fixtura(W * 0.5, dobra, 1),
    ])
    .expect("2 ossos")
}

/// A MESMA pele com os tendões **colapsados** — o CONTROLO que separa *«a lei não faz nada»* de
/// *«a fixtura não a alimenta»*.
///
/// ⚠️ **O nome diz o defeito de propósito:** enquanto ela se chamava `pele` foi usada de boa-fé por
/// duas sondas e por um doc-comment do produto.
fn pele_degenerada(dobra: f64) -> Skin {
    Skin::new(vec![
        osso_da_fixtura(0.0, 0.0, 0),
        osso_da_fixtura(W * 0.5, dobra, 0),
    ])
    .expect("2 ossos")
}

/// ⭐⭐⭐ **GATE — nenhuma pele multi-osso desta fixtura colapsa os tendões.**
///
/// O censo varre as peles que este ficheiro constrói e exige tendões **distintos**, com uma
/// excepção NOMEADA: a [`pele_degenerada`], que existe para ser o controlo.
///
/// ⚠️ **As duas metades são obrigatórias.** Sem a primeira, uma fixtura nova volta a colapsar em
/// silêncio; sem a segunda (*a degenerada AINDA colapsa*), alguém «cura» o controlo e as duas
/// colunas das sondas passam a ler o mesmo número — e a tabela volta a não ter quem a contradiga.
#[test]
fn a_fixtura_deste_ficheiro_nao_colapsa_os_tendoes() {
    let sas: [(&str, Skin); 3] = [
        ("pele", pele(0.8)),
        ("pele_com_tendoes", pele_com_tendoes(0.8)),
        ("pele_do_dono", pele_do_dono(0.6)),
    ];
    for (nome, k) in sas {
        let t: Vec<u32> = k.bones().iter().map(|b| b.tendon).collect();
        let mut vistos = t.clone();
        vistos.sort_unstable();
        vistos.dedup();
        assert_eq!(
            vistos.len(),
            t.len(),
            "a fixtura `{nome}` colapsa os tendões ({t:?}) — as colunas do campo viram uma só"
        );
    }
    // ⚠️ O CONTROLO: a degenerada tem de CONTINUAR degenerada.
    let t: Vec<u32> = pele_degenerada(0.8)
        .bones()
        .iter()
        .map(|b| b.tendon)
        .collect();
    assert_eq!(
        t,
        vec![0, 0],
        "a `pele_degenerada` é o controlo desta família — curá-la apaga a comparação"
    );
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

// ════════════════════════════════════════════════════════════════════════════════════════════
// SONDAS DA LENTE A — auditoria do report do dono de 2026-09-20
// «os pesos não estão na malha mas sim nos pontos do vetor. não houve nenhuma melhora.»
//
// ⛔ Estas funções são SONDAS: elas IMPRIMEM. Os gates que afirmam são os irmãos acima.
// ⚠️ Nenhuma delas mede RELÓGIO — a workstation esteve a `load 26` durante a auditoria, e a
//    lei da casa diz que nenhuma leitura de tempo vale acima de `load ~5`. Todas as grandezas
//    aqui são GEOMÉTRICAS, logo imunes à carga.
// ════════════════════════════════════════════════════════════════════════════════════════════

#[expect(clippy::cast_precision_loss, reason = "contagens pequenas, sonda")]
fn f(n: usize) -> f64 {
    n as f64
}

/// `p50`, `p90` e o MÁXIMO — ⛔ nunca só uma média: *uma régua que mede um extremo global não vê
/// um defeito local, e uma que mede a média não vê nenhum dos dois.*
fn percentis(v: &mut [f64]) -> (f64, f64, f64) {
    if v.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    v.sort_by(f64::total_cmp);
    let q = |fr: f64| v[((f(v.len() - 1)) * fr).round() as usize];
    (q(0.5), q(0.9), *v.last().expect("não vazia"))
}

/// A diagonal da caixa do caminho — o denominador que torna um desvio LEGÍVEL.
fn diagonal(path: &VecPath) -> f64 {
    let c = path.cooked();
    let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for k in 0..c.contour_count() {
        let Some((vs, _)) = c.contour(k) else {
            continue;
        };
        for v in vs {
            for p in [v.anchor, v.in_handle, v.out_handle] {
                x0 = x0.min(p[0]);
                y0 = y0.min(p[1]);
                x1 = x1.max(p[0]);
                y1 = y1.max(p[1]);
            }
        }
    }
    (x1 - x0).hypot(y1 - y0)
}

// ─────────────────────────── A BARRA REAL DA CENA DO DONO ───────────────────────────
//
// ⚠️ Ela é o `cook(RoundRect, [-8.5, 2.0], [-1.5, 3.0], &[0.5])` do
// `ph2d_skeleton_live::barra_da_cena_tests_support`, com a cadeia de TRÊS ossos daquele ficheiro.
// ⛔ Reconstruída aqui porque aquela crate DEPENDE desta — importá-la seria um ciclo.

const PASSO: f64 = (-1.8 - -8.2) / 3.0;
const BASE: [f64; 2] = [-8.2, 2.5];

fn barra_do_dono() -> VecPath {
    cook(ShapeKind::RoundRect, [-8.5, 2.0], [-1.5, 3.0], &[0.5])
}

/// Os três eixos no espaço da forma — o que o `tendons_and_axes` entrega quando a forma tem a
/// transformação identidade (que é o caso da fixtura real).
fn eixos_do_dono() -> Vec<Handle> {
    (0..3)
        .map(|k| {
            let x0 = PASSO.mul_add(f(k), BASE[0]);
            Handle {
                a: [x0, BASE[1]],
                b: [x0 + PASSO, BASE[1]],
            }
        })
        .collect()
}

/// A pele de TRÊS ossos em CADEIA, com os tendões `0`, `1`, `2`.
///
/// ⛔⛔ **O `tendon` é escrito À MÃO e isso NÃO é zelo:** a [`ph2d_skeleton::SkinBone::new`] escreve
/// `tendon: 0` em todo osso que constrói (o doc dela di-lo: *«`0` é o neutro honesto de um osso
/// SOZINHO»*), e o [`ph2d_skeleton::Skin::weights_from`] lê `por_tendao[b.tendon]` — ⇒ numa pele
/// de `N` ossos todos com tendão `0`, **todos leem a mesma casa da tabela** e a normalização
/// entrega `1/N` a cada um, em TODO ponto. *A tabela é lida e deitada fora.*
fn pele_do_dono(dobra: f64) -> Skin {
    let mut bones = Vec::new();
    let mut org = BASE;
    let mut ang = 0.0_f64;
    for k in 0..3 {
        let rest = Xform([1.0, 0.0, 0.0, 1.0, PASSO.mul_add(f(k), BASE[0]), BASE[1]]);
        if k > 0 {
            ang += dobra;
        }
        let (s, c) = ang.sin_cos();
        let pose = Xform([c, s, -s, c, org[0], org[1]]);
        let b =
            SkinBone::new(rest, PASSO, 1.0, pose, Xform::IDENTITY).expect("repouso não-singular");
        bones.push(SkinBone {
            tendon: u32::try_from(k).expect("3 ossos"),
            ..b
        });
        org = [PASSO.mul_add(c, org[0]), PASSO.mul_add(s, org[1])];
    }
    Skin::new(bones).expect("3 ossos")
}

/// A pele de DOIS ossos da fixtura deste ficheiro, **com os tendões corrigidos** — o controlo que
/// separa *«a lei não faz nada»* de *«a fixtura não a alimenta»*.
fn pele_com_tendoes(dobra: f64) -> Skin {
    let osso = |x0: f64, rot: f64, t: u32| {
        let (s, c) = rot.sin_cos();
        let b = SkinBone::new(
            Xform([1.0, 0.0, 0.0, 1.0, x0, H * 0.5]),
            W * 0.5,
            1.0,
            Xform([c, s, -s, c, x0, H * 0.5]),
            Xform::IDENTITY,
        )
        .expect("repouso não-singular");
        SkinBone { tendon: t, ..b }
    };
    Skin::new(vec![osso(0.0, 0.0, 0), osso(W * 0.5, dobra, 1)]).expect("2 ossos")
}

// ─────────────────────── A SUBDIVISÃO DO BIND, replicada ───────────────────────

/// O comprimento do polígono de controlo de um segmento — o majorante que o bind usa.
fn comprimento_do_seg(a: &ph2d_vec_scene::VecVertex, b: &ph2d_vec_scene::VecVertex) -> f64 {
    let d = |p: [f64; 2], q: [f64; 2]| (p[0] - q[0]).hypot(p[1] - q[1]);
    d(a.anchor, a.out_handle) + d(a.out_handle, b.in_handle) + d(b.in_handle, b.anchor)
}

/// ⭐ **A subdivisão do BIND, replicada** — `ph2d_skeleton_live::subdivisao` vive numa crate que
/// depende desta, logo não é importável. O passo é o osso mais curto a dividir por `3`, e o corte é
/// o de de Casteljau, que é **exacto**.
///
/// ⚠️ **A guarda da QUINA VIVA está aqui de propósito:** sem ela um `RoundRect` perde recuo e a
/// fixtura deixa de ser a peça do dono.
fn subdivide_como_o_bind(path: &mut VecPath, eixos: &[Handle]) -> usize {
    let curto = eixos
        .iter()
        .map(|h| (h.b[0] - h.a[0]).hypot(h.b[1] - h.a[1]))
        .filter(|l| *l > 0.0 && l.is_finite())
        .fold(f64::INFINITY, f64::min);
    if !(curto.is_finite() && curto > 0.0) {
        return 0;
    }
    let alvo = curto / 3.0;
    let quinas = path.has_live_corner();
    let recuo = |vs: &[ph2d_vec_scene::VecVertex], fechado: bool, i: usize| -> f64 {
        if vs.get(i).is_none_or(|v| v.corner_size() <= 0.0) {
            return 0.0;
        }
        ph2d_vec_scene::corner_live::corner_at(vs, fechado, i)
            .map_or(0.0, |x| x.setback.min(x.max_setback))
    };
    let par = |p: &VecPath, seg: usize, salto: usize| -> (f64, f64) {
        let Some((c, local)) = p.locate_segment(seg) else {
            return (0.0, 0.0);
        };
        let Some((vs, fechado)) = p.contour(c) else {
            return (0.0, 0.0);
        };
        let n = vs.len();
        (
            recuo(vs, fechado, local),
            recuo(vs, fechado, (local + salto) % n),
        )
    };
    let mut cortes = 0usize;
    for _ in 0..16 {
        let mut longos = Vec::new();
        for c in 0..path.contour_count() {
            let Some((vs, fechado)) = path.contour(c) else {
                continue;
            };
            let n = vs.len();
            let ultimo = if fechado { n } else { n.saturating_sub(1) };
            for i in 0..ultimo {
                longos.push(comprimento_do_seg(&vs[i], &vs[(i + 1) % n]));
            }
        }
        let alvos: Vec<usize> = longos
            .into_iter()
            .enumerate()
            .filter(|(_, l)| *l > alvo)
            .map(|(i, _)| i)
            .collect();
        if alvos.is_empty() {
            break;
        }
        for i in alvos.into_iter().rev() {
            if path.verts.len() >= 4096 {
                return cortes;
            }
            let antes = quinas.then(|| (path.clone(), par(path, i, 1)));
            if ph2d_vec_scene::split_segment(path, i, 0.5).is_none() {
                continue;
            }
            if let Some((copia, (ra, rb))) = antes {
                let (da, db) = par(path, i, 2);
                if ra - da > 1e-9 || rb - db > 1e-9 {
                    *path = copia;
                    continue;
                }
            }
            cortes += 1;
        }
    }
    cortes
}

/// ⭐⭐⭐ **SONDA 0 — A FIXTURA ESTÁ VIVA?**
///
/// Ela responde a uma pergunta que precede todas as outras: *o peso guardado chega a mover alguma
/// coisa?* Se todos os ossos partilharem o tendão `0`, a tabela é lida e **normalizada para
/// uniforme**, e toda medição feita sobre ela mede outro programa.
///
/// ```text
/// bash scripts/ph2d-run.sh cargo test -p ph2d-vec-skin --lib diag_a_saude_da_fixtura -- --nocapture
/// ```
#[test]
fn diag_a_saude_da_fixtura() {
    eprintln!("\n=== SONDA 0 — a fixtura está viva? ===");
    for (nome, pele) in [
        ("pele_degenerada (tendões [0,0])", pele_degenerada(0.8)),
        ("pele_com_tendoes (à mão)", pele_com_tendoes(0.8)),
    ] {
        let t: Vec<u32> = pele.bones().iter().map(|b| b.tendon).collect();
        // Dois pontos bem separados, com linhas de peso OPOSTAS.
        let mut w = pele.scratch();
        pele.weights_corrected([1.0, H * 0.5], Some(&[1.0, 0.0]), &mut w, &[]);
        let a = w.clone();
        pele.weights_corrected([1.0, H * 0.5], Some(&[0.0, 1.0]), &mut w, &[]);
        eprintln!(
            "  {nome:34} tendões={t:?}  w(linha=[1,0])={a:?}  w(linha=[0,1])={w:?}  \
             {}",
            if a == w {
                "⛔ A TABELA NÃO MOVE NADA"
            } else {
                "ok"
            }
        );
    }
    let pd = pele_do_dono(0.6);
    let t: Vec<u32> = pd.bones().iter().map(|b| b.tendon).collect();
    eprintln!("  barra do dono (3 ossos)            tendões={t:?}");
}

/// ⭐⭐⭐ **SONDA 1 — A PROVENIÊNCIA DE CADA PONTO QUE SE MOVE.**
///
/// Para cada uma das três metades de um vértice, quanto de cada camada:
///
/// | camada | o que é |
/// |---|---|
/// | `|ingénuo − fonte|` | a lei dos NÓS: peso da tabela guardada, linha `3k`, **sem campo** |
/// | `|curva − ingénuo|` | o que o ajuste das alças acrescenta (sem campo) |
/// | `|campo − curva|`   | **o que o CAMPO DO DOMÍNIO acrescenta** |
///
/// ```text
/// bash scripts/ph2d-run.sh cargo test -p ph2d-vec-skin --lib diag_a_proveniencia -- --nocapture
/// ```
#[test]
fn diag_a_proveniencia_de_cada_ponto() {
    eprintln!("\n=== SONDA 1 — de onde vem o peso de cada ponto ===");
    for (nome, mut fonte, eixos, pele_f) in [
        (
            "barra do dono (como o bind entrega)",
            barra_do_dono(),
            eixos_do_dono(),
            pele_do_dono(0.6),
        ),
        (
            "rectângulo 40x10, 2 ossos",
            forma(),
            eixos(),
            pele_com_tendoes(0.8),
        ),
    ] {
        let cortes = subdivide_como_o_bind(&mut fonte, &eixos);
        let Some(campo) = campo_do_caminho(&fonte, &eixos) else {
            eprintln!("  {nome}: sem campo");
            continue;
        };
        let tabela = pesos_dos_pontos(&fonte, &campo);
        let nos = fonte.verts_all().count();
        let diag = diagonal(&fonte);

        let corre = |curva: bool, com_campo: bool| {
            let mut p = fonte.clone();
            if curva {
                crate::curva::aplica_pela_curva_com(
                    &pele_f,
                    &mut p,
                    &tabela,
                    &[],
                    true,
                    com_campo.then_some(&campo),
                );
            } else {
                crate::aplica_corrigido_com(&pele_f, &mut p, &tabela, &[], true);
            }
            p
        };
        let ingenuo = corre(false, false);
        let curva_sem = corre(true, false);
        let curva_com = corre(true, true);

        let lista = |a: &VecPath, b: &VecPath, j: usize| -> Vec<f64> {
            let (ca, cb) = (a.cooked(), b.cooked());
            let mut out = Vec::new();
            for k in 0..ca.contour_count() {
                let (Some((va, _)), Some((vb, _))) = (ca.contour(k), cb.contour(k)) else {
                    continue;
                };
                for (x, y) in va.iter().zip(vb) {
                    let (p, q) = match j {
                        0 => (x.anchor, y.anchor),
                        1 => (x.in_handle, y.in_handle),
                        _ => (x.out_handle, y.out_handle),
                    };
                    out.push((p[0] - q[0]).hypot(p[1] - q[1]));
                }
            }
            out
        };

        eprintln!(
            "\n  ── {nome} — {nos} nós ({cortes} cortes do bind), diagonal {diag:.4}, \
             {} ossos",
            campo.ossos()
        );
        eprintln!(
            "     {:<12} {:>32} {:>32} {:>32}",
            "", "|ingénuo−fonte| (lei do NÓ)", "|curva−ingénuo| (ajuste)", "|campo−curva| (CAMPO)"
        );
        for (j, classe) in ["âncora", "alça in", "alça out"].iter().enumerate() {
            let cols = [
                (&fonte, &ingenuo),
                (&ingenuo, &curva_sem),
                (&curva_sem, &curva_com),
            ]
            .map(|(a, b)| percentis(&mut lista(a, b, j)));
            eprintln!(
                "     {classe:<12} {:>10.6}/{:>9.6}/{:>9.6} {:>10.6}/{:>9.6}/{:>9.6} \
                 {:>10.6}/{:>9.6}/{:>9.6}",
                cols[0].0,
                cols[0].1,
                cols[0].2,
                cols[1].0,
                cols[1].1,
                cols[1].2,
                cols[2].0,
                cols[2].1,
                cols[2].2,
            );
        }
        eprintln!("     (cada célula é p50/p90/max)");

        // De onde veio a linha guardada de cada NÓ: do campo amostrado nele, ou herdada do
        // vértice de malha mais próximo (a âncora caiu FORA da malha)?
        let cozido = fonte.cooked();
        let (mut dentro, mut fora) = (0usize, 0usize);
        for k in 0..cozido.contour_count() {
            let Some((vs, _)) = cozido.contour(k) else {
                continue;
            };
            for v in vs {
                if campo.linha(v.anchor).is_some() {
                    dentro += 1;
                } else {
                    fora += 1;
                }
            }
        }
        eprintln!(
            "     âncoras DENTRO da malha: {dentro} · FORA (peso herdado do vizinho): {fora}"
        );
    }
}

// ─────────────────── O CONTRAFACTUAL: a lei da MALHA DENSA (a mídia IMAGEM) ───────────────────

/// Cada vértice da malha deformado pelo peso DELE — é literalmente o que a 2.ª mídia faz.
fn malha_deformada(campo: &CampoDoDominio, pele: &Skin) -> Vec<[f64; 2]> {
    let mut w = pele.scratch();
    (0..campo.malha.rest.len())
        .map(|i| {
            let local = campo.local_do_vertice(i).expect("régua não-nula");
            let linha = campo.linha_do_vertice(i).expect("linha").to_vec();
            pele.weights_corrected(local, Some(&linha), &mut w, &[]);
            pele.blend(local, &w)
        })
        .collect()
}

/// O ponto `p` pela lei da MALHA DENSA — baricêntricas sobre as posições JÁ deformadas.
///
/// ⚠️ **Ela NÃO é a lei da curva.** A curva interpola os PESOS e mistura uma vez; esta mistura
/// cada vértice e interpola as POSIÇÕES. São duas leis, e a diferença entre elas é medida.
fn ponto_da_malha_densa(
    campo: &CampoDoDominio,
    def: &[[f64; 2]],
    p_local: [f64; 2],
) -> Option<[f64; 2]> {
    let pm = [
        (p_local[0] - campo.regua[0]) * campo.regua[2],
        (p_local[1] - campo.regua[1]) * campo.regua[2],
    ];
    for t in &campo.malha.tris {
        let (i0, i1, i2) = (t[0] as usize, t[1] as usize, t[2] as usize);
        let (a, b, c) = (
            campo.malha.rest[i0],
            campo.malha.rest[i1],
            campo.malha.rest[i2],
        );
        let den = (b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1]);
        if den.abs() < 1e-12 {
            continue;
        }
        let u = ((pm[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (pm[1] - a[1])) / den;
        let v = ((b[0] - a[0]) * (pm[1] - a[1]) - (pm[0] - a[0]) * (b[1] - a[1])) / den;
        if u < -1e-9 || v < -1e-9 || u + v > 1.0 + 1e-9 {
            continue;
        }
        let k = 1.0 - u - v;
        return Some([
            v.mul_add(def[i2][0], u.mul_add(def[i1][0], k * def[i0][0])),
            v.mul_add(def[i2][1], u.mul_add(def[i1][1], k * def[i0][1])),
        ]);
    }
    None
}

/// Uma cúbica de um contorno AUTORADO (o mesmo que o algoritmo percorre).
fn cubica_de(vs: &[ph2d_vec_scene::VecVertex], k: usize, n: usize) -> [[f64; 2]; 4] {
    let (a, b) = (&vs[k], &vs[(k + 1) % n]);
    [a.anchor, a.out_handle, b.in_handle, b.anchor]
}

fn eval_cubica(c: &[[f64; 2]; 4], t: f64) -> [f64; 2] {
    let u = 1.0 - t;
    let (b0, b1, b2, b3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
    [
        b3.mul_add(
            c[3][0],
            b2.mul_add(c[2][0], b1.mul_add(c[1][0], b0 * c[0][0])),
        ),
        b3.mul_add(
            c[3][1],
            b2.mul_add(c[2][1], b1.mul_add(c[1][1], b0 * c[0][1])),
        ),
    ]
}

fn dist(a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

/// A linha do nó `k` (índice PLANO) na tabela por ponto de controlo.
fn linha_plana(tabela: &[f64], k: usize, ossos: usize) -> &[f64] {
    &tabela[k * 3 * ossos..k * 3 * ossos + ossos]
}

/// Uma fixtura completa, já subdividida como o bind faz.
struct Caso {
    nome: &'static str,
    fonte: VecPath,
    campo: CampoDoDominio,
    tabela: Vec<f64>,
    pele: Skin,
}

fn casos() -> Vec<Caso> {
    let mut out = Vec::new();
    let mut monta = |nome, mut fonte: VecPath, eixos: Vec<Handle>, pele: Skin, sub: bool| {
        if sub {
            subdivide_como_o_bind(&mut fonte, &eixos);
        }
        if let Some(campo) = campo_do_caminho(&fonte, &eixos) {
            let tabela = pesos_dos_pontos(&fonte, &campo);
            out.push(Caso {
                nome,
                fonte,
                campo,
                tabela,
                pele,
            });
        }
    };
    monta(
        "barra do dono · 8 nós (como o artista desenha)",
        barra_do_dono(),
        eixos_do_dono(),
        pele_do_dono(0.6),
        false,
    );
    monta(
        "barra do dono · 34 nós (como o BIND entrega)",
        barra_do_dono(),
        eixos_do_dono(),
        pele_do_dono(0.6),
        true,
    );
    monta(
        "barra do dono · 34 nós · dobra FORTE 1,2",
        barra_do_dono(),
        eixos_do_dono(),
        pele_do_dono(1.2),
        true,
    );
    monta(
        "rectângulo 40x10 · 20 nós · 2 ossos",
        forma(),
        eixos(),
        pele_com_tendoes(0.8),
        true,
    );
    out
}

/// ⭐⭐⭐ **SONDA 2 — QUANTAS AMOSTRAS LEEM O CAMPO, e quantas caem na mistura.**
///
/// Replica exactamente o que a [`crate::curva::correccao_das_alcas`] pergunta: `AMOSTRAS` pontos
/// por segmento em `t = (i + ½)/N`, mais as duas consultas da `direccao` em `t = 0` e `t = 1`.
///
/// ```text
/// bash scripts/ph2d-run.sh cargo test -p ph2d-vec-skin --lib diag_quantas_amostras -- --nocapture
/// ```
#[test]
fn diag_quantas_amostras_leem_o_campo() {
    eprintln!("\n=== SONDA 2 — o campo é consultado quantas vezes, e responde? ===");
    eprintln!("  (AMOSTRAS = {})", crate::curva::AMOSTRAS);
    for caso in casos() {
        let (mut sim_aj, mut nao_aj) = (0usize, 0usize);
        let (mut sim_dir, mut nao_dir) = (0usize, 0usize);
        for c in 0..caso.fonte.contour_count() {
            let Some((vs, fechado)) = caso.fonte.contour(c) else {
                continue;
            };
            let n = vs.len();
            let segs = if fechado { n } else { n.saturating_sub(1) };
            for k in 0..segs {
                let cub = cubica_de(vs, k, n);
                for i in 0..crate::curva::AMOSTRAS {
                    let t = (f(i) + 0.5) / f(crate::curva::AMOSTRAS);
                    if caso.campo.linha(eval_cubica(&cub, t)).is_some() {
                        sim_aj += 1;
                    } else {
                        nao_aj += 1;
                    }
                }
                for t in [0.0, 1.0] {
                    if caso.campo.linha(eval_cubica(&cub, t)).is_some() {
                        sim_dir += 1;
                    } else {
                        nao_dir += 1;
                    }
                }
            }
        }
        let tot = sim_aj + nao_aj;
        eprintln!(
            "  {:<48} ajuste: {sim_aj}/{tot} no campo ({:.1} %) · direcção: {sim_dir}/{} ({:.1} %)",
            caso.nome,
            100.0 * f(sim_aj) / f(tot.max(1)),
            sim_dir + nao_dir,
            100.0 * f(sim_dir) / f((sim_dir + nao_dir).max(1)),
        );
    }
}

/// ⭐⭐⭐ **SONDA 3 — O TECTO: quanto do erro é do PESO e quanto é do AJUSTE da cúbica.**
///
/// Cinco curvas, todas amostradas densamente no MESMO `t`:
///
/// | curva | o que é |
/// |---|---|
/// | `malha` | a lei da MALHA DENSA — baricêntricas sobre vértices deformados (a mídia IMAGEM) |
/// | `lei+campo` | `blend(C(t), campo.linha(C(t)))` — o que a lei da curva PERSEGUE |
/// | `lei+mist` | `blend(C(t), lerp(ra, rb, t))` — a mesma sem campo |
/// | `entregue` | a CÚBICA que o produto escreve (com ou sem campo) |
/// | `ingénuo` | a cúbica da lei dos nós sozinha |
///
/// ```text
/// bash scripts/ph2d-run.sh cargo test -p ph2d-vec-skin --lib diag_o_tecto_do_ajuste -- --nocapture
/// ```
#[test]
fn diag_o_tecto_do_ajuste_da_cubica() {
    eprintln!("\n=== SONDA 3 — o tecto: o PESO ou o AJUSTE? (p50/p90/max, e % da diagonal) ===");
    const DENSO: usize = 200;
    for caso in casos() {
        let diag = diagonal(&caso.fonte);
        let ossos = caso.campo.ossos();
        let def = malha_deformada(&caso.campo, &caso.pele);

        let corre = |curva: bool, com: bool| {
            let mut p = caso.fonte.clone();
            if curva {
                crate::curva::aplica_pela_curva_com(
                    &caso.pele,
                    &mut p,
                    &caso.tabela,
                    &[],
                    true,
                    com.then_some(&caso.campo),
                );
            } else {
                crate::aplica_corrigido_com(&caso.pele, &mut p, &caso.tabela, &[], true);
            }
            p
        };
        let (ingenuo, sem, com) = (corre(false, false), corre(true, false), corre(true, true));

        // Uma coluna por grandeza medida.
        let mut e_lei = Vec::new(); // |lei+campo − lei+mist|   : o que o campo VALE na lei
        let mut e_malha_lei = Vec::new(); // |malha − lei+campo| : as duas leis do padrão-ouro
        let mut e_fit = Vec::new(); // |entregue(campo) − lei+campo| : o resíduo do AJUSTE
        let mut e_com = Vec::new(); // |entregue(campo) − malha|     : o produto contra o ouro
        let mut e_sem = Vec::new(); // |entregue(sem)   − malha|
        let mut e_ing = Vec::new(); // |ingénuo         − malha|

        let mut w = caso.pele.scratch();
        let mut base = 0usize;
        for c in 0..caso.fonte.contour_count() {
            let (Some((vf, fechado)), Some((vi, _)), Some((vs, _)), Some((vc, _))) = (
                caso.fonte.contour(c),
                ingenuo.contour(c),
                sem.contour(c),
                com.contour(c),
            ) else {
                continue;
            };
            let n = vf.len();
            let segs = if fechado { n } else { n.saturating_sub(1) };
            for k in 0..segs {
                let src = cubica_de(vf, k, n);
                let (ci, cs, cc) = (
                    cubica_de(vi, k, n),
                    cubica_de(vs, k, n),
                    cubica_de(vc, k, n),
                );
                let ra = linha_plana(&caso.tabela, base + k, ossos).to_vec();
                let rb = linha_plana(&caso.tabela, base + (k + 1) % n, ossos).to_vec();
                for i in 0..=DENSO {
                    let t = f(i) / f(DENSO);
                    let p = eval_cubica(&src, t);
                    let mistura: Vec<f64> = (0..ossos)
                        .map(|j| (rb[j] - ra[j]).mul_add(t, ra[j]))
                        .collect();
                    let do_campo = caso.campo.linha(p).unwrap_or_else(|| mistura.clone());
                    caso.pele.weights_corrected(p, Some(&do_campo), &mut w, &[]);
                    let lei_campo = caso.pele.blend(p, &w);
                    caso.pele.weights_corrected(p, Some(&mistura), &mut w, &[]);
                    let lei_mist = caso.pele.blend(p, &w);
                    let Some(ouro) = ponto_da_malha_densa(&caso.campo, &def, p) else {
                        continue; // fora da malha: não há padrão-ouro para comparar
                    };
                    e_lei.push(dist(lei_campo, lei_mist));
                    e_malha_lei.push(dist(ouro, lei_campo));
                    e_fit.push(dist(eval_cubica(&cc, t), lei_campo));
                    e_com.push(dist(eval_cubica(&cc, t), ouro));
                    e_sem.push(dist(eval_cubica(&cs, t), ouro));
                    e_ing.push(dist(eval_cubica(&ci, t), ouro));
                }
            }
            base += n;
        }

        eprintln!("\n  ── {} — diagonal {diag:.4}", caso.nome);
        let linha = |rot: &str, v: &mut Vec<f64>| -> (f64, f64, f64) {
            let (a, b, c) = percentis(v);
            eprintln!(
                "     {rot:<40} {a:>10.6} {b:>10.6} {c:>10.6}   (max = {:.3} % da peça)",
                100.0 * c / diag
            );
            (a, b, c)
        };
        eprintln!("     {:<40} {:>10} {:>10} {:>10}", "", "p50", "p90", "max");
        let lei = linha("o que o CAMPO vale na LEI", &mut e_lei);
        linha("|malha densa − lei da curva|", &mut e_malha_lei);
        let fit = linha("resíduo do AJUSTE (entregue−lei)", &mut e_fit);
        let com = linha("PRODUTO com campo vs OURO", &mut e_com);
        let sem = linha("PRODUTO sem campo vs OURO", &mut e_sem);
        let ing = linha("lei dos NÓS sozinha vs OURO", &mut e_ing);
        eprintln!(
            "     ⇒ o campo compra {:.6} do erro (max {:.6} → {:.6}, {:.1} %); \
             o AJUSTE deixa {:.6} ({:.1} % do erro que fica)",
            sem.2 - com.2,
            sem.2,
            com.2,
            100.0 * (sem.2 - com.2) / sem.2.max(1e-30),
            fit.2,
            100.0 * fit.2 / com.2.max(1e-30),
        );
        eprintln!(
            "     ⇒ o ajuste das alças compra {:.6} sobre a lei dos nós (max {:.6} → {:.6}); \
             a LEI do campo vale {:.6}",
            ing.2 - sem.2,
            ing.2,
            sem.2,
            lei.2,
        );
    }
}

/// ⭐⭐⭐ **SONDA 4 — A PORTA DE ABLAÇÃO bissecta?** — `PH2D_SKIN_CAMPO=0`.
///
/// ⚠️ **A porta é lida no SÍTIO DE CHAMADA e o valor viaja como o `Option<&CampoDoDominio>`**
/// (`campo.then_some(guardado.campo.as_ref()).flatten()`, em `ph2d_skeleton_live::skin_live`) ⇒
/// o A/B honesto é sobre o ARGUMENTO, que é exactamente o que a porta escolhe. ⛔ Pôr a env var
/// aqui seria um canal entre testes — o `cargo test` corre-os em threads do mesmo processo.
///
/// ```text
/// bash scripts/ph2d-run.sh cargo test -p ph2d-vec-skin --lib diag_a_ablacao -- --nocapture
/// ```
#[test]
fn diag_a_ablacao_do_campo() {
    eprintln!("\n=== SONDA 4 — a ablação do campo, em quatro formas ===");
    eprintln!(
        "  lei_do_campo_activa() = {} (sem a env var posta)",
        crate::curva::lei_do_campo_activa()
    );
    for caso in casos() {
        let diag = diagonal(&caso.fonte);
        let corre = |com: bool| {
            let mut p = caso.fonte.clone();
            crate::curva::aplica_pela_curva_com(
                &caso.pele,
                &mut p,
                &caso.tabela,
                &[],
                true,
                com.then_some(&caso.campo),
            );
            p
        };
        let (sem, com) = (corre(false), corre(true));
        let (mut d_no, mut d_alca) = (Vec::new(), Vec::new());
        for c in 0..sem.contour_count() {
            let (Some((a, _)), Some((b, _))) = (sem.contour(c), com.contour(c)) else {
                continue;
            };
            for (x, y) in a.iter().zip(b) {
                d_no.push(dist(x.anchor, y.anchor));
                d_alca.push(dist(x.in_handle, y.in_handle));
                d_alca.push(dist(x.out_handle, y.out_handle));
            }
        }
        let (_, _, mn) = percentis(&mut d_no);
        let (a50, a90, amax) = percentis(&mut d_alca);
        eprintln!(
            "  {:<48} âncoras max {mn:.9} · alças {a50:.6}/{a90:.6}/{amax:.6} \
             (max {:.3} % da peça)",
            caso.nome,
            100.0 * amax / diag
        );
    }
}

/// ⭐⭐⭐ **SONDA 5 — UMA MANCHA DE PINCEL: onde é que ela aterra?**
///
/// A outra metade do report. O pincel de peso pousa uma [`Correccao`] no ESPAÇO; ela é somada
/// dentro da [`ph2d_skeleton::Skin::weights_corrected`], que corre **nos dois sítios** (na âncora,
/// pela lei dos nós; e em cada amostra, pela lei da curva). ⇒ a pergunta não é *«ela faz alguma
/// coisa?»* mas **de quanto, e em que metade do vértice**.
///
/// ⚠️ O raio é o de FÁBRICA do pincel (`40 px` a `100 ppm` ⇒ `0,40` de mundo), sobre uma barra que
/// tem `1,0` de altura.
///
/// ```text
/// bash scripts/ph2d-run.sh cargo test -p ph2d-vec-skin --lib diag_a_mancha_do_pincel -- --nocapture
/// ```
#[test]
fn diag_a_mancha_do_pincel() {
    use ph2d_skeleton::{Correccao, Especie};
    eprintln!("\n=== SONDA 5 — a mancha do pincel: onde aterra, e de quanto ===");
    for caso in casos() {
        let diag = diagonal(&caso.fonte);
        // A junta do meio, na borda de baixo da peça — onde o artista corrigiria.
        let centro = [PASSO.mul_add(1.0, BASE[0]), 2.0];
        let dab = Correccao {
            tendon: 0,
            centro,
            raio: 0.40,
            especie: Especie::Soma(1.0),
        };
        let corre = |cs: &[Correccao]| {
            let mut p = caso.fonte.clone();
            crate::curva::aplica_pela_curva_com(
                &caso.pele,
                &mut p,
                &caso.tabela,
                cs,
                true,
                Some(&caso.campo),
            );
            p
        };
        let (sem, com) = (corre(&[]), corre(std::slice::from_ref(&dab)));
        let mut por_classe = [Vec::new(), Vec::new(), Vec::new()];
        for c in 0..sem.contour_count() {
            let (Some((a, _)), Some((b, _))) = (sem.contour(c), com.contour(c)) else {
                continue;
            };
            for (x, y) in a.iter().zip(b) {
                por_classe[0].push(dist(x.anchor, y.anchor));
                por_classe[1].push(dist(x.in_handle, y.in_handle));
                por_classe[2].push(dist(x.out_handle, y.out_handle));
            }
        }
        // E no DESENHO: o contorno cozido, amostrado denso.
        let mut no_desenho = Vec::new();
        for c in 0..sem.cooked().contour_count() {
            let (ca, cb) = (sem.cooked(), com.cooked());
            let (Some((a, fa)), Some((b, _))) = (ca.contour(c), cb.contour(c)) else {
                continue;
            };
            if a.len() != b.len() {
                continue;
            }
            let n = a.len();
            let segs = if fa { n } else { n.saturating_sub(1) };
            for k in 0..segs {
                let (u, v) = (cubica_de(a, k, n), cubica_de(b, k, n));
                for i in 0..=40 {
                    let t = f(i) / 40.0;
                    no_desenho.push(dist(eval_cubica(&u, t), eval_cubica(&v, t)));
                }
            }
        }
        let cols = por_classe.map(|mut v| percentis(&mut v));
        let (_, d90, dmax) = percentis(&mut no_desenho);
        eprintln!(
            "\n  ── {} — diagonal {diag:.4}, mancha em [{:.3}, {:.3}] r=0,40",
            caso.nome, centro[0], centro[1]
        );
        for (j, classe) in ["âncora", "alça in", "alça out"].iter().enumerate() {
            eprintln!(
                "     {classe:<10} p50 {:>10.6}  p90 {:>10.6}  max {:>10.6}  ({:.3} % da peça)",
                cols[j].0,
                cols[j].1,
                cols[j].2,
                100.0 * cols[j].2 / diag
            );
        }
        eprintln!(
            "     DESENHO (contorno cozido, denso)  p90 {d90:.6}  max {dmax:.6}  ({:.3} % da peça)",
            100.0 * dmax / diag
        );
    }
}

/// ⭐ **SONDA 6 — a mancha ALCANÇA alguma coisa?** O controlo da [`diag_a_mancha_do_pincel`]:
/// sem ele, um `0,000000` lê-se como *«a lei não faz nada»* quando pode ser *«a fixtura não a
/// alcança»*.
#[test]
fn diag_o_alcance_da_mancha() {
    use ph2d_skeleton::{Correccao, Especie};
    eprintln!("\n=== SONDA 6 — a mancha alcança? (controlo da sonda 5) ===");
    let centro = [PASSO.mul_add(1.0, BASE[0]), 2.0];
    for caso in casos() {
        let nos = caso.fonte.verts_all().count();
        let mut min_no = f64::MAX;
        let mut min_am = f64::MAX;
        let mut perto = ([0.0, 0.0], 0.0);
        for c in 0..caso.fonte.contour_count() {
            let Some((vs, fechado)) = caso.fonte.contour(c) else {
                continue;
            };
            let n = vs.len();
            for v in vs {
                min_no = min_no.min(dist(v.anchor, centro));
            }
            let segs = if fechado { n } else { n.saturating_sub(1) };
            for k in 0..segs {
                let cub = cubica_de(vs, k, n);
                for i in 0..crate::curva::AMOSTRAS {
                    let t = (f(i) + 0.5) / f(crate::curva::AMOSTRAS);
                    let p = eval_cubica(&cub, t);
                    let d = dist(p, centro);
                    if d < min_am {
                        min_am = d;
                        perto = (p, t);
                    }
                }
            }
        }
        // O `w` no ponto mais perto, com e sem a mancha.
        let dab = Correccao {
            tendon: 0,
            centro,
            raio: 0.40,
            especie: Especie::Soma(1.0),
        };
        let linha = caso.campo.linha(perto.0);
        let mut w0 = caso.pele.scratch();
        let mut w1 = caso.pele.scratch();
        caso.pele
            .weights_corrected(perto.0, linha.as_deref(), &mut w0, &[]);
        caso.pele
            .weights_corrected(perto.0, linha.as_deref(), &mut w1, &[dab]);
        eprintln!(
            "  {:<48} {nos:>3} nós · nó mais perto {min_no:.4} · amostra mais perta {min_am:.4} \
             (t={:.4}) · campo={} · w {:?} → {:?}",
            caso.nome,
            perto.1,
            if linha.is_some() { "Some" } else { "None" },
            w0.iter()
                .map(|x| (x * 1e4).round() / 1e4)
                .collect::<Vec<_>>(),
            w1.iter()
                .map(|x| (x * 1e4).round() / 1e4)
                .collect::<Vec<_>>(),
        );
    }
}

/// ⛔⛔⛔ **SONDA 7 — A TABELA PUBLICADA em [`crate::curva::lei_do_campo_activa`] foi medida numa
/// fixtura DEGENERADA?**
///
/// Ela diz *«rectângulo `40 × 10`, dois ossos … como o bind entrega (20 nós) · dobra `0,8` ⇒
/// `3,480` (`8,7 %`)»*. A [`pele`] deste ficheiro constrói os dois ossos pela
/// [`ph2d_skeleton::SkinBone::new`], que escreve `tendon: 0` nos DOIS ⇒ o
/// [`ph2d_skeleton::Skin::weights_from`] lê a MESMA casa da tabela para os dois ossos.
///
/// Esta sonda corre o MESMO A/B com as duas peles — a do ficheiro e a de tendões distintos.
#[test]
fn diag_a_tabela_publicada_saiu_de_uma_fixtura_degenerada() {
    eprintln!("\n=== SONDA 7 — a tabela publicada, com e sem o colapso de tendões ===");
    let eixos = eixos();
    let mut fonte = forma();
    let cortes = subdivide_como_o_bind(&mut fonte, &eixos);
    let campo = campo_do_caminho(&fonte, &eixos).expect("campo");
    let tabela = pesos_dos_pontos(&fonte, &campo);
    eprintln!(
        "  rectângulo 40x10 · {} nós ({cortes} cortes) · dobra 0,8 · «% da peça» = /W = 40",
        fonte.verts_all().count()
    );
    for (rot, pele_f) in [
        ("pele_degenerada     — tendões [0,0]", pele_degenerada(0.8)),
        ("pele_com_tendoes()  — tendões [0,1]", pele_com_tendoes(0.8)),
    ] {
        let corre = |com: bool| {
            let mut p = fonte.clone();
            crate::curva::aplica_pela_curva_com(
                &pele_f,
                &mut p,
                &tabela,
                &[],
                true,
                com.then_some(&campo),
            );
            p
        };
        let (sem, com) = (corre(false), corre(true));
        let mut d = Vec::new();
        let mut parados = 0usize;
        let mut total = 0usize;
        for c in 0..sem.contour_count() {
            let (Some((a, _)), Some((b, _))) = (sem.contour(c), com.contour(c)) else {
                continue;
            };
            for (x, y) in a.iter().zip(b) {
                d.push(dist(x.anchor, y.anchor));
                d.push(dist(x.in_handle, y.in_handle));
                d.push(dist(x.out_handle, y.out_handle));
            }
        }
        // Quantos pontos da FONTE o `w` deixa PARADOS (soma zero ⇒ a mistura devolve o ponto)?
        let mut w = pele_f.scratch();
        for c in 0..fonte.contour_count() {
            let Some((vs, _)) = fonte.contour(c) else {
                continue;
            };
            for (k, v) in vs.iter().enumerate() {
                let linha = linha_plana(&tabela, k, campo.ossos());
                pele_f.weights_corrected(v.anchor, Some(linha), &mut w, &[]);
                total += 1;
                if w.iter().sum::<f64>() <= 0.0 {
                    parados += 1;
                }
            }
        }
        let (p50, p90, max) = percentis(&mut d);
        eprintln!(
            "     {rot}  Δ(com−sem) p50 {p50:.6} p90 {p90:.6} max {max:.6} ({:.2} % de W) \
             · nós com w=0 (NÃO se movem): {parados}/{total}",
            100.0 * max / W
        );
    }
}

/// ⭐⭐⭐ **SONDA 8 — O CAMPO EM SI É SÃO?** O perfil do peso ao longo da barra.
///
/// Se o desenho já está a `0,05 %` do padrão-ouro (sonda 3), então uma deformação que o dono ache
/// má **não pode ser da fiação do peso** — ou é da LEI da mistura, ou é do próprio campo. Esta
/// sonda mede a segunda hipótese: a LARGURA DA TRANSIÇÃO de cada osso, em unidades da ALTURA da
/// peça. *Uma transição muito mais estreita que a peça vinca; uma muito mais larga borra a junta.*
#[test]
fn diag_o_perfil_do_campo_ao_longo_da_peca() {
    eprintln!("\n=== SONDA 8 — o campo em si: largura da transição ===");
    let mut fonte = barra_do_dono();
    let eixos = eixos_do_dono();
    subdivide_como_o_bind(&mut fonte, &eixos);
    let campo = campo_do_caminho(&fonte, &eixos).expect("campo");
    let n = campo.ossos();
    // A linha média da barra: y = 2,5, x de -8,2 a -1,8.
    let amostras: Vec<(f64, Vec<f64>)> = (0..=64)
        .filter_map(|i| {
            let x = (-1.8_f64 - -8.2).mul_add(f(i) / 64.0, -8.2);
            campo.linha([x, 2.5]).map(|l| (x, l))
        })
        .collect();
    eprintln!(
        "  {} amostras na linha média, {n} ossos · altura da peça = 1,0 · passo do osso = {PASSO:.4}",
        amostras.len()
    );
    for j in 0..n {
        // A largura em que o peso `j` vai de 0,1 a 0,9 — a transição.
        let (mut lo, mut hi) = (f64::NAN, f64::NAN);
        for (x, l) in &amostras {
            if l[j] >= 0.1 && lo.is_nan() {
                lo = *x;
            }
            if l[j] >= 0.9 {
                hi = *x;
            }
        }
        let pico = amostras.iter().map(|(_, l)| l[j]).fold(0.0_f64, f64::max);
        eprintln!(
            "     osso {j}: pico {pico:.4} · 0,1 em x={lo:>7.3} · 0,9 em x={hi:>7.3} · \
             transição {:.3} (= {:.2} alturas da peça)",
            (hi - lo).abs(),
            (hi - lo).abs()
        );
    }
    eprintln!("  perfil (x, pesos):");
    for (x, l) in amostras.iter().step_by(8) {
        eprintln!(
            "     x={x:>7.3}  {:?}",
            l.iter()
                .map(|v| (v * 1000.0).round() / 1000.0)
                .collect::<Vec<_>>()
        );
    }
}

/// A CURVA amostrada — `n` pontos por segmento cúbico, na ordem do contorno.
///
/// ⚠️ **Ela existe porque um ponto de controlo NÃO é o desenho:** uma alça que anda `δ` move a
/// curva no máximo `4/9 · δ` (o pico da base de Bernstein interior), logo medir alças **majora** o
/// que o artista vê — e medir âncoras **subestima**, porque elas podem não se mexer nada.
fn amostra_a_curva(p: &VecPath, n: usize) -> Vec<[f64; 2]> {
    let mut out = Vec::new();
    for c in 0..p.contour_count() {
        let Some((vs, fechado)) = p.contour(c) else {
            continue;
        };
        let pares = if fechado {
            vs.len()
        } else {
            vs.len().saturating_sub(1)
        };
        for i in 0..pares {
            let (a, b) = (&vs[i], &vs[(i + 1) % vs.len()]);
            for k in 0..n {
                let t = k as f64 / n as f64;
                let u = 1.0 - t;
                let (w0, w1, w2, w3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
                out.push(std::array::from_fn(|d| {
                    w3.mul_add(
                        b.anchor[d],
                        w2.mul_add(
                            b.in_handle[d],
                            w1.mul_add(a.out_handle[d], w0 * a.anchor[d]),
                        ),
                    )
                }));
            }
        }
    }
    out
}

/// ⭐⭐⭐ **SONDA 8 — A TABELA PUBLICADA, RE-MEDIDA NAS TRÊS LINHAS.**
///
/// A [`crate::curva::lei_do_campo_activa`] publicava três números tirados da [`pele`] deste
/// ficheiro, que escreve `tendon: 0` em **ambos** os ossos ([`SkinBone::new`] não recebe tendão) —
/// as duas colunas do campo colapsam numa e cinco dos vinte nós ficam com `w = 0`, logo **não se
/// movem**. Esta sonda corre as MESMAS três linhas com a [`pele_com_tendoes`], que é a única
/// diferença entre as duas colunas.
///
/// ⚠️ **Ela imprime as duas colunas de propósito:** um número honesto ao lado do número que ele
/// substitui é o que impede a tabela de voltar a ser copiada à mão.
#[test]
fn diag_a_tabela_honesta_das_tres_linhas() {
    use ph2d_skeleton::{Correccao, Especie};
    eprintln!("\n=== SONDA 8 — a tabela do `lei_do_campo_activa`, re-medida ===");
    eprintln!("  «% da peça» = max / W = 40 · Δ = (com campo) − (sem campo)");
    eprintln!(
        "  {:<46} | {:^17} | {:^25}",
        "linha da tabela", "FIXTURA DEGENERADA", "FIXTURA HONESTA"
    );
    eprintln!(
        "  {:<46} | {:>7} {:>9} | {:>7} {:>7} {:>8}",
        "", "alças", "curva", "alças", "curva", "% de W"
    );

    let eixos = eixos();
    let dab = Correccao {
        tendon: 1,
        centro: [W * 0.5, 0.0],
        raio: 4.0,
        especie: Especie::Soma(1.0),
    };
    let linhas: [(&str, bool, f64, &[Correccao]); 3] = [
        (
            "como o artista desenha (4 nós) · dobra 0,8",
            false,
            0.8,
            &[],
        ),
        ("como o BIND entrega (20 nós) · dobra 0,8", true, 0.8, &[]),
        (
            "como o bind entrega · dobra 1,5 · peso pintado",
            true,
            1.5,
            std::slice::from_ref(&dab),
        ),
    ];

    for (rot, subdividir, dobra, correcoes) in linhas {
        let mut fonte = forma();
        if subdividir {
            subdivide_como_o_bind(&mut fonte, &eixos);
        }
        let campo = campo_do_caminho(&fonte, &eixos).expect("campo");
        let tabela = pesos_dos_pontos(&fonte, &campo);
        let mut col = Vec::new();
        for pele_f in [pele_degenerada(dobra), pele_com_tendoes(dobra)] {
            let corre = |com: bool| {
                let mut p = fonte.clone();
                crate::curva::aplica_pela_curva_com(
                    &pele_f,
                    &mut p,
                    &tabela,
                    correcoes,
                    true,
                    com.then_some(&campo),
                );
                p
            };
            let (sem, com) = (corre(false), corre(true));
            let (mut d, mut anc) = (Vec::new(), Vec::new());
            for c in 0..sem.contour_count() {
                let (Some((a, _)), Some((b, _))) = (sem.contour(c), com.contour(c)) else {
                    continue;
                };
                for (x, y) in a.iter().zip(b) {
                    anc.push(dist(x.anchor, y.anchor));
                    d.push(dist(x.anchor, y.anchor));
                    d.push(dist(x.in_handle, y.in_handle));
                    d.push(dist(x.out_handle, y.out_handle));
                }
            }
            // Os nós que a tabela deixa com soma zero — eles NÃO se movem, com ou sem campo.
            let mut w = pele_f.scratch();
            let (mut parados, mut total) = (0usize, 0usize);
            for c in 0..fonte.contour_count() {
                let Some((vs, _)) = fonte.contour(c) else {
                    continue;
                };
                for (k, v) in vs.iter().enumerate() {
                    let linha = linha_plana(&tabela, k, campo.ossos());
                    pele_f.weights_corrected(v.anchor, Some(linha), &mut w, correcoes);
                    total += 1;
                    if w.iter().sum::<f64>() <= 0.0 {
                        parados += 1;
                    }
                }
            }
            let mut curva: Vec<f64> = amostra_a_curva(&sem, 16)
                .into_iter()
                .zip(amostra_a_curva(&com, 16))
                .map(|(x, y)| dist(x, y))
                .collect();
            let (_, _, cmax) = percentis(&mut curva);
            let (_, _, max) = percentis(&mut d);
            let (_, _, amax) = percentis(&mut anc);
            col.push((max, 100.0 * max / W, parados, total, cmax, amax));
        }
        assert_eq!(
            col[1].5, 0.0,
            "as âncoras não se movem — é a lei que esta sonda documenta"
        );
        eprintln!(
            "  {rot:<46} | {:>7.3} {:>9.3} | {:>7.3} {:>7.3} {:>7.2}%",
            col[0].0,
            col[0].4,
            col[1].0,
            col[1].4,
            100.0 * col[1].4 / W
        );
    }
    eprintln!("  âncoras: 0,000 nas SEIS células — ligar o campo NÃO move um único nó.");
    eprintln!("  ⇒ a `curva` da fixtura HONESTA é a tabela que o doc publica.");
}
