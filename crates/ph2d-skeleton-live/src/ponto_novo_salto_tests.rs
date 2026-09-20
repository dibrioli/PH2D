//! ⭐⭐⭐ **O DESENHO NÃO SALTA AO GANHAR UM PONTO** — a metade do [`super::tests`] que mede o que o
//! artista VÊ, e não o que a fonte guarda.
//!
//! ⛔ Ele vive ao lado do irmão e não dentro dele por **tecto de LOC** (`702` contra `700`, em
//! 2026-09-19). *O ficheiro é uma unidade de manutenção; a lei é a mesma.*

use super::*;
use ph2d_ecs::Transform;
use ph2d_vec_scene::{ShapeKind, VecScene, cook};

use super::tests::{
    LEI_DA_CURVA, LEI_INGENUA, ancoras, fonte, osso, palco, palco_desenhado, palco_do_produto,
};

/// ⭐⭐⭐ **A FORMA QUASE NÃO SE MOVE, e o «quase» é medido e tem mecanismo.**
///
/// O desenho é uma Bézier cujos pontos de controlo saem de **duas** linhas de pesos, e um corte
/// de Casteljau na geometria de repouso só comuta com a deformação quando as duas são iguais. ⇒ o
/// desvio é proporcional a quanto o peso varia ao longo daquele segmento, e a barra sai da **medição
/// com a pose DOBRADA**, que é o pior caso do palco.
///
/// ⛔ **A barra é uma FRACÇÃO da peça e não um número absoluto** — uma barra em unidades de mundo
/// mediria o tamanho da fixtura.
#[test]
fn o_ponto_novo_quase_nao_move_a_forma() {
    let (mut sim, mut cena, mapa, id, [_, ponta]) = palco();
    // A pose DOBRADA: é com o osso virado que as duas linhas de peso de um segmento divergem.
    sim.world_mut()
        .get_mut::<Transform>(ponta)
        .expect("Transform")
        .rotation = 0.8;
    crate::skin_live::recook_com(&sim, &mut cena, LEI_INGENUA);
    let antes = ancoras(&cena, id);

    assert!(
        insere_ponto_com(&mut sim, &mapa, id, 0, 0.5, LEI_INGENUA).is_some(),
        "a porta recusou uma forma que ESTA' presa"
    );
    crate::skin_live::recook_com(&sim, &mut cena, LEI_INGENUA);
    let depois = ancoras(&cena, id);
    assert_eq!(depois.len(), antes.len() + 1);

    // As âncoras VELHAS têm de ficar onde estavam — o ponto novo entra entre a 0 e a 1.
    let mut pior = 0.0_f64;
    for (k, p) in antes.iter().enumerate() {
        let q = if k == 0 { depois[0] } else { depois[k + 1] };
        pior = pior.max((p[0] - q[0]).hypot(p[1] - q[1]));
    }
    let diagonal = 40.0_f64.hypot(10.0);
    let fraccao = pior / diagonal;
    eprintln!(
        "[ponto-novo] pior desvio de ancora = {pior:.6} ({:.4} % da peca)",
        fraccao * 100.0
    );
    assert!(
        fraccao < 1e-9,
        "as ancoras VELHAS moveram-se {fraccao} da peca — o corte mexeu em quem nao devia"
    );
}

/// A curva desenhada, amostrada — o que o olho vê, e não os pontos de controlo.
fn polilinha(cena: &VecScene, id: VecPathId) -> Vec<[f64; 2]> {
    const AMOSTRAS: usize = 400;
    let Some(caminho) = cena.path(id) else {
        return Vec::new();
    };
    let cozido = caminho.cooked();
    let mut out = Vec::new();
    for c in 0..cozido.contour_count() {
        let Some((verts, fechado)) = cozido.contour(c) else {
            continue;
        };
        let n = verts.len();
        let ultimo = if fechado { n } else { n.saturating_sub(1) };
        for i in 0..ultimo {
            let a = &verts[i];
            let b = &verts[(i + 1) % n];
            for k in 0..AMOSTRAS {
                let t = k as f64 / AMOSTRAS as f64;
                let u = 1.0 - t;
                let (w0, w1, w2, w3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
                out.push([
                    w0 * a.anchor[0]
                        + w1 * a.out_handle[0]
                        + w2 * b.in_handle[0]
                        + w3 * b.anchor[0],
                    w0 * a.anchor[1]
                        + w1 * a.out_handle[1]
                        + w2 * b.in_handle[1]
                        + w3 * b.anchor[1],
                ]);
            }
        }
    }
    out
}

/// ⭐⭐⭐ **O DESENHO NÃO SALTA AO GANHAR UM PONTO** — a lei, com o controlo positivo dentro.
///
/// ⛔⛔⛔ **Este gate SUBSTITUI o `a_curva_desenhada_move_se_o_que_o_peso_varia_no_segmento`, que
/// afirmava o CONTRÁRIO** — que a curva *tinha* de se mexer, que isso era refinamento e que a
/// catraca media quanto. **Report do dono, 2026-09-19: *«o ponto criado na malha já conectada aos
/// ossos deforma a malha»*.** Eu tinha medido o salto, chamado-lhe refinamento e dito-lhe que era
/// normal; *a régua dele é a que manda, porque o desenho é o que o artista vê.*
///
/// | fixtura | ANTES (corte de repouso) | AGORA (com compensação) |
/// |---|---|---|
/// | aresta CRUA (um segmento sobre os dois ossos) | `18,89 %` da peça | **`0,0000 %`** |
/// | aresta DESENHADA em 8, pior segmento | `0,91 %` | **`0,0000 %`** |
/// | escada `1 → 2 → 4 → 8` | `18,89 → 3,13 → 1,00 %` | **`~1e-14`** em todos |
///
/// ⭐⭐ **O CONTROLO POSITIVO é o que impede este gate de ser vácuo:** ele corre a MESMA fixtura pelo
/// caminho **sem** pele (`pele = None`), que é o corte de repouso de antes, e exige que ali a curva
/// se mexa muito. *Sem ele, uma fixtura em que a pele é a identidade passaria com a compensação
/// apagada.*
///
/// ⚠️ **A régua amostra 400 pontos por segmento, e o número não é decoração:** com `24` ela lia
/// `0,0059 %` sobre uma lei exacta — a distância que ela media era o erro de CORDA da própria
/// polilinha grossa, não o da curva. *Uma régua discreta mede a discretização dela antes de medir o
/// produto.*
#[test]
fn o_desenho_nao_salta_ao_ganhar_um_ponto() {
    /// O salto ao inserir no segmento `seg`, em unidades de mundo. `compensa` desliga a lei nova.
    fn salto(pedacos: usize, rotacao: f32, seg: usize, compensa: bool) -> f64 {
        let (mut sim, mut cena, mapa, id, [_, ponta]) = if pedacos <= 1 {
            palco()
        } else {
            palco_desenhado(pedacos)
        };
        sim.world_mut()
            .get_mut::<Transform>(ponta)
            .expect("Transform")
            .rotation = rotacao;
        crate::skin_live::recook_com(&sim, &mut cena, LEI_INGENUA);
        let antes = polilinha(&cena, id);
        assert!(antes.len() > 8, "a fixtura tem de ter curva para medir");
        if compensa {
            assert!(insere_ponto_com(&mut sim, &mapa, id, seg, 0.5, LEI_INGENUA).is_some());
        } else {
            // ⛔ O caminho de ANTES, à mão: a fonte parte-se sem a pele, logo sem compensação.
            let e = Entity::from_bits(*mapa.get(&id).expect("entidade"));
            let skin = sim.world().get::<SkinBind>(e).expect("presa").clone();
            let mut fonte = crate::skinned_mesh::le(&skin.source).expect("le");
            let (_, bytes) = insere_na_fonte(&mut fonte, seg, 0.5, None, &[], PASSAGENS)
                .expect("o corte de repouso");
            sim.world_mut()
                .get_mut::<SkinBind>(e)
                .expect("presa")
                .source = bytes;
        }
        crate::skin_live::recook_com(&sim, &mut cena, LEI_INGENUA);
        polilinha(&cena, id)
            .iter()
            .map(|p| ph2d_skeleton::dist2_to_polyline(*p, &antes).sqrt())
            .fold(0.0_f64, f64::max)
    }

    const PEDACOS: usize = 8;
    let diagonal = 40.0_f64.hypot(10.0);

    // ⛔ O segmento onde o peso MAIS varia — e a fixtura tem de o conter, senão a lei preserva a
    // forma AO BIT por construção e o gate passa por vácuo. O sítio é a JUNTA.
    let (sim, _c, mapa, id, _) = palco_desenhado(PEDACOS);
    let f = fonte(&sim, &mapa, id);
    let (pior_seg, gap) = (0..PEDACOS)
        .map(|k| {
            let (a, b) = (
                f.linha_do_no(k).expect("a"),
                f.linha_do_no(k + 1).expect("b"),
            );
            let g = a
                .iter()
                .zip(b)
                .map(|(x, y)| (x - y).abs())
                .fold(0.0_f64, f64::max);
            (k, g)
        })
        .max_by(|x, y| x.1.total_cmp(&y.1))
        .expect("ha' segmentos");
    assert!(
        gap > 1e-3,
        "nenhum segmento da aresta desenhada tem pesos diferentes nas pontas ({gap}) — a fixtura \
         nao contem o fenomeno"
    );

    let crua = salto(1, 0.8, 0, true) / diagonal * 100.0;
    let desenhada = salto(PEDACOS, 0.8, pior_seg, true) / diagonal * 100.0;
    let repouso = salto(PEDACOS, 0.0, pior_seg, true);
    let sem_lei = salto(1, 0.8, 0, false) / diagonal * 100.0;
    eprintln!(
        "[ponto-novo] salto da curva: aresta CRUA={crua:.6} % · DESENHADA em {PEDACOS} (pior \
         segmento {pior_seg}, gap {gap:.4})={desenhada:.6} % · repouso={repouso:.9} · CONTROLO sem \
         compensacao={sem_lei:.4} % da peca"
    );

    // ⭐ **O CONTROLO POSITIVO, primeiro:** sem a lei nova esta mesma fixtura salta muito.
    assert!(
        sem_lei > 5.0,
        "o caminho SEM compensacao saltou so' {sem_lei:.4} % — a fixtura deixou de conter o \
         fenomeno, e as barras abaixo passam por vacuo"
    );
    for (nome, v) in [("crua", crua), ("desenhada", desenhada)] {
        assert!(
            v < 1e-3,
            "a aresta {nome} saltou {v:.6} % da peca ao ganhar um ponto — o dono recusou isto por \
             escrito: «o ponto criado na malha ja' conectada aos ossos deforma a malha»"
        );
    }
    assert!(
        repouso < 1e-9,
        "em REPOUSO o corte mexeu na curva ({repouso}) — ali a compensacao e' a IDENTIDADE ao bit, \
         logo o que mexeu foi o corte, e esta regua esta' a medir a coisa errada"
    );
}

/// ⭐⭐⭐ **E NÃO SALTA EM NENHUMA PROFUNDIDADE** — cortar o mesmo segmento `1 → 2 → 4 → 8` vezes.
///
/// ⛔⛔ **Este gate SUBSTITUI o `a_escada_da_subdivisao_diz_se_o_salto_e_refinamento`, e a premissa
/// dele MORREU.** Ele media se os saltos **encolhiam** (`18,89 → 3,13 → 1,00 %`) para provar que o
/// desvio era uma aproximação a ser refinada. Com a compensação não há desvio nenhum para encolher:
/// os três degraus leem `~1e-14`, que é a aritmética da máquina. *Uma escada que já não tem degraus
/// não se mede pela inclinação.*
#[test]
fn o_desenho_nao_salta_em_nenhuma_profundidade() {
    let construir = |cortes: usize| -> Vec<[f64; 2]> {
        let (mut sim, mut cena, mapa, id, [_, ponta]) = palco();
        sim.world_mut()
            .get_mut::<Transform>(ponta)
            .expect("Transform")
            .rotation = 0.8;
        for _ in 0..cortes {
            let n = fonte(&sim, &mapa, id).path.verts_all().count();
            for seg in (0..n).rev() {
                if seg < cortes_da_aresta(&sim, &mapa, id) {
                    let _ = insere_ponto_com(&mut sim, &mapa, id, seg, 0.5, LEI_INGENUA);
                }
            }
        }
        crate::skin_live::recook_com(&sim, &mut cena, LEI_INGENUA);
        polilinha(&cena, id)
    };
    let degraus: Vec<Vec<[f64; 2]>> = (0..4).map(construir).collect();
    let diagonal = 40.0_f64.hypot(10.0);
    let desvios: Vec<f64> = (0..degraus.len() - 1)
        .map(|k| {
            degraus[k + 1]
                .iter()
                .map(|p| ph2d_skeleton::dist2_to_polyline(*p, &degraus[k]).sqrt())
                .fold(0.0_f64, f64::max)
                / diagonal
                * 100.0
        })
        .collect();
    eprintln!("[ponto-novo] escada da subdivisao (% da peca): {desvios:?}");
    assert!(
        degraus.iter().all(|d| d.len() > 8),
        "algum degrau da escada ficou sem curva: a fixtura nao esta' a subdividir"
    );
    assert!(
        desvios.iter().all(|d| *d < 1e-6),
        "a escada tem degraus: {desvios:?} — antes da compensacao ela lia 18,89 -> 3,13 -> 1,00 %, \
         e a lei nova existe para os levar a zero"
    );
}

/// Quantos segmentos a aresta de baixo tem agora (ela começa com `1`).
fn cortes_da_aresta(sim: &SimWorld, mapa: &VecEntityMap, id: VecPathId) -> usize {
    fonte(sim, mapa, id).path.verts_all().count() - 3
}

/// ⭐⭐⭐ **A SEGUNDA PASSAGEM DA COMPENSAÇÃO NÃO É ZELO — ela é exigida por uma MANCHA.**
///
/// ⛔⛔ **Ele nasceu de uma MUTAÇÃO SOBREVIVENTE:** trocar `0..2` por `0..1` passava a suíte inteira.
/// A razão é que no corpus de então **nada** fazia o peso depender da POSIÇÃO — ossos rectos, sem
/// manchas —, e ali a primeira passagem já acerta. *Uma linha que a mutação não consegue matar não é
/// lei; ou se apaga, ou se lhe dá a fixtura que a torna observável.*
///
/// A mancha é a correcção que o pincel de peso pinta: ela é uma **bolha no espaço**, logo a linha de
/// pesos do vértice **muda quando ele se move** — e a compensação move-o. ⇒ a primeira passagem
/// resolve com a linha da âncora provisória e deixa resíduo; as seguintes repetem com a âncora já
/// movida, e cada uma divide o resíduo por **~55** (`0,0772 → 0,0014 → 2,6e-5 → 4,7e-7 → 0`).
///
/// ⚠️⚠️ **E a fixtura mordeu DUAS vezes antes de conter o fenómeno.** A 1.ª punha a mancha com o
/// ponto a nascer no **cume** dela, onde o `clamp(0,1)` do `corrige` **satura**: ali o peso volta a
/// ser constante e uma passagem basta — *uma mancha saturada não é uma mancha, é um planalto*. A
/// 2.ª escrevia o braço de «uma passagem» à mão e **não fazia crescer a tabela de pesos**, logo o
/// que ela media era a tabela e não a passagem — *um controlo que não percorre a MESMA porta compara
/// dois programas*.
#[test]
fn a_segunda_passagem_e_exigida_por_uma_mancha() {
    let com = |passagens_a_mais: bool| -> f64 {
        let (mut sim, mut cena, mapa, id, [raiz, ponta]) = palco();
        sim.world_mut()
            .get_mut::<Transform>(ponta)
            .expect("Transform")
            .rotation = 0.8;
        // ⭐ A MANCHA: centrada no meio da aresta de baixo, que é onde o ponto vai nascer.
        let e = Entity::from_bits(*mapa.get(&id).expect("entidade"));
        let osso = sim
            .world()
            .get::<ph2d_ecs::StableId>(raiz)
            .copied()
            .expect("o osso tem identidade");
        sim.world_mut()
            .get_mut::<SkinBind>(e)
            .expect("presa")
            .correcoes
            .push(ph2d_skeleton_ecs::CorreccaoDePeso {
                bone: osso,
                // ⛔⛔ **O ponto tem de cair na ENCOSTA da bolha, e não no cume.** A 1.ª redacção
                // punha o centro em `[20, 0]` com `delta 0,9` — o ponto nascia exactamente no cume,
                // onde o `clamp(0,1)` do [`ph2d_skeleton::Skin::corrige`] **satura**: ali o peso
                // volta a ser CONSTANTE, a fixtura deixa de depender da posição, e uma passagem
                // basta. *Uma mancha saturada não é uma mancha, é um planalto.*
                centro: [10.0, 0.0],
                raio: 25.0,
                especie: ph2d_skeleton::Especie::Soma(0.35),
            });
        crate::skin_live::recook_com(&sim, &mut cena, LEI_INGENUA);
        let antes = polilinha(&cena, id);
        // ⭐⭐ **A MESMA PORTA nos dois lados, e só a PASSAGEM muda.** A 1.ª redacção escrevia o
        // braço de «uma passagem» à mão e esquecia-se de crescer a tabela de pesos — o que ela media
        // era a tabela, e a mutação que punha `PASSAGENS = 1` sobrevivia. *Um controlo que não
        // percorre a mesma porta compara dois programas.*
        let skin = sim.world().get::<SkinBind>(e).expect("presa").clone();
        let mut fonte = crate::skinned_mesh::le(&skin.source).expect("le");
        let pele = crate::skin_live::resolve(&sim, &skin, e, &crate::skin_live::bone_index(&sim));
        let correcoes = skin.correcoes_resolvidas();
        let n = if passagens_a_mais { PASSAGENS } else { 1 };
        let (_, bytes) =
            insere_na_fonte(&mut fonte, 0, 0.5, pele.as_ref(), &correcoes, n).expect("insere");
        sim.world_mut()
            .get_mut::<SkinBind>(e)
            .expect("presa")
            .source = bytes;
        crate::skin_live::recook_com(&sim, &mut cena, LEI_INGENUA);
        polilinha(&cena, id)
            .iter()
            .map(|p| ph2d_skeleton::dist2_to_polyline(*p, &antes).sqrt())
            .fold(0.0_f64, f64::max)
    };
    let uma = com(false);
    let duas = com(true);
    eprintln!("[ponto-novo] com MANCHA: uma passagem={uma:.6} · duas passagens={duas:.9}");
    assert!(
        uma > 1e-4,
        "com UMA passagem o desenho saltou so' {uma} — a fixtura nao contem o fenomeno (a mancha \\
         nao esta' a fazer o peso depender da posicao), e a barra abaixo passa por vacuo"
    );
    assert!(
        duas < uma * 0.05,
        "a segunda passagem so' baixou o salto de {uma} para {duas} — ou ela nao esta' a correr, ou \\
         a lei precisa de mais do que duas"
    );
}

/// ⭐⭐⭐ **COM A LEI DA CURVA, ACRESCENTAR UM PONTO QUASE NÃO MOVE NADA — e sem compensação nenhuma.**
///
/// ⛔⛔⛔ **É a F30 a DISSOLVER a F28.** A compensação existe porque o desenho era a Bézier dos pontos
/// de controlo deformados, e um corte no repouso não comuta com essa lei. Com o desenho a ser a
/// **imagem verdadeira** da fonte, partir a fonte deixa de o mover: a curva é a mesma, só está
/// escrita com mais pedaços.
///
/// ⚠️⚠️ **E compensar aqui ESTRAGA**, que é a segunda metade deste gate: a compensação move os
/// pontos de controlo da FONTE para acertar uma curva que já não é a desenhada — medido `11,11 %` da
/// peça, contra `0,02 %` sem ela. *Uma cura fica errada no dia em que o defeito que ela curava deixa
/// de existir*, e é por isso que a decisão sai da mesma porta que o `recook` lê.
///
/// ⛔⛔ **E o «POR CONSTRUÇÃO» desta redacção MORREU em 2026-09-19, com a conciliação das alças**
/// (report do dono: *«muitas irregularidades … mau tratamento das alças dos handles»*). O ajuste das
/// alças deixou de ser resolvido segmento a segmento **em isolamento**: as duas alças de um nó
/// passam a rodar juntas ([`ph2d_vec_skin::curva`]), logo o resultado num segmento depende do
/// **VIZINHO** — e partir um segmento muda a vizinhança de um nó. ⇒ o salto deixa de ser `0,00 %`
/// exacto e passa a ser **`0,0201 %`** da peça (`0,008` unidades numa peça de `40 × 10`).
///
/// ⚠️ **A barra sai do VALE entre duas medições e não de um número confortável:** `0,0201 %` com a
/// lei de hoje contra **`11,11 %`** com a compensação ligada — `550×`. *O que este gate afirma
/// continua a ser o que importa ao artista: acrescentar um ponto não mexe no desenho a olho.*
#[test]
fn com_a_lei_da_curva_o_ponto_novo_nao_move_nada() {
    let (mut sim, mut cena, mapa, id, [_, ponta]) = palco_do_produto();
    sim.world_mut()
        .get_mut::<Transform>(ponta)
        .expect("Transform")
        .rotation = 0.8;
    crate::skin_live::recook_com(&sim, &mut cena, LEI_DA_CURVA);
    let antes = polilinha(&cena, id);
    assert!(antes.len() > 8, "a fixtura tem de ter curva para medir");

    assert!(insere_ponto_com(&mut sim, &mapa, id, 0, 0.5, LEI_DA_CURVA).is_some());
    crate::skin_live::recook_com(&sim, &mut cena, LEI_DA_CURVA);
    let depois = polilinha(&cena, id);
    let salto = depois
        .iter()
        .map(|p| ph2d_skeleton::dist2_to_polyline(*p, &antes).sqrt())
        .fold(0.0_f64, f64::max);

    // ⭐ **A FONTE ganhou o ponto** — senão este zero seria o de um gesto que não aconteceu.
    let n = fonte(&sim, &mapa, id).path.verts_all().count();

    let diagonal = 40.0_f64.hypot(10.0);
    eprintln!(
        "[ponto-novo] com a LEI DA CURVA: salto = {:.6} % da peca · a fonte tem {n} nos",
        salto / diagonal * 100.0
    );
    // ⛔⛔ **A contagem já não é `5`, e a premissa morreu duas vezes no mesmo dia:** a subdivisão do
    // bind põe os nós que os ossos pedem, e a fixtura passou a ser a do PRODUTO. O que se afirma é
    // o que importa — *a fonte ganhou UM ponto* —, e não um literal.
    let sem_o_ponto = {
        let (s2, _c, m2, i2, _) = palco_do_produto();
        fonte(&s2, &m2, i2).path.verts_all().count()
    };
    assert_eq!(
        n,
        sem_o_ponto + 1,
        "a fonte NAO ganhou o ponto: o zero abaixo seria vacuo"
    );
    assert!(
        salto / diagonal < 1e-3,
        "com a lei da curva acrescentar um ponto moveu o desenho {:.4} % da peca — ou a compensacao \
         da F28 voltou a correr (ela ESTRAGA aqui, 11,11 %), ou o bind deixou de subdividir. ⚠️ Na \
         forma GROSSA isto vale 1,40 %: a correccao das alcas nao segue uma feicao mais fina do que \
         um segmento, e a cura daquele mundo e' a subdivisao",
        salto / diagonal * 100.0
    );
}

/// ⭐⭐⭐ **A SUBDIVISÃO DO BIND DISSOLVE O SALTO — a compensação desta suíte deixou de ter sujeito
/// no caminho de OMISSÃO** (ordem do dono, 2026-09-19).
///
/// ⚠️ **É a segunda vez que esta lei é dissolvida por baixo, e por isso ela é MEDIDA e não
/// suposta:** a F30 já lhe tinha tirado o sujeito no desenho (a arte passou a ser a imagem
/// verdadeira da curva), e agora a subdivisão tira-lho na GEOMETRIA (a forma já tem pontos que
/// cheguem). ⛔ *A compensação não é apagada:* ela continua a ser o que segura um ficheiro GRAVADO
/// antes de 19/09 e o caminho `bind_com(.., false)`, que é o sujeito de todos os outros gates deste
/// ficheiro.
///
/// A régua é o salto do desenho ao ganhar um ponto, em % da diagonal da peça — a mesma das irmãs.
#[test]
fn a_subdivisao_do_bind_dissolve_o_salto() {
    let salto_com = |subdividir: bool| {
        let mut sim = SimWorld::default();
        let mut cena = VecScene::new();
        let mut mapa = VecEntityMap::new();
        let id = cena.push_path(cook(ShapeKind::Rectangle, [0.0, 0.0], [40.0, 10.0], &[]));
        ph2d_vec_entities::entities::sync(&mut sim, &mut cena, &mut mapa);
        let raiz = osso(&mut sim, "Arm", [0.0, 5.0], 20.0, None);
        let ponta = osso(&mut sim, "Forearm", [20.0, 0.0], 20.0, Some(raiz));
        ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
        assert_eq!(
            crate::skin_live::bind_com(&mut sim, &cena, &mapa, &[id], Some(raiz), subdividir),
            1
        );
        sim.world_mut()
            .get_mut::<Transform>(ponta)
            .expect("pose")
            .rotation = 0.8;
        // ⛔⛔ **A LEI DA CURVA FICA DESLIGADA, e é o que torna esta medição legível.** A F30 já
        // tinha dissolvido este salto — *o desenho deixou de ser a curva dos pontos de controlo* —,
        // logo com ela ligada os dois lados leem `0,000002 %` e o gate não distingue nada.
        // *Para medir a segunda dissolução é preciso desligar a primeira.*
        crate::skin_live::recook_com(&sim, &mut cena, LEI_INGENUA);
        let antes = cena.path(id).expect("caminho").clone();
        // ⛔ **SEM COMPENSAÇÃO** — o corte de repouso cru, que é o caminho de ANTES da F28. É a
        // única forma de ver o salto que ela existe para curar: com a compensação ligada os dois
        // lados leem zero, e o gate não distingue nada.
        {
            let e = Entity::from_bits(*mapa.get(&id).expect("entidade"));
            let skin = sim.world().get::<SkinBind>(e).expect("presa").clone();
            let mut f = crate::skinned_mesh::le(&skin.source).expect("le");
            let (_, bytes) =
                insere_na_fonte(&mut f, 0, 0.5, None, &[], PASSAGENS).expect("o corte de repouso");
            sim.world_mut()
                .get_mut::<SkinBind>(e)
                .expect("presa")
                .source = bytes;
        }
        crate::skin_live::recook_com(&sim, &mut cena, LEI_INGENUA);
        let depois = cena.path(id).expect("caminho").clone();
        crate::test_support::pior_desvio_do_desenho(&antes, &depois) / 41.23 * 100.0
    };
    let grosso = salto_com(false);
    let subdividido = salto_com(true);
    eprintln!(
        "[ponto-novo] salto pela lei dos pontos de controlo: forma grossa={grosso:.6} % · \
         subdividida pelo bind={subdividido:.6} %"
    );
    assert!(
        grosso > 1e-3,
        "a forma GROSSA deixou de saltar ({grosso} %) — sem isso a linha de baixo nao afirma nada"
    );
    assert!(
        subdividido < grosso / 10.0,
        "a subdivisao do bind nao dissolveu o salto ({subdividido} % contra {grosso} %)"
    );
}
