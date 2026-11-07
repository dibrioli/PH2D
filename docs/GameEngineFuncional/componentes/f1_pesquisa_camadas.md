# Com quem este objeto colide: pesquisa e estado (07/11/2026)

[F] documentação ou código · [O] opinião da comunidade · [I] inferência. Nenhum resultado veio do Reddit. **Não existe
enquete, estudo de uso ou contagem de erros:** «mais bem aceito» aqui é opinião informada.

## O que o app tem (conferido no código)

- Matriz triangular simétrica de 36 células, global: `crates/ph2d-panel-physics/src/cards.rs:344` (`cartao_das_camadas`),
  portada para cartões em `ab1c5c7dec`, sem remoção. O Inspector do corpo nunca teve quadros, em nenhum commit achado.
- `MAX_LAYERS = 8` (`ph2d-physics/src/world/layers.rs:49`). **Não há medição:** a nota diz que o limite é a legibilidade da
  matriz no painel. O motor permite 32. Subir mexe em ids fixos (36 células, 8 slots), `[u8; N]` gravado, canais e testes.
- Camada única por objeto; sem máscara por objeto. `groups_for` traduz para `InteractionGroups` (AND).

**Depois desta pesquisa (07/11):** o dono decidiu matriz global + fila no objeto e 32 camadas; `MAX_LAYERS` subiu para 32 (ficha f1, onda 5). A secção acima descreve o estado de antes.

## Os modelos [F]

| Motor | Modelo | Interface | Entre A e B | Teto |
|---|---|---|---|---|
| Godot 4 | layer + mask por objeto | grade por objeto, nomes no projeto | direcional | 32 |
| Unity 2D | camada + matriz global; Include/Exclude por colisor desde 2022.2 | matriz + listas | conflito por prioridade; empate sem contato | 32 |
| Unreal | Object Type + Ignore/Overlap/Block + presets | preset; tabela ao abrir | Ignore vence | 18 custom |
| Cocos | matriz que inicializa group + mask | matriz | AND | ~32 |
| Roblox | um grupo por parte | Lista e Tabela dos mesmos dados | checkbox por par | 32 |
| Defold, GDevelop | group + mask | nomes / não verificada | mútua | 16 |
| O3DE | layers + grupos com nome | painel central | mútua | 64 |
| Box2D 3 | category + mask + groupIndex | código | ambos os lados | 64 bits |
| Rapier | memberships + filter | código | AND ou OR; se diferirem, AND | 32 |

## O que se diz [O]

- Godot: [#2775](https://github.com/godotengine/godot-proposals/issues/2775) (58 👍 na assimetria; «the current way it works
  confuses people»; 13 👍 a renomear mask para «interacts with»; contra: com dezenas de objetos, repete-se trabalho).
  [#3032](https://github.com/godotengine/godot-proposals/issues/3032): sugestão «checks for collisions with».
- Plugin [Collision Presets](https://store.godotengine.org/asset/adrien-lucas/collision-presets/): em projeto grande, gerir camadas
  «quickly becomes error-prone».
- Unity: [overrides](https://discussions.unity.com/threads/2d-physics-in-unity-2022-2.1363719/): «finally an intuitive solution».
  Queixa: [física e render ligadas](https://discussions.unity.com/t/separate-physics-and-rendering-layers/683444).
- Unreal: canal novo com padrão Ignore faz o [trigger não disparar](https://answers.unrealengine.com/questions/232105/view.html).

## Conclusões [I]

- **Convergência:** por baixo, «sou × aceito» por objeto; por cima, cada família caminha para o meio (exceção por objeto na Unity,
  presets no Godot, conjuntos nomeados em Unreal e O3DE).
- Nomes vencem números. Matriz é boa como mapa do projeto e ruim em cartão estreito.
- Matriz + visão a partir do objeto: não encontrada em nenhum editor. É ausência na busca, não prova.
- **Nosso 8 está abaixo de todos.** Norma 32.
- **O Rapier não faz o Godot sozinho (pela doc, não corrido):** com AND, A ter B e B não ter A dá nada; com OR, os dois colidem.
  Empurrão de um lado só já existe: Dominance. Exceção «ignorar» cabe no AND; «incluir» pediria hook, a testar antes de prometer.

Fontes completas: `git log` deste ficheiro; o relatório da pesquisa foi condensado aqui.
