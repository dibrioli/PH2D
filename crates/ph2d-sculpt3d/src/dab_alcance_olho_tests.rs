//! ⭐⭐⭐⭐ **QUEM DECIDE A FOLHA QUE O ARTISTA VÊ** — os gates da terceira
//! condição da [`super::dab_alcance`], a da NORMAL.
//!
//! ⚠️ **O corte é de RESPONSABILIDADE e a fronteira é a PERGUNTA:** o irmão
//! [`super::tests`] responde *«a superfície liga isto?»* (o passeio, o tecto e a
//! razão) e aqui responde-se *«é a folha que o olho vê, e em que INSTANTE isso
//! se decide?»*. Ele nasceu em 2026-09-19 quando o ficheiro cruzou o tecto de
//! `700` linhas ao ganhar a memória do traço — ⛔ **curado por corte, nunca por
//! uma entrada nova no `FILE_OVERAGE_OK`**.

use super::{Alcance, OlhoDoTraco};
use ph2d_mesh::{QueryScratch, shapes};

/// ⭐⭐⭐ **A terceira condição NÃO COME as peças aprovadas** — o outro lado da
/// medição, e o que impede que apertá-la vire licença.
///
/// ⚠️ *Uma cura medida só do lado do defeito é metade de uma medição.* Nestas
/// peças o corte tem de ser **exactamente zero**: elas são superfícies de UMA
/// folha, e ali a pergunta «qual das duas folhas?» não tem sujeito.
#[test]
fn a_folha_do_olho_nao_corta_uma_peca_de_uma_folha_so() {
    let olho = [0.0f32, 0.0, -1.0];
    // ⚠️⚠️ **A POPULAÇÃO é a pegada, e a rugosa `0,24` NÃO está aqui — com
    // número.** Ali um pincel pequeno apanha o lábio da ruga vizinha e a
    // pegada CRUA perde `3` de `9`; medido pelo que de facto **SE MOVE** (o
    // caminho do produto, `diag_o_que_a_normal_comeria_nas_pecas_aprovadas`),
    // o corte é `0,00 %` em `R = 0,20` e `0,40` e `1,5 %` em `0,65` com a barra
    // a `0,00` — os três vértices vivem na borda da consulta, onde a queda já é
    // ~zero. *Uma régua sobre a pegada crua conta vértices que não pesam nada*,
    // e foi por isso que a 1.ª redacção deste gate reprovou sobre produto
    // correcto. A rugosa é gateada pela tabela do vale, na régua certa.
    let pecas = [
        ("esfera lisa", ph2d_mesh::shapes::uv_sphere(64, 128, 1.0)),
        ("sculpt_sphere", ph2d_mesh::shapes::sculpt_sphere(1.0)),
    ];
    let mut piso = 0usize;
    for (nome, m) in &pecas {
        // O ponto que o raio do pick atinge: o mais perto do olho.
        let alvo = m
            .positions()
            .iter()
            .copied()
            .max_by(|a, b| a[2].partial_cmp(&b[2]).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or([0.0, 0.0, 1.0]);
        for raio in [0.20f32, 0.40, 0.65] {
            let mut p = Vec::new();
            let mut q = ph2d_mesh::QueryScratch::default();
            m.verts_in_sphere(alvo, raio, &mut q, &mut p);
            let n0 = p.len();
            let mut a = Alcance::default();
            let cortou = a.corta(m, alvo, olho, raio, &mut p, None);
            piso += n0;
            assert_eq!(
                cortou, 0,
                "{nome} R={raio}: a mascara cortou {cortou} de {n0} numa peca de UMA folha"
            );
        }
    }
    // ⚠️ **Piso de população:** sem ele uma consulta que devolvesse pegada vazia
    // deixaria os nove `assert_eq!(0, 0)` trivialmente verdes.
    assert!(
        piso > 500,
        "as seis celulas juntam so' {piso} vertices — a fixtura nao contem o fenomeno"
    );
}

/// ⛔⛔⛔ **SE O CORTE ESVAZIA A PEGADA, ELE NÃO CORRE.**
///
/// A lei escolhe entre DUAS folhas; quando toda a pegada aponta para longe do
/// olho não há duas, e cortar entrega um pincel que **não faz nada**. ⚠️ Esta
/// cerca foi escrita por **seis gates vermelhos** com a mesma mensagem, entre
/// eles o `a_footprint_entirely_facing_away_still_fits_a_sane_plane`.
#[test]
fn uma_pegada_toda_virada_ao_contrario_nao_e_esvaziada() {
    let m = ph2d_mesh::shapes::uv_sphere(48, 96, 1.0);
    // O polo SUL, com o olho a vir de cima: tudo na pegada aponta para longe.
    let alvo = m
        .positions()
        .iter()
        .copied()
        .min_by(|a, b| a[2].partial_cmp(&b[2]).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or([0.0, 0.0, -1.0]);
    let olho = [0.0f32, 0.0, -1.0];
    let raio = 0.40f32;
    let mut p = Vec::new();
    let mut q = ph2d_mesh::QueryScratch::default();
    m.verts_in_sphere(alvo, raio, &mut q, &mut p);
    let n0 = p.len();
    assert!(n0 > 20, "a fixtura nao contem o fenomeno: pegada de {n0}");
    let mut a = Alcance::default();
    a.corta(&m, alvo, olho, raio, &mut p, None);
    assert_eq!(
        p.len(),
        n0,
        "a pegada toda virada foi cortada: {} de {n0} — o pincel fica INERTE e mudo",
        n0 - p.len()
    );
}

/// ⭐⭐ **A CERCA POR VERBO** — quem RELAXA uma região não lê o olho.
///
/// ⛔ Uma região relaxada só de um lado fica torta (medido: o
/// `smoothing_the_lip_of_an_open_mesh_does_not_suck_it_inward` reprova, porque o
/// lábio de uma malha aberta deixa de acompanhar), e os mesmos verbos são os do
/// FILTRO, que corre a peça inteira **sem cursor nenhum**.
///
/// ⚠️ **A régua é o BARRO e não a tabela:** o mesmo gesto com o mesmo olho tem
/// de dar o mesmo `f32` quando o verbo é de relaxar, e resultados DIFERENTES
/// quando ele deposita — senão o gate ficaria verde sobre um predicado que não
/// chega ao produto.
#[test]
fn quem_relaxa_nao_le_o_olho_e_quem_deposita_le() {
    use crate::{Brush, Dab, SculptStroke, Symmetry, Verb};

    let raio = 0.40f32;

    let dif = |a: &[[f32; 3]], b: &[[f32; 3]]| -> f32 {
        a.iter()
            .zip(b)
            .map(|(p, q)| {
                (p[0] - q[0])
                    .abs()
                    .max((p[1] - q[1]).abs())
                    .max((p[2] - q[2]).abs())
            })
            .fold(0.0f32, f32::max)
    };
    let (de_cima, de_baixo) = ([0.0f32, 0.0, -1.0], [0.0f32, 0.0, 1.0]);

    let corre_em = |malha: &ph2d_mesh::Mesh, verbo: Verb, olho: [f32; 3]| -> Vec<[f32; 3]> {
        let mut m = malha.clone();
        let b = Brush {
            verb: verbo,
            radius: raio,
            strength: 1.0,
            surface_only: true,
            ..Brush::default()
        };
        let mut st = SculptStroke::default();
        st.begin(&m);
        let alvo = m
            .positions()
            .iter()
            .copied()
            .max_by(|a, b| a[2].partial_cmp(&b[2]).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or([0.0, 0.0, 1.0]);
        st.dab(&mut m, &b, &Dab::at(alvo, raio, olho), Symmetry::default());
        m.positions().to_vec()
    };

    // ⛔⛔ **E o lado do RELAX corre na MESMA chapa de duas folhas.** A 1.ª
    // redacção usava a esfera rugosa e a mutação *«apaga a cerca por verbo»*
    // SOBREVIVEU: ali, virando o olho, a pegada inteira passa a apontar ao
    // contrário e a **cerca da pegada vazia** desarma a lei de qualquer maneira
    // ⇒ o gate media o caso em que não há nada a cercar. *As duas metades têm de
    // correr na fixtura que TEM duas folhas, senão só uma delas afirma algo.*
    let relax = dif(
        &corre_em(&chapa_fina(), Verb::Smooth, de_cima),
        &corre_em(&chapa_fina(), Verb::Smooth, de_baixo),
    );
    assert!(
        relax <= 1e-6,
        "o Smooth leu o OLHO: os dois lados divergiram {relax:.6e} — uma regiao \
         relaxada so' de um lado fica torta, e o FILTRO nao tem cursor nenhum"
    );

    // ⭐ **O CONTROLO**: um verbo que DEPOSITA tem de ler o olho, senão este
    // gate estaria a medir uma lei que não chega ao produto.
    //
    // ⛔⛔ **E ele tem de correr numa peça de DUAS FOLHAS.** A 1.ª redacção usava
    // a esfera e leu `0,000000e0`: virando o olho, a calota INTEIRA passa a
    // apontar ao contrário e a **cerca da pegada vazia** desarma a lei — ou
    // seja, o controlo media exactamente o caso em que a lei não corre. *Uma
    // fixtura de uma folha só não pode testar a lei que escolhe entre duas.*
    let deposita = dif(
        &corre_em(&chapa_fina(), Verb::Draw, de_cima),
        &corre_em(&chapa_fina(), Verb::Draw, de_baixo),
    );
    assert!(
        deposita > 1e-4,
        "o CONTROLO: o Draw NAO leu o olho ({deposita:.6e}) — a lei nao chega ao produto"
    );
}

/// Uma chapa FINA de duas folhas — a peça que tem duas respostas para «qual
/// folha?». ⚠️ Ela é a mesma proporção da cena `=50` (lado `≫` espessura).
fn chapa_fina() -> ph2d_mesh::Mesh {
    use ph2d_mesh::Face;
    const N: usize = 31;
    const MEIO: f32 = 0.6;
    const T: f32 = 0.06;
    let passo = 2.0 * MEIO / (N - 1) as f32;
    let mut pos = Vec::with_capacity(2 * N * N);
    for z in [T * 0.5, -T * 0.5] {
        for i in 0..N {
            for j in 0..N {
                pos.push([-MEIO + passo * i as f32, -MEIO + passo * j as f32, z]);
            }
        }
    }
    let f = |i: usize, j: usize| (i * N + j) as u32;
    let t = |i: usize, j: usize| (N * N + i * N + j) as u32;
    let mut faces = Vec::new();
    for i in 0..N - 1 {
        for j in 0..N - 1 {
            faces.push(Face::quad(
                f(i, j),
                f(i + 1, j),
                f(i + 1, j + 1),
                f(i, j + 1),
            ));
            faces.push(Face::quad(
                t(i, j),
                t(i, j + 1),
                t(i + 1, j + 1),
                t(i + 1, j),
            ));
        }
    }
    for k in 0..N - 1 {
        faces.push(Face::quad(f(k + 1, 0), f(k, 0), t(k, 0), t(k + 1, 0)));
        faces.push(Face::quad(
            f(k, N - 1),
            f(k + 1, N - 1),
            t(k + 1, N - 1),
            t(k, N - 1),
        ));
        faces.push(Face::quad(f(0, k), f(0, k + 1), t(0, k + 1), t(0, k)));
        faces.push(Face::quad(
            f(N - 1, k + 1),
            f(N - 1, k),
            t(N - 1, k),
            t(N - 1, k + 1),
        ));
    }
    ph2d_mesh::Mesh::from_parts(pos, faces).expect("a chapa é construída aqui e é válida")
}

// ⛔⛔⛔⛔ **A `RAZAO_MAXIMA` FICOU SEM TRABALHO, e o registo fica aqui.**
//
// Ela shipou de manhã (2026-09-19) como a lei desta wave e foi **subsumida pela
// lei da NORMAL na mesma tarde**. Medido:
//
// * com a constante inerte (`1e9`), das **`697`** corridas de `ph2d-sculpt3d` +
//   `ph2d-app-sculpt3d` cai **UMA** — e era um gate que media a própria razão;
// * construído de propósito o regime em que ela devia ser a única a decidir (um
//   verbo que RELAXA, que não lê o olho, numa chapa de duas folhas, com
//   `R = 0,20` e `d = 0,12`, dentro da banda `3,5·t < 2R` calculada à mão), ela
//   lê **`7` de `64` vértices de trás a escapar — com ela e SEM ela, o mesmo
//   número**: ali quem corta é o [`ALCANCE_TECTO`], e os `7` que fogem são os
//   **laterais**, que a razão nunca apanhou por construção (`√(L² + t²) → L`).
//
// ⇒ *nenhuma fixtura deste repo a distingue.* Um gate escrito para a justificar
// seria vácuo, e **um gate vácuo é pior que nenhum** — por isso não há gate
// aqui, há esta nota. A remoção é wave própria (ela toca os gates da manhã, a
// 4.ª metade do gate da cena e o `README` da pasta), e a mutação dela fica
// **NOMEADA** no `docs/3D/geodesica/mutacao_2026-09-19.sh`.

/// ⭐⭐⭐⭐ **A RAZÃO continua a ser a ÚNICA protecção onde o olho não decide.**
///
/// ⛔⛔ **Medido em 2026-09-19: com a [`RAZAO_MAXIMA`] posta inerte (`1e9`), das
/// `697` corridas das duas crates cai UMA** — e era um gate que media a própria
/// razão. *Uma lei que nenhuma mutação mata não é lei.* ⇒ ou ela sai, ou se
/// nomeia onde é que ela ainda trabalha, e há um sítio: **os verbos que RELAXAM
/// não lêem o olho** ([`crate::Verb::a_folha_do_olho_decide`]), logo ali a
/// terceira condição não corre e a razão é tudo o que separa as duas folhas.
///
/// ⚠️ O mesmo vale para o olho degenerado e para a cerca da pegada vazia — três
/// portas pelas quais a lei nova se desarma de propósito.
#[test]
fn onde_o_olho_nao_decide_a_razao_ainda_separa_as_folhas() {
    use crate::{Brush, Dab, SculptStroke, Symmetry, Verb};
    // ⛔⛔ **A chapa tem de ter RELEVO, e o meu próprio aviso mordeu-me:** uma
    // chapa PLANA é o ponto fixo do laplaciano, logo o `Smooth` é inerte nela e
    // a 1.ª redacção reprovou no controlo positivo. ⇒ a frente ganha uma bossa
    // analítica antes (só a FRENTE — pôr relevo nos dois lados esconderia
    // exactamente o que o gate mede).
    let meia = 0.06f32 * 0.5;
    let repouso = {
        let base = chapa_fina();
        let mut pos = base.positions().to_vec();
        for q in &mut pos {
            if (q[2] - meia).abs() >= 1e-5 {
                continue;
            }
            let r = ((q[0] - (0.6 - 0.12)).powi(2) + q[1] * q[1]).sqrt();
            if r < 0.20 {
                let k = 1.0 - r / 0.20;
                q[2] += 0.02 * k * k * (1.0 + 0.8 * (30.0 * r).sin());
            }
        }
        ph2d_mesh::Mesh::from_parts(pos, base.faces().to_vec())
            .expect("a chapa com bossa é derivada de uma chapa válida")
    };
    // ⛔⛔ **O RAIO E A DISTÂNCIA SAEM DE UMA CONTA, não de um palpite** — e a
    // 1.ª redacção usava `R = 0,10`, onde a mutação da razão SOBREVIVEU porque
    // ali quem corta é o [`ALCANCE_TECTO`]. Para a razão ser quem decide é
    // preciso `2d + t ≤ 2R` (o tecto deixa passar) **e** `(2d + t)/t > 3,5` (a
    // razão corta) ⇒ `3,5·t < 2R`, ou seja **`R > 1,75·t`**. Com `t = 0,06` isso
    // é `R > 0,105`; com `R = 0,20` a banda é `0,075 < d < 0,17`.
    let raio = 0.20f32;
    const D_DA_BEIRA: f32 = 0.12;
    const BARRA_DA_FUGA: f32 = 9.9;
    // ⚠️ **O centro sai da MALHA e não da aritmética:** a bossa levantou a frente,
    // e um centro escrito à mão em `z = meia` fica ABAIXO da superfície — a
    // consulta erra a pegada e o controlo positivo reprova (reprovou).
    let centro = repouso
        .positions()
        .iter()
        .copied()
        .filter(|q| (q[2] - meia) > -1e-5)
        .min_by(|a, b| {
            let alvo = 0.6 - D_DA_BEIRA;
            let da = (a[0] - alvo).powi(2) + a[1] * a[1];
            let db = (b[0] - alvo).powi(2) + b[1] * b[1];
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap_or([0.6 - D_DA_BEIRA, 0.0, meia]);

    let mut m = repouso.clone();
    let b = Brush {
        // ⭐ Um verbo que RELAXA ⇒ o olho não decide, e só a razão fica.
        verb: Verb::Smooth,
        radius: raio,
        strength: 1.0,
        surface_only: true,
        ..Brush::default()
    };
    let mut st = SculptStroke::default();
    st.begin(&m);
    st.dab(
        &mut m,
        &b,
        &Dab::at(centro, raio, [0.0, 0.0, -1.0]),
        Symmetry::default(),
    );

    let (mut tocou_frente, mut tocou_costas) = (0usize, 0usize);
    for i in 0..repouso.positions().len() {
        let (a, c) = (repouso.positions()[i], m.positions()[i]);
        let v = [a[0] - c[0], a[1] - c[1], a[2] - c[2]];
        if (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt() <= 1e-7 {
            continue;
        }
        if (a[2] + meia).abs() < 1e-5 {
            tocou_costas += 1;
        } else {
            tocou_frente += 1;
        }
    }
    // ⚠️ **O controlo positivo primeiro:** sem ele um `Smooth` inerte numa chapa
    // PLANA (que é o ponto fixo do laplaciano!) deixaria as duas contagens a
    // zero e o gate verde sobre o nada.
    assert!(
        tocou_frente > 0,
        "o CONTROLO: o Smooth nao tocou a frente — a fixtura nao contem o fenomeno"
    );
    // ⚠️ **A barra é uma FRACÇÃO e sai de um vale medido**, e a razão pela qual
    // ela não é zero está na própria lei: para um ponto de trás a `L` de lado o
    // ar mede `√(L² + t²)` e a razão tende para `1` — *ela nunca separou os
    // laterais*, e é por isso que a lei da normal teve de existir. Aqui ela
    // apanha o miolo, e o que escapa é a orla.
    let frac = tocou_costas as f32 / tocou_frente as f32;
    println!("  frente {tocou_frente} · costas {tocou_costas} · fracção {frac:.3}");
    assert!(
        frac < BARRA_DA_FUGA,
        "com o olho fora de jogo escaparam {tocou_costas} de {tocou_frente} \
         ({frac:.3}): a RAZAO deixou de apanhar o miolo"
    );
}

/// ⭐⭐⭐⭐ **A FOLHA ESCOLHE-SE UMA VEZ, E O VEREDITO NÃO MUDA A MEIO DO TRAÇO.**
///
/// O report de 19/09 (*«Snake Hook … má topologia a face POSTERIOR do traço»*)
/// é esta lei lida na malha VIVA: um gancho puxa um tubo, a face de trás dele
/// vira-se para longe do olho **por construção**, e o vértice que andava sai da
/// pegada a meio do gesto. *Quem já andava pára enquanto o vizinho continua* —
/// e isso é um rasgo, que o passe de topologia depois refina.
///
/// ⛔⛔ **As DUAS metades, e cada uma sozinha mente:**
///
/// * sem o **controlo**, um `assert` de sobrevivência ficaria verde num arranjo
///   onde a condição nem chega a armar;
/// * sem a **memória**, ele ficaria verde sobre a lei que o dono reprovou.
#[test]
fn a_folha_escolhe_se_uma_vez_e_nao_a_meio_do_traco() {
    // ⚠️⚠️ **A FIXTURA TEM DE SER UMA PEÇA ONDE SÓ A NORMAL CORTA — e a 1.ª
    // redacção deste gate usava a CHAPA FINA, onde ela não é.** Ali as costas
    // são cortadas **também** pela [`RAZAO_MAXIMA`] (`11,0`), logo devolver-lhes
    // a memória do olho salvava `38` de `166` e o gate reprovava sobre uma lei
    // correcta. Numa ESFERA a superfície liga tudo (razão `~1,1`) e o único que
    // corta é o olho, que é o sujeito.
    let m = shapes::uv_sphere(48, 96, 1.0);
    let olho = [0.0, 0.0, -1.0];
    let alvo = [0.0, 0.0, 1.0];
    // ⚠️ **O raio é `1,8` e o número é CONTADO, não escolhido:** a consulta é
    // pela CORDA, logo um raio `r` alcança até `θ = 2·asin(r/2)` do polo — e a
    // condição só morde acima de `θ = 107°` (`−cos θ > 0,30`). A `1,3` a pegada
    // pára aos `79°` e o gate lia *«0 cortados de 1689»* sobre a lei certa: *uma
    // fixtura que não alcança o regime mede o nada*.
    let raio = 1.8;

    let pegada_crua = || {
        let mut p = Vec::new();
        let mut q = QueryScratch::default();
        m.verts_in_sphere(alvo, raio, &mut q, &mut p);
        p
    };

    // ── O CONTROLO: sem memória, a lei corta quem se virou ──
    let mut sem = pegada_crua();
    let n0 = sem.len();
    let cortados = Alcance::default().corta(&m, alvo, olho, raio, &mut sem, None);
    assert!(
        cortados > 50 && n0 > 400,
        "a fixtura nao contem o fenomeno: {cortados} cortados de {n0}"
    );
    let virados: Vec<u32> = pegada_crua()
        .into_iter()
        .filter(|v| !sem.contains(v))
        .collect();

    // ── A LEI: os MESMOS vértices, declarados como já capturados com a normal
    //    virada ao artista, FICAM ──
    let n = m.positions().len();
    let mut stamp = vec![0u32; n];
    let mut slot = vec![u32::MAX; n];
    let mut base_nrm = Vec::new();
    for (k, &v) in virados.iter().enumerate() {
        stamp[v as usize] = 7;
        slot[v as usize] = k as u32;
        base_nrm.push([0.0, 0.0, 1.0]);
    }
    let mut com = pegada_crua();
    Alcance::default().corta(
        &m,
        alvo,
        olho,
        raio,
        &mut com,
        Some(&OlhoDoTraco {
            stamp: &stamp,
            slot: &slot,
            base_nrm: &base_nrm,
            epoca: 7,
        }),
    );
    let sobreviveram = virados.iter().filter(|v| com.contains(v)).count();
    assert_eq!(
        sobreviveram,
        virados.len(),
        "{} dos {} vertices que o traco JA' capturava sairam da pegada a meio do \
         gesto — e' o rasgo do report do gancho",
        virados.len() - sobreviveram,
        virados.len()
    );

    // ⚠️⚠️ **E ela é POR VÉRTICE, não um interruptor:** com METADE declarada, só
    // essa metade fica. Sem esta terceira parte, uma implementação que desligasse
    // a condição inteira assim que a memória existisse ficaria verde — e ela
    // apagaria a cura da parede fina, onde as costas NUNCA são capturadas.
    let metade = virados.len() / 2;
    let mut stamp2 = vec![0u32; n];
    let mut slot2 = vec![u32::MAX; n];
    let mut nrm2 = Vec::new();
    for (k, &v) in virados.iter().take(metade).enumerate() {
        stamp2[v as usize] = 7;
        slot2[v as usize] = k as u32;
        nrm2.push([0.0, 0.0, 1.0]);
    }
    let mut meia = pegada_crua();
    Alcance::default().corta(
        &m,
        alvo,
        olho,
        raio,
        &mut meia,
        Some(&OlhoDoTraco {
            stamp: &stamp2,
            slot: &slot2,
            base_nrm: &nrm2,
            epoca: 7,
        }),
    );
    let lembrados = virados
        .iter()
        .take(metade)
        .filter(|v| meia.contains(v))
        .count();
    let esquecidos = virados
        .iter()
        .skip(metade)
        .filter(|v| meia.contains(v))
        .count();
    assert!(metade > 20, "a metade tem so' {metade} vertices");
    assert_eq!(
        (lembrados, esquecidos),
        (metade, 0),
        "a memoria tem de valer VERTICE A VERTICE: {lembrados} de {metade} \
         lembrados ficaram e {esquecidos} esquecidos ficaram com eles"
    );

    // ⛔⛔⛔ **E A CERCA JULGA PELA MESMA GRANDEZA QUE O CORTE — esta parte
    // nasceu de uma MUTAÇÃO SOBREVIVENTE.**
    //
    // A cerca do [`super`] (*«se toda a pegada aponta para longe, há uma folha
    // só e o corte não corre»*) lia a normal VIVA enquanto o corte já lia a
    // congelada. Nenhuma fixtura do corpus separava as duas — nelas a cerca arma
    // dos dois modos —, e o regime que separa é este: **uma pegada inteiramente
    // virada ao contrário, de que o traço já capturou metade pela frente**.
    //
    // ⚠️ Ali as duas leituras dão produtos OPOSTOS: pela viva a cerca não arma e
    // não se corta nada; pela congelada ela arma, e o que sai é exactamente o que
    // o traço nunca apanhou. *A segunda é a certa — quem decide o que é «a folha
    // do artista» é a memória do traço, e uma cerca que pergunte a outra coisa
    // desarma o corte no dia em que o barro dá a volta.*
    let atras = [0.0, 0.0, -1.0];
    let crua_atras = || {
        let mut p = Vec::new();
        let mut q = QueryScratch::default();
        m.verts_in_sphere(atras, 1.0, &mut q, &mut p);
        p
    };
    let toda_virada = crua_atras();
    assert!(toda_virada.len() > 100, "a calota de tras esta' vazia");
    let mut stamp3 = vec![0u32; n];
    let mut slot3 = vec![u32::MAX; n];
    let mut nrm3 = Vec::new();
    for (k, &v) in toda_virada.iter().enumerate() {
        if k % 2 == 0 {
            stamp3[v as usize] = 7;
            slot3[v as usize] = nrm3.len() as u32;
            nrm3.push([0.0, 0.0, 1.0]);
        }
    }
    let mut p4 = crua_atras();
    Alcance::default().corta(
        &m,
        atras,
        atras,
        1.0,
        &mut p4,
        Some(&OlhoDoTraco {
            stamp: &stamp3,
            slot: &slot3,
            base_nrm: &nrm3,
            epoca: 7,
        }),
    );
    let (mut lembrados4, mut esquecidos4) = (0usize, 0usize);
    for (k, &v) in toda_virada.iter().enumerate() {
        if !p4.contains(&v) {
            continue;
        }
        if k % 2 == 0 {
            lembrados4 += 1;
        } else {
            esquecidos4 += 1;
        }
    }
    assert_eq!(
        (lembrados4, esquecidos4),
        (nrm3.len(), 0),
        "numa pegada toda virada ao contrario de que o traco ja' capturava \
         metade, ficaram {lembrados4} lembrados (de {}) e {esquecidos4} \
         esquecidos: a cerca e o corte deixaram de julgar pela mesma grandeza",
        nrm3.len()
    );
}
