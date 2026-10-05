# HANDOFF DE INTEGRAÇÃO — `line/motion-value`, 2026-10-05 (O ABERTO DO PASSE DE FORMAS: MUTAÇÕES, A DOBRA, O APP, A EMISSÃO TRACEJADA)

> Leitor: o agente integrador e a próxima janela desta linha. O plano, as tabelas e os veredictos vivem no
> [doc 121 §9.16–§9.17](../121_as_formas_na_placa.md); aqui fica o que um merge precisa de saber. O handoff anterior
> (03–04/10) já está no `main`.

## §0 — IDENTIDADE

| | |
|---|---|
| ramo · worktree | `line/motion-value` · `Worktrees/line-motion-value/` |
| merge-base | `5d596eaaf` (= `main` quando a onda começou; nenhum rebase foi preciso) |
| HEAD | o commit deste handoff |
| commits da onda | `81df613b3` memória PT-BR · `071e56a5d` sonda intercalada (§9.16) · `08c58fd87` regra de medir · `bfad80ea1` índice DevOps · `014c93238` handoff de continuação · `64b08daab` plano §9.17 · `3609f28d2` construção (candidatos + portas) · `353156e55` critério da série G32 · `4f7f511ad` resultados · `df46623f5` a DOBRA · `f42e1b618` mutações + memória · este |

## §1 — O QUE A LINHA ENTREGA

- **(b) Os pedaços do §9.15 dobrados** (`A1a` ajuste na contagem · `A1b` total no percurso · `B1` prefixo das arestas
  ESCRITAS · `B2` junta uma vez por troço): `override` → código fixo, ramos `false` apagados, e o recurso «ajuste = 0 ⇒
  calcula-o aqui» do `percorre` (morto desde o A1a). Decidido por **deixar-um-de-fora** na sonda intercalada: tirar
  qualquer um piora as tracejadas (até `+12,6 %` na RTX). Registos: `16/17` shaders iguais ao `F` por `override`.
- **(d) A emissão tracejada medida até ao fundo e os quatro candidatos RECUSADOS** (`L` o adiado no laço · `H` a
  geometria por troço · `P` as arestas por peça · `G32` grupos de `32`, este também na série justificada das
  contínuas: `−2,4 %` contra `≥ 3 %`). As ablações partem os `0,160` ms da emissão (iGPU): `0,066` andar o laço dos
  pedaços, `0,094` a geometria e as arestas. Zero derrames e `112` VGPRs não mexem no tempo (duas ondas). A alavanca
  que resta é o paralelismo dentro de uma cópia — topologia já recusada (§9.14); NÃO reconstruída.
- **(c) O app medido:** a `=127` densa tracejada passa de `128` para **`64` MB** de arestas com os pedaços do §9.15
  (reservadas `4 507 648 → 2 544 640`), as duas placas; a contínua `32` MB; parede presa a `60` fps.
- **(a) Mutações dos pedaços do §9.15: `7/7` sangraram** (m6 na iGPU, subgrupo `64`).
- **(e) Registos** `base → F`: só o `cs_escreve` completo (`30 140 → 47 112` B, `12 → 28` derrames — o A1b inlinava o
  `emite_pedaco` duas vezes) e o `cs_conta` completo (`3 964 → 4 384` B) mudam; o `F` dobrado `46 216` B, `26`.
- **Instrumentos:** a sonda intercalada imprime `conta` e `escreve` à parte, as arestas reservadas · escritas e a
  imagem de cada variante contra a do `F` byte a byte; `mede_formas_na_placa.sh` SEM espera de calma e com
  `VARIANTES`/`TRACEJADOS`; `mede_intercalado.sh` aceita (e diz alto) o SIGSEGV da RTX depois do `test result: ok`.

## §2 — SUPERFÍCIE DE COLISÃO

| ficheiro | o quê |
|---|---|
| `crates/ph2d-shape-gpu/src/contorno.wgsl` | os `override` do §9.15 dobrados (`−40` linhas) |
| `crates/ph2d-shape-gpu/src/contorno.rs` | sem `compactas`; o `garante` SAIU (`695 → 624`) |
| `crates/ph2d-shape-gpu/src/contorno_capacidade.rs` | o `garante` mudou-se para cá + a linha `[formas] arestas: capacidade …` (`73 → 144`) |
| `crates/ph2d-shape-gpu/src/pass.rs` · `lib.rs` | `CONSTANTES_DO_CONTORNO` (vazia) e `VarianteDoPasse::do_ambiente` |
| `crates/ph2d-app-motion/src/motion_shape_placa.rs` | sem variante de sonda, a placa nasce com `do_ambiente()` |
| `crates/ph2d-app-motion/src/motion_shape_placa_gpu_intercalada_tests.rs` | variantes `F` · `F-D` · `F-c2`; colunas novas |
| `docs/Motion Nodes/…` · `project-memory/…` | doc 121 §9.17, ferramentas, este handoff, uma memória |

- **Foundational tocado:** nenhum. **Shell (`shells/desktop`):** não tocada (delta `0`). **Contratos congelados:** nenhum.
- **API pública nova** (`ph2d-shape-gpu`): `pub const CONSTANTES_DO_CONTORNO: &[&str]` · `VarianteDoPasse::do_ambiente()`.
- **Env nova (porta de medição):** `PH2D_FORMAS_CONSTANTES=NOME=v,…` — lida ao criar a placa do app; nomes fora da
  lista vão a `stderr` como «ignorada». Vazia/ausente = o produto. Sem custo em execução.
- **Ids/consts:** os `override` `AJUSTE_NA_CONTAGEM` · `TOTAL_NO_PERCURSO` · `ARESTAS_COMPACTAS` · `JUNTA_UMA_POR_TROCO`
  SAÍRAM (quem os passar por `com_constantes` recebe erro de pipeline do wgpu: constante inexistente).
- **Itens partilhados apagados:** nenhum. **Listas/catracas baixadas:** nenhuma.

## §3 — ⚠️ O que uma leitura rápida do diff entende ao contrário

- O `contorno.rs` encolhe `71` linhas mas nada se perdeu: o `garante` é o MESMO texto em `contorno_capacidade.rs`
  (submódulo `capacidade`, `pub(super)`), com uma linha de relato a mais.
- `CONSTANTES_DO_CONTORNO` vazia não é resto: é a porta do PRÓXIMO candidato (`MEDIR_VELOCIDADE.md`: peça nova nasce
  `override`). A porta `PH2D_FORMAS_CONSTANTES` mediu o A/B do app com UM binário.
- ⚠️ **Na RTX o `A1b` muda `183` B da camada nas densas tracejadas** (o `F−A1b` e o `base` diferem do `F` pelo mesmo):
  o total de um fechado somado no percurso é a mesma soma termo a termo, mas o compilador da NVIDIA funde-a de outro
  modo e uma emenda no limite vira. Os gates contra o Vello passam; na iGPU é igual byte a byte.

## §4 — ⛔ Premissas do briefing que a medição derrubou

- «A próxima alavanca das tracejadas é a EMISSÃO por peça»: é, mas **não se encurta por arrumação** — quatro
  arrumações (inline único, hoisting por troço, arestas por peça, grupo de `32`) tiram `≤ 0,014` ms dos `0,244`.
- «Os registos do `cs_escreve` completo pesam»: com `72` cópias (duas ondas) `0` derrames e `112` VGPRs não mudam
  o tempo.
- O critério do (b) que eu escrevi («tirar não melhora nenhuma cena `> 2 %`») tirava o `B1` — era cego à TROCA entre
  cenas (`−2,2 %` conformes contra `+16 %` densas). Ficou o critério do §9.15; a falha está escrita no doc e na
  memória [`feedback_a_leave_one_out_criterion_is_blind_to_a_trade_between_scenes`](../../../project-memory/feedback_a_leave_one_out_criterion_is_blind_to_a_trade_between_scenes.md).

## §5 — A PROVA DE FECHO (corrida nesta árvore, HEAD `df46623f5`, base `5d596eaaf`)

`nextest-impacted` **`15 713/15 713`** · `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` ✓ · clippy
`-D warnings --all-targets --all-features` de `ph2d-shape-gpu`, `ph2d-gpu`, `ph2d-app-motion` ✓ · `fmt --check` ✓ ·
`machete` ✓ · `check-standalone-optional` ✓ · `check-workflow-packages` ✓ · `ph2d-shape-gpu --test it --ignored`
**`14/14` iGPU e `14/14` RTX** · `motion_shape_placa --ignored --skip sonda_` `5/5` nas duas · `ph2d-gpu-cook formas`
`2/2` nas duas · tecto de LOC ✓ · censos `114/114`. Mutações `7/7` (§1). Depois de `df46623f5` só docs e memória.

## §6 — ⏳ O QUE FICA ABERTO

| item | o endereço |
|---|---|
| ⛔ a emissão tracejada por cópia no proxy de telemóvel | no fundo do modelo «um fio por cópia» (§9.17 d); só o paralelismo dentro da cópia a move, e o GRUPO por cópia está recusado (§9.14). Quem voltar, mede primeiro, por ablação, a emissão dupla contra a fase em série |
| o SIGSEGV da RTX ao sair de um processo com dezenas de pipelines | com `13` variantes volta mesmo com `__GL_SHADER_DISK_CACHE=0`; os dados saem antes; o roteiro di-lo |
| a mordida do traço rente depois de uma quina | divergência DECLARADA (§9.9), sem acção |
| `fk.rs` duplicado (bug #11) · o contacto dos colisores na placa | fora de escopo (Física / wave própria) |

## §7 — SMOKE (o dono)

Binário compilado nesta árvore (`rm -rf target/*/incremental` e `bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke` 2×; a 2.ª saída):

```
    Finished `smoke` profile [optimized] target(s) in 0.21s      (zero «Compiling»)
```

Fotografado a `1930 × 1040` (RTX): as estrelas amarelas esticadas com o contorno azul tracejado, os cartões Falloff ·
Vortex · Attractor · Curl Noise · Integrate · Duplicator · Scale · Output; `59`–`60` fps; `[formas] 1024 copias … pela
PLACA (do dispositivo)`, arestas `4` MB.

1. No terminal:
   ```
   cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && env PH2D_GPU_COOK_DEMO=127 PH2D_TRACO_ESTICADO_TRACEJADO=1 cargo run -p ph2d-host-desktop --profile smoke
   ```
2. Não precisa clicar em nada: abre a cena das estrelas amarelas com o contorno azul em traços, a mexerem com a
   simulação (os cartões Vortex, Attractor e Curl Noise no grafo de baixo).
3. Tem de acontecer: igual ao smoke de 04/10 — a cena certa desde o primeiro instante, os traços certinhos, nada a
   piscar, a barra de baixo a ~60 fps. Esta onda não mudou o que se vê; mudou a memória que a cena usa (metade).
4. Deu errado se: algum traço faltar, ficar torto ou piscar, a janela engasgar ao abrir, ou a barra cair muito abaixo
   de 60.

## §8 — A UMA LINHA para o `CLAUDE.md` §5 (o integrador aplica)

Troca só o link do handoff na entrada **Motion Nodes**: `Último: [handoff 05/10](docs/Motion%20Nodes/handoffs/HANDOFF_INTEGRACAO_line_motion_value_2026-10-05.md)`. A frase do módulo não muda.

## §9 — PERFIL DO LOOP DO AGENTE (`bash scripts/agent-loop-profile.sh`, verbatim)

```
  ✗ paralelismo de ferramenta              1.13/passo   alvo: >= 1,5  (8% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                296   alvo: <= 800  (uma janela nova por onda de trabalho)
  ✗ cargo test : cargo check                729 : 234   alvo: <= 1,0  razao 3.1x (baseline: 4,3x)
  ✗ edicoes pela ferramenta Edit                  33%   alvo: >= 80%  (1788 por script; baseline: 48%)
  ✗ contexto relido por passo (media)         460 mil   alvo: <= 250 mil  (set/2026: 606 mil — 82% do custo)
  ✓ contexto no inicio da sessao               63 mil   alvo: <= 80 mil  (02/10: 380 mil, CLAUDE.md a 710 KB)
```
