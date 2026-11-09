//! «Hears» (plano 13 Onda B, item 6): os sinais que o `on_signal` do topo compara, no idioma das
//! cenas e da ajuda (`if signal == "coin" then`), com qualquer nome de parâmetro.

use ph2d_code_buffer::Text;
use ph2d_code_syntax::Syntax;

fn ouvidos(src: &str) -> Vec<(String, usize)> {
    let text = Text::new(src);
    Syntax::new(&text)
        .heard_signals(&text)
        .into_iter()
        .map(|(n, r)| (n, text.line_of_byte(r.start)))
        .collect()
}

#[test]
fn os_sinais_comparados_pelo_on_signal_pela_ordem_e_um_por_nome() {
    let src = "ph2d.property(\"speed\", 220)\n\
               function on_signal(self, signal, from)\n\
               \x20 if signal == \"coin\" then\n\
               \x20   self.coins = self.coins + 1\n\
               \x20 elseif \"door\" == signal then\n\
               \x20   print(\"open\")\n\
               \x20 end\n\
               \x20 if signal ~= \"coin\" then return end\n\
               end\n";
    assert_eq!(
        ouvidos(src),
        [("coin".to_owned(), 2), ("door".to_owned(), 4)],
        "o `==` dos dois lados, o `~=` conta, e o repetido fica na 1.ª linha"
    );
}

#[test]
fn o_nome_do_parametro_e_o_do_script() {
    let src = "function on_signal(self, nome)\n  if nome == \"salta\" then end\nend\n";
    assert_eq!(ouvidos(src), [("salta".to_owned(), 1)]);
}

#[test]
fn controlos_nada_fora_do_on_signal_nem_noutra_variavel_nem_sem_parametro() {
    // Controlo positivo: a mesma comparação, mas dentro do `update` — não é ouvir.
    assert!(
        ouvidos("function update(self, signal)\n  if signal == \"x\" then end\nend\n").is_empty()
    );
    // Outra variável comparada dentro do `on_signal`.
    assert!(
        ouvidos("function on_signal(self, signal)\n  if self.mode == \"x\" then end\nend\n")
            .is_empty()
    );
    // Um `on_signal` local não é o gancho que o motor chama.
    assert!(
        ouvidos("local function on_signal(self, s)\n  if s == \"x\" then end\nend\n").is_empty()
    );
    // Só o `self`: não há parâmetro do sinal.
    assert!(ouvidos("function on_signal(self)\n  if self == \"x\" then end\nend\n").is_empty());
}
