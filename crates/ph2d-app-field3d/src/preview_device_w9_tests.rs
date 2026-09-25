//! ⭐⭐⭐⭐ **OS GATES DAS CURAS DA `W9`** — a fita inerte, a lei do dono e a cache do chão.
//!
//! As três curas têm a mesma forma e por isso o mesmo tipo de régua: **a saída é byte-idêntica por
//! construção e a economia é invisível a toda régua de valor** ⇒ cada uma afirma o PIXEL *e* a
//! CONTA, e cada uma leva o CONTROLO que a impede de ficar verde a afirmar nada.
//!
//! ⚠️ **Eles saíram do [`super`] por um tecto de LOC** (2026-09-23, `1 951` contra `700`), e a
//! fronteira que o tecto forçou é a certa: o irmão mede *o divisor depois do dispositivo* e estes
//! medem *o que as curas da `W9` compraram*.

use super::*;

/// ⭐⭐⭐⭐ **A FITA DA PEÇA SAI DO SHADER DO PINTOR E A IMAGEM NÃO MUDA UM BYTE.**
///
/// Ver [`ph2d_field_gpu::paint::PaintSetup::le_o_campo`] para o grafo de chamadas que o decidiu:
/// no quadro de MOVIMENTO, com nada a ler a curvatura, o `pinta` e o `pinta_bordas` alcançam a fita
/// por **nenhum** caminho que corra. ⇒ substituí-la por uma constante é byte-idêntico *por
/// construção*, e o que se compra é o texto do shader deixar de mudar com a peça.
///
/// ⚠️ **A cena leva CHÃO de propósito.** A primeira medição desta cura correu sem ele, e o chão é
/// exactamente onde um segundo caminho para a fita poderia estar escondido (o `ceu_do_chao` chama
/// `field()`): ele corre no passe da MARCHA, que continua a levar a fita, e esta metade é o que o
/// afirma em vez de o supor.
///
/// ⛔ **O CONTROLO vem primeiro:** uma imagem toda de fundo é trivialmente igual a outra imagem
/// toda de fundo. *Sem ele, apagar a peça faria este gate passar.*
#[test]
#[ignore = "precisa de GPU"]
fn a_fita_inerte_no_pintor_nao_muda_um_byte() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let materiais = [ph2d_material::OpenPbr::default().prepare()];
    let olhar = ph2d_view_transform::Look::default();
    const BG: [u8; 4] = [0, 0, 0, 0];
    let doc = crate::smoke::scene(0);
    let reg = crate::smoke::sampled_registry();
    let cam = ph2d_field_render::Orbit::default();
    let luz = [crate::gpu_frame::tests_lampada(&cam)];
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: None,
    };
    let chao = Some(ph2d_field_render::Ground { height: -1.0 });
    let pinta = |fita_inerte: bool| {
        crate::gpu_frame::paint_com(
            t,
            &doc,
            &reg,
            &cam,
            &luz,
            &surfaces,
            &ph2d_field_render::Presentation::of(olhar),
            BG,
            chao,
            LW,
            LH,
            false,
            crate::gpu_frame::Sonda {
                fita_inerte,
                ..crate::gpu_frame::Sonda::default()
            },
        )
        .expect("o pintor")
    };
    let com = pinta(true);
    let sem = pinta(false);

    // ── O CONTROLO: a peça está na imagem ─────────────────────────────────────────────────────
    let do_fundo = sem
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| **p == BG)
        .count();
    let total = sem.rgba.len() / 4;
    assert!(
        do_fundo * 10 < total * 9 && do_fundo * 10 > total,
        "CONTROLO: {do_fundo} de {total} píxeis são o fundo — uma imagem quase vazia (ou quase \
         cheia) é trivialmente igual a outra, e este gate não afirmaria nada"
    );

    assert_eq!(
        com.rgba, sem.rgba,
        "a fita inerte mudou a imagem — alguma coisa no quadro de MOVIMENTO LÊ o campo da peça \
         dentro do pintor, e o predicado `le_o_campo` não a nomeia"
    );
}

/// ⭐⭐⭐⭐ **E A ECONOMIA MEDE-SE PELA CONTA, porque é invisível à imagem.**
///
/// O gate irmão afirma que as duas rotas dão a MESMA imagem — e é exactamente isso que torna a cura
/// impossível de medir por valor: *uma cura apagada também não muda a imagem*. ⇒ a régua é o
/// [`ph2d_field_gpu::trace::Tracer::compiled`], a contagem de pipelines que o cache já compilou.
///
/// **O mecanismo:** o cache tem por chave o TEXTO do shader. Com a fita da peça lá dentro, cada
/// estrutura nova é um texto novo e compila **tudo**; com ela fora, só os passes da MARCHA — que
/// levam a fita de verdade — é que recompilam.
///
/// ⚠️ **A segunda peça tem de ser outra ESTRUTURA e não outro número:** arrastar um raio já não
/// recompilava nada antes desta cura (o texto não muda, só o armazém `k`), e uma fixtura assim
/// mediria zero dos dois lados.
///
/// **Medido (2026-09-21): `SEM a cura cresceu 4 pipelines · COM a cura cresceu 2`** — os dois que
/// ficam são os da MARCHA (`centro_e_luz` e `bordas`), que levam a fita de verdade; os dois que
/// saem são o `pinta` e o `pinta_bordas`.
///
/// ⛔⛔ **ELE LÊ UM CONTADOR GLOBAL, logo não sobrevive a correr em PARALELO com os irmãos.** O
/// [`ph2d_field_gpu::trace::Tracer`] vive num `Arc<Mutex<_>>` do módulo e o `compiled()` conta o
/// processo inteiro: sob `cargo test` (threads dentro de UM processo) um irmão a pintar no meio do
/// `cresce` entra nesta conta. ⭐ Sob o `nextest` — que é o portão desta casa e dá **um processo
/// por teste** — ele é sólido. *A prova de mutação desta wave leu «SOBREVIVEU» duas vezes por
/// causa disto, e a cura foi `--test-threads=1` no ARNÊS, nunca uma barra mais frouxa aqui.*
#[test]
#[ignore = "precisa de GPU"]
fn a_fita_inerte_faz_o_cache_acertar_na_peca_seguinte() {
    use ph2d_field::{FieldDoc, NodeId, Primitive, Xform};
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let materiais = [ph2d_material::OpenPbr::default().prepare()];
    let olhar = ph2d_view_transform::Look::default();
    const BG: [u8; 4] = [0, 0, 0, 0];
    let reg = crate::smoke::sampled_registry();
    let cam = ph2d_field_render::Orbit::default();
    let luz = [crate::gpu_frame::tests_lampada(&cam)];
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: None,
    };
    // ⛔⛔⛔ **OS DOIS LADOS TÊM DE SER PRIMITIVAS DIFERENTES, e a 1.ª redacção deste gate era um
    // VÁCUO por causa disso.** Ela variava o RAIO entre os dois lados — e um raio é uma
    // **constante da fita** (`Instr::Const` → `k[i]`), logo duas bolas de raios diferentes dão o
    // MESMO texto de shader. ⇒ o 2.º lado acertava no cache que o 1.º acabara de encher, lia
    // crescimento **ZERO**, e `com < sem` passava até com a cura apagada. *Uma mutação SOBREVIVENTE
    // mostrou-o: o gate media o cache já quente, não a cura.*
    let uma = |k: bool| {
        FieldDoc::new(
            vec![ph2d_field_eval::leaf(
                if k {
                    Primitive::Sphere { radius: 0.5 }
                } else {
                    Primitive::Box {
                        half: [0.4, 0.4, 0.4],
                        round: 0.0,
                        chamfer: 0.0,
                    }
                },
                Xform::at(0.0, 0.0, 0.0),
            )],
            NodeId(0),
        )
        .expect("a peça de uma")
    };
    let duas = |k: bool| {
        let folha = |x: f32| {
            ph2d_field_eval::leaf(
                if k {
                    Primitive::Sphere { radius: 0.4 }
                } else {
                    Primitive::Box {
                        half: [0.3, 0.3, 0.3],
                        round: 0.0,
                        chamfer: 0.0,
                    }
                },
                Xform::at(x, 0.0, 0.0),
            )
        };
        FieldDoc::new(
            vec![
                folha(-0.3),
                folha(0.3),
                ph2d_field::Node {
                    xform: Xform::IDENTITY,
                    kind: ph2d_field::NodeKind::Combine {
                        op: ph2d_field::Op::Union(ph2d_field::Blend::Sharp),
                        children: vec![NodeId(0), NodeId(1)],
                    },
                    mods: Vec::new(),
                    verb: None,
                },
            ],
            NodeId(2),
        )
        .expect("a peça de duas")
    };
    let pinta = |doc: &FieldDoc, fita_inerte: bool| {
        crate::gpu_frame::paint_com(
            t,
            doc,
            &reg,
            &cam,
            &luz,
            &surfaces,
            &ph2d_field_render::Presentation::of(olhar),
            BG,
            None,
            LW,
            LH,
            false,
            crate::gpu_frame::Sonda {
                fita_inerte,
                ..crate::gpu_frame::Sonda::default()
            },
        )
        .expect("o pintor");
    };
    let conta = || t.lock().expect("o traçador").compiled();

    let cresce = |esfera: bool, fita_inerte: bool| {
        pinta(&uma(esfera), fita_inerte);
        let antes = conta();
        pinta(&duas(esfera), fita_inerte);
        conta() - antes
    };
    // ⚠️ **Cada lado na sua FAMÍLIA de primitiva**, pela razão do bloco acima: assim as quatro
    // peças são quatro textos, e o crescimento que cada lado lê é o dele.
    let sem = cresce(true, false);
    let com = cresce(false, true);
    println!("  SEM a cura cresceu {sem} pipelines · COM a cura cresceu {com}");

    assert!(
        sem > 0,
        "CONTROLO: sem a cura, mudar a ESTRUTURA da peça compilou {sem} pipelines — se é zero, a \
         fixtura não muda o texto do shader e o gate mede o nada"
    );
    assert!(
        com < sem,
        "com a fita inerte a peça seguinte compilou {com} pipelines e sem ela {sem} — a cura não \
         está a tirar a peça do texto do pintor"
    );
}

/// ⭐⭐⭐⭐ **A METADE QUE TORNA AS OUTRAS DUAS LOAD-BEARING: quem LÊ o campo continua a levá-lo.**
///
/// As duas irmãs medem o caminho em que a fita SAI. Sozinhas, elas ficam verdes sobre uma cura que
/// a tirasse **sempre** — e aí o jade (que lê a curvatura) e o ricochete (que marcha) passariam a
/// ler um campo constante, em silêncio. ⇒ *este gate corre os dois regimes em que a fita TEM de
/// ficar, e afirma que a porta é inerte lá.*
///
/// ⛔⛔ **E o CONTROLO de cada metade é a própria fixtura:** um material que não lê a curvatura, ou
/// um quadro de movimento, não distinguem uma cura certa de uma cura apagada. Por isso a 1.ª metade
/// usa **subsuperfície MACIÇA** (a condição exacta do `Surface::reads_curvature`) e a 2.ª o quadro
/// **ASSENTE** (onde o `ao_rays` abre).
#[test]
#[ignore = "precisa de GPU"]
fn quem_le_o_campo_continua_a_leva_lo_no_shader() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let olhar = ph2d_view_transform::Look::default();
    const BG: [u8; 4] = [0, 0, 0, 0];
    let doc = crate::smoke::scene(1);
    let reg = crate::smoke::sampled_registry();
    let cam = ph2d_field_render::Orbit::default();
    let luz = [crate::gpu_frame::tests_lampada(&cam)];
    let pinta = |mats: &[ph2d_material::Surface], assente: bool, fita_inerte: bool| {
        let surfaces = ph2d_field_render::Surfaces {
            all: mats,
            owners: None,
        };
        crate::gpu_frame::paint_com(
            t,
            &doc,
            &reg,
            &cam,
            &luz,
            &surfaces,
            &ph2d_field_render::Presentation::of(olhar),
            BG,
            None,
            LW,
            LH,
            assente,
            crate::gpu_frame::Sonda {
                fita_inerte,
                ..crate::gpu_frame::Sonda::default()
            },
        )
        .expect("o pintor")
    };

    // ── 1. O JADE: subsuperfície MACIÇA é a condição exacta do `Surface::reads_curvature` ──────
    let jade = [ph2d_material::OpenPbr {
        subsurface_weight: 1.0,
        geometry_thin_walled: false,
        ..ph2d_material::OpenPbr::default()
    }
    .prepare()];
    assert!(
        jade[0].reads_curvature(),
        "CONTROLO: a fixtura do jade não lê a curvatura — a metade abaixo mediria o nada"
    );
    assert_eq!(
        pinta(&jade, false, true).rgba,
        pinta(&jade, false, false).rgba,
        "com um material que LÊ a curvatura a porta tem de ser inerte — se a imagem muda, a cura \
         está a tirar a fita a quem precisa dela"
    );

    // ── 2. O QUADRO ASSENTE: ali o `ao_rays` abre e o `assa_sondas` MARCHA ─────────────────────
    let liso = [ph2d_material::OpenPbr::default().prepare()];
    assert!(
        !liso[0].reads_curvature(),
        "CONTROLO: o material de omissão lê a curvatura — então a metade 2 não isola o ricochete"
    );
    assert_eq!(
        pinta(&liso, true, true).rgba,
        pinta(&liso, true, false).rgba,
        "no quadro ASSENTE a porta tem de ser inerte — o `assa_sondas` marcha o campo, e uma fita \
         constante dá-lhe um mundo cheio"
    );

    // ── 3. O ESTILO: a tinta de CURVATURA é o terceiro leitor, e ele lê pela apresentação ──────
    //
    // ⛔⛔ **Esta metade nasceu de uma MUTAÇÃO SOBREVIVENTE (auditoria de 2026-09-23):** apagar
    // `|| pres.reads_curvature()` da porta [`ph2d_field_render::curvatura::alguem_le`] passava as
    // duas metades acima — elas medem o MATERIAL e o RICOCHETE, e nenhuma acorda o estilo. *Uma
    // porta com duas metades e um gate que exercita uma delas é meia porta.*
    //
    // ⚠️ A condição exacta é a do `ph2d_style::Style::reads_curvature`: uma das duas tintas de
    // curvatura deixar de ser o neutro.
    let mut estilo = ph2d_style::Style::default();
    estilo.curvature.concave = [0.2, 0.3, 0.9];
    assert!(
        estilo.reads_curvature(),
        "CONTROLO: a fixtura do estilo não lê a curvatura — a metade abaixo mediria o nada"
    );
    let com_estilo = ph2d_field_render::Presentation {
        style: estilo,
        ..ph2d_field_render::Presentation::of(olhar)
    };
    let pinta_com_estilo = |fita_inerte: bool| {
        let surfaces = ph2d_field_render::Surfaces {
            all: &liso,
            owners: None,
        };
        crate::gpu_frame::paint_com(
            t,
            &doc,
            &reg,
            &cam,
            &luz,
            &surfaces,
            &com_estilo,
            BG,
            None,
            LW,
            LH,
            false,
            crate::gpu_frame::Sonda {
                fita_inerte,
                ..crate::gpu_frame::Sonda::default()
            },
        )
        .expect("o pintor")
    };
    assert_eq!(
        pinta_com_estilo(true).rgba,
        pinta_com_estilo(false).rgba,
        "com a TINTA DE CURVATURA do estilo ligada a porta tem de ser inerte — se a imagem muda, \
         a cura está a tirar a fita a quem precisa dela pela apresentação"
    );
}

/// ⭐⭐⭐⭐ **RETIRAR A LEI DO DONO A UMA PEÇA DE MATERIAL ÚNICO NÃO MUDA UM BYTE.**
///
/// A decisão vive no [`crate::materials::Table::build`] e o gate dela é o
/// `n_folhas_com_o_mesmo_material_nao_pedem_lei_do_dono`, que afirma a ESCOLHA. Este afirma o
/// **PIXEL**, que é o que a escolha promete: *com todos os materiais iguais o `dono_mix` devolve um
/// par cujos `ler_mat` dão o MESMO `Mat`, logo a lei calcula `ca + (ca − ca) · t` — que é `ca`
/// exactamente, porque o termo é `0,0 · t`.*
///
/// ⛔⛔ **Sem esta metade a cura seria uma promessa de álgebra.** Ela vale `300×` no caminho do
/// artista (acrescentar uma forma: `2 310 → 7,85 ms`), e uma cura desse tamanho sobre uma conta que
/// ninguém correu é onde um defeito mudo se instala.
///
/// ⭐ **E o CONTROLO é a segunda metade:** com materiais DISTINTOS a lei do dono TEM de mudar a
/// imagem. Sem ele, um `dono_mix` que devolvesse sempre a folha `0` passava a primeira metade e
/// pintava a peça inteira com o material da primeira folha.
#[test]
#[ignore = "precisa de GPU"]
fn a_lei_do_dono_e_inerte_numa_peca_de_material_unico() {
    use ph2d_field::{FieldDoc, NodeId, Primitive, Xform};
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let olhar = ph2d_view_transform::Look::default();
    const BG: [u8; 4] = [0, 0, 0, 0];
    let reg = crate::smoke::sampled_registry();
    let cam = ph2d_field_render::Orbit::default();
    let luz = [crate::gpu_frame::tests_lampada(&cam)];

    let folha =
        |x: f32| ph2d_field_eval::leaf(Primitive::Sphere { radius: 0.35 }, Xform::at(x, 0.0, 0.0));
    let doc = FieldDoc::new(
        vec![
            folha(-0.3),
            folha(0.3),
            ph2d_field::Node {
                xform: Xform::IDENTITY,
                kind: ph2d_field::NodeKind::Combine {
                    op: ph2d_field::Op::Union(ph2d_field::Blend::Sharp),
                    children: vec![NodeId(0), NodeId(1)],
                },
                mods: Vec::new(),
                verb: None,
            },
        ],
        NodeId(2),
    )
    .expect("as duas");
    let postas: Vec<FieldDoc> = [-0.3f32, 0.3]
        .into_iter()
        .map(|x| FieldDoc::new(vec![folha(x)], NodeId(0)).expect("a folha"))
        .collect();
    let owners = ph2d_field_eval::owners::Owners::new(
        &postas,
        &reg,
        ph2d_field_render::hit_tolerance(
            cam.half_extent,
            f32::from(u16::try_from(LH).unwrap_or(u16::MAX)),
        ),
    );
    let pinta = |mats: &[ph2d_material::Surface], com_dono: bool| {
        let surfaces = ph2d_field_render::Surfaces {
            all: mats,
            owners: com_dono.then_some(&owners),
        };
        crate::gpu_frame::paint(
            t,
            &doc,
            &reg,
            &cam,
            &luz,
            &surfaces,
            &ph2d_field_render::Presentation::of(olhar),
            BG,
            None,
            LW,
            LH,
            false,
        )
        .expect("o pintor")
    };

    // ── 1. MATERIAL ÚNICO: a lei do dono é inerte ao bit ──────────────────────────────────────
    let iguais = [
        ph2d_material::OpenPbr::default().prepare(),
        ph2d_material::OpenPbr::default().prepare(),
    ];
    let com = pinta(&iguais, true);
    let sem = pinta(&iguais, false);
    let do_fundo = sem
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| **p == BG)
        .count();
    let total = sem.rgba.len() / 4;
    assert!(
        do_fundo * 10 < total * 9 && do_fundo * 10 > total,
        "CONTROLO: {do_fundo} de {total} píxeis são o fundo — uma imagem quase vazia é \
         trivialmente igual a outra"
    );
    assert_eq!(
        com.rgba, sem.rgba,
        "retirar a lei do dono a uma peça de material ÚNICO mudou a imagem — a conta \
         `ca + (ca − ca) · t` não está a dar `ca`, e a decisão do `Table::build` é insegura"
    );

    // ── 2. O CONTROLO: com materiais DISTINTOS ela TEM de mudar a imagem ──────────────────────
    let distintos = [
        ph2d_material::OpenPbr {
            base_color: [0.9, 0.1, 0.1],
            ..ph2d_material::OpenPbr::default()
        }
        .prepare(),
        ph2d_material::OpenPbr {
            base_color: [0.1, 0.1, 0.9],
            ..ph2d_material::OpenPbr::default()
        }
        .prepare(),
    ];
    assert_ne!(
        pinta(&distintos, true).rgba,
        pinta(&distintos, false).rgba,
        "CONTROLO: com materiais DISTINTOS a lei do dono não mudou a imagem — então a metade de \
         cima não afirma nada, e um `dono_mix` que devolvesse sempre a folha 0 passaria"
    );
}

/// ⛔⛔⛔ **O VASO DESCE POR FÓRMULA, E A FITA DELE CABE NUMA MÃO** — e este gate SUBSTITUI um cuja
/// premissa morreu no mesmo dia.
///
/// # A premissa que morreu, e ela era minha
///
/// De manhã este sítio tinha o `nenhuma_regiao_do_vaso_paga_a_arvore_inteira`: ele varria uma
/// grelha `32×32` em `(u, v)`, compilava a região de cada célula e exigia que o pior caso cortasse
/// pelo menos `2×` — a cura da costura do eixo. ⚠️ **À tarde o dono mandou o torno descer por
/// FÓRMULA**, e uma fórmula **não tem arestas para cortar**: a região dela é a identidade, o pior
/// caso é `1,0×`, e o gate reprovou sobre produto correcto.
///
/// ⭐ A LEI da costura do eixo continua gateada, no sítio onde ela vive e sobre uma fixtura que a
/// contém — `ph2d-field-eval`,
/// `profile_index::eixo_tests::uma_regiao_sobre_a_costura_nao_paga_a_arvore_inteira`. O que se
/// perdeu foi a metade *«na peça do dono»*, porque a peça do dono **deixou de tomar aquele
/// caminho**. ⇒ *o que fica aqui é a afirmação NOVA que o produto passou a fazer.*
///
/// # O que ele afirma
///
/// Que a peça da cena `5` — `24` primitivas desenhadas, `931` linhas pela lei exacta — desce em
/// **`124`** linhas, e que a fita dela é pequena o bastante para o prévio não ter de baixar a
/// resolução (medido: o divisor foi de `2` para `1`, `32,53 → 16,63 ms`).
///
/// ⭐ **E o CONTROLO é a lei exacta ao lado**: sem ele, uma fórmula que devolvesse uma constante
/// passaria a primeira metade.
#[test]
fn o_vaso_desce_por_formula_e_a_fita_cabe_numa_mao() {
    // ⚠️ Medido: `124` linhas pela fórmula contra `931` pela lei exacta. A barra é `200` — folga de
    // `60 %` sobre o medido e `4,6×` abaixo da lei exacta —, ⛔ não um número redondo: ela é o degrau
    // em que a fita deixa de caber no orçamento do prévio com divisor `1` (`4,32 + 0,030 × 200 ≈
    // 10,3 ms` contra os `16,7`).
    const TECTO: usize = 200;
    let doc = crate::smoke::scenes::vaso(ph2d_field::DEFAULT_PROFILE_RESOLUTION);
    let ph2d_field::NodeKind::Leaf(ph2d_field::Primitive::Revolve { profile }) =
        &doc.nodes()[0].kind
    else {
        panic!("a cena 5 é um Revolve — se deixou de ser, este gate mede outra peça");
    };
    let pela_formula = ph2d_field_eval::Field::new(&doc)
        .tape_shape()
        .expect("a fita do torno")
        .guardados;
    let exacta = ph2d_field_eval::Field::from_tree(
        &ph2d_field_eval::profile::probe_sd_revolve_exacto(profile),
    )
    .tape_shape()
    .expect("a fita do contorno desenhado")
    .guardados;
    assert!(
        pela_formula <= TECTO,
        "a peça da cena 5 desce em {pela_formula} linhas contra o tecto de {TECTO} — ou a cerca da \
         fidelidade passou a recusá-la (e ela voltou ao contorno desenhado), ou o grau subiu"
    );
    // ⭐ **O CONTROLO**: a lei exacta tem de ser MUITO maior, senão este gate não mede cura nenhuma.
    assert!(
        exacta >= 4 * pela_formula,
        "CONTROLO: a lei exacta desce em {exacta} linhas contra {pela_formula} da fórmula — sem uma \
         ordem de grandeza entre as duas, a metade de cima não afirma nada"
    );
}

/// ⭐⭐⭐⭐ **O MODO DE OMISSÃO DO MODELADOR VAI À PLACA** — e este gate substitui a catraca que
/// afirmava o contrário.
///
/// # ⛔ A dívida que aqui vivia, e como ela morreu
///
/// Até 2026-09-23 este ficheiro tinha o
/// `a_marcha_no_dispositivo_ainda_pergunta_o_modo_e_isso_e_divida`: uma catraca **AO CONTRÁRIO**,
/// que exigia que a condição do dispositivo nomeasse `Shading::Render` — porque o `#[default]` é o
/// **`Matcap`** e, sem ele na placa, o arrasto de uma cena recém-aberta era **todo** de CPU
/// (`90,17 ms` e `D=3` contra `16,63` e `D=1`).
///
/// ⚠️⚠️ **A cura NÃO foi tirar o modo daquela condição** — e é por isso que aquela catraca teria
/// ficado **VERDE sobre a dívida paga**, que é o pior estado possível de um gate. A leitura dela
/// era que havia **uma** pergunta a separar; medida, há **duas leis de pintura**:
///
/// | lei | o que ela lê | armazéns |
/// |---|---|---:|
/// | material | materiais · céu · olhar · lâmpadas · curvatura | `9` |
/// | **matcap** | a normal de vista e uma fotografia | **`8`** |
///
/// ⇒ a condição do material **continua** a nomear o `Render` (e está certa: aquela lei precisa do
/// que só o `Render` monta), e o matcap ganhou um **ramo e um passe próprios**. ⭐ O piso garantido
/// do WebGPU é `8`: *o modo de omissão corre em toda placa conforme, e o de material só onde há
/// folga* ([`ph2d_field_gpu::matcap::ARMAZENS`] contra [`ph2d_field_gpu::paint::ARMAZENS`], com
/// gate naquela crate).
///
/// ⛔⛔ **E a leitura de que abrir a MARCHA ao matcap era a cura estava REFUTADA por medição:** o
/// [`crate::gpu_frame::march`] devolve o G-buffer pelo barramento (`49,8 MB` a `1920×1080`,
/// `119`–`123 ms`) — *mais lento do que a CPU inteira*. O que ganha é a IMAGEM não atravessar o
/// barramento.
///
/// # ⚠️ A régua lê o CÓDIGO e não a prosa
///
/// O comentário que explica a wave nomeia o `Shading` meia dúzia de vezes, e uma varredura do
/// ficheiro inteiro acusaria a própria nota. ⇒ cada metade recorta a **expressão** dela.
#[test]
fn o_modo_de_omissao_e_pintado_no_dispositivo() {
    const FONTE: &str = include_str!("smoke_draw_thread.rs");

    // ⭐⭐⭐ **METADE 1 — o matcap tem ramo e ele passa pela porta do dispositivo.**
    let agulha = "if matches!(p.shading, crate::shading::Shading::Matcap)";
    let i = FONTE.find(agulha).expect(
        "⛔ O RAMO DO MATCAP NO DISPOSITIVO DESAPARECEU: o modo de OMISSÃO do modelador volta a \
         traçar inteiro na CPU (`90,17 ms` e `D=3` contra `16,63` e `D=1`), que é o report \
         «ao arrastar fica grosseiro ainda» de 2026-09-23 a voltar",
    );
    let resto = &FONTE[i..];
    let ramo = &resto[..resto.find('{').expect("um `if` abre um bloco")];
    assert!(
        ramo.contains("takes_the_frame"),
        "o ramo do matcap deixou de passar pela porta do dispositivo: {ramo}"
    );
    assert!(
        resto.contains("pinta_matcap("),
        "o ramo do matcap existe e NÃO chama o passe que pinta — um ramo que cai para a CPU em \
         silêncio lê-se, no relógio, exactamente como não ter ramo nenhum"
    );

    // ⭐⭐ **METADE 2 — o CONTROLO: são DUAS leis, e a do material continua a pedir o modo.**
    //
    // ⚠️ Sem esta metade, colapsar as duas condições numa só ficaria verde — e o material seria
    // despachado sem os materiais, o céu e o olhar que só o `Render` monta.
    let j = FONTE
        .find("let pelo_dispositivo =")
        .expect("a condição do pintor de MATERIAL saiu do despacho do quadro");
    let expressao = {
        let r = &FONTE[j..];
        &r[..r.find(';').expect("uma atribuição acaba num `;`")]
    };
    assert!(
        expressao.contains("takes_the_frame") && expressao.contains("Shading::Render"),
        "as duas leis de pintura colapsaram numa: o pintor de MATERIAL precisa do que só o \
         `Render` monta, e o matcap não ({expressao})"
    );
}

/// ⭐⭐⭐⭐ **Os gates da cache do campo do chão** — ver o cabeçalho do [`chao`].
#[path = "preview_device_w9_chao_tests.rs"]
mod chao;

/// ⭐⭐⭐⭐ **Os gates das sondas guardadas na placa** — ver o cabeçalho do [`sondas`].
#[path = "preview_device_w9_sondas_tests.rs"]
mod sondas;
