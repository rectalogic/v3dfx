// Copyright (C) 2026 Andrew Wason
// SPDX-License-Identifier: GPL-3.0-or-later

use std::ffi::CStr;

use super::{PluginInfo, V3dfxPlugin};

pub type SourcePlugin = V3dfxPlugin<SourceInfo, 0>;

pub struct SourceInfo;
impl PluginInfo for SourceInfo {
    const NAME: &'static CStr = c"3D mixer3";
    const EXPLANATION: &'static CStr = c"Renders 3D scenes with 3 input videos";
}
