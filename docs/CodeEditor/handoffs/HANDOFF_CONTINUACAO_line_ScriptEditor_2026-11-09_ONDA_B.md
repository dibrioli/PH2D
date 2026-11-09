# HANDOFF de continuação — line/ScriptEditor, 09/11: a Onda B (a Bancada) começa aqui

> Leitor: a LLM da próxima janela. Plano: [`13_plano_bancada.md`](../13_plano_bancada.md) §4 Onda B (itens 6–10),
> §2 (a linguagem visual herdada da GEF02) e §6 (decisões D1–D5). Decisões do dono:
> [`00_decisoes_do_dono.md`](../00_decisoes_do_dono.md) linhas 10–11. Último handoff de integração:
> [`HANDOFF_INTEGRACAO_line_ScriptEditor_2026-11-08_ONDAS_A_C_D.md`](HANDOFF_INTEGRACAO_line_ScriptEditor_2026-11-08_ONDAS_A_C_D.md).

## 1. Estado

- `line/ScriptEditor` foi **reposta no `main` f254e42bf1** (09/11) depois da rodada de integração de 6 linhas
  ([`HANDOFF_INTEGRACAO_rodada_2026-11-09.md`](../../IntegracaoMultiAgente/handoffs/HANDOFF_INTEGRACAO_rodada_2026-11-09.md), suíte final 34 117/34 117). Os hashes velhos ficaram em
  `backup/line-ScriptEditor-2026-11-08` (só referência; apagar quando a Onda B integrar).
- Ondas **A, C e D no `main`**, todas com **smoke do dono OK** (D em 09/11: cena 12, passos 1–8, e cena 4 «o erro
  pára o jogo»).
- **A GEF02 está no `main`** (aba Collision Layers, cartões, frase do pé, `ColorToken::layer`) ⇒ a Onda B está
  desbloqueada.

## 2. A tarefa: Onda B, itens 6–10 (do plano, sem re-litigar)

- **D1:** é a Bancada: maquete B da crítica (`/home/enio/Documentos/GameEngineSamples/critica_ph2d.html#editor_scripts`, fora do repo), mais a grelha
  «Who has what» e a frase fixa do pé.
- **D2:** a cor é **por OBJECTO** (não por script). É a mesma na grelha, na margem, no Console e no contorno do canvas.

1. **In this script** (abaixo da lista Scripts): Properties · Functions · Hears, cada um com `ln N`; o clique salta
   para a linha.
2. **Marcas na margem:**
   - ◆ em cada `ph2d.property` e ⚡ na função de sinal;
   - pontos na cor de cada objecto cujo valor difere do padrão;
   - pairar ou clicar abre o **popover «em quem»** (`Hero 220 · default` / `Hero 2 180 · changed`, botão
     «Select Hero 2»).
3. **Who has what:** grelha objectos × propriedades no pé do código.
   - É dobrável, como o cartão «Player · 1 object» da aba de camadas.
   - A célula alterada acende na cor do objecto, e a coluna sob o cursor também acende.
   - Clicar numa célula edita o valor pelo MESMO comando do cartão Script do Inspector, logo com o mesmo undo.
4. **A frase do pé**, fixa e em negrito: o que está sob o rato e o que o clique faz.
5. **Pairar um objecto** (na grelha, no popover ou no Console) acende o contorno dele no canvas.

⛔ O que fica fora está no §5 do plano: minimapa, janela separada e copiar o código da aba de camadas (reusa-se por
`main`).

## 3. O mapa medido (09/11, explorador; confira o código antes de confiar numa ausência)

| Peça | Onde | Estado |
|---|---|---|
| Factos do script | `crates/ph2d-code-syntax/src/facts.rs:73` `declared_properties` · `:82` `property_declarations` · `:105` `symbols` · `:300` `top_function_body` · `:318` `top_statement_line` | existe |
| **Hears** (sinais que o `on_signal` compara) | — | **falta**: um facto novo em `facts.rs` |
| Quem usa o script | `ph2d-script/src/host.rs:373` `objects_using` → `scene_reload.rs:108`; chamada em `ph2d-app-code/src/frame.rs:244` | existe |
| Valor por instância × padrão | `ph2d-script/src/component.rs:52` `LuauScript.own` · `props.rs:164` `PropDecl{name, default, hint}` | existe: diferente = chave em `own` com valor ≠ `default` |
| Editar o valor (o mesmo do Inspector) | `ph2d-panel-inspector/src/event_script.rs:18` `apply_script_event` → `ScriptFieldEdit` (`ph2d-editor-core/src/script_edits.rs:205`) → `ph2d-app-components/src/script_inspector.rs:232` `apply` | existe; **o empilhar do undo NÃO foi confirmado**: ache-o antes de ligar a grelha |
| Margem | `ph2d-panel-code/src/debug_marks.rs:122` `paint_debug_gutter` (+ selo do problema, `worst_problem`) | existe; ◆/⚡/pontos entram aqui sem tapar o ponto de parada nem o selo |
| Popover «em quem» | — | **falta** |
| Grelha | análogo: `ph2d-panel-physics/src/aba/quadro.rs:360` `linha_acesa` · `:364` `coluna_acesa` | **falta** (reusar o idioma) |
| Cartão dobrável «N object» | `ph2d-panel-physics/src/aba/mod.rs:407` `titulo_da_camada` · `aba/camada.rs:41` `monta`, `:160` `pinta` | existe; a função de dobrar não foi localizada |
| Frase do pé | `ph2d-panel-physics/src/aba/mod.rs:379` `frase_do_pe` (pura, devolve `String`) · `:294` `par_do_pe` | existe: o modelo |
| **Cor por objecto** | só há `aba/mod.rs:287` `cor_da_camada(n)` = `ColorToken::layer(n)` (por camada) | **falta**: decisão técnica delegada, ver §4 |
| Contorno no canvas | `ph2d-vec-render/src/hover_outline.rs:32` e `ph2d-app-vec/src/pick.rs:72` (só vetor) · `ph2d-editor-core/src/paint_contorno_camada.rs:44` (contorno de UI) | **falta** o genérico, para qualquer objecto |
| Coluna Scripts | `ph2d-panel-code/src/script_list.rs:114` `width` · `:124` `area` · `:128` `paint` · `:448` `hit` | existe |
| Pé, Console e cartão Stopped | `ph2d-panel-code/src/console.rs`, `console/problems.rs` · `ph2d-app-code/src/console.rs:250..267` · `ph2d-app-code/src/debug_card.rs` | existe; a grelha divide o pé com eles |

## 4. Decisões técnicas a tomar (delegadas, padrão-ouro; registe-as no plano)

- **Cor por objecto:**
  - estável entre sessões, por isso nunca pelos bits da entidade (CLAUDE.md §5.1 Editor: `StableId`/nome);
  - distinta entre os objectos do MESMO script;
  - vinda da paleta de tokens (HR-15: zero hex).
  - Sugestão a medir: o índice na ordem estável dos `objects_using` do script, sobre a paleta `ColorToken::layer(n)`.
  - Verifique o contraste no tema moderno e no clássico.
- **Hears:** o que conta como «ouvir» um sinal (o `on_signal` que compara `name == "x"`?). Defina-o pelo idioma
  real das cenas `ph2d-code-scenes`, nunca por um caso inventado.
- **Contorno no canvas:** um caminho genérico (bbox/forma do objecto pela câmara do canvas). Prove o clique e o
  contorno no ECRÃ, não só no mundo (memória: régua que pára no mundo aprova clique partido).

## 5. Restrições medidas

- **LOC (`architecture_the_shell_only_shrinks`, conta `src`+`tests` recursivo):**
  - `ph2d-app-code` está em 26 530 contra o tecto de 26 628: **98 linhas de folga**. A Bancada NÃO cabe lá.
  - Lógica pura e pintura vão para as crates da família (`ph2d-code-viewmodel` 204/304, `ph2d-panel-code`) ou para
    uma crate nova da família (Folha), com a linha dela na tabela `TETOS_DAS_CRATES_DA_FAMILIA` (a isenção viaja
    com o código).
  - Cada ficheiro tem no máximo 700 linhas.
- **Cena de smoke nova:** `PH2D_CODE_SMOKE=13`. `ph2d-app-code/src/smoke.rs:11` diz `CENAS = 12`, e o teste espera
  `13 → None`: suba os dois. Confira a fonte de `CENAS` em `ph2d-app-components/src/code_smoke.rs`.
  - Atualize o §5.1 para `1..13` só no fecho da linha.
- **UI:**
  - HR-15 sempre; app só em inglês.
  - Tudo em cartões (`docs/CartoesUI/README.md` + receita 11).
  - Hit rect = rect pintado.
- **Maquete:** a foto da cena 13 e a maquete B vão lado a lado ANTES do smoke, acabamento incluído (memória de
  07/11).

## 6. Aberto herdado (não é desta onda, salvo se tocar o mesmo sítio)

Do handoff de integração §6:

1. No quadro da paragem, os relógios de passo fixo da shell avançam todos os ticks desse quadro (cura =
   reordenar `fase_fixed_step_clocks`).
2. Um ponto numa linha vazia depois do último código arma no fim implícito do chunk e nunca pára no Play.
3. O redesenho do tooltip com o rato parado não foi verificado à vista.
4. A paragem em Fullscreen/Window e o Ctrl+/ pelo carácter não têm teste ponta-a-ponta.
5. Acabamentos:
   - larguras por coluna na grelha de variáveis (a grelha «Who has what» resolve o mesmo problema: partilhem a
     regra);
   - o âmbar sai cáqui;
   - F8/F10 ao lado dos botões;
   - os pontos de parada seguem o texto por gravar.

## 7. Como a onda fecha (`/pd-onda-fechar`)

1. `cargo fmt --all`.
2. clippy das crates tocadas.
3. Testes da onda por nome (sem suíte).
4. A cena 13 fotografada (`docs/Components/ferramentas/fotografa_cena.sh PH2D_CODE_SMOKE=13 1930 1040 <png> 8`) e
   comparada com a maquete.
5. O binário `--profile smoke` compilado 2×.
6. Commit e este handoff atualizado.
7. O smoke ao dono em passos numerados.

Não integre, não pushe.
