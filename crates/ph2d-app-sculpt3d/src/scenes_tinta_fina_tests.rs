//! **OS GATES DA CENA DA TINTA FINA** (`=52`) — irmão (`#[path]`) da [`super`].

use super::*;

/// ⭐⭐ **ELA ABRE COM O PINCEL DE PINTURA NA MÃO E O ARAME LIGADO — as TRÊS
/// metades.**
///
/// O roteiro diz *«o pincel de PINTURA já está na sua mão»* e *«o arame já está
/// ligado»*, e as duas frases são AFIRMAÇÕES sobre o arranque. Cada metade falha
/// por um motivo diferente:
///
/// 1. **a LEI** — com a cena armada, o verbo escolhido é o da pintura;
/// 2. **o ARAME** — o `arma` liga-o, e sem ele o passo (4) perde o CONTROLO
///    inteiro (*«conte os quadrados outra vez»*);
/// 3. **o FIO** — o prólogo da crate chama esta cena.
///
/// ⛔ Sem a terceira, *um gate que chama a função em vez de percorrer a rota
/// afirma que a lei existe e nunca que a cena a usa* (§24).
#[test]
fn a_cena_da_tinta_fina_abre_com_o_pincel_e_o_arame() {
    assert_eq!(
        verbo_da_cena(true),
        Some(Verb::Paint),
        "a =52 não arma o pincel de pintura, e o roteiro promete-o na 1.ª linha"
    );
    assert_eq!(
        verbo_da_cena(false),
        None,
        "o prólogo desta cena mexe no pincel de OUTRA cena"
    );

    const ROTEADOR: &str = include_str!("scenes.rs");
    const ESTA: &str = include_str!("scenes_tinta_fina.rs");
    assert!(
        ROTEADOR.contains("tinta_fina::arma(cena);"),
        "o prólogo da crate não chama a =52: a lei existe e ninguém a corre"
    );
    let codigo: Vec<&str> = ESTA
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect();
    assert!(
        codigo.iter().any(|l| l.contains("pub(crate) fn arma(")),
        "a extracção de código partiu-se: o `arma` não está nas {} linhas colhidas",
        codigo.len()
    );
    assert!(
        codigo.iter().any(|l| l.contains("switch_verb_parts(")),
        "o `arma` deixou de passar pela porta de troca de ferramenta — o pincel \
         nasceria com o rótulo da pintura e a afinação do verbo de fábrica"
    );
    assert!(
        codigo.iter().any(|l| l.contains("cena.wireframe = true")),
        "o `arma` deixou de ligar o arame: o passo (4) do roteiro manda CONTAR os \
         quadrados, e sem eles a metade que separa esta cena da =51 não se vê"
    );
    assert!(
        codigo
            .iter()
            .any(|l| l.contains("verbo_da_cena(tinta_fina_scene())")),
        "o `arma` deixou de perguntar pela cena: ele armaria a pintura em TODA \
         cena do módulo"
    );
}

/// ⭐⭐⭐⭐ **O `8x` É O PRIMEIRO DEGRAU DA FILEIRA QUE ALCANÇA A DENSIDADE DA
/// CENA IRMÃ — e é essa a barra da grossura desta peça.**
///
/// ⛔⛔⛔ **A redacção anterior deste gate ESCOLHIA dois números, e uma mutação
/// passou por entre eles.** Ela pedia *«pelo menos `4×` mais grossa que a
/// irmã»* e *«entre `400` e `4 000` vértices»*, e nenhuma das duas barras
/// nomeava recurso nenhum: a mutação que leva as [`LATITUDES`] de `24` para
/// `96` — `738 → 3 042` vértices, uma peça **quatro vezes mais fina** —
/// **SOBREVIVEU** às duas. *Um limite que não diz de que recurso é, é um
/// palpite à espera de um smoke* (CLAUDE.md §0.0).
///
/// ⭐ **A barra DERIVA-SE, e os dois lados dela já existem no produto:** a
/// fileira tem quatro chips (`Mesh · 2x · 4x · 8x`) e a peça da `=51` é a
/// densidade em que a marca por vértice **já sai limpa** — foi essa cena que o
/// dono aprovou (*«temos a pintura funcionando»*). ⇒ a lei é
///
/// > **o chip que o roteiro manda carregar tem de ser o PRIMEIRO da fileira que
/// > alcança essa densidade.**
///
/// Ela aperta pelos dois lados sem uma constante escolhida:
///
/// * **peça fina de mais** — o `4x` já lá chega, e o degrau que esta cena
///   ensina no topo deixa de ter o que mostrar (é a mutação acima);
/// * **peça grossa de mais** — nem o `8x` chega, e a cena promete um detalhe
///   que o produto não entrega.
///
/// ⚠️ **A contagem sai da [`ph2d_mesh_colors::Tinta`] e NUNCA da fórmula `L²`:**
/// os pólos de uma esfera UV são leques de TRIÂNGULOS, e aquele multiplicador
/// só descreve quads — é a mesma armadilha que obrigou o gate do tecto a trocar
/// a esfera por um toro.
#[test]
fn o_oito_e_o_primeiro_degrau_que_alcanca_a_densidade_da_cena_irma() {
    use ph2d_panel_sculpt3d::state::DetalheDaTinta;

    /// Quantas amostras a fileira dá a esta malha em cada chip — pelo PRODUTO.
    fn amostras(m: &ph2d_mesh::Mesh, d: DetalheDaTinta) -> usize {
        match d.nivel() {
            None => m.vert_count(),
            Some(k) => ph2d_mesh_colors::Tinta::nova(
                m.vert_count(),
                m.faces().iter().map(ph2d_mesh::Face::verts),
                k,
            )
            .amostras()
            .len(),
        }
    }

    fn primeiro_a_alcancar(m: &ph2d_mesh::Mesh, limpa: usize) -> Option<DetalheDaTinta> {
        DetalheDaTinta::ALL
            .into_iter()
            .find(|d| amostras(m, *d) >= limpa)
    }

    let limpa = crate::scenes::pintura::peca().vert_count();
    let m = peca();
    // ⛔⛔ **A PREMISSA MORREU EM 2026-09-20 e a morte está aqui, à vista:** esta
    // linha era `*DetalheDaTinta::ALL.last()` — *«o degrau da lição É o topo da
    // fileira»* —, e as duas grandezas coincidiam **por acidente** até o dono
    // mandar acrescentar o `16×`. A lei que fica é a que a cena de facto
    // ensina, e ela mora numa const com dois consumidores.
    let licao = super::DEGRAU_DA_LICAO;

    let tabela: Vec<(DetalheDaTinta, usize)> = DetalheDaTinta::ALL
        .into_iter()
        .map(|d| (d, amostras(&m, d)))
        .collect();
    assert_eq!(
        primeiro_a_alcancar(&m, limpa),
        Some(licao),
        "a densidade LIMPA é {limpa} amostras (a peça da =51, a cena que o dono \
         aprovou) e esta peça lê {tabela:?}: o chip que o roteiro manda carregar \
         deixou de ser o primeiro a alcançá-la"
    );

    // ⭐⭐ **E a const tem de ser a que o ROTEIRO NOMEIA** — sem esta metade ela
    // é uma segunda resposta à pergunta *«que degrau esta cena ensina?»*, e as
    // duas divergem no dia em que alguém reescrever o texto.
    const ESTA_CENA: &str = include_str!("scenes_tinta_fina.rs");
    let agulha = format!("Carregue no `{}`", licao.label());
    assert!(
        ESTA_CENA.contains(&agulha),
        "o roteiro deixou de mandar carregar no `{}` -- a const e o texto \
         separaram-se, e o gate acima passa a medir um degrau que o dono nunca vê",
        licao.label()
    );

    // ⭐⭐ **O CONTROLO é a própria mutação** — sem ele esta régua podia estar a
    // responder `topo` por vácuo, e a fixtura tem de CONTER o fenómeno.
    let fina = ph2d_mesh::shapes::uv_sphere(LATITUDES * 4, LONGITUDES, 1.0);
    let acha = primeiro_a_alcancar(&fina, limpa);
    assert!(
        acha.is_some() && acha != Some(licao),
        "o CONTROLO desta régua não reproduz o fenómeno: uma peça 4x mais fina \
         ({} vértices) devia alcançar a densidade limpa ANTES do degrau da lição, \
         e ela lê {acha:?}",
        fina.vert_count()
    );
}

/// ⭐⭐⭐⭐ **TODO NOME QUE O ROTEIRO PÕE ENTRE CRASES É UM NOME QUE O DONO LÊ NA
/// TELA** — o molde da irmã `=51`, com a população que esta cena acrescenta.
///
/// ⚠️⚠️ **E ela acrescenta UMA população nova: os chips da fileira nova.** Os
/// rótulos deles saem do [`ph2d_panel_sculpt3d::state::DetalheDaTinta`] e não de
/// um `tr("` do pintor, logo a colheita da irmã **não os vê** — *um censo sobre
/// UMA das populações lê-se, num relatório, como um censo sobre todas*, e é
/// exactamente a forma que a `=51` pagou quando a caixa de cor nasceu.
#[test]
fn todo_nome_entre_crases_do_roteiro_existe_na_tela() {
    use ph2d_i18n::tr;
    use ph2d_panel_sculpt3d::rows::rows;
    use ph2d_panel_sculpt3d::state::{DetalheDaTinta, Sculpt3dUi, UiLevel};

    /// O que o roteiro nomeia e **não** é um rótulo de tela: teclas.
    const TECLAS: &[&str] = &[
        "P",
        "Ctrl+Z",
        "Ctrl+Shift+Z",
        "Ctrl+Shift+S",
        "Ctrl+O",
        "Ctrl+Shift+E",
    ];

    const ESTA_CENA: &str = include_str!("scenes_tinta_fina.rs");
    let roteiro: Vec<&str> = ESTA_CENA
        .lines()
        .filter(|l| !l.trim_start().starts_with("//") && l.contains("[sculpt3d]"))
        .collect();
    assert!(
        roteiro.len() > 25,
        "a colheita do roteiro deu {} linhas: a extracção partiu-se",
        roteiro.len()
    );
    let roteiro = roteiro.join("\n");

    let crases = roteiro.matches('`').count();
    assert_eq!(
        crases % 2,
        0,
        "o roteiro tem {crases} crases, um número ÍMPAR: há uma solta, e o censo \
         abaixo deixa de saber onde acaba um nome"
    );

    let mut ui = Sculpt3dUi {
        ui_level: UiLevel::Basic,
        ..Sculpt3dUi::default()
    };
    ui.brush.verb = verbo_da_cena(true).expect("a cena declara um verbo");

    let mut na_tela: Vec<String> = Vec::new();
    na_tela.extend(
        rows()
            .filter(|r| r.visible(&ui))
            .map(|r| tr(r.label).to_string()),
    );
    na_tela.extend(
        ph2d_sculpt3d::Verb::ALL
            .iter()
            .map(|v| v.label().to_string()),
    );
    // ⭐ **A POPULAÇÃO NOVA** — os chips da fileira que esta cena ensina.
    na_tela.extend(DetalheDaTinta::ALL.iter().map(|d| d.label().to_string()));
    const PINTOR: &str = include_str!("../../ph2d-panel-sculpt3d/src/paint/body.rs");
    const PINTOR_COR: &str = include_str!("../../ph2d-panel-sculpt3d/src/paint/brush_cor.rs");
    for fonte in [PINTOR, PINTOR_COR] {
        for pedaco in fonte.split("tr(\"").skip(1) {
            if let Some(chave) = pedaco.split('"').next() {
                na_tela.push(tr(chave).to_string());
            }
        }
    }
    na_tela.extend(TECLAS.iter().map(|s| (*s).to_string()));

    let pintados: Vec<&String> = na_tela.iter().take(na_tela.len() - TECLAS.len()).collect();
    for t in TECLAS {
        assert!(
            !pintados.iter().any(|p| p.as_str() == *t),
            "`{t}` está na lista de teclas E é um rótulo que o painel pinta: a \
             excepção deixou de ser uma excepção"
        );
    }
    assert!(
        pintados.len() > 40,
        "o censo colheu só {} rótulos de tela: a extracção partiu-se",
        pintados.len()
    );

    let mut nomeados = 0usize;
    for (i, pedaco) in roteiro.split('`').enumerate() {
        if i % 2 == 0 {
            continue;
        }
        nomeados += 1;
        assert!(
            !pedaco.contains('\n'),
            "o roteiro parte um nome em duas linhas ({pedaco:?})"
        );
        assert!(
            na_tela.iter().any(|n| n == pedaco),
            "o roteiro manda o dono procurar `{pedaco}` e NADA na tela se chama \
             assim — ele vai procurar uma linha que não existe."
        );
    }
    assert!(
        nomeados >= 6,
        "o roteiro nomeia só {nomeados} controlos: este censo ficou sem sujeito"
    );
}

/// ⭐⭐⭐⭐ **A PEÇA DESTA CENA NÃO TEM DE QUE IGUALAR, E O ROTEIRO CALA-SE —
/// as duas metades, porque cada uma sozinha mente.**
///
/// ⛔⛔⛔ **Este gate chamava-se `a_peca_da_cena_tem_de_que_igualar` e o report
/// do dono de 23/09 inverteu-o.** Ele exigia que o `Even Detail` fizesse algo
/// nesta peça, e fazia: a lei ancorava na MEDIANA e as faces do pólo — mais
/// pequenas — **desciam** para se igualarem às do equador. *O dono viu isso
/// como o que era* (**«com even detail o resultado é pior em todas as áreas e
/// em todas as resoluções inclusive a 16x; a resolução fica bem baixa»**), e a
/// cura foi o `k` passar a ser um **PISO** ([`niveis_igualados`]).
///
/// ⭐⭐⭐ **Com o piso, ela é INERTE nesta peça, e isso é uma propriedade da
/// FORMA e não um defeito:** numa esfera UV a área de uma face é `∝ sin θ`,
/// logo a maioria vive perto do equador, que é onde elas são MAIORES ⇒ **a
/// mediana senta-se no topo da distribuição** e ninguém está acima dela.
/// Medido: `768` faces, dispersão de área `15,3×` (`1,97` degraus da escada) e
/// **`0`** faces a subir.
///
/// ⚠️⚠️ **A dispersão NÃO é a pergunta, e é por isso que este gate a mede e não
/// a acusa:** o que decide é de que LADO da mediana ela está. Uma peça pode ter
/// `15×` de dispersão e nada para uma lei de um sentido só fazer.
///
/// ⭐ **E o CONTROLO é um CILINDRO**, sem o qual isto passaria com a lei
/// APAGADA: as duas tampas são leques de triângulos gordos e os lados são
/// quadriláteros finos ⇒ `24` das `72` faces sobem um degrau. *A lei está viva;
/// é esta peça que não é sujeito dela.*
#[test]
fn a_peca_da_cena_nao_tem_de_que_igualar_e_o_roteiro_cala_se() {
    let k = super::DEGRAU_DA_LICAO
        .nivel()
        .expect("o degrau da lição arma um plano");

    let niveis_de = |m: &ph2d_mesh::Mesh| -> Vec<u8> {
        let faces = || m.faces().iter().map(ph2d_mesh::Face::verts);
        let topo = ph2d_mesh_colors::Topologia::nova(m.vert_count(), faces(), 0);
        ph2d_mesh_colors::niveis_igualados(&topo, &m.face_areas(), k, 1)
    };
    let distintos = |ns: &[u8]| {
        let mut d = ns.to_vec();
        d.sort_unstable();
        d.dedup();
        d
    };

    // (1) A peça da cena: a lei corre e não tem o que subir.
    let m = super::peca();
    let ns = niveis_de(&m);
    assert_eq!(
        distintos(&ns),
        vec![k],
        "a peça da =52 passou a ter de que igualar — se isto acontecer, o \
         passo do roteiro pode VOLTAR, e ele foi retirado por ser inerte"
    );

    // (2) ⭐ E o CONTROLO: a lei NÃO está morta.
    let cil = ph2d_mesh::shapes::cylinder(24, 1.0, 2.0);
    let nc = niveis_de(&cil);
    assert!(
        distintos(&nc).len() >= 2,
        "o CONTROLO: num cilindro a igualação tem de subir alguma face, e saiu \
         {:?} — sem isto a metade (1) passa com a lei apagada",
        distintos(&nc)
    );
    assert!(
        nc.iter().all(|n| *n >= k),
        "e NINGUÉM desce abaixo do degrau pedido: {:?}",
        distintos(&nc)
    );
}

/// ⭐⭐⭐ **E O ROTEIRO NÃO NOMEIA A CAIXA `Even Detail`, porque nesta peça ela
/// é inerte** — a outra metade do gate acima.
///
/// ⚠️ *Um passo que manda carregar num interruptor que não faz nada é a espécie
/// que o §5.0 chama de **pior que uma cena ausente***, e ele existiu aqui entre
/// 22 e 23/09, com a lei que o dono reprovou por baixo.
///
/// ⛔ **A caixa FICA no painel**, e isso é deliberado: com o piso ela nunca
/// pode tirar resolução, logo o pior caso dela é não fazer nada. Quem quiser
/// repor o passo tem de trazer uma peça que seja sujeito dele — e a metade (1)
/// do gate acima reprova no dia em que esta passar a ser.
#[test]
fn o_roteiro_da_cena_nao_nomeia_a_caixa_da_tinta_igualada() {
    // ⚠️ O roteiro vive DENTRO do `eprintln!` (ele sai por terminal, que é a
    //    isenção do HR-15 — uma `const` solta perde-a), logo lê-se pelo
    //    ficheiro, como o gate irmão já fazia.
    let rotulo = ph2d_i18n::tr("panel.sculpt3d.tinta_igualada");
    let roteiro = include_str!("scenes_tinta_fina.rs");
    assert!(
        !roteiro.contains(&format!("`{rotulo}`")),
        "o roteiro voltou a nomear a caixa `{rotulo}` numa peça em que ela é \
         INERTE — reponha o passo só com uma peça que seja sujeito dele"
    );
}

/// ⭐⭐⭐ **A CAIXA `Even Detail` É UM INTERRUPTOR VIVO, e ela só é oferecida com
/// um plano armado.**
///
/// ⛔⛔ **A metade do ROTEIRO saiu daqui em 23/09 e a premissa dela está no
/// gate irmão acima:** ele exigia que a cena NOMEASSE a caixa, e desde que o
/// `k` é um piso ela é inerte nesta peça — *um passo que manda carregar num
/// interruptor que não faz nada é pior que uma cena ausente*. O que fica é o
/// que continua a ser verdade: **a caixa existe, é alcançável, e o painel
/// oferece-a exactamente onde ela tem sujeito.**
///
/// ⛔ E a metade NEGATIVA é a que a torna honesta: sem um plano armado não há
/// retícula para igualar, e oferecê-la seria um controlo morto sob o dedo.
#[test]
fn a_caixa_da_tinta_igualada_e_um_interruptor_vivo() {
    let mut ui = ph2d_panel_sculpt3d::Sculpt3dUi {
        tinta_detalhe: super::DEGRAU_DA_LICAO,
        ..Default::default()
    };
    assert!(
        ph2d_panel_sculpt3d::interruptor_oferecido(
            &ui,
            ph2d_panel_sculpt3d::ids::SCULPT3D_TINTA_IGUALADA
        ),
        "a caixa não é oferecida com o plano armado — o roteiro manda clicar \
         numa linha que o painel não desenha"
    );
    ui.tinta_detalhe = ph2d_panel_sculpt3d::state::DetalheDaTinta::Malha;
    assert!(
        !ph2d_panel_sculpt3d::interruptor_oferecido(
            &ui,
            ph2d_panel_sculpt3d::ids::SCULPT3D_TINTA_IGUALADA
        ),
        "a caixa é oferecida SEM plano — sem retícula não há o que igualar"
    );
}
/// ⛔⛔⛔ **A SONDA DO REPORT DE 23/09** (*«com Even Detail o resultado é pior
/// em todas as áreas e em todas as resoluções, inclusive a 16x — a resolução
/// fica bem baixa»*, com duas fotos).
///
/// Ela imprime, para a peça DESTA cena e para cada degrau da fileira, o que
/// cada uma das **três** âncoras possíveis entrega: o histograma de níveis, o
/// máximo, e a contagem de amostras contra o plano uniforme.
///
/// ⚠️ É um INSTRUMENTO e não uma lei — ela fica versionada porque a leitura de
/// hoje e a do dia em que isto voltar valem uma pela outra.
#[test]
fn diag_as_tres_ancoras_na_peca_da_cena() {
    use std::collections::BTreeMap;
    let m = super::peca();
    let faces = || m.faces().iter().map(ph2d_mesh::Face::verts);
    let areas = m.face_areas();
    let topo = ph2d_mesh_colors::Topologia::nova(m.vert_count(), faces(), 0);

    let (mut lo, mut hi) = (f32::MAX, 0.0f32);
    for a in areas.iter().filter(|a| **a > 0.0) {
        lo = lo.min(*a);
        hi = hi.max(*a);
    }
    eprintln!(
        "peca: {} faces, area {lo:.3e}..{hi:.3e} (razao {:.1}x, {:.2} degraus)",
        areas.len(),
        hi / lo,
        0.5 * (hi / lo).log2()
    );

    // As tres ancoras: o alvo de densidade sai da face MEDIANA, da MAIOR
    // (o piso: ninguem fica mais grosso do que o `k` daria) ou da MENOR.
    let mut d: Vec<f32> = areas
        .iter()
        .filter(|a| **a > 0.0)
        .map(|a| a.sqrt())
        .collect();
    d.sort_by(|a, b| a.partial_cmp(b).expect("sem NaN"));
    let raiz_med = d[(d.len() - 1) / 2];
    let raiz_min = d[0];
    let raiz_max = d[d.len() - 1];

    for k in 1..=4u8 {
        let lado = f32::from(1u16 << k);
        for (nome, raiz) in [
            ("MEDIANA", raiz_med),
            ("PISO   ", raiz_min),
            ("TECTO  ", raiz_max),
        ] {
            let ks = ph2d_mesh_colors::niveis_por_area(&topo, &areas, lado / raiz, 1);
            let mut h: BTreeMap<u8, usize> = BTreeMap::new();
            for x in &ks {
                *h.entry(*x).or_default() += 1;
            }
            let uni = ph2d_mesh_colors::Tinta::nova(m.vert_count(), faces(), k);
            let gra = ph2d_mesh_colors::Tinta::graduada(
                m.vert_count(),
                faces(),
                &ks,
                ks.iter().copied().min().unwrap_or(0),
            )
            .expect("a lista descreve a malha");
            let (u, g) = (uni.amostras().len(), gra.amostras().len());
            #[allow(clippy::cast_precision_loss)]
            let razao = g as f32 / u as f32;
            eprintln!("k={k} {nome}: hist {h:?}  amostras {u} -> {g} ({razao:.2}x)");
        }
    }
}

#[test]
fn diag_o_histograma_da_igualacao_na_peca_da_cena() {
    use std::collections::BTreeMap;
    let m = crate::scenes::tinta_fina::peca();
    let faces = || m.faces().iter().map(ph2d_mesh::Face::verts);
    let areas = m.face_areas();
    let topo = ph2d_mesh_colors::Topologia::nova(m.vert_count(), faces(), 0);
    let mut lo = f32::MAX;
    let mut hi = 0.0f32;
    for a in &areas {
        if *a > 0.0 {
            lo = lo.min(*a);
            hi = hi.max(*a);
        }
    }
    eprintln!(
        "peca: {} faces, area {lo:.3e}..{hi:.3e} (razao {:.1}x)",
        areas.len(),
        hi / lo
    );
    for k in 1..=4u8 {
        let ks = ph2d_mesh_colors::niveis_igualados(&topo, &areas, k, 1);
        let mut h: BTreeMap<u8, usize> = BTreeMap::new();
        for x in &ks {
            *h.entry(*x).or_default() += 1;
        }
        let uni = ph2d_mesh_colors::Tinta::nova(m.vert_count(), faces(), k);
        let gra = ph2d_mesh_colors::Tinta::graduada(
            m.vert_count(),
            faces(),
            &ks,
            ks.iter().copied().min().unwrap_or(0),
        )
        .unwrap();
        eprintln!(
            "k={k}: hist {h:?}  max={}  amostras {} -> {}",
            ks.iter().copied().max().unwrap_or(0),
            uni.amostras().len(),
            gra.amostras().len()
        );
    }
}
