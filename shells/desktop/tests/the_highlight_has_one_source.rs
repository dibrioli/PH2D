//! **O REALCE TEM UMA FONTE SÓ** — a linha que acende e a forma que ganha contorno são o **mesmo
//! objecto**, ou não são nada.
//!
//! # A classe de defeito
//!
//! O realce de proveniência tem **dois produtores** (o ponteiro sobre o canvas · o ponteiro sobre
//! uma linha da Hierarquia) e **dois consumidores** em pontos DIFERENTES do quadro: a Hierarquia
//! publica cedo (`snapshots::publish`, ~2450) e o contorno desenha tarde (~8520).
//!
//! ⚠️ **Se cada consumidor picasse por si, eles picariam contra mapas vivos diferentes** — o
//! `vec_live_drawn` é reescrito no fim do quadro. A linha acesa e a forma contornada passariam a
//! ser objectos diferentes **em movimento**, e cada metade continuaria correcta sozinha: é a
//! assinatura exacta do defeito que esta linha corrigiu três vezes em 2026-08-23 (*pintar e
//! despachar têm de ler a MESMA fonte*).
//!
//! # A lei
//!
//! *Há UM pick de hover por quadro, no topo do `run_render_frame`, e os dois consumidores leem o
//! campo que ele escreve.*

use std::path::{Path, PathBuf};

fn shell(rel: &str) -> String {
    let p: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

/// ⛔ **O PICK DE HOVER ACONTECE UMA VEZ.**
///
/// A agulha é o nome da porta. Um segundo `pick_hovered_object` em qualquer sítio da shell é, por
/// construção, um segundo pick contra outro estado do quadro.
#[test]
fn the_hover_pick_happens_exactly_once() {
    let mut total = 0;
    for rel in [
        "src/render_loop/mod.rs",
        "src/render_loop/snapshots.rs",
        "src/input_dispatch.rs",
    ] {
        total += shell(rel).matches("pick_hovered_object(").count();
    }
    assert_eq!(
        total, 1,
        "o realce passou a ser picado {total} vezes por quadro — os consumidores estão em pontos \
         diferentes do frame, e dois picks dão dois objectos assim que o mapa vivo mudar entre eles"
    );
}

/// ⛔ **E OS DOIS CONSUMIDORES LEEM O MESMO CAMPO.**
///
/// ⚠️ É a outra metade: um pick só não basta se um dos consumidores voltar a derivar a resposta
/// por outro caminho (o `hot_id` cru, a seleção, o primeiro da lista). O campo é o contrato.
#[test]
fn both_consumers_read_the_one_field() {
    let frame = shell("src/render_loop/mod.rs");
    // ⚠️ **A âncora é a PORTA, não a forma da atribuição.** A 1.ª versão casava com
    // `self.hovered_object = hovered` — a linha exacta que existia — e expirou no mesmo dia, ao
    // estender o realce a todos os objectos. *Uma âncora que copia a implementação de uma lei
    // expira sempre que a lei se muda de casa; a que nomeia a porta sobrevive.*
    assert!(
        frame.contains("self.hovered_object = self.pick_hovered_object("),
        "o campo do quadro deixou de ser escrito pela porta — sem ele não há fonte única a ler"
    );
    // ⭐ E a GEOMETRIA é resolvida com o objecto, no mesmo sítio: resolvê-la no sítio de desenho
    // seria escolher o objecto duas vezes.
    assert!(
        frame.contains("self.hover_outline = match self.hovered_object"),
        "o contorno deixou de ser resolvido junto com o objecto que ele desenha"
    );
    // O contorno do canvas.
    assert!(
        frame.contains("let Some(bits) = self.hovered_object")
            && frame.contains("!self.hover_outline.is_empty()"),
        "o contorno do canvas deixou de ler os campos do quadro"
    );
    // A Hierarquia, pela mesma resposta passada ao `publish`.
    assert!(
        frame.contains("self.hovered_object,"),
        "a Hierarquia deixou de receber a resposta do quadro — ela voltaria a derivar a sua"
    );
    assert!(
        shell("src/render_loop/snapshots.rs").contains("entry.hovered = true"),
        "a linha da Hierarquia deixou de acender"
    );
}

/// ⛔ **A HIERARQUIA NÃO PICA O CANVAS.**
///
/// ⚠️ O `snapshots.rs` recebe a resposta RESOLVIDA, e nunca os ingredientes. Se ele ganhasse um
/// pick próprio, ele picaria com o que tem à mão — os pedaços destruturados do `AppGfx`, sem o
/// mapa vivo fundido — e acenderia a linha de um objecto que o clique não pega.
#[test]
fn the_hierarchy_does_not_pick_the_canvas() {
    let snap = shell("src/render_loop/snapshots.rs");
    assert!(
        !snap.contains("pick_all_at_world"),
        "o publicador da Hierarquia ganhou um pick próprio — ele não tem o mapa vivo FUNDIDO à \
         mão, então a linha acesa deixaria de ser a forma que o clique pega"
    );
}

/// ⛔ **O COMPOSTO DE PICK EXISTE UMA VEZ.**
///
/// ⚠️ *"O que este ponto pega"* é vetor + Flip + sprites, nessa ordem de z — e a lista estava
/// **copiada duas vezes** dentro do `input_dispatch` (o clique com modificador e o clique simples)
/// quando o realce de proveniência chegou. Três cópias é como o realce acende uma coisa e o clique
/// pega outra: cada cópia está certa sozinha, e nenhuma fica vermelha.
///
/// ⚠️ **A agulha é `hits.extend(…pick_sprites_at_world(`, e a precisão é a lei.** Um pick de
/// sprites SOZINHO é legítimo e existe: o conta-gotas de corpo de um joint quer um corpo físico, e
/// um corpo físico é uma sprite — perguntar pelo vetor ali seria oferecer o que não serve. O que
/// **não** pode nascer outra vez é o ACRÉSCIMO de sprites a uma lista de hits, que é a forma de um
/// composto. *Uma agulha que acusa o inocente é pior que não haver agulha.*
#[test]
fn the_object_pick_composite_exists_once() {
    let offenders: Vec<String> = ["src/input_dispatch.rs", "src/render_loop/mod.rs"]
        .iter()
        .filter(|rel| shell(rel).contains("hits.extend(ph2d_render::pick_sprites_at_world("))
        .map(|rel| (*rel).to_string())
        .collect();
    assert!(
        offenders.is_empty(),
        "estes sítios montam um segundo composto de pick — a porta é \
         `hover_highlight::pick_objects_at`, e ela existe porque o realce e o clique têm de \
         concordar sobre o que está sob o dedo:\n  {}",
        offenders.join("\n  ")
    );
    // ⚠️ **O CONTROLE:** a porta tem de existir e usar as três fontes — sem isto o gate ficaria
    // verde no dia em que alguém apagasse o pick de sprites por completo.
    let door = shell("src/hover_highlight.rs");
    for source in [
        "vec_gizmo_view::pick_all_at_world",
        "flip_gizmo_view::pick_all_at_world",
        "pick_sprites_at_world",
    ] {
        assert!(
            door.contains(source),
            "a porta do pick perdeu a fonte `{source}` — um objecto dessa família deixaria de ser \
             apontável, e o realce ficaria mudo sobre ele"
        );
    }
}

/// ⛔ **O REALCE NÃO TEM PORTÃO DE MODO** (Enio, 2026-08-23).
///
/// ⚠️ Ele nasceu guardado por `vector_active`, e a pergunta que ele responde — *"qual destes
/// objectos é este?"* — não é de modo nenhum: o clique pega objecto em qualquer um. Um portão aqui
/// dava resposta só onde ela já era mais fácil.
#[test]
fn the_highlight_has_no_mode_gate() {
    let frame = shell("src/render_loop/mod.rs");
    let at = frame
        .find("if let Some(bits) = self.hovered_object")
        .expect("o sítio que desenha o contorno desapareceu");
    let head = &frame[at.saturating_sub(400)..at];
    assert!(
        !head.contains("if vector_active"),
        "o contorno voltou a ser guardado por um modo — ele segue o CLIQUE, e o clique pega \
         objecto em qualquer modo"
    );
}

/// ⛔ **O SOM DE UI CONFIRMA O QUE A MÃO FEZ, e o HOVER nunca soa** (estudo de UI viva, D1).
///
/// ⚠️ É a única linha que separa um som de UI bom de um irritante. Passar o rato por uma fileira de
/// botões é o gesto mais barato e mais frequente do editor; sonorizá-lo transforma navegar num
/// chocalho — e é a irmã exacta da cerca do §6.2 (*o realce de uma lista obedece ao cursor*).
///
/// ⚠️ **A 1ª versão deste gate media PROXIMIDADE a uma palavra** (`hover`/`point(` a três linhas do
/// armamento) e um mutante que armava o som **dentro do `pick_hovered_object`** passou por ela — o
/// contexto local não dizia `hover` nenhum. *Um oráculo de vizinhança mede a redacção, não a lei.*
///
/// ⇒ o que se afirma agora é preciso e verificável: **o pipeline do ponteiro é MUDO** (o módulo do
/// realce não arma som nenhum), e os sítios que armam são uma **lista explícita** — um sítio novo
/// aparece no diff em vez de nascer em silêncio.
#[test]
fn the_ui_sound_never_follows_the_pointer() {
    assert!(
        !shell("src/hover_highlight.rs").contains("pending_ui_sound"),
        "o módulo do REALCE armou um som — ele corre a cada movimento do ponteiro, e navegar \
         viraria um chocalho"
    );
}

/// ⛔ **OS SÍTIOS QUE ARMAM UM SOM SÃO ESTES, e nenhum outro.**
///
/// ⚠️ A lista é a feature: cada entrada é uma coisa que **a mão fez** (uma escolha no pie menu, uma
/// consolidação, um interruptor, uma recusa). Um sítio novo aqui é um ato deliberado — e um som que
/// nasça noutro sítio, sem passar por esta lista, é a forma exacta de o app começar a comentar o
/// que ele próprio decidiu.
#[test]
fn only_the_listed_gestures_arm_a_sound() {
    // (ficheiro, quantos armamentos ele tem, o que eles CONFIRMAM)
    const ARMED: &[(&str, usize, &str)] = &[
        ("src/radial_input.rs", 1, "escolher no pie menu"),
        (
            // ⭐ +1 (plano 32 W8): **fazer o conjunto de estados do Morph**. É deliberado e é da
            // mesma espécie do primeiro — um COMMIT: o clique cria um objecto, reparenta as formas
            // escolhidas e esconde-as, tudo de vez. A mão fez uma coisa grande, e o som confirma-a.
            // ⛔ Nada nas transições em si soa: elas correm **durante a reprodução**, e um som por
            // transição seria o app a comentar o que o motor decidiu.
            "src/render_loop/mod.rs",
            4,
            "consolidar a booleana · o interruptor da preview de poses · fazer o conjunto de \
             Morph States · o interruptor da preview do Morph",
        ),
        (
            "src/input_dispatch.rs",
            2,
            "as duas recusas da trava do Painter",
        ),
    ];
    for (rel, want, what) in ARMED {
        let n = shell(rel).matches("pending_ui_sound = Some").count();
        assert_eq!(
            n, *want,
            "{rel} arma {n} som(ns) e a lista diz {want} ({what}) — se o novo é deliberado, \
             acrescente-o AQUI com o que ele confirma"
        );
    }
    // ⚠️ **E nenhum outro ficheiro da shell arma**: sem esta metade, a lista acima seria uma
    // contagem de quem já lá está, e o sítio novo nasceria noutro ficheiro sem ninguém ver.
    let listed: Vec<&str> = ARMED.iter().map(|(r, _, _)| *r).collect();
    let mut strays = Vec::new();
    walk_shell_src(&mut |rel, src| {
        if !listed.contains(&rel.as_str()) && src.contains("pending_ui_sound = Some") {
            strays.push(rel);
        }
    });
    assert!(
        strays.is_empty(),
        "estes ficheiros armam um som fora da lista:\n  {}",
        strays.join("\n  ")
    );
}

/// Percorre `src/` da shell, chamando `f(caminho_relativo, fonte)`.
fn walk_shell_src(f: &mut dyn FnMut(String, String)) {
    fn go(dir: &Path, root: &Path, f: &mut dyn FnMut(String, String)) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                go(&p, root, f);
            } else if p.extension().and_then(|s| s.to_str()) == Some("rs")
                && let Ok(s) = std::fs::read_to_string(&p)
                && let Ok(rel) = p.strip_prefix(root)
            {
                f(format!("src/{}", rel.display()), s);
            }
        }
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    go(&root, &root, f);
}

/// ⛔ **E UM SOM POR QUADRO, drenado com `take`.**
///
/// ⚠️ Sem o `take`, um gesto que enchesse o canal e não o limpasse tocaria em **todos** os quadros
/// seguintes — um clique viraria um zumbido, e o artista desligaria a feature inteira.
#[test]
fn the_ui_sound_channel_is_drained_with_take() {
    let frame = shell("src/render_loop/mod.rs");
    assert!(
        frame.contains("if let Some(what) = self.pending_ui_sound.take()"),
        "o canal do som deixou de ser drenado com `take` — um clique viraria um zumbido"
    );
}
