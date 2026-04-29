# Spring Bone Secondary Motion Contracts

## Purpose

Spring bones add lightweight secondary motion to modular overlays.

Use cases:

- hair modules,
- ponytails,
- animal tails,
- ears,
- scarves,
- straps,
- bags,
- tool charms,
- hanging plants/signs,
- fishing line.

Base character templates must remain bald and clean-shaven. Spring bone systems apply to modular overlays only.

## SpringBoneProfileDef

```rust
pub struct SpringBoneProfileDef {
    pub id: String,
    pub display_name: String,
    pub chains: Vec<SpringBoneChainDef>,
    pub collision: SpringBoneCollisionDef,
    pub simulation: SpringBoneSimulationDef,
}
```

## SpringBoneChainDef

```rust
pub struct SpringBoneChainDef {
    pub id: String,
    pub root_bone: String,
    pub bones: Vec<String>,
    pub stiffness: f32,
    pub damping: f32,
    pub gravity: [f32; 3],
    pub drag: f32,
    pub max_angle_degrees: f32,
}
```

## SpringBoneCollisionDef

```rust
pub struct SpringBoneCollisionDef {
    pub enabled: bool,
    pub radius: f32,
    pub capsule_colliders: Vec<String>,
}
```

## SpringBoneSimulationDef

```rust
pub struct SpringBoneSimulationDef {
    pub fixed_timestep: f32,
    pub solver_iterations: u32,
    pub reset_on_teleport: bool,
    pub pause_when_offscreen: bool,
}
```

## Performance rules

- Simulate only nearby visible characters.
- Disable spring motion for distant NPCs.
- Prefer baked motion for crowd scenes.
- Keep hair/strap chains short.
- Editor preview may run higher fidelity than runtime.

## Suggested defaults

```txt
Hair strand chain: 3–5 bones
Scarf chain:       4–8 bones
Tail chain:        4–7 bones
Bag strap chain:   2–4 bones
Tool charm chain:  2–3 bones
```

