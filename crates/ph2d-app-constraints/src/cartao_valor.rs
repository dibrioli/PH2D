//! **Os ▾ do Follow Value** (`docs/BlenderConstraints/06_W5_resto.md` §3.1): que propriedades um
//! objecto tem — as cinco da pose e cada campo que ele publica em `CanaisLidos` —, com o nome do
//! dope-sheet (o rótulo do grupo `CampoInfo::rotulo`, o número da entrada e o `FieldKey::membro`), a
//! ordem das peças do cartão e a escrita da escolha.

use std::collections::BTreeMap;

use bevy_ecs::entity::Entity;
use ph2d_constraints::{Kind, Prop};
use ph2d_ecs::canal_constraint::{CANAIS, DE, PARA};
use ph2d_ecs::{CanaisLidos, FieldKey, World};
use ph2d_editor_core::constraint_edits::{
    AjustePropriedade, MAX_PROPRIEDADES, OpcaoPropriedade, Peca, PropriedadeDoValor as P,
};
use ph2d_i18n::tr;

use crate::campos::CANAIS_DO_TIPO;
use crate::valor::chave;

/// O espelho do painel de um [`Prop`].
#[must_use]
pub fn espelho(p: Prop) -> P {
    match p {
        #[expect(clippy::cast_possible_truncation, reason = "cinco canais")]
        Prop::Pose(c) => P::Canal(CANAIS_DO_TIPO.iter().position(|&x| x == c).unwrap_or(0) as u8),
        Prop::Campo(k) => P::Campo(k),
    }
}

/// O inverso de [`espelho`] (`None` = canal fora da lista).
#[must_use]
pub fn prop(p: P) -> Option<Prop> {
    Some(match p {
        P::Canal(i) => Prop::Pose(*CANAIS_DO_TIPO.get(usize::from(i))?),
        P::Campo(k) => Prop::Campo(k),
    })
}

/// Os campos da entidade que o ▾ oferece: com espécie conhecida e que não sejam texto — os grupos
/// PEQUENOS primeiro (uma forma longa só perde pontos no tecto), depois pelo endereço.
fn campos(w: &World, e: Option<Entity>) -> Vec<FieldKey> {
    let Some(lidos) = e.and_then(|e| Some((e, w.get_resource::<CanaisLidos>()?))) else {
        return Vec::new();
    };
    let (e, lidos) = lidos;
    let mut ks: Vec<FieldKey> = lidos
        .campos(e.to_bits())
        .map(|(k, _)| k)
        .filter(|k| k.info().is_some_and(|i| !i.texto()))
        .collect();
    let mut tamanho: BTreeMap<_, usize> = BTreeMap::new();
    for k in &ks {
        *tamanho.entry(k.grupo()).or_default() += 1;
    }
    ks.sort_by_key(|k| (tamanho[&k.grupo()], *k));
    ks.truncate(MAX_PROPRIEDADES - CANAIS.len());
    ks
}

/// ⭐ O nome de um campo como o dope-sheet o escreve: o grupo («Fill», com o número da entrada se o
/// grupo se parte por ela) e, se o grupo tem mais de um membro em `todos`, o membro («R», «Point X
/// 3»).
fn nome(k: FieldKey, todos: &[FieldKey]) -> String {
    let grupo = tr(k.info().map_or("panel.timeline.prop.field", |i| i.rotulo));
    let grupo = match k.grupo() {
        Some((_, _, i)) if i > 0 => format!("{grupo} {i}"),
        _ => grupo.to_owned(),
    };
    let membros: Vec<_> = todos.iter().filter(|o| o.mesmo_grupo(k)).collect();
    if membros.len() < 2 {
        return grupo;
    }
    let membro = tr(k.membro().unwrap_or("panel.timeline.prop.field"));
    let irmaos: Vec<_> = membros.iter().filter(|o| o.campo == k.campo).collect();
    let membro = if irmaos.len() < 2 {
        membro.to_owned()
    } else if irmaos.iter().any(|o| o.i != k.i) {
        format!("{membro} {}.{}", k.i + 1, k.j + 1)
    } else {
        format!("{membro} {}", k.j + 1)
    };
    format!("{grupo} · {membro}")
}

fn rotulo_de(p: P, todos: &[FieldKey]) -> String {
    match p {
        P::Canal(i) => tr(CANAIS.get(usize::from(i)).copied().unwrap_or(CANAIS[0])).to_owned(),
        P::Campo(k) => nome(chave(k), todos),
    }
}

/// As opções do ▾ sobre a entidade `e` (`None` = só a pose).
#[must_use]
pub fn opcoes(w: &World, e: Option<Entity>) -> Vec<OpcaoPropriedade> {
    let ks = campos(w, e);
    #[expect(clippy::cast_possible_truncation, reason = "cinco canais")]
    let pose = (0..CANAIS.len()).map(|i| P::Canal(i as u8));
    let campos = ks.iter().map(|k| P::Campo([k.familia, k.campo, k.i, k.j]));
    pose.chain(campos)
        .map(|prop| OpcaoPropriedade {
            prop,
            rotulo: rotulo_de(prop, &ks),
        })
        .collect()
}

fn ajuste(w: &World, e: Option<Entity>, campo: u16, valor: Prop) -> AjustePropriedade {
    let opcoes = opcoes(w, e);
    let valor = espelho(valor);
    let atual = opcoes
        .iter()
        .find(|o| o.prop == valor)
        .map_or_else(|| rotulo_de(valor, &campos(w, e)), |o| o.rotulo.clone());
    AjustePropriedade {
        campo,
        rotulo: if campo == DE {
            "panel.inspector.constraint.follow"
        } else {
            "panel.inspector.constraint.drive"
        },
        opcoes,
        valor,
        atual,
    }
}

/// Os ▾ do item: o Follow sobre o ALVO, o Drive sobre o DONO. Vazio para os outros tipos.
#[must_use]
pub fn propriedades(
    w: &World,
    k: &Kind,
    dono: Entity,
    alvo: Option<Entity>,
) -> Vec<AjustePropriedade> {
    let Kind::Valor(v) = k else {
        return Vec::new();
    };
    vec![
        ajuste(w, alvo, DE, v.fonte),
        ajuste(w, Some(dono), PARA, v.destino),
    ]
}

/// A ordem do cartão do Follow Value: Follow, a faixa de entrada, Drive, a de saída, Mix,
/// Extrapolate, Spring e (com mola) Duration e Bounce. Vazia para os outros tipos (a de sempre).
#[must_use]
pub fn ordem(k: &Kind) -> Vec<Peca> {
    let Kind::Valor(_) = k else {
        return Vec::new();
    };
    let mut o = vec![
        Peca::Propriedade(0),
        Peca::Numero(0),
        Peca::Numero(1),
        Peca::Propriedade(1),
        Peca::Numero(2),
        Peca::Numero(3),
        Peca::Escolha(0),
        Peca::Escolha(1),
        Peca::Escolha(2),
    ];
    if crate::campos::numeros(k).len() > 4 {
        o.extend([Peca::Numero(4), Peca::Numero(5)]);
    }
    o
}

/// Escreve o Follow (`DE`) ou o Drive (`PARA`); `true` se mudou.
pub fn escrever(k: &mut Kind, campo: u16, p: P) -> bool {
    let (Kind::Valor(v), Some(p)) = (&mut *k, prop(p)) else {
        return false;
    };
    let antes = *v;
    match campo {
        DE => v.fonte = p,
        PARA => v.destino = p,
        _ => return false,
    }
    *v != antes
}
