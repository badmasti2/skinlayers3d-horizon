# 3D Skin Layers for Horizon — prototype 0.1.0

An independent native implementation of the requested 3D skin-overlay effect.
This is not the Fabric JAR converted to Rust and is not an official tr7zw release.
No Java mod source or assets are included.

## Included

- Pixel-depth hat, jacket, sleeves and trouser overlays for classic and slim players.
- Player-head hat geometry through Horizon's shared model loader.
- Existing model transforms, animations and layer visibility are preserved.
- Transparent pixels use the renderer's existing texture alpha test, including on the pixel sides.
- Geometry builds once per model load; skins can change without rebuilding geometry.

## Compatibility and status

Pinned to Horizon c9d474dd9116e84c243f8992d75c11314df56f06 (Nightly 2026-11-01),
the source revision used by the installed native build when this was prepared.
Uses horizon-mixins 4342a0bfbe71d4d14185528d64e496d6022f16a1.

The Horizon mixin planner accepts the target and module. Four isolated Rust tests
pass for texel counts, classic/slim dimensions, transparency sampling, mirrored UVs,
outward normals, face winding, preservation of poses and exclusion of other models.
These tests use matching lightweight model types, not a full compiled game.

**The user reports that this version works in Horizon.**
Automated full-game integration and performance checks have not been run.

This version uses the standard 64x64 skin pixel grid and a fixed depth of 0.75
model units into the original overlay surface. It does not reproduce the original
mod's configuration UI, distance fallback, special ears, compatibility integrations
or optimization that removes hidden faces. It generates geometry for transparent
pixels too and relies on GPU alpha rejection; crowded scenes can cost more to render.

## Build

After explicitly trusting this native code, build the unpacked folder with:

```powershell
horizon-launcher native-build "PATH_TO_SOURCE" --instance Horizon --trust-native-code
```

The launcher must be on PATH, or use its installed executable's full path.
The launcher may display its own trust prompt. Native mods compile into a separate
game build and run with your account's permissions.

Check the result with a skin that has transparent holes in its outer layer, in
third person and the inventory preview. Check slim arms, walking, crouching, first
person sleeves, heads, hidden layer settings and frame rate. Other enabled native
mods must also pass the combined build. The package does not change saves.

To remove the effect, return to the previous game build or disable this mod and
rebuild the instance through the launcher.

## Source

The package contains one cancellable HEAD injection into
`horizon_client::mobs::EntityModelDocument::model` and a support module.
Only the three specified player model keys are intercepted; every other key
continues through Horizon's original loader.

License: MIT. Original implementation, 2026.


## Run isolated mesh tests

rustc --edition 2024 --test tests/mesh.rs -o mesh-tests

Run the resulting mesh-tests executable. No Minecraft assets are required.

