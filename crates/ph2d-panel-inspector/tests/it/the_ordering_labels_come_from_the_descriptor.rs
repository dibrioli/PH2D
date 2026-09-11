//! **A §7 Ordering é a seção PILOTO da F0** — os rótulos das linhas dela vêm do descritor
//! ([ADR-0166](../../../docs/architecture/decisions/0166-the-inspector-shows-what-the-object-has-and-components-attach-through-one-palette-filtered-by-object-type.md),
//! [plano §F0](../../../docs/Components/05_plano_de_implementacao.md)), e este arquivo prova
//! as duas metades disso.
//!
//! # Por que a ligação valeu a pena, com a evidência
//!
//! Antes desta wave a mesma string vivia em **dois** sítios: o literal no pintor
//! (`sections/ordering.rs`) e o `FieldDesc::name` no catálogo. Ao ligá-los, os dois
//! discordavam **em duas das dez linhas**:
//!
//! | linha | o pintor dizia | o descritor dizia |
//! |---|---|---|
//! | `SortingGroup.field(1)` | `Sort At Root` | `Sort at Root` |
//! | `YSort.field(1)` | `Y-Sort` | `Enabled` |
//!
//! *Duas derivações da mesma lista é o defeito; uma é a cura* — é a lei que o
//! `widget/command_palette.rs` já tinha pago no seu próprio doc-comment. Quem manda é o
//! **produto**: o descritor foi corrigido para o que o Inspector pinta, não o contrário.
//!
//! ⚠️ **Este gate não substitui o seam** (que CLICA e mede o efeito). Ele responde a uma
//! pergunta anterior e mais barata: *a string que o artista lê tem um dono só?*

use ph2d_component_desc::desc_for;

/// **Os rótulos, ao byte, como a §7 os pinta hoje.**
///
/// É o gate do *"visualmente idêntica à atual"* que a F0 prometeu: se alguém mexer num nome
/// no catálogo, esta lista fica vermelha e nomeia qual. ⚠️ Ele não é decoração — o descritor
/// agora é a ÚNICA fonte, então uma edição descuidada lá **renomeia a UI** sem passar por
/// nenhum arquivo de painel.
#[test]
fn the_ordering_rows_read_exactly_what_shipped() {
    // (nome canónico, field_id ou None p/ marcador, o rótulo que a §7 pinta)
    let expected: &[(&str, Option<u16>, &str)] = &[
        ("ph2d::ecs::ZIndexOverride", Some(1), "Z Index"),
        ("ph2d::ecs::ZAsRelative", Some(1), "Z as Relative"),
        ("ph2d::ecs::ShowBehindParent", None, "Show Behind Parent"),
        ("ph2d::ecs::OrderInLayer", Some(1), "Order in Layer"),
        ("ph2d::ecs::YSort", Some(1), "Y-Sort"),
        ("ph2d::ecs::SortingGroup", None, "Sorting Group"),
        ("ph2d::ecs::SortingGroup", Some(1), "Sort At Root"),
        ("ph2d::ecs::TopLevel", None, "Top Level"),
    ];
    for (canonical, field_id, want) in expected {
        let d = desc_for(canonical).unwrap_or_else(|| panic!("'{canonical}' sem descritor"));
        let got = match field_id {
            Some(id) => {
                d.field(*id)
                    .unwrap_or_else(|| panic!("'{canonical}' sem campo {id}"))
                    .name
            }
            None => d.display_name,
        };
        assert_eq!(
            got, *want,
            "'{canonical}' mudou o rotulo que a secao 7 pinta: '{got}' (era '{want}'). \
             O descritor e' a fonte UNICA — mexer nele renomeia a UI.",
        );
    }
}

/// **A outra metade: o pintor não pode ter voltado a escrever os rótulos à mão.**
///
/// Gate **estrutural, sobre o fonte** (o idioma dos `architecture_*` da casa): se um literal
/// de rótulo reaparecer no sítio de emissão de linha, houve uma segunda fonte outra vez — e
/// ela ficaria verde em todo teste de comportamento, porque as duas concordariam **no dia em
/// que foram escritas**. É precisamente a divergência da tabela acima, que levou meses a
/// aparecer porque ninguém as comparou.
#[test]
fn the_painter_does_not_hardcode_the_row_labels() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/sections/ordering.rs"
    ))
    .expect("ordering.rs legivel");

    // ⚠️ **Uma INVOCAÇÃO, não uma linha** — e esta distinção custou uma prova de mutação.
    // A 1.ª versão deste gate perguntava se a *linha* `yy = cb!(…)` continha aspas, e ficou
    // VERDE sobre um rótulo escrito à mão: depois de derivar, as invocações passaram a ser
    // multi-linha, então o literal cai na linha SEGUINTE, que o gate nunca olhava. *Um gate
    // de fonte escrito contra a forma que eu esperava, e não contra a forma que o código
    // tem.* Por isso ele agora acumula o bloco inteiro, do `cb!(` até ao `);`.
    let mut block = String::new();
    let mut open = false;
    let mut start_line = 0usize;
    for (n, line) in src.lines().enumerate() {
        let t = line.trim_start();
        if !open && (t.starts_with("yy = cb!(") || t.starts_with("yy = ni!(")) {
            open = true;
            start_line = n + 1;
            block.clear();
        }
        if !open {
            continue;
        }
        block.push_str(t);
        if !t.ends_with(");") {
            continue;
        }
        open = false;
        // Um bloco derivado NOMEIA a porta. Um bloco à mão traz o rótulo entre aspas — e
        // note que `contains('"')` sozinho não serve, porque a porta derivada também leva
        // aspas (o nome canónico é uma string). A pergunta certa é pela PORTA.
        assert!(
            block.contains("field_label(") || block.contains("marker_label("),
            "ordering.rs:{start_line}: linha emitida sem passar pelo descritor — o rotulo \
             vem de `field_label` / `marker_label`, senao voltam a existir duas fontes para \
             a mesma string (foi assim que 'Sort At Root' e 'Sort at Root' coexistiram):\n  \
             {block}",
        );
    }
    assert!(
        !open,
        "ordering.rs: uma invocacao de `cb!`/`ni!` ficou sem fecho `);` — o gate nao a leu \
         inteira, e um gate que nao le nao mede",
    );
}
