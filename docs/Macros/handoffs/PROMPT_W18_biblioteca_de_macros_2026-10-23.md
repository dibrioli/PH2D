# PROMPT W18 — a biblioteca de macros: criar, guardar, exportar, importar (2026-10-23)

> Rota «linha reaberta» (`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`):
> `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Macros-Automação && pwd && git branch --show-current`.
> A linha continua depois da W17 (aprovada no smoke; [handoff W17](HANDOFF_INTEGRACAO_line_Macros_2026-10-23_W17.md)).
> O [prompt do fecho da W17](PROMPT_W17_fecho_da_linha_2026-10-23.md) fica para o FIM da W18 (um portão
> batched só sobre as duas), salvo se o Enio mandar integrar a W17 antes — então faça o fecho primeiro.
> Protocolo: `/pd-feature` — **plano primeiro** (doc `docs/Macros/16_W18_plano.md`), produto final, UI
> incluída; ADR novo (formato de ficheiro + portabilidade), porque o ADR-0186 não decide biblioteca.

## O pedido do dono (verbatim, 23/10)

«ainda não temos sistema de criação, salvamento, exportação, importação de macros. Precisamos construir.»

## As decisões do dono (23/10, respostas verbatim)

1. **Para quê:** «As duas coisas» — biblioteca pessoal de receitas **e** ficheiro para enviar a outra pessoa
   (o mesmo ficheiro serve as duas).
2. **O que a macro leva:** «A macro deve envolver o objeto inicial e sua criação. Ou seja, a macro grava a
   criação da imagem ou outro objeto do projeto, por exemplo, uma pintura de 1024x1024 tem na macro a
   criação dessa img de 1024x1024.» ⇒ a macro é AUTOSSUFICIENTE: aplicá-la cria o objeto (tipo, tamanho,
   estado inicial) e toca a criação dele. Não há «esticar à tela de outro tamanho»: o objeto vem com ela.
3. **Aplicar onde já há desenho:** «Escolho na hora» — ao aplicar, o app pergunta: criar o objeto da macro
   (novo) ou tocar os passos POR CIMA de um objeto existente compatível (a decidir no plano o que é
   «compatível», e o que acontece com tamanho diferente nesse caso — pergunte ao dono se o plano não
   fechar sozinho).
4. **Onde:** «lista no painel de macro com a possibilidade de salvar como asset no painel de asset» ⇒ uma
   secção Biblioteca no painel MACROS (guardar, aplicar, renomear, apagar, exportar, importar) **e** «guardar
   como asset»: a macro aparece no painel de Assets (`ph2d-i18n/src/asset_browser.rs` é o ponto de entrada
   das palavras desse painel) e de lá se aplica/arrasta.

## O que já existe (inventário de 23/10 — confira no código antes de construir)

- A macro persiste SÓ como componente: `ph2d_ecs::Macro { doc, modo }` (`crates/ph2d-ecs/src/macro_doc.rs:26`),
  registado em `scene/registry.rs:653`, viaja no `.ph2dproj` pelo `WorldSnapshot` (gate de ida e volta em
  `scene/save_tests.rs:420`).
- O formato já é versionado: `crates/ph2d-macro/src/codec.rs` (`MAGIC = PH2DMACRO`, 1.2, `exige`) — base
  natural do ficheiro próprio (extensão a decidir no ADR, p. ex. `.ph2dmacro`).
- **Não existe:** ficheiro próprio, exportar/importar, biblioteca, aplicar a outro objecto, duplicar.
- O que hoje amarra a macro ao objecto de origem (o que o plano tem de resolver): `Track.target` = `StableId`
  (`ph2d-macro/src/doc.rs:43`); o estado inicial são bytes embutidos (`Track.initial`); o tocador exige
  texturas e presets por NOME (`ErroDoInicial::Texturas`, `inicial.rs:37`; `ErroDaMacro::Preset`,
  `tocar.rs:32`) ⇒ um ficheiro para outra pessoa tem de levar (ou recusar em voz alta) o que não existir
  do outro lado.
- A criação do objecto NÃO é gravada hoje (a gravação começa com o objecto já feito): a decisão 2 pede um
  passo «nasce o objecto X de W×H» no início — Painter e Motion (as duas famílias com trilha).
- Painel MACROS hoje: Gravar, Parar, Renomear, Apagar, Dica, Fechar, edição dos passos
  (`crates/ph2d-panel-macros/src/{ids.rs,seam.rs,corpo.rs}`).

## O que entregar

Plano (com as recusas medidas) → ADR → implementação nas duas famílias → cena de smoke nova
(`PH2D_MACRO_SMOKE=7`, contada no roteiro) que ensina o caminho inteiro: gravar uma pintura, guardar na
biblioteca, exportar o ficheiro, importar noutro projecto, aplicar (as duas escolhas da decisão 3) e vê-la
nascer — fotografada antes de ir ao dono. Handoff no fim; integrar só por ordem do Enio.
