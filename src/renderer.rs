// Copyright (C) 2026 Andrew Wason
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::Path;

use frei0r_rs2::{slice_to_bytes, slice_to_bytes_mut};
use renderling::{
    atlas::{AtlasImage, AtlasImageFormat, AtlasTexture, convert_pixels},
    context::Context,
    glam::UVec2,
    gltf::{Animator, GltfDocument, GltfNode},
    material::Materials,
    stage::Stage,
};

const VIDEO_NAME_PREFIX: &str = "Video";

pub struct Renderer<const S: usize> {
    width: u32,
    height: u32,
    context: Context,
    stage: Stage,
}

pub struct Scene<const S: usize> {
    video_textures: [AtlasTexture; S],
    animators: Vec<Animator>,
}

impl<const S: usize> Renderer<S> {
    pub fn new(width: u32, height: u32) -> Self {
        let context =
            futures_lite::future::block_on(async { Context::headless(width, height).await });
        let stage: Stage = context.new_stage();
        Self {
            width,
            height,
            context,
            stage,
        }
    }

    pub fn load(&mut self, gltf_path: impl AsRef<Path>) -> anyhow::Result<Scene<S>> {
        let (schema, buffers, images) = gltf::import(gltf_path.as_ref())?;
        let document = GltfDocument::from_gltf(&self.stage, &schema, buffers, images)?;

        let video_textures = self.load_video_textures(&schema, &document)?;
        let animators = Self::load_animators(&document);

        let _model = document.into_gpu_only();

        Ok(Scene {
            video_textures,
            animators,
        })
    }

    fn load_video_textures(
        &self,
        schema: &gltf::Document,
        document: &GltfDocument,
    ) -> anyhow::Result<[AtlasTexture; S]> {
        let blanks: [AtlasImage; S] = core::array::from_fn(|_| AtlasImage {
            pixels: vec![0u8; (self.width * self.height * 4) as usize],
            size: UVec2::new(self.width, self.height),
            format: AtlasImageFormat::R8G8B8A8,
            apply_linear_transfer: true,
        });
        let video_textures: [AtlasTexture; S] = self
            .stage
            .add_images(blanks)?
            .try_into()
            .map_err(|_| anyhow::anyhow!("const size"))?;
        (0..S).for_each(|i| {
            let name = format!("{VIDEO_NAME_PREFIX}{}", i + 1);
            if let Some(video_material) = schema.materials().find(|m| m.name() == Some(&name))
                && let Some(material_index) = video_material.index()
            {
                document.materials[material_index].set_albedo_texture(&video_textures[i]);
            }
        });
        Ok(video_textures)
    }

    fn load_animators(document: &GltfDocument) -> Vec<Animator> {
        // Animators: one per clip, driving every node in the scene
        let all_nodes: Vec<&GltfNode> = document
            .recursive_nodes_in_scene(document.default_scene.unwrap_or_default())
            .collect();
        document
            .animations
            .clone()
            .into_iter()
            .map(|anim| Animator::new(all_nodes.iter().cloned(), anim))
            .collect()
    }

    pub fn render(
        &mut self,
        time: f64,
        scene: &mut Scene<S>,
        inframes: [&[u32]; S],
        output: &mut [u32],
    ) -> anyhow::Result<()> {
        #[expect(clippy::needless_range_loop)]
        for i in 0..S {
            let materials: &Materials = self.stage.as_ref();
            let atlas = materials.atlas();
            let desc = scene.video_textures[i].descriptor();
            let bytes = convert_pixels(
                slice_to_bytes(inframes[i]).iter().copied(),
                AtlasImageFormat::R8G8B8A8,
                atlas.get_texture().texture.format(),
                true,
            );
            self.context.runtime().queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &atlas.get_texture().texture,
                    mip_level: 0,
                    origin: desc.origin(),
                    aspect: wgpu::TextureAspect::All,
                },
                &bytes,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * desc.size_px.x),
                    rows_per_image: Some(desc.size_px.y),
                },
                desc.size_as_extent(),
            );
        }

        for a in scene.animators.iter_mut() {
            a.progress(0.33 /*XXX need delta */)?;
        }

        let frame = self.context.get_next_frame()?;
        self.stage.render(&frame.view());
        let image = futures_lite::future::block_on(async { frame.read_image().await })?;
        slice_to_bytes_mut(output).copy_from_slice(&image);
        frame.present();

        Ok(())
    }
}
