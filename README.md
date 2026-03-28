# Lead V3
Because 3rd times the charm!!!
Following [PBR Book, 3rd edition](https://pbr-book.org/3ed-2018)

## Sample outputs of the renderer (512x512)
Output of the path tracing integrator on a cornell-box-like input. 144 samples per pixel
![Path Tracing Integrator Output](https://github.com/kakuking/leadV3/blob/ad5a98cda0a84d7b3b2f2ee37e32205f0363b9c4/z_output/path.png)

Another output of the path tracing integrator on a cornell-box-like input with a glass sphere in the center. 144 samples per pixel
![Path Tracing Integrator Output with a Glass Sphere](https://github.com/kakuking/leadV3/blob/ad5a98cda0a84d7b3b2f2ee37e32205f0363b9c4/z_output/glass_path.png)

Output of the direct-lighting integrator on a cornell-box-like input. 144 samples per pixel
![Direct Lighting Integrator Output](https://github.com/kakuking/leadV3/blob/dc15ba4516de2cfe44917c37d0d6df8f9cee618d/z_output/direct.png)

Output of the color integrator on the cornell-box-like input. 144 samples per pixel
![Color Integrator Output](https://github.com/kakuking/leadV3/blob/a4dcc7a1ece4c7a44ba954d493d1cbbe43ead0b2/z_output/color.png)

Output of the normal renderer, which on intersection, converts the normal at intersection point to a color. 4 samples per pixel
![Normal Renderer Output](https://github.com/kakuking/leadV3/blob/4447d525c2316d0460d7f4fc8f4ab80cc32280de/output/normal.png)

- [x] Added Basic Maths 
- [x] Added Vectors, Normals, Points, Bounding Boxes
- [x] Added Interaction and Surface Interactions
- [x] Added Shapes
- [x] Added Sphere
- [x] Added XML loading
- [x] Add meshes
- [x] Added BVH Acceleration Structure
- [x] Added benchmark between brute force and BVH
- [x] Added Triangle Meshes (including loading)
- [x] Added a test scene  of loading and rendering an image
- [x] Added Camera Enum and Orthographic Camera
- [x] Added Sampler Enum and Stratified Sampler
- [x] Added Transform handling to scene
- [x] Add Filters and Film 
- [x] Added scene loading to filters and films
- [x] Added Lights and Point Lights
- [x] Added Light loading to scene
- [x] Integrators to come and Sampling
- [x] Added integrators to scene loading and handling
- [x] Calling Render just works
- [x] Added saving to png/exr/ppm option (by inputting filename)
- [x] Added Normal Integrator
- [x] Scene handles primitives not shapes now
- [x] Fixed a bug in computing bsdf at intersected
- [x] Added Area Light and Diffuse Area Light
- [x] Finish direct integrator
- [x] Added Perfect Mirror material
- [x] Added Checkerboard, UV, Constant, Scale Textures and added them to scene handling
- [x] Added Path Tracing renderer
- [x] Fixed a terrible bug in BxDF and BxDFt that swapped wo and wi
- [x] Added a glass material
- [ ] Add more lights, integrators, and so on...