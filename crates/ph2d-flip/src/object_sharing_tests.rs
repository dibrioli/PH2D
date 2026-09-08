//! ⭐⭐⭐ **Os gates da PARTILHA POR DESENHO** (F8.3, 2026-09-08) — módulo-irmão do
//! [`super::object_tests`], pelo teto de 700 LOC do workspace e por ASSUNTO: ali medem-se as
//! **ops** do objecto (frames, camadas, refcount); aqui, o que a captura do undo paga por elas.
//!
//! ⚠️ **A fixtura é partilhada de propósito** (`super::object_tests::object_with_one_frame`) —
//! duas maneiras de montar o mesmo objecto seriam duas respostas à mesma pergunta, e a que
//! envelhece é a que ninguém corrige.

use super::object_tests::object_with_one_frame;
use super::*;
use std::sync::Arc;

/// ⭐⭐⭐ **Clonar o objecto NÃO copia a arte — só os ponteiros.**
///
/// # A medição que pediu esta wave
///
/// A captura do undo clona o documento a cada passo. Medido em `ph2d-flip/tests/measure_doc_clone`:
/// numa animação de **96 quadros**, `99,0 %` do que cada passo copiava era **desperdício** — a
/// pilha de `256` estados custava `912 MB`, e crescia com o tamanho da animação. Com a partilha por
/// desenho ela custa `9,5 MB` e **deixa de crescer**.
///
/// ⚠️ **A régua é `Arc::ptr_eq`, não a igualdade de valores.** Dois desenhos iguais mas em memórias
/// diferentes passariam num `assert_eq!` e seriam exactamente o defeito que esta wave cura.
#[test]
fn cloning_an_object_shares_every_drawing() {
    let (o, _l, _d) = object_with_one_frame();
    let copia = o.clone();
    assert_eq!(o.drawing_count(), copia.drawing_count());
    for (a, b) in o.drawings.iter().zip(copia.drawings.iter()) {
        assert!(
            Arc::ptr_eq(a, b),
            "clonar o objecto copiou a arte — a captura do undo faz exactamente isto a cada passo, \
             e era isso que custava 912 MB numa animacao de 96 quadros"
        );
    }
}

/// ⭐⭐ **E escrever num desenho separa SÓ ELE** — o copy-on-write do [`Arc::make_mut`].
///
/// ⚠️ **É o CONTROLO do gate acima**, e sem ele a partilha seria indistinguível de um documento que
/// nunca muda: um `drawing_mut` que devolvesse sempre o mesmo `Arc` passaria o primeiro gate e
/// **destruiria o undo do Flip em silêncio**.
#[test]
fn writing_to_one_drawing_leaves_the_others_shared() {
    let mut o = FlipObject::new(FlipObjectId(0), "Obj");
    let l = o.add_layer("L");
    let d0 = o
        .insert_frame(l, 0, Hold::Implicit, KeyKind::Keyframe)
        .expect("d0");
    let d1 = o
        .insert_frame(l, 5, Hold::Implicit, KeyKind::Keyframe)
        .expect("d1");
    let antes = o.clone();

    // Um traço no PRIMEIRO desenho — o gesto do artista.
    o.drawing_mut(d0)
        .expect("d0 existe")
        .strokes
        .push(crate::FlipStroke::new());

    assert!(
        !Arc::ptr_eq(&antes.drawings[d0.0 as usize], &o.drawings[d0.0 as usize]),
        "escrever nao separou o desenho tocado — o passo anterior estaria a ver a arte NOVA, e o \
         Ctrl+Z nao teria o que repor"
    );
    assert!(
        Arc::ptr_eq(&antes.drawings[d1.0 as usize], &o.drawings[d1.0 as usize]),
        "escrever num desenho copiou os OUTROS — e' precisamente o desperdicio que a wave cura"
    );
}

/// ⛔⛔ **E o formato NÃO se move um byte.**
///
/// `serde` com a feature `rc` escreve `Arc<T>` exactamente como `T`. ⚠️ **Sem este gate, embrulhar
/// o campo seria uma mudança de formato SILENCIOSA** — o postcard é posicional, e um ficheiro
/// gravado ontem seria lido errado sem uma palavra.
///
/// A impressão digital foi medida na árvore **antes** da wave: `8591` bytes, soma `454572`.
#[test]
fn sharing_a_drawing_does_not_move_a_byte_of_the_format() {
    let mut doc = crate::FlipDoc::new();
    let oid = doc.push_object("Personagem");
    let obj = doc.object_mut(oid).expect("nasceu");
    let layer = obj.add_layer("Traco");
    for f in 0..8i32 {
        let Some(d) = obj.insert_frame(layer, f, Hold::default(), KeyKind::default()) else {
            continue;
        };
        let draw = obj.drawing_mut(d).expect("nasceu");
        for k in 0..3usize {
            let mut s = crate::FlipStroke::new();
            for i in 0..10 {
                s.push_point(crate::Point::at(Vec2::new(
                    k as f32 + i as f32 * 0.5,
                    (i as f32).sin(),
                )));
            }
            draw.strokes.push(s);
        }
    }
    let b = doc.to_bytes().expect("serializa");
    let soma: u64 = b.iter().map(|&x| u64::from(x)).sum();
    assert_eq!(
        (b.len(), soma),
        (8591, 454_572),
        "o formato MOVEU-SE. A feature `rc` do serde escreve `Arc<T>` como `T`; se ela sair do \
         `Cargo.toml`, ou se o campo mudar de forma, todo ficheiro ja' gravado passa a ser lido \
         errado EM SILENCIO (o postcard e' posicional)."
    );
}

/// ⭐⭐⭐ **Escrever num desenho que NINGUÉM partilha não copia nada.**
///
/// # Este gate nasceu de uma mutação SOBREVIVENTE
///
/// Os dois gates acima mediam a **topologia** da partilha — quem aponta para quem — e uma mutação
/// que trocava o [`Arc::make_mut`] por um `Arc::new(clone)` **incondicional** passava nos dois: o
/// desenho tocado muda de ponteiro (que é o que eles exigem) e os outros nem são tocados.
///
/// ⚠️ *Um gate que mede a topologia não vê o CUSTO.* A metade que faltava é esta: no caminho comum
/// — o artista a desenhar num documento que nenhum passo de undo ainda fotografou — o
/// copy-on-write **não pode copiar**. Sem ela, um traço numa animação de 96 quadros pagaria uma
/// cópia de desenho a cada evento de ponteiro.
#[test]
fn writing_to_an_unshared_drawing_copies_nothing() {
    let (mut o, _l, d) = object_with_one_frame();
    // Ninguém mais tem este objecto: nenhum `Arc` do array tem um segundo dono.
    let antes = Arc::as_ptr(&o.drawings[d.0 as usize]);
    o.drawing_mut(d)
        .expect("existe")
        .strokes
        .push(crate::FlipStroke::new());
    assert_eq!(
        antes,
        Arc::as_ptr(&o.drawings[d.0 as usize]),
        "escrever num desenho EXCLUSIVO copiou-o — o copy-on-write virou copy-always, e o caminho \
         comum (desenhar) passa a pagar uma copia por evento de ponteiro"
    );
}

/// ⭐⭐ **E a partilha SOBREVIVE às operações de camada.**
///
/// # Também nasceu de uma mutação sobrevivente
///
/// O [`FlipObject::recompute_users`] corre em toda remoção de camada e reescreve `users` em **cada**
/// desenho. Escrito sem a guarda `if d.users() != c`, ele chama `Arc::make_mut` em todos — e
/// **desfaz em silêncio** a partilha que a wave comprou, num gesto que não tocou arte nenhuma.
///
/// ⚠️ O gate da topologia não o via: ele nunca chegava a chamar `recompute_users`.
#[test]
fn removing_a_layer_keeps_the_untouched_drawings_shared() {
    let mut o = FlipObject::new(FlipObjectId(0), "Obj");
    let manter = o.add_layer("Manter");
    let sair = o.add_layer("Sair");
    let d_manter = o
        .insert_frame(manter, 0, Hold::Implicit, KeyKind::Keyframe)
        .expect("d_manter");
    o.insert_frame(sair, 0, Hold::Implicit, KeyKind::Keyframe)
        .expect("d_sair");
    let antes = o.clone();

    assert!(o.remove_layer(sair), "a camada existia");

    assert!(
        Arc::ptr_eq(
            &antes.drawings[d_manter.0 as usize],
            &o.drawings[d_manter.0 as usize]
        ),
        "remover uma camada desfez a partilha de um desenho que ela nao tocou — o `recompute_users` \
         reescreveu `users` em todos, e cada escrita separa um `Arc` do passo de undo anterior"
    );
}
