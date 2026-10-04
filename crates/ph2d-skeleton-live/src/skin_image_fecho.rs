//! ⭐⭐⭐ **A COSTURA DA IMAGEM PRESA, E A ORDEM DAS FACES** — a malha que o quadro desenha.
//!
//! # O defeito, medido (cena `PH2D_VEC_BONE_SMOKE=4` a `(36°, −144°)`)
//!
//! O risquinho é a cor do FUNDO, pura (não uma mistura): **um vão entre dois membros**, e não uma
//! fenda da malha — `0` nós pendurados em `9 091` triângulos, e os triângulos dos dois lados de cada
//! pixel descoberto ficam a `~350 px` um do outro no repouso. A borda de cima do membro de baixo e a
//! do membro dobrado de volta quase se encostam: até `1,5` texel de largura.
//!
//! # A lei: COSER o que é mais fino que dois texels
//!
//! Onde a borda da malha posada ENCARA outra parte dela (a outra borda fica do lado de fora das duas)
//! a menos de [`VAO_MAXIMO_EM_TEXELS`], o vão é cosido: cada amostra da borda liga-se ao ponto mais
//! perto da outra parte por dois triângulos, e a tinta de cada lado é a da SUA beira. ⚠️ O porquê do
//! número está na const; o que importa aqui é o que a lei NÃO faz — ela não arredonda os «V» de
//! dentro das dobras (são a forma da arte, aprovada pelo dono), só cose fios.
//!
//! ⛔⛔ **A lei anterior foi RECUSADA pelo smoke do dono** (2026-10-02, duas rodadas): a bola da
//! silhueta do desenho sobre a borda da malha (`fecho_da_borda`) custava até `58 ms` por imagem por
//! quadro (*«queda de FPS»*) e saltava entre bico e arco de uma pose para a vizinha (*«ora redonda
//! ora pontuda»*) — a escada da grelha na margem transparente era parede para a bola. Medições na
//! [fila §F48](../../../docs/Skeleton/01_a_fila.md). A costura não procura nada: ela é CONTÍNUA na
//! pose por construção (as pontas de cada troço cosido são interpoladas, não amostradas).
//!
//! ⚠️ **Um quadro com costura posa na CPU**: ela nasce no espaço POSADO e a malha da placa traz o
//! repouso. Sem costura a placa posa como sempre, ao bit. `PH2D_SKIN_COSTURA=0` desliga.
//!
//! # ⭐⭐ A ORDEM DAS FACES: o osso mais adiante na corrente pinta por cima
//!
//! Ordem do dono (2026-10-02): *«as faces influenciadas por um osso têm z-index aleatório, e ao se
//! sobrepor às do outro osso misturam-se; melhor seria as do último osso por cima»*. A ordem dos
//! triângulos É a ordem do desenho (as duas portas, placa e CPU), e a da grelha é a das CÉLULAS — na
//! dobra forte os pedaços dos dois membros intercalavam-se. ⇒ [`ordena_pelo_osso`], UMA vez por bind,
//! na gaveta da malha desenhada ([`crate::skin_bake_cache::desenhada_da_arte`]).

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::{Rc, Weak};

use ph2d_poly2d::Mesh2d;
use ph2d_render::SpriteMesh;
use ph2d_skeleton::{Correccao, Skin, Xform};

use crate::skin_image_arte::{BordasDaMalha, PontoDaArte};
use crate::skinned_mesh::SkinnedMesh;

/// ⭐⭐⭐ **O vão mais largo que se cose: `2` texels.**
///
/// ⚠️ O número é da ARTE e não um palpite: cada borda de uma imagem tem `~1` texel de borda suave,
/// logo duas bordas a menos de dois texels uma da outra não têm UM texel de fundo inteiro entre si —
/// o que se vê ali é sempre um fio, nunca uma forma. Medido na cena `=4` (`diag_a_largura_dos_vaos`,
/// `(36°, −120°…−160°)` de `4°` em `4°`): o risquinho do report tem `16` nós da borda abaixo de
/// `1,5` texel e o resto da varredura `0`–`2` nós abaixo de `2` (as pontas dos «V»); entre `2` e `3`
/// texels há `8`–`10` nós em TODAS as poses — o fundo dos «V», que é forma e fica.
pub const VAO_MAXIMO_EM_TEXELS: f64 = 2.0;

/// ⭐⭐⭐ **Só se cose entre partes da pele a mais de `1,25` osso uma da outra** (a diferença das
/// chaves de osso de [`ordena_pelo_osso`]).
///
/// O risquinho nasce quando a corrente DÁ A VOLTA e um membro encosta num que NÃO é o vizinho; o «V»
/// de dentro de uma junta é entre vizinhos e é forma. Medido na cena `=4` (a diferença das chaves
/// nos pontos que seriam cosidos):
///
/// | pose | onde | ossos de distância |
/// |---|---|---|
/// | `(120°, 120°)` · `(36°, −131,25°)` | a ponta de um «V» | `1,000`–`1,046` |
/// | `(36°, −144°)` | o risquinho do report | `1,556`–`1,637` |
/// | `(36°, −146°)` | o mesmo vão, mais fechado | `~2,0` |
///
/// ⇒ `1,25` fica `0,2` acima do «V» e `0,3` abaixo do vão mais apertado. ⛔ Sem o filtro a
/// `(40°, 40°)` cosia as pontas dos «V» (`42` triângulos) e tirava a malha à placa sem fio nenhum.
pub const OSSOS_DE_DISTANCIA: f64 = 1.25;

/// ⭐⭐⭐ **A malha que o quadro desenha** — a porta do [`crate::skin_image::attach_skin_meshes`].
///
/// `bordas` são as desta malha ([`bordas_da`]); vazias, não há costura (um pedaço de 9-slice tem
/// outra numeração).
#[must_use]
#[expect(
    clippy::too_many_arguments,
    reason = "a porta do produto mais os anéis da borda"
)]
pub fn malha_desenhada(
    mesh: Mesh2d,
    p2l: Xform,
    pele: &Skin,
    pesos: &[f64],
    anchor: [f32; 2],
    size: [f32; 2],
    correcoes: &[Correccao],
    bordas: &BordasDaMalha,
) -> Option<SpriteMesh> {
    let vazias = BordasDaMalha::default();
    malha_desenhada_com(
        mesh,
        p2l,
        pele,
        pesos,
        [anchor, size],
        correcoes,
        if lei_da_costura_activa() {
            bordas
        } else {
            &vazias
        },
        crate::skin_image_gpu::a_placa_posa(),
    )
}

/// `PH2D_SKIN_COSTURA=0` desliga a costura — ver o cabeçalho.
#[must_use]
pub fn lei_da_costura_activa() -> bool {
    costura_de(std::env::var("PH2D_SKIN_COSTURA").ok().as_deref())
}

/// A leitura da porta, PURA — ligada salvo `"0"`. ⚠️ É ela que o gate mede.
#[must_use]
pub fn costura_de(valor: Option<&str>) -> bool {
    valor != Some("0")
}

/// ⭐⭐⭐ **A CHAVE DE OSSO de uma linha de pesos** — a profundidade MÉDIA, pesada, dos ossos que a
/// movem (`Σ wⱼ·pⱼ / Σ wⱼ`, `pⱼ` de [`crate::esqueletos::profundidades`]): quem tem chave maior pinta
/// por cima. ⚠️ Uma média e não o osso dominante: uma zona de mistura fica ENTRE os dois membros, e
/// a ordem não dá um salto onde o peso cruza `0,5`. `prof` que não fecha com a linha cai no índice
/// da coluna (o que a chave era até 2026-10-04 — ⛔ errado na ordem `to_bits`, ver a porta).
#[must_use]
pub fn chave_de_osso(w: &[f64], prof: &[f64]) -> f64 {
    let soma: f64 = w.iter().sum();
    #[expect(clippy::cast_precision_loss, reason = "índice de osso")]
    let pos: f64 = w
        .iter()
        .enumerate()
        .map(|(j, p)| {
            p * prof
                .get(j)
                .copied()
                .filter(|_| prof.len() == w.len())
                .unwrap_or(j as f64)
        })
        .sum();
    if soma > 0.0 { pos / soma } else { 0.0 }
}

/// ⭐⭐ **Ordena os triângulos pelo OSSO que os move** — o mais FUNDO na corrente desenha-se depois.
///
/// A chave de um triângulo é a média da [`chave_de_osso`] dos três vértices. A ordenação é ESTÁVEL —
/// faces do mesmo osso mantêm a ordem da grelha. Sem tabela (a lei derivada) a ordem fica como está.
pub fn ordena_pelo_osso(tris: &mut [[u32; 3]], pesos: &[f64], vertices: usize, prof: &[f64]) {
    let ossos = pesos.len() / vertices.max(1);
    if ossos < 2 || pesos.len() != ossos * vertices {
        return;
    }
    let chave: Vec<f64> = pesos
        .chunks_exact(ossos)
        .map(|w| chave_de_osso(w, prof))
        .collect();
    let de = |t: &[u32; 3]| {
        t.iter()
            .map(|&v| chave.get(v as usize).copied().unwrap_or(0.0))
            .sum::<f64>()
    };
    tris.sort_by(|a, b| de(a).total_cmp(&de(b)));
}

/// A [`malha_desenhada`] com as portas do ambiente escolhidas — a dos gates. `bordas` vazias = sem
/// costura.
#[must_use]
#[expect(
    clippy::too_many_arguments,
    reason = "a porta do produto mais as duas leis do ambiente"
)]
pub fn malha_desenhada_com(
    mesh: Mesh2d,
    p2l: Xform,
    pele: &Skin,
    pesos: &[f64],
    [anchor, size]: [[f32; 2]; 2],
    correcoes: &[Correccao],
    bordas: &BordasDaMalha,
    placa: bool,
) -> Option<SpriteMesh> {
    let costura = costura(&mesh, bordas, p2l, pele, pesos, [anchor, size], correcoes);
    // ⭐⭐⭐ **QUEM POSA: a PLACA, por omissão** (F9 W2, 2026-09-20). Os dois caminhos entregam um
    // `SpriteMesh`; a diferença é se o `local` traz o POSADO (a CPU, a referência) ou o REPOUSO mais
    // a tabela que o `vs_main` lê. ⚠️ `PH2D_SKIN_GPU=0` bissecta. ⛔ A lei que decide (a medição do
    // §0.0) mora no cabeçalho do [`crate::skin_image_gpu`].
    let construtor = if placa && costura.tris.is_empty() {
        crate::skin_image_gpu::sprite_mesh_para_a_placa
    } else {
        crate::skin_image::posed_sprite_mesh_corrigida
    };
    let mut malha = construtor(mesh, p2l, pele, pesos, anchor, size, correcoes)?;
    let base = u32::try_from(malha.local.len()).ok()?;
    malha.local.extend(costura.local);
    malha.uv.extend(costura.uv);
    malha
        .tris
        .extend(costura.tris.into_iter().map(|t| t.map(|i| i + base)));
    Some(malha)
}

/// ⭐⭐ **Os anéis da borda** — as arestas que só UM triângulo usa, encadeadas pela orientação dele
/// (o de fora num sentido, os buracos no outro).
///
/// ⚠️ Custa `~0,93 ms` na malha da cena (`9 091` triângulos) e só depende da TOPOLOGIA ⇒ quem
/// desenha pede-os ao [`bordas_da`], que os guarda por malha.
#[must_use]
pub fn aneis_da_borda(tris: &[[u32; 3]]) -> Vec<Vec<u32>> {
    let mut arestas: Vec<(u32, u32)> = tris
        .iter()
        .flat_map(|t| [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])])
        .collect();
    arestas.sort_unstable();
    let partilhada = |a: u32, b: u32| arestas.binary_search(&(b, a)).is_ok();
    let mut seguinte: BTreeMap<u32, u32> = arestas
        .iter()
        .filter(|&&(a, b)| !partilhada(a, b))
        .copied()
        .collect();
    let mut aneis = Vec::new();
    while let Some((a0, b0)) = seguinte.pop_first() {
        let mut anel = vec![a0];
        let mut b = b0;
        while b != a0 {
            anel.push(b);
            let Some(c) = seguinte.remove(&b) else {
                // ⚠️ Uma borda que não fecha (um vértice com DUAS arestas de borda a sair, onde dois
                // pedaços se tocam por um canto) não é um contorno — o anel fica de fora.
                anel.clear();
                break;
            };
            b = c;
        }
        if anel.len() >= 3 {
            aneis.push(anel);
        }
    }
    aneis
}

/// A gaveta dos anéis: o endereço da malha → a prova (`Weak`) e os anéis.
type Bordas = BTreeMap<usize, (Weak<SkinnedMesh>, Rc<BordasDaMalha>)>;

/// Um encontro da borda com a outra parte: `(segmento, t nele, distância)`.
type Encontro = (usize, f64, f64);

thread_local! {
    /// Os anéis por malha desenhada — a chave é a gaveta da `skin_bake_cache` (o `Rc` dela), e a
    /// entrada morre com ela.
    static BORDAS: RefCell<Bordas> = const { RefCell::new(BTreeMap::new()) };
}

/// ⭐⭐ **Os anéis da borda desta malha desenhada**, calculados UMA vez por gaveta.
///
/// ⚠️ A identidade é o `Rc` da gaveta ([`crate::skin_bake_cache::desenhada_da_arte`]) e o `Weak`
/// prova-a: um endereço reaproveitado por outra malha não casa, e as entradas mortas saem.
#[must_use]
pub fn bordas_da(malha: &Rc<SkinnedMesh>) -> Rc<BordasDaMalha> {
    let chave = Rc::as_ptr(malha) as usize;
    BORDAS.with(|b| {
        let mut b = b.borrow_mut();
        if let Some((w, aneis)) = b.get(&chave)
            && w.upgrade().is_some_and(|m| Rc::ptr_eq(&m, malha))
        {
            return Rc::clone(aneis);
        }
        b.retain(|_, (w, _)| w.strong_count() > 0);
        let aneis = aneis_da_borda(&malha.mesh.tris);
        let arte = malha.mascara.as_ref().map_or_else(Vec::new, |m| {
            crate::skin_image_arte::anel_da_arte(&malha.mesh, m, &aneis)
        });
        let bordas = Rc::new(BordasDaMalha { aneis, arte });
        b.insert(chave, (Rc::downgrade(malha), Rc::clone(&bordas)));
        bordas
    })
}

/// O que a costura acrescenta: pontos POSADOS, a UV de cada um e os triângulos.
#[derive(Default)]
struct Costura {
    local: Vec<[f32; 2]>,
    uv: Vec<[f32; 2]>,
    tris: Vec<[u32; 3]>,
}

/// Um segmento da borda posada: as pontas (índices em `pos`/`uv`), o anel e o vizinho de cada lado.
#[derive(Clone, Copy)]
struct Seg {
    a: usize,
    b: usize,
    antes: usize,
    depois: usize,
}

/// ⭐⭐⭐ **O que a costura acrescenta nesta pose** — vazia quando nada encara nada.
fn costura(
    mesh: &Mesh2d,
    bordas: &BordasDaMalha,
    p2l: Xform,
    pele: &Skin,
    pesos: &[f64],
    [anchor, size]: [[f32; 2]; 2],
    correcoes: &[Correccao],
) -> Costura {
    let mut out = Costura::default();
    let aneis = &bordas.aneis;
    let ids: Vec<u32> = aneis.iter().flatten().copied().collect();
    if ids.is_empty() || mesh.rest.is_empty() {
        return out;
    }
    // ⭐ **A borda posa-se pela PORTA da malha** — uma malha só de borda, sem triângulos, com a
    // tabela das mesmas linhas: uma segunda conta da pose divergiria da que a arte desenha.
    let ossos = pesos.len() / mesh.rest.len();
    let linhas: Vec<f64> = ids
        .iter()
        .flat_map(|&v| {
            let v = v as usize;
            pesos
                .get(v * ossos..(v + 1) * ossos)
                .unwrap_or(&[])
                .iter()
                .copied()
        })
        .collect();
    let so_borda = Mesh2d {
        rest: ids.iter().map(|&v| mesh.rest[v as usize]).collect(),
        tris: Vec::new(),
        size: mesh.size,
    };
    let Some(borda) = crate::skin_image::posed_sprite_mesh_corrigida(
        so_borda, p2l, pele, &linhas, anchor, size, correcoes,
    ) else {
        return out;
    };
    let mut pos: Vec<[f64; 2]> = borda
        .local
        .iter()
        .map(|p| [f64::from(p[0]), f64::from(p[1])])
        .collect();
    let mut uv = borda.uv;
    let mut segs = Vec::with_capacity(pos.len());
    let mut k0 = 0;
    for anel in aneis {
        let n = anel.len();
        for i in 0..n {
            // Um segmento por nó: o índice do segmento `i` do anel é o do nó `i`.
            segs.push(Seg {
                a: k0 + i,
                b: k0 + (i + 1) % n,
                antes: k0 + (i + n - 1) % n,
                depois: k0 + (i + 1) % n,
            });
        }
        k0 += n;
    }
    // O lado de FORA: o do anel de maior área (o de fora) diz o sentido de todos.
    let mut sinal = sinal_dos_aneis(&pos, aneis.iter().map(Vec::len));
    let [a, b, c, d, _, _] = p2l.0;
    let texel = (a * d - b * c).abs().sqrt();
    let vao = VAO_MAXIMO_EM_TEXELS * texel;
    if !(vao > 0.0 && vao.is_finite()) || sinal == 0.0 {
        return out;
    }
    // ⭐⭐ **Só se cose entre partes a mais de [`OSSOS_DE_DISTANCIA`]** — a chave de cada nó é a da
    // ordem das faces (`Σ wⱼ·j / Σ wⱼ`).
    if ossos < 2 {
        return out;
    }
    let mut chave: Vec<f64> = linhas.chunks_exact(ossos).map(chave_da_coluna).collect();
    // ⭐ **A saída rápida**: os segmentos por faixa de `1/4` de osso, a caixa de cada faixa, e só se
    // segue quando duas faixas que PODEM estar a mais de `OSSOS_DE_DISTANCIA` (os índices a `≥ 5`
    // faixas: a diferença entre membros delas chega a `(|i − j| + 1)/4`) têm as caixas a menos de
    // `2·vão`. Na pose recta e nas dobras sem contacto a costura custa a borda posada e isto.
    {
        let chave_do = |s: &Seg| 0.5 * (chave[s.a] + chave[s.b]);
        let mut faixas: BTreeMap<i64, [f64; 4]> = BTreeMap::new();
        for sg in &segs {
            #[expect(clippy::cast_possible_truncation, reason = "faixa de osso")]
            let f = (chave_do(sg) * 4.0).floor() as i64;
            let c = faixas
                .entry(f)
                .or_insert([f64::MAX, f64::MAX, f64::MIN, f64::MIN]);
            for q in [pos[sg.a], pos[sg.b]] {
                *c = [
                    c[0].min(q[0]),
                    c[1].min(q[1]),
                    c[2].max(q[0]),
                    c[3].max(q[1]),
                ];
            }
        }
        let perto = |a: &[f64; 4], b: &[f64; 4]| {
            a[0] - 2.0 * vao <= b[2]
                && b[0] - 2.0 * vao <= a[2]
                && a[1] - 2.0 * vao <= b[3]
                && b[1] - 2.0 * vao <= a[3]
        };
        let ha_par = faixas
            .iter()
            .any(|(i, a)| faixas.range(i + 5..).any(|(_, b)| perto(a, b)));
        if !ha_par {
            return out;
        }
    }
    // ⭐⭐⭐ A5-a: com a máscara da tinta, o vão mede-se e cose-se sobre o ANEL DA ARTE (a malha passa
    // da tinta; na cúspide de uma tampa ela dizia «sem vão»). A saída rápida acima fica na malha:
    // ela passa da arte, logo se a malha não encara nada a arte também não.
    if !bordas.arte.is_empty()
        && let Some(b) = borda_da_arte(
            mesh,
            &bordas.arte,
            p2l,
            pele,
            pesos,
            [anchor, size],
            correcoes,
            ossos,
        )
    {
        (pos, uv, chave, segs, sinal) = b;
    }
    let normal = |s: &Seg| {
        let (p, q) = (pos[s.a], pos[s.b]);
        let (tx, ty) = (q[0] - p[0], q[1] - p[1]);
        let l = tx.hypot(ty).max(f64::MIN_POSITIVE);
        [ty / l * sinal, -tx / l * sinal]
    };
    let chave_do = |s: &Seg| 0.5 * (chave[s.a] + chave[s.b]);
    let longe = |si: usize, sj: usize| {
        let (a, b) = (&segs[si], &segs[sj]);
        sj != si
            && sj != a.antes
            && sj != a.depois
            && (chave_do(a) - chave_do(b)).abs() > OSSOS_DE_DISTANCIA
    };
    // A grelha dos segmentos, com células de `2·vão`: um ponto encontra numa vizinhança 3×3 todo
    // segmento a menos de `2·vão` dele — o dobro do que se cose, para as pontas se interpolarem.
    // ⚠️ Um vector ORDENADO por célula e não um mapa: é refeita a cada quadro.
    let celula = 2.0 * vao;
    #[expect(clippy::cast_possible_truncation, reason = "células de uma sprite")]
    let cel = |x: f64| (x / celula).floor() as i64;
    let mut grelha: Vec<((i64, i64), usize)> = Vec::with_capacity(segs.len() * 2);
    for (si, s) in segs.iter().enumerate() {
        let (p, q) = (pos[s.a], pos[s.b]);
        for gx in cel(p[0].min(q[0]))..=cel(p[0].max(q[0])) {
            for gy in cel(p[1].min(q[1]))..=cel(p[1].max(q[1])) {
                grelha.push(((gx, gy), si));
            }
        }
    }
    grelha.sort_unstable();
    let na_celula = |c: (i64, i64)| {
        let ini = grelha.partition_point(|e| e.0 < c);
        grelha[ini..]
            .iter()
            .take_while(move |e| e.0 == c)
            .map(|e| e.1)
    };
    // O ponto mais perto da OUTRA parte que encara `p` (do segmento `si`): `(segmento, t, distância)`.
    let encara = |p: [f64; 2], si: usize| -> Option<Encontro> {
        let n = normal(&segs[si]);
        let mut melhor: Option<Encontro> = None;
        for gx in cel(p[0]) - 1..=cel(p[0]) + 1 {
            for gy in cel(p[1]) - 1..=cel(p[1]) + 1 {
                for sj in na_celula((gx, gy)) {
                    if !longe(si, sj) {
                        continue;
                    }
                    let o = segs[sj];
                    let (q0, q1) = (pos[o.a], pos[o.b]);
                    let dd = [q1[0] - q0[0], q1[1] - q0[1]];
                    let l2 = dd[0] * dd[0] + dd[1] * dd[1];
                    let t = if l2 > 0.0 {
                        (((p[0] - q0[0]) * dd[0] + (p[1] - q0[1]) * dd[1]) / l2).clamp(0.0, 1.0)
                    } else {
                        0.0
                    };
                    let w = [q0[0] + t * dd[0] - p[0], q0[1] + t * dd[1] - p[1]];
                    let no = normal(&o);
                    // Os dois de FORA um do outro: o vão está à frente de cada borda.
                    if w[0] * n[0] + w[1] * n[1] <= 0.0 || w[0] * no[0] + w[1] * no[1] >= 0.0 {
                        continue;
                    }
                    let dist = w[0].hypot(w[1]);
                    if melhor.is_none_or(|m| dist < m.2) {
                        melhor = Some((sj, t, dist));
                    }
                }
            }
        }
        melhor
    };
    // Só se amostra o segmento que tem um candidato LONGE nas células à volta dele.
    let tem_candidato = |si: usize| {
        let (p, q) = (pos[segs[si].a], pos[segs[si].b]);
        (cel(p[0].min(q[0])) - 1..=cel(p[0].max(q[0])) + 1).any(|gx| {
            (cel(p[1].min(q[1])) - 1..=cel(p[1].max(q[1])) + 1)
                .any(|gy| na_celula((gx, gy)).any(|sj| longe(si, sj)))
        })
    };
    let uv_em = |s: &Seg, t: f64| {
        let (u0, u1) = (uv[s.a], uv[s.b]);
        #[expect(clippy::cast_possible_truncation, reason = "t em [0, 1]")]
        let t = t as f32;
        [u0[0] + t * (u1[0] - u0[0]), u0[1] + t * (u1[1] - u0[1])]
    };
    let ponto = |s: &Seg, t: f64| {
        let (p, q) = (pos[s.a], pos[s.b]);
        [p[0] + t * (q[0] - p[0]), p[1] + t * (q[1] - p[1])]
    };
    for (si, s) in segs.iter().enumerate() {
        if !tem_candidato(si) {
            continue;
        }
        let (p, q) = (pos[s.a], pos[s.b]);
        let comprimento = (q[0] - p[0]).hypot(q[1] - p[1]);
        // Uma amostra por texel, as duas pontas incluídas.
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "amostras"
        )]
        let n = ((comprimento / texel).ceil() as usize).max(1);
        #[expect(clippy::cast_precision_loss, reason = "amostras")]
        let amostras: Vec<(f64, Option<Encontro>)> = (0..=n)
            .map(|k| {
                let t = k as f64 / n as f64;
                (t, encara(ponto(s, t), si))
            })
            .collect();
        // ⚠️ Cada vão cose-se de UM lado só — o do segmento de índice menor —, senão as duas metades
        // pintavam a mesma faixa duas vezes (uma beira translúcida sairia mais escura).
        let dentro = |m: &Option<Encontro>| m.is_some_and(|(sj, _, d)| sj > si && d < vao);
        for par in amostras.windows(2) {
            let ((t0, m0), (t1, m1)) = (par[0], par[1]);
            let (d0, d1) = (dentro(&m0), dentro(&m1));
            // ⭐ A ponta de um troço é onde o vão REAL passa `vao` — bissecção sobre a distância, e
            // não a amostra nem uma interpolação: é isto que faz a costura crescer e encolher
            // CONTÍNUA com a pose. ⛔ A 1.ª redacção interpolava a distância linearmente entre as
            // amostras, e quando o ponto mais perto muda de segmento entre elas a conta mente: medido,
            // pedaços a coser vãos de `3,9` texels (a lei é `2`).
            let fronteira = |mut t_in: f64, mut t_out: f64| {
                for _ in 0..12 {
                    let t = 0.5 * (t_in + t_out);
                    if dentro(&encara(ponto(s, t), si)) {
                        t_in = t;
                    } else {
                        t_out = t;
                    }
                }
                t_in
            };
            let (ta, tb) = match (d0, d1) {
                (true, true) => (t0, t1),
                (true, false) => (t0, fronteira(t0, t1)),
                (false, true) => (fronteira(t1, t0), t1),
                (false, false) => continue,
            };
            let (pa, pb) = (ponto(s, ta), ponto(s, tb));
            let (Some(fa), Some(fb)) = (encara(pa, si), encara(pb, si)) else {
                continue;
            };
            // ⭐⭐ A5-a: cada METADE do vão estica a cor da SUA beira até ao meio — um quadrilátero
            // só misturava a UV de um membro com a do outro e apanhava a textura entre as duas (uma
            // risca castanha das pintas na cúspide, FOTOGRAFADA).
            let (qa, qb) = (ponto(&segs[fa.0], fa.1), ponto(&segs[fb.0], fb.1));
            let meio = |a: [f64; 2], b: [f64; 2]| [0.5 * (a[0] + b[0]), 0.5 * (a[1] + b[1])];
            let (ma, mb) = (meio(pa, qa), meio(pb, qb));
            let (ua, ub) = (uv_em(s, ta), uv_em(s, tb));
            let (va, vb) = (uv_em(&segs[fa.0], fa.1), uv_em(&segs[fb.0], fb.1));
            let base = out.local.len() as u32;
            for (pt, uv) in [
                (pa, ua),
                (pb, ub),
                (mb, ub),
                (ma, ua),
                (ma, va),
                (mb, vb),
                (qb, vb),
                (qa, va),
            ] {
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "metros locais de uma sprite"
                )]
                out.local.push([pt[0] as f32, pt[1] as f32]);
                out.uv.push(uv);
            }
            for b in [base, base + 4] {
                out.tris.push([b, b + 1, b + 2]);
                out.tris.push([b, b + 2, b + 3]);
            }
        }
    }
    out
}

/// A chave de uma linha de pesos pela COLUNA (`Σ wⱼ·j / Σ wⱼ`) — a da costura.
fn chave_da_coluna(w: &[f64]) -> f64 {
    let soma: f64 = w.iter().sum();
    #[expect(clippy::cast_precision_loss, reason = "índice de osso")]
    let pos: f64 = w.iter().enumerate().map(|(j, p)| p * j as f64).sum();
    if soma > 0.0 { pos / soma } else { 0.0 }
}

/// O sentido de fora: o do anel de maior área diz o de todos (`pos` são os anéis em sequência).
fn sinal_dos_aneis(pos: &[[f64; 2]], tamanhos: impl Iterator<Item = usize>) -> f64 {
    let mut k = 0;
    let mut maior = 0.0_f64;
    for n in tamanhos {
        let area: f64 = (0..n)
            .map(|i| {
                let (p, q) = (pos[k + i], pos[k + (i + 1) % n]);
                p[0] * q[1] - q[0] * p[1]
            })
            .sum();
        if area.abs() > maior.abs() {
            maior = area;
        }
        k += n;
    }
    maior.signum()
}

/// A borda que a costura mede: posições POSADAS, UV, chave e segmentos, e o sentido de fora.
type Borda = (Vec<[f64; 2]>, Vec<[f32; 2]>, Vec<f64>, Vec<Seg>, f64);

/// ⭐⭐⭐ **A borda posada sobre o ANEL DA ARTE** (A5-a) — cada ponto é a mistura baricêntrica dos
/// vértices POSADOS do seu triângulo, posados pela MESMA porta da malha: ele está onde a imagem
/// desenha aquele texel, com a UV dele.
#[expect(clippy::too_many_arguments, reason = "a porta da malha mais o anel")]
fn borda_da_arte(
    mesh: &Mesh2d,
    arte: &[Vec<PontoDaArte>],
    p2l: Xform,
    pele: &Skin,
    pesos: &[f64],
    [anchor, size]: [[f32; 2]; 2],
    correcoes: &[Correccao],
    ossos: usize,
) -> Option<Borda> {
    let mut ids: Vec<u32> = arte
        .iter()
        .flatten()
        .flat_map(|p| mesh.tris[p.tri as usize])
        .collect();
    ids.sort_unstable();
    ids.dedup();
    let linha = |v: u32| {
        pesos
            .get(v as usize * ossos..(v as usize + 1) * ossos)
            .unwrap_or(&[])
    };
    let linhas: Vec<f64> = ids.iter().flat_map(|&v| linha(v).iter().copied()).collect();
    let so = Mesh2d {
        rest: ids.iter().map(|&v| mesh.rest[v as usize]).collect(),
        tris: Vec::new(),
        size: mesh.size,
    };
    let posada = crate::skin_image::posed_sprite_mesh_corrigida(
        so, p2l, pele, &linhas, anchor, size, correcoes,
    )?;
    let (mut pos, mut uv, mut chave, mut segs) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for anel in arte {
        let (base, n) = (pos.len(), anel.len());
        for (i, p) in anel.iter().enumerate() {
            let t = mesh.tris[p.tri as usize];
            let w = [1.0 - p.uv[0] - p.uv[1], p.uv[0], p.uv[1]];
            let k = [0, 1, 2].map(|j| ids.binary_search(&t[j]).ok());
            let [Some(k0), Some(k1), Some(k2)] = k else {
                return None;
            };
            let ks = [k0, k1, k2];
            pos.push([0, 1].map(|c| {
                (0..3)
                    .map(|j| w[j] * f64::from(posada.local[ks[j]][c]))
                    .sum()
            }));
            // A COR: o ponto mais para dentro da tinta ([`PontoDaArte::cor`]).
            #[expect(clippy::cast_possible_truncation, reason = "UV em f32 como a da malha")]
            uv.push([0, 1].map(|c| (p.cor[c] / f64::from(mesh.size[c])) as f32));
            let row: Vec<f64> = (0..ossos)
                .map(|o| {
                    (0..3)
                        .map(|j| w[j] * linha(t[j]).get(o).copied().unwrap_or(0.0))
                        .sum()
                })
                .collect();
            chave.push(chave_da_coluna(&row));
            segs.push(Seg {
                a: base + i,
                b: base + (i + 1) % n,
                antes: base + (i + n - 1) % n,
                depois: base + (i + 1) % n,
            });
        }
    }
    let sinal = sinal_dos_aneis(&pos, arte.iter().map(Vec::len));
    Some((pos, uv, chave, segs, sinal))
}

#[cfg(test)]
#[path = "skin_image_fecho_tests.rs"]
mod tests;
