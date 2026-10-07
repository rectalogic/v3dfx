// Copyright (C) 2026 Andrew Wason
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    ffi::{CStr, CString},
    marker::PhantomData,
};

use crate::renderer::{Renderer, Scene};

pub mod filter;
pub mod mixer2;
pub mod mixer3;
pub mod source;

pub struct V3dfxPlugin<PI: PluginInfo, const S: usize> {
    gltf_path: CString,

    renderer: Renderer<S>,
    scene: Option<Result<Scene<S>, ()>>,
    _phantom: PhantomData<PI>,
}

impl<PI, const S: usize> V3dfxPlugin<PI, S>
where
    PI: PluginInfo,
{
    fn new(width: u32, height: u32) -> Self {
        env_logger::init();
        Self {
            gltf_path: c"".to_owned(),
            renderer: std::thread::spawn(|| Renderer::new(width, height))
                .join()
                .unwrap(),
            scene: None,
            _phantom: PhantomData,
        }
    }

    fn update(&mut self, time: f64, inframes: [&[u32]; S], outframe: &mut [u32]) {
        let scene = match self.scene {
            Some(Ok(ref mut scene)) => scene,
            Some(Err(())) => return,
            None => {
                if let Ok(gltf_path) = self.gltf_path.to_str()
                    && let Ok(gltf_path) = std::path::absolute(gltf_path)
                    && let Some(gltf_path) = gltf_path.to_str()
                {
                    match self.renderer.load(gltf_path) {
                        Ok(scene) => {
                            self.scene = Some(Ok(scene));
                            self.scene.as_mut().unwrap().as_mut().unwrap()
                        }
                        Err(e) => {
                            log::error!("v3dfx: failed to create renderer scene `{e:?}'");
                            self.scene = Some(Err(()));
                            return;
                        }
                    }
                } else {
                    log::error!("v3dfx: invalid gltf_path `{:?}'", self.gltf_path);
                    self.scene = Some(Err(()));
                    return;
                }
            }
        };

        if let Err(e) = self.renderer.render(time, scene, inframes, outframe) {
            log::error!("v3dfx: failed to render frame: {e:?}");
            self.scene = Some(Err(()));
        }
    }
}

pub trait PluginInfo {
    const NAME: &'static CStr;
    const EXPLANATION: &'static CStr;
}

impl<PI, const S: usize> frei0r_rs2::Plugin<S> for V3dfxPlugin<PI, S>
where
    PI: PluginInfo + Send + 'static,
{
    const PARAMS: &'static [frei0r_rs2::ParamInfo<Self>] = &[frei0r_rs2::ParamInfo::new_string(
        c"gltf_path",
        c"Path to GLTF/GLB scene file",
        |plugin| plugin.gltf_path.as_c_str(),
        |plugin, value| value.clone_into(&mut plugin.gltf_path),
    )];

    fn info() -> frei0r_rs2::PluginInfo {
        frei0r_rs2::PluginInfo {
            name: PI::NAME,
            author: c"Andrew Wason",
            color_model: frei0r_rs2::ColorModel::RGBA8888,
            major_version: env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap(),
            minor_version: env!("CARGO_PKG_VERSION_MINOR").parse().unwrap(),
            explanation: Some(PI::EXPLANATION),
        }
    }

    fn new(width: usize, height: usize) -> Self {
        Self::new(width as u32, height as u32)
    }

    fn update(&mut self, time: f64, inframes: [&[u32]; S], outframe: &mut [u32]) {
        self.update(time, inframes, outframe)
    }
}
