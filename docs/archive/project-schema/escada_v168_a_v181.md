# Degraus `168 → 171` e `176 → 181` da escada do `PROJECT_SCHEMA`, arquivados verbatim

Saíram de `shells/desktop/src/project_schema.rs` em 2026-11-08 (`line/GameEngineFuncional02`, B2): pagam a catraca da shell. `171 → 172` e `172 → 176` já estavam em `escada_v171_a_v172.md` e `escada_v172_a_v176.md`.

```text
/// # ⭐ 168 → 169 (2026-09-23) — a VIDA e o DANO (plano 28, W3)
///
/// `Health` e `Damage` passam a REGISTADOS — dois tipos e UM degrau, como o raio. ⚠️ O registo
/// esperou uma wave e isso foi MEDIDO como defeito: a cópia de um molde leva só o que está
/// registado, logo toda cópia de uma fábrica nascia SEM vida (report do dono: *«ninguém sumiu ao
/// levar muitos tiros»*). ⚠️ **São componentes da FÍSICA** ⇒ sobe o registo dela (`+2`) e os dois
/// espelhos (`ph2d-render`, `ph2d-script`) NÃO se mexem.
///
/// ⛔ **Sem degrau de migração**, pela mesma decisão — um v168 é recusado em voz alta.
///
/// # ⭐ 169 → 170 (2026-09-24) — a BARRA DE VIDA (plano 28, W4)
///
/// `HealthBar` passa a REGISTADO — um tipo e um degrau, componente da FÍSICA (o registo dela `+1`,
/// os espelhos não se mexem). Registado no mesmo commit que a secção do Inspector e o descritor.
///
/// ⛔ **Sem degrau de migração**, pela mesma decisão — um v169 é recusado em voz alta.
///
/// # ⭐ 170 → 171 (2026-09-24) — o IMPACTO (plano 28, W5)
///
/// SEIS campos no `Health` (a pausa da morte · o piscar · o empurrão aceite · os números, a cor e a
/// altura deles) e TRÊS no `Damage` (a pausa no golpe · o empurrão · o empurrão para cima). ⛔ **ZERO
/// componentes registados novos** ⇒ os três contadores do registo **não se mexem** — o degrau
/// existe porque o postcard é posicional e um v170 seria lido errado **em silêncio**. ⚠️ O
/// `BlinkOff` é DERIVADO e NÃO registado, de propósito (o molde do `MasterPiece`).
///
/// ⛔ **Sem degrau de migração**, pela mesma decisão — um v170 é recusado em voz alta.
///
///
/// # `176 → 177` — **os TIPOS de dano e o dano que DURA** (plano 28, W6, 2026-09-29)
///
/// `Damage` ganha o TIPO (`kind`) e as três grandezas da aflição (`over_time_per_s`,
/// `over_time_s`, `over_time_every_s`); `Health` ganha as RESISTÊNCIAS (`resistances`, uma
/// linha por tipo com a taxa e o «absorve»). ⛔ **ZERO componentes registados novos** — são CAMPOS
/// de dois que já viajam, logo os três contadores NÃO se mexem.
///
/// ⚠️ **Sem degrau de migração, e está certo:** um documento anterior não tem tipo em dano nenhum
/// nem resistência em vida nenhuma, e os valores de fábrica (`kind` vazio, `over_time_per_s = 0`,
/// `resistances` vazia) são exactamente o golpe de antes **ao bit** (`Taxa::NEUTRA`). ⛔ O degrau
/// existe pelo motivo de sempre — *o postcard é POSICIONAL*: sem ele um v176 seria lido errado em
/// silêncio, e com ele o load recusa em voz alta.
///
/// ⚠️ **A tripla NÃO vê este degrau** — nem a forma do `FlipDoc` nem a da `VecScene` mudam.
///
/// # `177 → 178` — **a NAVEGAÇÃO** (plano 30, W4, 2026-10-01)
///
/// `NavRegion` e `NavAgent` passam a REGISTADOS — dois tipos e UM degrau, como o raio e a vida, no
/// mesmo commit que as secções do Inspector (a lição da vida: um componente por registar é invisível
/// à cópia de uma fábrica, ao `Ctrl+Z` e ao ficheiro de uma vez). ⚠️ **São da FÍSICA** ⇒ sobe o
/// registo dela (`+2`) e os dois espelhos (`ph2d-render`, `ph2d-script`) NÃO se mexem. ⛔ O `NavNow`
/// NÃO é registado: é derivado da memória da ponte, que vai no anel de checkpoints.
///
/// ⛔ **Sem degrau de migração**, pela mesma decisão — um v177 é recusado em voz alta.
///
/// ⚠️ **A tripla NÃO vê este degrau** — nem a forma do `FlipDoc` nem a da `VecScene` mudam.
///
/// # `178 → 179` — **o DESVIO entre agentes** (plano 30, W5, 2026-10-02)
///
/// `NavAgent` ganha `avoidance` (o último campo: desvia dos outros corpos que andam, sem sair da
/// área andável). ⛔ **ZERO componentes registados novos** — é um CAMPO de um que já viaja, logo os
/// três contadores NÃO se mexem. ⛔ Sem degrau de migração, pela mesma decisão: um v178 é recusado
/// em voz alta — *o postcard é POSICIONAL*, e sem o degrau ele seria lido errado em silêncio.
///
/// ⚠️ **A tripla NÃO vê este degrau** — nem a forma do `FlipDoc` nem a da `VecScene` mudam.
///
/// # `179 → 180` — **o que FERE, o que CUSTA e os ATALHOS** (plano 30, W7, 2026-10-03)
///
/// UM degrau para três mudanças da mesma wave: `NavAgent` ganha `avoid_harm` (o último campo) e
/// `NavCostArea` e `NavLink` passam a REGISTADOS no mesmo commit que as secções do Inspector. ⚠️
/// **São da FÍSICA** ⇒ o registo dela sobe `+2` (`42 → 44`) e os dois espelhos (`ph2d-render`,
/// `ph2d-script`) NÃO se mexem. ⛔ Sem degrau de migração: um v179 é recusado em voz alta.
///
/// ⚠️ **A tripla NÃO vê este degrau** — nem a forma do `FlipDoc` nem a da `VecScene` mudam.
///
/// # `180 → 181` — **a TEXTURA da forma** (`line/3DModeling`, AS_TEXTURAS, 2026-10-03)
///
/// `ph2d_field_ecs::FieldTexture` passa a REGISTADO (o registo do campo `8 → 9`; o `copy_optional`
/// leva-o): a fonte (nenhuma, o pacote CC0 embutido, ou ficheiros por CAMINHO), o ladrilho, a
/// mistura das vistas e o relevo. Componente NOVO, como o raio e a vida: um documento anterior
/// abre sem textura. ⛔ **Sem degrau de migração**, pela decisão de sempre.
///
/// ⚠️ **A tripla NÃO vê este degrau** — nem a forma do `FlipDoc` nem a da `VecScene` mudam.
```
