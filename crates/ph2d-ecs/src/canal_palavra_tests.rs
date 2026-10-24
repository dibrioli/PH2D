use super::*;
use crate::canal_componente::{chaves_do_rotulo, todas};

fn canal(rotulo: &str) -> FieldKey {
    chaves_do_rotulo(rotulo)[0]
}

fn chave(s: &'static str) -> Option<Palavra> {
    Some(Palavra::Chave(Cow::Borrowed(s)))
}

fn do_artista(s: &str) -> Option<Palavra> {
    Some(Palavra::DoArtista(s.to_owned()))
}

/// ⭐ **A escolha do catálogo diz a chave da OPÇÃO, não o índice** — «Score · Source 3.00» era a
/// variante `TagCount` (smoke do dono, `=32`).
///
/// Mutação que deve sangrar: `Especie::Catalogo => None` (a coluna volta ao número).
#[test]
fn a_escolha_do_catalogo_diz_a_chave_da_opcao() {
    let w = World::new();
    let fonte = canal("component.field.label_fields.1");
    assert_eq!(
        palavra(&w, 0, fonte, 3.0),
        chave("panel.inspector.hud.source_tag")
    );
    assert_eq!(
        palavra(&w, 0, fonte, 9.0),
        None,
        "um índice que nenhuma opção tem"
    );
}

/// ⭐ A camada e o script dizem o NOME do artista; o script `0` é «nenhum».
#[test]
fn a_camada_e_o_script_dizem_o_nome_do_artista() {
    let mut w = World::new();
    let camada = canal("component.field.sorting_layer.1");
    assert_eq!(
        palavra(&w, 0, camada, 3.0),
        do_artista("Foreground"),
        "as de fábrica"
    );
    let script = canal("component.field.script_fields.1");
    assert_eq!(palavra(&w, 0, script, 0.0), chave(NENHUM));
    assert_eq!(
        palavra(&w, 0, script, 2.0),
        None,
        "sem a biblioteca publicada"
    );
    w.insert_resource(NomesDosScripts(BTreeMap::from([(
        2,
        "bob.luau".to_owned(),
    )])));
    assert_eq!(palavra(&w, 0, script, 2.0), do_artista("bob.luau"));
    assert_eq!(
        palavra(&w, 0, script, 5.0),
        None,
        "um id que a biblioteca não tem"
    );
}

/// Ligado/desligado, o texto da tabela, e o NÚMERO que fica número.
#[test]
fn ligado_texto_e_numero() {
    use crate::canal::familia;
    use crate::canal_script::{Tipo, chave as prop};
    let mut w = World::new();
    let mut t = TextosDosCanais::default();
    let nome = t.indice(Some("pts"));
    w.insert_resource(t);
    let mut textos = TextosDosCanais::default();
    let b = prop("active", Tipo::Bool, 0, &mut textos);
    assert_eq!(b.map(|k| k.familia), Some(familia::SCRIPT));
    let b = b.unwrap_or(FieldKey::new(0, 0, 0, 0));
    assert_eq!(palavra(&w, 0, b, 1.0), chave(LIGADO));
    assert_eq!(palavra(&w, 0, b, 0.0), chave(DESLIGADO));
    let fonte_nome = canal("component.field.label_fields.2");
    assert_eq!(palavra(&w, 0, fonte_nome, nome), do_artista("pts"));
    let inicio = canal("component.field.counter_fields.2");
    assert_eq!(palavra(&w, 0, inicio, 3.0), None, "um Int é a resposta");
}

/// ⭐⭐ **Todo canal discreto do catálogo declarou a espécie certa** — uma escolha de `options!`
/// (também `Int`, o `TopDownPlayer`) é `Catalogo` e cada opção tem chave; nenhum discreto de escolha
/// caiu em `Numero`.
#[test]
fn toda_escolha_do_catalogo_tem_chave_por_opcao() {
    let mut n = 0;
    for k in todas() {
        let Some((_, f, _)) = crate::canal_componente::resolver(k) else {
            continue;
        };
        if let ph2d_component_desc::View::Options(o) = f.view {
            assert_eq!(
                k.info().map(|i| i.especie),
                Some(Especie::Catalogo),
                "{}",
                f.label_key
            );
            for i in 0..o.len() as u64 {
                assert!(crate::canal_componente::chave_da_opcao(k, i).is_some());
                n += 1;
            }
        }
    }
    assert!(n >= 133, "o censo do F27 F contou 133 opções; agora {n}");
}

/// Um código sem variante (`""` na lista: a pele `0` do widget) fica NÚMERO, nunca palavra vazia.
///
/// Mutação que deve sangrar: `de_lista` devolver `Some` para `""`.
#[test]
fn um_codigo_sem_variante_fica_numero() {
    use crate::canal::familia;
    let w = World::new();
    let pele = FieldKey::new(familia::VETOR, crate::canal_vetor::WIDGET_PELE, 0, 0);
    assert_eq!(palavra(&w, 0, pele, 0.0), None);
    assert_eq!(
        palavra(&w, 0, pele, 1.0),
        chave("panel.vector.widget.kind.button")
    );
}
