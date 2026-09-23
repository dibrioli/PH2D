//! `PH2D_COMPOSITE_SMOKE` — a cena do **Composite Brush**, montada com a pilha do dono (2026-09-23).
//!
//! ```text
//! cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && \
//!   PH2D_COMPOSITE_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke
//! ```
//!
//! Ordem do dono: *«monte uma cena completa com o composite pronto para testarmos até o fim das
//! melhorias»*. ⚠️ **Ao contrário das irmãs, esta cena ARMA o pincel** — e é de propósito: o que
//! ela existe para julgar não é o valor de fábrica, é a pilha que o dono montou à mão na foto de
//! 2026-09-22 (o report *«ainda está lenta e engasgando com pincel grande»*), e remontá-la à mão a
//! cada smoke é o passo que ninguém repete igual duas vezes. O meio continua o de fábrica (o
//! Digital): a pilha vive nele.
//!
//! ## A tela
//!
//! Uma IMAGEM gerada, e não papel branco: numa tela chapada o borrão e o esfregão não têm o que
//! mostrar (medido: numa tela uniforme o borrão é um no-op). Ela tem um degradê de cor, uma grelha
//! escura a cada `64 px` (o esfregão arrasta-a, o borrão amolece-a) e discos de cor.
//!
//! ## A pilha (a da foto, posição `1` = o TOPO)
//!
//! | | operação | força | tamanho | cor |
//! |---|---|---|---|---|
//! | 1 | Blur | `1` | `2.048` | — |
//! | 2 | Brush | `0.133` | `0.574` | branco |
//! | 3 | Brush | `0.204` | `1.002` | vermelho |
//! | 4 | Brush | `0.176` | `1.221` | preto |
//! | 5 | Smear | `0.596` | `1` | — |
//! | 6 | Erase | `0.104` | `1` | Image |
//!
//! e o pincel em `Size 0.4` (`82,8 px` de raio).

use ph2d_asset::{AssetDb, AssetId};
use ph2d_core::Vec2;
use ph2d_ecs::SimWorld;
use ph2d_render::SpriteRenderer;
use std::collections::BTreeMap;

/// **O maior nível a que este roteador responde** — ele é de PRESENÇA (`var_os(..).is_some()`),
/// como o do `taper_smoke`, logo o único nível com significado é o `1`.
pub const NIVEIS: u32 = 1;

/// O lado da tela, em píxeis — o mesmo `1024²` da bancada `diag_preco_da_pilha`, que mediu esta
/// pilha, para o que o dono sente e o que a sonda mede serem o mesmo regime.
pub const LADO: u32 = 1024;

/// Se a cena está armada. Barato o bastante para ser perguntado por quadro.
pub fn enabled() -> bool {
    std::env::var_os("PH2D_COMPOSITE_SMOKE").is_some()
}

/// A tela gerada: degradê de cor, grelha escura a cada `64 px`, e discos de cor.
#[must_use]
pub fn imagem(lado: u32) -> Vec<u8> {
    let l = lado as f32;
    let mut px = vec![0u8; (lado * lado * 4) as usize];
    let discos = [
        (0.25f32, 0.30f32, 0.09f32, [230u8, 60, 60]),
        (0.70, 0.28, 0.12, [60, 120, 230]),
        (0.45, 0.70, 0.10, [250, 200, 40]),
        (0.80, 0.75, 0.07, [40, 170, 90]),
    ];
    for y in 0..lado {
        for x in 0..lado {
            let (u, v) = (x as f32 / l, y as f32 / l);
            let mut c = [
                (70.0 + 150.0 * u) as u8,
                (120.0 + 80.0 * (1.0 - v)) as u8,
                (200.0 - 120.0 * u) as u8,
            ];
            for &(cx, cy, r, cor) in &discos {
                if (u - cx).hypot(v - cy) < r {
                    c = cor;
                }
            }
            if x % 64 < 3 || y % 64 < 3 {
                c = [30, 30, 40];
            }
            let i = ((y * lado + x) * 4) as usize;
            px[i..i + 3].copy_from_slice(&c);
            px[i + 3] = 255;
        }
    }
    px
}

/// Montar a tela quando `PH2D_COMPOSITE_SMOKE=1`, UMA vez, devolvendo os bits da entidade para
/// quem chama pôr a selecção nela. Imprime O QUE montou e o roteiro.
///
/// ⚠️ O "uma vez" é deste módulo (um `AtomicBool`) e não um campo da `App`: a shell é composição,
/// e um campo lá por cada cena é o que a catraca `the_shell_only_shrinks` existe para não deixar
/// crescer.
pub fn spawn_if_enabled(
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    cell_idx: u32,
    pixels_per_meter: f32,
    atlas_asset_map: &mut BTreeMap<u32, AssetId>,
) -> Option<u64> {
    use std::sync::atomic::{AtomicBool, Ordering};
    static FEITO: AtomicBool = AtomicBool::new(false);
    if !enabled() || FEITO.swap(true, Ordering::Relaxed) {
        return None;
    }
    match ph2d_image_import::spawn_rgba(
        sim,
        renderer,
        asset_db,
        cell_idx,
        LADO,
        LADO,
        imagem(LADO),
        Vec2::new(0.0, 0.0),
        pixels_per_meter,
        atlas_asset_map,
        "Composite",
    ) {
        Ok((label, bits)) => {
            println!(
                "PH2D_COMPOSITE_SMOKE: tela '{label}' montada ({LADO}x{LADO}, uma imagem com \
                 degrade, grelha e discos). O Painter abre sozinho e arma a pilha da foto \
                 com o pincel em Size 0.4."
            );
            // ⚠️ O roteiro vive DENTRO do `println!`: o censo de texto da casa isenta o que sai por
            // terminal, e uma `const` solta perde essa isenção.
            println!(
                "PH2D_COMPOSITE_SMOKE: O ROTEIRO\n\
  1) A cena abre com o pincel JA na mao. Em cima a direita, clique na aba 'Painter' (ao lado \
     de 'Inspector'): a caixa 'Composite Brush' vem ligada, com as SEIS camadas da sua foto \
     (1 Blur em cima ... 6 Erase em baixo), e o pincel em Size 0.4.\n\
  2) Risque DEPRESSA, em curvas e em zigue-zague, por cima da grelha e dos discos. O traco tem de \
     acompanhar o rato sem engasgar. Deu errado se: ele fica para tras do cursor, ou salta aos \
     bocados.\n\
  3) Olhe o traco de perto: a grelha deve aparecer arrastada (Smear) e amolecida (Blur), com a \
     tinta branca, vermelha e preta por cima. Deu errado se: aparecem RECTANGULOS ou degraus ao \
     longo do traco, ou a borda do traco fica serrilhada em blocos.\n\
  4) Faca o MESMO gesto devagar e depressa, lado a lado. Os dois tracos devem ter o mesmo aspecto. \
     Deu errado se: o traco rapido sai diferente do lento (mais claro, mais escuro, com falhas).\n\
  5) Ctrl+Z: o traco inteiro tem de desaparecer de uma vez. Deu errado se: fica resto de tinta ou \
     um rectangulo mais claro onde ele estava.\n\
  6) Desligue a caixa 'Composite Brush', risque, e volte a ligar: sem ela e o pincel simples; com \
     ela volta a pilha, igual.\n\
  7) Com as setas da camada 5 (Smear), suba-o ate ficar ACIMA do Blur e risque. Este arranjo usa o \
     caminho antigo de proposito (pode ser mais lento); a imagem tem de continuar limpa.\n\
  8) Mude o Size do pincel para 0.2 e para 0.6 e risque: mais pequeno tem de ser mais leve, e \
     nenhum dos dois pode deixar rectangulos."
            );
            Some(bits)
        }
        Err(e) => {
            eprintln!("PH2D_COMPOSITE_SMOKE: nao consegui montar a tela: {e}");
            None
        }
    }
}

/// Armar a pilha da foto e o pincel na primeira vez que o Painter liga um documento sob a cena.
/// UMA vez: depois disso tudo o que o artista mexer é dele.
pub fn arm_brush_once(painter: &mut ph2d_tool_painter::PainterTool) {
    use std::sync::atomic::{AtomicBool, Ordering};
    static ARMADO: AtomicBool = AtomicBool::new(false);
    if !enabled() || ARMADO.swap(true, Ordering::Relaxed) {
        return;
    }
    arma_a_pilha_do_dono(painter);
    println!(
        "PH2D_COMPOSITE_SMOKE: pilha armada — {} camadas, Composite Brush ligado, Size 0.4.",
        painter.composite_len()
    );
}

/// **A pilha da foto, montada pelas portas públicas do Painter** — sem ler o ambiente, para o gate
/// a poder medir (um gate que lê o ambiente mede a máquina).
///
/// ⚠️ Pelas MESMAS portas que o painel usa (`acrescenta_camada` · `set_composite_layer_*` ·
/// `set_brush_size_norm`), e não escrevendo os campos: é isso que faz a quota e o piso do tamanho
/// valerem aqui como valem para o artista.
pub fn arma_a_pilha_do_dono(painter: &mut ph2d_tool_painter::PainterTool) {
    // A pilha, posição 0 = o TOPO. As operações vão pelos números que o painel usa (`Brush 0 ·
    // Smear 1 · Blur 2 · Erase 3`), e cada camada nova entra no FUNDO — logo cria-se de cima para
    // baixo.
    const PILHA: [(u8, f32, f32, Option<[f32; 3]>); 6] = [
        (2, 1.0, 2.048, None),
        (0, 0.133, 0.574, Some([1.0, 1.0, 1.0])),
        (0, 0.204, 1.002, Some([1.0, 0.0, 0.0])),
        (0, 0.176, 1.221, Some([0.0, 0.0, 0.0])),
        (1, 0.596, 1.0, None),
        (3, 0.104, 1.0, None),
    ];
    while painter.composite_len() > 0 {
        painter.retira_camada(0);
    }
    for (pos, &(op, forca, tamanho, cor)) in PILHA.iter().enumerate() {
        painter.acrescenta_camada(op);
        painter.set_composite_layer_strength(pos, forca);
        painter.set_composite_layer_size(pos, tamanho);
        match cor {
            Some(rgb) => painter.set_composite_layer_color(pos, rgb),
            None => painter.clear_composite_layer_color(pos),
        }
    }
    if !painter.composite_enabled() {
        painter.toggle_composite();
    }
    painter.set_brush_size_norm(0.4);
}

#[cfg(test)]
mod tests {
    /// **A cena monta a pilha da FOTO do dono** — e não outra. A régua é o snapshot que o PAINEL
    /// pinta (`brush_settings`), logo o que este gate afirma é o que o dono vê no cartão.
    ///
    /// ⚠️ Cada coluna é uma decisão da foto de 2026-09-22 (operação · força · tamanho · cor), e a
    /// ordem é do TOPO para baixo: a linha `1` do painel é a posição `0`, que a pilha aplica por
    /// último. ⛔ Uma pilha invertida mediria outro produto — foi a pergunta mais importante da
    /// auditoria de 2026-09-23 sobre a sonda de preço, e esta cena não pode reabri-la.
    #[test]
    fn a_cena_monta_a_pilha_da_foto() {
        let mut p = ph2d_tool_painter::PainterTool::default();
        super::arma_a_pilha_do_dono(&mut p);
        let s = p.brush_settings();
        assert!(s.composite_enabled, "o Composite Brush tem de vir ligado");
        assert_eq!(s.composite_len, 6, "as seis camadas da foto");
        assert_eq!(
            &s.composite_ops[..6],
            &[2, 0, 0, 0, 1, 3],
            "Blur em cima, três Brush, Smear, Erase em baixo"
        );
        let forcas = [1.0, 0.133, 0.204, 0.176, 0.596, 0.104];
        let tamanhos = [2.048, 0.574, 1.002, 1.221, 1.0, 1.0];
        for i in 0..6 {
            assert!(
                (s.composite_strength[i] - forcas[i]).abs() < 1e-6,
                "camada {}: força {} contra {}",
                i + 1,
                s.composite_strength[i],
                forcas[i]
            );
            assert!(
                (s.composite_size[i] - tamanhos[i]).abs() < 1e-6,
                "camada {}: tamanho {} contra {}",
                i + 1,
                s.composite_size[i],
                tamanhos[i]
            );
        }
        assert_eq!(
            &s.composite_color_authored[..6],
            &[false, true, true, true, false, false],
            "só os três Brush têm cor própria"
        );
        assert_eq!(s.composite_color[1], [1.0, 1.0, 1.0], "a camada 2 é branca");
        assert_eq!(
            s.composite_color[2],
            [1.0, 0.0, 0.0],
            "a camada 3 é vermelha"
        );
        assert_eq!(s.composite_color[3], [0.0, 0.0, 0.0], "a camada 4 é preta");
        assert_eq!(
            s.composite_erase_scope[5], 0,
            "a borracha apaga a imagem (Image)"
        );
        // O pincel em Size 0.4 — lido pela mesma lei do slider, não por uma conta à mão.
        assert!(
            (s.size_norm - 0.4).abs() < 1e-3,
            "o pincel tem de vir em Size 0.4 (lido {})",
            s.size_norm
        );
    }

    /// Armar DUAS vezes dá a mesma pilha — a montagem começa por esvaziar o que lá estiver, senão uma
    /// sessão que já tinha camadas ficava com a quota esgotada e a pilha pela metade.
    #[test]
    fn armar_sobre_uma_pilha_existente_da_a_mesma_pilha() {
        let mut p = ph2d_tool_painter::PainterTool::default();
        p.acrescenta_camada(0);
        p.acrescenta_camada(0);
        super::arma_a_pilha_do_dono(&mut p);
        let s = p.brush_settings();
        assert_eq!(s.composite_len, 6);
        assert_eq!(&s.composite_ops[..6], &[2, 0, 0, 0, 1, 3]);
    }

    /// A tela tem textura — num campo uniforme o borrão e o esfregão não têm o que mostrar.
    #[test]
    fn a_tela_nao_e_chapada() {
        let px = super::imagem(128);
        let cores: std::collections::BTreeSet<[u8; 3]> = px
            .as_chunks::<4>()
            .0
            .iter()
            .map(|c| [c[0], c[1], c[2]])
            .collect();
        assert!(
            cores.len() > 50,
            "a tela tem de ter variação: {} cores",
            cores.len()
        );
        assert!(
            px.as_chunks::<4>().0.iter().all(|c| c[3] == 255),
            "a tela é opaca"
        );
    }
}
