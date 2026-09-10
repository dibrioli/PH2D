//! **O TECLADO DA CÂMERA, medido — e os dois censos de ALCANÇABILIDADE.**
//!
//! Irmão do [`super`] pelo tecto de LOC, e o corte é por **responsabilidade**:
//! estes dois gates não são sobre viewports, são sobre **o teclado** — eles
//! foram parar ao ficheiro da divisão por acidente de escrita, e é aqui que o
//! sujeito deles mora.
//!
//! ⚠️ **Os dois são censos de FONTE**, e a razão é a mesma: o `App::sculpt3d_key`
//! precisa de uma surface de janela real, então nenhum teste deste repositório
//! consegue apertar uma tecla nele. O que se pode afirmar sem ela é a **forma**
//! do despacho — e as duas formas que matam um atalho em silêncio são duas
//! claims da mesma tecla e um `return` a montante.
//!
//! ```text
//! cargo test -p ph2d-host-desktop --bins sculpt3d_keys_view
//! ```

/// ⭐⭐⭐ **DUAS TECLAS QUE PODEM CASAR O MESMO EVENTO: A SEGUNDA É MORTA.**
///
/// ⛔⛔ **Este gate nasceu de um defeito meu, escrito e apanhado no mesmo dia
/// (2026-09-08).** A divisão dos viewports foi ligada a `K::Backquote` **sem
/// modificador** — e a crase sozinha já tinha dono neste mesmo ficheiro (ela
/// abre o painel, e o roteiro da cena `=37` manda o artista usá-la). O braço do
/// painel corre antes e devolve `true`: a tecla nova **compilava, não dava
/// warning nenhum, e nunca corria**.
///
/// ⚠️ **Nenhuma sonda deste repo vê isto.** O censo de ids de painel mede
/// registo; os `seam_*` medem que o clique chega à ferramenta. *Um `match` de
/// teclado com dois braços que casam o mesmo evento é a espécie de controlo
/// morto que só a ORDEM de leitura revela.*
///
/// # ⛔⛔ E a PRIMEIRA redacção deste gate era fraca — a mutação SOBREVIVEU
///
/// Ela comparava as guardas como **texto**: `!ctrl && !shift` e `` (vazia,
/// sempre verdadeira) são strings diferentes, então tirar o `&& ctrl` do braço
/// novo — que é exactamente o defeito original — passava. *Uma régua que
/// pergunta «as guardas são iguais?» responde a outra pergunta que não a
/// «podem as duas ser verdadeiras ao mesmo tempo?».*
///
/// ⇒ cada guarda é **avaliada** sobre as quatro combinações de `(ctrl, shift)`,
/// e o gate exige que os conjuntos sejam **disjuntos**. Um símbolo que o censo
/// não saiba avaliar entra em pânico — *um censo que não entende o que lê tem
/// de dizê-lo, e não devolver «nada a acusar»*.
///
/// # ⛔ O PONTO CEGO, e ele é NOMEADO em vez de tapado
///
/// Uma guarda pode ser do **bloco que envolve** o braço, e não da linha dele: o
/// `K::KeyJ` aparece duas vezes com a mesma condição escrita, e não colidem
/// porque uma delas vive dentro de um `if shift { … }`. A primeira redacção
/// acusou-a — *um censo textual que não conhece o contexto acusa o vivo, e a
/// cura que ele manda aplicar é a errada*.
///
/// ⇒ ele compara **só arms no mesmo NÍVEL DE INDENTAÇÃO**, que é o proxy honesto
/// de *«no mesmo bloco»*. Ele deixa passar uma colisão entre blocos diferentes
/// com a mesma indentação, e isso está declarado aqui em vez de ser um silêncio.
#[test]
fn nenhuma_tecla_e_reivindicada_duas_vezes_com_a_mesma_guarda() {
    // ⚠️ `BTreeMap` e não `HashMap`: é a espinha do determinismo desta casa
    // (HR-5 + ADR-0022), e aqui ela dá de graça uma listagem ORDENADA quando o
    // censo imprime o que achou.
    use std::collections::BTreeMap;

    /// **O que este braço reivindica**: as teclas e a máscara de `(ctrl, shift)`.
    ///
    /// ⚠️ **Ele lê a condição INTEIRA, e não «a primeira tecla e o resto»** — um
    /// braço pode nomear várias teclas (`code == K::Comma || code == K::Period`),
    /// e as guardas de modificador são tokens soltos ligados por `&&`.
    /// ⛔ Qualquer palavra que não seja uma dessas entra em pânico: *um censo que
    /// não entende o que lê tem de dizê-lo, e não devolver «nada a acusar»*.
    fn reivindica(cond: &str) -> (Vec<String>, u8) {
        let mut teclas = Vec::new();
        let mut guardas: Vec<&str> = Vec::new();
        let limpo = cond
            .replace("&&", " ")
            .replace("||", " ")
            .replace(['(', ')'], " ");
        let mut it = limpo.split_whitespace().peekable();
        while let Some(t) = it.next() {
            match t {
                "code" => {
                    assert_eq!(it.next(), Some("=="), "forma inesperada depois de `code`");
                    let k = it.next().expect("falta a tecla depois de `==`");
                    teclas.push(
                        k.strip_prefix("K::")
                            .unwrap_or_else(|| panic!("`{k}` nao e' uma tecla `K::…`"))
                            .to_string(),
                    );
                }
                "ctrl" | "!ctrl" | "shift" | "!shift" => guardas.push(t),
                outro => panic!(
                    "o censo nao sabe avaliar `{outro}` na condicao `{cond}` -- ele nao pode \
                     devolver «nada a acusar» sobre o que nao entende"
                ),
            }
        }
        let mut mask = 0u8;
        for bit in 0..4u8 {
            let (ctrl, shift) = (bit & 1 != 0, bit & 2 != 0);
            let ok = guardas.iter().all(|t| match *t {
                "ctrl" => ctrl,
                "!ctrl" => !ctrl,
                "shift" => shift,
                _ => !shift,
            });
            if ok {
                mask |= 1 << bit;
            }
        }
        (teclas, mask)
    }

    let fonte = include_str!("sculpt3d_keys.rs");
    let mut vistos: BTreeMap<String, Vec<(String, u8)>> = BTreeMap::new();
    for linha in fonte.lines() {
        let l = linha.trim();
        // ⚠️ Só condições, nunca prosa: um comentário que cite `code == K::X`
        // não reivindica tecla nenhuma, e contar prosa é o defeito que todo
        // censo textual paga uma vez.
        if l.starts_with("//") || !l.starts_with("if code == K::") {
            continue;
        }
        let indent = linha.len() - linha.trim_start().len();
        let cond = l
            .trim_start_matches("if ")
            .trim_end()
            .trim_end_matches('{')
            .trim();
        let (teclas, mask) = reivindica(cond);
        for k in teclas {
            // ⚠️ **A indentação faz parte da chave** — ver o ponto cego no doc.
            vistos
                .entry(format!("{k}@{indent}"))
                .or_default()
                .push((cond.to_string(), mask));
        }
    }
    assert!(
        !vistos.is_empty(),
        "o censo nao achou braco de tecla nenhum -- ele ficou cego (o `sculpt3d_keys` mudou de \
         forma, e um censo cego le^-se como aprovado)"
    );
    let mut colisoes = Vec::new();
    for (tecla, arms) in &vistos {
        for i in 0..arms.len() {
            for j in (i + 1)..arms.len() {
                if arms[i].1 & arms[j].1 != 0 {
                    colisoes.push(format!(
                        "K::{tecla}: `{}` e `{}` aceitam o MESMO evento",
                        arms[i].0, arms[j].0
                    ));
                }
            }
        }
    }
    let mut multi: Vec<_> = vistos
        .iter()
        .filter(|(_, a)| a.len() > 1)
        .map(|(k, a)| (k.clone(), a.clone()))
        .collect();
    multi.sort();
    println!("teclas com mais de um braco: {multi:?}");
    assert!(
        colisoes.is_empty(),
        "{colisoes:?} -- o SEGUNDO braco nunca corre: o primeiro casa o mesmo evento e \
         devolve `true`. Uma tecla que compila e nunca corre nao da' warning nenhum."
    );
}

/// ⭐⭐⭐ **UM BLOCO DE MODIFICADOR QUE DEVOLVE `false` É DONO DE TUDO ABAIXO
/// DELE.**
///
/// ⛔⛔⛔ **REPORT DO ENIO, 2026-09-08: *«o atalho das 4 viewports não
/// funciona»* — e o gate irmão desta suíte, escrito no mesmo dia CONTRA esta
/// família de defeito, não o viu.**
///
/// O `sculpt3d_key` tem um catch-all:
///
/// ```text
/// if ctrl {
///     if code != K::KeyZ { return false; }   // ← daqui para baixo, Ctrl+ é dele
///     …
/// }
/// ```
///
/// Ele existe por um bom motivo — sem ele um `Ctrl+1` dispararia o verbo do
/// dígito `1` —, e o preço é que **todo braço que exija `ctrl` e venha depois
/// dele está morto**. Foi o que matou a tecla da divisão *e* o `Ctrl+Numpad1`
/// (a vista oposta), sem um warning e sem o outro gate se mexer.
///
/// ⚠️⚠️ **O gate irmão pergunta *«duas teclas iguais?»*, e a lei verdadeira é
/// *«esta tecla é ALCANÇÁVEL?»*.** Duas claims da mesma tecla é só **uma** das
/// formas de uma ficar inalcançável; um `return` a montante é outra, e a
/// primeira régua é cega à segunda. *Uma régua que mede um caso de uma família
/// lê-se como se medisse a família.*
#[test]
fn nenhum_braco_de_tecla_vive_debaixo_de_um_catch_all_do_mesmo_modificador() {
    let fonte = include_str!("sculpt3d_keys.rs");
    // Onde cada modificador passa a ser propriedade de um catch-all: um bloco
    // `if <mod> {` cujo corpo contém um `return false;` sem condição de tecla.
    let mut dono: Vec<(&str, usize)> = Vec::new();
    for m in ["ctrl", "shift"] {
        let abre = format!("if {m} {{");
        let mut de = 0usize;
        while let Some(i) = fonte[de..].find(&abre) {
            let at = de + i;
            // O corpo até ao fecho na mesma indentação — chega olhar as ~12
            // linhas seguintes, que é onde um catch-all mora.
            let corpo: String = fonte[at..].lines().take(12).collect::<Vec<_>>().join("\n");
            if corpo.contains("return false;") {
                dono.push((m, at));
            }
            de = at + abre.len();
        }
    }
    assert!(
        !dono.is_empty(),
        "o censo nao achou catch-all nenhum -- ou o ficheiro mudou de forma, ou ele ficou cego \
         (e um censo cego le^-se como aprovado)"
    );
    let mut mortos = Vec::new();
    for (linha_n, linha) in fonte.lines().enumerate() {
        let l = linha.trim();
        if l.starts_with("//") || !l.starts_with("if code == K::") {
            continue;
        }
        let at = fonte.find(linha).unwrap_or(0);
        for (m, dono_at) in &dono {
            if l.contains(&format!("&& {m}")) && at > *dono_at {
                mortos.push(format!("linha {}: `{l}`", linha_n + 1));
            }
        }
    }
    // ⚠️ **Em português de propósito:** o `typos` do `ship.sh` reprova o PLURAL de
    // «catch-all» (ele quer `all` ou `falls`), e este vermelho so' aparecia na
    // arvore fundida. ⛔ E a 1.a cura falhou por a nota que a explicava conter a
    // propria palavra -- *um comentario sobre um lint passa pelo lint*.
    println!("quem apanha tudo: {dono:?}");
    assert!(
        mortos.is_empty(),
        "{mortos:?} -- estes bracos exigem um modificador cujo catch-all ja' devolveu `false` \
         acima deles: eles COMPILAM e NUNCA correm. Ou o braco sobe, ou o atalho sai do \
         `sculpt3d_key` (foi o que a tecla da divisao fez, para o despacho)"
    );
}
