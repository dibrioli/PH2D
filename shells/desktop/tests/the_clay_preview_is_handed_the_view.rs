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

use std::fs;

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
        if head[from..].contains("alpha_stencil: Some(self.stencil_at(") {
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
        "o pincel entregue ao preview do barro não carrega o estêncil desta peça: \
         o carimbo volta a colar no barro para quem OLHA, enquanto o dab segue \
         preso à tela"
    );
}
