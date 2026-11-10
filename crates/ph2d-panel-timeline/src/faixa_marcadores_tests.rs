//! Portões da faixa «Markers» (maquete B6): o nome vira para a esquerda só quando não cabe à direita.

use super::*;

/// ⭐ **O nome vira à esquerda perto do fim** (`.c13ml.rl`: «land» no fim do clipe), nunca antes: com
/// lugar à direita fica à direita (a `DX` do centro do chip); sem lugar, acaba a `DX` antes do chip.
/// *Mutação: virar sempre ⇒ reprova.*
#[test]
fn o_nome_vira_a_esquerda_so_quando_nao_cabe_a_direita() {
    let xs = [100.0, 400.0, 790.0];
    let borda = (90.0, 800.0);
    assert_eq!(
        lugar_do_nome(&xs, 0, borda, 40.0),
        (100.0 + DX, 400.0 - DX - 100.0 - DX)
    );
    let (x, w) = lugar_do_nome(&xs, 2, borda, 40.0);
    assert_eq!((x, w), (790.0 - DX - 40.0, 40.0), "land vira à esquerda");
    let (x, _) = lugar_do_nome(&xs, 1, borda, 40.0);
    assert!(x > 400.0, "top cabe à direita e fica lá");
}
