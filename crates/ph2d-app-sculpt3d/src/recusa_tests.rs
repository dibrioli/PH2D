//! Os gates da porta das recusas — **as três razões, e o censo que obriga a
//! quarta a existir.**

use ph2d_mesh::Mesh;
use ph2d_sculpt3d::{Brush, Verb};

use super::Entradas;

/// Uma tigela: meia bola, com boca.
///
/// ⚠️ **Pela porta que COMPACTA** (`scenes::boundary::tigela`) — a versão
/// montada à mão deixa **órfãos**, e é um defeito que esta crate já pagou duas
/// vezes (a cena `=42` e o arnês do censo dos knobs).
fn tigela() -> Mesh {
    crate::scenes::boundary::tigela()
}

fn bola() -> Mesh {
    ph2d_mesh::shapes::uv_sphere(16, 24, 1.0)
}

fn entradas<'a>(mesh: &'a Mesh, brush: &'a Brush, outras_pecas: usize) -> Entradas<'a> {
    Entradas {
        brush,
        tem_referencia: false,
        mesh,
        outras_pecas,
        outras_escondidas: 0,
    }
}

/// ⛔⛔⛔ **O CENSO — toda PORTA de «entrada ausente» do motor tem VOZ.**
///
/// A família é a dos predicados `precisa_d*` do motor: cada um declara uma
/// **entrada que pode não existir**, e um gesto a que falte essa entrada **não
/// move um único vértice**. *Um pincel que não faz nada e não diz porquê é
/// indistinguível de um pincel partido.*
///
/// ⚠️ **Derivado do ficheiro que os DECLARA, e não de uma lista aqui:** um
/// predicado novo daquela família reprova este gate até alguém lhe dar voz — que
/// é a diferença entre uma lista que alguém tem de se lembrar de estender e uma
/// que não fica verde sem a extensão.
///
/// ⚠️ **As linhas de comentário são peneiradas**: os ficheiros citam os nomes na
/// prosa (este doc inclusive). *Um censo textual que não separa prosa de código
/// mente nos dois sentidos.*
///
/// ⛔ **A recusa do passe de topologia NÃO entra**, e a ausência é a decisão: ela
/// não é um facto do pen-down — o passe só sabe que não mudou nada **depois** de
/// correr. Ela tem voz própria (`dyntopo::queixa_do_passe`).
#[test]
fn toda_porta_de_entrada_ausente_tem_recusa() {
    const PREDICADOS: &str = include_str!("../../ph2d-sculpt3d/src/brush_verb_predicados.rs");
    const ESCALA: &str = include_str!("../../ph2d-sculpt3d/src/brush_scale.rs");
    const RECUSA: &str = include_str!("recusa.rs");

    let familia: Vec<&str> = [PREDICADOS, ESCALA]
        .iter()
        .flat_map(|f| f.lines())
        .map(str::trim)
        .filter(|l| !l.starts_with("//"))
        .filter_map(|l| l.split_once("pub fn precisa_d"))
        .filter_map(|(_, r)| r.split_once('(').map(|(n, _)| n))
        .collect();
    assert!(
        familia.len() >= 3,
        "a extracção achou {} portas da família `precisa_d*` — o motor declara \
         mais, logo ela deixou de ler os ficheiros como eles estão escritos e \
         este gate passaria a medir o vácuo: {familia:?}",
        familia.len()
    );
    let mudas: Vec<&&str> = familia
        .iter()
        .filter(|n| !RECUSA.contains(&format!("precisa_d{n}(")))
        .collect();
    assert!(
        mudas.is_empty(),
        "estas portas declaram uma ENTRADA que pode faltar e o pen-down não diz \
         nada quando ela falta: {mudas:?} — o artista lê isso como «a ferramenta \
         não funciona», que é a conclusão cara"
    );
}

/// ⛔⛔⛔ **E O PEN-DOWN CHAMA-A** — a metade que nenhum teste de unidade pode
/// exercitar, porque o `input_down` pede um `AppHost`.
///
/// ⚠️ *Uma porta com a lei certa e zero chamadores produz o MESMO app que uma
/// lei ausente* — e esta crate já pagou essa forma: um gate que chama a função
/// em vez de percorrer a rota afirma que a peça certa existe, nunca que o
/// produto a usa.
///
/// ⚠️ **`include_str!` e não `read_to_string`:** se o pen-down mudar de ficheiro
/// isto **deixa de compilar**, em vez de ficar verde a medir nada.
#[test]
fn o_pen_down_diz_a_recusa() {
    const PEN_DOWN: &str = include_str!("input_down.rs");
    let corpo: String = PEN_DOWN
        .lines()
        .map(str::trim)
        .filter(|l| !l.starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        corpo.contains("diz_a_recusa_do_pen_down()"),
        "o pen-down nao chama a porta das recusas — a lei existe e o artista \
         continua sem saber porque o gesto nao fez nada"
    );
}

/// ⛔⛔ **O CONTORNO numa peça FECHADA diz porquê** — e a metade que importa é a
/// **negativa**: numa tigela ele **cala-se**.
///
/// ⚠️ *Sem a metade negativa, uma recusa incondicional passaria* — e um pincel
/// que se queixa sempre é ruído que o artista aprende a ignorar, exactamente
/// quando ela passar a ser verdade.
#[test]
fn o_contorno_numa_peca_fechada_diz_porque() {
    let b = Brush {
        verb: Verb::Boundary,
        ..Brush::default()
    };
    let fechada = bola();
    let motivo = entradas(&fechada, &b, 0).recusa().unwrap_or_default();
    assert!(
        // ⚠️ **A palavra mudou de LÍNGUA em 2026-09-19, por ordem do dono (*«tudo em inglês»*).** A
        //    lei é a mesma — *a recusa NOMEIA a entrada que falta* —, e o que se ajustou foi a palavra
        //    que a prova: estas frases são as que o ARTISTA lê, e o app é inglês.
        motivo.contains("RIM"),
        "com o contorno numa bola fechada a recusa foi `{motivo}` — ela tem de \
         nomear a BEIRA, que é a entrada que falta"
    );
    let aberta = tigela();
    assert_eq!(
        entradas(&aberta, &b, 0).recusa(),
        None,
        "numa tigela o contorno tem bordo e nada a dizer — uma recusa \
         incondicional é ruído que o artista aprende a ignorar"
    );
}

/// ⛔⛔ **O PROJECTAR sozinho na cena diz porquê**, e o controlo é a cena com
/// duas peças.
#[test]
fn o_projectar_sozinho_na_cena_diz_porque() {
    let b = Brush {
        verb: Verb::SceneProject,
        ..Brush::default()
    };
    let m = bola();
    let motivo = entradas(&m, &b, 0).recusa().unwrap_or_default();
    assert!(
        // ⚠️ **A palavra mudou de LÍNGUA em 2026-09-19, por ordem do dono (*«tudo em inglês»*).** A
        //    lei é a mesma — *a recusa NOMEIA a entrada que falta* —, e o que se ajustou foi a palavra
        //    que a prova: estas frases são as que o ARTISTA lê, e o app é inglês.
        motivo.contains("ANOTHER piece"),
        "com o projectar sozinho na cena a recusa foi `{motivo}` — ela tem de \
         nomear a outra peça, que é a entrada que falta"
    );
    assert_eq!(
        entradas(&m, &b, 1).recusa(),
        None,
        "com uma segunda peça na cena ele tem o que precisa e cala-se"
    );
}

/// ⛔⛔ **A PILHA DE MULTIRESOLUÇÃO** — a razão que já existia, agora pela mesma
/// porta que as outras duas.
#[test]
fn os_verbos_de_deslocamento_sem_pilha_dizem_porque() {
    let m = bola();
    for verb in [Verb::EraseMultires, Verb::SmearMultires] {
        let b = Brush {
            verb,
            ..Brush::default()
        };
        let motivo = entradas(&m, &b, 0).recusa().unwrap_or_default();
        assert!(
            // ⚠️ **A palavra mudou de LÍNGUA em 2026-09-19, por ordem do dono (*«tudo em inglês»*).** A
            //    lei é a mesma — *a recusa NOMEIA a entrada que falta* —, e o que se ajustou foi a palavra
            //    que a prova: estas frases são as que o ARTISTA lê, e o app é inglês.
            motivo.contains("multiresolution"),
            "com o {} sem pilha a recusa foi `{motivo}`",
            verb.label()
        );
        let com = Entradas {
            tem_referencia: true,
            ..entradas(&m, &b, 0)
        };
        assert_eq!(
            com.recusa(),
            None,
            "com pilha o {} tem o que precisa e cala-se",
            verb.label()
        );
    }
}

/// ⭐ **E um verbo COMUM nunca é recusado** — o controlo que impede uma recusa
/// incondicional de passar os três gates acima.
#[test]
fn um_verbo_comum_nao_e_recusado() {
    let m = bola();
    for verb in [Verb::Draw, Verb::Smooth, Verb::Clay, Verb::Pose] {
        let b = Brush {
            verb,
            ..Brush::default()
        };
        assert_eq!(
            entradas(&m, &b, 0).recusa(),
            None,
            "o {} não pede entrada nenhuma e foi recusado",
            verb.label()
        );
    }
}

/// ⭐⭐⭐ **E A RECUSA DIZ A CURA: *«está escondida»* não é *«não há»*.**
///
/// ⚠️ **As duas levam o artista a gestos OPOSTOS** — uma manda criar geometria,
/// a outra manda abrir um olho na Hierarquia —, e num contador só elas leem-se
/// exactamente igual. *Uma recusa que nomeia o facto certo e a cura errada é
/// mais cara que nenhuma: o artista faz o trabalho e o pincel continua inerte.*
#[test]
fn a_recusa_separa_a_peca_que_falta_da_peca_escondida() {
    let b = Brush {
        verb: Verb::SceneProject,
        ..Brush::default()
    };
    let m = bola();
    // (1) Cena de uma peça só: não HÁ outra.
    let motivo = entradas(&m, &b, 0).recusa().unwrap_or_default();
    assert!(
        // ⚠️ **A palavra mudou de LÍNGUA em 2026-09-19, por ordem do dono (*«tudo em inglês»*).** A
        //    lei é a mesma — *a recusa NOMEIA a entrada que falta* —, e o que se ajustou foi a palavra
        //    que a prova: estas frases são as que o ARTISTA lê, e o app é inglês.
        motivo.contains("ANOTHER piece in the scene"),
        "sem outra peça a recusa tem de falar de a criar, e foi `{motivo}`"
    );
    // (2) A outra existe e está escondida: a cura é o olho.
    let escondida = Entradas {
        outras_escondidas: 1,
        ..entradas(&m, &b, 0)
    };
    let motivo = escondida.recusa().unwrap_or_default();
    assert!(
        // ⚠️ **A palavra mudou de LÍNGUA em 2026-09-19, por ordem do dono (*«tudo em inglês»*).** A
        //    lei é a mesma — *a recusa NOMEIA a entrada que falta* —, e o que se ajustou foi a palavra
        //    que a prova: estas frases são as que o ARTISTA lê, e o app é inglês.
        motivo.contains("hidden") && motivo.contains("Hierarchy"),
        "com o único alvo escondido a recusa tem de nomear o olho, e foi \
         `{motivo}`"
    );
    // (3) ⭐ E o CONTROLO: com uma à vista ele cala-se, escondidas ou não.
    let com = Entradas {
        outras_escondidas: 3,
        ..entradas(&m, &b, 1)
    };
    assert_eq!(
        com.recusa(),
        None,
        "com uma peça À VISTA ele tem o que precisa — as escondidas não o \
         impedem de nada"
    );
}
