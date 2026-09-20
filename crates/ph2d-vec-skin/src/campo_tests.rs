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

pub(super) const W: f64 = 40.0;
pub(super) const H: f64 = 10.0;

pub(super) fn forma() -> VecPath {
    cook(ShapeKind::Rectangle, [0.0, 0.0], [W, H], &[])
}

/// Dois ossos ao longo do eixo maior, com a junta no MEIO — é ela que curva o campo.
pub(super) fn eixos() -> Vec<Handle> {
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
pub(super) fn osso_da_fixtura(x0: f64, rot: f64, t: u32) -> SkinBone {
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
pub(super) fn pele(dobra: f64) -> Skin {
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
pub(super) fn pele_degenerada(dobra: f64) -> Skin {
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
pub(super) fn a_fixtura_deste_ficheiro_nao_colapsa_os_tendoes() {
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
pub(super) fn ponto_do_segmento(path: &VecPath, i: usize, t: f64) -> [f64; 2] {
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
pub(super) fn linha_do_no(tabela: &[f64], k: usize, ossos: usize) -> &[f64] {
    &tabela[k * 3 * ossos..k * 3 * ossos + ossos]
}

/// ⭐⭐⭐ **O DEFEITO QUE ESTA WAVE CURA, medido** — a recta entre dois nós contra o campo.
///
/// ⚠️ **As DUAS metades são obrigatórias.** Sozinha, a primeira (`≥ 0,30` numa aresta que cruza a
/// junta) podia ser ruído da amostragem ou um erro da régua; é a segunda (**exactamente `0`** nas
/// arestas que não cruzam junta nenhuma) que prova que a régua sabe ler um acordo perfeito e que o
/// número da primeira é a LEI.
#[test]
pub(super) fn a_recta_entre_dois_nos_erra_o_campo_onde_ele_dobra() {
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
pub(super) fn nos_nos_o_campo_e_a_tabela_guardada_concordam_ao_bit() {
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
pub(super) fn o_campo_muda_o_desenho_entre_os_nos_e_nao_nos_nos() {
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
pub(super) fn um_caminho_aberto_nao_tem_campo() {
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
pub(super) fn diag_o_preco_do_campo() {
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
pub(super) fn o_campo_e_interpolado_e_nao_um_degrau_por_vertice() {
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

// ─────────────── AS FIXTURAS PARTILHADAS, e a lei que as guarda mora com elas ───────────────
//
// ⚠️ Elas vivem AQUI e não no irmão das sondas porque é aqui que o
// `a_fixtura_deste_ficheiro_nao_colapsa_os_tendoes` as varre: *uma fixtura e o censo que a impede
// de apodrecer são a mesma responsabilidade.*

pub(super) fn barra_do_dono() -> VecPath {
    cook(ShapeKind::RoundRect, [-8.5, 2.0], [-1.5, 3.0], &[0.5])
}

/// Os três eixos no espaço da forma — o que o `tendons_and_axes` entrega quando a forma tem a
/// transformação identidade (que é o caso da fixtura real).
pub(super) fn eixos_do_dono() -> Vec<Handle> {
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
pub(super) fn pele_do_dono(dobra: f64) -> Skin {
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
pub(super) fn pele_com_tendoes(dobra: f64) -> Skin {
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

/// O comprimento do polígono de controlo de um segmento — o majorante que o bind usa.
pub(super) fn comprimento_do_seg(
    a: &ph2d_vec_scene::VecVertex,
    b: &ph2d_vec_scene::VecVertex,
) -> f64 {
    let d = |p: [f64; 2], q: [f64; 2]| (p[0] - q[0]).hypot(p[1] - q[1]);
    d(a.anchor, a.out_handle) + d(a.out_handle, b.in_handle) + d(b.in_handle, b.anchor)
}

/// ⭐ **A subdivisão do BIND, replicada** — `ph2d_skeleton_live::subdivisao` vive numa crate que
/// depende desta, logo não é importável. O passo é o osso mais curto a dividir por `3`, e o corte é
/// o de de Casteljau, que é **exacto**.
///
/// ⚠️ **A guarda da QUINA VIVA está aqui de propósito:** sem ela um `RoundRect` perde recuo e a
/// fixtura deixa de ser a peça do dono.
pub(super) fn subdivide_como_o_bind(path: &mut VecPath, eixos: &[Handle]) -> usize {
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

pub(super) const PASSO: f64 = (-1.8 - -8.2) / 3.0;

pub(super) const BASE: [f64; 2] = [-8.2, 2.5];

#[expect(clippy::cast_precision_loss, reason = "contagens pequenas, sonda")]
pub(super) fn f(n: usize) -> f64 {
    n as f64
}
