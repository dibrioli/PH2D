---
name: feedback-widely-constructed-type-favors-optional-component-over-appended-field
description: "adding a per-entity property: appending a field to a type built at N sites costs N edits and recurs; an optional presence-override component costs zero churn and no schema bump"
metadata:
  node_type: memory
  type: feedback
---

Ao adicionar uma propriedade por-entidade, a escolha entre **apendar um campo** num tipo existente e **um componente opcional novo** é decidida pelo **número de sítios de construção** do tipo — não pela nota do plano.

Caso `line/physics` W8 (gravity scale, 2026-07-19): a nota do W1 previa apendar `gravity_scale` ao `RigidBody`. Medido: `RigidBody` é construído como literal `{ kind }` em **~80 sítios** (fixtures de toda wave). Apendar um campo obrigatório = 80 edições mecânicas arriscadas em testes de OUTRAS waves — e **recorreria** para cada um dos 4 campos previstos (damping/ccd/can-sleep). O componente opcional de **presence-override** (o idioma que o resto do Inspector já usa: `ZIndexOverride`/`YSort`/`BlendMode`/`MaskInteraction` — ausente = default, presente = override) custou **zero churn** nos 80 sítios e **ZERO bump de `PROJECT_SCHEMA`** (componente novo cunha blob-key próprio pelo hash do type-name; old files só não têm a key ⇒ default — o precedente do `PhysicsJoint`/W3). Bônus: o tipo original fica intocado, então operações que o reescrevem (o braço `Kind` do apply) **preservam a nova propriedade de graça**.

**Why:** um campo apendado num tipo positional (postcard) força TODO construtor a mudar E um bump de schema (o arquivo antigo lê errado calado); um componente registrado é aditivo. Quando o tipo é construído em muitos lugares, o custo do campo é O(sítios) e RECORRE por propriedade; o componente é O(1). A nota do plano ("apende ao X") foi escrita quando X tinha poucos construtores — re-avalie contra a realidade atual ([[feedback-a-deferral-notes-bar-may-exceed-the-projects-policy]]).

**How to apply:** antes de apendar um campo, `git grep -c "<Tipo> {"` os sítios de construção. Muitos (dezenas) + a propriedade é opcional/tem default neutro ⇒ **componente opcional de presence-override**, não campo. Corrija a nota do plano no mesmo commit. Exceção: um dado que precisa viajar num *recipe* já-positional e não-serializado (ex.: `BodyDesc`, reconstruído no rewind) SIM ganha o campo lá — mas isso é a fronteira interna, não o componente ECS autorado. Relacionado: [[feedback-a-default-that-fits-the-majority-is-still-a-law]] · [[feedback-frozen-contract-can-pick-the-architecture]].
