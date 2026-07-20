# ADR-0134 — Wet Paint: a simulação de fluido VOLTA, CPU-first, com paridade testada — e o modo desligado é byte-idêntico

- **Status:** Accepted (ordem explícita do Enio, 2026-07-20; supersede [ADR-0096](0096-remove-watercolor-fluid-pivot-mixer-brush.md) NESTE ponto — ver "A cerca")
- **Data:** 2026-07-20
- **Decisor(es):** Enio + Claude (`line/Painter`)
- **Linha:** `line/Painter` (Modo L, workstation)
- **Pré-requisitos / herança:** app de referência funcional [`docs/Painter/ph2d_wet_paint/`](../../Painter/ph2d_wet_paint/)
  (SPEC.md = fonte única · engine DOM-free · testes de aceitação §18) ·
  [ADR-0040-amendment-2](0040-tool-as-isolated-feature-crate.md) (`Tool::on_tick`, criado exatamente para "aquarela live") ·
  o choke point `stamp_dabs_inner` (a lei do impasto/sculpt: UMA lista de dabs) ·
  handoff [`HANDOFF_line_Painter_wet_paint_2026-07-20.md`](../../HANDOFF_line_Painter_wet_paint_2026-07-20.md)
- **Tags:** painter · wet-paint · fluid-sim · determinism · cpu-first · parity · drop-crate

---

## Contexto — a cerca de Chesterton

**[ADR-0096](0096-remove-watercolor-fluid-pivot-mixer-brush.md) (2026-06-15) REMOVEU uma simulação
de aquarela shallow-water** deste repo (`ph2d-painter-wash`): lenta, complexa, acoplada às sessões
GPU da época, e sem critério de aceitação que não fosse "parece certo". A cerca existe e este ADR
não finge o contrário: **ele a derruba neste ponto específico**, nomeando o que mudou desde então.

1. **Existe um app de referência PRONTO, FUNCIONAL e ACEITO** —
   [`docs/Painter/ph2d_wet_paint/`](../../Painter/ph2d_wet_paint/), HTML + ES modules puros, zero
   dependências. O `SPEC.md` (767 linhas) é especificação comportamental completa com **12 testes
   de aceitação (§18)** — incluindo orçamentos de depósito calibrados em DUAS bitolas de pincel
   (§18.11: massa ±12%, água ±12%, cobertura em faixa) e estrutura de "lanes" (§18.12). O alvo
   deixou de ser irrefutável: é uma suíte executável.
2. **O engine é DOM-free e determinístico** (`js/engine/`, ~2 700 linhas): RNG semeado
   (splitmix32) + hashes inteiros, zero `Math.random`. **Mapeia 1:1 para uma crate Rust**, e os
   testes §18 viram gates de paridade quase de graça.
3. **CPU-first com cadências orçadas**, não pipeline GPU acoplado: passes com cadência própria
   (§5 do SPEC), bbox ativo (o solver só itera onde há fluido), tabela de opacidade pré-computada
   (nenhum `pow` por texel — a lição HR-5 que o wash de 2026 não tinha).
4. **Ordem explícita do Enio** (2026-07-20), com as três regras registradas no handoff §4:
   zero regressão · integração total aos recursos do painter · incompatível se esconde.

O ADR-0096 continua válido para o que ele removeu (o wash antigo e seus gates GPU); o pivot
"mixer-brush" dele foi realizado por outra via (o render-path óptico do Watercolor atual). Este
ADR adiciona um **quarto modo** ao lado dos três existentes — não reescreve nenhum.

## Decisão

**O PH2D ganha um quarto modo de pintura, "Wet Paint"** (rótulo de UI), física real estilo
Rebelle: fluido raso/capilar sobre papel, pigmento em duas camadas (suspenso viaja com o fluxo;
assentado gruda no papel), sangramento wet-on-wet, gotejamento por tilt, secagem com aro escurecido.

### O nome e o namespace (fixado aqui, barato agora e caro depois)

- **UI/rótulo:** "Wet Paint" (seção do painel, toasts) — inglês, como toda UI (HR-15).
- **Código:** crate **`ph2d-wet-paint`**; campos/ids com prefixo **`wetpaint_`**
  (`BrushSpec::wetpaint`, `PAINTER_WETPAINT_*`). ⚠️ O prefixo `wet_*` cru **já pertence ao
  Watercolor** (`wet_rewet`, `wet_dilution`, `wet_soak`, o card Wetness…) — usá-lo criaria dois
  donos para um namespace. `wetpaint_` é inequívoco e greppável.

### A crate (drop-crate, ADR-0075)

`ph2d-wet-paint` porta o engine de `js/engine/` módulo a módulo (rng · opacity · tuning ·
colorops · grid · paper · brush · stroke · solver · drying · sim · trail · tools · painter-façade
· render), **pura** (sem UI, sem editor-core, sem GPU), espelhando a divisão de módulos do JS
(LOC caps desde o dia zero). Única dependência: `libm = "=0.2.16"` (precedente `ph2d-ecs`).

### Determinismo (a lei do porte)

- **A aritmética espelha o JS:** contas em `f64`, armazenamento em `f32` (o que o JS faz com
  `Float32Array` — todo store arredonda round-to-nearest-ties-even, idêntico ao `as f32` do
  Rust). Isso remove a classe inteira de "divergiu porque a precisão mudou".
- **Transcendentais só via `libm`** (cross-OS bit-idêntico): `cos/sin` no bake do papel (frio) e
  no fingering (extensão, default 0), `pow` no sRGB do K–M (checkbox, default OFF). `sqrt` é IEEE
  e fica nativo. Hot path do solver = aritmética + `sqrt`, transcendental-free (HR-5).
- **Semântica JS explicitada num módulo próprio** (`jsmath.rs`): `Math.imul` = wrapping i32 ·
  `|0` = ToInt32 · `Math.round` arredonda meio para **+∞** (≠ `f64::round` do Rust) ·
  `& mask` opera no i32 com complemento-de-dois (negativo vira wrap positivo). Cada helper com
  doc do porquê — é onde um porte silenciosamente diverge.
- RNG = splitmix32 semeado + `hash2` stateless, portados bit-a-bit.

### O contrato de neutralidade (a regra 1 do Enio, executável)

- **Modo OFF ⇒ todo caminho existente é byte-idêntico** — gate de fingerprint no padrão
  `impasto_off_is_byte_identical`. Os smokes aprovados (painter/watercolor/impasto/sculpt/AA)
  são o contrato; nenhum deles se move.
- **Extensões §17 neutras por default** — o teste §18.10 (bit-identidade compiladas vs bypassed)
  porta junto e vira gate.

### A lei de integração (regra 2 — os canais JÁ cavados; não inventar juntas)

- **Dabs entram por `stamp_dabs_inner`** (o choke point do impasto/sculpt) ⇒ Symmetry, Tiling,
  shape editors, pressão, Jitter de graça — hoje e daqui a seis meses.
- **Silhueta = `silhouette_at`** (falloff × hardness × Shape × footprint elíptico); a bristle
  texture do SPEC §7 compõe como o Grain compõe (fator multiplicativo), nunca substitui.
- **Grain do artista substitui a bristle default** (mesma lei do painter normal); **Paper entra
  pelo slot `BrushSpec::paper`** — os 3 presets procedurais do SPEC §4 viram MAIS UMA fonte do
  MESMO slot, nunca um segundo sistema de papel (a extração de substrato pendente do doc 19
  ganha aqui seu segundo consumidor).
- **Pressão sintética do SPEC §8 é SUBSTITUÍDA pela pressão real** do stroke engine do painter.
- **O tick da sim = `Tool::on_tick`** (o amendment-2 existe para isto) — a água continua andando
  e secando após o pen-up, a 40 Hz fixos por acumulador.
- **Incompatível não é pintado** (regra 3): controle sem sentido no modo some do painel, com
  gate de presença E ausência (precedente `impasto_hides_the_accumulate_row`).

### Kill criterion (ANTES do build, DIRETIVA §5)

- **Depósito:** o delta por-move do modo obedece a barra da casa — alvo ≤ 4 ms/move, **kill
  8 ms/move @2048²/4096²**, medido como delta contra o mesmo caminho com o modo OFF (formato de
  `impasto_perf_kill_criterion`).
- **Sim viva:** um tick de 40 Hz custa **≤ 8 ms no flood upper-bound do §18** (a cena de ~110k
  células molhadas) — metade de um frame de 60 fps; acima disso a sim não existe nesta forma.
  O custo escala com a ÁREA MOLHADA (bbox ativo), não com o canvas — o W0 mede e grava a tabela.
- Duas reconstruções de topologia sem fechar ⇒ PARE e prove o modelo (two-strikes).

## Consequências

- Os testes §18.1–.12 rodam em Rust como a suíte de paridade da crate (mesmos números, mesmas
  tolerâncias); `node` não é nem requisito de build nem de CI.
- O shell JS (`js/app/`) é descartado — nosso shell já tem canvas/zoom/undo/layers/export/i18n.
- Os ~39 knobs do tuning panel NÃO viram 39 sliders: meia dúzia curada no painel; o resto vira
  constante calibrada com nome e valor documentados no código (a tabela §16 é a fonte).
- Undo: a sessão molhada entra no `ModelSnapshot` **no mesmo commit** que criar cada plano novo
  (a lição §10.4 do impasto — o bug do `mats` fora do snapshot).
