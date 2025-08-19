# playground-systems-physics

Basic 2D physics system with simple gravity and force simulation (placeholder for future expansion).

## Overview

The Physics System provides basic 2D physics simulation. Currently implements simple Newtonian mechanics with gravity and forces. This is a minimal implementation that will be expanded for game development needs.

### Current Features
- 2D vector math (position, velocity, acceleration)
- Basic physics bodies with mass
- Gravity simulation
- Force application
- Static and dynamic bodies
- Simple Euler integration

## Status: Minimal Implementation

This system is a placeholder for future physics functionality. The current implementation is sufficient for basic movement but lacks:
- Collision detection
- Rigid body dynamics
- Joints and constraints
- Spatial partitioning
- Continuous collision detection
- 3D support

## Usage

### Basic Setup
```rust
use playground_systems_physics::{PhysicsSystem, PhysicsBody, Vector2};

// Create and initialize physics system
let mut physics = PhysicsSystem::new();
physics.initialize()?;

// Set gravity (default is Earth-like: 0, -9.81)
physics.set_gravity(Vector2::new(0.0, -9.81));
```

### Creating Bodies
```rust
// Create dynamic body
let player = PhysicsBody::new(
    "player".to_string(),
    Vector2::new(0.0, 10.0), // Start position
)
.with_mass(70.0); // 70kg

physics.add_body(player)?;

// Create static body (doesn't move)
let ground = PhysicsBody::new(
    "ground".to_string(),
    Vector2::new(0.0, 0.0),
)
.as_static();

physics.add_body(ground)?;
```

### Simulation Loop
```rust
// Game loop
loop {
    let delta_time = 1.0 / 60.0; // 60 FPS
    
    // Apply forces
    physics.apply_force("player", Vector2::new(100.0, 0.0))?; // Push right
    
    // Step simulation
    physics.step(delta_time)?;
    
    // Get updated positions
    if let Some(body) = physics.get_body("player") {
        println!("Player at: ({}, {})", body.position.x, body.position.y);
    }
}
```

### Vector Math
```rust
use playground_systems_physics::Vector2;

let v1 = Vector2::new(3.0, 4.0);
let v2 = Vector2::new(1.0, 2.0);

// Basic operations (manual for now)
let sum = Vector2::new(v1.x + v2.x, v1.y + v2.y);
let magnitude = v1.magnitude(); // 5.0
let normalized = v1.normalize(); // (0.6, 0.8)
```

## PhysicsBody Structure
```rust
pub struct PhysicsBody {
    pub id: String,
    pub position: Vector2,
    pub velocity: Vector2,
    pub acceleration: Vector2,
    pub mass: f32,
    pub is_static: bool,
}
```

## Future Roadmap

### Phase 1: Collision Detection (Next)
- AABB collision detection
- Circle collision detection
- Collision callbacks
- Trigger volumes

### Phase 2: Advanced Dynamics
- Rotational dynamics
- Friction and restitution
- Constraints and joints
- Compound shapes

### Phase 3: Optimization
- Spatial partitioning (quadtree/octree)
- Broad phase optimization
- Continuous collision detection
- Island sleeping

### Phase 4: 3D Support
- 3D vectors and transforms
- 3D collision shapes
- Quaternion rotations
- Mesh colliders

### Phase 5: Integration
- Integration with ECS (use core/ecs)
- Network synchronization
- Deterministic simulation
- Physics materials system

## Architecture Notes

When expanded, this system will:
- Use core/ecs for internal state management
- Integrate with rendering for debug visualization
- Support both 2D and 3D physics
- Provide deterministic simulation for networking
- Use fixed timestep with interpolation

## Dependencies

Currently minimal:
- `serde`: Serialization
- `thiserror`: Error handling

Future dependencies will include:
- `playground-core-ecs`: State management
- `nalgebra`: Advanced math
- `parry2d/3d`: Collision detection (maybe)
- `rapier2d/3d`: Full physics engine (maybe)

## See Also

- [systems/logic](../logic/README.md) - Will manage this system
- [plugins/combat](../../plugins/combat/README.md) - Will use physics
- [plugins/platformer](../../plugins/platformer/README.md) - Future physics demo