//! ⭐⭐⭐ **A REDE CONHECE TODO ESCRITOR DERIVADO DO PASSE DO DESENHO.**
//!
//! # O buraco que este gate fecha, e quem o nomeou
//!
//! O §F4.6d curou o passo fantasma dos verbos tardios com uma **rede** ([`crate::vec_tree_settle`])
//! que reconcilia a árvore antes da fotografia — e deixou escrito o limite dela:
//!
//! > ⛔ *«A rede não é derivável: nada mede que ela contenha todos os escritores derivados do passe
//! > do desenho. Um passe novo que alguém acrescente ao `render_loop` e esqueça aqui só é apanhado
//! > se o arnês o tiver.»*
//!
//! Esse é o caminho mais provável de o report do dono voltar: **por uma porta nova**, com a suíte
//! inteira verde. Este gate fecha-o.
//!
//! # A régua
//!
//! O passe do desenho tem uma janela bem delimitada — do assentamento do pivô vectorial
//! (`vec_transform::settle_origins`) à projecção (`vec_scene.reorder_to`). **Toda função chamada
//! nessa janela ou é RECONCILIAÇÃO — e então tem de estar na rede — ou é um DRENO de intenção**, e
//! esses são nomeados um a um abaixo.
//!
//! ⚠️ **A distinção não é de estilo, é de natureza:**
//!
//! - um **escritor derivado** (`assign_missing_*`, o assentamento de pivô, a projecção) roda por si
//!   e converge o estado — se não correr antes da fotografia, o quadro SEGUINTE converge sozinho, e
//!   o `post_frame_undo` lê isso como acção do artista;
//! - um **dreno** (`drain_reparent`, `restack`) só faz alguma coisa quando houve um gesto, e o
//!   gesto já é o passo. Correr um dreno na rede seria aplicar a intenção duas vezes.
//!
//! ⭐ **Este gate achou um buraco no minuto em que foi escrito:** o
//! `crate::flip_transform::settle_origins` — o pivô dos objectos **Flip** — corria no passe do
//! desenho e **não estava na rede**. É o MESMO mecanismo que o gate do *duplicar* já tinha
//! apanhado para o pivô vectorial (uma entidade cunhada tarde nasce com `Transform::default()`, e
//! o quadro seguinte assenta-a sozinho), noutra mídia.

const RENDER_LOOP: &str = include_str!("../src/render_loop/mod.rs");
const NET_SRC: &str = include_str!("../src/vec_tree_settle.rs");

/// ⚠️⚠️ **O CÓDIGO da rede, sem a prosa — e esta linha nasceu de uma mutação SOBREVIVENTE.**
///
/// A 1.ª redacção varria o ficheiro inteiro, e apagar a chamada ao
/// `crate::flip_transform::settle_origins` deixava o gate **VERDE**: o caminho continuava lá, no
/// **comentário** que explica por que ele foi acrescentado. *Um censo textual que não separa prosa
/// de código mente nos DOIS sentidos* — é lei desta casa, e esta régua pagou-a na estreia.
fn net_code() -> String {
    code_only(NET_SRC)
}

/// O CÓDIGO de `src`, sem as linhas de comentário — **a porta, com dois consumidores**.
///
/// ⚠️ Os dois censos deste ficheiro precisam dela, e a razão é a mesma dos dois lados: um caminho
/// citado numa frase não é uma chamada. *Escrever esta filtragem duas vezes seria duas respostas à
/// pergunta «o que é código aqui?», e a que envelhece é a que ninguém corrige.*
fn code_only(src: &str) -> String {
    src.lines()
        .filter(|l| {
            let s = l.trim_start();
            !(s.starts_with("//") || s.starts_with('*'))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// ⛔ **Os DRENOS de intenção** — correm na janela e **não** pertencem à rede.
///
/// ⚠️ Acrescentar um nome aqui é declarar *«isto só age quando houve um gesto»*. Se for falso, o
/// passo fantasma volta pela porta que este gate existe para fechar.
const DRAINS: &[&str] = &[
    // Drena a fila que o Blend encheu — vazia em todo quadro sem aquele gesto.
    "crate::vec_entities::restack",
    // Drena o `HierReparentIntent` do arrasto da Hierarquia (F4.6c). ⚠️ O `take()` no sítio dele é
    // load-bearing: aplicá-lo duas vezes reordenaria duas.
    "hero_intents::drain_reparent",
];

/// As funções chamadas na janela da reconciliação do passe do desenho.
fn writers_in_the_drawing_pass() -> Vec<String> {
    let a = RENDER_LOOP
        .find("crate::vec_transform::settle_origins(")
        .expect("o assentamento do pivô vectorial sumiu do render_loop — a janela começa nele");
    let b = RENDER_LOOP[a..]
        .find("vec_scene.reorder_to(")
        .expect("a projecção sumiu do render_loop — a janela acaba nela")
        + a;
    let corpo: String = code_only(&RENDER_LOOP[a..b]);

    let mut out: Vec<String> = Vec::new();
    for (i, _) in corpo.match_indices("::") {
        // Recua até ao início do caminho (`crate::a::b`, `ph2d_ecs::c`, `hero_intents::d`).
        let inicio = corpo[..i]
            .rfind(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == ':'))
            .map_or(0, |p| p + 1);
        let fim = corpo[i..]
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == ':'))
            .map_or(corpo.len(), |p| i + p);
        let caminho = &corpo[inicio..fim];
        let chamada = corpo[fim..].starts_with('(');
        let raiz = caminho.split("::").next().unwrap_or("");
        let interessa = matches!(raiz, "crate" | "hero_intents") || raiz.starts_with("ph2d_");
        if chamada && interessa && !out.iter().any(|o| o == caminho) {
            out.push(caminho.to_string());
        }
    }
    out.sort();
    out
}

/// **Mutação que deve sangrar:** apagar uma linha da rede (`vec_tree_settle.rs`) — este gate nomeia
/// a função que ficou de fora, e diz porque ela importa.
#[test]
fn the_net_knows_every_derived_writer_of_the_drawing_pass() {
    let net = net_code();
    let mut faltam: Vec<String> = Vec::new();
    for f in writers_in_the_drawing_pass() {
        if DRAINS.contains(&f.as_str()) {
            continue;
        }
        // ⚠️⚠️ **A chave é o CAMINHO, nunca o último segmento — e esta régua já mordeu por isso.**
        // A 1.ª redacção comparava por `rsplit("::")` e leu VERDE sobre o buraco que ela existia
        // para achar: `flip_transform::settle_origins` casava com o `vec_transform::settle_origins`
        // que a rede tem. *Dois módulos com a mesma função lêem-se iguais por nome solto*, e é
        // exactamente a família que este repo já pagou (`feedback_matching_by_a_loose_name…`).
        if !net.contains(&f) {
            faltam.push(f);
        }
    }
    assert!(
        faltam.is_empty(),
        "estes escritores derivados correm no passe do DESENHO e a rede do fim do quadro não os \
         tem: {faltam:?}\n\n\
         Um escritor derivado converge o estado SOZINHO. Fora da rede, a fotografia guarda o \
         estado por convergir e o quadro seguinte converge-o sem entrada nenhuma — que é a \
         definição do passo fantasma (report do dono, 2026-09-07).\n\
         cura: acrescente-o ao `vec_tree_settle.rs`. Se ele só age depois de um GESTO, ele é um \
         dreno e o sítio é a lista `DRAINS` deste ficheiro — com a razão escrita."
    );
}

/// ⭐⭐⭐ **E TODA PONTE documento ⟺ árvore tem de estar na rede — o censo por PADRÃO.**
///
/// # Porque este segundo censo existe, e o que ele achou
///
/// O censo por JANELA acima não alcança os `sync`: eles correm ~470 linhas mais acima, junto de
/// consumidores imediatos (o balde arma os preenchimentos novos logo a seguir ao vectorial). ⛔
/// Alargar a janela até lá arrastaria centenas de chamadas que nada têm de reconciliação, e a lista
/// de isenções passaria a ser o gate.
///
/// ⇒ a régua é o **PADRÃO**, e ela é exacta: *toda ponte `<mídia>_entities::sync` que o passe do
/// desenho corre tem de correr também na rede.* Um `sync` é bidireccional por construção — apagar a
/// entidade pela Hierarquia leva o objecto do documento **no quadro seguinte** —, e é essa
/// convergência tardia que nasce como passo fantasma.
///
/// ⭐ **Achou o `crate::flip_entities::sync`**, o irmão exacto do vectorial: mesma forma, mesma
/// latência, e fora da rede. ⚠️ E a régua apanha a **mídia seguinte** de graça: quem escrever um
/// `mesh_entities::sync` vê o gate vermelho antes de o defeito existir.
#[test]
fn every_document_to_tree_bridge_is_in_the_net() {
    let net = net_code();
    // ⚠️ **Sem a prosa dos DOIS lados**: um `// crate::mesh_entities::sync(…)` comentado no passe
    // do desenho não é uma chamada, e acusá-lo mandaria alguém pôr na rede código que não corre.
    let desenho = code_only(RENDER_LOOP);
    let mut faltam: Vec<String> = Vec::new();
    for (i, _) in desenho.match_indices("_entities::sync(") {
        let inicio = desenho[..i]
            .rfind(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == ':'))
            .map_or(0, |p| p + 1);
        let caminho = &desenho[inicio..i + "_entities::sync".len()];
        if !net.contains(caminho) && !faltam.iter().any(|f| f == caminho) {
            faltam.push(caminho.to_string());
        }
    }
    assert!(
        faltam.is_empty(),
        "estas pontes documento ⟺ árvore correm no passe do DESENHO e a rede não as tem: \
         {faltam:?}\n\n\
         Um `sync` é bidireccional: apagar a entidade pela Hierarquia leva o objecto do documento \
         no quadro SEGUINTE. Fora da rede, a fotografia guarda um mundo e um documento que \
         discordam — o passo fantasma do report de 2026-09-07, noutra mídia."
    );
}

/// ⛔ **E a lista de drenos não pode crescer em silêncio.**
///
/// Cada nome nela é uma afirmação — *«isto só age quando houve um gesto»* — que ninguém volta a
/// conferir. ⚠️ Ela **só encolhe**: um dreno que saiu da janela é uma linha que já não descreve
/// nada, e a próxima pessoa lê-a como se ainda descrevesse.
#[test]
fn every_named_drain_still_runs_in_that_window() {
    let na_janela = writers_in_the_drawing_pass();
    let mortos: Vec<&&str> = DRAINS
        .iter()
        .filter(|d| !na_janela.iter().any(|f| f == *d))
        .collect();
    assert!(
        mortos.is_empty(),
        "estes nomes da lista `DRAINS` já não correm na janela da reconciliação: {mortos:?} — a \
         isenção não descreve nada e tem de sair"
    );
}
