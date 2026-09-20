//! O corpus do ORÁCULO (Godot 4.7.2, `godot_export_probe.gd`) sobre a lei pura — uma pergunta,
//! um gate. Cada caso é a história que a sonda correu: grava com o script **v1**, o script muda
//! para **v2**, e lê-se o que o objecto usa.

use super::*;

fn num(name: &str, v: f64) -> PropDecl {
    PropDecl {
        name: name.into(),
        default: ScriptValue::Number(v),
        hint: PropHint::default(),
    }
}

fn own(pares: &[(&str, ScriptValue)]) -> BTreeMap<String, ScriptValue> {
    pares
        .iter()
        .map(|(k, v)| ((*k).to_owned(), v.clone()))
        .collect()
}

fn valor(r: &Resolution, name: &str) -> (ScriptValue, Origin) {
    let p = r
        .values
        .iter()
        .find(|p| p.name == name)
        .unwrap_or_else(|| panic!("`{name}` não foi resolvida: {r:?}"));
    (p.value.clone(), p.origin)
}

/// **Q1** — sem valor próprio, o default muda `4 → 7` ⇒ o objecto lê `7`.
#[test]
fn q1_sem_valor_proprio_segue_o_default_novo() {
    let v2 = [num("speed", 7.0)];
    let r = resolve(Some(&v2), &own(&[]));
    assert_eq!(
        valor(&r, "speed"),
        (ScriptValue::Number(7.0), Origin::Default)
    );
    assert!(r.orphans.is_empty() && r.kept.is_empty());
}

/// **Q2** — com `9` próprio, o default muda ⇒ o objecto guarda o `9`.
#[test]
fn q2_com_valor_proprio_guarda_o_dele() {
    let v2 = [num("speed", 7.0)];
    let r = resolve(Some(&v2), &own(&[("speed", ScriptValue::Number(9.0))]));
    assert_eq!(valor(&r, "speed"), (ScriptValue::Number(9.0), Origin::Own));
}

/// ⛔ **D1 contra Q3** — o alvo lê `7` (um `4` igual ao default não é gravado). Aqui o `4` foi
/// POSTO, e fica `4`.
#[test]
fn d1_um_valor_posto_igual_ao_default_continua_proprio_quando_o_default_muda() {
    let v1 = [num("speed", 4.0)];
    let mut proprios = own(&[]);
    put(&mut proprios, "speed", ScriptValue::Number(4.0));
    // No v1 ele já é PRÓPRIO, e não por coincidência com o default.
    assert_eq!(
        valor(&resolve(Some(&v1), &proprios), "speed"),
        (ScriptValue::Number(4.0), Origin::Own)
    );
    let v2 = [num("speed", 7.0)];
    assert_eq!(
        valor(&resolve(Some(&v2), &proprios), "speed"),
        (ScriptValue::Number(4.0), Origin::Own),
        "D1: a chave que o artista PÔS manda — o alvo lê 7 aqui (Q3), e é a perda que se recusa"
    );
    // E só o `forget` o larga.
    assert!(forget(&mut proprios, "speed"));
    assert_eq!(
        valor(&resolve(Some(&v2), &proprios), "speed"),
        (ScriptValue::Number(7.0), Origin::Default)
    );
    assert!(!forget(&mut proprios, "speed"), "nada a largar da 2.ª vez");
}

/// ⛔ **D2 contra Q4/Q4b** — a propriedade sai: o valor fica, NOMEADO; ela volta: o valor volta.
#[test]
fn d2_um_valor_cuja_propriedade_saiu_fica_nomeado_e_volta_com_ela() {
    let proprios = own(&[("speed", ScriptValue::Number(9.0))]);
    let sem = [num("outra", 1.0)];
    let r = resolve(Some(&sem), &proprios);
    assert_eq!(
        r.orphans,
        vec![Orphan {
            name: "speed".into(),
            value: ScriptValue::Number(9.0),
            why: OrphanWhy::Missing,
        }]
    );
    // O órfão NÃO chega ao objecto.
    assert!(r.values.iter().all(|p| p.name != "speed"));
    // Q4b no alvo: re-gravar e devolver a propriedade lê o DEFAULT (4). Aqui lê o 9.
    let volta = [num("speed", 4.0)];
    assert_eq!(
        valor(&resolve(Some(&volta), &proprios), "speed"),
        (ScriptValue::Number(9.0), Origin::Own)
    );
}

/// **Q5** — `9` gravado e a propriedade passa a texto ⇒ lê-se o default novo, e o `9` é nomeado.
#[test]
fn q5_um_numero_numa_propriedade_que_passou_a_texto_le_o_default_e_fica_nomeado() {
    let v2 = [PropDecl {
        name: "speed".into(),
        default: ScriptValue::Text("lento".into()),
        hint: PropHint::default(),
    }];
    let r = resolve(Some(&v2), &own(&[("speed", ScriptValue::Number(9.0))]));
    assert_eq!(
        valor(&r, "speed"),
        (ScriptValue::Text("lento".into()), Origin::Default)
    );
    assert_eq!(
        r.orphans[0].why,
        OrphanWhy::WrongKind {
            declared: ScriptValueKind::Text
        }
    );
    assert_eq!(r.orphans[0].value, ScriptValue::Number(9.0), "intacto");
}

/// ⛔ **D3 contra Q5b** — o alvo lê `"rapido"` como `0`. Aqui não se converte nada.
#[test]
fn d3_um_texto_numa_propriedade_que_passou_a_numero_nao_vira_zero() {
    let v2 = [num("speed", 4.0)];
    let r = resolve(
        Some(&v2),
        &own(&[("speed", ScriptValue::Text("rapido".into()))]),
    );
    assert_eq!(
        valor(&r, "speed"),
        (ScriptValue::Number(4.0), Origin::Default),
        "D3: o default, nunca um 0 inventado"
    );
    assert_eq!(r.orphans.len(), 1);
    assert_eq!(r.orphans[0].value, ScriptValue::Text("rapido".into()));
}

/// **Q6** — `15` gravado, a faixa estreita para `0..10` ⇒ lê-se `15`: a faixa é pista.
#[test]
fn q6_a_faixa_nao_prende_o_valor_gravado() {
    let v2 = [PropDecl {
        name: "speed".into(),
        default: ScriptValue::Number(4.0),
        hint: PropHint {
            min: Some(0.0),
            max: Some(10.0),
            step: None,
            options: Vec::new(),
        },
    }];
    let r = resolve(Some(&v2), &own(&[("speed", ScriptValue::Number(15.0))]));
    assert_eq!(valor(&r, "speed"), (ScriptValue::Number(15.0), Origin::Own));
    assert_eq!(r.values[0].hint.max, Some(10.0), "a pista chega ao painel");
}

/// **Q7** — a ordem é a da DECLARAÇÃO, nunca a do nome nem a do mapa gravado.
#[test]
fn q7_a_ordem_e_a_da_declaracao() {
    let decls = [num("zeta", 1.0), num("alpha", 2.0), num("label", 3.0)];
    // O mapa gravado é por nome (alpha < label < zeta) — e não pode vazar para a ordem.
    let r = resolve(
        Some(&decls),
        &own(&[
            ("alpha", ScriptValue::Number(9.0)),
            ("zeta", ScriptValue::Number(8.0)),
        ]),
    );
    let ordem: Vec<&str> = r.values.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(ordem, ["zeta", "alpha", "label"]);
}

/// **Q9** — duas instâncias do mesmo script são independentes.
#[test]
fn q9_duas_instancias_nao_se_tocam() {
    let v2 = [num("speed", 7.0)];
    let a = own(&[("speed", ScriptValue::Number(9.0))]);
    let b = own(&[]);
    assert_eq!(
        valor(&resolve(Some(&v2), &a), "speed").0,
        ScriptValue::Number(9.0)
    );
    assert_eq!(
        valor(&resolve(Some(&v2), &b), "speed").0,
        ScriptValue::Number(7.0)
    );
}

/// **Q10** e o cabeçalho do módulo — declarações DESCONHECIDAS guardam tudo, e nada é órfão.
#[test]
fn q10_com_o_script_desconhecido_os_valores_ficam_e_nada_e_orfao() {
    let proprios = own(&[
        ("speed", ScriptValue::Number(9.0)),
        ("label", ScriptValue::Text("x".into())),
    ]);
    let r = resolve(None, &proprios);
    assert!(r.values.is_empty());
    assert!(
        r.orphans.is_empty(),
        "⛔ tratar «não sei» como «nada» oferecia Remove ao lado de todo valor"
    );
    assert_eq!(r.kept.len(), 2);
    // E «sei que não declara nada» é OUTRA resposta: ali são órfãos.
    assert_eq!(resolve(Some(&[]), &proprios).orphans.len(), 2);
}

#[test]
fn as_declaracoes_mal_formadas_sao_recusadas_com_o_nome() {
    let ok = num("speed", 1.0);
    assert_eq!(check_decl(&[], &ok), Ok(()));
    for mau in ["", "2x", "a-b", "é", "a b"] {
        assert_eq!(
            check_decl(&[], &num(mau, 1.0)),
            Err(DeclError::BadName(mau.into())),
            "`{mau}`"
        );
    }
    assert_eq!(check_decl(&[], &num("_ok9", 1.0)), Ok(()));
    assert_eq!(
        check_decl(&[], &num("id", 1.0)),
        Err(DeclError::Reserved("id".into()))
    );
    assert_eq!(
        check_decl(std::slice::from_ref(&ok), &ok),
        Err(DeclError::Twice("speed".into()))
    );
    assert_eq!(
        check_decl(&[], &num("x", f64::NAN)),
        Err(DeclError::BadDefault("x".into()))
    );
    let com = |default: ScriptValue, hint: PropHint| PropDecl {
        name: "x".into(),
        default,
        hint,
    };
    let passo = |s| PropHint {
        step: Some(s),
        ..PropHint::default()
    };
    assert_eq!(
        check_decl(&[], &com(ScriptValue::Bool(true), passo(1.0))),
        Err(DeclError::BadHint("x".into())),
        "pista num sim/não"
    );
    assert_eq!(
        check_decl(&[], &com(ScriptValue::Number(1.0), passo(0.0))),
        Err(DeclError::BadHint("x".into()))
    );
    assert_eq!(
        check_decl(
            &[],
            &com(
                ScriptValue::Number(1.0),
                PropHint {
                    min: Some(2.0),
                    max: Some(1.0),
                    step: None,
                    options: Vec::new(),
                }
            )
        ),
        Err(DeclError::BadHint("x".into()))
    );
    // min == max é uma faixa de um ponto, e é legal.
    assert_eq!(
        check_decl(
            &[],
            &com(
                ScriptValue::Number(1.0),
                PropHint {
                    min: Some(1.0),
                    max: Some(1.0),
                    step: None,
                    options: Vec::new(),
                }
            )
        ),
        Ok(())
    );
}

#[test]
fn os_valores_viajam_no_fio_e_a_ordem_das_variantes_e_o_contrato() {
    // ⚠️ postcard é posicional: a TAG é o índice da variante. Trocar a ordem leria um Bool como
    // Number, em silêncio.
    for (v, tag) in [
        (ScriptValue::Number(1.5), 0u8),
        (ScriptValue::Bool(true), 1),
        (ScriptValue::Text("a".into()), 2),
    ] {
        let bytes = postcard::to_allocvec(&v).expect("serializa");
        assert_eq!(bytes[0], tag, "{v:?}");
        let back: ScriptValue = postcard::from_bytes(&bytes).expect("volta");
        assert_eq!(back, v);
    }
}

#[test]
fn um_script_nao_oferece_mais_propriedades_do_que_o_painel_pinta() {
    let cheias: Vec<PropDecl> = (0..PROPS_MAX).map(|i| num(&format!("p{i}"), 0.0)).collect();
    assert_eq!(
        check_decl(&cheias[..PROPS_MAX - 1], &cheias[PROPS_MAX - 1]),
        Ok(()),
        "a última que cabe entra"
    );
    assert_eq!(
        check_decl(&cheias, &num("mais", 0.0)),
        Err(DeclError::TooMany("mais".into()))
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// ⭐⭐⭐ O ENUM — um texto cuja declaração traz uma LISTA.
// ─────────────────────────────────────────────────────────────────────────────

/// Uma declaração de enum com as opções `opts` e o default `padrao`.
fn enum_decl(padrao: &str, opts: &[&str]) -> PropDecl {
    PropDecl {
        name: "mode".into(),
        default: ScriptValue::Text(padrao.into()),
        hint: PropHint {
            options: opts.iter().map(|o| (*o).to_owned()).collect(),
            ..PropHint::default()
        },
    }
}

fn texto(v: &str) -> ScriptValue {
    ScriptValue::Text(v.into())
}

/// ⭐⭐⭐ **O DEFEITO QUE A WAVE MATA: um valor fora da lista NÃO CHEGA ao script.**
///
/// ⚠️⚠️ **As três metades, e cada uma sozinha mente:**
///
/// 1. o objecto lê o **DEFAULT** (sem isto, `"fst"` chega ao script, que compara com `"fast"` e
///    cai no ramo errado **em silêncio** — o que ele fazia até hoje, medido pela sonda do §5.0);
/// 2. o valor é **NOMEADO** como órfão (sem isto ele desaparece da vista do artista, que fica sem
///    saber porque a escolha dele não valeu — a perda que a divergência D2 existe para impedir);
/// 3. o valor gravado fica **INTACTO** (encostá-lo à opção mais parecida é o *«aceita e mente»*
///    que esta casa já pagou três vezes).
///
/// **Mutações que devem sangrar:** tirar o `aceita` da 1.ª metade do `resolve` · tirar o braço
/// `NotAnOption` da 2.ª · fazer o `aceita` devolver `true` sempre.
#[test]
fn um_valor_fora_da_lista_nao_chega_ao_script_e_e_nomeado() {
    let decls = [enum_decl("fast", &["slow", "fast"])];
    let own: BTreeMap<String, ScriptValue> = [("mode".to_owned(), texto("fst"))].into();
    let r = resolve(Some(&decls), &own);

    assert_eq!(
        r.values[0].value,
        texto("fast"),
        "o objecto tem de ler o DEFAULT — senao `fst` chega ao script e o ramo errado corre calado"
    );
    assert_eq!(r.values[0].origin, Origin::Default);
    assert_eq!(
        r.orphans.len(),
        1,
        "e o valor tem de ser NOMEADO — um valor que some sem explicacao e' a perda do D2"
    );
    assert_eq!(r.orphans[0].why, OrphanWhy::NotAnOption);
    assert_eq!(
        r.orphans[0].value,
        texto("fst"),
        "e fica INTACTO: encosta'-lo a' opcao mais parecida e' o «aceita e mente»"
    );
}

/// ⭐⭐ **O CONTROLO: um valor QUE ESTÁ na lista aplica-se, e é próprio.**
///
/// ⚠️ Sem esta metade, um `aceita` que devolvesse `false` sempre passaria no gate de cima — *uma
/// cerca que recusa tudo satisfaz toda régua que só mede recusas*.
#[test]
fn um_valor_da_lista_aplica_se_e_e_proprio() {
    let decls = [enum_decl("fast", &["slow", "fast"])];
    let own: BTreeMap<String, ScriptValue> = [("mode".to_owned(), texto("slow"))].into();
    let r = resolve(Some(&decls), &own);
    assert_eq!(r.values[0].value, texto("slow"));
    assert_eq!(r.values[0].origin, Origin::Own);
    assert!(r.orphans.is_empty(), "um valor da lista nao e' orfao");
}

/// ⭐⭐ **E SEM lista o texto continua LIVRE — ao bit.**
///
/// ⚠️ Esta é a metade que prova que a wave não mexeu no caminho de omissão: toda propriedade de
/// texto que já existia continua a aceitar o que o artista escrever.
#[test]
fn sem_lista_o_texto_continua_livre() {
    let decls = [PropDecl {
        name: "label".into(),
        default: texto("oi"),
        hint: PropHint::default(),
    }];
    let own: BTreeMap<String, ScriptValue> = [("label".to_owned(), texto("qualquer coisa"))].into();
    let r = resolve(Some(&decls), &own);
    assert_eq!(r.values[0].value, texto("qualquer coisa"));
    assert_eq!(r.values[0].origin, Origin::Own);
    assert!(r.orphans.is_empty());
}

/// ⛔⛔ **O TIPO vem ANTES da lista** — e a ordem é a afirmação.
///
/// Um NÚMERO gravado numa propriedade que passou a enum é `WrongKind`, nunca `NotAnOption`:
/// *«o script quer um texto»* diz ao artista a coisa útil; *«não é uma das opções»* mandá-lo-ia
/// procurar o valor numa lista onde ele nunca poderia estar.
#[test]
fn um_numero_numa_propriedade_de_enum_e_do_tipo_errado_e_nao_fora_da_lista() {
    let decls = [enum_decl("fast", &["slow", "fast"])];
    let own: BTreeMap<String, ScriptValue> = [("mode".to_owned(), ScriptValue::Number(3.0))].into();
    let r = resolve(Some(&decls), &own);
    assert_eq!(
        r.orphans[0].why,
        OrphanWhy::WrongKind {
            declared: ScriptValueKind::Text
        },
        "o tipo e' a queixa mais util das duas, e por isso e' a primeira"
    );
}

/// ⭐⭐⭐ **AS QUATRO RECUSAS de uma lista malformada — e ela falha FECHADA.**
///
/// ⚠️ A quarta é a mais importante: sem ela um objecto que não pôs nada leria um valor que a
/// própria declaração recusa, e o `resolve` cairia num default que ele próprio nomearia órfão se
/// alguém o tivesse posto à mão.
#[test]
fn uma_lista_malformada_recusa_a_declaracao_inteira() {
    let casos: [(&str, PropDecl); 5] = [
        (
            "opcoes num default que nao e' TEXTO",
            PropDecl {
                name: "n".into(),
                default: ScriptValue::Number(1.0),
                hint: PropHint {
                    options: vec!["a".into()],
                    ..PropHint::default()
                },
            },
        ),
        ("uma opcao VAZIA", enum_decl("a", &["a", ""])),
        ("opcoes REPETIDAS", enum_decl("a", &["a", "a"])),
        ("o default FORA da lista", enum_decl("z", &["a", "b"])),
        ("mais opcoes do que o painel pinta", {
            let muitas: Vec<String> = (0..=OPCOES_MAX).map(|i| format!("o{i}")).collect();
            PropDecl {
                name: "n".into(),
                default: texto("o0"),
                hint: PropHint {
                    options: muitas,
                    ..PropHint::default()
                },
            }
        }),
    ];
    for (porque, d) in casos {
        assert!(
            check_decl(&[], &d).is_err(),
            "uma lista com «{porque}» tem de RECUSAR a declaracao — um parser que falha ABERTO \
             manda a lista malformada DESENHAR (a lei que o L-System pagou)"
        );
    }
    // ⭐ E o CONTROLO: uma lista boa passa.
    assert!(check_decl(&[], &enum_decl("a", &["a", "b"])).is_ok());
}
