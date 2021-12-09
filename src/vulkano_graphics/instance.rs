use std::sync::Arc;
use vulkano::instance::Instance;
use vulkano::Version;

const LIST_AVAILABLE_LAYERS: bool = true;

const ENABLE_VALIDATION_LAYERS: bool = false;

// "VK_LAYER_LUNARG_screenshot", "VK_LAYER_LUNARG_monitor", "VK_LAYER_LUNARG_gfxreconstruct", "VK_LAYER_LUNARG_device_simulation", "VK_LAYER_LUNARG_api_dump"
const VALIDATION_LAYERS: &[&str] = &["VK_LAYER_LUNARG_api_dump"];

pub fn create() -> Arc<Instance> {
  let required_extensions = vulkano_win::required_extensions();

  if LIST_AVAILABLE_LAYERS {
    let layers: Vec<_> = vulkano::instance::layers_list().unwrap().collect();
    let layer_names = layers.iter().map(|l| l.name());
    println!(
      "Using layers {:?}",
      layer_names.clone().collect::<Vec<&str>>()
    );
  }

  if ENABLE_VALIDATION_LAYERS {
    Instance::new(
      None,
      Version::V1_1,
      &required_extensions,
      VALIDATION_LAYERS.iter().cloned(),
    )
    .unwrap()
  } else {
    Instance::new(None, Version::V1_1, &required_extensions, None).unwrap()
  }
}