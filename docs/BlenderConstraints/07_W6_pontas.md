# W6 — as pontas abertas da W1–W5 (27/10/2026)

> Ordem do dono 27/10: *«entre em loop de implementação e com agentes ajudantes resolva tudo. Smoke só ao final de
> tudo após auditoria e correções. Não pare até acabar.»* Merge-base `c3662cd8e4`. Cena de smoke nova = **12**.

## 0. As seis pontas (fonte: handoffs W3, W4.5, W5.4, W5.5)

| # | Ponta | Onde mora | Desenho (decisão técnica) |
|---|---|---|---|
| 1 | Duplicar um canvas não remapeia os `StableId` do `UiArranjo` | `ph2d-ecs/src/hud_arranjo.rs`, `ph2d-app-components/src/instance_refs.rs` (`REMAPPERS`), `instantiate.rs:304` | remapeador do `UiArranjo` na tabela única; o descritor declara a referência (o censo `every_remapper_names_a_declared_object_ref` vê-a) |
| 2 | `Comportamento::pico` (Atrair) sem campo no cartão | `ph2d-app-constraints/src/campos_comportamento.rs`, `ph2d-ecs/src/canal_constraint.rs` (canal 52) | campo Metros como o irmão `RAIO`; Auto Key pelo canal |
| 3 | Juntar uma regra numa componente de 200 pontos: 17–33 ms no quadro | `ph2d-vec-constraints/src/{lib.rs:a_mais,posto.rs}` | MEDIR a divisão (`Sistema::monta` × posto) primeiro; a cura tem de dar a MESMA resposta booleana que o `a_mais` de hoje (que fica como referência nos testes). Cache da base do `CadCores` recusada no desenho: a pose muda a cada passo do arrasto e o Jacobiano depende dela |
| 4 | A mola não reage a arrastar o corpo com a timeline parada | `ph2d-app-constraints/src/relogio.rs:100`, `ph2d-constraints-ecs/src/relogio.rs:96` | idioma Spine 4.2: com a timeline PARADA, a mola corre AO VIVO (relógio de parede) sobre a pose MOSTRADA, só no preview, partindo do estado determinístico do quadro; tocar/arrastar o tempo continua o caminho puro `(doc, t)` |
| 5 | Prender a ponto de simulação (sem id por ponto) | `ph2d-node-motion-emitter` (coluna `id`), `ph2d-eval-motion` `MotionCookPump`, `Kind::PontoDeForma` | levar o `id` da partícula até à saída do cook; `Kind::PontoDeSimulacao`; partícula morta ⇒ item inactivo |
| 6 | Toast «Signal: tique» a cada tique da cena 11 | `shells/desktop/src/render_loop/fase_signal_outbox.rs:145` | a decidir com o código lido: opção por Timer (cena 11 cala o seu) — sem crescer a shell |

## 1. Regras da onda

Gates red-first por nome; suíte completa só na integração; clippy só no fecho; shell a 1 linha do tecto
(`189 040`/`189 041`) — a cura é mover; `shape_handles_live.rs` a 700 linhas exactas.

## 2. Ponta 3 — medido

**Divisão (antes da cura):** o posto é ~98 % do custo. `Sistema::monta` 0,13–0,15 ms, resíduos+linhas 0,10–0,12 ms,
componente 0,05–0,09 ms; o Gram–Schmidt denso duas vezes (`r²n`, base ortonormal DENSA: a base de uma cadeia enche-se
toda) 13,9–14,5 ms. ⇒ a cura é no posto, não na montagem.

**Cura:** `posto::Posto` = QR de Householder **esparsa com pivô** das linhas aceites (como colunas). A linha nova
(normalizada, como antes) passa pelos reflexos (`Qᵀl`) e a norma do que sobra FORA dos pivôs é **exactamente** a
distância ao espaço das aceites — a mesma grandeza que o Gram–Schmidt mede, com a mesma `TOL_POSTO = 1e-8`. Um reflexo
só se aplica se tocar o suporte do momento (fila por variável, em ordem: o salto é exacto, produto zero); linha abaixo da
tolerância não cria reflexo (nada de direcção inventada); regra recusada desfaz os reflexos das irmãs. `estado()` usa a
mesma rotina. O denso fica como `PostoDenso` (`#[cfg(test)]`), a referência.

**Gates** (`crates/ph2d-vec-constraints/src/posto_tests.rs`): `the_sparse_rank_gives_the_dense_answer_on_every_fixture_scene`
(18 cenas da W4 — as de 800 ficam de fora, o denso custa minutos em `dev` —, na pose do desenho e na resolvida: as
MESMAS redundantes e, linha a linha, a mesma distância a `1e-11`) e `..._for_random_new_rules` (`a_mais` de 104 regras
sorteadas, semente fixa, de todos os tipos, mais 64 de cópias e «tirada e reposta», em escada/misto 50 e 200; vê os dois
vereditos). Mutações: `TOL` 1e-8 → 0,3 sangra os dois; esquecer o reflexo seguinte ao enchimento (`k + 1` → `k + 2`)
sobreviveu ao veredito e sangra a régua da distância (foi por isso que ela entrou).

**Medido** (perfil `smoke`, denso × esparso intercalados no MESMO processo com a ordem rodada, mínimo de 9 rodadas,
ms; `loadavg` 19,4 em 32 núcleos — outras linhas a compilar):

| cena | `a_mais` denso | `a_mais` esparso | posto denso | posto esparso |
|---|---|---|---|---|
| escada 202 | 14,90 | **0,45** | 14,54 | 0,15 |
| misto 200 | 14,25 | **0,87** | 13,87 | 0,52 |

No editor (`cad_medida_tests.rs` `the_worst_drag_step_in_the_editor`, 7 rodadas, mínimo, `loadavg` 19,4; a W4.4 a load 43
na coluna «antes»):

| cena | juntar uma regra antes → agora | `a_mais` cru agora | análise das cores antes → agora |
|---|---|---|---|
| escada 202 | 17,3 → **0,47** | 0,44 | 43,5 → 9,0 |
| misto 200 | 33,1 → **1,32** | 1,28 | 45,6 → 12,5 |

⇒ juntar uma regra a 200 pontos cabe no quadro em que a guia acende, abaixo do critério de 2 ms do arrasto. ⚠️ O misto
mede 0,87 ms numa sonda e 1,28 ms na outra (binários e ordem de corrida diferentes; o `a_mais` cru acompanha a porta do
editor, então a diferença não é do editor).
