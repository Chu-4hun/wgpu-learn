use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{
    asset_manager::obj_import::ObjLoader,
    gpu::context::GpuContext,
    model::Model,
    texture::Texture,
};

pub(crate) mod io;
pub mod obj_import;

pub struct AssetManager {
    pub gpu_context: Arc<GpuContext>,
    pub model_cache: HashMap<PathBuf, Arc<Model>>,
    texture_layout: wgpu::BindGroupLayout,
}

impl AssetManager {
    pub fn new(gpu_context: Arc<GpuContext>) -> Self {
        let model_cache: HashMap<PathBuf, Arc<Model>> = HashMap::new();
        let texture_layout = Texture::create_bind_group_layout(&gpu_context.device);

        Self {
            gpu_context,
            model_cache,
            texture_layout,
        }
    }
    pub async fn load_obj(&mut self, path: impl AsRef<Path>) -> anyhow::Result<Arc<Model>> {
        let path = path.as_ref().to_path_buf();
        if let Some(model) = self.model_cache.get(&path) {
            return Ok(model.clone());
        }

        let model =
            Arc::new(ObjLoader::load_model(&path, &self.gpu_context, &self.texture_layout).await?);

        self.model_cache.insert(path, model.clone());

        Ok(model)
    }
    
    pub fn texture_layout(&self) -> &wgpu::BindGroupLayout {
        &self.texture_layout
    }
}
