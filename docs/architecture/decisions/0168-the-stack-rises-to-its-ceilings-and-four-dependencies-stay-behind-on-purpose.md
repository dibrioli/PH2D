# ADR-0168 — O stack sobe até os TETOS, e o que fica para trás fica por MEDIÇÃO

- **Status:** Accepted
- **Data:** 2026-08-29
- **Linha:** `chore/stack-upgrade-2026-08` (jornada de atualização do stack)
- **Toca:** `Cargo.toml` de 18 crates · `scripts/ship.sh` · `scripts/stack-audit.sh`
- **Não move:** contrato congelado nenhum (CLAUDE.md §6)
- **Registo medido:** [`docs/Atualizar Stack/04_registro.md`](../../Atualizar%20Stack/04_registro.md)

## Contexto

O pedido do Enio foi *«o padrão ouro em tudo, o estado da arte; ao fim não quero nada armengado»*.
A leitura ingénua disso é **subir tudo ao mais recente**. A leitura correcta está no §1 do
`CLAUDE.md`: **«o mais recente possível» ≠ «o mais recente»** — porque uma dependência pode ser
**segurada** por outra, e forçar não dá erro de resolução: dá **duas cópias**, e um tipo de uma não
serve à outra.

Esta jornada subiu o que dava e mediu o que não dava. Este ADR regista **o que ficou para trás e
por quê**, para que ninguém pague a mesma semana daqui a dois meses.

## Decisão

**O stack sobe até ao teto de cada dependência. Onde há teto, ficamos no maior valor que serve a
TODOS os donos — nunca forçando uma segunda cópia.**

### O que subiu

`rustc` 1.98 · `wgpu` 28→**29** · `vello` 0.9→**0.10** · `parley` 0.10→**0.11** ·
`bevy_ecs` 0.18.1→**0.19.1** · `rapier2d` 0.28→**0.35.3** (com `parry2d` 0.30, `glamx` 0.3) ·
`usvg`→0.48 · `rfd`→0.17 · `mlua`→0.12 · `cpal`→0.18 · `wasmtime`→48 · `criterion`→0.8 ·
`linesweeper`→0.4.

### ⛔ O que NÃO subiu, e o mecanismo de cada recusa

#### 1. `wgpu` 30 — segurado pelo `vello` (a recusa mais importante deste ADR)

| facto | valor, medido em 2026-08-29 |
|---|---|
| temos | `29.0.4`, **uma só cópia** (lockstep com `wgpu-core/-hal/-types`, `naga`) |
| topo publicado | **`30.0.1`** |
| quem segura | **`vello` 0.10.0**, que pede `^29.0.3` |
| quantos terceiros seguram | **um só** — nenhum `egui`, nenhum `wgpu-profiler` na árvore |
| existe saída pelo `vello`? | **não**: `0.10.0` é a mais recente publicada, e `0.9`/`0.10` partilham `^29.0.3` |
| quem declara `wgpu` | **10 crates, todas nossas** — movem-se em lockstep |

⚠️ **A costura onde as duas cópias partem tem endereço, e não é desempenho — é tipo.**
[`ph2d-render/src/vello_pass.rs`](../../../crates/ph2d-render/src/vello_pass.rs) passa um
`wgpu::Device` **nosso** para dentro do `vello::Renderer` (linha 46), e um `wgpu::TextureFormat`
nosso para dentro das `RendererOptions`. Com duas cópias, esses tipos são nominalmente distintos e
a chamada **não compila**.

**Exposição, se um dia acontecer:** 193 ficheiros importam `wgpu::`, em 18 crates; **124 deles
(64%) tocam num dos quatro tipos que não atravessam duas cópias** (`TextureFormat` 76 ·
`Device` 68 · `Queue` 24 · `Surface` 4).

⭐ **Gatilho de reconferência, preciso:** *a recusa cai no dia em que sair um `vello` > 0.10.0 que
peça `wgpu` `^30`.* Nada mais precisa de mudar do nosso lado. O marcador vive em
[`ph2d-render/Cargo.toml`](../../../crates/ph2d-render/Cargo.toml), ao lado da declaração.

#### 2. `glam` 0.33 — a unificação custaria o SIMD do renderizador

Cadeia lida no grafo de features (não deduzida):
`rapier2d/enhanced-determinism` → `parry2d/enhanced-determinism` → `glamx/scalar-math` →
**`glam/scalar-math`**.

A árvore tem **duas** cópias do `glam`: a **0.33.6** da física e a **0.30.10** das nossas oito
crates de desenho. O Cargo unifica features **por versão**, e é isso que deixa a física correr em
escalar (exigência do determinismo entre sistemas, HR-5) enquanto o desenho mantém SIMD.

⭐⭐ **As duas cópias não são resíduo: são o mecanismo que deixa as duas metades ter políticas
diferentes.** Unificar imporia `scalar-math` a `ph2d-core`, `-mesh-render`, `-vector`, `-anim`,
`-vec-edit`, `-vector-font`, `-vector-doc` e `-vector-traits`.

E o outro lado da balança está **contado**, não estimado: `Affine3`/`Vec3A` **0 usos** ·
`ISizeVec*` **0** · a correcção de `escalar / matriz` **0** · e o `Vec2::angle_between` removido é
**6 sítios a reescrever**, ou seja *custo*. ⇒ **Um ganho de zero perde para qualquer custo, e por
isso esta recusa não precisou de um número de desempenho — precisou de contar os usos.**

#### 3. Os outros tetos (informativos, reconferidos a cada `ship.sh`)

`accesskit` 0.24.1 (topo 0.25) ← `parley` · `skrifa` 0.44 (topo 0.46) ← `parley`, `usvg`, `vello` ·
`pollster` 0.4 (topo 1.0.1) ← `rfd` · `core-graphics` 0.23.2 (topo 0.25) · e dois onde **nenhuma
versão serve a todos** (`miniz_oxide` ← `ctt`/`exr`/`png`; `thiserror` ← `psd` que ainda pede `^1`).
⚠️ Estes dois últimos **já são duas cópias hoje**, e é benigno: são folhas de compressão/derive que
não aparecem na nossa superfície de tipos.

### Alternativas rejeitadas

| alternativa | por que não |
|---|---|
| **forçar `wgpu` 30 e viver com duas cópias** | não compila — `vello_pass.rs` atravessa `Device`/`TextureFormat` |
| **abandonar o `vello`** | é o rasterizador de todo o desenho vectorial; ADR-0108 |
| **congelar tudo onde estava** | perde 13 subidas que não têm teto nenhum, incluindo correcções |
| **unificar o `glam`** | ver §2 — troca o SIMD do renderizador por zero usos medidos |
| ⛔ **desligar `enhanced-determinism`** | parte o `physics_ecs_c9` entre os 3 sistemas (HR-5) |

## Consequências

- **O inventário de tetos passa a ser um PASSO, não um ponteiro.** O `ship.sh` imprime-o inteiro
  antes do veredito de push (tarefa G2). ⚠️ E **não** como um `✓`: o `stack-audit.sh` sai `0`
  sempre, então um ✓/✗ diria só «correu» e esconderia o que ele produziu.
- ⛔ **A sonda deixou de confundir «não tem teto» com «não consegui perguntar».** Em modo `--tetos`
  ela agora nomeia toda crate que não pôde ser consultada e diz que o número está **incompleto** —
  o defeito foi apanhado ao ver a MESMA corrida dar `6` e depois `7`.
  ⚠️ E a 1.ª cura estava no sítio errado (filtrar pela lista de vigilância), porque essa lista é
  ela própria **derivada** das respostas da rede: sem rede nasce vazia e a cura ficava muda
  exactamente no caso que devia gritar. **A prova de mutação — bloquear a rede — é que o mostrou.**
- Cada teto tem **dono nomeado** e **gatilho de reconferência**, não uma data arbitrária: um teto
  cai quando o dono dele publicar, e é isso que se vigia.

## ⛔ Recusas MEDIDAS

| # | o que foi recusado | mecanismo | onde |
|---|---|---|---|
| 1 | `wgpu` 30 | `vello` 0.10.0 pede `^29.0.3`; `Device`/`TextureFormat` atravessam em `vello_pass.rs` | este ADR §1 |
| 2 | unificar o `glam` em 0.33 | `enhanced-determinism` propaga `scalar-math` a 8 crates de desenho; ganho medido = 0 usos | [registo §15](../../Atualizar%20Stack/04_registro.md) |
| 3 | `#![allow]` para o `manual_slice_fill` partido do clippy 1.98 | a transformação é boa; só o `--fix` é que emite forma partida (236/236) | [registo §2](../../Atualizar%20Stack/04_registro.md) |
| 4 | ligar `friction_in_bias_pass` / `warmstart_joints` na rapier 0.35 | curava um teste ao preço de pilhas altas a tombar | `crates/ph2d-physics/src/world.rs` |
