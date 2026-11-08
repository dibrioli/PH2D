//! ⭐ **A cena 9 do editor de scripts** (`PH2D_CODE_SMOKE=9`, 8b + 8c — `docs/CodeEditor/07_arrastar_para_o_codigo.md`
//! §3.4 e §3.10): a nave azul corre o `ligar.luau`, aberto numa aba e escolhida (o Inspector mostra os
//! cartões dela); o dono arrasta NOMES do Inspector para o código e, com Alt, a «Door» da Hierarquia
//! vira uma propriedade de objecto que ele depois aponta à «Lamp» largando-a no cartão Script.

pub use ph2d_code_scenes::inspector::{LIGAR_AT, LIGAR_LUAU, LIGAR_NAME, monta};

/// A porta da shell: monta, anuncia e escolhe a nave (o Inspector mostra os cartões dela).
pub fn montar(cx: crate::scene_ctx::SceneCtx<'_>) -> Option<u64> {
    let pecas = monta(cx.sim, cx.vec_scene, cx.vec_entities);
    println!(
        "[code-smoke] =9 arrastar do Inspector: a nave azul corre o ligar.luau, já aberto numa aba e \
         escolhida. ① «Rotation» (cartão Transform) e ② «giro» (cartão Script) arrastados para a \
         linha que se escreve no update: a nave gira. ③ Alt + arrastar «Door» para a linha vazia do topo: a porta \
         sobe e desce. ④ Largar «Lamp» no campo «door» do cartão Script: agora é a lâmpada."
    );
    pecas.first().map(|e| e.to_bits())
}
