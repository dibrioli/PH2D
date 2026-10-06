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
//! # ⭐⭐⭐ A5-a: o vão mede-se na TINTA, e cada triângulo cosido entra à profundidade do SEU membro
//!
//! Com a máscara do bind a costura mede e cose sobre o anel da arte
//! ([`crate::skin_image_arte::anel_da_arte`], a marcha axial) e não sobre a borda da malha, que passa
//! da tinta. Cada triângulo cosido entra logo depois do triângulo de onde vem a sua beira: a metade do
//! membro de trás fica por baixo da tinta do da frente. Medições: fila §F59 e o commit da lei.
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

use crate::skin_image_arte::BordasDaMalha;
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
    let costura = crate::skin_image_costura::costura(
        &mesh,
        bordas,
        p2l,
        pele,
        pesos,
        [anchor, size],
        correcoes,
    );
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
    let novos: Vec<[u32; 3]> = costura.tris.iter().map(|t| t.map(|i| i + base)).collect();
    crate::skin_image_costura::junta(&mut malha.tris, &novos, &costura.slot);
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
        let bordas = Rc::new(BordasDaMalha::da(&malha.mesh, malha.mascara.as_ref()));
        b.insert(chave, (Rc::downgrade(malha), Rc::clone(&bordas)));
        bordas
    })
}

#[cfg(test)]
#[path = "skin_image_fecho_tests.rs"]
mod tests;
