# Oddio

Oddio is a game-oriented audio library that is:

- **Lightweight**: Fast compilation, few dependencies, and a simple interface
- **Sans I/O**: Send output wherever you like
- **Real-time**: Audio output is efficient and wait-free: no glitches until you run out of CPU
- **3D**: Spatialization with doppler effects and propagation delay available out of the box
- **Extensible**: Implement `Signal` for custom streaming synthesis and filtering
- **Composable**: `Signal`s can be transformed without obstructing the inner `Signal`'s controls

### Example

```rust
let (mut scene_handle, scene) = oddio::split(oddio::SpatialScene::new(sample_rate, 0.1));

// In audio callback:
let out_frames = oddio::frame_stereo(data);
oddio::run(&scene, output_sample_rate, out_frames);

// In game logic:
let frames = oddio::FramesSignal::from(oddio::Frames::from_slice(sample_rate, &frames));
let mut handle = scene_handle.control::<oddio::SpatialScene, _>()
    .play(frames, position, velocity, 1000.0);

// When position/velocity changes:
handle.control::<oddio::Spatial<_>, _>().set_motion(position, velocity);
```
