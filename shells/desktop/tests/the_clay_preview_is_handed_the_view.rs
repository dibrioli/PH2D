//! **O PREVIEW NO BARRO RECEBE A VISTA DESTA PEÇA.**
//!
//! ⚠️ **Este gate existe porque uma MUTAÇÃO SOBREVIVEU.** Tirar o
//! `alpha_stencil` do pincel que o `sync_mesh` monta não derrubou nenhum dos
//! nove gates do preview — e o motivo é estrutural, não um buraco de fixture:
//! aquela função exige um `wgpu::Device`, então **nenhum gate de CPU a
//! alcança**. É a mesma cegueira que a `line/anim` mediu no overlay do motion
//! path (*"com o `draw` em `true` literal os 20 testes ficam VERDES e só o
//! arch-gate sangra"*).
//!
//! O defeito que ele previne é mudo: o carimbo continua preso à tela **para o
//! dab** (que passa por outra porta) e volta a colar no barro **para o
//! preview** — as duas metades da mesma ferramenta discordando sobre onde o
//! padrão está, com o artista mirando pelo que ele vê.
//!
//! ⚠️ **E ele ganhou uma segunda metade, porque o defeito voltou por outra
//! porta:** o preview RECEBIA a vista e mesmo assim discordava do dab, porque a
//! montava com outra ÂNCORA (o centro da peça contra o acerto do cursor).
//! Medido, o carimbo desenhado no barro saía **24,8% maior** que o depositado.
//! Hoje a chamada não tem âncora — e é isso que este gate afirma: **os dois
//! consumidores fazem a MESMA chamada, letra por letra**. Um parâmetro novo ali
//! é a forma exata de a divergência renascer.

use std::fs;

/// A chamada que os dois sítios têm de fazer. ⚠️ **O argumento faz parte da
/// afirmação:** *"pergunte a vista desta peça"* é uma pergunta sem ponto, e um
/// `stencil_for(pose, algo)` seria a âncora de volta.
const CALL: &str = "alpha_stencil: Some(self.stencil_for(pose))";

/// Onde o laço de slots mora. ⚠️ **A família e não o nome**: este módulo já
/// partiu dois arquivos por teto de LOC, e um gate que fixasse o endereço ficaria
/// verde por vácuo no terceiro corte — o controle positivo abaixo é o que
/// transforma *"não achei"* em falha alta.
const ROOT: &str = "src";

#[test]
fn the_clay_preview_is_handed_the_view() {
    let mut seen = 0usize;
    let mut ok = false;
    for entry in fs::read_dir(ROOT).expect("o shell tem um src/") {
        let path = entry.expect("entrada legível").path();
        if path.extension().is_none_or(|e| e != "rs") {
            continue;
        }
        let src = fs::read_to_string(&path).expect("arquivo legível");
        // O sítio é identificado pelo que ele FAZ — entregar um pincel ao
        // `PreviewState::refresh` —, nunca por uma linha ou um nome de função.
        let Some(call) = src.find("preview.refresh(") else {
            continue;
        };
        seen += 1;
        // A janela é o bloco ANTES da chamada: é ali que o pincel é montado.
        let head = &src[..call];
        let from = head.len().saturating_sub(900);
        if head[from..].contains(CALL) {
            ok = true;
        }
    }
    assert_eq!(
        seen, 1,
        "o preview do barro passou a ser entregue em {seen} sítios — este gate \
         mede UM, e com dois ele deixaria de dizer qual deles carrega a vista"
    );
    assert!(
        ok,
        "o pincel entregue ao preview do barro não carrega o estêncil desta peça \
         por `{CALL}`: ou ele não recebe a vista — e o carimbo volta a colar no \
         barro para quem OLHA —, ou ele a pede com uma ÂNCORA, e aí os dois \
         desenham tamanhos diferentes"
    );
}

/// **E O DAB PERGUNTA A MESMA COISA** — a outra ponta da divergência.
///
/// ⚠️ **Sem esta metade o gate acima é satisfeito por um preview correto ao lado
/// de um dab que pergunta de outro jeito** — que é exatamente o estado que o
/// report de 2026-08-09 descreve: *a tinta da máscara projetada no objeto não
/// corresponde ao que realmente está sendo esculpido*.
#[test]
fn the_dab_asks_the_view_the_same_way_the_preview_does() {
    let src = fs::read_to_string("src/sculpt3d_space.rs").expect("o dono do espaço da cena existe");
    // O controle positivo: se o `armed_brush` se mudar de arquivo, isto falha
    // ALTO em vez de varrer o vazio e passar.
    assert!(
        src.contains("fn armed_brush"),
        "o `armed_brush` mudou-se de arquivo — este gate estaria a ler o vazio"
    );
    assert!(
        src.contains(CALL),
        "o pincel do DAB não pede a vista por `{CALL}`: os dois consumidores \
         voltaram a montar o estêncil de jeitos diferentes"
    );
}
