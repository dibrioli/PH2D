# Oráculo GDevelop *Health* — a extensão REAL, corrida sem interface, quadro a quadro

> Instrumento de medição, não produto. Ele põe a extensão **Health** do GDevelop (MIT) a
> correr no **runtime GDJS do próprio GDevelop**, com o código gerado pelo **gerador de código
> do próprio GDevelop**, sobre cenários **nossos**, e grava cada passo (um quadro) como fixture
> com cabeçalho. Nada aqui reimplementa a extensão (CLAUDE.md §0.9: *o alvo é um oráculo que se
> CORRE*).

## A porta que funcionou

```
libGD.wasm 5.6.282 (tirado do app.asar INSTALADO)
  + carregador de extensões do IDE (newIDE EventsFunctionsExtensionsLoader, empacotado no gdcore-tools)
      → gd.BehaviorCodeGenerator gera o JS do comportamento «Health::Health»
  + projecto NOSSO (1 Sprite «Hero» com o comportamento, eventos da cena gerados)
      → gd.Exporter.exportWholePixiProject  (o exportador HTML5 do GDevelop)
  → jogo exportado (index.html + GDJS Runtime 5.6.282 INSTALADO + código gerado)
  → Chrome headless do sistema (puppeteer-core), laço de jogo trocado por
    SceneStack.step(1000/60) manual — o mesmo renderAndStep que o laço real chama
```

- **As ACÇÕES passam pelos eventos do GDevelop.** O driver só escreve variáveis de cena
  (`Cmd1..4`, `A1..4`); a cena tem `4 ranhuras × 26 comandos` eventos *standard* gerados
  (`CompareNumbers(CmdN = id)` → `Health::Health::<Acção>(Hero, Health, …)`), compilados pelo
  gerador de eventos do exportador. Ex. do código gerado:
  `…getBehavior("Health").Hit(runtimeScene.getScene().getVariables().getFromIndex(1).getAsNumber(), true, false, null)`.
- **As LEITURAS chamam os métodos públicos gerados** (`getBehavior("Health").IsJustDamaged(null)`,
  `.Health(null)`, …) — é exactamente o que um evento compila. Controlo: `Health`,
  `ShieldPoints` e `IsDead` são **também** lidos por eventos (`SetNumberVariable` /
  condição `IsDead`) e têm de bater com a chamada directa em todo passo.
- **Três momentos por quadro:** `antes` (fase de eventos, antes das acções; já depois do
  `doStepPreEvents` do comportamento) · `depois` (fase de eventos, depois das acções) · `fim`
  (entre quadros, depois do `doStepPostEvents` e do render).
- **Estado registado por passo:** as 19 expressões + 8 condições públicas, os `_get*()`
  (propriedades internas), os dois timers de objecto que a extensão usa
  (`__Health.TimeSinceLastHit`, `__Health.ShieldDuration`), o tempo, e **quantos sorteios** o
  passo fez (e os valores, quando semeado).

## Reproduzir do zero (UM comando)

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components
bash scripts/ph2d-run.sh bash docs/Components/ferramentas/gdevelop_health/oraculo.sh
# só alguns cenários:
bash scripts/ph2d-run.sh bash docs/Components/ferramentas/gdevelop_health/oraculo.sh b_cooldown h_temporizacao_flags
# ver uma fixture como tabela:
python3 docs/Components/ferramentas/gdevelop_health/mostra.py docs/Components/ferramentas/gdevelop_health/fixtures/b_cooldown.json.gz --acoes
```

`oraculo.sh` = `setup.sh` (monta `vendor/` se faltar; rede na 1.ª vez) → `exporta.mjs`
(~0,5 s) → `corre.mjs` (~1 min para os 18 cenários, cada um corrido 3×). Sai `exit 1` se
qualquer controlo falhar. Medido: apagando `vendor/` e correndo de novo, os `passos` das 18
fixtures saem **iguais** aos da corrida anterior (só o campo `data` do cabeçalho muda).

**Nenhuma janela:** Chrome `headless: true` (modo novo), com `DISPLAY` e `WAYLAND_DISPLAY`
vazios no ambiente dele; nada de `spectacle`, nada de ecrã virtual (não foi preciso).

## Proveniência e licenças (fixadas)

| Peça | Origem | Versão / id | Licença |
|---|---|---|---|
| GDevelop (libGD.js/.wasm, GDJS Runtime) | pacote AUR `gdevelop-bin`, `/usr/lib/gdevelop` (`app.asar` → `www/libGD.*`; `GDJS/Runtime`) | `5.6.282-1` (libGD diz `5.6.282-0`) | MIT |
| Carregador de extensões do IDE | npm `gdcore-tools@2.0.0-gd-v5.6.269-autobuild` (arthuro555) — **só** `dist/loaders.cjs` (código do newIDE empacotado); o libGD e o Runtime dele **não** são usados | 5.6.269 | MIT |
| Extensão Health | `GDevelopApp/GDevelop-extensions` `extensions/reviewed/Health.json` @ `3f71acc8f3d68cea9839654ea8ff606162a4cb39` (HEAD de 2026-09-20; último commit no ficheiro `93773d33…`, 2026-02-21) — sha256 `011f36a8…5f1109`, verificado pelo `setup.sh` | v0.4.0, `@4ian` | MIT |
| `@electron/asar` (extrair o asar) | npm | 4.3.0 | MIT |
| `puppeteer-core` (cliente CDP) | npm, sem descarregar browser | 25.12.0 | Apache-2.0 |
| Browser | `google-chrome-stable` do sistema | 154.0.8037.57 | — (só executa) |

Os artefactos de terceiros vivem em `vendor/` com `.gitignore` `*` — **nada vai para o git**,
o `setup.sh` re-obtém tudo. O jogo exportado fica em `vendor/jogo/` (com `projecto.json` e
`oraculo_meta.json` para auditoria). O código gerado pelo carregador do IDE é escrito em
`os.tmpdir()`; o `oraculo.sh` aponta o `TMPDIR` para `vendor/tmp` durante o export.

## Ficheiros

| Ficheiro | Papel |
|---|---|
| `oraculo.sh` | o comando único |
| `setup.sh` | monta `vendor/` (idempotente, verifica o sha256 da extensão) |
| `comandos.mjs` | a tabela ÚNICA de comandos (acção pública + parâmetros `yesorno` fixos) |
| `cenarios.mjs` | as entradas nossas (um cenário = uma fixture) |
| `exporta.mjs` | constrói o projecto e exporta-o com o exportador do GDevelop |
| `oraculo_driver.js` | injectado no `index.html`: passo manual, ganchos de leitura, sorteios |
| `corre.mjs` | Chrome headless, três corridas por cenário, controlos, escreve `fixtures/*.json.gz` |
| `mostra.py` | lê uma fixture e imprime a tabela passo a passo |

## Formato da fixture (`fixtures/<cenário>.json.gz`)

`{ cabecalho, inicial, passos[] }` — comprimida como os outros corpora de oráculo do repo, uma
linha JSON por passo. O `cabecalho` traz versões, extensão (URL/commit/sha256), browser, data,
comando, a política de tempo, o significado de cada fase, a tabela de comandos, a política do
aleatório, **os controlos** e o **cenário de entrada inteiro**. Cada passo:
`{passo, acoes, antes, depois, fim, sorteios{n, valores}, tempo_desde_inicio_ms, dt_efectivo_ms, chk_por_eventos, nota?}`;
`antes/depois/fim = {publico, bruto, timers}`. `NaN`/`Infinity` são gravados como texto.

### Controlos (em todo cenário, no cabeçalho)

- **reprodutivel** — corrido 2× em páginas novas, saída igual ao byte.
- **ler_antes_nao_perturba** — 3.ª corrida sem a leitura `antes` dá o mesmo `depois`/`fim`
  (as expressões/condições que lemos não têm efeito lateral).
- **expressoes_por_eventos_iguais_as_chamadas** — `Health`/`ShieldPoints`/`IsDead` por eventos = por chamada.
- **erros_da_pagina / pedidos_404** — vazios (qualquer `console.error` ou `pageerror` reprova).

## O que NÃO funcionou (e porquê)

1. **`TMPDIR` comprido** → Chrome morre com *«Socket path too long»* (`SingletonSocket` em
   `vendor/tmp/…`). Cura: `TMPDIR` só no export; o perfil do Chrome é um `mkdtemp` curto em
   `/tmp`, apagado no fim.
2. **Sem `layout.updateBehaviorsSharedData(project)`** o runtime acusa
   *«Can't find shared data for behavior with name: Health»* (o IDE faz isto ao acrescentar um
   comportamento; um script tem de o fazer à mão).
3. **`/favicon.ico` 404** sujava o controlo de erros → servido como `204`.
4. **Não tentado:** Node puro sem DOM (o PIXI/renderer precisaria de stubs — o Chrome headless
   já dá o runtime inteiro sem mentira nenhuma) e o libGD 5.6.269 do próprio `gdcore-tools`
   (preferimos a versão INSTALADA; o risco de usar o `loaders.cjs` 5.6.269 com o libGD 5.6.282
   ficou medido como nulo: export sem erro, classe gerada presente, zero erros de página).

## Limitações conhecidas

- **Semente:** a extensão não tem semente — sorteia com `gdjs.randomFloatInRange(0,1)` →
  `Math.random()`. O HARNESS substitui `Math.random` por `mulberry32(semente)` e **conta** cada
  sorteio (valores guardados só quando semeado). É o gerador do harness, não o do GDevelop.
- **Tempo:** `dt` fixo `1000/60` ms; os timers acumulam em vírgula flutuante (30 passos →
  `0,5000000000000002` s; 60 passos → `0,9999999999999991` s). **Uma fronteira exacta
  (`t == cooldown`) é decidida pelo arredondamento** — um gate não deve assentar nela sem
  reproduzir a mesma acumulação.
- **Valores iniciais** das propriedades vêm do objecto com os defaults da extensão (Health 100,
  MaxHealth 100, ShieldDuration 5, o resto 0/false); os cenários mudam-nos por acções no passo 0.
  Não se testou configurar o objecto com outros valores iniciais.
- Um objecto só; o render corre (PIXI 7.4.2 em SwiftShader) mas nada é desenhado de relevante.
- `HitAtLeastOnce` é uma condição **privada** na extensão (lida na mesma, como as outras).
- As fixtures ainda **não** são gates Rust — são o corpus que os gates do nosso `Health` vão ler.

## O que as 18 fixtures dizem (observado, não lido)

Ordem do quadro no runtime: `doStepPreEvents` do comportamento (repõe as flags «just», regenera
vida e escudo, expira o escudo) → **eventos** (as acções) → pós-eventos.

| Tema | Observado | Fixture(s) |
|---|---|---|
| **Pipeline** | **cooldown → esquiva → plana → percentual → escudo → vida**: durante o cooldown o golpe não sorteia nem esquiva (`g4`: 0 sorteios, sem `IsJustDodged`); um golpe que a armadura zera **ainda sorteia** (`f1`/`e5`) ⇒ esquiva antes da armadura; flat 5 + 50 % sobre 25 dá **10** (não 7,5); com escudo 10, `Hit+Shield+Armor 25` deixa a vida intacta e `40` põe **7,5** na vida ⇒ armadura antes do escudo | `f3`, `g4`, `f1`, `e5` |
| **Cooldown** | o golpe dentro da janela é **IGNORADO por inteiro** (nem reduzido, nem sorteio, `PreviousDamageTaken` fica, flags ficam) e **não** reinicia a janela; activo sse `TimeSinceLastHit < cooldown` (estrito) | `b` |
| **Quem arma o cooldown** | só um golpe que **entra** (dano > 0 na vida **ou no escudo**): o só-escudo arma; esquivado, `Hit 0`, `Hit −10` e golpe zerado pela armadura **não** armam (nem repõem o `TimeSinceLastHit`); `TriggerDamageCooldown` arma sem dano; 2 `Hit` no mesmo quadro → o 2.º é ignorado | `b`, `e5`, `g2`, `a` |
| **Clamp** | a vida **NÃO** pára em 0: desce a negativo (`−20`, `−30`); `IsDead` ⇔ `Health ≤ 0` (0 já é morto); golpe em morto continua a tirar; `Heal` em morto **ressuscita** (`−30 + 50 = 20`); `SetHealth` limita em cima (150 → 100) e não em baixo (−10 fica); `Hit −10` não cura (vida igual) mas grava `PreviousDamageTaken = −10` | `a` |
| **Cura / overheal** | sem overheal pára no máximo e `PreviousHealAmount` = o **aplicado** (50 pedidos a 70 → 30); com overheal passa (100 → 130); `SetMaxHealth` abaixo da vida **corta a vida** mesmo com overheal (130 → 120); `Heal 0` acende `IsJustHealed`; `Heal −20` **tira** vida e acende a flag | `c` |
| **Regeneração** | `rate × dt` no pre-events, quando `TimeSinceLastHit > delay` (estrito); o relógio começa no **onCreated** (sem golpe nenhum regenera 1 s depois da criação); `SetHealth` **não** repõe o atraso; um golpe só-escudo **repõe** (pausa a regen da VIDA); nunca passa do máximo mesmo com overheal; delay 0 ⇒ regenera no quadro seguinte ao golpe | `d`, `d2` |
| **Escudo — absorção** | absorve antes da vida; o excesso passa à vida com `BlockExcessDamage` OFF e **não** passa com ON (só o golpe que parte o escudo; o seguinte, já sem escudo, entra); `Hit` sem `UseShield` ignora o escudo; `ActivateShield` **SUBSTITUI** os pontos (30 → 10), limitado por `MaxShield` **só se `MaxShield > 0`** (0 de fábrica = sem tecto); `SetShieldPoints` **não** é limitado (90 com máx 50) | `e1`, `e2` |
| **Escudo — duração** | activo sse o timer `< duração`; ao expirar os pontos vão a **0** no pre-events seguinte; `ActivateShield` sem renovar com a duração já gasta deixa pontos **inactivos** (o `Hit+Shield` vai à vida); `RenewShieldDuration` reactiva; **duração 0 = nunca expira**; antes da 1.ª activação `ShieldTimeRemaining` diz 5 com o escudo inactivo | `e3` |
| **Escudo — regeneração** | `rate × dt` quando `TimeSinceLastHit > delay`, pára no `MaxShield`; **depois de expirar, a regen a partir de 0 REACTIVA o escudo com duração nova** (o timer volta a 0) — ciclo | `e4` |
| **Esquiva** | 1 sorteio por golpe que passa o cooldown (mesmo com chance 0, dano 0 ou armadura a zerar); esquivado sse `r < chance`; esquivado ⇒ `IsJustDodged`, `PreviousDamageTaken := 0`, escudo intocado, sem cooldown; semeada (`g3`) a sequência e os veredictos estão gravados | `g1`–`g4`, `h` |
| **Flags «just»** | acendem na acção, valem para os eventos **depois** dela no mesmo quadro e até ao fim do quadro, e apagam no `doStepPreEvents` do quadro seguinte ⇒ **1 quadro**; ler **antes** da acção no mesmo quadro dá `false`; golpes em quadros seguidos mantêm-na acesa; `Hit`+`Heal` no mesmo quadro acendem as duas; golpe só-escudo acende `IsShieldJustDamaged` e **não** `IsJustDamaged` (e põe `PreviousDamageTaken = 0`) | `h`, `e1` |
