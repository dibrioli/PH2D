// comandos.mjs — a tabela ÚNICA de comandos do oráculo (lida pelo exportador e pelo driver).
//
// Cada comando é UMA acção PÚBLICA da extensão Health, com os parâmetros fixos que
// não cabem numa variável (os `yesorno`, que o GDevelop recebe como literal "yes"/"no").
// O exportador gera, para cada ranhura e cada comando, um evento STANDARD do GDevelop:
//
//     condição  BuiltinCommonInstructions::CompareNumbers(CmdN = id)
//     acção     Health::Health::<Acção>(Hero, Health, <params>)
//
// ⇒ a acção passa pelo GERADOR DE CÓDIGO de eventos do GDevelop (conversão de "yes",
// selecção de objectos, contexto de função), exactamente como num jogo autorado.

/** @typedef {{ nome: string, tipo: string, params: (s: number) => string[] }} Comando */

const num = (s) => `A${s}`; // o argumento numérico da ranhura s (variável de cena)

/** @type {Comando[]} */
export const COMANDOS = [
  // Hit(DamageValue, UseShield, UseArmor)
  { nome: "Hit", tipo: "Health::Health::Hit", params: (s) => [num(s), "no", "no"] },
  { nome: "Hit+Shield", tipo: "Health::Health::Hit", params: (s) => [num(s), "yes", "no"] },
  { nome: "Hit+Armor", tipo: "Health::Health::Hit", params: (s) => [num(s), "no", "yes"] },
  { nome: "Hit+Shield+Armor", tipo: "Health::Health::Hit", params: (s) => [num(s), "yes", "yes"] },
  { nome: "SetHealth", tipo: "Health::Health::SetHealth", params: (s) => [num(s)] },
  { nome: "Heal", tipo: "Health::Health::Heal", params: (s) => [num(s)] },
  { nome: "AllowOverHealing:yes", tipo: "Health::Health::AllowOverHealing", params: () => ["yes"] },
  { nome: "AllowOverHealing:no", tipo: "Health::Health::AllowOverHealing", params: () => ["no"] },
  { nome: "TriggerDamageCooldown", tipo: "Health::Health::TriggerDamageCooldown", params: () => [] },
  { nome: "RenewShieldDuration", tipo: "Health::Health::RenewShieldDuration", params: () => [] },
  { nome: "ActivateShield:renew", tipo: "Health::Health::ActivateShield", params: (s) => [num(s), "yes"] },
  { nome: "ActivateShield:norenew", tipo: "Health::Health::ActivateShield", params: (s) => [num(s), "no"] },
  { nome: "SetShieldBlockExcessDamage:yes", tipo: "Health::Health::SetShieldBlockExcessDamage", params: () => ["yes"] },
  { nome: "SetShieldBlockExcessDamage:no", tipo: "Health::Health::SetShieldBlockExcessDamage", params: () => ["no"] },
  // Acções com operador (ActionWithOperator): Set<X>Op(operador, valor). Usamos sempre "=".
  ...[
    "SetMaxHealthOp",
    "SetHealthRegenRateOp",
    "SetHealthRegenDelayOp",
    "SetCooldownDurationOp",
    "SetChanceToDodgeOp",
    "SetFlatDamageReductionOp",
    "SetPercentDamageReductionOp",
    "SetMaxShieldOp",
    "SetShieldPointsOp",
    "SetShieldRegenRateOp",
    "SetShieldRegenDelayOp",
    "SetShieldDurationOp",
  ].map((op) => ({ nome: op, tipo: `Health::Health::${op}`, params: (s) => ["=", num(s)] })),
];

/** Quantas acções cabem num mesmo quadro (ranhuras executadas por ordem 1..N). */
export const RANHURAS = 4;

/** id numérico de cada comando (0 = ranhura vazia). */
export const ID_DE = Object.fromEntries(COMANDOS.map((c, i) => [c.nome, i + 1]));
