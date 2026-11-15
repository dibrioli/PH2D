# Prompt de continuação — GEF 07 Ray Sensor: as melhorias (15/11)

Cole o bloco numa janela nova aberta na raiz do PH2D.

```
/pd-linha-assumir GameEngineFuncional
Worktree: /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional (linha à frente do main; nada integrado).
Estado: GEF 07 Ray Sensor — auditoria de 16 achados curada e 4 abertos resolvidos; smoke do dono OK 15/11 em tudo.
Handoff: docs/GameEngineFuncional/handoffs/HANDOFF_CONTINUACAO_2026-11-15_07_RAY_SENSOR.md (ler inteiro: o que existe
e NÃO se reconstrói). Ficha: docs/GameEngineFuncional/componentes/07_ray_sensor.md.

Item: AS MELHORIAS DO RAY SENSOR (dono 15/11: «vamos às melhorias do nosso ray»). Fonte: a secção 🔦 Ray Sensor de
/home/enio/Documentos/GameEngineSamples/critica_ph2d.html, parte «6 O que falta fazer» (RS-1..RS-12; Godot CORRIDO em
/home/enio/Documentos/GameEngineSamples/fonte_critica/oraculo_raio/). ⚠️ Outro agente edita essa pasta: SÓ LEIA.
Lição do dono (memória feedback_a_critique_that_counts_fields_hides_the_capability_gap): a capacidade vale mais que
contar campos.

Fazer NUM BLOCO (§0.10), cada um com o gate que faltava (vermelho antes, verde depois), mutação, e o oráculo do Godot
CORRIDO por passo onde o Godot tem a mesma coisa (§0.9; Godot MIT; placa via `PH2D_GPU=1 bash scripts/ph2d-run.sh`):
 RS-6  Enabled — caixa; liga/desliga em execução; canal da Timeline e script. Desligado: sem marcas, sem sinais, e
       desligar a ver emite On Exit (decidir e dizer).
 RS-5  «Seeing» — estado «está a ver» enquanto toca, lido pelas regras (Signal Actions/condições que EXISTEM) e pelo
       script. ⛔ Planilha de Eventos abandonada (dono 13/11): não a cite nem a recomece.
 RS-7  O script lê o acerto: objeto, ponto, normal, distância e a lista com See all hits (hoje não lê: nada em
       crates/ph2d-script*). Molde: a API Luau existente (api_help_*), d.luau para o luau-lsp.
 RS-8  Raio pontual por script: ph2d.raycast(de, até, filtro) com os MESMOS filtros do componente (uma porta, a
       cast_ray_query do ph2d-physics), resposta na hora (hitscan).
 RS-1  Thickness: >0 = círculo varrido (o motor já varre: ph2d-physics world/sweep.rs); desenho no canvas largo.
 RS-2  Cone (graus): vê quem está dentro do leque e não tapado (com Walls block); desenho em fatia; leitura e sinais
       por objeto como o See all hits.
 RS-3  Apontar para um objeto: MEDIR primeiro se a constraint Look At no objeto do raio já resolve; se sim, atalho
       «Aim at» com conta-gotas no cartão; se não, campo no motor (StableId, como o Ignore up to).
 RS-12 Ignorar objetos que não são da família: lista de objetos pelo conta-gotas (recomendado: casa com o Ignore up
       to e com o Actors to Ignore do Unreal) — o Only see continua um modo só.
 RS-4  Material do acerto: CONFIRMAR primeiro o que existe (só achei atrito/quique em body_law.rs, sem material com
       nome). Se não existe, propor ao dono em 1 linha antes de construir (é decisão de produto).
 RS-9  Medir o custo de See all hits (fase de 200 peças) ANTES de qualquer limite; tabela no doc-comment (§0.0).
 RS-10 Gate de determinismo: dois raios em ordem de criação trocada ⇒ mesmos acertos e mesmas arestas.
 RS-11 Gate: apagar o alvo ⇒ o raio vê o seguinte no MESMO tique.
Cartão: cada campo novo é canal do catálogo (gate architecture_every_editable_field_is_a_channel), com dica, nos dois
modos; PROJECT_SCHEMA sobe (contado; degrau no fim do RaySensor; sem leitor velho, §0.11). Smoke: a cena 26
(crates/ph2d-physics-scene/src/cena_ray.rs) ganha o que mostrar cada melhoria (um guarda com cone, um tiro hitscan
por script) — fotografada antes do smoke.

Depois: fotos do cartão (cartões ON e PH2D_CARDS=0) olhadas; fmt/clippy 1× (PH2D_SUITE=fecho); gates por nome; handoff
de continuação atualizado; binário smoke compilado 2×; smoke ao dono em passos numerados com fotos como link relativo
à raiz do repo principal ([foto](Worktrees/line-GameEngineFuncional/docs/…png)). Próximo item da fila: Motion Input
(23_refundacao_plano.md), começando pela TABELA DE CAPACIDADES antes das maquetes.

Regras: testes por nome; clique REAL e animação para todo controle; checkbox em moldura; rótulo que não cabe = «…»;
acabamento = estado da arte; catracas de LOC (mover/cortar; a shell só encolhe; ph2d-app-components quase no tecto —
testes em shells/game-scenes/tests/it); delegue leitura e verificação (explorador, verificador, mutacao); janela
grande avisa. Não toque na line-InputMap nem em /home/enio/Documentos/GameEngineSamples. Não integre, não pushe.
```
