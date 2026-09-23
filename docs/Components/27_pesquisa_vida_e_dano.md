# 27 — Pesquisa: VIDA e DANO (2026-09-23)

> Pedido do dono: *«Vida e dano. Antes pesquisa para descobrir os sistemas mais aclamados pelos
> usuários.»* Duas frentes em paralelo, as duas registadas aqui: **(A)** o que os motores dão e o que
> os utilizadores elogiam e de que se queixam (fontes na web, cada afirmação com URL); **(B)** o que a
> NOSSA composição já exprime hoje (§5.0 do roteador: *medir antes de construir*), com `ficheiro:linha`.
>
> ⚠️ Marca **[inferência]** o que não veio directamente de uma fonte.

---

## §1 — Resposta curta

1. **Nenhum motor grande traz vida e dano completos de fábrica.** O conjunto mais rico e pronto é a
   extensão **Health do GDevelop** (MIT); o padrão mais elogiado em código é **hitbox/hurtbox**
   (quem acerta ≠ quem é acertado, e o ALVO decide o próprio dano), popular no **Godot** (MIT).
2. **A nossa composição já faz «um inimigo que morre à 3.ª bala»** (§3.1) — com três componentes e
   uma tabela de sinais. ⇒ o que se constrói **não** é o contador de vida: é o que a composição
   **não** consegue exprimir (§3.2), e é exactamente a lista do que os utilizadores mais pedem.
3. **O lado aprovado a correr como oráculo é o GDevelop** (pipeline linear, público igual ao nosso,
   MIT) — ⚠️ **não está instalado nesta máquina** (o arsenal de `_ComoInvestigarApps` não o lista) ⇒
   a instalação e a porta sem interface são o passo zero da wave.

---

## §2 — (A) O que os outros dão

| sistema | vida | invencibilidade | escudo/armadura | regen/dano contínuo | tipos de dano | eventos | licença |
|---|---|---|---|---|---|---|---|
| **GDevelop Health** ([wiki](https://wiki.gdevelop.io/gdevelop5/extensions/health/)) | inicial + máx.; «Is dead» | `DamageCooldown` + acção | escudo (máx., duração, regen, atraso) · `BlockExcessDamage` · armadura fixa e % · **esquiva** | regen de vida e escudo com atraso | — | just damaged/healed/dodged · «Previous damage taken» · sobre-cura opcional | **MIT** ([LICENSE](https://github.com/4ian/GDevelop/blob/master/LICENSE.md)) |
| **Godot** | nada embutido | — | — | — | — | — | **MIT** |
| ↳ hitbox/hurtbox ([GDQuest](https://www.gdquest.com/library/hitbox_hurtbox_godot4/)) | `take_damage()` no dono | à mão | — | — | — | sinais | — |
| ↳ addon cluttered-code ([GitHub](https://github.com/cluttered-code/godot-health-hitbox-hurtbox)) | actual/máx.; `damageable`/`healable`/`killable`/`revivable` | — | modificadores no hurtbox | — | `HealthAction.type` | dano · cura · morte · reviver | MIT |
| **Unity** | nada; padrão `IDamageable` ([Medium](https://medium.com/nerd-for-tech/idamageable-interface-unity-45bf961d141)) | à mão ([dev.to](https://dev.to/aleksandrhovhannisyan/invincibility-frames-in-unity-how-to-make-a-player-flash-when-taking-damage-4nkm)) | — | — | — | — | proprietária |
| **Unreal clássico** | o Actor não tem vida **[inferência]** | — | — | — | `DamageType` ([Epic](https://www.unrealengine.com/en-US/blog/damage-in-ue4)) | Apply(Point/Radial)Damage | EULA |
| **Unreal GAS** ([tranek](https://raw.githubusercontent.com/tranek/GASDocumentation/master/README.md)) | Health/MaxHealth | por tags | escudo antes da vida | efeitos periódicos | meta-atributo Damage | tags | EULA |
| **Roblox Humanoid** ([docs](https://create.roblox.com/docs/reference/engine/classes/Humanoid#TakeDamage)) | Health/MaxHealth em [0, máx] | `ForceField` | — | 1 %/s de fábrica | — | morte/ressurgir | proprietária |
| **Construct 3** | variáveis | comportamento **Flash** ([manual](https://www.construct.net/en/make-games/manuals/construct-3/behavior-reference/flash)) | addon pago | addon | — | addon | proprietária |
| **RPG Maker MZ** ([blog](https://www.rpgmakerweb.com/blog/a-primer-on-database-traits-and-effects)) | HP/MaxHP | — | — | % por turno | **elementos com taxa** | estados | proprietária |

⭐ **A ordem de cálculo do GDevelop, tal como a documentação a expõe:** dano → recarga activa? →
esquiva → redução fixa → redução % → escudo → vida. Um pipeline linear e explícito — é a referência
mais próxima do nosso público, e é o que um oráculo tem de confirmar **passo a passo** (§0.9).

### §2.1 — O que se elogia (ordenado por frequência nas fontes — **[inferência]** na ordem)

1. **hitbox ≠ hurtbox, e o alvo decide** — resistências e knockback por entidade saem de graça.
2. **Invencibilidade temporária com piscar** — GDevelop, Construct (Flash), todos os tutoriais.
3. **Equipas / sem fogo amigo / não se ferir a si próprio** ([saltmire](https://saltmire.github.io/godot-4-hitbox-hurtbox.html)).
4. **Eventos** «levou dano» · «curado» · «morreu».
5. **Escudo, armadura (fixa e %), esquiva.**
6. **Regeneração com atraso** e **dano contínuo** (veneno).
7. **Tipos de dano com resistências/fraquezas** (elementos do RPG Maker, `DamageType`).
8. **Sensação do impacto:** flash branco `~0,1 s` · hitstop `~0,05 s` (golpe final `~0,15 s`) ·
   knockback ([SmashWiki](https://www.ssbwiki.com/Hitlag), [paragraph](https://paragraph.com/@repokuaaa/finding-the-weight-how-i-built-a-tactile-combat-system)).
9. **Barra de vida com rasto atrasado** («dano provisório», [SF wiki](https://streetfighter.fandom.com/wiki/Provisional_Damage)) e **números de dano**.
10. **Sobre-cura** opcional e **reviver**.

### §2.2 — As queixas (cada uma é um gate a escrever ANTES da lei)

- **Dano a dobrar por golpe** — uma espada que atravessa dois hurtboxes num frame ([dredyson](https://dredyson.com/fix-duplicate-hit-detection-in-godot-4-6-2-area3d-hurt-hit-boxes-a-beginners-step-by-step-guide-to-resolving-race-conditions-collisionshape3d-vs-area3d-disabling-and-blacklist-dictionary-workarou/)); `OnTriggerStay2D` a dar dano em todo frame ([Unity](https://discussions.unity.com/t/trying-to-damage-every-1-second-with-ontriggerstay2d/904298)).
- **Golpe perdido em silêncio** — hitbox activado já sobreposto não dispara `area_entered` ([saltmire](https://saltmire.github.io/godot-4-hitbox-hurtbox.html)).
- **Vida fora dos limites / dois caminhos de escrita** — Roblox: `TakeDamage` negativo passava do
  máximo e a escrita directa ignorava o `ForceField` ([devforum](https://devforum.roblox.com/t/humanoidtakedamage-allows-health-to-exceed-maxhealth/215692)); GAS: limitar no `PreAttributeChange` não persiste.
- **Ordem de eventos** — «Is just damaged» do GDevelop falhava com a condição ACIMA do evento que
  causa o dano ([fórum](https://forum.gdevelop.io/t/solved-i-cant-use-health-extension-to-is-health-just-damaged/71894)).
- **Confusão camada/máscara e hitbox/hurtbox** torna o fogo amigo imprevisível; o GDQuest renomeou os
  nós (`HitArea2D`/`HurtArea2D`) para o lembrar.
- **Complexidade do GAS** — C++, «overkill» num jogo simples ([fórum Epic](https://forums.unrealengine.com/t/c-using-gameplay-ability-system-in-a-straightforward-fps-is-it-overkill/2032910)).
- ⚠️ **Unity: não há queixas de Asset Store com fonte** (os textos das avaliações não eram acessíveis).

---

## §3 — (B) O que a NOSSA composição já exprime

### §3.1 — O inimigo de 3 HP JÁ se autora (com um inimigo nascido de uma `Factory`)

- `Counter { vida, start: 3 }` · `SignalOnHit("golpe")` + `SignalTagFilter(bala)` ·
  `SignalActions [golpe, Myself → AddToCounter −1]` · `[golpe, Myself → Destroy, alvo Other]` (a bala) ·
  `CounterWatch [vida AtMost 0, Own → "morri"]` · `[morri, Myself → Destroy]`.
- O próprio cabeçalho do [`counter_watch.rs:37-40`](../../crates/ph2d-ecs/src/counter_watch.rs) chama
  isto *«o suplente #24 `Health` … é composição»*, e o âmbito por-objecto fechou em 20/09
  (`HANDOFF_…_OS_ABERTOS_2026-09-20.md:48`, gate `com_o_ambito_do_objecto_so_o_que_chegou_a_zero_morre`).
- ⚠️ **Ressalvas medidas:** um inimigo posto À MÃO no documento não pode receber `Destroy` (só
  `Hide`) · **uma entidade tem UM `Counter`** (`counter.rs:44`) ⇒ um inimigo com `WeaponFire` (o pente
  é o `Counter` dele) **não tem onde guardar a vida**.

### §3.2 — O que a composição NÃO exprime (e é o que se constrói)

| falta | medido | o que os outros fazem |
|---|---|---|
| **invencibilidade após o golpe** | a tabela de sinais tem **zero campos de guarda** (`state_machine.rs:8-13`); nenhum verbo desliga o `SignalOnHit` | GDevelop `DamageCooldown`; Roblox `ForceField` |
| **dano carregado por quem bate** | o `arg` do `AddToCounter` é fixo na linha do RECEPTOR (`signal_actions_bridge.rs:216-224`); o sinal não leva número (*«o CONTRATO é o nome»*, `ph2d-runtime/src/lib.rs:61`) | hitbox/hurtbox, `IDamageable(DamageInfo)`, `HealthAction.amount` |
| **tecto e «definir valor»** | `AddToCounter +n` cura **sem tecto**; não há verbo que ponha um valor (`22_plano_weapon_fire.md:25`) | todos limitam a `[0, máx]` |
| **knockback** | nenhum verbo aplica impulso nem velocidade | à mão em todos; essencial em «game feel» |
| **dados do contacto no sinal** | a física tem normal e impulso (`bridge/contacts.rs:44,47`), mas o `SignalEvent` leva só nome/quem/outro (`bridge/signals.rs:34-46`) | ponto de impacto (`ApplyPointDamage`) |
| **barra de vida** (HUD e sobre a cabeça) | o HUD só tem RÓTULOS (`hud.rs:76-87`); a barra está listada como ausente (`25_tabela_comparativa_godot.md:151`) e o rótulo por-objecto é *«feature nova — nomeada, não construída»* (`hud.rs:194-196`) | barra + rasto atrasado |
| **escudo · armadura · esquiva · regen · dano contínuo · tipos** | nada | GDevelop (os cinco primeiros), RPG Maker/`DamageType` (tipos) |
| **flash de dano** | ✅ **já existe por composição** — `Tween` no canal `Silhueta` (`ph2d-tween/src/canal.rs:39-48`, *«o flash de dano»*) + `Timer` | Construct Flash |
| **morte como evento** | ✅ o `"morri"` do `CounterWatch` cumpre o papel; o `Destroy` gera `DeathCause::Killed` **sem** sinal | «Is dead», `on_death` |

---

## §4 — Recomendação (o plano sai daqui, doc 28)

**Duas peças, no idioma hitbox/hurtbox que é o mais elogiado:**

- **`Health` (Vida) — no ALVO:** máximo e valor inicial, **limitado a `[0, máx]` por construção**
  (uma porta só de escrita — a queixa do Roblox), **invencibilidade após o golpe** com piscar,
  sobre-cura opcional, e **três sinais** (levou dano · curado · morreu). Ele **não usa o `Counter`**
  (liberta o inimigo com arma), mas o HUD tem de o ler como lê um contador.
- **`Damage` (Dano) — em QUEM BATE:** a quantidade, o tipo, a equipa (sem fogo amigo) e *«acerta UMA
  vez por contacto»* (a queixa nº 1). ⇒ o número viaja de quem bate para quem recebe **sem** o sinal
  passar a carregar números (o contrato do `ph2d-runtime` fica intacto: a ponte de vida lê o
  `Damage` do `outro` do contacto).

**Por ondas, cada uma com smoke:** W0 o oráculo (instalar o GDevelop e correr a extensão Health sem
interface sobre entradas nossas, passo a passo pelo pipeline do §2) · W1 `Health` + `Damage` com os
gates das queixas do §2.2 escritos primeiro · W2 a barra de vida (HUD e sobre a cabeça, com rasto
atrasado) · W3 escudo, armadura, esquiva, regeneração · W4 tipos e resistências + dano contínuo ·
W5 a sensação (knockback — com a normal do contacto a chegar ao sinal —, hitstop, números de dano) ·
W6 cena e tutorial.

⛔ **Não se reconstrói o que existe:** o flash é o `Tween` · o abanão é o `CameraShake` · o renascer
é o `rewind_runtime` (a vida **renasce** num rebobinar, como todo runtime desta família) · a morte
remove pela mesma porta do `Destroy` (só quem é transitório).
