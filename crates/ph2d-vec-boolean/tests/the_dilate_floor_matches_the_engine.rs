//! ⭐⭐ **O PISO DO OFFSET DO DOCUMENTO É O MESMO DO MOTOR.**
//!
//! O `ph2d_vec_scene::MIN_DILATE` decide se uma camada da pilha de aparência **está dilatada** (e
//! portanto se a shell a manda cozer); o [`ph2d_vec_boolean::MIN_OFFSET`] decide se o motor
//! **responde**.
//!
//! ⚠️ **O número está escrito duas vezes de propósito:** o documento é uma crate-FOLHA e não pode
//! depender do motor booleano — seria uma dependência ao contrário, e ela puxaria o `linesweeper`
//! para dentro do tipo que o save serializa. O gate vive **deste lado**, que é o que já depende dos
//! dois.
//!
//! ⛔ **Sem ele os dois derivam, e o modo de falha é MUDO:** o documento diria *"esta camada está
//! dilatada"*, o motor devolveria vazio, e a camada **desapareceria** em vez de desenhar onde
//! estava. O artista leria isso como *"apaguei a camada"*.

#[test]
fn the_dilate_floor_matches_the_engine() {
    assert_eq!(
        ph2d_vec_scene::MIN_DILATE,
        ph2d_vec_boolean::MIN_OFFSET,
        "o piso do DOCUMENTO e o do MOTOR divergiram: uma camada que o documento julga dilatada \
         passaria a receber geometria vazia, e desapareceria em silencio"
    );
}
