# HANDOFF DE CONTINUAÇÃO — `line/UIUX`, 24/10 — F28: um valor discreto fora do vocabulário do campo

- Branch `line/UIUX`, worktree `Worktrees/line-UIUX`. Base: `main` `c3662cd8e4` (a `line/CodeEditor` já integrada; a colisão do
  `options!`/`CampoInfo` do F27 §3 foi reconciliada na integração por `acab41422a` — nenhuma linha viva a reconciliar). **Sem integrar.**
  Anterior: [F27 F](HANDOFF_CONTINUACAO_line_UIUX_2026-10-24_F27_F.md).
- Censo, veredito dos herdados e recusas: [`../F28_plano.md`](../F28_plano.md). **Não está copiado aqui.**
- Estado: construído e verificado por nome; **smoke do dono (§4) por fazer**. Próxima LLM: janela nova.

## 1 — Commits (`git log --oneline c3662cd8e4..HEAD`)

| commit | o quê |
|---|---|
| `cfd9432574` | `feat`: o valor discreto é preso ao vocabulário do campo (mundo, chave, vetor) + cena 34 + 9 gates |
| `aa2710cfa3` | `test`: o bloco do traço tem buracos de propósito (ajuste de `canal_tests`) |
| (este) | este handoff + F28_plano (F25 §6.9 fechado por desenho) |

## 2 — O que mudou

Família curada num bloco: **um valor discreto fora do vocabulário do campo** (a palavra na dope-sheet mentia, a chave guardava fração
ou índice arbitrário).

### 2.1 A lei: `ph2d_ecs::canal_palavra::prende(k, v)`

- `Opcoes` → índice mais próximo com chave NÃO vazia (`""` é buraco; empate desce).
- `Ligado` → 0/1. `Catalogo` → preso ao número de opções (`canal_componente::n_opcoes`, novo).
- `Camada`/`Script`/`Texto`/`PorIrmao` → inteiro ≥ 0.
- **`Continuo` e `Numero` intactos**: `Numero` guarda âncoras fracionárias da Interface e dominância negativa da física (medido na
  onda: o 1.º rascunho arredondava `Numero` e partia ambos).

### 2.2 Onde se aplica (três fronteiras)

1. `CanaisEscritos::escrever` (`ph2d-ecs/src/canal.rs`) — o funil único do mundo para curva, pilha e expressão.
2. A chave: `TimelineDoc::insert_key`/`upsert_key` e `SetKeyValue` (`intent_apply.rs`) via `PropKind::prende` (`prop.rs`).
3. Fronteira do escritor do vetor: topo de `ph2d_app_vec::canais_campos::escrever`.

### 2.3 Censo A (resumo; tabela no plano)

- (a) **Mistura:** o canal declarava 22 opções e o Vetor desenha 19: os códigos 4/12/13 (Linear Burn, Vivid Light, Linear Light) diziam
  essa palavra na dope-sheet com chip e desenho em Normal. Cura: `MISTURAS_DO_VETOR` (buracos 4/12/13) para `MISTURA` e
  `PILHA_MISTURA` (`canal_vetor.rs`). A mistura de camada do Flip mantém as 22 (o menu dele lista as 22 por código).
- (b) `ENVELOPE_WARP`: o censo errava — o painel já traduz por `nomes_do_motor::efeito`. Só o gate que amarra as três listas
  (`WarpStyle::label`, `chave_do_efeito`, `WARPS`). A ponte texto→chave **continua** (fraqueza anotada, não mudada).
- (c) `TINTA_TRACO`: buracos 1–3 do tipo ficam (espaço de códigos partilhado com o preenchimento: Pattern=4, Brush=5; renumerar =
  formato, recusado). O traço deixou de declarar paragens/pontos/gradientes (canais sem leitor).
- (d) `TEXTO_ALINHAMENTO`/`PILHA_DILATA_JUNCAO` agora presos (200 → 2).

### 2.4 Gates (cada um visto vermelho por mutação: 9/9 sangraram, corridas pelo agente de mutação)

`o_menu_de_mistura_e_a_palavra_dizem_o_mesmo_modo` (app-vec; MISTURA e PILHA_MISTURA → `op(MISTURAS)`) ·
`um_valor_fora_da_lista_cai_na_escolha_mais_proxima` e `a_curva_so_escreve_escolhas_que_existem` (ecs) ·
`a_chave_de_um_discreto_guarda_uma_escolha` (timeline; variantes `SetKeyValue` e insert/upsert) ·
`alinhamento_e_juncao_so_aceitam_as_tres_escolhas` · `o_traco_so_declara_o_que_tem` ·
`os_presets_do_envelope_dizem_a_palavra_do_canal` (app-vec; tirar «Rise»). Ficheiro novo
`crates/ph2d-app-vec/src/vocabulario_tests.rs`. Um vermelho achado e curado (`aa2710cfa3`):
`ph2d-ecs canal_tests::todo_campo_da_tinta_tem_informacao` exigia o bloco do traço sem buracos (lei antiga); o bloco do
preenchimento continua sem buracos, o conjunto exacto do traço é o gate do app-vec.

### 2.5 A cena `PH2D_AUTOKEY_SMOKE=34`

`crates/ph2d-app-vec/src/autokey_smoke_vocabulario.rs` (o despachante em `autokey_smoke.rs` mapeia "34"). Um rectângulo, 19 chaves de
Blend a cada 0,2 s na ordem do menu + uma 20.ª pedida como Vivid Light (12) que guarda Hard Light (11); cabeça parada em 0,8 s
(chave 5 = linha 4 = Lighten). Log medido: «20 chaves de Blend (tem de ser 20) · linhas do menu = código guardado em 19/19 ·
pedido 12 (vivid_light) guardou 11 (hard_light) · relógio em 0.8 s = linha 4 = lighten». Fotos lidas: chip «Lighten» na secção
Appearance e «Light…» na dope-sheet, na mesma chave; com `PH2D_LANG=teste` ambos pseudo-traduzidos (a palavra é chave).

### 2.6 Herdados (veredito no plano §B)

F18–F26 §6.5 fechado (gates em `canais_junta_tests`) · F25 §6.6 desenho · §6.7 curado por esta onda · §6.8 desenho (a lista de
camadas só existe no painel do Flip) · **§6.9 fechado por desenho** (`vec_selection.rs:107`: a seleção sob um Envelope vira o
CONTENTOR sozinho, que entra em `autokey_amostra::populacao`; gate `selecting_an_envelope_child_selects_only_the_container`,
`vec_selection.rs:460`) · §6.11 marcas/menu de grupo/ADR-0191 fechados pela F27 · F18–F26 §6.9 (`canal.junta.*`) não é defeito
(tabela única `ph2d-i18n/src/canais.rs`; o pseudo-locale deriva dela).

## 3 — Superfície de colisão (um merge pode partir)

- Tocado: `crates/ph2d-ecs/src/{canal.rs,canal_palavra.rs,canal_componente.rs,canal_vetor.rs}` (+ `canal_palavra_tests.rs`,
  `canal_tests.rs`) · `crates/ph2d-timeline/src/{doc.rs,intent_apply.rs,prop.rs}` (+ `tests/it/canal_field.rs`) ·
  `crates/ph2d-app-vec/src/{canais_campos.rs,autokey_smoke.rs,lib.rs}` + ficheiros novos `autokey_smoke_vocabulario.rs`,
  `vocabulario_tests.rs`.
- **Semântica:** qualquer linha que escreva `CanaisEscritos` ou chaves com valor discreto fora do vocabulário passa a tê-lo
  preso; uma lista `Opcoes` nova com buracos `""` significa «não é escolha».
- **Formato:** nenhum — `PROJECT_SCHEMA` igual, `TimelineDoc` continua v23, ADR 0, registos de componentes iguais.
- **Delta de contadores:** cenas de smoke +1 (`=34`; o roteador do `CLAUDE.md` §5 diz `1..33`). **Número provisório — o integrador
  reconta** (próxima: 35). Mais nada a recontar.

## 4 — Smoke do dono (em linguagem simples)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && PH2D_AUTOKEY_SMOKE=34 cargo run -p ph2d-host-desktop --profile smoke`
2. Abre com um quadrado azul e a linha do tempo embaixo com a linha «Path 0 · Blend» e 20 losangos; à direita, na secção
   Appearance, a caixa de mistura diz «Lighten».
3. Clique em cada losango (ou use as setas de quadro): a palavra ao lado de «Path 0 · Blend» e a caixa de Appearance dizem sempre
   o mesmo nome. O último losango diz «Hard Light» (foi pedido «Vivid Light», que o vetor não desenha).
4. Deu errado se: a palavra da linha do tempo difere da caixa, ou o último losango diz «Vivid Light» ou um número.

Dica: se a palavra aparecer cortada, alargue a coluna esquerda da linha do tempo.

## 5 — Verificação

- `cargo fmt --all` limpo; `PH2D_SUITE=onda clippy -p ph2d-ecs -p ph2d-timeline -p ph2d-app-vec --all-targets -D warnings` verde.
- Testes por nome verdes: `ph2d-ecs` `canal` · `ph2d-timeline` `canal`/`intent` · `ph2d-app-vec` `vocabulario`/`canais`/`marcas` ·
  `ph2d-app-physics` `canais` · `ph2d-app-flip` `canais` · `ph2d-app-components` `canais`/`palavra` · `ph2d-host-desktop` `autokey`.
- Mutação: 9/9 sangraram (§2.4).
- **Suíte completa NÃO corrida** (dono 23/10: só antes da integração; quem corre é o integrador).

## 6 — ABERTO, por ordem

1. Smoke do dono `=34` (§4).
2. Fora da família, seguem abertos: perf de `linhas::layout`; «gate de fecho da LINHA»; undo da timeline separado do undo do vetor.
3. A ponte texto→chave dos nomes do motor (`nomes_do_motor`, ENVELOPE_WARP): cura futura = o painel receber a chave, não o rótulo.
4. **FORA** (igual à F27): Timers, Tweens, StateMachine, CounterWatch, SignalActions, VecAnchors, ShakeEmitter, resistências,
   `ph2d-panel-painter-layers`, gravidade do mundo, física a meio, IkBendSide, Rope Length, áudio do mixer.
5. Próxima cena: `PH2D_AUTOKEY_SMOKE=35`.
6. **NÃO integrar nem fazer push sem ordem do Enio.** Na integração reconta-se só o número da cena.

## 7 — Report do dono depois do smoke (25/10): a tinta do balde órfã

*«Se uso o balde de tinta para colocar cor numa forma, depois deleto a forma, fica um path no canvas que não aceita ser
deletado.»* Smoke `=34`: **OK** (dono, 25/10).

- **Mecanismo:** o balde insere um `VecPath` novo com `VecBucketFill` (sementes + âncoras de arco nas formas-fronteira,
  sem dono), publicado como DERIVADO (`entities.rs` `view_state`) e por isso fora do `is_pickable`. Apagar a forma não
  tem cascata (`remove_path`, `vec_delete_selected`); o `bucket::upkeep` não reencontra face e CONGELA a tinta (lei de
  2026-09-02, para bordas que mudam de forma) — congelada e inseleccionável para sempre.
- **Cura:** `bucket::upkeep` apaga a tinta que não ganha face E tem uma âncora num caminho que já não existe. A borda que só
  mudou de forma continua a congelar; o undo devolve forma e tinta juntas (instantâneo da cena). Serve o Delete do canvas e
  o da Hierarquia (o `sync` despawna a entidade a seguir).
- **Gate:** `bucket_tests::apagar_a_forma_pintada_leva_a_tinta_do_balde` (com dois controlos: borda inteira fica; borda
  aberta congela). Mutação (`remove_path` dos órfãos → no-op): **vermelho**; restaurado: verde. `bucket` 24/24 verdes,
  clippy `ph2d-app-vec` verde.
- **Smoke do dono:** (1) `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && cargo run -p ph2d-host-desktop --profile smoke`
  (2) desenhe um quadrado no Vetor, pegue o balde e clique dentro dele (3) seleccione o quadrado e apague-o (Delete): a cor
  some junto (4) errado: fica uma mancha de cor que não se seleciona. Ctrl+Z traz o quadrado e a cor juntos.
