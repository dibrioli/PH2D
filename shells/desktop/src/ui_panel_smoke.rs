//! **A cena do PAINEL GERADO** — `PH2D_BUILD_SMOKE=62` (plano UI/UX W8b).
//!
//! # A pergunta desta cena
//!
//! *Eu desenhei um painel, e o app me devolveu o CÓDIGO dele.*
//!
//! Uma moldura chamada **Color** com filhos VESTIDOS — duas seções, um slider, um toggle, um
//! botão, uma swatch, dois botões de ícone, uma faixa de abas e um dropdown — e **um que é só
//! desenho**. No fim ela imprime, no stderr, o código-fonte que o gerador escreveu, que é o
//! artefato desta wave.
//!
//! ⚠️ **O filho de desenho puro é o CONTROLE da cena**, e ele não é decoração: ele prova que *só
//! quem VESTE vira row*. Sem ele, um gerador que transformasse todo filho em linha passaria
//! despercebido, e o painel gerado teria uma row que não faz nada — o item-de-menu-morto na sua
//! forma mais cara.
//!
//! ⚠️ **E a cena imprime o número que a torna válida:** quantas rows o plano tem, comparado com o
//! que a tabela [`AUTHORED`] descreve. Se o log disser **PARE**, pare — o resto não diz nada.
//! (O número não é escrito aqui de propósito: ele já esteve errado neste cabeçalho, que dizia
//! *"se não forem quatro"* enquanto a cena tinha nove. Quem o conta é o [`expected_rows`].)
//!
//! # A W8b.3 põe a ARTE do outro lado do fio
//!
//! Um retângulo laranja ao lado do painel, **fora** da moldura, com a row *Opacity* já presa a
//! ele. Arrastar o slider desvanece a arte; prender o *Visible* à mão é o gesto que a fatia
//! acrescenta. ⚠️ E o `Reset` é o CONTROLE: um `Button` não dirige, e a linha *Drives* nem
//! aparece nele.
//!
//! # A W8b.2 abre o painel, e a cena diz a verdade sobre ele
//!
//! ⚠️ **O painel na tela mostra a tabela COMMITADA, não a que o log acabou de imprimir.** É o que
//! codegen é: o app escreve o código, alguém o commita, o `cargo` o compila. Esconder isso faria o
//! artista renomear um filho, ver o log mudar, olhar o painel intacto e concluir que ele está
//! quebrado — então o roteiro diz na cara, e o gate de staleness é quem garante que os dois estão
//! sincronizados no `main`.

use ph2d_editor::widget::WidgetKind;
use ph2d_vec_scene::{Paint, Rgba8, VecPath, VecPathId, rectangle, star};

/// A moldura, e os filhos que ela contém: `(caixa, nome, o que ele veste)`.
///
/// ⚠️ **UMA TABELA, DOIS CONSUMIDORES:** a cena constrói a árvore a partir dela, e o gate de
/// staleness (`the_generated_panel_is_not_stale`) constrói o mundo a partir dela para emitir o
/// código e comparar com o arquivo commitado. Uma segunda lista escrita à mão no gate divergiria
/// desta no dia em que uma row entrasse — e o gate ficaria verde sobre o painel errado.
pub(crate) const AUTHORED: [([f64; 4], &str, Option<WidgetKind>); 21] = [
    ([-2.0, -4.9, 2.0, 2.4], "Color", None),
    (
        [-1.8, 1.4, 1.8, 2.2],
        "Appearance",
        Some(WidgetKind::SectionHeader),
    ),
    ([-1.8, 0.4, 1.8, 1.1], "Opacity", Some(WidgetKind::Slider)),
    ([-1.8, -0.6, 1.8, 0.1], "Visible", Some(WidgetKind::Toggle)),
    ([-1.8, -1.6, 1.8, -0.9], "Reset", Some(WidgetKind::Button)),
    (
        [-1.8, -2.6, 1.8, -1.9],
        "Tint",
        Some(WidgetKind::ColorSwatch),
    ),
    (
        [-0.4, -3.7, 0.4, -2.9],
        "Play",
        Some(WidgetKind::IconButton),
    ),
    (
        [-0.4, -4.7, 0.4, -3.9],
        "Trash",
        Some(WidgetKind::IconButton),
    ),
    ([-1.7, 1.25, 1.7, 1.35], "Backdrop", None),
    // **A FAMÍLIA DE LISTA** — uma faixa de abas e os TRÊS filhos que são as opções dela.
    //
    // ⚠️ As três não são rows, e é a lei de posse (`takes_options`): elas vivem DENTRO do
    // controle, então quem as reclama é ele. A prova está na contagem, que o `expected_rows`
    // deriva desta mesma tabela.
    ([-1.8, -5.7, 1.8, -5.0], "View", Some(WidgetKind::Tabs)),
    ([-1.7, -5.6, -0.7, -5.1], "Design", None),
    ([-0.6, -5.6, 0.4, -5.1], "Preview", None),
    ([0.5, -5.6, 1.7, -5.1], "Code", None),
    // **O DROPDOWN** — a mesma família, e a única cujas opções ficam ESCONDIDAS até se pedir.
    //
    // ⚠️ Ele é o que separa *"o controle desenha as N opções"* de *"o controle desenha uma e
    // guarda o resto"*: as três acima estão na tela desde que a moldura abriu, e estas três só
    // existem depois de um clique. Sem ele, o passe diferido não teria nada a diferir.
    ([-1.8, -6.7, 1.8, -6.0], "Blend", Some(WidgetKind::Dropdown)),
    ([-1.7, -6.6, 1.7, -6.1], "Normal", None),
    ([-1.7, -7.6, 1.7, -7.1], "Multiply", None),
    ([-1.7, -8.6, 1.7, -8.1], "Screen", None),
    // **A SEGUNDA SEÇÃO** — e ela é o que torna a lei do colapso VERIFICÁVEL a olho.
    //
    // ⚠️ Com um cabeçalho só, *"esconde as rows até o PRÓXIMO"* e *"esconde tudo abaixo"* dão a
    // mesma foto: não há nada depois para a diferença aparecer. Com duas, dobrar a de cima tem
    // de deixar a de baixo inteira na tela — e é essa a pergunta de olho que o passo 20 faz.
    (
        [-1.8, -9.7, 1.8, -8.9],
        "Advanced",
        Some(WidgetKind::SectionHeader),
    ),
    ([-1.8, -10.7, 1.8, -10.0], "Seed", Some(WidgetKind::Slider)),
    (
        [-1.8, -11.7, 1.8, -11.0],
        "Dither",
        Some(WidgetKind::Checkbox),
    ),
    // **O READOUT** — e ele e' o CONTROLE de um gate, nao enfeite.
    //
    // ⚠️ Uma barra de progresso DESENHA e nunca responde, e o `a_display_only_row_is_not_clickable`
    // precisa de exactamente uma dessas na tabela para ter sujeito. Ele ficou sem uma no dia em
    // que a swatch passou a abrir o picker — e **recusou-se a medir o vazio** em vez de ficar
    // verde, que e' a razao de o controle positivo dele existir.
    (
        [-1.8, -12.7, 1.8, -12.0],
        "Loading",
        Some(WidgetKind::ProgressBar),
    ),
];

/// **Quem POSSUI quem** — os controles de lista da cena e os filhos que são as opções deles.
///
/// ⚠️ **Por nome e não por índice, e a razão é um defeito que este arquivo já produziu:** a versão
/// por índice contou a tabela à mão, errou por um, e pendurou as opções na entidade errada. O
/// sintoma foi a faixa sair **sem opção nenhuma** — e a contagem de rows deu **certa por acidente**
/// (a entidade adotada não vestia widget, então não virou row), o que fez o `PARE` passar sobre o
/// painel errado. Um índice contado à mão numa tabela que cresce é um erro à espera; o nome não.
///
/// ⚠️ **E ela é uma LISTA desde o segundo dono**, não um par de consts: a versão anterior tinha
/// `TAB_OPTIONS` e `TABS_OWNER` soltos, o que é a forma que só descreve **um** controle — o
/// dropdown teria de duplicar as duas, e o terceiro dono duplicá-las-ia outra vez.
const OPTION_OWNERS: [(&str, &[&str]); 2] = [
    ("View", &["Design", "Preview", "Code"]),
    ("Blend", &["Normal", "Multiply", "Screen"]),
];

/// **DE QUEM cada linha da tabela é filha** — a porta única do parentesco.
///
/// ⚠️ **Ela existe porque a tabela tem DOIS construtores de mundo** (a cena do smoke e o harness
/// do gate de staleness), e o doc da `AUTHORED` já avisava que uma segunda derivação diverge no
/// dia em que a árvore mudasse de forma. Esse dia foi hoje: a família de LISTA pendura as opções
/// no CONTROLE, e o harness — que parenteava tudo na moldura — emitiu um golden com a faixa sem
/// opção nenhuma. O gate ficaria verde sobre um painel que o artista não desenhou.
#[must_use]
pub(crate) fn authored_parent(i: usize) -> Option<usize> {
    if i == FRAME {
        return None;
    }
    if let Some((owner, _)) = OPTION_OWNERS
        .iter()
        .find(|(_, opts)| opts.contains(&AUTHORED[i].1))
    {
        return AUTHORED.iter().position(|(_, n, _)| n == owner);
    }
    Some(FRAME)
}

/// **Quantas rows esta tabela descreve** — vestida **e** filha directa da moldura.
///
/// ⚠️ **DERIVADA, e não um literal.** Um `8` escrito à mão só sabe dizer *"o número mudou"*, e a
/// pergunta que o `PARE` faz é outra: *a lei de posse continua de pé?* Derivá-la da MESMA
/// [`authored_parent`] que constrói a árvore é o que faz o aviso disparar quando uma opção escapa
/// para a moldura — e ficar quieto quando alguém simplesmente acrescenta um controle.
#[must_use]
pub(crate) fn expected_rows() -> usize {
    AUTHORED
        .iter()
        .enumerate()
        .filter(|(i, (_, _, k))| k.is_some() && authored_parent(*i) == Some(FRAME))
        .count()
}

/// **O ícone ESCOLHIDO de uma row**, quando o artista escolheu um em vez de desenhar.
///
/// ⚠️ **Porta única, e chaveada pelo NOME e não pela posição** — o irmão do [`authored_fill`]. A
/// tabela acima já é uma tupla de três, e o `clippy::type_complexity` acabou de cobrar a quinta
/// posição no código gerado: um quarto elemento aqui repetiria a lição na mesma sessão.
///
/// ⚠️ **A cena leva as DUAS rotas lado a lado, e é isso que a torna uma prova:** *Play* desenha a
/// ESTRELA que o artista fez, *Trash* mostra o glifo do catálogo. Com só uma delas, *"a escolha
/// vence"* e *"o botão sempre desenha a forma"* seriam a mesma foto.
#[must_use]
pub(crate) fn authored_icon(name: &str) -> Option<&'static str> {
    (name == "Trash").then_some("trash")
}

/// **A GEOMETRIA de cada forma da tabela** — e para UMA delas ela é o GLIFO.
///
/// ⚠️ **Porta única, pela mesma razão do [`authored_fill`]:** o gate de staleness constrói o mundo
/// desta tabela para emitir o código, e o SVG do ícone atravessa o `RowSpec` — uma segunda
/// construção do lado do gate faria o golden concordar com uma forma que ninguém desenha.
///
/// ⚠️ **A ESTRELA é o CONTROLE do ícone, e ela não é enfeite.** Um retângulo num botão de ícone
/// seria indistinguível de *nenhum ícone*: a moldura do chip já é um retângulo. Com uma figura
/// reconhecível, *"o glifo é o desenho"* e *"o pintor põe um glifo qualquer"* deixam de ser a
/// mesma foto — que é a pergunta de olho desta fatia.
#[must_use]
pub(crate) fn authored_path(r: &[f64; 4], kind: Option<WidgetKind>) -> VecPath {
    if matches!(kind, Some(WidgetKind::IconButton)) {
        let (cx, cy) = ((r[0] + r[2]) * 0.5, (r[1] + r[3]) * 0.5);
        return star([cx, cy], (r[2] - r[0]) * 0.5, (r[3] - r[1]) * 0.5, 5, 0.45);
    }
    rectangle([r[0], r[1]], [r[2], r[3]])
}

/// **A tinta de cada forma da tabela** — e para UMA delas ela é o CONTEÚDO, não a decoração.
///
/// ⚠️ Ela é `pub(crate)` porque o gate de staleness constrói a MESMA cena para emitir o código: a
/// cor da swatch atravessa o `RowSpec`, então uma segunda tabela de cores no lado do gate faria o
/// golden concordar com uma cena que ninguém desenha.
///
/// ⚠️ **A swatch tem cor PRÓPRIA de propósito.** Com o azul dos irmãos vestidos, *"a swatch mostra
/// o preenchimento DELA"* e *"a swatch pinta um azul fixo"* seriam indistinguíveis na foto — e a
/// pergunta de olho deste smoke é exactamente que os dois lados concordem.
#[must_use]
pub(crate) const fn authored_fill(i: usize, kind: Option<WidgetKind>) -> [u8; 3] {
    match (i == FRAME, kind) {
        (true, _) => [30, 34, 46],
        (_, Some(WidgetKind::ColorSwatch)) => [214, 92, 64],
        (_, Some(_)) => [64, 84, 128],
        (_, None) => [44, 48, 58],
    }
}

/// A moldura é a primeira linha; os filhos são o resto.
const FRAME: usize = 0;

/// **A ARTE** — a forma que as rows vão DIRIGIR (W8b.3), e ela não é filha da moldura.
///
/// ⚠️ Fora da tabela `AUTHORED` de propósito: aquela lista descreve *o painel*, e o gate de
/// staleness constrói o mundo a partir dela para emitir o código. Uma estrela ali dentro entraria
/// na conta do painel; aqui ela é o que o painel MEXE — os dois lados do fio, e cada um no seu
/// lugar.
const STAR: usize = AUTHORED.len();

pub(crate) fn frame(app: &mut crate::App, f: u32) {
    match f {
        3 => build(app),
        5 => name_and_parent(app),
        7 => bind_the_slider(app),
        9 => announce(app),
        11 => open_the_panel(app),
        _ => {}
    }
}

fn build(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    for (i, (r, _, kind)) in AUTHORED.iter().enumerate() {
        let mut p: VecPath = authored_path(r, *kind);
        // A moldura é escura; quem veste fica visível; o desenho puro fica apagado, para a foto
        // dizer qual é qual — e a swatch leva a própria (ver [`authored_fill`]).
        let c = authored_fill(i, *kind);
        p.fill = Some(Paint::Solid(Rgba8::new(c[0], c[1], c[2], 255)));
        gfx.vec_scene.push_path(p);
    }
    // A arte, ao lado do painel: um quadrado grande e quente, para o desvanecer ser óbvio.
    let mut star: VecPath = rectangle([2.6, -1.0], [5.0, 1.4]);
    star.fill = Some(Paint::Solid(Rgba8::new(232, 150, 60, 255)));
    gfx.vec_scene.push_path(star);
}

fn path_ids(app: &crate::App) -> Vec<VecPathId> {
    app.gfx
        .as_ref()
        .map(|g| g.vec_scene.paths().iter().map(|p| p.id).collect())
        .unwrap_or_default()
}

/// Nomeia, pendura os filhos na moldura e veste quem veste.
///
/// ⚠️ Num frame POSTERIOR ao `build`, e é obrigatório: a entidade de uma forma nasce no
/// `vec_entities::sync`, que corre no frame do desenho.
fn name_and_parent(app: &mut crate::App) {
    let ids = path_ids(app);
    if ids.len() < AUTHORED.len() {
        return;
    }
    let ents: Vec<_> = ids
        .iter()
        .map(|&id| {
            app.vec_entities
                .get(&id)
                .map(|&bits| ph2d_ecs::Entity::from_bits(bits))
        })
        .collect();
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let Some(frame_e) = ents[FRAME] else { return };
    gfx.sim
        .world_mut()
        .entity_mut(frame_e)
        .insert(ph2d_ecs::VecFrame);
    for (i, (_, name, kind)) in AUTHORED.iter().enumerate() {
        let Some(e) = ents[i] else { continue };
        let Ok(mut ent) = gfx.sim.world_mut().get_entity_mut(e) else {
            continue;
        };
        ent.insert(ph2d_ecs::Name::new(*name));
        if let Some(k) = kind {
            ent.insert(ph2d_ecs::VecWidget { kind: k.code() });
        }
        if let Some(slug) = authored_icon(name) {
            ent.insert(ph2d_ecs::VecWidgetIcon { slug: slug.into() });
        }
        // ⚠️ **As opções penduram no CONTROLE, não na moldura** — é o que faz delas opções. Numa
        // moldura elas seriam três rows soltas, que é exactamente o que a lei de posse impede.
        if let Some(pi) = authored_parent(i)
            && let Some(pe) = ents[pi]
        {
            ent.insert(ph2d_ecs::ChildOf(pe));
        }
    }
    // A arte ganha nome e NÃO ganha pai — ela vive na cena, não no painel.
    if let Some(e) = ents.get(STAR).copied().flatten()
        && let Ok(mut ent) = gfx.sim.world_mut().get_entity_mut(e)
    {
        ent.insert(ph2d_ecs::Name::new("Star"));
    }
}

/// Prende a row **Opacity** à estrela — o que o artista faria com o conta-gotas.
///
/// ⚠️ Um lado do fio vem pronto e o outro **não**: com os dois prontos o smoke provaria o
/// resolvedor e não o GESTO, e é o gesto que esta fatia acrescenta. O roteiro manda o artista
/// prender o Toggle com a mão.
fn bind_the_slider(app: &mut crate::App) {
    let ids = path_ids(app);
    if ids.len() <= STAR {
        return;
    }
    let (slider, star) = (ids[2], ids[STAR]);
    let Some(gfx) = app.gfx.as_mut() else { return };
    crate::vec_widget_edit::bind(&mut gfx.sim, &app.vec_entities, slider, star);
}

/// Abre o painel autorado, do mesmo jeito que o interruptor da seção Frame o abre.
///
/// ⚠️ Escreve a MESMA chave que o chip escreve (`visibility_key`), e não um literal: se a moldura
/// for renomeada, o painel muda de identidade e um literal aqui abriria um painel que não existe —
/// em silêncio, que é a cicatriz do painel de física do W2b.
fn open_the_panel(app: &mut crate::App) {
    if let Some(hero) = app.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
        hero.panel_visibility
            .insert(ph2d_panel_authored::visibility_key(), true);
    }
}

fn announce(app: &mut crate::App) {
    let ids = path_ids(app);
    let Some(gfx) = app.gfx.as_ref() else { return };
    if ids.len() <= STAR {
        eprintln!("[ui-panel] ⚠️ a cena nao montou — PARE");
        return;
    }
    let Some(frame_e) = app
        .vec_entities
        .get(&ids[FRAME])
        .map(|&b| ph2d_ecs::Entity::from_bits(b))
    else {
        eprintln!("[ui-panel] ⚠️ a moldura nao tem entidade — PARE");
        return;
    };
    let spec = crate::ui_panel_spec::of(&gfx.sim, &gfx.vec_scene, frame_e);
    eprintln!(
        "[ui-panel] a moldura '{}' descreve um painel de {} row(s).",
        spec.title,
        spec.rows.len()
    );
    if spec.rows.len() != expected_rows() {
        eprintln!(
            "[ui-panel] ⚠️ **PARE**: eram para ser {} rows. O 'Backdrop' e' desenho puro, e as \
             SEIS opcoes ('View' e 'Blend') pertencem aos controles delas — se aparecerem como \
             linhas soltas, a lei de posse quebrou.",
            expected_rows()
        );
        return;
    }
    eprintln!(
        "[ui-panel] o codigo gerado:\n{}",
        ph2d_ui_codegen::emit(&spec)
    );
    eprintln!("[ui-panel] o roteiro:");
    eprintln!("  (⚠️ **'re-rodar'** abaixo = feche o app e rode o MESMO comando de novo. Ele");
    eprintln!("   importa porque o BLOCO DE CODIGO acima e' derivado da cena a CADA lancamento,");
    eprintln!("   enquanto o painel compilado so' muda pelo passo 9.)");
    eprintln!("  1. ⚠️ **A PROVA DA WAVE** esta' acima: o bloco de codigo. Cada row dele e' um");
    eprintln!("     filho VESTIDO da moldura, na ordem em que voce os ve na Hierarquia.");
    eprintln!("  2. ⚠️ **O CONTROLE**: o 'Backdrop' e' filho da moldura e NAO aparece no codigo.");
    eprintln!("     Ele e' desenho — um fundo —, e desenho nao vira controle. Um gerador que");
    eprintln!("     transformasse todo filho em row daria um painel com uma linha morta.");
    eprintln!("  3. Renomeie um filho na Hierarquia e re-rode: o rotulo e a chave acompanham.");
    eprintln!("     O nome que o artista digita e' o que o painel mostra.");
    eprintln!("  4. Reordene os filhos: as rows sairem noutra ordem. A ordem da arvore E' a");
    eprintln!("     decisao — e' ela que o auto layout flui.");
    eprintln!("  5. Tire o `VecWidget` de um filho (secao Widget -> nenhum): ele sai do codigo e");
    eprintln!("     volta a ser desenho.");
    eprintln!("  6. ⚠️ **O PAINEL ESTA' NA TELA** (W8b.2), docado a' ESQUERDA do inspector. Cada");
    eprintln!("     row dele e' uma linha do bloco de codigo acima, compilada. Arraste o slider,");
    eprintln!("     aperte o toggle, clique o botao: eles RESPONDEM — o comportamento e' o dos");
    eprintln!("     widgets do catalogo, nao um interpretador do canvas.");
    eprintln!("  7. ⚠️ **O 'Backdrop' nao esta' no painel** — o CONTROLE, outra vez: quem so'");
    eprintln!("     desenha nao vira row, e quem nao RESPONDE (o cabecalho de secao) nao acende");
    eprintln!("     sob o rato. Um controle que acende e nao faz nada e' pior que um que falta.");
    eprintln!("  8. Feche pelo X do painel e reabra pelo pill **UI** no topbar (ao lado do TOK).");
    eprintln!("     Ha' TRES abridores — o X, o pill, e o chip **Show as Panel** da secao Frame");
    eprintln!("     (com a moldura 'Color' selecionada) — e os tres escrevem o MESMO fato: feche");
    eprintln!("     por um e os outros dois apagam sozinhos. ⚠️ O pill e' o unico que nao exige");
    eprintln!("     a ferramenta Vector em maos nem a moldura selecionada.");
    eprintln!("  9. ⚠️ **O QUE O PAINEL MOSTRA E' A TABELA COMMITADA, nao a do log.** Renomeie um");
    eprintln!("     filho e re-rode: o CODIGO acima muda, o painel NAO — ate' alguem colar o");
    eprintln!("     codigo em crates/ph2d-panel-authored/src/generated/panel.rs e recompilar.");
    eprintln!("     E' o que codegen e'; esconder isso faria o painel parecer quebrado.");
    eprintln!(" 10. ⚠️ **A ROW MEXE NA ARTE** (W8b.3): arraste o slider **Opacity** e olhe o");
    eprintln!("     retangulo laranja a' direita — ele DESVANECE. A tinta autorada nao e'");
    eprintln!("     tocada: leve a zero e volte, e a cor volta exatamente como estava.");
    eprintln!(" 11. **O GESTO e' seu:** selecione o filho 'Visible' na Hierarquia, na secao");
    eprintln!("     Widget Skin aperte **Bind Shape...** e clique no retangulo laranja. Agora o");
    eprintln!("     toggle do painel APAGA e ACENDE a arte. (O slider ja' vinha preso — o");
    eprintln!("     roteiro traz um lado pronto e deixa o outro para voce fazer.)");
    eprintln!(" 12. ⚠️ **O CONTROLE:** selecione 'Reset' (um Button) — a linha *Drives* nao");
    eprintln!("     aparece. Um botao produz um EVENTO, nao um valor; oferecer-lhe o vinculo");
    eprintln!("     daria um conta-gotas que resolve e nao faz nada.");
    eprintln!(" 13. ⚠️ **A POSICAO do controle SOBREVIVE ao arquivo** (W8b.4): ponha o slider em");
    eprintln!("     ~30%, **Ctrl+S**, feche o app, reabra e **Ctrl+O** — a estrela volta a 30%.");
    eprintln!("     A TINTA nao foi tocada: o que o arquivo guarda e' onde o CONTROLE esta'.");
    eprintln!(" 14. E o **Ctrl+Z** move o slider de volta junto com a arte — mover um controle e'");
    eprintln!("     uma edicao, e um passo por GESTO (nao por frame). ⚠️ Abrir a cena, ao");
    eprintln!("     contrario, nao autora nada: a fila de undo nasce vazia.");
    eprintln!(" 15. ⚠️ **O DESENHO E' O GLIFO**: a ultima row chama-se 'Play' e e' um botao de");
    eprintln!("     icone. Olhe a ESTRELA — ela e' a forma que voce desenhou, endireitada e");
    eprintln!("     encaixada na caixa de 24x24 do icone, com a moldura do botao a' volta. No");
    eprintln!("     canvas e no painel tem de ser **a mesma estrela**: se so' um dos dois a");
    eprintln!("     mostrar, PARE — e' exatamente a divergencia que a porta unica existe para");
    eprintln!("     impedir. E o glifo SEGUE o desenho, com TRES relogios diferentes: edite os");
    eprintln!("     nos da estrela no modo Node e o CANVAS muda na hora (a pele e' cozida a cada");
    eprintln!("     frame); o bloco de codigo do log so' muda ao re-rodar; e o PAINEL compilado");
    eprintln!("     so' muda pelo passo 9. Um re-run NAO basta para o painel.");
    eprintln!("     ⚠️ **O limite, dito:** girar a forma pelo gizmo NAO gira o glifo — ele e' o");
    eprintln!("     desenho autorado, nao a pose.");
    eprintln!(
        " 16. ⚠️ **A SEGUNDA ROTA, na row de baixo:** 'Trash' e' um botao de icone tambem, e"
    );
    eprintln!("     ele NAO mostra a forma que o veste — mostra o LIXO do catalogo do editor,");
    eprintln!("     porque o artista ESCOLHEU. As duas rotas estao lado a lado de proposito: com");
    eprintln!("     so uma delas, 'a escolha vence' e 'o botao sempre desenha a forma' seriam a");
    eprintln!("     mesma foto. Selecione 'Trash', na secao Widget Skin aperte **Icon...** e");
    eprintln!("     escolha outro glifo: o canvas E o painel mudam juntos. Escolha **Drawing** no");
    eprintln!("     topo da lista e ele volta a desenhar a estrela — tirar a escolha E' voltar ao");
    eprintln!("     desenho, nao um terceiro estado.");
    eprintln!(" 17. ⚠️ **A SECAO DOBRA**: clique no cabecalho **Appearance** do painel. Todas as");
    eprintln!("     rows sob ele somem e o painel ENCOLHE — nao fica um buraco do tamanho delas.");
    eprintln!(
        "     O cabecalho continua la' (e' a alca de volta); clique outra vez e elas voltam."
    );
    eprintln!("     ⚠️ E o colapso e' o do APP, o mesmo dos 23 paineis escritos a' mao — nao um");
    eprintln!("     segundo que dobraria por regras proprias.");
    eprintln!(" 18. ⚠️ **A FAMILIA DE LISTA: as opcoes sao os FILHOS.** A row **View** e' uma");
    eprintln!("     faixa de abas com tres opcoes — 'Design', 'Preview', 'Code' —, e elas nao sao");
    eprintln!("     rows: sao filhos QUE ELA POSSUI. Renomeie um deles na Hierarquia e a aba muda");
    eprintln!("     de nome; se em vez disso aparecer uma linha nova solta no painel, a lei de");
    eprintln!("     posse quebrou. Clique noutra aba: ela acende.");
    eprintln!(" 19. ⚠️ **E a ultima row, 'Blend', ESCONDE as opcoes ate' se pedir** — e' um");
    eprintln!("     dropdown, o unico do catalogo que nao cabe num passe de pintura so'. Clique");
    eprintln!("     no chip: a lista abre POR CIMA de tudo, inclusive do canto de");
    eprintln!("     redimensionar. Escolha 'Screen': ela **fecha** e o chip passa a dizer");
    eprintln!("     'Screen'. Se a lista ficar aberta depois da escolha, PARE — o clique");
    eprintln!("     seguinte, que voce daria para a fechar, escolheria outra coisa.");
    eprintln!(
        "     ⚠️ **A pergunta de olho**: arraste o painel para BAIXO ate' o chip ficar perto"
    );
    eprintln!("     do fundo da tela e abra outra vez. A lista tem de virar para CIMA — e as");
    eprintln!("     opcoes tem de responder ao clique **onde estao desenhadas**.");
    eprintln!(" 20. ⚠️ **A SEGUNDA SECAO, e e' ela que torna o passo 17 uma PROVA:** ha' agora um");
    eprintln!("     cabecalho **Advanced** com 'Seed' e 'Dither' sob ele. Dobre o **Appearance**:");
    eprintln!("     as rows dele somem, o painel encolhe, e o **Advanced com as suas duas rows");
    eprintln!("     continua inteiro na tela**. Com um cabecalho so', 'esconde ate' o PROXIMO' e");
    eprintln!("     'esconde tudo abaixo' davam a mesma foto — nao havia nada depois para a");
    eprintln!("     diferenca aparecer. Dobre os dois: o painel fica so' com os dois cabecalhos.");
    eprintln!(
        "     ⚠️ E o defeito que este passo achou NAO era o colapso: era o REGISTO. A tabela"
    );
    eprintln!(
        "     viva e' publicada por quadro e o `populate` corria uma vez, no arranque, sobre"
    );
    eprintln!("     a tabela COMPILADA — entao qualquer row que voce autora e o codigo colado");
    eprintln!("     ainda nao tem nascia *pintada, com retangulo de hit, e morta sob o rato*. Era");
    eprintln!("     invisivel porque as chaves COINCIDIAM: o golden vem desta mesma cena.");
    eprintln!(" 21. ⚠️ **O CONTROLE do passo 20**: desenhe voce um retangulo NOVO dentro da");
    eprintln!("     moldura, de-lhe um nome e vista-o de **Section Header** pela secao Widget");
    eprintln!("     Skin. Ele aparece no painel **e dobra na hora** — sem colar codigo nenhum e");
    eprintln!("     sem recompilar. E' o passo 9 pelo avesso: o que a tabela viva ACRESCENTA");
    eprintln!("     responde ja'; o que ela RENOMEIA no codigo colado continua a precisar do");
    eprintln!("     passo 9.");
    eprintln!(" 22. ⚠️ **O QUE VOCE CRIA COMPORTA-SE COMO O QUE VEIO NO CODIGO.** Naquele mesmo");
    eprintln!("     retangulo, troque o tipo para **Checkbox** e clique-o no painel: ele tem de");
    eprintln!("     FICAR MARCADO. Depois um **Toggle**, um **Slider**: cada um responde como o");
    eprintln!("     irmao dele que veio da tabela compilada.");
    eprintln!(
        "     ⚠️ A causa do report de 09/08 (*'o meu nao fica checado mas emite sinal; o seu"
    );
    eprintln!(
        "     nao emite sinal mas pode ser checado'*) e' UMA so', e ela e' o caminho NORMAL:"
    );
    eprintln!("     **Wear a Widget cria um BUTTON**, e so' depois voce escolhe o tipo. O estado");
    eprintln!("     do Button ficava no store, entao o painel nao tinha o que marcar e o clique");
    eprintln!("     caia no `Fired` dos BOTOES — que e' o unico intent que vira SINAL. Com");
    eprintln!("     PH2D_SIGNAL_LOG=1, um checkbox NAO pode imprimir '[signal]'; um Button pode.");
    eprintln!("     ⚠️ E o inverso tambem: um **Section Header** que vira Checkbox tem de MARCAR,");
    eprintln!("     nao dobrar — a marca de secao e' consultada ANTES do tipo do widget.");
    eprintln!(" 23. ⚠️ **A SWATCH ABRE O PICKER, e a cor VOLTA para a forma.** Clique na row");
    eprintln!("     **Tint**: o picker OKLCH abre **na cor da forma**, nao num cinzento. Escolha");
    eprintln!("     outra: a swatch acompanha E o retangulo do canvas que a veste muda junto —");
    eprintln!("     eles sao a MESMA cor, lida do preenchimento do desenho. Se a swatch voltar a'");
    eprintln!("     cor antiga no quadro seguinte, PARE: o picker abriu e a viagem de volta nao");
    eprintln!("     aconteceu, que e' pior que uma swatch muda porque PARECE funcionar.");
    eprintln!(
        "     ⚠️ O picker e' o do APP, o mesmo do Painter e do Vector — o `pointer_down` diz"
    );
    eprintln!("     em codigo que qualquer painel que pinte uma ColorSwatch e a registre o ganha.");
    eprintln!(" 24. ⚠️ **O ICONE NAO TRANSBORDA MAIS.** Olhe as rows **Play** e **Trash**: a");
    eprintln!("     estrela e o lixo ficam DENTRO da moldura do botao, com folga. A caixa do");
    eprintln!("     glifo media 32 px numa row de 28 — 4 px mais alta que o botao que a contem —,");
    eprintln!(
        "     e so' um glifo que PREENCHE a viewbox o revelava: os do catalogo deixam margem"
    );
    eprintln!("     dentro dos 24x24 deles, a estrela do artista nao. Compare com os chips da");
    eprintln!("     TopBar: eles NAO se moveram (tem folga de sobra, e a lei do estilo e' 'no");
    eprintln!("     visual change on migration').");
    eprintln!(" 25. ⚠️ **O CONTROLE do passo 24**: a row **Loading** e' uma barra de progresso —");
    eprintln!("     ela DESENHA e nunca responde. Clique nela: nada acontece, e nada deve. Ela");
    eprintln!("     esta' na cena porque um gate precisa de exactamente uma row assim para ter");
    eprintln!("     sujeito, e ele ficou sem uma no dia em que a swatch passou a abrir o picker.");
}

#[cfg(test)]
#[path = "ui_panel_smoke_tests.rs"]
mod tests;
