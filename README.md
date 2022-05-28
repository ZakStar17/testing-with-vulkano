# Vulkano testing zone

Welcome to the repository where I mostly test new concepts for me related to Vulkan and graphics programming. You are free to see the project and use it as you wish.

The current used Vulkano version is `0.29.0`, latest at the time of writing this project.

## Current state of this project

- 3d scene with multiple different models;
- First person camera with flying controls;
- For now no lighting, complex models or textures;
  
I try to optimize everything as much as I can (without complicating everything too much).
This is how thing are currently drawn:

- Single vertex, uniform and instance buffer (with no indirect drawing);
- Special command buffer that uses push constants to calculate model-projection-view matrices;
- Multiple main command buffers that do not get recreated each frame;

Currently working on:

- Textures and model loading;

Current problems:

- Buffers used in copying instance information don't get unlocked after signaling their fence. I tested with a simplified file (with the same operations) and everything worked fine, so most probably it's something conflicting between the other objects. More information in comments from inside the project.
  
## Controls

- AWSD: Normal movement;
- Space / LControl: Go up / down;
- LShift: Sprint (go faster);
- C: Lock / unlock mouse;
- Mouse wheel: Zoom;
- Arrow keys: Move first square;
- Numpad: Move first cube;

## Running program and reading docs

- `cargo run --release` to run in release (optimized mode);
- `cargo doc --open` to build and open project documentation;
