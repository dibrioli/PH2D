//! ⭐⭐ **A SONDA do tamanho da lista de tags** — o instrumento de onde sai o
//! [`ph2d_panel_inspector::ids::INSP_TAGS_OPT`], e o gate que o prende à medição.
//!
//! # ⛔ Porque este número NÃO podia ser escolhido
//!
//! A árvore de tags **não tem cap** (medido na W1: `9 344` tags abrem em `9,2 ms`), ao contrário do
//! `TIMERS_MAX` ou do `ANCHORS_MAX`, onde o array de ids copia o cap do modelo. Aqui o recurso é
//! outro: o POPOVER, que o [`HeroLayout::popover_region`] prende à altura da coluna do Inspector e
//! que rola. ⇒ a pergunta com resposta é *«quantas linhas o artista vê de uma vez?»*, e o array tem
//! de cobrir **pelo menos um ecrã cheio** — senão rolar seria a única forma de ver o que a busca já
//! tinha encontrado, que é o oposto do que uma busca faz.
//!
//! ⚠️ **A sonda IMPRIME** (`--nocapture`) e o gate compara. Sem o número impresso ao lado, a
//! próxima pessoa a mexer na altura da linha não sabe que moveu este teto.

use ph2d_editor_core::screens::layout::HeroLayout;
use ph2d_editor_core::zones::Rect;

/// As três janelas: a de trabalho do dono, um portátil, e a mais pequena que o app assume.
const JANELAS: [(&str, Rect); 3] = [
    (
        "workstation 2560x1440",
        Rect {
            x: 0.0,
            y: 0.0,
            w: 2560.0,
            h: 1440.0,
        },
    ),
    (
        "portatil 1600x900",
        Rect {
            x: 0.0,
            y: 0.0,
            w: 1600.0,
            h: 900.0,
        },
    ),
    (
        "tablet 1280x800",
        Rect {
            x: 0.0,
            y: 0.0,
            w: 1280.0,
            h: 800.0,
        },
    ),
];

/// Quantas linhas de opção cabem na região do popover desta janela.
fn linhas_visiveis(janela: Rect) -> usize {
    let regiao = HeroLayout::for_viewport(janela).popover_region();
    // Uma opção tem a altura do CHIP, e o chip de uma row de formulário é a linha do app.
    (regiao.h / ph2d_tokens::ROW_H_PX).floor().max(0.0) as usize
}

/// ⭐⭐⭐ **O array de opções cobre um ecrã cheio na maior janela** — e a tabela fica impressa.
///
/// ⚠️ **A barra é a janela MAIOR**, não a menor: é ela que mostra mais linhas de uma vez, logo é
/// ela que exige mais do array. Medir pela menor deixaria o ecrã grande com uma lista cortada a
/// meio de um espaço que ainda tinha.
///
/// **Mutação que deve sangrar:** encolher o `INSP_TAGS_OPT` para 16.
#[test]
fn a_lista_de_tags_cobre_um_ecra_cheio() {
    let cap = ph2d_panel_inspector::ids::INSP_TAGS_OPT.len();
    let mut pior = 0usize;
    println!("\n  janela                    região do popover   linhas visíveis");
    for (nome, j) in JANELAS {
        let regiao = HeroLayout::for_viewport(j).popover_region();
        let n = linhas_visiveis(j);
        pior = pior.max(n);
        println!("  {nome:<24}  {:>8.1} px        {n:>3}", regiao.h);
    }
    println!(
        "  INSP_TAGS_OPT.len() = {cap}   (linha de {:.1} px)\n",
        ph2d_tokens::ROW_H_PX
    );
    assert!(
        cap >= pior,
        "a lista de tags mostra {cap} opcoes e a maior janela tem espaco para {pior} de uma vez — \
         o artista rolaria para ver o que a busca ja' tinha encontrado"
    );
}
