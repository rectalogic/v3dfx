// Copyright (C) 2026 Andrew Wason
// SPDX-License-Identifier: GPL-3.0-or-later

use std::ffi::CStr;

use super::{PluginInfo, V3dfxPlugin};

pub type Mixer2Plugin = V3dfxPlugin<Mixer2Info, 2>;

pub struct Mixer2Info;
impl PluginInfo for Mixer2Info {
    const NAME: &'static CStr = c"3D mixer2";
    const EXPLANATION: &'static CStr = c"Renders 3D scenes with 2 input videos";
}
