//! ⭐⭐⭐ **O LADRILHO DO PONTO** — o disco que uma corrente **sem aparência** amostra.
//!
//! Módulo irmão do [`super`] por RESPONSABILIDADE, não por teto de LOC: aqui mora *que forma
//! um ponto tem*, e lá *como um ladrilho entra no átlas*. É uma lei com régua própria (a
//! cobertura ao longo do raio), e ela não se lê no meio do empacotador.
//!
//! A razão de o ladrilho existir está em [`super::DOT_TILE_KEY`].

/// A rampa da borda, **em texels**. Um disco de arestas duras a esta resolução cintila quando
/// se move por baixo de um texel por pixel — que é onde um ponto vive —, e um texel de rampa é
/// o mínimo que o filtro bilinear do sampler consegue interpolar sem serrilha.
///
/// ⚠️ **Em texels e não uma fracção do raio**: a serrilha é um facto da AMOSTRAGEM (o tamanho
/// do texel), não do tamanho do disco. Escrita como fracção, um ladrilho maior teria uma borda
/// proporcionalmente mais larga e o ponto ficaria progressivamente mais esborratado.
const RAMPA_TEXELS: f32 = 1.0;

/// A fracção do ladrilho que o disco ocupa, de bordo a bordo. Menos de `1` deixa uma margem
/// transparente à volta: sem ela o disco encosta na fronteira da região e o **mip-chain** do
/// átlas mistura-o com o que o empacotador puser ao lado (o mesmo motivo por que o ladrilho
/// branco não é 1×1 — ver [`super::TextureAtlas::insert_white_tile`]).
const DIAMETRO: f32 = 0.875;

/// O ladrilho RGBA de um disco branco de lado `px`, **premultiplicado pela cobertura**.
///
/// Ver [`super::TextureAtlas::insert_dot_tile`] para a razão de o RGB acompanhar a alfa em vez
/// de ser `255` chapado.
#[must_use]
pub(super) fn make_dot_tile(px: u32) -> Vec<u8> {
    let n = px as usize;
    let mut out = vec![0u8; n * n * 4];
    let centro = px as f32 * 0.5;
    let raio = px as f32 * DIAMETRO * 0.5;
    for y in 0..n {
        for x in 0..n {
            // O centro do texel, não o canto: com o canto o disco fica meio texel fora do
            // sítio nos dois eixos, e a assimetria vê-se num ponto de 8 px no ecrã.
            let dx = (x as f32 + 0.5) - centro;
            let dy = (y as f32 + 0.5) - centro;
            let d = dx.hypot(dy);
            // `1` bem dentro, `0` bem fora, e uma rampa linear de [`RAMPA_TEXELS`] entre as
            // duas — a mesma forma que o `smoothstep` de uma borda macia, sem a curva, porque
            // aqui a rampa tem a largura de um texel e a curvatura não é observável.
            let cobertura = ((raio - d) / RAMPA_TEXELS).clamp(0.0, 1.0);
            let v = (cobertura * 255.0 + 0.5) as u8;
            let i = (y * n + x) * 4;
            out[i] = v;
            out[i + 1] = v;
            out[i + 2] = v;
            out[i + 3] = v;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atlas::DEMO_TILE_PX;

    /// O centro é opaco, a quina é vazia, e **a borda é uma RAMPA** — as três metades, porque
    /// cada uma sozinha passa sobre uma forma errada: só o centro passa num ladrilho todo
    /// branco, só a quina passa num ladrilho todo vazio, e as duas juntas passam sobre um
    /// disco de aresta DURA, que é precisamente o que cintila.
    #[test]
    fn o_ponto_e_um_disco_com_a_borda_macia() {
        let px = DEMO_TILE_PX;
        let n = px as usize;
        let t = make_dot_tile(px);
        assert_eq!(t.len(), n * n * 4, "RGBA justo");
        let a = |x: usize, y: usize| t[(y * n + x) * 4 + 3];
        let c = n / 2;
        assert_eq!(a(c, c), 255, "o centro e' opaco");
        assert_eq!(a(0, 0), 0, "a quina e' vazia");
        // A margem da fronteira: sem ela o mip mistura o disco com o vizinho do empacotador.
        for k in 0..n {
            assert_eq!(a(k, 0), 0, "a fileira de cima toca a fronteira em {k}");
            assert_eq!(a(0, k), 0, "a coluna da esquerda toca a fronteira em {k}");
        }
        // A rampa: ao atravessar a borda pelo eixo, ALGUM texel tem de ser parcial. Um disco
        // duro dá só `255` e `0`, e este assert e' o unico que o separa.
        let parciais = (0..n).filter(|&x| (1..255).contains(&a(x, c))).count();
        assert!(
            parciais >= 2,
            "a borda tem de ser rampa (parciais = {parciais})"
        );
    }

    /// ⭐ **O RGB acompanha a ALFA** — o ladrilho é premultiplicado, e é isso que faz a borda
    /// ler-se igual nas duas convenções de composição desta casa.
    #[test]
    fn o_ponto_e_premultiplicado() {
        let px = DEMO_TILE_PX;
        let t = make_dot_tile(px);
        for q in t.as_chunks::<4>().0 {
            assert_eq!(
                [q[0], q[1], q[2]],
                [q[3], q[3], q[3]],
                "RGB tinha de acompanhar a cobertura"
            );
        }
    }

    /// O disco é **redondo**, e a régua é a que o dono lê: a largura da fileira do meio tem de
    /// bater com a altura da coluna do meio, e uma quina tem de estar de fora.
    ///
    /// ⚠️ **Sem a segunda metade um QUADRADO passa** — ele tem a mesma largura e a mesma
    /// altura —, e o quadrado é exactamente a forma que esta wave existe para não desenhar.
    #[test]
    fn o_ponto_e_redondo_e_nao_um_quadrado() {
        let px = DEMO_TILE_PX;
        let n = px as usize;
        let t = make_dot_tile(px);
        let a = |x: usize, y: usize| t[(y * n + x) * 4 + 3];
        let c = n / 2;
        let largura = (0..n).filter(|&x| a(x, c) > 127).count();
        let altura = (0..n).filter(|&y| a(c, y) > 127).count();
        assert_eq!(largura, altura, "um disco e' tao largo quanto alto");
        assert!(largura > n / 2, "e ocupa o ladrilho ({largura} de {n})");
        // ⭐ **O discriminador contra um QUADRADO:** um texel cuja COLUNA está dentro do disco
        // e cuja DIAGONAL está fora. Em `0,15·n` a distância ao centro vale `0,35·n` pelo eixo
        // (dentro de `r = 0,4375·n`) e `0,35·n·√2 = 0,495·n` na diagonal (fora) — num quadrado
        // os dois estariam cheios, e é por isso que as duas metades andam juntas.
        let d = (n as f32 * 0.15) as usize;
        assert!(
            a(d, c) > 0,
            "o CONTROLO: a coluna em {d} tem de estar dentro"
        );
        assert_eq!(a(d, d), 0, "e a diagonal no mesmo x tem de sair do disco");
    }
}
