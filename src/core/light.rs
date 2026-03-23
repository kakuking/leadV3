use crate::core::interaction::Interaction;

    //    VisibilityTester(const Interaction &p0, const Interaction &p1)
    //        : p0(p0), p1(p1) { }
    //    const Interaction &P0() const { return p0; }
    //    const Interaction &P1() const { return p1; }
    //    bool Unoccluded(const Scene &scene) const;
    //    Spectrum Tr(const Scene &scene, Sampler &sampler) const;

pub struct VisibilityTester {
    p0: Interaction,
    p1: Interaction
}