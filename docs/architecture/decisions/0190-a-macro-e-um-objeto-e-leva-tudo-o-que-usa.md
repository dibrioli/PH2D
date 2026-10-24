# ADR-0190 — A macro é um OBJECTO e leva tudo o que usa

- **Status:** Accepted — o PRODUTO é do DONO (23/10, decisões 1–8 em `docs/Macros/16_W18_plano.md` §0); o desenho
  técnico é delegado.
- **Data:** 2026-10-23
- **Linha:** `line/Macros-Automação` (W18)
- **Amparo:** [plano W18](../../Macros/16_W18_plano.md) · [ADR-0186](0186-a-macro-grava-comandos-resolvidos-no-relogio-da-macro.md)
  (o formato e as famílias) · ADR-0164 (a receita: `MasterRoot`).

> Número recontado na integração (24/10): máximo do `main` = `0189`; a `line/UIUX` já usa `0191` contando
> com este. Fica `0190`.

## Contexto

Até à W17 uma macro vivia SÓ como componente do objecto gravado: sem ficheiro, sem biblioteca, sem maneira de a
aplicar noutro sítio. O dono pediu «criação, salvamento, exportação, importação» e decidiu (23/10): o mesmo
ficheiro serve a receita pessoal e o envio a outra pessoa; **a macro leva o objecto inicial e a sua criação**;
aplicá-la pergunta «novo ou por cima»; por cima de uma imagem de outro tamanho a pintura fica **no tamanho
original, onde o gizmo do objecto Macro está**, e entra na imagem de baixo; **a macro é um objecto completo e
independente**, criado pelo «+» da Hierarchy, com o gizmo do vazio.

## Decisão

1. **Objecto Macro = `Transform` + `Name` + `Macro`**, sem marcador próprio. A lei da gravação passa a «o objecto
   que tem a `Macro` e os filhos dele»: cada trilha de família aponta o filho pelo `StableId` (`Track::target`) e
   leva a **ficha de nascimento** (`Track::nasce`: instante, nome, transformada local, pai). O tamanho e o
   conteúdo inicial ficam no estado inicial da família — uma verdade só. Uma sprite com `Macro` (antes da W18) é
   um objecto Macro que é a sua própria imagem.
2. **Formato:** `Track` etiqueta 8 (`nasce`), `exige "objeto"`, MINOR 3 — só quando usado (as gravações sem ficha
   continuam 1.0–1.2 ao byte). **Ficheiro `.ph2dmacro`** = o mesmo codec + duas etiquetas de topo só do ficheiro
   (4 modo, 5 nome). O `PROJECT_SCHEMA` não se mexe.
3. **Autossuficiente e PROVADO ao exportar:** os presets e as texturas do Painter já vão nos passos e no inicial;
   o Motion leva só o grafo do dono (`transpor::so_do_dono`: ids remapeados, recusa nomeada a qualquer fio de
   outro dono); a macro inteira toca até ao fim antes de se escrever o ficheiro — falhou, não se escreve.
4. **Duas casas:** a biblioteca do utilizador (pasta `$XDG_DATA_HOME/ph2d/macros`, um ficheiro por macro, o nome é
   o do ficheiro) e os assets do projecto (a **receita**: `Name` + `Macro` + `MasterRoot`, pela porta do *Make
   Prefab*; o índice mostra-a como `AssetKind::Macro`, derivado).
5. **Aplicar: uma porta, dois modos.** *Novo*: nasce o objecto Macro e cada filho pela porta de produto da
   família, o editor fica com o FIM da reprodução. *Por cima*: Painter — o inicial passa a ser a imagem alvo
   (`sobre_documento`, etiqueta 6 = o tamanho gravado) e um **`Encaixe`** afim px-gravado → px-alvo, lido da
   transformada VIVA do objecto Macro a cada reprodução, entra na porta única das posições e na do raio do pincel
   (`authored_spec`); Motion — os nós da macro juntam-se aos do objecto (`transpor::aplica`), os dele intocados.

## Alternativas recusadas

| Alternativa | Porquê não |
|---|---|
| Marcador `MacroObjeto` | duas verdades para «é macro?» |
| `w×h` no cabeçalho da macro | o tamanho já está no inicial da família |
| Loja própria de assets de macro | o índice é derivado do mundo; a receita já viaja no projecto |
| Exportar o inicial do Motion como está | é o documento do PROJECTO inteiro |
| Esticar a pintura à imagem de baixo | decisão do dono: tamanho original, onde o gizmo está |
| Assar o encaixe ao aplicar | mover o objecto Macro tem de mudar a reprodução seguinte |

## Consequências e o que fica aberto

- **W18 grava uma imagem e um Motion por objecto Macro** (o palco do Painter toca a 1.ª trilha da família): mais
  é recusado em voz alta (`one_per_family`).
- **A família «mundo»** (pôr um collider, simular a física — decisão 7: «tudo pode ser macro») é a W19: a
  reprodução de um objecto que ainda não nasceu precisa de plano e medição próprios. Até lá o painel diz o que não
  ficou gravado.
- O encaixe escala o raio do pincel; o ângulo fixo do dab, a inclinação da caneta e os outros ajustes em px não
  seguem a rotação/escala do encaixe (medido no gate `por_cima_com_rotacao_e_escala` só para a espessura).
