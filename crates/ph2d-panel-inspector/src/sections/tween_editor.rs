//! ⭐⭐⭐ **O EDITOR do tween aberto** (suplente #22) — os campos daquele que a lista escolheu.
//!
//! ⚠️ **A moldura é o [`super::tween`]** (o cabeçalho, a lista e os dois botões); aqui vive só o que
//! descreve UM tween. *A lista escolhe; o editor mostra* — a frase que o cabeçalho da irmã já
//! escrevia antes de o corte existir.
//!
//! # ⭐⭐ A QUEIXA vem antes dos números
//!
//! | aviso | o que se passa |
//! |---|---|
//! | `no timer at this slot` | ⛔ **o tween não tem relógio** — ele nem chega a correr |
//! | `the timer at this slot has no duration` | ⛔ há relógio e ele **nunca dispara** (duração `0`) |
//! | `this object has no sprite` | o canal escreve num campo do `Sprite`, e não há nenhum |
//! | `from and to are the same` | ele corre e **não move nada** |
//! | `a silhouette that holds stays lit` | ⚠️ funciona, e quase de certeza não é o que se quer |
//!
//! ⚠️ **Os dois primeiros são de outra espécie que os dois últimos:** ali o tween **não corre**,
//! aqui ele corre. *Dizer «ele não move nada» a quem não tem relógio é mandá-lo resolver a metade
//! errada* — a lei da recusa dos pincéis.
//!
//! ⛔⛔ **A ordem NÃO vive aqui**, e é isso que a torna testável: ela é a porta
//! [`InspectorTweenRow::queixa`], e o gate dela corre **sem um device**.
//!
//! # ⭐⭐⭐ Onde mora o TEMPO — a pergunta do dono, em 2026-09-19
//!
//! *«onde selecciono o tempo?»*, no smoke da `PH2D_TWEEN_SMOKE=1`. ⛔ **Um tween é uma função PURA
//! do relógio, logo ele não tem duração nenhuma** — quem a tem é o `Timers[i]` do MESMO índice. O
//! painel sabia-o e **não o dizia**, e a pergunta é a medição disso.
//!
//! ⚠️ **A linha é um READOUT e nunca um segundo campo**, e a razão é o próprio modelo: aquele
//! relógio **não é privado deste tween** (ele publica um sinal, ele arranca uma cutscene, ele conta
//! para uma fábrica), logo editá-lo daqui mudaria em silêncio o que mais estivesse pendurado nele.
//! *Duas superfícies sobre um valor divergem no dia em que uma ganhar clamp* — a lei que os três
//! chips de `Detail` da escultura pagaram.
//!
//! # ⚠️ A curva é onze botões, e por isso são TRÊS fileiras
//!
//! A coluna do Inspector tem ~300 px; onze numa fileira dão ~25 px cada, e um rótulo que não cabe é
//! um chip que o artista não lê. *O corte não é do modelo — as `33` curvas do motor continuam todas
//! alcançáveis —, é da LARGURA.*

use super::*;
use ph2d_editor_core::tween_edits::{InspectorTweenRow, TweenQueixa};
use ph2d_i18n::{tr, tr_with};
use ph2d_tween::{AoAcabar, Canal};

/// **A CHAVE de cada queixa — a PORTA, e não um `match` dentro do pintor.**
///
/// ⛔ Ela traduz o enum da lei numa chave de i18n, e é o único sítio onde as duas coisas se tocam:
/// a lei não conhece a língua e o pintor não decide a ordem.
#[must_use]
const fn chave_da_queixa(q: TweenQueixa) -> &'static str {
    match q {
        TweenQueixa::SemRelogio => "panel.inspector.tween.no_timer_at_this_slot",
        TweenQueixa::RelogioSemDuracao => "panel.inspector.tween.the_timer_here_has_no_duration",
        TweenQueixa::SemSprite => "panel.inspector.tween.this_object_has_no_sprite",
        TweenQueixa::Inerte => "panel.inspector.tween.from_and_to_are_the_same",
        TweenQueixa::SilhuetaQueFica => "panel.inspector.tween.a_silhouette_that_holds_stays_lit",
    }
}

/// ⭐⭐⭐ **A COLUNA DO NOME desta secção — UMA, para a duração, o De/Para e as SEIS escolhas.**
///
/// ⛔⛔ Ela era medida em DOIS sítios com listas diferentes (o relógio só com `Duration`, o De/Para
/// só com `From`/`To`) e as escolhas não entravam em nenhuma — duas colunas dentro de uma secção.
/// *Uma coluna por espécie de linha é uma coluna por linha com outro nome.*
pub(super) fn seccao_do_tween(
    text_system: &mut TextSystem,
    campos: usize,
) -> ph2d_editor_core::property_row::Seccao {
    let nomes: Vec<&str> = [
        "panel.inspector.tween.duration_seconds",
        "panel.inspector.tween.from",
        "panel.inspector.tween.to",
        "panel.inspector.tween.preset",
        "panel.inspector.tween.channel",
        "panel.inspector.tween.curve",
        "panel.inspector.tween.ease",
        "panel.inspector.tween.when_done",
        "panel.inspector.tween.cycle",
    ]
    .into_iter()
    .map(tr)
    .collect();
    ph2d_editor_core::property_row::Seccao::medida(text_system, campos, &nomes)
}

/// **Uma escolha com nome** — pela porta [`ph2d_editor_core::property_row::paint_choice_row`].
///
/// ⭐⭐ **`pub(super)` desde o suplente #23**: PATH FOLLOW, SHAKE e o EMISSOR fazem a mesma pergunta.
///
/// ⛔⛔ **Ela partia os chips em blocos de largura IGUAL** (`cabem_por_fileira`, medida no rótulo
/// mais largo) e pintava o nome POR CIMA. Hoje a forma é da porta, que MEDE: uma escolha que cabe
/// numa fileira ao lado do nome vai ao lado; uma que não cabe é uma PALETA a toda a largura, e cada
/// peça leva o que a PALAVRA dela pede — que é o que a foto de 2026-09-19 (os `Positio…` cortados)
/// pedia, sem a medida escrita à parte.
#[allow(clippy::too_many_arguments)]
pub(super) fn grupo(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    label: &str,
    ids_: &[NodeId],
    labels: &[&str],
    sel: usize,
    seccao: ph2d_editor_core::property_row::Seccao,
) -> f32 {
    // ⚠️ **A selecção vem do SNAPSHOT**, nunca do store: o store guarda o visual do botão, e ler
    //    dali qual está aceso faria o realce sobreviver à troca de objecto.
    let segmentos: Vec<(&str, bool, NodeId)> = ids_
        .iter()
        .zip(labels.iter())
        .enumerate()
        .map(|(i, (&id, &t))| (t, i == sel, id))
        .collect();
    ph2d_editor_core::property_row::paint_choice_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        label,
        &segmentos,
        seccao,
    )
}

/// ⭐⭐⭐ **O bloco do RELÓGIO** — a duração e os dois interruptores, dentro da secção do tween.
/// Devolve o `y` seguinte.
///
/// ⛔⛔ **Eles escrevem no MESMO `Timers[i]` que a secção TIMERS, pela MESMA porta.** Não é uma
/// segunda superfície sobre um valor (a armadilha que os três chips de `Detail` da escultura
/// pagaram): é a porta com **dois chamadores**, como o teclado e o menu do `project_io`. Quem
/// clampa, quem satura e quem recusa continua a ser um só.
///
/// ⚠️ **A legenda NOMEIA o timer** porque ele não é privado deste tween — ele pode estar a arrancar
/// uma cutscene, a alimentar uma fábrica ou a publicar um sinal, e o artista tem de saber que é o
/// mesmo objecto que vê na outra secção.
#[allow(clippy::too_many_arguments)]
fn relogio(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    row: &InspectorTweenRow,
    slot: usize,
) -> f32 {
    let mut cur_y = super::rows::aviso(
        scene,
        text_system,
        theme,
        x,
        w,
        y,
        &tr_with(
            "panel.inspector.tween.clock_is_timer",
            &[("n", &(slot + 1))],
        ),
        ColorToken::Text3,
    );
    let sec = seccao_do_tween(text_system, 1);
    cur_y = super::rows::fields_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.tween.duration_seconds"),
        &[crate::ids::INSP_TWEEN_DURACAO],
        0.1, // LITERAL-PX-OK: passo de scrub em SEGUNDOS, como o da secção TIMERS
        None,
        sec,
    );
    // ⚠️ **O valor das caixas vem do SNAPSHOT**, nunca do store: ler dali faria a caixa sobreviver
    // à troca de objecto — a lei que a §11 escreveu para o `Playing`.
    ph2d_editor_core::property_row::paint_check_rows(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        &[
            (
                crate::ids::INSP_TWEEN_REPEAT,
                tr("panel.inspector.tween.repeat"),
                row.repeat,
            ),
            (
                crate::ids::INSP_TWEEN_AUTOSTART,
                tr("panel.inspector.tween.autostart"),
                row.autostart,
            ),
        ],
        sec,
    )
}

/// ⭐ **As duas linhas de CAMPOS de um canal numérico** — o outro ramo do [`editor`].
///
/// ⚠️ **Ela existe porque o ramo da COR é exclusivo deste**: separá-los em duas funções é o que
/// impede alguém de pintar os dois e dar ao artista duas respostas para a mesma pergunta.
#[allow(clippy::too_many_arguments)]
fn campos_de_para(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    canal: Canal,
) -> f32 {
    let mut cur_y = y;
    let n = canal.aridade();
    let seccao = seccao_do_tween(text_system, n);
    for (label, ids_) in [
        (tr("panel.inspector.tween.from"), &crate::ids::INSP_TWEEN_DE),
        (tr("panel.inspector.tween.to"), &crate::ids::INSP_TWEEN_PARA),
    ] {
        cur_y = super::rows::fields_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            label,
            &ids_[..n],
            0.05, // LITERAL-PX-OK: passo de arrasto — adimensional numa cor, metros numa pose
            None,
            seccao,
        );
    }

    cur_y
}
/// ⭐⭐ **Os rótulos dos quatro selectores desta secção, RESOLVIDOS.**
///
/// ⚠️ **Eles saem daqui e não do corpo do [`editor`] por MEDIÇÃO:** a integração de 2026-09-20 pôs
/// os quatro motores (`Preset` · `Canal` · `AoAcabar` · `Ciclo`) a publicar `label_key()`, e as
/// quatro linhas a resolvê-la levaram aquela função a `205` de um tecto de `200`. A cura de um
/// tecto é o CORTE, nunca uma entrada no `FN_OVERAGE_OK` — e o corte é por RESPONSABILIDADE: isto
/// é *«a palavra que cada chip mostra»*, que é um assunto só.
///
/// ⭐ A lei que eles obedecem é a da fronteira dos motores: **o motor publica a CHAVE e quem pinta
/// é que a resolve** — um motor que devolvesse a palavra seria uma segunda tabela de strings.
struct Rotulos;

impl Rotulos {
    fn presets() -> Vec<&'static str> {
        ph2d_tween::Preset::ALL
            .iter()
            .map(|p| tr(p.label_key()))
            .collect()
    }
    fn canais() -> Vec<&'static str> {
        Canal::ALL.iter().map(|c| tr(c.label_key())).collect()
    }
    fn fins() -> Vec<&'static str> {
        AoAcabar::ALL.iter().map(|a| tr(a.label_key())).collect()
    }
    fn ciclos() -> Vec<&'static str> {
        ph2d_tween::Ciclo::ALL
            .iter()
            .map(|c| tr(c.label_key()))
            .collect()
    }
}

/// ⭐ **A FORMA do movimento — a família e o ease**, as duas escolhas que respondem à mesma
/// pergunta (*como o valor viaja de um ponto ao outro*). Saiu do [`editor`] por tecto de função
/// (`201/200` quando a escolha passou pela porta com o nome ao lado).
#[allow(clippy::too_many_arguments)]
fn curva_e_ease(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    mut y: f32,
    row: &InspectorTweenRow,
    sec_escolhas: ph2d_editor_core::property_row::Seccao,
) -> f32 {
    let familias: Vec<&str> = ph2d_anim::EasingFamily::ALL
        .iter()
        .map(|f| ph2d_i18n::tr(f.label_key()))
        .collect();
    y = grupo(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        tr("panel.inspector.tween.curve"),
        &crate::ids::INSP_TWEEN_FAMILIA,
        &familias,
        row.familia as usize,
        sec_escolhas,
    );
    let modos: Vec<&str> = ph2d_anim::EasingMode::ALL
        .iter()
        .map(|m| ph2d_i18n::tr(m.label_key()))
        .collect();
    y = grupo(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        tr("panel.inspector.tween.ease"),
        &crate::ids::INSP_TWEEN_MODO,
        &modos,
        row.modo as usize,
        sec_escolhas,
    );
    y
}

/// O editor do tween aberto. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(super) fn editor(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    row: &InspectorTweenRow,
    tem_sprite: bool,
    slot: usize,
) -> f32 {
    // ⭐ A coluna da secção, UMA vez — ver [`seccao_do_tween`].
    let sec_escolhas = seccao_do_tween(text_system, 1);
    let mut cur_y = y;
    // ⚠️ **A QUEIXA primeiro** — quem não vê nada mexer não quer afinar uma curva.
    //
    // ⚠️ **O `n` vai SEMPRE**, e o [`ph2d_i18n::tr_with`] ignora o que a frase não pede: as quatro
    // queixas que não nomeiam o relógio ficam byte a byte como eram, e a quinta (*«o timer N não tem
    // duração»*) diz a QUAL ir. *Um argumento a mais é grátis; um `match` a decidir quem o recebe
    // seria a segunda tabela sobre a mesma pergunta.*
    if let Some(q) = row.queixa(tem_sprite) {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            &tr_with(chave_da_queixa(q), &[("n", &(slot + 1))]),
            ColorToken::Text3,
        );
    }
    // ⭐⭐⭐ **Os PRESETS primeiro** — eles reescrevem tudo o que vem a seguir, e é isso que os põe
    // em cima: *um botão que muda os cinco campos abaixo dele lê-se; um que os muda acima, não.*
    let presets = Rotulos::presets();
    cur_y = grupo(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.tween.preset"),
        &crate::ids::INSP_TWEEN_PRESET,
        &presets,
        // ⚠️ **NENHUM fica aceso**, e é a decisão: um preset não é um MODO — depois do clique ele
        // desaparece e sobram os cinco campos. Acender um deles prometeria um estado que o
        // componente não guarda, e ele mentiria no instante em que o artista afinasse um número.
        usize::MAX,
        sec_escolhas,
    );
    // ⭐⭐⭐ **O RELÓGIO, AQUI** — ver o cabeçalho deste ficheiro.
    //
    // ⚠️ **Vem DEPOIS dos presets de propósito:** eles reescrevem a duração, e vê-la mudar debaixo
    // do botão é o que prova ao artista que UM clique fez as duas coisas. ⛔ E o bloco inteiro
    // desaparece quando não há relógio, porque aí quem fala é a queixa — *duas superfícies sobre a
    // mesma ausência ensinam que são dois problemas*.
    if row.duracao_us.is_some() {
        cur_y = relogio(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            row,
            slot,
        );
    }
    let canal = Canal::from_tag(row.canal);
    let canais = Rotulos::canais();
    cur_y = grupo(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.tween.channel"),
        &crate::ids::INSP_TWEEN_CANAL,
        &canais,
        canal.tag() as usize,
        sec_escolhas,
    );

    // ⭐ **Uma componente ou quatro — DERIVADO do canal**, nunca uma segunda lista.
    // ⭐⭐⭐ **UMA COR É UMA AMOSTRA, NUNCA QUATRO NÚMEROS** — report do dono, 2026-09-19 (com foto):
    // *«por que usar cores em números se temos caixas selectoras?»*.
    //
    // ⚠️ **A escolha é DERIVADA do canal** ([`Canal::e_cor`]) e os dois caminhos são exclusivos:
    // pintar os dois daria duas respostas a *«que cor é esta?»*, e elas divergiriam no primeiro
    // arrasto de um dos campos.
    if canal.e_cor() {
        cur_y = super::color_tint::bloco_de_cores(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            &[
                (
                    crate::ids::INSP_TWEEN_COR_DE,
                    tr("panel.inspector.tween.from"),
                    row.de,
                ),
                (
                    crate::ids::INSP_TWEEN_COR_PARA,
                    tr("panel.inspector.tween.to"),
                    row.para,
                ),
            ],
        ) + ph2d_tokens::control_gap_px();
    } else {
        cur_y = campos_de_para(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            canal,
        );
    }

    cur_y = curva_e_ease(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        row,
        sec_escolhas,
    );
    let fins = Rotulos::fins();
    cur_y = grupo(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.tween.when_done"),
        &crate::ids::INSP_TWEEN_AO_ACABAR,
        &fins,
        row.ao_acabar as usize,
        sec_escolhas,
    );
    // ⭐⭐⭐ **O CICLO — o *ping-pong* que o dono pediu em 2026-09-19.**
    //
    // ⚠️ **Ele vem DEPOIS do `When Done` de propósito:** os dois falam do tempo, e a ordem é a da
    // pergunta que o artista faz — *o que acontece DENTRO de uma volta* lê-se depois de *o que
    // acontece no FIM*, porque é o fim que ele já conhece do resto do painel.
    let ciclos = Rotulos::ciclos();
    grupo(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.tween.cycle"),
        &crate::ids::INSP_TWEEN_CICLO,
        &ciclos,
        row.ciclo as usize,
        sec_escolhas,
    )
}
