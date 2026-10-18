# Oráculos de explicação de erro (I8, 17/10)

Pergunta: algum programa já põe uma EXPLICAÇÃO ao lado da mensagem de erro? Resposta e tabela em
[`../medicoes_onda2/I8_explicacoes.md`](../medicoes_onda2/I8_explicacoes.md) §4; saída crua em
[`../medicoes_onda2/I8_oraculos_bruto.txt`](../medicoes_onda2/I8_oraculos_bruto.txt).

Correr (Godot 4.7.2, `luau-lsp` fixado, `rustc`, `python3`, `lua`, `luajit` instalados):

```bash
P=$(mktemp -d) && cp -r docs/CodeEditor/ferramentas/oraculo_explicacoes/entradas/* "$P"/
bash docs/CodeEditor/ferramentas/oraculo_explicacoes/corre_tudo.sh "$P" > /tmp/I8_oraculos_bruto.txt
```

As entradas são nossas (os mesmos 8 erros em GDScript, Luau, Rust, Python e Lua). A secção dos
links do `luau-lsp` lê `web/lint.html.txt`, o texto de `https://luau.org/lint` (não guardado aqui:
descarregue-o para `"$P"/web/` antes, ou essa secção sai vazia). `godot_lsp.py` é o cliente LSP do
editor headless (o `.godot/` é recriado pelo Godot).
