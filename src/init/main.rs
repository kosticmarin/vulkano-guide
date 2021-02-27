use vulkano::device::Device;
use vulkano::device::DeviceExtensions;
use vulkano::device::Features;
use vulkano::instance::Instance;
use vulkano::instance::InstanceExtensions;
use vulkano::instance::PhysicalDevice;

fn main() {
    // Initialization
    let instance = Instance::new(None, &InstanceExtensions::none(), None)
        .expect("failed to create vulkan instance");

    let physical = PhysicalDevice::enumerate(&instance)
        .next()
        .expect("no device available");

    // Device creation
    let queue_family = physical
        .queue_families()
        .find(|&q| q.supports_graphics())
        .expect("couldn't find a graphical queue family");

    #[allow(unused_variables)]
    let (device, queues) = {
        Device::new(
            physical,
            &Features::none(),
            &DeviceExtensions::none(),
            [(queue_family, 0.5)].iter().cloned(),
        )
        .expect("failed device creation")
    };
}
