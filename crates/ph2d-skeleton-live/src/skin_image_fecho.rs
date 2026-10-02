//! ⭐⭐⭐ **O FECHO DA IMAGEM PRESA** — a 2.ª mídia sob a lei da silhueta da pele (o aberto do
//! [handoff de 2026-10-01](../../../docs/Skeleton/handoffs/HANDOFF_INTEGRACAO_line_Vector_A_SILHUETA_DA_PELE_2026-10-01.md)
//! §6: *«a IMAGEM presa mostra um risquinho no encontro dos membros»*).
//!
//! # O defeito, medido (cena `PH2D_VEC_BONE_SMOKE=4` a `(36°, −144°)`)
//!
//! O risquinho é a cor do FUNDO, pura (não uma mistura): **um vão entre dois membros**, e não uma
//! fenda da malha — `0` nós pendurados em `9 091` triângulos, e os triângulos dos dois lados de cada
//! pixel descoberto ficam a `~350 px` um do outro no repouso. A borda de cima do membro de baixo e a
//! do membro dobrado de volta quase se encostam e deixam uma cunha de `~20 px²`. No DESENHO a mesma
//! cunha é fechada pela bola da [`ph2d_vec_boolean::silhueta_da_pele`]; a imagem não passava por lei
//! nenhuma.
//!
//! # A lei: a MESMA bola, sobre a borda da malha posada
//!
//! A borda da malha (as arestas de um só triângulo) posada pela porta de sempre é um contorno; o
//! [`ph2d_vec_boolean::fecho_da_borda`] devolve o que o fecho lhe ACRESCENTA, e isso é triangulado
//! e desenhado a seguir à malha. ⚠️ **A UV de um ponto acrescentado é a do
//! ponto mais perto na borda** — a tinta da beira estica-se para dentro do vão, que é o que o
//! preenchimento de um desenho faz com a cor dele. Numa arte de margem transparente a beira da malha
//! é transparente e o enchimento não pinta nada: o «V» da ARTE fica, o vão entre bordas OPACAS fecha.
//!
//! ⚠️ **Um quadro com enchimento posa na CPU**: o enchimento nasce no espaço POSADO e a malha da placa
//! traz o repouso. Sem enchimento a placa posa como sempre, ao bit.
//!
//! ⛔⛔ **E o fecho nasce DESLIGADO** ([`lei_do_fecho_da_imagem_activa`], `PH2D_SKIN_FECHO_IMAGEM=1`
//! liga) — o smoke do dono de 2026-10-02 reprovou-o: *«queda de FPS»* e *«ora redonda ora pontuda»*.
//! Medido: a bola sobre a borda CRUA da malha (`279` nós, sem contacto, `(40°, 40°)`) custa `57 ms`
//! por imagem por quadro, contra `0,19 ms` da malha sem ele; e a escada da grelha na margem
//! transparente é parede para a bola (`(36°, −141,5°)…(36°, −148,5°)` sem fecho nenhum). A cura é o
//! fecho sobre o CONTORNO DA ARTE, com orçamento medido — ver o handoff F48 §3b.
//!
//! # ⭐⭐ A ORDEM DAS FACES: o osso mais adiante na corrente pinta por cima
//!
//! Ordem do dono (2026-10-02): *«as faces influenciadas por um osso têm z-index aleatório, e ao se
//! sobrepor às do outro osso misturam-se; melhor seria as do último osso por cima»*. A ordem dos
//! triângulos É a ordem do desenho (as duas portas, placa e CPU), e a da grelha é a das CÉLULAS — na
//! dobra forte os pedaços dos dois membros intercalavam-se. ⇒ [`ordena_pelo_osso`], UMA vez por bind,
//! na gaveta da malha desenhada ([`crate::skin_bake_cache::assada_da_arte`]).

use ph2d_poly2d::Mesh2d;
use ph2d_render::SpriteMesh;
use ph2d_skeleton::{Correccao, Skin, Xform};
use ph2d_vec_scene::{Contour, VecPath, VecVertex};

/// ⭐⭐ **Um vão mais fino que `1/32` de TEXEL não é vão** — a espessura média (`2·área/perímetro`)
/// abaixo da qual o que o fecho acrescenta é ruído da bola sobre uma polilinha, e sai.
///
/// ⚠️ A régua de uma IMAGEM é o pixel DELA: um pedaço mais fino que `1/32` de texel não muda a
/// cobertura de pixel nenhum até `3 200 %` de zoom. Medido na cena `=4` (texel = `0,01 m`):
///
/// | pose | pedaços | espessura média | |
/// |---|---|---|---|
/// | `(40°, 40°)` | 5 | `0,0008`–`0,0031` texel | ruído da bola sobre a polilinha, `0,007 px²` |
/// | `(36°, −144°)` | 1 | `0,79` texel | o risquinho do report (`20,3 px²`) |
/// | `(120°, 120°)` | 2 | `0,12` e `0,88` texel | os bicos dos dois «V» (`13,2 px²`) |
///
/// ⇒ `1/32 = 0,031` fica `10×` acima do ruído e `4×` abaixo do vão mais fino. ⛔ Sem ele, a pose
/// sem contacto acrescentava `40` triângulos invisíveis e tirava a malha à PLACA.
pub const ESPESSURA_MINIMA_EM_TEXELS: f64 = 1.0 / 32.0;

/// Em quantas cordas se achata uma cúbica do fecho — os arcos da bola não passam de `90°`, e a `8`
/// a flecha é `0,5 %` do raio.
const CORDAS_POR_CURVA: usize = 8;

/// ⭐⭐⭐ **A malha que o quadro desenha** — a porta do [`crate::skin_image::attach_skin_meshes`].
#[must_use]
pub fn malha_desenhada(
    mesh: Mesh2d,
    p2l: Xform,
    pele: &Skin,
    pesos: &[f64],
    anchor: [f32; 2],
    size: [f32; 2],
    correcoes: &[Correccao],
) -> Option<SpriteMesh> {
    malha_desenhada_com(
        mesh,
        p2l,
        pele,
        pesos,
        [anchor, size],
        correcoes,
        lei_do_fecho_da_imagem_activa(),
        crate::skin_image_gpu::a_placa_posa(),
    )
}

/// `PH2D_SKIN_FECHO_IMAGEM=1` liga o fecho da imagem — ver o cabeçalho (nasce desligado).
#[must_use]
pub fn lei_do_fecho_da_imagem_activa() -> bool {
    fecho_da_imagem_de(std::env::var("PH2D_SKIN_FECHO_IMAGEM").ok().as_deref())
}

/// A leitura da porta, PURA — desligada salvo `"1"`. ⚠️ É ela que o gate mede.
#[must_use]
pub fn fecho_da_imagem_de(valor: Option<&str>) -> bool {
    valor == Some("1")
}

/// ⭐⭐ **Ordena os triângulos pelo OSSO que os move** — cada face desenha-se depois das de um osso
/// anterior da corrente.
///
/// A chave de um vértice é a posição MÉDIA, pesada, dos ossos da tabela (`Σ wⱼ·j / Σ wⱼ`; as colunas
/// vêm na ordem do [`crate::skin_live::skeleton_of`], que desce da raiz) e a de um triângulo é a média
/// dos três. ⚠️ Uma média e não o osso dominante: a face de uma zona de mistura fica ENTRE os dois
/// membros, e a ordem não dá um salto onde o peso cruza `0,5`. A ordenação é ESTÁVEL — faces do
/// mesmo osso mantêm a ordem da grelha. Sem tabela (a lei derivada) a ordem fica como está.
pub fn ordena_pelo_osso(tris: &mut [[u32; 3]], pesos: &[f64], vertices: usize) {
    let ossos = pesos.len() / vertices.max(1);
    if ossos < 2 || pesos.len() != ossos * vertices {
        return;
    }
    let chave: Vec<f64> = pesos
        .chunks_exact(ossos)
        .map(|w| {
            let soma: f64 = w.iter().sum();
            #[expect(clippy::cast_precision_loss, reason = "índice de osso")]
            let pos: f64 = w.iter().enumerate().map(|(j, p)| p * j as f64).sum();
            if soma > 0.0 { pos / soma } else { 0.0 }
        })
        .collect();
    let de = |t: &[u32; 3]| {
        t.iter()
            .map(|&v| chave.get(v as usize).copied().unwrap_or(0.0))
            .sum::<f64>()
    };
    tris.sort_by(|a, b| de(a).total_cmp(&de(b)));
}

/// A [`malha_desenhada`] com as duas portas do ambiente escolhidas — a dos gates.
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
    contacto: bool,
    placa: bool,
) -> Option<SpriteMesh> {
    let fecho = if contacto {
        enchimento(&mesh, p2l, pele, pesos, [anchor, size], correcoes)
    } else {
        None
    };
    // ⭐⭐⭐ **QUEM POSA: a PLACA, por omissão** (F9 W2, 2026-09-20). Os dois caminhos entregam um
    // `SpriteMesh`; a diferença é se o `local` traz o POSADO (a CPU, a referência) ou o REPOUSO mais
    // a tabela que o `vs_main` lê. ⚠️ `PH2D_SKIN_GPU=0` bissecta. ⛔ A lei que decide (a medição do
    // §0.0) mora no cabeçalho do [`crate::skin_image_gpu`].
    let construtor = if placa && fecho.is_none() {
        crate::skin_image_gpu::sprite_mesh_para_a_placa
    } else {
        crate::skin_image::posed_sprite_mesh_corrigida
    };
    let mut malha = construtor(mesh, p2l, pele, pesos, anchor, size, correcoes)?;
    if let Some(f) = fecho {
        let base = u32::try_from(malha.local.len()).ok()?;
        malha.local.extend(f.local);
        malha.uv.extend(f.uv);
        malha
            .tris
            .extend(f.tris.into_iter().map(|t| t.map(|i| i + base)));
    }
    Some(malha)
}

/// O que o fecho acrescenta: pontos POSADOS, a UV de cada um e os triângulos.
struct Enchimento {
    local: Vec<[f32; 2]>,
    uv: Vec<[f32; 2]>,
    tris: Vec<[u32; 3]>,
}

/// ⭐⭐ **Os anéis da borda** — as arestas que só UM triângulo usa, encadeadas pela orientação dele
/// (o de fora num sentido, os buracos no outro: a regra não-zero lê-os sem mais nada).
#[must_use]
pub fn aneis_da_borda(tris: &[[u32; 3]]) -> Vec<Vec<u32>> {
    let mut arestas: Vec<(u32, u32)> = tris
        .iter()
        .flat_map(|t| [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])])
        .collect();
    arestas.sort_unstable();
    let partilhada = |a: u32, b: u32| arestas.binary_search(&(b, a)).is_ok();
    let mut seguinte: std::collections::BTreeMap<u32, u32> = arestas
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

/// A viragem em `a`, em graus, de `p → a → n`.
fn viragem(p: [f64; 2], a: [f64; 2], n: [f64; 2]) -> f64 {
    let (u, v) = ([a[0] - p[0], a[1] - p[1]], [n[0] - a[0], n[1] - a[1]]);
    (u[0] * v[1] - u[1] * v[0])
        .atan2(u[0] * v[0] + u[1] * v[1])
        .abs()
        .to_degrees()
}

/// ⭐⭐⭐ **O que o fecho acrescenta a esta malha nesta pose** — `None` quando nada.
fn enchimento(
    mesh: &Mesh2d,
    p2l: Xform,
    pele: &Skin,
    pesos: &[f64],
    [anchor, size]: [[f32; 2]; 2],
    correcoes: &[Correccao],
) -> Option<Enchimento> {
    let aneis = aneis_da_borda(&mesh.tris);
    let ids: Vec<u32> = aneis.iter().flatten().copied().collect();
    if ids.is_empty() {
        return None;
    }
    // ⭐ **A borda posa-se pela PORTA da malha** — uma malha só de borda, sem triângulos, com a
    // tabela das mesmas linhas: uma segunda conta da pose divergiria da que a arte desenha.
    let ossos = pesos.len() / mesh.rest.len().max(1);
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
    let borda = crate::skin_image::posed_sprite_mesh_corrigida(
        so_borda, p2l, pele, &linhas, anchor, size, correcoes,
    )?;
    let pos = |k: usize| [f64::from(borda.local[k][0]), f64::from(borda.local[k][1])];
    // O contorno posado e as QUINAS de repouso (a lei da F42): a viragem que cada nó tem no bind.
    let mut quinas = Vec::new();
    let mut contornos = Vec::new();
    let mut segmentos = Vec::new();
    let mut k0 = 0;
    for anel in &aneis {
        let n = anel.len();
        let verts = (0..n)
            .map(|i| {
                let r = |j: usize| mesh.rest[anel[j % n] as usize];
                let vira = viragem(r(i + n - 1), r(i), r(i + 1));
                if vira > ph2d_vec_boolean::overlap::PAREDE_MINIMA {
                    quinas.push((pos(k0 + i), vira));
                }
                segmentos.push((k0 + i, k0 + (i + 1) % n));
                VecVertex::corner(pos(k0 + i))
            })
            .collect();
        contornos.push(verts);
        k0 += n;
    }
    let mut contornos = contornos.into_iter();
    let caminho = VecPath {
        verts: contornos.next()?,
        closed: true,
        subpaths: contornos.map(Contour::new_closed).collect(),
        ..VecPath::default()
    };
    let pecas = ph2d_vec_boolean::fecho_da_borda(&caminho, &quinas);
    if pecas.is_empty() {
        return None;
    }
    // O lado de um texel no espaço local: a raiz do determinante da régua `pixel → local`.
    let [a, b, c, d, _, _] = p2l.0;
    let texel = (a * d - b * c).abs().sqrt();
    let mut out = Enchimento {
        local: Vec::new(),
        uv: Vec::new(),
        tris: Vec::new(),
    };
    for peca in pecas {
        if !peca.subpaths.is_empty() {
            avisa("um vao com um buraco dentro");
            continue;
        }
        let anel = limpa_o_anel(achata(&peca.verts));
        let perimetro: f64 = (0..anel.len())
            .map(|i| {
                let (a, b) = (anel[i], anel[(i + 1) % anel.len()]);
                (b[0] - a[0]).hypot(b[1] - a[1])
            })
            .sum();
        let espessura =
            2.0 * ph2d_poly2d::signed_area(&anel).abs() / perimetro.max(f64::MIN_POSITIVE);
        // ⚠️ **O diagnóstico da família** (`PH2D_BONE_LOG=1`): cada vão, com a espessura em texels.
        if std::env::var_os("PH2D_BONE_LOG").is_some() {
            #[expect(clippy::cast_precision_loss, reason = "um anel de poucos pontos")]
            let n = anel.len() as f64;
            eprintln!(
                "[bone] fecho da imagem: vao de {:.4} texel de espessura media, {:.2} texel² em \
                 ({:.3}, {:.3})",
                espessura / texel,
                ph2d_poly2d::signed_area(&anel).abs() / (texel * texel),
                anel.iter().map(|p| p[0]).sum::<f64>() / n,
                anel.iter().map(|p| p[1]).sum::<f64>() / n,
            );
        }
        if espessura < ESPESSURA_MINIMA_EM_TEXELS * texel {
            continue;
        }
        let Some(tris) = ph2d_poly2d::triangulate(&anel) else {
            if std::env::var_os("PH2D_BONE_LOG").is_some() {
                eprintln!("[bone] fecho da imagem: SEM triangulacao, anel {anel:?}");
            }
            avisa("um vao sem triangulacao");
            continue;
        };
        let base = u32::try_from(out.local.len()).ok()?;
        for &q in &anel {
            out.uv.push(uv_da_borda(q, &segmentos, &pos, &borda.uv));
            #[expect(
                clippy::cast_possible_truncation,
                reason = "metros locais de uma sprite"
            )]
            out.local.push([q[0] as f32, q[1] as f32]);
        }
        out.tris
            .extend(tris.into_iter().map(|t| t.map(|i| i + base)));
    }
    (!out.tris.is_empty()).then_some(out)
}

/// O anel de `verts` achatado — uma corda por recta, [`CORDAS_POR_CURVA`] por cúbica.
fn achata(verts: &[VecVertex]) -> Vec<[f64; 2]> {
    let mut out = Vec::new();
    for i in 0..verts.len() {
        let (a, b) = (&verts[i], &verts[(i + 1) % verts.len()]);
        out.push(a.anchor);
        if a.out_handle == a.anchor && b.in_handle == b.anchor {
            continue;
        }
        for k in 1..CORDAS_POR_CURVA {
            #[expect(clippy::cast_precision_loss, reason = "k < 8")]
            let t = k as f64 / CORDAS_POR_CURVA as f64;
            let u = 1.0 - t;
            let c = [u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t];
            let p = [a.anchor, a.out_handle, b.in_handle, b.anchor];
            out.push([
                c.iter().zip(&p).map(|(c, p)| c * p[0]).sum(),
                c.iter().zip(&p).map(|(c, p)| c * p[1]).sum(),
            ]);
        }
    }
    out
}

/// ⭐⭐ **O anel sem os nós de área ZERO** — repetidos, e os que ficam numa recta com os vizinhos,
/// incluindo os que voltam por ela (um FIO que sai e regressa).
///
/// ⛔ Report do dono de 2026-10-02 (*«ora redonda ora pontuda»*): onde o enchimento encosta na borda,
/// a subtracção pode devolver o vão com um fio pela aresta da malha; a triangulação recusa o anel e
/// o bico saía em ponta naquela pose — a `(36°, −131,25°)`, com os vizinhos a `0,25°` redondos.
/// ⚠️ A área não muda: um fio não tem largura.
fn limpa_o_anel(mut anel: Vec<[f64; 2]>) -> Vec<[f64; 2]> {
    // Relativo ao comprimento dos dois lados: `1e-9` fica milhões de vezes abaixo de uma curva
    // achatada (`1,6e-6` no anel medido) e acima do arredondamento da recta (`1e-18`).
    const RECTA: f64 = 1e-9;
    let mut i = 0;
    while anel.len() > 3 && i < anel.len() {
        let n = anel.len();
        let (p, a, q) = (anel[(i + n - 1) % n], anel[i], anel[(i + 1) % n]);
        let (u, v) = ([a[0] - p[0], a[1] - p[1]], [q[0] - a[0], q[1] - a[1]]);
        let (lu, lv) = (u[0].hypot(u[1]), v[0].hypot(v[1]));
        if lu == 0.0 || lv == 0.0 || (u[0] * v[1] - u[1] * v[0]).abs() <= RECTA * lu * lv {
            anel.remove(i);
            // ⚠️ Recua: tirar este nó pode pôr o anterior numa recta (o fio desfaz-se de fora para dentro).
            i = i.saturating_sub(1);
        } else {
            i += 1;
        }
    }
    anel
}

/// A UV do ponto da borda mais perto de `q` — a tinta da beira.
fn uv_da_borda(
    q: [f64; 2],
    segmentos: &[(usize, usize)],
    pos: &impl Fn(usize) -> [f64; 2],
    uv: &[[f32; 2]],
) -> [f32; 2] {
    let mut melhor = (f64::INFINITY, [0.0_f32; 2]);
    for &(i, j) in segmentos {
        let (a, b) = (pos(i), pos(j));
        let d = [b[0] - a[0], b[1] - a[1]];
        let l2 = d[0] * d[0] + d[1] * d[1];
        let t = if l2 > 0.0 {
            (((q[0] - a[0]) * d[0] + (q[1] - a[1]) * d[1]) / l2).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let dist = (q[0] - a[0] - t * d[0]).hypot(q[1] - a[1] - t * d[1]);
        if dist < melhor.0 {
            #[expect(clippy::cast_possible_truncation, reason = "t em [0, 1]")]
            let t = t as f32;
            melhor = (
                dist,
                [
                    uv[i][0] + t * (uv[j][0] - uv[i][0]),
                    uv[i][1] + t * (uv[j][1] - uv[i][1]),
                ],
            );
        }
    }
    melhor.1
}

/// ⚠️ Um vão que o fecho devolve e que não se consegue desenhar fica por pintar — uma vez por sessão.
fn avisa(porque: &str) {
    static AVISADO: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if !AVISADO.swap(true, std::sync::atomic::Ordering::Relaxed) {
        eprintln!("[bone] o fecho de uma imagem presa deixou {porque} por pintar");
    }
}

#[cfg(test)]
#[path = "skin_image_fecho_tests.rs"]
mod tests;
