# ADR-0188 — O editor de scripts mora no PH2D, e o script mora no projeto

- **Status:** Accepted — o PRODUTO é decisão do DONO (15/10: script *«dentro»* do projeto · `luau-lsp`
  empacotado *«sim»* · IA *«por enquanto desligada; depois da forma mais completa e poderosa possível, no fim»*
  · referência a objetos *«sim… e tudo que torne tudo mais fácil, rápido e intuitivo»*); o desenho técnico é
  delegado.
- **Data:** 2026-10-15
- **Linha:** `line/CodeEditor` (Onda 0)
- **Amparo:** [relatório-plano](../../CodeEditor/reports/Editor%20de%20scripts%20estado%20da%20arte.md) (estado da
  arte com fontes, medições do `luau-lsp`) · [decisões do dono](../../CodeEditor/00_decisoes_do_dono.md) ·
  [Onda 0](../../CodeEditor/01_onda0.md).

> ⚠️ **Número PROVISÓRIO** (§5.0): contado em 15/10 contra o máximo do `main` (`0186`) e a `0187` já tomada na
> `line/MiroClone` (`ea06f4f2d1`). Quem integrar reconta-o.

## Contexto — a força que obriga a decidir agora

O `ph2d-script` corre Luau (mlua 0.12, sandbox, recarga por hash, `self` preservado), mas o texto é editado
FORA do app, e o plano que o entregou excluiu por escrito um editor interno
([`13_plano_script_properties.md` §3.9](../../Components/13_plano_script_properties.md)). Medido em 15/10: o erro
chega ao Inspector como `String` cortada na 1.ª linha e sem linha/coluna (`module.rs` `runtime_message`); o
`ph2d.d.luau` gerado descreve uma API que a VM não tem (`scene.*`, `message.*`) e omite a que tem (`set`, `get`,
`property`, `motion.*`); o caminho do script é lido relativo ao cwd do processo; a HR-17 não tem executor; o
mlua NÃO traz o Analysis do Luau (mlua#543), e o `luau-lsp` 1.70.1 responde em ≤ 1 ms e rechecha 7 500 linhas em
~30 ms sobre definições nossas.

## Decisão

1. **O editor de scripts é parte do PH2D**, num documento em aba (como os quadros), aberto por «Edit script»
   no cartão Script. Revoga a linha «Editor de texto dentro do app» do plano 13 §3.9. Todo o código do
   editor fica atrás da feature `editor` (HR-7): o jogo de release não o leva.
2. **O texto do script é DADO DO PROJETO.** Uma `ScriptLibrary` (crate de dados pura, sem mlua) guarda
   `ScriptId → { nome único, texto }`, gravada no `.ph2dproj` como blob com versão própria (molde do `Macro`:
   cabeçalho + major/minor, leitura tolerante). O `LuauScript` deixa de guardar um caminho e passa a apontar
   um `ScriptId`; vários objetos partilham o mesmo script. Projetos antigos importam o ficheiro do caminho no
   load; caminho em falta = recusa legível no cartão, NUNCA script vazio. Importar e exportar ficheiro
   continuam possíveis. O texto fica FORA do `ProjectState` (molde `motion`/`timeline`): o editor tem histórico
   próprio por transações, e o diff global por quadro não vê cada tecla. Degrau novo do `PROJECT_SCHEMA`.
3. **A API Luau tem UMA fonte: a tabela declarativa em `ph2d-script`**, verificada contra a VM real por
   introspeção (as chaves das tabelas `ph2d`/`motion` montadas pela mesma fiação = as da tabela, nos dois
   sentidos). Dela saem o `ph2d.d.luau` (sintaxe `declare <global>: {…}`, estável) e a documentação. A HR-17
   passa a ser executada pelo `luau-lsp analyze --definitions` fixado sobre os exemplos, com uma fixtura de
   erros de propósito como controlo.
4. **A inteligência vem do `luau-lsp` fixado, como PROCESSO-FILHO empacotado** (stdio, versão e sha256 por
   sistema num manifesto). Não é plugin de runtime (ADR-0075): não carrega código no processo, não estende o
   PH2D, é uma ferramenta com versão fixa como o compilador. Plano B (shim C++ do Analysis numa biblioteca
   com símbolos ocultos, só diagnósticos) apenas se o spike da Onda 0 reprovar. Proibido: ligar estaticamente
   um segundo Luau ao lado do do mlua (ODR).
5. **Um script pode apontar outro objeto** — revoga a regra «scripts só falam por sinais» (handoff SCRIPT
   16/09 §9). A referência é durável (`StableId`, resolvida por nome com o editor a verificar o nome), e
   arrastar da Hierarchy/Inspector/assets gera código tipado. Desenho na Onda 3.
6. **IA: zero superfície até a última onda**, que a entrega completa e sempre verificada contra o
   `ph2d.d.luau`, com prévia e um passo de undo.

## Alternativas recusadas

| Alternativa | Por que não |
|---|---|
| Ficheiro solto ao lado do projeto (o de hoje) | Decisão do dono; mover o ficheiro quebra a ligação e o projeto não viaja inteiro. |
| Texto do script como campo do `LuauScript` | Duplicaria o texto por objeto e poria cada tecla no diff global do undo. |
| Analysis do Luau no processo agora | 28,7 s de build C++, reescrever o `luau-lsp`, risco de ODR com o Luau do mlua — sem ganho medido. |
| Portas do Luau em Rust puro (`luau-rs`, `ruau`, `luaur`) | Experimentais em 10/2026 («not ready for production»); reavaliar contra o `luau-lsp` como oráculo. |
| Copiar o editor do Zed (`rope`/`text`/`editor`) | GPL-3.0: só clean-room a partir dos blogs; o `sum_tree` (Apache-2.0) pode ser vendorado com NOTICE. |
| `parley::PlainEditor` como motor do editor | Uma string, um estilo, relayout inteiro por mudança: só serve campos de uma linha. |

## Consequências

- `PROJECT_SCHEMA` sobe um degrau (formato gravado ⇒ suíte imprescindível no fecho).
- O `ph2d.d.luau` deixa de sair do catálogo MCP; a metade MCP do `ph2d-bindgen` (HR-10) fica como está.
- O orçamento do script passa de relógio de parede a contagem de instruções (Onda 4) para a máquina do tempo
  ser reprodutível.
