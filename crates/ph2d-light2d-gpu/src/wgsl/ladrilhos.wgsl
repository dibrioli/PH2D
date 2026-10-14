
// ⭐ W5q — o LADRILHO 8×8 da grade fina que o anel resolve (`ladrilhos.rs`: um bit por ladrilho, em
// linhas de `⌈largura / 8⌉`, aceso onde há corpo da captura fina ou a janela). Cada módulo declara
// `ladrilhos` no seu grupo; `largura` = a da grade fina.
const LADO_DO_LADRILHO: i32 = 8;

fn ladrilho_aceso(t: vec2<i32>, largura: i32) -> bool {
    let n = (largura + LADO_DO_LADRILHO - 1) / LADO_DO_LADRILHO;
    let i = u32((t.y / LADO_DO_LADRILHO) * n + t.x / LADO_DO_LADRILHO);
    return ((ladrilhos[i >> 5u] >> (i & 31u)) & 1u) != 0u;
}
