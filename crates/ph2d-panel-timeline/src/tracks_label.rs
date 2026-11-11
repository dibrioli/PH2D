//! **Como este app NOMEIA um canal animado** — `prop_label` e `track_label`.
//!
//! Módulo irmão do `tracks.rs` pelo cap de 600 LOC, cortado pelo assunto que os
//! dois já separavam: *como o dope-sheet DESENHA* × *que palavra ele mostra*.
//!
//! ⚠️ **`track_label` é porta ÚNICA, e a razão é literal:** o card de Expressão
//! nasceu montando esta string por conta própria, e o doc-comment dele prometia
//! `"Ball · Position Y"` desde o primeiro dia enquanto o código escrevia
//! `#nnnn`. A row e o card do mesmo canal dizendo coisas diferentes é
//! exactamente o que uma segunda cópia produz.

use ph2d_timeline::PropKind;

/// The display label for a property (the panel's presentation of `PropKind`).
pub(crate) fn prop_label(p: PropKind) -> &'static str {
    match p {
        PropKind::TranslationX
        | PropKind::TranslationY
        | PropKind::Rotation
        | PropKind::ScaleX
        | PropKind::ScaleY
        | PropKind::Opacity
        | PropKind::Position => palavra_do_inspector(p),
        PropKind::TimeRemap => ph2d_i18n::tr("panel.timeline.prop.time"),
        PropKind::Morph => ph2d_i18n::tr("panel.timeline.prop.morph"),
        PropKind::JointMotorTarget => ph2d_i18n::tr("panel.timeline.prop.motor_target"),
        PropKind::JointMotorSpeed => ph2d_i18n::tr("panel.timeline.prop.motor_speed"),
        PropKind::JointRestLength => ph2d_i18n::tr("panel.timeline.prop.rest_length"),
        PropKind::JointMaxLength => ph2d_i18n::tr("panel.timeline.prop.max_length"),
        PropKind::BoneBendInX => ph2d_i18n::tr("panel.timeline.prop.bone_bend_in_x"),
        PropKind::BoneBendInY => ph2d_i18n::tr("panel.timeline.prop.bone_bend_in_y"),
        PropKind::BoneBendOutX => ph2d_i18n::tr("panel.timeline.prop.bone_bend_out_x"),
        PropKind::IkBendSide => ph2d_i18n::tr("panel.timeline.prop.ik_bend_side"),
        PropKind::BoneBendOutY => ph2d_i18n::tr("panel.timeline.prop.bone_bend_out_y"),
        PropKind::MacroTime => ph2d_i18n::tr("panel.timeline.prop.macro_time"),
        // O rótulo do GRUPO, do vocabulário da família (a forma inteira lê-se «Forma»).
        PropKind::Field(k) => match ph2d_ecs::canal_componente::nome_do_cartao(k) {
            Some(i) => palavra_do_cartao(i),
            None => ph2d_i18n::tr(k.info().map_or("panel.timeline.prop.field", |i| i.rotulo)),
        },
    }
}

/// O campo do catálogo que o cartão junta numa linha de glifos diz a palavra da linha mais o eixo
/// («Flip image H», `NOMES_DO_CARTAO`), como [`palavra_do_inspector`] diz «Position X».
fn palavra_do_cartao(i: usize) -> &'static str {
    use ph2d_i18n::tr;
    static NOMES: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
    let n = NOMES.get_or_init(|| {
        (ph2d_ecs::canal_componente::NOMES_DO_CARTAO.iter())
            .map(|&(.., linha, eixo)| {
                let linha = tr(linha).split(" (").next().unwrap_or_default();
                format!("{linha} {}", tr(eixo))
            })
            .collect()
    });
    n.get(i).map_or("", String::as_str)
}

/// ⭐ **Dono 10/11: o canal do Transform diz a palavra do INSPECTOR** («Position X», nunca
/// «Translate X»). A fonte é a chave do cartão do Inspector, sem a unidade entre parênteses (a
/// Timeline põe a sua, [`propriedade_snap`]) e com o glifo do eixo do mesmo cartão: renomear lá
/// renomeia cá. Gate `the_channel_says_the_inspectors_word`.
fn palavra_do_inspector(p: PropKind) -> &'static str {
    use ph2d_i18n::tr;
    static NOMES: std::sync::OnceLock<[String; 7]> = std::sync::OnceLock::new();
    let sem_unidade = |k: &str| tr(k).split(" (").next().unwrap_or_default().to_owned();
    let eixo = |k: &str, g: &str| format!("{} {}", sem_unidade(k), tr(g));
    let n = NOMES.get_or_init(|| {
        [
            eixo("card.insp.position_px", "card.glyph.x"),
            eixo("card.insp.position_px", "card.glyph.y"),
            sem_unidade("card.insp.rotation_deg"),
            eixo("card.insp.scale", "card.glyph.x"),
            eixo("card.insp.scale", "card.glyph.y"),
            sem_unidade("card.insp.opacity"),
            sem_unidade("card.insp.position_px"),
        ]
    });
    let i = match p {
        PropKind::TranslationX => 0,
        PropKind::TranslationY => 1,
        PropKind::Rotation => 2,
        PropKind::ScaleX => 3,
        PropKind::ScaleY => 4,
        PropKind::Opacity => 5,
        _ => 6,
    };
    &n[i]
}

/// **How this app names an animated channel** — `Ball · Position X`, or the short id
/// `Position X  #7294` when the object published no name (FASE C.3 do plano 12).
///
/// A porta ÚNICA, e a razão é literal: o card de Expressão nasceu montando esta string
/// por conta própria, e o doc-comment dele **prometia** `"Ball · Position Y"` desde o
/// primeiro dia enquanto o código escrevia `#nnnn` — a row e o card do mesmo canal
/// dizendo coisas diferentes é exactamente o que uma segunda cópia produz.
///
/// **O nome vem primeiro** porque é o que o artista varre numa coluna estreita: com
/// seis tracks do mesmo objeto, a propriedade é o que muda de linha para linha, e o
/// nome é a âncora que diz de quem é o bloco.
///
/// Um canal de família cujo grupo se parte por entrada (a tinta e o traço da camada `i` da pilha)
/// leva o número da entrada — «Fill 2» —, senão a camada e a base liam-se iguais.
pub(crate) fn track_label(name: Option<&str>, entity: u64, prop: PropKind) -> String {
    compoe(dono(name, prop), entity, nome_do_canal(prop))
}

/// O nome do dono do canal: o do objecto, ou «Scene» para um canal do mundo (F30 §B).
fn dono(name: Option<&str>, prop: PropKind) -> Option<&str> {
    let cena = matches!(prop, PropKind::Field(k) if k.da_cena());
    name.or(cena.then(|| ph2d_i18n::tr("canal.mundo.cena")))
}

/// Só a propriedade («Fill 2», «Rotation») — o que a faixa diz sob o cabeçalho do objecto. Pública
/// para o portão «o canal diz o nome do campo do Inspector» (`ph2d-app-components`).
pub fn nome_do_canal(prop: PropKind) -> String {
    match prop {
        PropKind::Field(k) => match k.grupo() {
            // Com `item` o sub é o id, nunca um número de linha (o nome vem de `channel_names`).
            Some((_, _, i)) if i > 0 && k.item.is_none() => format!("{} {i}", prop_label(prop)),
            _ => prop_label(prop).to_owned(),
        },
        _ => prop_label(prop).to_owned(),
    }
}

/// ⭐ **Nos cartões, a faixa sob o cabeçalho do objecto** (doc 15 onda 2): a propriedade com a unidade
/// do projecto entre parênteses quando o canal tem grandeza («Radius (px)», «Angle (°)»); o objecto
/// já está escrito em negrito na linha de cima ([`objeto_snap`]).
pub(crate) fn propriedade_snap(
    snap: &ph2d_timeline::TimelineViewSnapshot,
    prop: PropKind,
    u: crate::tracks_value::Unidades,
) -> String {
    let base = snap
        .channel_name(prop)
        .map_or_else(|| nome_do_canal(prop), str::to_owned);
    match crate::tracks_value::sufixo_da_unidade(prop, u) {
        Some(su) => format!("{base} ({})", su.trim()),
        None => base,
    }
}

/// O cabeçalho do objecto nos cartões: o nome, ou o id curto quando ele não publicou nenhum.
pub(crate) fn objeto_snap(snap: &ph2d_timeline::TimelineViewSnapshot, entity: u64) -> String {
    let prop = snap
        .tracks
        .iter()
        .find(|t| t.entity == entity)
        .map(|t| t.prop);
    match prop.and_then(|p| dono(snap.object_name(entity), p)) {
        Some(n) => n.to_owned(),
        // O id curto sai da porta única: o rótulo vazio deixa só «  #nnnn».
        None => compoe(None, entity, String::new()).trim().to_owned(),
    }
}

/// [`track_label`] lido do instantâneo: o nome do objecto, e o da propriedade quando a row tem um
/// (`TimelineViewSnapshot::channel_name` — a que o script declara).
pub(crate) fn track_label_snap(
    snap: &ph2d_timeline::TimelineViewSnapshot,
    entity: u64,
    prop: PropKind,
) -> String {
    match snap.channel_name(prop) {
        Some(n) => compoe(snap.object_name(entity), entity, n.to_owned()),
        None => track_label(snap.object_name(entity), entity, prop),
    }
}

fn compoe(name: Option<&str>, entity: u64, rotulo: String) -> String {
    match name {
        Some(n) => format!("{n} · {rotulo}"),
        // O fallback é o rótulo que o dope-sheet sempre teve. Ele NÃO é sinal de erro:
        // um objeto sem `Name` é transiente, e mostrar `#nnnn` continua distinguindo
        // duas rows do mesmo tipo em objetos diferentes.
        None => format!("{rotulo}  #{}", entity % 10_000),
    }
}
