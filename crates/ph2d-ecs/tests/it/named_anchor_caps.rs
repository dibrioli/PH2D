//! **Os caps CONGELADOS do `NamedAnchor`** — o arch-gate que o
//! [ADR-0072](../../../docs/architecture/decisions/0072-named-anchor-unification.md) §2.7 declara.
//!
//! ⚠️ **Ele estava declarado num ADR `Accepted` e nunca foi escrito** (auditoria de 2026-08-22).
//! O ADR chega a dizer onde ele mora — «arch-gate `named_anchor_caps` em
//! `crates/ph2d-render/tests/`» — e `git grep named_anchor_caps` dava **zero**. É a quarta vez
//! que esta linha encontra um gate declarado por escrito e ausente do disco: os três da §5 do
//! Inspector (`inspector_section_count_canonical` · `sprite_inspector_i18n_keys_present` ·
//! `bulk_edit_confirmation_required_fields`) tinham a mesma forma.
//!
//! ⚠️ **E mora no `ph2d-ecs`, não no `ph2d-render` que o ADR indicava.** O `ph2d-render` não
//! depende do tipo; escrever o gate onde o ADR dizia obrigaria a inventar a dependência, que é
//! pior do que corrigir a morada. *Um gate mora onde vê o que mede.*
//!
//! # A divergência DELIBERADA que este ficheiro regista
//!
//! O ADR §2.7 escreve «`NamedAnchor.name` length ≤ 64 **chars**»; o código conta **bytes**
//! ([`ANCHOR_NAME_MAX_BYTES`]). A escolha é do código e está justificada onde ela vive: um emoji
//! são 4 bytes, e contar caracteres deixaria o mesmo nome caber numa máquina e não noutra
//! conforme a normalização — enquanto o postcard prefixa em bytes. *Contar a mesma coisa que o
//! formato conta* é o que mantém o hash de cozimento estável.

use ph2d_ecs::{
    ANCHOR_NAME_MAX_BYTES, ANCHORS_MAX, AnchorData, AnchorNameError, DICT_MAX_DEPTH, DICT_MAX_KEYS,
    NamedAnchor, NamedAnchorList,
};

/// Os quatro números do §2.7. ⚠️ **Baixá-los é uma quebra de contrato** (`Bump → ADR-0072-amendment`),
/// e subi-los também: o `INSP_ANCHOR_ROW` do painel tem exatamente `ANCHORS_MAX` entradas, e há
/// gate na shell a prender os dois.
#[test]
fn the_frozen_caps_are_the_numbers_the_adr_froze() {
    assert_eq!(ANCHOR_NAME_MAX_BYTES, 64, "cap de nome");
    assert_eq!(ANCHORS_MAX, 64, "cap de ancoras por sprite");
    assert_eq!(DICT_MAX_DEPTH, 4, "profundidade do Dict");
    assert_eq!(DICT_MAX_KEYS, 32, "chaves por nivel do Dict");
}

/// ⚠️ **A `AnchorData` está CONGELADA em SEIS variantes** (§2.7, Lens E E23) — `None` · `Str` ·
/// `Int` · `Float` · `Color` · `Dict`. Uma sétima exige emenda ao ADR.
///
/// O `match` exaustivo é o que torna isto um gate e não um comentário: acrescentar uma variante
/// **não compila** este ficheiro até alguém vir aqui e contar de novo.
#[test]
fn anchor_data_still_has_the_six_frozen_variants() {
    let all = [
        AnchorData::None,
        AnchorData::Str(String::new()),
        AnchorData::Int(0),
        AnchorData::Float(0.0),
        AnchorData::Color([0.0; 4]),
        AnchorData::Dict(Box::default()),
    ];
    assert_eq!(all.len(), 6, "a AnchorData deixou de ter seis variantes");
    for v in &all {
        // ⚠️ Sem braço-curinga: uma variante nova quebra a COMPILAÇÃO deste teste.
        let named = match v {
            AnchorData::None => "None",
            AnchorData::Str(_) => "Str",
            AnchorData::Int(_) => "Int",
            AnchorData::Float(_) => "Float",
            AnchorData::Color(_) => "Color",
            AnchorData::Dict(_) => "Dict",
        };
        assert!(!named.is_empty());
    }
}

/// **O cap de contagem é IMPOSTO, não declarado** — e falha com a sua própria razão.
///
/// ⚠️ Até 2026-08-22 a lista cheia devolvia `TooLong`, o erro do NOME. As duas mensagens liam
/// igual («over the limit of 64») e só estavam certas por coincidência de os dois tetos serem 64.
#[test]
fn a_full_list_refuses_with_its_own_reason_not_the_names() {
    let mut l = NamedAnchorList::new();
    for n in 0..ANCHORS_MAX {
        l.insert(NamedAnchor::socket(format!("a{n}")))
            .expect("cabe ate' ao cap");
    }
    assert_eq!(l.len(), ANCHORS_MAX);
    assert_eq!(
        l.insert(NamedAnchor::socket("uma_a_mais")),
        Err(AnchorNameError::ListFull),
        "a lista cheia tem de dizer que esta' CHEIA, nao que o nome e' comprido"
    );
    // E um nome comprido continua a ser um erro de NOME, mesmo numa lista com espaço.
    let mut room = NamedAnchorList::new();
    let long = "n".repeat(ANCHOR_NAME_MAX_BYTES + 1);
    assert_eq!(
        room.insert(NamedAnchor::socket(long)),
        Err(AnchorNameError::TooLong)
    );
}

/// O `Dict` respeita profundidade e chaves — e o gate prova **os dois lados**: dentro do cap
/// passa, fora não. Um teste que só provasse a recusa passaria com tudo recusado.
#[test]
fn the_dict_caps_hold_on_both_sides() {
    // Dentro: profundidade 1, uma chave.
    let mut shallow = ph2d_ecs::SortedSmallVec::default();
    shallow.insert_sorted("k".into(), AnchorData::Int(1));
    assert!(AnchorData::Dict(Box::new(shallow)).within_caps());

    // Fora, por PROFUNDIDADE: aninha um a mais que o cap.
    let mut deep = AnchorData::Int(0);
    for _ in 0..=DICT_MAX_DEPTH {
        let mut d = ph2d_ecs::SortedSmallVec::default();
        d.insert_sorted("k".into(), deep);
        deep = AnchorData::Dict(Box::new(d));
    }
    assert!(!deep.within_caps(), "a profundidade nao foi imposta");

    // Fora, por CHAVES: uma a mais no mesmo nível.
    let mut wide = ph2d_ecs::SortedSmallVec::default();
    for n in 0..=DICT_MAX_KEYS {
        wide.insert_sorted(format!("k{n}"), AnchorData::None);
    }
    assert!(
        !AnchorData::Dict(Box::new(wide)).within_caps(),
        "o numero de chaves nao foi imposto"
    );
}
