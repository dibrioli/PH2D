//! Os gates da cena do osso que pinta — a régua da arte contra a lei da pele.
//!
//! ⚠️ **Ele é um ficheiro e não um `mod` em linha**, e a razão é o tecto de LOC: a cena, os gates e
//! a bancada são três assuntos, e o irmão [`bancada`] precisa de ser filho DESTE módulo para herdar
//! as fixturas — um `#[path]` dentro de um `mod` em linha resolve contra um directório que não
//! existe.

use super::*;

use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_render::Sprite;

/// O `pixels_per_meter` de omissão do projecto — o mesmo que a cena recebe.
const PPM: f32 = 100.0;

/// A escala da câmera do smoke, em pixels de ECRÃ por metro de mundo — medida no ecrã em
/// 2026-09-15. É ela que converte as réguas de metros para o que o olho vê.
const PX_POR_METRO: f64 = 152.0;

/// **A CENA MONTADA, pelas portas do PRODUTO** — devolve `(mundo, entidade da arte)`.
///
/// ⚠️ Ela monta a corrente com a [`super::eixos`] (a mesma que a [`super::build`] usa), escreve
/// a força pela [`super::forca_do_osso`], prende pelo `bind_image` e dobra pela [`super::dobra`]
/// — ⛔ **nenhuma cinemática escrita aqui**. Uma segunda cadeia de transformações neste ficheiro
/// seria uma segunda resposta à mesma pergunta, e mediria uma cena que o dono não vê.
///
/// `altura_px` e `forca` são parâmetros para os gates de CONTROLO poderem pedir exactamente as
/// duas redacções que o dono reprovou.
fn cena(altura_px: u32, forca: Option<f64>) -> (SimWorld, Entity) {
    cena_com(altura_px, forca, ph2d_poly2d::GridOptions::default())
}

/// A [`cena`] com a grelha do bind escolhida — só a sonda da densidade a usa com outra.
fn cena_com(
    altura_px: u32,
    forca: Option<f64>,
    grelha: ph2d_poly2d::GridOptions,
) -> (SimWorld, Entity) {
    cena_dobrada(altura_px, forca, grelha, super::DOBRA_GRAUS)
}

/// ⭐⭐⭐ **A [`cena`] com o ÂNGULO da dobra escolhido** — e ela existe por um defeito de RÉGUA.
///
/// ⛔⛔ **Repor um ÂNGULO não é repor uma POSE.** A 1.ª redacção da bancada varria a dobra
/// escrevendo `t.rotation = graus` nos ossos de uma cena **já dobrada**, e a `graus = 0` leu
/// `1,426` de esticão máximo — sobre um mapa que em repouso é a identidade. O controlo que a
/// desmascarou é o que ficou: uma cena que **nunca** dobra lê `4,97e-16 m` de desvio e as três
/// poses saem `[1,0,0,1,0,0]` exactas. *A pose de repouso de um osso filho não é «rotação zero»:
/// ela é o que a `bone::create` escreveu, e escrever zero por cima é outra pose.*
fn cena_dobrada(
    altura_px: u32,
    forca: Option<f64>,
    grelha: ph2d_poly2d::GridOptions,
    graus: f32,
) -> (SimWorld, Entity) {
    let mut sim = SimWorld::default();
    // ⭐⭐ **A PORTA DO PRODUTO**, e é ela que escreve o alcance: um `None` aqui mede a cena tal
    // como o dono a vê. Ver o doc da [`super::corrente`] — a mutação que a criou.
    let ossos = super::corrente(&mut sim, PPM).expect("a corrente monta");
    if let Some(f) = forca {
        for &ent in &ossos {
            if let Some(mut bone) = sim.world_mut().get_mut::<ph2d_skeleton_ecs::Bone>(ent) {
                bone.strength = f;
            }
        }
    }
    // A sprite que a `spawn_rgba` cria: o tamanho em metros é `pixels / ppm`, âncora ao centro.
    let tamanho = [super::LARGURA_PX as f32 / PPM, altura_px as f32 / PPM];
    let e = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Sprite::atlas(0, tamanho, [1.0, 1.0, 1.0, 1.0]),
        ))
        .id();
    assert!(
        ph2d_skeleton_live::skin_live::bind_image(
            &mut sim,
            e,
            &super::branco(super::LARGURA_PX, altura_px),
            [super::LARGURA_PX, altura_px],
            PPM,
            grelha,
            ossos.first().copied(),
        ),
        "o bind tinha de acontecer: ha' osso, ha' tinta e a pose nao e' singular"
    );
    // ⚠️ SOMA a partir da pose de repouso, que é o que a [`super::dobra`] faz.
    for osso in ossos.iter().skip(1) {
        if let Some(mut t) = sim.world_mut().get_mut::<Transform>(*osso) {
            t.rotation += graus.to_radians();
        }
    }
    (sim, e)
}

/// A dobra da pele — as quatro colunas da [`ph2d_skeleton::fold`].
fn dobra_da_cena(altura_px: u32, forca: Option<f64>) -> ph2d_skeleton::fold::FoldReport {
    let (sim, e) = cena(altura_px, forca);
    let pele = ph2d_skeleton_live::skin_live::skin_of(&sim, e).expect("a pele resolve");
    let (mx, my) = (
        f64::from(super::LARGURA_PX) / f64::from(PPM) / 2.0,
        f64::from(altura_px) / f64::from(PPM) / 2.0,
    );
    ph2d_skeleton::fold::measure(&pele, [-mx, -my, mx, my], 65)
}

/// ⭐⭐⭐ **O DESVIO DA FACETA, em pixels de ECRÃ** — quanto o afim de cada triângulo erra o campo.
///
/// ⚠️ É a régua do PRODUTO (`ph2d_poly2d::deviation`, a mesma que o `Smooth` consulta), sobre a
/// malha que a cena de facto guarda e o campo que ela de facto aplica.
fn faceta_da_cena(altura_px: u32, forca: Option<f64>) -> f64 {
    faceta_com(altura_px, forca, true)
}

/// [`faceta_da_cena`] com a lei escolhida — `false` mede a EUCLIDIANA derivada, que é o que as
/// duas redacções reprovadas de facto corriam.
fn faceta_com(altura_px: u32, forca: Option<f64>, usar_pesos: bool) -> f64 {
    let (sim, e) = cena(altura_px, forca);
    let (mut sm, p2l, pele) = campo_da_cena(&sim, e);
    if !usar_pesos {
        sm.pesos.clear();
    }
    let posadas = posadas_da_cena(&sm, p2l, &pele);
    let mut w2 = pele.scratch();
    let d = ph2d_poly2d::deviation_attrs(
        &sm.mesh,
        &posadas,
        &sm.pesos,
        sm.ossos(),
        &mut |q, pesos| ponto_do_produto(&pele, p2l.apply(q), pesos, &mut w2),
    );
    d * PX_POR_METRO
}

/// ⭐⭐⭐ **A LEI QUE O PRODUTO CORRE, numa porta só** — com a tabela guardada quando ela existe.
///
/// ⛔⛔ **Ela existe por um defeito que esta wave quase shipou:** as réguas desta cena chamavam
/// `pele.point`, que é a lei EUCLIDIANA derivada. Desde que a pele de imagem passou ao
/// padrão-ouro isso mede um programa que ninguém corre — a arte no ecrã responde aos pesos
/// GUARDADOS. *Um arnês que não usa a lei do produto mede outro programa, e o smoke reprovado
/// é a única coisa que o diz.*
fn ponto_do_produto(
    pele: &ph2d_skeleton::Skin,
    p: [f64; 2],
    pesos: &[f64],
    w: &mut [f64],
) -> [f64; 2] {
    if pesos.is_empty() {
        pele.point(p, w)
    } else {
        pele.point_with(p, pesos, w)
    }
}

/// A malha guardada, a régua `pixel → local` e a pele desta cena.
fn campo_da_cena(
    sim: &SimWorld,
    e: Entity,
) -> (
    ph2d_skeleton_live::skinned_mesh::SkinnedMesh,
    ph2d_skeleton::Xform,
    ph2d_skeleton::Skin,
) {
    let sm = ph2d_skeleton_live::skin_image::skinned_mesh_of(sim, e).expect("malha guardada");
    let (p2l, pele) =
        ph2d_skeleton_live::skin_image::deform_field(sim, e, sm.mesh.size, PPM).expect("campo");
    (sm, p2l, pele)
}

/// Os vértices da malha guardada, já onde o desenho os põe.
fn posadas_da_cena(
    sm: &ph2d_skeleton_live::skinned_mesh::SkinnedMesh,
    p2l: ph2d_skeleton::Xform,
    pele: &ph2d_skeleton::Skin,
) -> Vec<[f64; 2]> {
    let mut w = pele.scratch();
    sm.mesh
        .rest
        .iter()
        .enumerate()
        .map(|(v, &q)| ponto_do_produto(pele, p2l.apply(q), sm.pesos_de(v), &mut w))
        .collect()
}

/// ⭐⭐⭐ **O VAZAMENTO: rodar só a PONTA da corrente e ver quanto a arte da RAIZ se mexe**, em
/// pixels de ecrã.
///
/// ⛔⛔⛔ **É a régua que nenhuma das outras substitui, e a que reverteu o veredito desta
/// jornada.** Faceta, esticão e círculo medem SUAVIDADE — e uma mistura larga de mais ganha nas
/// três **por construção**, porque fazer toda a arte responder a todos os ossos é suavíssimo.
/// Foi por isso que a `strength = 2,0` pareceu a melhor lei durante um dia inteiro: ela não é
/// um rig, é um borrão global, e rodar a ponta arrasta a raiz `26 px`.
///
/// ⚠️ **A ponta sai da porta do produto** (`chain_ends`), nunca do maior `to_bits`: a 1.ª
/// redacção desta régua escolheu por bits, rodou a RAIZ, e leu `154 px` de vazamento **sobre a
/// lei que não pode vazar**. *O controlo que a desmascarou é o mesmo que fica no gate.*
fn vazamento_da_cena(forca: Option<f64>) -> f64 {
    vazamento_com(forca, true)
}

/// [`vazamento_da_cena`] com a lei escolhida — `false` mede a EUCLIDIANA derivada, que é a
/// coluna de comparação da bancada.
fn vazamento_com(forca: Option<f64>, usar_pesos: bool) -> f64 {
    let pose = |extra: f32| -> Vec<[f64; 2]> {
        let (mut sim, e) = cena(super::ALTURA_PX, forca);
        if extra != 0.0 {
            let pontas = ph2d_skeleton_live::skin_live::chain_ends(&sim);
            let ponta = Entity::from_bits(*pontas.last().expect("ha' ponta de corrente"));
            if let Some(mut t) = sim.world_mut().get_mut::<Transform>(ponta) {
                t.rotation += extra;
            }
        }
        let (sm, p2l, pele) = campo_da_cena(&sim, e);
        if usar_pesos {
            return posadas_da_cena(&sm, p2l, &pele);
        }
        let mut w = pele.scratch();
        sm.mesh
            .rest
            .iter()
            .map(|&q| pele.point(p2l.apply(q), &mut w))
            .collect()
    };
    let antes = pose(0.0);
    let depois = pose(0.4);
    let (sim, e) = cena(super::ALTURA_PX, forca);
    let (sm, _, _) = campo_da_cena(&sim, e);
    // A RAIZ: o terço esquerdo da arte, que o ÚLTIMO osso não devia tocar.
    let limite = f64::from(super::LARGURA_PX) / 3.0;
    let mut pior = 0.0_f64;
    for (v, r) in sm.mesh.rest.iter().enumerate() {
        if r[0] > limite {
            continue;
        }
        pior =
            pior.max((antes[v][0] - depois[v][0]).hypot(antes[v][1] - depois[v][1]) * PX_POR_METRO);
    }
    pior
}

/// ⭐ **A PARTIÇÃO DA UNIDADE da malha guardada** — o pior `|Σ pesos − 1|` e quantos vértices
/// ficaram sem dono nenhum.
fn particao_da_cena(altura_px: u32, forca: Option<f64>) -> (f64, usize) {
    let (sim, e) = cena(altura_px, forca);
    let (sm, _, _) = campo_da_cena(&sim, e);
    let mut pior = 0.0_f64;
    let mut sem_dono = 0usize;
    for v in 0..sm.mesh.rest.len() {
        let pesos = sm.pesos_de(v);
        if pesos.is_empty() {
            sem_dono += 1;
            continue;
        }
        let soma: f64 = pesos.iter().sum();
        pior = pior.max((soma - 1.0).abs());
    }
    (pior, sem_dono)
}

/// ⭐ **QUANTO UMA CIRCUNFERÊNCIA DESENHADA AO CENTRO SAI FORA DE REDONDO** — `r_max / r_min`.
///
/// ⚠️ É o que o dono de facto olha: ele arrasta uma *Ellipse* e julga a forma dela. O raio é
/// `40 %` da altura, que é o tamanho da circunferência da foto de 2026-09-15.
fn circulo_fora_de_redondo(altura_px: u32, forca: Option<f64>) -> f64 {
    circulo_de(cena(altura_px, forca), altura_px)
}

/// O círculo com a dobra escolhida — a coluna da bancada.
fn circulo_dobrado(graus: f32) -> f64 {
    circulo_de(
        cena_dobrada(
            super::ALTURA_PX,
            None,
            ph2d_poly2d::GridOptions::default(),
            graus,
        ),
        super::ALTURA_PX,
    )
}

fn circulo_de((sim, e): (SimWorld, Entity), altura_px: u32) -> f64 {
    let (sm, p2l, pele) = campo_da_cena(&sim, e);
    let mut w = pele.scratch();
    let centro = [
        f64::from(super::LARGURA_PX) / 2.0,
        f64::from(altura_px) / 2.0,
    ];
    let r = f64::from(altura_px) * 0.4;
    let n = 720;
    // ⚠️ **Os pesos de um ponto QUALQUER da arte saem do triângulo que o contém** — é o mesmo
    // baricêntrico que o refinamento usa, e é o que o ecrã de facto desenha (cada triângulo
    // leva a textura por um afim). ⛔ Perguntar o campo contínuo aqui mediria uma arte que o
    // rasterizador não pinta.
    let pesos_em = |q: [f64; 2]| -> Vec<f64> {
        let m = sm.ossos();
        if m == 0 {
            return Vec::new();
        }
        let mut melhor: Option<(f64, Vec<f64>)> = None;
        for t in &sm.mesh.tris {
            let (a, b, c) = (
                sm.mesh.rest[t[0] as usize],
                sm.mesh.rest[t[1] as usize],
                sm.mesh.rest[t[2] as usize],
            );
            let den = (b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1]);
            if den.abs() < 1e-12 {
                continue;
            }
            let u = ((q[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (q[1] - a[1])) / den;
            let v = ((b[0] - a[0]) * (q[1] - a[1]) - (q[0] - a[0]) * (b[1] - a[1])) / den;
            let fora = (-u).max(-v).max(u + v - 1.0).max(0.0);
            if melhor.as_ref().is_none_or(|(f, _)| fora < *f) {
                let (pa, pb, pc) = (
                    sm.pesos_de(t[0] as usize),
                    sm.pesos_de(t[1] as usize),
                    sm.pesos_de(t[2] as usize),
                );
                melhor = Some((
                    fora,
                    (0..m)
                        .map(|k| (1.0 - u - v) * pa[k] + u * pb[k] + v * pc[k])
                        .collect(),
                ));
            }
        }
        melhor.map(|(_, w)| w).unwrap_or_default()
    };
    let pts: Vec<[f64; 2]> = (0..n)
        .map(|i| {
            let t = std::f64::consts::TAU * f64::from(i) / f64::from(n);
            let q = [centro[0] + r * t.cos(), centro[1] + r * t.sin()];
            ponto_do_produto(&pele, p2l.apply(q), &pesos_em(q), &mut w)
        })
        .collect();
    let c = pts
        .iter()
        .fold([0.0, 0.0], |a, p| [a[0] + p[0], a[1] + p[1]]);
    let c = [c[0] / f64::from(n), c[1] / f64::from(n)];
    let (mut rmin, mut rmax) = (f64::INFINITY, 0.0_f64);
    for p in &pts {
        let d = (p[0] - c[0]).hypot(p[1] - c[1]);
        rmin = rmin.min(d);
        rmax = rmax.max(d);
    }
    rmax / rmin
}

/// ⭐⭐⭐ **NENHUM PEDAÇO DA ARTE DESTA CENA ESTÁ FORA DO ALCANCE DOS OSSOS.**
///
/// ⛔⛔ **Este é o gate que não existia, e a ausência dele custou um smoke reprovado.** Um ponto
/// órfão salta em salto seco para o osso mais próximo, e um salto seco num mapa contínuo
/// **rasga a arte** — foi isso que o dono fotografou em 2026-09-15.
///
/// ⚠️ **As duas colunas, e a que decide é a primeira:** a `inverted` lia `0,00 %` sobre a foto
/// do rasgo, porque inverter e rasgar são defeitos diferentes. Medir só a inversão é o que me
/// deixou escrever «bem abaixo do ângulo em que o mapa dobra» sobre uma cena partida.
#[test]
fn nenhum_pedaco_da_arte_fica_fora_do_alcance_dos_ossos() {
    let r = dobra_da_cena(super::ALTURA_PX, None);
    // ⚠️ Piso de população: uma caixa fora da pele devolveria zero amostras e um relatório
    // limpo que não mediu nada — a própria régua avisa disso por escrito.
    assert!(
        r.samples > 3_000,
        "a regua mediu {} amostras: ela nao esta' sobre a arte",
        r.samples
    );
    assert_eq!(
        r.orphan,
        0.0,
        "{:.2}% da arte esta' ORFA (fora do raio de todo osso) — ela vai saltar em salto seco \
         para o osso mais perto, e o dono ve' a arte RASGADA",
        r.orphan * 100.0
    );
    assert_eq!(
        r.inverted,
        0.0,
        "{:.2}% da arte sai DO AVESSO (det_min {:.4})",
        r.inverted * 100.0,
        r.det_min
    );
}

/// ⭐⭐⭐ **A ARTE NÃO SAI FACETADA — e este é o gate do SEGUNDO report do dono.**
///
/// ⛔⛔⛔ *«A malha deforma a curva»* (2026-09-15, com a seta na borda de cima). O mapa estava
/// **contínuo e injectivo** (`0,00 %` órfã, `0,00 %` do avesso) e mesmo assim a arte saía com
/// quinas: junto da borda do suporte dos pesos a curvatura do campo explode, e a malha pinta
/// **um afim por triângulo**. *Um mapa pode estar perfeito e a malha que o amostra não o seguir.*
///
/// ⚠️ **A barra é `1,5 px` de ecrã**, e o número tem os dois lados medidos. ⛔ Ela **não** é a
/// tolerância do `Smooth` (`0,5 px`): esse refinamento está estruturalmente desligado nesta
/// densidade (o handoff §13 tem a aritmética), e uma barra que o produto não consegue honrar
/// seria um número que mente.
///
/// ## O que mudou em 2026-09-15, e o que NÃO mudou
///
/// | | faceta | vazamento |
/// |---|---:|---:|
/// | euclidiana, alcance de fábrica (a 2.ª foto reprovada) | `6,78 px` | `0,00 px` |
/// | euclidiana, `strength = 2,0` (o que shipou em 14/09) | `0,40 px` | ⛔ `26,51 px` |
/// | **padrão-ouro, alcance de fábrica** | **`1,27 px`** | ⭐ `0,78 px` |
///
/// ⭐⭐ **A comparação honesta é a 1.ª linha contra a 3.ª** — as duas leis que NÃO vazam, isto é,
/// as duas que são rigs. Ali o padrão-ouro leva a faceta de `6,78` para `1,27 px` (`5,3×`) sem
/// um triângulo a mais. A 2.ª linha ganha esta coluna **por construção**: um alcance que cobre
/// a arte inteira mistura tudo com tudo, o que é suavíssimo e não é um rig.
///
/// ⚠️ **E a densidade da malha mudou por baixo deste número no mesmo dia** (`780` → `2 430`
/// peças): a `GridOptions` passou a ser um ORÇAMENTO em vez de um passo em pixels. À densidade
/// antiga o padrão-ouro lia `2,65 px`, acima desta barra — *o gate ficaria vermelho e estaria
/// certo.*
#[test]
fn a_arte_nao_sai_facetada() {
    let px = faceta_da_cena(super::ALTURA_PX, None);
    assert!(
        px <= 1.5,
        "a malha erra o campo em {px:.2} px de ecra: a arte sai com quinas e uma circunferencia \
         desenhada por cima dela tambem — e' o report «a malha deforma a curva»"
    );
}

/// ⭐⭐⭐ **OS DOIS CONTROLOS, E AGORA ELES AFIRMAM DUAS COISAS: a lei antiga ainda produz os
/// dois defeitos, e a lei nova CURA-OS.**
///
/// ⛔⛔ **Sem a 1.ª metade os gates acima são vácuos** — uma régua que responde «limpo» a tudo
/// aprovaria qualquer cena. ⛔⛔ **E sem a 2.ª metade eles não afirmam que a jornada serviu para
/// alguma coisa:** o padrão-ouro entrou para curar exactamente estas duas fotos, e um gate que
/// só guarda as réguas deixaria a cura por provar.
///
/// ⚠️ **As duas metades correm sobre a MESMA cena e a MESMA malha** — o que muda é só se a
/// tabela de pesos guardada é consultada. *É a comparação mais justa que existe: o mesmo
/// desenho, o mesmo rig, o mesmo instante, duas leis.*
#[test]
fn as_duas_redaccoes_reprovadas_sao_acusadas_na_lei_antiga_e_curadas_na_nova() {
    // ── 1.ª foto — o canvas QUADRADO com os ossos no alcance de fábrica: a arte RASGA. ──
    //
    // ⚠️ O órfão é um facto da lei EUCLIDIANA (fora do raio de todo osso a pele salta em salto
    // seco), e é por isso que o controlo dele se mede na `fold`, que amostra o campo contínuo.
    let quadrado = dobra_da_cena(super::LARGURA_PX, Some(1.0));
    assert!(
        quadrado.orphan > 0.30,
        "a regua leu so' {:.2}% de orfas no canvas QUADRADO que foi reprovado: ela deixou de \
         ver o defeito, e o gate irmao passou a nao afirmar nada",
        quadrado.orphan * 100.0
    );
    // ⭐ E a CURA: na mesma cena, com os pesos guardados, nenhum vértice fica sem dono.
    let (pior, sem_dono) = particao_da_cena(super::LARGURA_PX, Some(1.0));
    assert_eq!(
        (sem_dono, pior < 1e-9),
        (0, true),
        "o padrao-ouro deixou {sem_dono} vertices sem dono (pior |soma-1| {pior:.2e}) no canvas \
         QUADRADO: a foto do RASGO voltaria a ser possivel"
    );

    // ── 2.ª foto — a tira com a arte na BORDA do alcance: o mapa fica limpo e a malha FACETA. ──
    let na_borda = dobra_da_cena(super::ALTURA_PX, Some(1.0));
    assert_eq!(
        (na_borda.orphan, na_borda.inverted),
        (0.0, 0.0),
        "o controlo da faceta deixou de ser o caso SUBTIL: com orfas ou inversao ele passa a \
         medir o defeito do irmao, e a licao — *um mapa limpo pode facetar* — evapora"
    );
    let antiga = faceta_com(super::ALTURA_PX, Some(1.0), false);
    assert!(
        antiga > 5.0,
        "a regua da faceta leu so' {antiga:.2} px na lei antiga: ela deixou de ver o defeito \
         que o dono fotografou"
    );
    // ⭐ E a CURA, sobre a MESMA malha: só a lei dos pesos muda.
    let nova = faceta_com(super::ALTURA_PX, Some(1.0), true);
    assert!(
        nova < antiga / 3.0,
        "o padrao-ouro so' levou a faceta de {antiga:.2} px para {nova:.2} px na cena da 2.a \
         foto: ele nao esta' a fazer o trabalho por que entrou"
    );
}

/// ⭐⭐⭐ **O RIG NÃO VAZA — rodar a PONTA da corrente não mexe na arte da RAIZ.**
///
/// ⛔⛔⛔ **Este gate substitui o `a_arte_vive_longe_da_borda_do_alcance`, e a troca é a lição
/// inteira de 2026-09-15.** Aquele exigia que a derivação da `strength` aterrasse onde a tabela
/// a mediu — uma afirmação sobre o ALCANCE, que com o padrão-ouro deixou de decidir o que quer
/// que seja para uma imagem. E, pior, ele guardava precisamente a configuração que este mede
/// como defeituosa: `strength = 2,0` é um alcance que cobre a arte inteira ⇒ toda a arte
/// responde a todos os ossos.
///
/// ⚠️⚠️ **É a única régua desta cena que uma mistura larga de mais NÃO ganha por construção.**
/// Faceta, esticão e círculo medem suavidade, e borrar tudo é suave. *Medir só suavidade é como
/// uma linha inteira de trabalho shipou um borrão global a chamar-se rig.*
///
/// ⚠️ A barra é `2 px` de ecrã, e os dois lados estão medidos: o produto entrega `0,78` e a
/// redacção que shipou em 2026-09-14 entregava **`26,37`**.
#[test]
fn o_rig_nao_vaza_a_ponta_nao_arrasta_a_raiz() {
    let px = vazamento_da_cena(None);
    assert!(
        px <= 2.0,
        "rodar so' a PONTA da corrente moveu a arte da RAIZ {px:.2} px de ecra: isto nao e' um              rig, e' uma mistura global — cada osso alcanca a arte inteira"
    );
}

/// ⭐⭐⭐ **A ARTE TEM DONO EM TODO O LADO — a partição da unidade, medida na malha GUARDADA.**
///
/// ⛔⛔ **É o que substitui a contagem de órfãs para uma imagem.** O órfão era um facto da lei
/// euclidiana: fora do raio de todo osso a pele saltava em salto seco para o mais próximo, e a
/// arte RASGAVA (1.º report do dono). O padrão-ouro resolve a energia sobre o DOMÍNIO, logo não
/// há fora — e o que se afirma passa a ser a propriedade que o substitui: **todo vértice tem
/// pesos e eles somam `1`**.
///
/// ⚠️ **Um vértice sem pesos nenhuns é o modo de falha real** (o solver não resolveu e a imagem
/// caiu na lei derivada, em silêncio) — e ele é contado à parte, porque a soma nem chega a ser
/// perguntada ali.
#[test]
fn toda_a_arte_tem_dono_e_os_pesos_somam_um() {
    let (pior, sem_dono) = particao_da_cena(super::ALTURA_PX, None);
    assert_eq!(
        sem_dono, 0,
        "{sem_dono} vertices da malha guardada nao tem peso nenhum: o solver do padrao-ouro nao              resolveu e esta arte caiu na lei derivada SEM ninguem avisar"
    );
    assert!(
        pior < 1e-9,
        "o pior |soma - 1| da malha guardada e' {pior:.2e}: a particao da unidade partiu-se, e              um ponto cuja soma nao e' 1 e' um ponto que a mistura desloca"
    );
}

/// ⭐⭐⭐ **A CENA DE FACTO DEFORMA A ARTE — e por uma quantidade MEDIDA.**
///
/// ⛔⛔ **É o gate que impede a cura de virar disfarce.** Depois do 3.º report do dono
/// (*«bem melhor. deformou um pouco»*) a tentação é baixar a dobra até a deformação sumir — e
/// isso seria uma **cena que ensina o contrário do que acontece** (CLAUDE.md §5.0): o que ela
/// existe para mostrar é justamente que a arte dobra e que as guias a seguem.
///
/// ## A varredura, medida no caminho do PRODUTO a partir do REPOUSO (bancada desta cena)
///
/// | graus/junta | esticão máx | esticão p99 | círculo | faceta |
/// |---:|---:|---:|---:|---:|
/// | **`0`** | **`1,0000`** | **`1,0000`** | **`1,0000`** | **`0,000 px`** |
/// | `6` | `1,1207` | `1,0882` | `1,0625` | `0,307` |
/// | `10` | `1,2006` | `1,1460` | `1,1065` | `0,511` |
/// | `15` | `1,2994` | `1,2188` | `1,1644` | `0,766` |
/// | **`25`** — esta cena | **`1,4918`** | `1,3587` | **`1,2913`** | **`1,270`** |
/// | `35` | `1,6752` | `1,4870` | `1,4355` | `1,764` |
/// | `45` | `1,8474` | `1,6078` | `1,6011` | `2,245` |
///
/// ⭐ **A linha `0` é o CONTROLO da própria tabela:** em repouso o mapa é a identidade e as
/// quatro colunas dizem-no ao bit. ⚠️ Ela custou uma medição: a 1.ª redacção da varredura
/// escrevia `rotation = graus` numa cena já dobrada e lia `1,426` de esticão **em repouso** —
/// ver a [`cena_dobrada`]. *Repor um ângulo não é repor uma pose.*
///
/// ⚠️⚠️ **O TECTO SUBIU de `1,20` para `1,35`, e não é uma barra afrouxada — é a ARTICULAÇÃO a
/// passar a existir.** Ele fora calibrado quando a cena corria com `strength = 2,0`, um alcance
/// que cobre a arte inteira: ali todo osso pesa em todo ponto, a deformação é quase uma
/// semelhança global, e uma circunferência desenhada por cima de uma junta que dobra `25°` sai
/// **redonda** (`1,15`) porque a junta quase não dobra a arte. ⛔ *O que mantinha o círculo
/// redondo era o rig não estar a articular* — e o preço disso está medido noutro gate: rodar a
/// ponta arrastava a raiz `26 px`.
///
/// ⭐ **O `1,35` sai do vale MEDIDO da tabela**, entre esta cena (`1,2913`) e o degrau seguinte
/// (`1,4355` a `35°`) — ⛔ não de um número escolhido para a cena passar.
///
/// ⚠️ **As DUAS metades:** o piso diz *«a cena ainda demonstra»* e o tecto diz *«a arte não está
/// a ser maltratada»*. Só o tecto seria um gate que uma cena plana passaria.
#[test]
fn a_cena_deforma_a_arte_o_bastante_para_demonstrar_e_nao_mais() {
    let fora = circulo_fora_de_redondo(super::ALTURA_PX, None);
    assert!(
        fora > 1.10,
        "a circunferencia sai {:.1}% fora de redondo: a dobra deixou de se VER, e a cena passa \
         a ensinar que prender arte a ossos nao faz nada",
        (fora - 1.0) * 100.0
    );
    assert!(
        fora < 1.35,
        "a circunferencia sai {:.1}% fora de redondo: a arte esta' a ser maltratada, e o dono \
         ve' isso antes de ver a guia que a cena existe para mostrar",
        (fora - 1.0) * 100.0
    );
}

/// ⭐⭐⭐ **A CENA PRENDE ANTES DE DOBRAR, E A ORDEM É LOAD-BEARING.**
///
/// ⛔⛔ O repouso de uma pele é **o instante do bind**. Dobrada primeiro, esta pose SERIA o
/// repouso e o canvas sairia RECTO — a cena montaria, imprimiria a linha de sucesso, e não
/// provaria nada. *Um smoke que monta e não demonstra é pior que um ausente: ele é acreditado.*
///
/// ⚠️ A régua é a POSIÇÃO no ficheiro, que é o que uma leitura rápida do diff inverte sem dar
/// por isso — mover a chamada da `dobra` para cima de `bind_image` é uma edição de duas linhas.
#[test]
fn the_scene_binds_before_it_bends() {
    let fonte = include_str!("smoke_bone_paint.rs");
    let bind = fonte.find("bind_image(\n").expect("a cena prende a imagem");
    let dobra = fonte
        .find("dobra(sim, &ossos);")
        .expect("a cena dobra os ossos");
    assert!(
        bind < dobra,
        "a cena dobra ANTES de prender: o repouso passa a ser a pose dobrada e o canvas sai \
         recto — ela montaria e nao provaria nada"
    );
}

/// ⭐ **O nível declarado é o que existe** — ver a nota do [`super::NIVEIS`].
#[test]
fn the_router_declares_only_the_scene_that_exists() {
    assert_eq!(super::NIVEIS, 1);
}

/// ⏱️ **A BANCADA das duas leis** — filha por ASSUNTO (e pelo tecto de LOC), e aqui dentro para
/// herdar as fixturas deste arnês: uma cópia delas mediria outra cena.
#[path = "smoke_bone_paint_bancada.rs"]
mod bancada;
