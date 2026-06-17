> ⚠️ **SUPERSEDED por [ADR-0096](architecture/decisions/0096-remove-watercolor-fluid-pivot-mixer-brush.md) (Enio 2026-06-14):** toda a simulação de aquarela/fluido/wash foi **REMOVIDA** do código (crate `ph2d-painter-wash` deletada, canvas voltou a CPU-residente). Doc mantido só como histórico. Norte atual = **Brush Engine (mixer-brush)**, ver [`docs/Novo Painter/`](Novo%20Painter/). Backups em `backups/wash_2026-06-14`.

# HANDOFF — SMOKE da aquarela K–M multi-pigmento (pro Enio validar)

> **O que foi feito (loop autônomo P0–P6, ADR-0080):** o campo molhado da aquarela agora carrega
> **pigmento misturável** (Kubelka–Munk espectral, 28 canais/célula) em vez de "cobertura cinza +
> UMA cor por traço". Resultado: **azul + amarelo molhados sangram num verde vibrante** (mistura
> SUBTRATIVA de tinta real), tanto DENTRO de um traço quanto **ENTRE traços** (o campo persiste
> enquanto molhado). O caminho de **uma cor** reproduz o look que você ratificou.
>
> Tudo commitado localmente (5 commits, NÃO pushado até você mandar — mas a sessão já preparou o
> ship+CI por instrução sua; veja §4). Norte: [ADR-0080](architecture/decisions/0080-watercolor-km-multipigment-field.md).

---

## §1 — O que olhar (a validação visual)

Abra o Painter com um pincel de **aquarela** (`fluid_enabled`). Olhe, em ordem de importância:

1. **A MÁGICA (azul + amarelo = verde) — cross-stroke wet-on-wet:**
   - Pinte um traço **AZUL**. Sem esperar secar, pinte um traço **AMARELO** por cima/cruzando.
   - **Esperado:** a zona de sobreposição vira **VERDE vibrante** (mistura subtrativa), NÃO um
     cinza/lamacento nem um cinza-azulado. Quanto mais molhado, mais sangra o verde.
   - Teste também **azul + vermelho → violeta** e **ciano + vermelho → neutro/cinza** (a assinatura
     da mistura física). Amarelo + vermelho → laranja.

2. **UMA cor = o look de antes (não-regressão):** pinte um traço de **uma única cor** (ex.: um azul,
   um vermelho escuro). Deve ficar **idêntico** ao que você ratificou: value-opacity (cor escura
   cobre mais), edge-darkening (borda mais escura), capilaridade (franja transparente macia),
   sharpness. Se algo mudou no traço de uma cor, é regressão — me avise.

3. **Preto/escuro ainda pintam:** um traço **preto** (ou um azul-marinho bem escuro) deve cobrir o
   papel normalmente (não sumir). A opacidade-por-valor continua: cor escura = mais opaca.

4. **Dentro de um traço (Color Dynamics):** se o pincel tiver jitter de cor (Color Dynamics), as
   variações de cor dos stamps se misturam no campo — o traço fica com transições de cor mais ricas
   (subtrativas), não um gradiente lavado.

5. **Capilaridade + sharpness + backruns** continuam funcionando (a física S0–S5c é a mesma; só o
   pigmento que ela transporta passou a ser misturável).

> Se o verde sair **fraco/escuro** ou a sobreposição sair **acinzentada**, anote o caso (cores +
> molhado) — pode ser tuning do `coverage_k`/value-opacity, não da mistura (a mistura é provada
> verde nos testes; o look final é o que ajustamos com você).

---

## §2 — Comandos (rodam sem abrir o app — já passaram nesta sessão)

```bash
SLOT='CARGO_TARGET_DIR="$PWD/target-slots/slot-brushoverhaul"'   # prefixe cada cargo

# Matemática + campo CPU + composite (P0–P2):
cargo test -p ph2d-painter-brush --lib            # inclui field_mix_*, single_color_*_legacy

# Paridade GPU↔CPU em Metal (P3) — a prova de que a GPU mistura igual à CPU:
cargo test -p ph2d-painter-fluid --features fluid --test gpu_parity --test composite_parity \
  -- --ignored --test-threads=1                   # inclui gpu_multi_pigment_subtractive_mix

# Cross-stroke wet-on-wet (P4) + tool:
cargo test -p ph2d-tool-painter --lib             # inclui fluid_cross_stroke_wet_on_wet_mixes

# Contratos congelados (P5) — nada regrediu:
cargo test -p ph2d-painter-contracts --test architecture_painter_contract_surface
```

Gates verdes nesta sessão: pigment_mix/diffusion/wet_composite (CPU), gpu_parity 14/14 +
composite_parity 7/7 (Metal), contract_surface 8/8, tool 215, painter-contracts 82.

---

## §3 — O que a feature faz por dentro (1 parágrafo)

Cada célula do campo guarda `ks[24]` (K/S espectral ponderado por massa) + `err[3]` (re-âncora) +
`mass` (cobertura) = 28 canais. `diffuse/advect/transfer/capillary/maccormack` transportam os 28
canais **linearmente** → quando dois pigmentos se encontram, seus K/S somam e a redução por-célula
(`reflectance(ks_acc/mass)`) dá a cor misturada — **transporte linear de K/S = mistura subtrativa**.
A GPU espelha isso (7 vec4/célula, paridade bit-a-bit). O composite reduz o campo por-pixel e faz o
glaze K–M sobre o backdrop. Um único pigmento reduz exatamente à cor escolhida (look preservado). O
campo **persiste entre traços enquanto molhado** (`begin_stroke` reusa o campo úmido; o dry-drop só
o larga quando seca) → wet-on-wet cross-stroke.

---

## §4 — Estado de ship (por sua instrução nesta sessão)

Você pediu: *"ao final ship englobando os commits de todos os agentes, depois CI e monitore até
verde."* A sessão fez/fará: `./scripts/ship.sh` (paridade com os gates do CI) → `git push origin
main` → babysit `gh run watch` até verde, corrigindo o que aparecer. Se você está lendo isto antes
do CI fechar, confira o link da run que a sessão deixou no chat.

**Limitação conhecida (documentada, não bug):** o campo de 28 canais é 7× a memória do antigo. Num
grid full-res 4K (≥2048² com `scale=1`) o buffer de pigmento estoura o `max_storage_buffer_binding_size`
default — mas o caminho de produção usa grid **low-res** (canvas/4), então não afeta o uso real; o
benchmark `perf_resident` pula esses tamanhos. 4K full-res-resident é um follow-up de GPU-residency.

— deixado pela sessão de 2026-06-09 (Claude). Próximo passo seu: o **smoke visual** acima.
