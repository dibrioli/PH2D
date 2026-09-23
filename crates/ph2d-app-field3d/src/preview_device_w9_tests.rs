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

/// ⭐⭐⭐⭐ **O CAMPO DO CHÃO REAPROVEITADO NÃO MUDA UM BYTE — e a cache tem de ACERTAR.**
///
/// A assadura custa `+4,98 ms` por quadro assente e o campo não depende de para onde a câmera olha
/// ([`crate::gpu_frame::ChaveDoChao`]) — orbitar é o gesto que a paga. Este gate afirma as duas
/// metades: a imagem não muda, **e** a cache de facto acertou.
///
/// ⛔⛔⛔ **A LÂMPADA É FIXA, e a 1.ª redacção desta régua era um VÁCUO sem isso.** O
/// [`crate::gpu_frame::tests_lampada`] faz a luz **seguir a câmera** — logo orbitar trocava a LUZ,
/// a chave faltava de qualquer maneira, e as duas colunas comparavam **duas assaduras frescas**.
/// Ela lia `0 de 2 073 600` píxeis diferentes e não afirmava nada sobre a cache. *O furo estava
/// escrito no comentário da minha própria sonda, e eu li-o depois de a correr.*
///
/// ⭐ **E a metade do ACERTO é o que a torna load-bearing:** sem ela, uma cache apagada passa este
/// gate — *duas assaduras frescas dão a mesma imagem por construção.*
#[test]
#[ignore = "precisa de GPU"]
fn o_campo_do_chao_reaproveitado_nao_muda_um_byte() {
    use std::sync::atomic::Ordering;
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let materiais = [ph2d_material::OpenPbr::default().prepare()];
    let olhar = ph2d_view_transform::Look::default();
    const BG: [u8; 4] = [0, 0, 0, 0];
    let doc = crate::smoke::scene(1);
    let reg = crate::smoke::sampled_registry();
    let chao = Some(ph2d_field_render::Ground { height: -1.0 });
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: None,
    };
    // ⚠️ **FIXA em MUNDO**, e não a `tests_lampada`: é isso que faz orbitar não mexer na chave.
    const LUZ: [ph2d_field_render::PointLamp; 1] = [ph2d_field_render::PointLamp {
        world: [1.6, 2.4, 1.2],
        radiance_at_one: [7.0, 7.0, 7.0],
    }];
    let pinta = |azimute: f32, chao_em_cache: bool| {
        let cam = ph2d_field_render::Orbit {
            rotation: ph2d_field_render::Orbit::from_yaw_pitch(azimute, 0.5).rotation,
            ..ph2d_field_render::Orbit::default()
        };
        crate::gpu_frame::paint_com(
            t,
            &doc,
            &reg,
            &cam,
            &LUZ,
            &surfaces,
            &ph2d_field_render::Presentation::of(olhar),
            BG,
            chao,
            LW,
            LH,
            true,
            crate::gpu_frame::Sonda {
                chao_em_cache,
                ..crate::gpu_frame::Sonda::default()
            },
        )
        .expect("o pintor")
    };

    // Enche a cache num azimute, e pinta noutro: o campo tem de ser o MESMO objecto.
    let _ = pinta(0.0, true);
    let antes = crate::gpu_frame::ACERTOS_DO_CHAO.load(Ordering::Relaxed);
    let com = pinta(1.0, true);
    let acertos = crate::gpu_frame::ACERTOS_DO_CHAO.load(Ordering::Relaxed) - antes;
    let sem = pinta(1.0, false);

    assert!(
        acertos > 0,
        "orbitar não acertou na cache do chão — sem isto as duas colunas abaixo são duas          assaduras frescas e a igualdade delas não afirma nada"
    );
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
        "CONTROLO: {do_fundo} de {total} píxeis são o fundo — uma imagem quase vazia é          trivialmente igual a outra"
    );
    assert_eq!(
        com.rgba, sem.rgba,
        "o campo do chão reaproveitado de outro azimute mudou a imagem — a chave está a excluir          um eixo que CHEGA ao campo"
    );
}

/// ⭐⭐⭐⭐ **E A ECONOMIA MEDE-SE PELA CONTA: orbitar reaproveita, trocar a LUZ reassa.**
///
/// A cache dá o MESMO campo nas duas rotas — que é o que a torna segura e o que a torna impossível
/// de medir por valor: *uma cache apagada também dá o mesmo campo*. ⇒ a régua é o
/// [`crate::gpu_frame::ACERTOS_DO_CHAO`].
///
/// ⛔ **O CONTROLO é a segunda metade**, e sem ele a cura lê-se como *«reaproveita sempre»*: trocar
/// a luz tem de FALTAR, senão orbitar com a luz noutro sítio entregaria a sombra de cor antiga.
///
/// ⚠️ **Ele lê um contador GLOBAL** — sob `cargo test` (threads num processo) um irmão a pintar no
/// meio disto entra na conta. Sob o `nextest`, que dá um processo por teste, é sólido; é a mesma
/// nota do `a_fita_inerte_faz_o_cache_acertar_na_peca_seguinte`.
#[test]
#[ignore = "precisa de GPU"]
fn a_cache_do_chao_falta_quando_a_luz_muda() {
    use std::sync::atomic::Ordering;
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let materiais = [ph2d_material::OpenPbr::default().prepare()];
    let olhar = ph2d_view_transform::Look::default();
    const BG: [u8; 4] = [0, 0, 0, 0];
    let doc = crate::smoke::scene(2);
    let reg = crate::smoke::sampled_registry();
    let chao = Some(ph2d_field_render::Ground { height: -1.0 });
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: None,
    };
    let pinta_com = |azimute: f32,
                     luz: &[ph2d_field_render::PointLamp],
                     half_extent: f32,
                     donos: Option<&ph2d_field_eval::owners::Owners>| {
        let cam = ph2d_field_render::Orbit {
            rotation: ph2d_field_render::Orbit::from_yaw_pitch(azimute, 0.5).rotation,
            half_extent,
            ..ph2d_field_render::Orbit::default()
        };
        let surfaces = ph2d_field_render::Surfaces {
            all: surfaces.all,
            owners: donos,
        };
        let _ = crate::gpu_frame::paint_com(
            t,
            &doc,
            &reg,
            &cam,
            luz,
            &surfaces,
            &ph2d_field_render::Presentation::of(olhar),
            BG,
            chao,
            LW,
            LH,
            true,
            crate::gpu_frame::Sonda::default(),
        )
        .expect("o pintor");
    };
    let luz_a = [ph2d_field_render::PointLamp {
        world: [1.6, 2.4, 1.2],
        radiance_at_one: [7.0, 7.0, 7.0],
    }];
    let luz_b = [ph2d_field_render::PointLamp {
        world: [-1.6, 2.4, -1.2],
        radiance_at_one: [7.0, 7.0, 7.0],
    }];
    let padrao = ph2d_field_render::Orbit::default().half_extent;
    let pinta = |azimute: f32, luz: &[ph2d_field_render::PointLamp]| {
        pinta_com(azimute, luz, padrao, None);
    };
    let conta = |f: &dyn Fn()| {
        let antes = crate::gpu_frame::ACERTOS_DO_CHAO.load(Ordering::Relaxed);
        f();
        crate::gpu_frame::ACERTOS_DO_CHAO.load(Ordering::Relaxed) - antes
    };

    pinta(0.0, &luz_a);
    let ao_orbitar = conta(&|| pinta(1.0, &luz_a));
    let ao_trocar_a_luz = conta(&|| pinta(1.0, &luz_b));

    assert!(
        ao_orbitar > 0,
        "orbitar reassou o campo do chão — é o gesto que paga os +4,98 ms e a cache existe para ele"
    );
    assert_eq!(
        ao_trocar_a_luz, 0,
        "CONTROLO: trocar a LUZ acertou na cache — ela entregaria a sombra de cor da luz antiga, e sem esta metade a cura lê-se como «reaproveita sempre»"
    );

    // ⭐⭐⭐ **O ZOOM ABAIXO DO CLAMP tem de FALTAR.** A tolerância de acerto é
    // `min(HIT_EPS, half_extent/(2·lado_px))` e só desce com `lado_px > 2500 × half_extent` — a
    // `1080` píxeis isso é `half_extent < 0,432`. *Acima do clamp o zoom não é chave e abaixo é*, e
    // foi um CONTROLO que derrubou a redacção que dava a câmera inteira por fora (ver os gates de
    // `chao_ricochete.rs`).
    assert!(
        ph2d_field_render::Sharpness::for_frame(0.2, LH as usize).hit
            < ph2d_field_render::Sharpness::for_frame(padrao, LH as usize).hit,
        "CONTROLO DO CONTROLO: a `0,2` de enquadramento o clamp não solta a {LH} píxeis — então a metade abaixo mede o nada"
    );
    pinta(0.0, &luz_a);
    let ao_aproximar_muito = conta(&|| pinta_com(0.0, &luz_a, 0.2, None));
    assert_eq!(
        ao_aproximar_muito, 0,
        "CONTROLO: um zoom ABAIXO do clamp da tolerância acertou na cache — a tolerância move o campo até 0,91 de um byte ali, e a chave tem de a levar"
    );

    // ── E a CERCA: uma peça com LEI DO DONO não é cacheada ────────────────────────────────────
    //
    // ⚠️ A chave só conhece a fita COMBINADA, e a lei do dono é função das FOLHAS. A cerca é
    // conservadora de propósito, e sem esta metade ninguém a acorda quando alguém a apagar.
    let postas: Vec<ph2d_field::FieldDoc> = doc
        .nodes()
        .iter()
        .take(1)
        .map(|n| {
            ph2d_field::FieldDoc::new(vec![n.clone()], ph2d_field::NodeId(0)).expect("a folha")
        })
        .collect();
    let donos = ph2d_field_eval::owners::Owners::new(
        &postas,
        &reg,
        ph2d_field_render::hit_tolerance(padrao, f32::from(u16::try_from(LH).unwrap_or(u16::MAX))),
    );
    pinta_com(0.0, &luz_a, padrao, Some(&donos));
    let com_dono = conta(&|| pinta_com(1.0, &luz_a, padrao, Some(&donos)));
    assert_eq!(
        com_dono, 0,
        "CONTROLO: uma peça com LEI DO DONO acertou na cache — a chave não a distingue, e a cerca do `campo_do_chao` existe para isso"
    );
}
