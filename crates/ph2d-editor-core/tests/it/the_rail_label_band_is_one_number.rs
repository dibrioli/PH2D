//! **A banda da sub-etiqueta do rail é UM número, lido — nunca copiado.**
//!
//! ⛔ Achado por auditoria em 2026-08-29: `LABEL_VISUAL_EXTENT_PX` e `LABEL_TO_CHIP_GAP_PX` eram
//! **privados** ao `tool_rail`, e o `cluster_painter` da topbar tinha **quatro literais** (`11.0`
//! e `3.0`, duas vezes cada) com o comentário *«mirror of rail's …»*.
//!
//! ⚠️ **Um espelho não é uma lei.** Mudar a constante do rail fazia a topbar discordar dele **em
//! silêncio** — a barra de cima e a barra da esquerda desalinhavam, e nada reprovava. A causa não
//! era descuido: era a **visibilidade**, que obrigava a copiar. Hoje as duas são `pub` e a topbar
//! lê-as.
//!
//! ⚠️ Este censo não olha para o valor `11.0` — ele pode e deve mudar. Olha para a **emparelhação**:
//! quem precisa da banda tem de a **ler**.

use std::path::Path;

#[test]
fn the_topbar_reads_the_rail_constants_instead_of_mirroring_them() {
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let dono = "widget/tool_rail.rs";

    let mut leituras = 0usize;
    let mut espelhos = Vec::new();
    let mut pilha = vec![raiz.clone()];
    while let Some(d) = pilha.pop() {
        for e in std::fs::read_dir(&d).expect("ler src") {
            let path = e.expect("entrada").path();
            if path.is_dir() {
                pilha.push(path);
                continue;
            }
            if path.extension().is_none_or(|x| x != "rs") {
                continue;
            }
            let rel = path
                .strip_prefix(&raiz)
                .expect("dentro de src")
                .to_string_lossy()
                .replace('\\', "/");
            let texto = std::fs::read_to_string(&path).expect("ler");
            for (i, linha) in texto.lines().enumerate() {
                // ⛔⛔ **O ESPELHO É TESTADO PRIMEIRO, e isso não é estilo.** A 1.ª versão deste
                // censo procurava a leitura antes e saía por `continue` — e a linha do espelho
                // **nomeia a constante dentro do próprio comentário**, logo era contada como
                // LEITURA e nunca chegava à verificação. A mutação (repor um espelho)
                // **SOBREVIVEU**. É a lei que esta casa já tinha escrita: *a forma nº 1 de esvaziar
                // um balde sem dar por isso é um `continue` a meio de um laço.*
                if linha.contains("mirror of rail's LABEL") {
                    espelhos.push(format!("{rel}:{}", i + 1));
                    continue;
                }
                // ⚠️ E a leitura conta-se sobre o CÓDIGO, não sobre a linha: uma menção num
                // comentário não é um consumidor.
                let codigo = linha.split("//").next().unwrap_or("");
                let nomeia = codigo.contains("LABEL_VISUAL_EXTENT_PX")
                    || codigo.contains("LABEL_TO_CHIP_GAP_PX");
                if nomeia && rel != dono {
                    leituras += 1;
                }
            }
        }
    }

    // A metade JUSTA: sem consumidores, este censo passaria por não haver nada que copiar.
    assert!(
        leituras >= 2,
        "a sonda tem de VER a topbar a ler as constantes do rail; contou {leituras}. Se os nomes \
         mudaram, o censo ficou cego e tem de ser reescrito, nunca apagado."
    );
    assert!(
        espelhos.is_empty(),
        "estes sitios ESPELHAM um numero do rail em vez de o ler: {espelhos:?}. As duas constantes \
         sao `pub` desde 2026-08-29 — importe-as de `crate::widget`. Um espelho faz a topbar \
         desalinhar do rail em silencio no dia em que a constante mudar."
    );
}
