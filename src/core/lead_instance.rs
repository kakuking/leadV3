use crate::core::{camera::Camera, integrator::Integrator, sampler::Sampler, scene::Scene};

pub struct Instance {
    pub scene: Scene,
    integrator: Integrator
}

impl Instance {
    pub fn new() -> Self {
        Self {
            scene: Scene::new(),

            // camera: Camera::Empty,
            // sampler: Sampler::Empty,
            integrator: Integrator::Empty
        }
    }

    pub fn set_scene(&mut self, scene: Scene) { self.scene = scene; }
    pub fn set_camera(&mut self, camera: Camera) { self.integrator.set_camera(camera); }
    pub fn set_sampler(&mut self, sampler: Sampler) { self.integrator.set_sampler(sampler); }
    pub fn set_integrator(&mut self, integrator: Integrator) { self.integrator = integrator; }

    pub fn get_integrator(&mut self) -> &mut Integrator { &mut self.integrator }

    pub fn init_scene(&mut self) { self.scene.init(); }

    pub fn preprocess(&mut self) {
        self.integrator.preprocess(&self.scene);
    }

    pub fn render(&mut self) {
        self.integrator.render(&self.scene);
    }
}