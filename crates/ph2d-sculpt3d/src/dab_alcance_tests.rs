//! Os gates de [`super::dab_alcance`] — a máscara de alcance.
//!
//! ⚠️ **Eles vivem DENTRO da crate de propósito:** a porta é `pub(crate)`, e a
//! metade que mais importa — *«numa peça convexa a máscara não corta nada»* —
//! precisa de a chamar **directamente**, com e sem. Pela env não se consegue: o
//! `OnceLock` fixa-a no primeiro toque do processo, então um teste que a
//! alternasse mediria o vizinho.

use super::{ALCANCE_TECTO, Alcance};
use ph2d_mesh::{Face, Mesh, QueryScratch, shapes};

/// Duas esferas soltas na MESMA malha, separadas por `folga` no equador — o caso
/// do report: *«esculpir um dedo mexe o vizinho»*.
///
/// ⚠️ **A superfície NÃO as liga**, e é isso que faz esta fixtura decidir: a
/// distância pela superfície é `∞` e nenhum epsilon a explica. ⛔ O
/// `shapes::cylinder` da casa não serve — ele tem dois anéis e não há vértice
/// nenhum a meia altura (a sonda pagou isso com oito células a `NaN`).
fn dois_dedos(folga: f32) -> Mesh {
    let uma = shapes::uv_sphere(48, 96, 1.0);
    let n = uma.vert_count() as u32;
    let dx = 2.0 + folga;
    let mut pos: Vec<[f32; 3]> = Vec::with_capacity(uma.vert_count() * 2);
    for p in uma.positions() {
        pos.push([p[0] - dx * 0.5, p[1], p[2]]);
    }
    for p in uma.positions() {
        pos.push([p[0] + dx * 0.5, p[1], p[2]]);
    }
    let mut faces = uma.faces().to_vec();
    for f in uma.faces() {
        let v: Vec<u32> = f.verts().iter().map(|i| i + n).collect();
        faces.push(if v.len() == 3 {
            Face::tri(v[0], v[1], v[2])
        } else {
            Face::quad(v[0], v[1], v[2], v[3])
        });
    }
    Mesh::from_parts(pos, faces).expect("duas esferas soltas continuam uma malha valida")
}

fn pegada(mesh: &Mesh, centro: [f32; 3], raio: f32) -> Vec<u32> {
    let mut s = QueryScratch::default();
    let mut out = Vec::new();
    mesh.verts_in_sphere(centro, raio, &mut s, &mut out);
    out
}

/// ⭐⭐⭐ **A METADE QUE PROTEGE TUDO O QUE JÁ ESTÁ APROVADO: numa peça convexa a
/// máscara corta ZERO.**
///
/// É esta que faz o caminho de omissão ser byte-idêntico — o peso de quem fica
/// nunca muda, então se ninguém sai, a saída é a mesma ao bit. ⛔ Um futuro
/// aperto do [`ALCANCE_TECTO`] reprova aqui, e é a intenção: a primeira coluna
/// da tabela dele mede exactamente o que se perde ao apertá-lo.
#[test]
fn numa_peca_convexa_a_mascara_nao_corta_nada() {
    let m = shapes::uv_sphere(64, 128, 1.0);
    let mut a = Alcance::default();
    for r in [0.10f32, 0.20, 0.35, 0.50] {
        let mut p = pegada(&m, [0.0, 0.0, 1.0], r);
        let antes = p.len();
        assert!(
            antes > 8,
            "a fixtura tem de apanhar vertices (raio {r}): {antes}"
        );
        let cortados = a.corta(&m, [0.0, 0.0, 1.0], [0.0, 0.0, -1.0], r, &mut p);
        assert_eq!(
            cortados, 0,
            "a mascara cortou {cortados} de {antes} numa ESFERA CONVEXA (raio {r}) -- \
             ali a superficie alcanca tudo, e cortar peso e' a cura a mexer no que \
             ja' esta' certo. O tecto e' {ALCANCE_TECTO}xR."
        );
    }
}

/// ⭐⭐⭐ **E a metade que CURA: o dedo vizinho sai do carimbo.**
///
/// ⚠️ **O olho vem de `+z` e o carimbo pousa no alto do dedo da ESQUERDA, junto
/// ao vinco** — que é o gesto real. ⛔ Um olho ao longo de `x` nunca produz este
/// carimbo: dali o raio bate no lado de fora do dedo da direita, e o vinco fica
/// ocluso.
#[test]
fn o_dedo_vizinho_sai_do_carimbo() {
    let folga = 0.05f32;
    let m = dois_dedos(folga);
    // Um ponto no alto do dedo da esquerda, a 20° do equador dele.
    let (s, c) = 20.0f32.to_radians().sin_cos();
    let centro = [-(2.0 + folga) * 0.5 + c, 0.0, s];
    let raio = 0.35f32;
    let mut p = pegada(&m, centro, raio);
    let direita = |v: u32, p: &Mesh| p.positions()[v as usize][0] > 0.0;
    let antes_dir = p.iter().filter(|&&v| direita(v, &m)).count();
    assert!(
        antes_dir > 0,
        "a fixtura nao produz o fenomeno: o carimbo nao alcanca o dedo da direita"
    );
    let antes_esq = p.len() - antes_dir;
    assert!(antes_esq > 0, "e tem de apanhar o dedo de CA' tambem");

    let mut a = Alcance::default();
    a.corta(&m, centro, [0.0, 0.0, -1.0], raio, &mut p);

    let depois_dir = p.iter().filter(|&&v| direita(v, &m)).count();
    let depois_esq = p.len() - depois_dir;
    assert_eq!(
        depois_dir, 0,
        "sobraram {depois_dir} de {antes_dir} vertices do dedo VIZINHO no carimbo -- \
         a superficie nao os liga de forma alguma"
    );
    assert_eq!(
        depois_esq,
        antes_esq,
        "a mascara comeu {} vertices do dedo de CA' -- ela cortaria o lado que o \
         artista esta' a esculpir, que e' defeito pior que o que ela cura",
        antes_esq - depois_esq
    );
}

/// ⭐⭐ **A SEMENTE cai no dedo de CÁ mesmo com a folga MUITO menor que uma
/// aresta — e é este gate que provou que o `SEMENTE_RECUO` era peso morto.**
///
/// A folga aqui é `0,002` contra uma aresta de `~0,065` no equador: se o vértice
/// mais próximo pudesse saltar de folha, era aqui. Ele não salta, e a razão está
/// escrita na nota apagada do [`super`] — *o centro do dab está SOBRE a folha de
/// cá, e todo vértice da de lá é um da de cá mais a espessura.*
#[test]
fn com_a_folga_menor_que_uma_aresta_a_semente_cai_no_dedo_de_ca() {
    let folga = 0.002f32; // a aresta da esfera 48x96 no equador mede ~0,065
    let m = dois_dedos(folga);
    let (s, c) = 20.0f32.to_radians().sin_cos();
    let centro = [-(2.0 + folga) * 0.5 + c, 0.0, s];
    let raio = 0.35f32;
    let mut p = pegada(&m, centro, raio);
    let antes_esq = p
        .iter()
        .filter(|&&v| m.positions()[v as usize][0] < 0.0)
        .count();
    let mut a = Alcance::default();
    a.corta(&m, centro, [0.0, 0.0, -1.0], raio, &mut p);
    let depois_esq = p
        .iter()
        .filter(|&&v| m.positions()[v as usize][0] < 0.0)
        .count();
    let depois_dir = p.len() - depois_esq;
    assert_eq!(depois_dir, 0, "o dedo vizinho ficou no carimbo");
    assert_eq!(
        depois_esq, antes_esq,
        "a semente caiu no dedo ERRADO: a mascara cortou o lado que se esculpe"
    );
}

/// ⚠️⚠️ **A época pode dar a volta, e nesse tique a máscara PÁRA DE MASCARAR.**
///
/// Com a época em `0`, a marca `0` do vector recém-criado volta a valer «visto
/// nesta corrida» ⇒ o `retain` **guarda tudo**, o dedo vizinho incluído. É a
/// mesma cerca que o [`ph2d_mesh::QueryScratch`] documenta, e este gate corre-a.
///
/// ⛔⛔ **A 1.ª redacção deste gate media uma ESFERA CONVEXA e a mutação que
/// apaga a cerca SOBREVIVEU** — ali não há nada para cortar, então «parou de
/// cortar» é invisível. *A fixtura tem de conter o fenómeno, e a de um gate de
/// cerca é a peça onde a cerca tem consequência.*
///
/// ⚠️ E a chamada de AQUECIMENTO é obrigatória: numa `Alcance` nova o primeiro
/// `corta` aloca os buffers e **reescreve a época para `0`**, o que apagaria o
/// `u32::MAX` que este gate acabou de forçar.
#[test]
fn a_volta_da_epoca_nao_apaga_a_mascara() {
    let folga = 0.05f32;
    let m = dois_dedos(folga);
    let (s, c) = 20.0f32.to_radians().sin_cos();
    let centro = [-(2.0 + folga) * 0.5 + c, 0.0, s];
    let raio = 0.35f32;
    let mut a = Alcance::default();

    // (a) aquecer — e' esta chamada que aloca os buffers.
    let mut aquecer = pegada(&m, centro, raio);
    a.corta(&m, centro, [0.0, 0.0, -1.0], raio, &mut aquecer);

    // (b) um passo antes do wrap, e a corrida a seguir cai em `0`.
    a.forcar_epoca_para_teste(u32::MAX);
    let mut p = pegada(&m, centro, raio);
    let antes_dir = p
        .iter()
        .filter(|&&v| m.positions()[v as usize][0] > 0.0)
        .count();
    assert!(antes_dir > 0, "a fixtura nao produz o fenomeno");
    a.corta(&m, centro, [0.0, 0.0, -1.0], raio, &mut p);
    let depois_dir = p
        .iter()
        .filter(|&&v| m.positions()[v as usize][0] > 0.0)
        .count();
    assert_eq!(
        depois_dir, 0,
        "na volta da epoca a mascara guardou {depois_dir} de {antes_dir} vertices do \
         dedo vizinho -- ela parou de mascarar, uma vez a cada quatro bilhoes de \
         dabs e impossivel de reproduzir sem esta porta"
    );
}

/// Sonda: o que a máscara corta na GRELHA PLANA do corpus do projectar — onde a
/// resposta certa é «nada», porque uma chapa é convexa por dentro.
#[test]
#[ignore = "sonda: imprime, nao afirma nada"]
fn diag_o_que_a_mascara_corta_numa_grelha_plana() {
    const LADO: usize = 41;
    let passo = 2.0 / (LADO - 1) as f32;
    let mut pos = Vec::new();
    for i in 0..LADO {
        for j in 0..LADO {
            pos.push([-1.0 + passo * i as f32, -1.0 + passo * j as f32, 0.0]);
        }
    }
    let at = |i: usize, j: usize| (i * LADO + j) as u32;
    let mut faces = Vec::new();
    for i in 0..LADO - 1 {
        for j in 0..LADO - 1 {
            faces.push(ph2d_mesh::Face::tri(
                at(i, j),
                at(i + 1, j),
                at(i + 1, j + 1),
            ));
            faces.push(ph2d_mesh::Face::tri(
                at(i, j),
                at(i + 1, j + 1),
                at(i, j + 1),
            ));
        }
    }
    let mesh = Mesh::from_parts(pos, faces).expect("a grelha");
    let centro = [0.0, 0.0, 0.0];
    for raio in [0.35f32, 0.40] {
        let p = pegada(&mesh, centro, raio);
        let mut q = p.clone();
        let mut a = Alcance::default();
        let cortou = a.corta(&mesh, centro, [0.0, 0.0, -1.0], raio, &mut q);
        println!("raio {raio:.2}: pegada {} · cortou {cortou}", p.len());
        if cortou > 0 {
            let ficou: std::collections::BTreeSet<u32> = q.iter().copied().collect();
            for &v in p.iter().filter(|v| !ficou.contains(v)).take(6) {
                let d = mesh.positions()[v as usize];
                let ar = ((d[0] - centro[0]).powi(2) + (d[1] - centro[1]).powi(2)).sqrt();
                println!("   cortado v={v} ar={ar:.4} ({:.2}x o raio)", ar / raio);
            }
        }
    }
}

/// ⭐⭐⭐ **O TECTO ABSOLUTO governa o TRABALHO, e é aí que ele tem de ser
/// medido.**
///
/// ⚠️⚠️ **Este gate nasceu de uma mutação SOBREVIVENTE.** Desde que a lei do
/// corte é a [`super::RAZAO_MAXIMA`], apagar o [`super::ALCANCE_TECTO`] **não
/// muda um único vértice da saída** — os quatro gates deste módulo ficam verdes
/// — e passa a varrer a malha INTEIRA a cada dab. *Um tecto que deixou de
/// governar a resposta e passou a governar só o custo precisa de mudar de
/// régua junto.*
///
/// ⭐ **A régua é uma CONTAGEM e não um relógio**, de propósito: uma contagem é
/// determinista, e um gate de razão entre dois tempos é candidato à família de
/// flakes de fan-out que o `CLAUDE.md` §5.0 mantém.
#[test]
fn a_varredura_e_limitada_pela_pegada_e_nao_pela_malha() {
    let mesh = shapes::uv_sphere(128, 256, 1.0);
    let n = mesh.vert_count();
    let centro = [0.0, 0.0, 1.0];
    let raio = 0.10;
    let mut p = pegada(&mesh, centro, raio);
    let mut a = Alcance::default();
    a.corta(&mesh, centro, [0.0, 0.0, -1.0], raio, &mut p);
    let vistos = a.visitados_no_teste();
    assert!(
        vistos > 20,
        "a varredura mal correu ({vistos} de {n}) — o gate esta' a medir um no-op"
    );
    // ⚠️ O tecto é `2 × 0,10 = 0,20` numa esfera de raio `1`, ou seja uma calota
    // de `~1 %` da area. Sem o tecto isto lê `n`.
    assert!(
        vistos * 20 < n,
        "a varredura fixou {vistos} de {n} vertices com um pincel de raio {raio} — \
         o tecto absoluto deixou de limitar o trabalho, e o custo passou a ser \
         O(malha) por dab"
    );
}

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
            let cortou = a.corta(m, alvo, olho, raio, &mut p);
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
    a.corta(&m, alvo, olho, raio, &mut p);
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
