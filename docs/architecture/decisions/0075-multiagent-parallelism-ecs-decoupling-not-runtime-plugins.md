# ADR-0075: Arquitetura de paralelismo multi-agente — ECS-decoupling + build-speed, NÃO plugins em runtime

**Status:** Accepted
**Data:** 2026-05-28
**Decisor(es):** Enio + LLM (pós deep-research multi-fonte com verificação adversarial 3-votos)

## Contexto

O PH2D é desenvolvido por **múltiplos agentes LLM em paralelo** numa única máquina
(Apple Silicon, **8 GiB RAM**, ≤3 agentes concorrentes aceitos). Após várias
tentativas de "isolamento total", persistia um emaranhado de conflitos:
(a) acoplamento de build (qualquer mudança recompila o workspace; deps recompiladas
~5× em target dirs isolados); (b) tangle git/dependência entre crates "isoladas";
(c) **costuras de schema compartilhado** — uma mudança de schema numa crate quebra
silenciosamente o golden test de outra (incidente σ-4: Sprite v3→v4 quebrou a golden
de cook do KTX2).

A hipótese tentadora era **inverter pra plugins/add-ons em runtime** (estilo Blender),
em Rust nativo (cdylib + `abi_stable`/`stabby` + hot-reload). Uma deep-research
(26 fontes, 25 claims, 24 confirmados 3-0) sobre como os grandes projetos Rust
resolvem isso **derrubou essa hipótese**:

- **Rust não tem ABI estável.** `repr(Rust)` é explicitamente instável — o layout de
  struct/enum "pode até diferir entre compilações" ([Rust Reference — Type Layout](https://doc.rust-lang.org/reference/type-layout.html)).
  Qualquer tipo compartilhado cruzando fronteira dinâmica é **UB**.
- **Zed** (editor Rust de produção) **rejeitou cdylib nativo e foi de WASM** justamente
  porque Rust não tem ABI estável ([zed.dev/blog/zed-decoded-extensions](https://zed.dev/blog/zed-decoded-extensions)).
  WASM está **fora de escopo** por decisão do Enio (all-Rust).
- **Fyrox** é o único engine com plugin Rust dinâmico + hot-reload (CHR), e a própria
  doc o chama de **"very new and experimental, based on wildly unsafe functionality
  which could result in memory corruption"**, com fallback pra static linking
  ([fyrox-book](https://fyrox-book.github.io/beginning/hot_reloading.html)).
- **Hot-reload Rust (Dioxus `subsecond`, `hot-lib-reloader`) é dev-only** e **CRASHA/UB
  exatamente em mudança de layout de struct** — a MESMA classe do nosso σ-4
  ([subsecond](https://docs.rs/subsecond/), [kra.hn](https://robert.kra.hn/posts/hot-reloading-rust/)).
- **`stabby`/`abi_stable`** existem mas **não garantem compat cross-compiler** por
  default (só "canaries" opt-in) e **sem uso verificado em produção large-scale**
  ([stabby](https://github.com/ZettaScaleLabs/stabby)).

**Conclusão dura:** plugin nativo em runtime (1) é instável/UB em Rust, (2) nenhum
projeto sério roda em produção, e (3) **nem resolve o problema** — a costura de schema
(σ-4) é o que destrói o hot-reload Rust. Não há fuga arquitetural por plugins.

O caminho **proven** nos grandes projetos Rust (Bevy — o análogo mais próximo: engine
Rust, ECS, wgpu) é **desacoplamento em compile-time (ECS + Plugin trait) + tooling de
build rápido**. `bevy_dylib` (dynamic linking ~5× incremental) é **dev-only build-speed,
não plugin runtime**, e proibido em release ([docs.rs/bevy_dylib](https://docs.rs/bevy_dylib/)).

## Decisão

1. **PH2D permanece um único binário Rust compilado (workspace monorepo).** **Não**
   haverá sistema de plugin nativo em runtime (`cdylib`/`abi_stable`/`stabby`) nem
   add-on em linguagem dinâmica/WASM para a camada de features. Isso é norte
   arquitetural, não débito.

2. **O desacoplamento é no nível de DADOS/SISTEMAS, não de binário.** Features são
   *systems*/Plugins (compile-time `Plugin` trait) que comunicam via **components +
   events/resources** do ECS (`bevy_ecs`) e **nunca chamam umas às outras diretamente**.
   A fronteira Sim/Present (ADR-0021) e os contratos congelados (ADR-0039/0040/0041)
   permanecem. Feature nova = drop-crate (A) + registro distribuído.

3. **A fricção de paralelismo é atacada por build-speed + gates, não por isolamento de
   runtime.** As alavancas operacionais (dev dynamic-linking, crates menores,
   `inventory`/`linkme` no lugar de codegen-sync central, sccache/cranelift, seleção de
   teste por impacto) vivem em [`DIRETRIZ §6`](../../IntegracaoMultiAgente/DIRETRIZ.md)
   e são refinadas pela deep-research de velocidade-de-agentes.

4. **A costura de schema (σ-4) é IRREDUTÍVEL** — nenhuma arquitetura a remove (plugins
   crasham nela). Gerencia-se com **gates de contrato em compile-time + contract/snapshot
   tests** (já praticado). Isso é estado-da-arte, não falha.

5. **O teto de 8 GiB RAM é aceito.** Concorrência **≤3 agentes**. O ganho-alvo é
   **velocidade de iteração por-agente** (loop edit→check→fix), não mais paralelismo.

## Consequências

**Positivas:** caminho proven (Bevy); zero UB de ABI; zero inferno de versão de
compilador; mantém segurança de tipo whole-program do Rust; o desacoplamento ECS é
testável e gateado; features continuam drop-crate.

**Negativas:** sem hot-swap de feature em runtime (hot-reload de dev é separado e
limitado); a costura de schema permanece (mitigada por gates, não eliminada);
recompilação cross-crate continua existindo (mitigada por build-speed).

**Neutras:** reforça decisões já tomadas (ECS ADR-0003/0021, contratos congelados,
drop-crate). Fecha definitivamente a porta de "plugin runtime" como linha de pesquisa.

## Alternativas consideradas

| Alternativa | Rejeição | Fonte |
|---|---|---|
| Plugin nativo cdylib em runtime (`abi_stable`/`stabby`) | Sem ABI estável (repr(Rust) instável = UB); sem produção verificada; não resolve schema-coupling | [stabby](https://github.com/ZettaScaleLabs/stabby), [Rust Reference](https://doc.rust-lang.org/reference/type-layout.html) |
| Extensões WASM (caminho do Zed) | Fora de escopo por decisão Enio (all-Rust, sem WASM) | [zed.dev](https://zed.dev/blog/zed-decoded-extensions) |
| Hot-reload nativo de feature (Fyrox CHR / `hot-lib-reloader` / `subsecond`) | Experimental/"wildly unsafe"/dev-only; crasha em mudança de schema | [fyrox-book](https://fyrox-book.github.io/beginning/hot_reloading.html), [subsecond](https://docs.rs/subsecond/) |
| "Mais coordenação/processo" sobre a arquitetura atual | Ótimo local — o conflito é estrutural (acoplamento compile-time), não comportamental | (análise interna) |

## Notas

- Esta ADR **fecha a linha de pesquisa "plugin/add-on runtime"**. Reabrir exige
  evidência nova de ABI estável Rust proven-em-produção.
- `bevy_dylib`-style dynamic linking é **adotável como dev build-speed** (não viola
  esta ADR — é compile-time/dev, não plugin runtime). Detalhe em DIRETRIZ §6.
- Deep-research bruta arquivada no transcript da sessão 2026-05-28 (run `wf_ec902a89-737`).
