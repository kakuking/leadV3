# Lead V3
Because 3rd times the charm!!!
Following [PBR Book, 3rd edition](https://pbr-book.org/3ed-2018)

## Sample outputs of the renderer (512x512 / 4 samples per pixel)
Output of the direct lighting integrator
![Direct Lighting Integrator Output](https://github.com/kakuking/leadV3/blob/69db778bbe404f57cd3282b10430d69574dca5c2/z_output/direct.png)

Output of the normal renderer, which on intersection, converts the normal at intersection point to a color
![Normal Renderer Output](https://github.com/kakuking/leadV3/blob/4447d525c2316d0460d7f4fc8f4ab80cc32280de/output/normal.png)

Output of the intersection renderer (incomplete direct renderer), which outputs red if an intersection occurs
![Intersection Renderer Output](https://github.com/kakuking/leadV3/blob/4447d525c2316d0460d7f4fc8f4ab80cc32280de/output/intersection.png)

- [x] Added Basic Maths 
- [x] Added Vectors, Normals, Points, Bounding Boxes
- [x] Added Interaction and Surface Interactions
- [x] Added Shapes[[](https://github.com/kakuking/leadV3/blob/3a8fa18cfd6c1d0e0e57859c2f1d9634da59a0d7/z_output/direct.png)](https://github.com/kakuking/leadV3/blob/3a8fa18cfd6c1d0e0e57859c2f1d9634da59a0d7/z_output/direct.png)
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
- [ ] Add more lights, integrators, and so on...