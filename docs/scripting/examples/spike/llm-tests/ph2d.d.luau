--!strict
-- Type declarations for PH2D Luau scripting API (spike S3 surface).
-- Mock para luau-analyze validar os 5 scripts canônicos do C8.
-- Implementação real virá em S2/S3 do roadmap pós-spike.

export type Entity = number
export type Component = string

export type Vec2 = { x: number, y: number }

export type QueryResult = {
    entity: Entity,
    [string]: any,
}

export type MessagePayload = { [string]: any }

declare ph2d: {
    -- Coroutines (C3 implementado)
    wait: (seconds: number) -> (),

    -- ECS (canônico = bevy_ecs 0.18, vide ADR-0003-rev2)
    spawn: (...any) -> Entity,
    despawn: (e: Entity) -> (),
    get: <T>(e: Entity, component: Component) -> T?,
    set: <T>(e: Entity, component: Component, value: T) -> (),
    query: (components: { Component }) -> { QueryResult },

    -- Sistemas (registered em pipeline)
    system: (name: string, fn: (dt: number) -> ()) -> (),

    -- Mensageria (estilo Defold; FIFO same-sender→same-target, hash interning)
    message_send: (target: Entity, message: string, payload: MessagePayload?) -> (),
    message_handler: (message: string, fn: (sender: Entity, payload: MessagePayload) -> ()) -> (),

    -- Storage lateral (HR-16: serializável, determinístico)
    state_table: (e: Entity) -> { [string]: any },

    -- Dialogue (storage lateral especializado)
    dialogue: {
        say: (text: string) -> (),
        choice: (options: { string }) -> number,
    },

    -- Animation primitives (tween)
    animate: {
        tween: (start: number, finish: number, duration: number, ease: string?) -> (frame: number) -> number,
    },
}
