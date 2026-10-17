// ⭐ W5z — os T e os R da holográfica em TEXTURAS `Rg32Uint` (`Config::hrc_em_texturas`): as páginas de
// uma textura são comprimíveis desde que nasce, e o vácuo lê-se sem tocar na DRAM (handoff W5z). O índice
// do buffer vira um texel por LADRILHOS de 8 × 8 (64 B × 8 linhas: o bloco da compressão) — em ordem de
// linha a leitura não passa da banda da DRAM (`sonda_compressao_das_texturas`). A largura é a de
// `hrc::LARGURA_DOS_NIVEIS`.
@group(3) @binding(5) var t_a: texture_2d<u32>;
@group(3) @binding(6) var t_b: texture_2d<u32>;
@group(3) @binding(7) var t_saida: texture_storage_2d<rg32uint, write>;
@group(3) @binding(8) var r_cima: texture_2d<u32>;
@group(3) @binding(9) var r_saida: texture_storage_2d<rg32uint, write>;

const LADRILHOS_POR_LINHA_HRC: u32 = 1024u;

fn texel_do_nivel(i: u32) -> vec2<u32> {
    let l = i >> 6u;
    return vec2<u32>(
        (l % LADRILHOS_POR_LINHA_HRC) * 8u + (i & 7u),
        (l / LADRILHOS_POR_LINHA_HRC) * 8u + ((i >> 3u) & 7u),
    );
}

fn ler_t_a(i: u32) -> vec2<u32> { return textureLoad(t_a, texel_do_nivel(i), 0).xy; }
fn ler_t_b(i: u32) -> vec2<u32> { return textureLoad(t_b, texel_do_nivel(i), 0).xy; }
fn ler_r_cima(i: u32) -> vec2<u32> { return textureLoad(r_cima, texel_do_nivel(i), 0).xy; }
fn escrever_t(i: u32, e: vec2<u32>) { textureStore(t_saida, texel_do_nivel(i), vec4<u32>(e, 0u, 0u)); }
fn escrever_r(i: u32, e: vec2<u32>) { textureStore(r_saida, texel_do_nivel(i), vec4<u32>(e, 0u, 0u)); }
