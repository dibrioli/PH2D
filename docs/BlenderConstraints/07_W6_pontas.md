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
