# Lead V3
Because 3rd times the charm!!!
Following [PBR Book, 3rd edition](https://pbr-book.org/3ed-2018)

## Sample outputs of the renderer (512x512)
Output of the volume path tracing integrator rendering a heterogeneous medium shaped like a cloud devil, 64 samples per pixel

![Volume Path Tracing Integrator Output with a Heterogeneous Medium shaped like a Cloud](https://github.com/kakuking/leadV3/blob/ac100ef85ca4be562c1d2fa58ffcdd41028b80c3/outputs/volume_display_cloud.png)

Output of the volume path tracing integrator rendering a heterogeneous medium shaped like a dust devil, 64 samples per pixel

![Volume Path Tracing Integrator Output with a Heterogeneous Medium shaped like a Dust Devil](https://github.com/kakuking/leadV3/blob/ac100ef85ca4be562c1d2fa58ffcdd41028b80c3/outputs/volume_display_devil.png)

Output of the volume path tracing integrator on a cornell-box-like input with a heterogeneous medium shaped like a Blender Monkey, 64 samples per pixel

![Volume Path Tracing Integrator Output with a Heterogeneous Medium shaped like the Blender Monkey](https://github.com/kakuking/leadV3/blob/ac100ef85ca4be562c1d2fa58ffcdd41028b80c3/outputs/volume_cornell_monkey.png)

Output of the volume path tracing integrator on a cornell-box-like input with a specular sphere that has a homogeneous medium inside it, 64 samples per pixel

![Volume Path Tracing Integrator Output with a Fresnel Sphere with an Occupying Homogeneous Medium](https://github.com/kakuking/leadV3/blob/ac100ef85ca4be562c1d2fa58ffcdd41028b80c3/outputs/volume_cornell_glass_media.png)

Output of the path tracing integrator on a cornell-box-like input. 144 samples per pixel

![Path Tracing Integrator Output](https://github.com/kakuking/leadV3/blob/ac100ef85ca4be562c1d2fa58ffcdd41028b80c3/outputs/path_cornell_meshes.png)

Another output of the path tracing integrator on a cornell-box-like input with a glass sphere in the center. 144 samples per pixel

![Path Tracing Integrator Output with a Glass Sphere](https://github.com/kakuking/leadV3/blob/ac100ef85ca4be562c1d2fa58ffcdd41028b80c3/outputs/path_cornell_glass.png)

Output of the direct-lighting integrator on a cornell-box-like input. 144 samples per pixel

![Direct Lighting Integrator Output](https://github.com/kakuking/leadV3/blob/ac100ef85ca4be562c1d2fa58ffcdd41028b80c3/outputs/direct_cornell.png)

Output of the color integrator on the cornell-box-like input. 144 samples per pixel

![Color Integrator Output](https://github.com/kakuking/leadV3/blob/ac100ef85ca4be562c1d2fa58ffcdd41028b80c3/outputs/color_cornell_empty.png)

Output of the normal renderer, which on intersection, converts the normal at intersection point to a color. 4 samples per pixel

![Normal Renderer Output](https://github.com/kakuking/leadV3/blob/ac100ef85ca4be562c1d2fa58ffcdd41028b80c3/outputs/normal_monkeys.png)

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
- [x] Fixed the glass material
- [x] Added Homogeneous Medium, and HG Phase function
- [x] Added Volume Path Integrator
- [x] Added BSSRDF and Tabulated BSSRDF (dont really use it rn tho)
- [x] Added heterogeneous media
- [x] Added VDB loading
- [x] Added directional light
- [x] Fixed heterogeneous volume (scales properly now)
- [ ] Add more lights, integrators, and so on...