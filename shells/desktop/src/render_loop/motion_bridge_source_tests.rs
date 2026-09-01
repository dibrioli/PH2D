//! **DE ONDE VEM O VALOR DE UMA LINHA** — o que um picker OFERECE, e quem DIRIGE uma linha.
//!
//! ⚠️ **Este arquivo existe por um TETO DE LOC** (HR-18, 600 para `shells/`), e o corte é por
//! PERGUNTA: o irmão [`super::param_tests`] pergunta *que FORMA tem esta linha?* (um slider, um
//! enum, uma caixa, um campo de texto); aqui pergunta-se *de onde sai o valor dela?* — o
//! namespace reservado que o picker esconde, o card que a dirige, e o canal que não é uma forma
//! desenhada. Elas partilham o `build_params_snapshot` e mais nada.

use super::params::build_params_snapshot;
use crate::motion_state::MotionState;
use ph2d_editor::ProjectSettings;

/// **The picker offers what the ARTIST named, never the editor's own values.**
///
/// The reserved namespace carries the cursor and a `$at:<name>` position beside every
/// published object. Both live in the table the picker reads, so without a filter they
/// appear as objects you can aim at — the namespace leaking into the UI it exists to
/// keep clean, and a list where half the entries are implementation.
#[test]
fn the_source_picker_hides_the_editors_reserved_namespace() {
    use ph2d_nodegraph::attr::{Column, Stream};
    let mut motion = MotionState::new();
    let at = |x: f32| Stream::new(1).with("P", Column::Vec2(vec![[x, 0.0]]));
    motion.pump.cook.set_external("Sun".to_string(), at(1.0));
    motion
        .pump
        .cook
        .set_external(ph2d_nodegraph::external::position_of("Sun"), at(1.0));
    motion
        .pump
        .cook
        .set_external(ph2d_nodegraph::external::CURSOR.to_string(), at(2.0));

    let opts = super::params::source_options_for_tests(&motion);
    assert_eq!(
        opts,
        vec!["Sun".to_string()],
        "only the artist's name is pickable: {opts:?}"
    );
}

/// **A row DIRIGIDA diz QUEM a dirige, e o nome é o que está escrito no card** (doc 88 B3).
///
/// Nasceu VERMELHO: a row carregava um `bool`. O artista via um número acentuado que não
/// obedecia ao dedo e não tinha uma palavra sobre a procedência — a resposta exigia sair do
/// inspector e caçar o fio no grafo, num nó que ele ainda não sabia qual era.
///
/// As duas metades, e a segunda é a que importa: o nome sai da porta única
/// `ph2d_node_registry::card_title`, então **um rename move os dois** — o card e a row. Uma
/// escada de fallbacks copiada aqui ficaria verde neste gate e mentiria no dia do rename, que
/// é justamente o dia em que o artista precisa do nome para achar o nó.
#[test]
fn a_driven_row_names_the_card_that_drives_it() {
    use ph2d_nodegraph::attr::{Column, Stream};
    use std::collections::BTreeMap;

    let mut motion = MotionState::new();
    let driver = motion.doc.graph.add_node("value.gain");
    let target = motion.doc.graph.add_node("value.gain");
    motion
        .doc
        .graph
        .drive_param(target, "strength", (driver, 0))
        .expect("o fio entra no param");
    // O tap é o caminho de um frame de GPU (o default do app); o memo estaria vazio.
    motion.gpu_tap = Some(BTreeMap::from([(
        driver,
        Stream::new(1).with("v", Column::Scalar(vec![42.0])),
    )]));
    ph2d_panel_motion_graph::set_graph_selection(vec![target.0]);

    let driven_by = |motion: &MotionState| -> Option<String> {
        build_params_snapshot(motion, ProjectSettings::default())
            .expect("o alvo resolve")
            .rows
            .iter()
            .find_map(|r| match r {
                ph2d_panel_motion_params::ParamRow::Scalar(s) if s.name == "strength" => {
                    Some(s.driven_by.clone())
                }
                _ => None,
            })
            .expect("a row do param dirigido existe")
    };

    // Sem rename: o card diz o nome do TIPO, e a row diz o mesmo.
    assert_eq!(
        driven_by(&motion).as_deref(),
        Some("Gain"),
        "a row dirigida nomeia o card que a dirige"
    );

    // Com rename: o nome do ARTISTA vence nos dois lugares.
    motion.doc.graph.set_label(driver, "Volume");
    assert_eq!(
        driven_by(&motion).as_deref(),
        Some("Volume"),
        "e o nome segue o rename — é a MESMA porta que escreve o título do card"
    );

    // O CONTROLE: sem fio não há nome. `driven_by` é o fato inteiro, então isto é o que
    // impede a row de nascer com dono e sem procedência.
    ph2d_panel_motion_graph::set_graph_selection(vec![driver.0]);
    assert_eq!(
        driven_by(&motion),
        None,
        "um param que ninguém dirige não tem quem o nomeie"
    );
    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
}

/// **O picker de canais CABE no teto do painel — e o teto não é o que o doc dizia.**
///
/// O doc-comment do `READ_CHANNELS` afirmava *"seven of them + Custom = 8 = the segmented
/// selector's ceiling"*. O teto real é `MAX_ENUM_OPTIONS = 48` (DERIVADO, com o
/// `CHANNELS_EXTRA_BASE` começando exatamente onde ele acaba) e a fileira **quebra em quatro
/// colunas**, crescendo a própria altura — então o 8 era um palpite sobre LARGURA vestindo a
/// palavra *teto*, e o canal `Falloff` (2026-08-09) o teria "estourado" sem nada acontecer.
///
/// ⚠️ **Este gate mora AQUI porque é o único lugar onde as duas metades se encontram:** a
/// tabela vive na crate do nó (que não conhece painel) e o teto vive na crate do painel (que
/// não conhece registry). Ele afirma a PROPRIEDADE — *toda opção pintada tem id próprio* —
/// e não a contagem de hoje, então um canal novo passa e o quadragésimo nono reprova, que é
/// exatamente onde a decisão volta a ser necessária.
#[test]
fn the_channel_picker_fits_the_panels_ceiling() {
    use ph2d_panel_motion_params::{MAX_ENUM_OPTIONS, ParamRow};
    let mut motion = MotionState::new();
    let attr = motion.doc.graph.add_node("value.attribute");
    ph2d_panel_motion_graph::set_graph_selection(vec![attr.0]);

    let snap =
        build_params_snapshot(&motion, ProjectSettings::default()).expect("attribute resolvable");
    let ch = snap
        .rows
        .iter()
        .find_map(|r| match r {
            ParamRow::Channels(c) => Some(c),
            _ => None,
        })
        .expect("o picker de canais é uma row de Channels");

    // O peso de um campo é OFERECIDO, não só legível se digitado (doc 89, folha 12).
    assert!(
        ch.channels
            .iter()
            .any(|(l, c, _)| *l == "Falloff" && *c == "falloff"),
        "o picker oferece o peso que as `field.*` escrevem: {:?}",
        ch.channels.iter().map(|(l, ..)| *l).collect::<Vec<_>>()
    );
    // A propriedade: cada canal + o "Custom…" final ganha um botão com id próprio. Acima do
    // teto o `.min(MAX_ENUM_OPTIONS)` do painter simplesmente PARA de desenhar — a opção
    // excedente nasceria invisível e inalcançável, em silêncio.
    let painted = ch.channels.len() + 1; // os canais curados + o "Custom…" final
    assert!(
        painted <= MAX_ENUM_OPTIONS,
        "{} canais + Custom = {painted} passam do teto de {MAX_ENUM_OPTIONS} — o excedente não é pintado",
        ch.channels.len()
    );
    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
}

/// ⛔⛔⛔ **NENHUMA CHAVE DERIVADA APARECE NO SELECTOR DE OBJECTOS** — a cerca que já existia e
/// que quem cunha chaves de CONTEÚDO não estava a cumprir.
///
/// # O defeito
///
/// Auditoria de seis lentes, doc 96 §5.5. As membranas publicam a geometria derivada na **mesma
/// tabela de externos** de que o picker de objectos tira as opções — e o filtro dele é
/// `!is_reserved(k)`. Sem o prefixo `$`, cada planta, forma, texto e tabela derivada aparecia ao
/// artista como uma *"Drawn shape"* escolhível: na cena `=108` são **cinco chips de lixo**, com
/// a gramática crua lá dentro, e clicar num planta a **PRÓPRIA planta** como folha dela.
///
/// ⚠️ O doc do `RESERVED_PREFIX` já dizia as duas metades da regra — *«o editor publica DENTRO
/// do namespace, e recusa publicar um nome do artista que já esteja nele»*. A **primeira** é que
/// não estava a ser cumprida.
///
/// # A régua: o que o PICKER mostra, não o que a tabela contém
///
/// ⚠️ **É uma família, não um caso** (`$lsysrib` · `$shape` · `$text` · `$table:`), então o gate
/// pergunta pela porta do produto (`publish_all` + `source_options`) em vez de nomear um
/// prefixo. Um cunhador NOVO que esqueça o `$` fica vermelho aqui sem ninguém o acrescentar a
/// uma lista.
#[test]
fn no_derived_key_is_offered_as_a_drawn_shape() {
    use crate::render_loop::motion_lsystem_testkit::{key_of, plant, publish_object};
    let (mut state, n) = plant(ph2d_node_source_lsystem::GEOMETRY_BRANCHES);
    // Um objecto de VERDADE, nomeado pelo artista — o controlo do próprio filtro: sem ele um
    // `source_options` vazio passaria calado.
    publish_object(&mut state, "folha", 7);
    // ⚠️ **A FAMÍLIA INTEIRA na cena, e não só a planta.** A 1.ª redacção publicava só um
    // `source.lsystem`, e as mutações que tiram o `$` das chaves da FORMA e do TEXTO
    // **sobreviveram** — a população não continha o fenómeno delas.
    for ty in ["source.shape", "source.text"] {
        state.doc.graph.add_node(ty);
    }

    // ⛔⛔ **O QUE AS MEMBRANAS PUBLICARAM, medido pela DIFERENÇA — e não pelo prefixo.**
    //
    // A 2.ª redacção iterava as chaves `is_reserved(...)` e exigia que nenhuma estivesse no
    // selector. ⚠️ **Isso é vacuoso exactamente no caso que interessa:** uma chave que perca o
    // `$` sai da população do laço e entra nas opções sem ninguém olhar — *o gate tinha a mesma
    // forma do defeito que caça*. As três mutações sobreviveram assim.
    //
    // ⇒ a régua é o CONJUNTO que a publicação acrescentou, capturado antes e depois. Ela não
    // depende da propriedade sob teste.
    let antes: std::collections::BTreeSet<String> =
        state.pump.cook.externals().keys().cloned().collect();
    crate::render_loop::motion_externals::publish_all(&mut state, 0.0);
    let derivadas: Vec<String> = state
        .pump
        .cook
        .externals()
        .keys()
        .filter(|k| !antes.contains(*k))
        .cloned()
        .collect();

    let key = key_of(&mut state, n);
    let opcoes = super::params::source_options_for_tests(&state);
    assert!(
        opcoes.iter().any(|o| o == "folha"),
        "o objecto que o ARTISTA nomeou tem de estar no selector: {opcoes:?}"
    );
    assert!(
        !opcoes.contains(&key),
        "a chave de conteúdo da planta está no selector — clicar nela planta a própria planta \
         como folha dela. Opções: {opcoes:?}"
    );
    // ⚠️ O controlo do próprio censo: três membranas na cena têm de ter publicado três chaves.
    assert!(
        derivadas.len() >= 3,
        "só {} chave(s) derivada(s) — a fixtura perdeu uma membrana: {derivadas:?}",
        derivadas.len()
    );
    for d in &derivadas {
        assert!(
            !opcoes.contains(d),
            "`{d}` foi publicada por uma MEMBRANA e está no selector de objectos do artista. \
              Derivadas: {derivadas:?}"
        );
    }
}
