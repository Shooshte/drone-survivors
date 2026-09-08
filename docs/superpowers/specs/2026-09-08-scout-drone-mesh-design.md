# Scout drone mesh

Replace the combat arena player placeholder with an original Blender-authored scout based on the supplied reference images and design notes. Use a narrow hover-bike silhouette, silver split armor over a graphite chassis, cyan running lights, three compact underside hover rotors (one beneath each wing and one beneath the nose), four forward sensor lenses, swept fins, an antenna, and two small modular equipment housings. The sensors dominate the weapons. No cockpit.

Export a self-contained GLB at assets/models/scout_drone.glb with +Y up and -Z forward. All geometry must fit the existing half extents (35, 15, 45). The mesh is 2.5× the initial design size, with rotor diameter reduced to 62% relative to the body. Scale is baked into the GLB and .blend. Keep the player entity as the movement/combat root, and load the visual scene beneath it. Mesh loading must not affect health, movement speed, projectiles, or restarts. Movement and contact bounds follow the enlarged model. Passive abilities are visual design context; XP, terrain modifiers, core-slot mechanics and balance changes are outside this mesh task.

Blender is preferred over procedural Bevy geometry because it provides an editable art source and a portable mesh. A primitive-only Bevy model would be lighter to author but less useful for future art work; an external texture workflow would add unnecessary dependencies for this metallic design. Retain the .blend and deterministic build script, and provide a rendered preview.

Validation: parse the actual GLB, check every transformed vertex against collision bounds, validate embedded material/geometry data, run the existing arena/combat tests, fmt and Clippy, and visually inspect Blender and Bevy renders.
