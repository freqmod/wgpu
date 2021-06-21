use glow::HasContext;
use std::sync::Arc;

// https://webgl2fundamentals.org/webgl/lessons/webgl-data-textures.html

impl super::Adapter {
    /// According to the OpenGL specification, the version information is
    /// expected to follow the following syntax:
    ///
    /// ~~~bnf
    /// <major>       ::= <number>
    /// <minor>       ::= <number>
    /// <revision>    ::= <number>
    /// <vendor-info> ::= <string>
    /// <release>     ::= <major> "." <minor> ["." <release>]
    /// <version>     ::= <release> [" " <vendor-info>]
    /// ~~~
    ///
    /// Note that this function is intentionally lenient in regards to parsing,
    /// and will try to recover at least the first two version numbers without
    /// resulting in an `Err`.
    /// # Notes
    /// `WebGL 2` version returned as `OpenGL ES 3.0`
    fn parse_version(mut src: &str) -> Result<(u8, u8), crate::InstanceError> {
        let webgl_sig = "WebGL ";
        // According to the WebGL specification
        // VERSION  WebGL<space>1.0<space><vendor-specific information>
        // SHADING_LANGUAGE_VERSION WebGL<space>GLSL<space>ES<space>1.0<space><vendor-specific information>
        let is_webgl = src.starts_with(webgl_sig);
        if is_webgl {
            let pos = src.rfind(webgl_sig).unwrap_or(0);
            src = &src[pos + webgl_sig.len()..];
        } else {
            let es_sig = " ES ";
            match src.rfind(es_sig) {
                Some(pos) => {
                    src = &src[pos + es_sig.len()..];
                }
                None => {
                    log::warn!("ES not found in '{}'", src);
                    return Err(crate::InstanceError);
                }
            }
        };

        let glsl_es_sig = "GLSL ES ";
        let is_glsl = match src.find(glsl_es_sig) {
            Some(pos) => {
                src = &src[pos + glsl_es_sig.len()..];
                true
            }
            None => false,
        };

        let (version, vendor_info) = match src.find(' ') {
            Some(i) => (&src[..i], src[i + 1..].to_string()),
            None => (src, String::new()),
        };

        // TODO: make this even more lenient so that we can also accept
        // `<major> "." <minor> [<???>]`
        let mut it = version.split('.');
        let major = it.next().and_then(|s| s.parse().ok());
        let minor = it.next().and_then(|s| {
            let trimmed = if s.starts_with('0') {
                "0"
            } else {
                s.trim_end_matches('0')
            };
            trimmed.parse().ok()
        });

        match (major, minor) {
            (Some(major), Some(minor)) => Ok((
                // Return WebGL 2.0 version as OpenGL ES 3.0
                if is_webgl && !is_glsl {
                    major + 1
                } else {
                    major
                },
                minor,
            )),
            _ => {
                log::warn!("Unable to extract the version from '{}'", version);
                Err(crate::InstanceError)
            }
        }
    }

    fn make_info(vendor_orig: String, renderer_orig: String) -> wgt::AdapterInfo {
        let vendor = vendor_orig.to_lowercase();
        let renderer = renderer_orig.to_lowercase();

        // opengl has no way to discern device_type, so we can try to infer it from the renderer string
        let strings_that_imply_integrated = [
            " xpress", // space here is on purpose so we don't match express
            "radeon hd 4200",
            "radeon hd 4250",
            "radeon hd 4290",
            "radeon hd 4270",
            "radeon hd 4225",
            "radeon hd 3100",
            "radeon hd 3200",
            "radeon hd 3000",
            "radeon hd 3300",
            "radeon(tm) r4 graphics",
            "radeon(tm) r5 graphics",
            "radeon(tm) r6 graphics",
            "radeon(tm) r7 graphics",
            "radeon r7 graphics",
            "nforce", // all nvidia nforce are integrated
            "tegra",  // all nvidia tegra are integrated
            "shield", // all nvidia shield are integrated
            "igp",
            "mali",
            "intel",
        ];
        let strings_that_imply_cpu = ["mesa offscreen", "swiftshader", "lavapipe"];

        //TODO: handle Intel Iris XE as discreet
        let inferred_device_type = if vendor.contains("qualcomm")
            || vendor.contains("intel")
            || strings_that_imply_integrated
                .iter()
                .any(|&s| renderer.contains(s))
        {
            wgt::DeviceType::IntegratedGpu
        } else if strings_that_imply_cpu.iter().any(|&s| renderer.contains(s)) {
            wgt::DeviceType::Cpu
        } else {
            wgt::DeviceType::DiscreteGpu
        };

        // source: Sascha Willems at Vulkan
        let vendor_id = if vendor.contains("amd") {
            0x1002
        } else if vendor.contains("imgtec") {
            0x1010
        } else if vendor.contains("nvidia") {
            0x10DE
        } else if vendor.contains("arm") {
            0x13B5
        } else if vendor.contains("qualcomm") {
            0x5143
        } else if vendor.contains("intel") {
            0x8086
        } else {
            0
        };

        wgt::AdapterInfo {
            name: renderer_orig,
            vendor: vendor_id,
            device: 0,
            device_type: inferred_device_type,
            backend: wgt::Backend::Gl,
        }
    }

    pub(super) unsafe fn expose(gl: glow::Context) -> Option<crate::ExposedAdapter<super::Api>> {
        let vendor = gl.get_parameter_string(glow::VENDOR);
        let renderer = gl.get_parameter_string(glow::RENDERER);
        let version = gl.get_parameter_string(glow::VERSION);
        log::info!("Vendor: {}", vendor);
        log::info!("Renderer: {}", renderer);
        log::info!("Version: {}", version);

        let ver = Self::parse_version(&version).ok()?;
        let extensions = gl.supported_extensions();
        log::info!("Extensions: {:?}", extensions);

        let mut features = wgt::Features::empty() | wgt::Features::NON_FILL_POLYGON_MODE;
        features.set(
            wgt::Features::DEPTH_CLAMPING,
            extensions.contains("GL_EXT_depth_clamp"),
        );
        features.set(wgt::Features::VERTEX_WRITABLE_STORAGE, ver >= (3, 1));

        let mut downlevel_flags = wgt::DownlevelFlags::empty()
            | wgt::DownlevelFlags::DEVICE_LOCAL_IMAGE_COPIES
            | wgt::DownlevelFlags::NON_POWER_OF_TWO_MIPMAPPED_TEXTURES
            | wgt::DownlevelFlags::CUBE_ARRAY_TEXTURES
            | wgt::DownlevelFlags::COMPARISON_SAMPLERS;
        downlevel_flags.set(wgt::DownlevelFlags::COMPUTE_SHADERS, ver >= (3, 1));
        downlevel_flags.set(
            wgt::DownlevelFlags::FRAGMENT_WRITABLE_STORAGE,
            ver >= (3, 1),
        );
        downlevel_flags.set(wgt::DownlevelFlags::INDIRECT_EXECUTION, ver >= (3, 1));
        downlevel_flags.set(wgt::DownlevelFlags::BASE_VERTEX, ver >= (3, 2));

        let max_texture_size = gl.get_parameter_i32(glow::MAX_TEXTURE_SIZE) as u32;
        let max_texture_3d_size = gl.get_parameter_i32(glow::MAX_3D_TEXTURE_SIZE) as u32;

        let min_uniform_buffer_offset_alignment =
            gl.get_parameter_i32(glow::UNIFORM_BUFFER_OFFSET_ALIGNMENT);
        let min_storage_buffer_offset_alignment = if cfg!(target_arch = "wasm32") {
            256
        } else {
            gl.get_parameter_i32(glow::SHADER_STORAGE_BUFFER_OFFSET_ALIGNMENT)
        };
        let max_uniform_buffers_per_shader_stage =
            gl.get_parameter_i32(glow::MAX_VERTEX_UNIFORM_BLOCKS)
                .min(gl.get_parameter_i32(glow::MAX_FRAGMENT_UNIFORM_BLOCKS)) as u32;

        let limits = wgt::Limits {
            max_texture_dimension_1d: max_texture_size,
            max_texture_dimension_2d: max_texture_size,
            max_texture_dimension_3d: max_texture_3d_size,
            max_texture_array_layers: gl.get_parameter_i32(glow::MAX_ARRAY_TEXTURE_LAYERS) as u32,
            max_bind_groups: crate::MAX_BIND_GROUPS as u32,
            max_dynamic_uniform_buffers_per_pipeline_layout: 8,
            max_dynamic_storage_buffers_per_pipeline_layout: 4,
            max_sampled_textures_per_shader_stage: 16,
            max_samplers_per_shader_stage: 16,
            max_storage_buffers_per_shader_stage: 8,
            max_storage_textures_per_shader_stage: 8,
            max_uniform_buffers_per_shader_stage,
            max_uniform_buffer_binding_size: gl.get_parameter_i32(glow::MAX_UNIFORM_BLOCK_SIZE)
                as u32,
            max_storage_buffer_binding_size: if ver >= (3, 1) {
                gl.get_parameter_i32(glow::MAX_SHADER_STORAGE_BLOCK_SIZE) as u32
            } else {
                0
            },
            max_vertex_buffers: gl.get_parameter_i32(glow::MAX_VERTEX_ATTRIB_BINDINGS) as u32,
            max_vertex_attributes: gl.get_parameter_i32(glow::MAX_VERTEX_ATTRIBS) as u32,
            max_vertex_buffer_array_stride: 2048,
            max_push_constant_size: 0,
        };

        let mut private_caps = super::PrivateCapability::empty();
        private_caps.set(
            super::PrivateCapability::EXPLICIT_LAYOUTS_IN_SHADER,
            ver >= (3, 1),
        );

        Some(crate::ExposedAdapter {
            adapter: super::Adapter {
                shared: Arc::new(super::AdapterShared {
                    context: gl,
                    private_caps,
                }),
            },
            info: Self::make_info(vendor, renderer),
            features,
            capabilities: crate::Capabilities {
                limits,
                downlevel: wgt::DownlevelCapabilities {
                    flags: downlevel_flags,
                    shader_model: wgt::ShaderModel::Sm5,
                },
                alignments: crate::Alignments {
                    buffer_copy_offset: wgt::BufferSize::new(4).unwrap(),
                    buffer_copy_pitch: wgt::BufferSize::new(4).unwrap(),
                    uniform_buffer_offset: wgt::BufferSize::new(
                        min_storage_buffer_offset_alignment as u64,
                    )
                    .unwrap(),
                    storage_buffer_offset: wgt::BufferSize::new(
                        min_uniform_buffer_offset_alignment as u64,
                    )
                    .unwrap(),
                },
            },
        })
    }
}

impl crate::Adapter<super::Api> for super::Adapter {
    unsafe fn open(
        &self,
        features: wgt::Features,
    ) -> Result<crate::OpenDevice<super::Api>, crate::DeviceError> {
        let gl = &self.shared.context;
        gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);
        let main_vao = gl.create_vertex_array().unwrap();
        gl.bind_vertex_array(Some(main_vao));

        Ok(crate::OpenDevice {
            device: super::Device {
                shared: Arc::clone(&self.shared),
                main_vao,
            },
            queue: super::Queue {
                shared: Arc::clone(&self.shared),
                features,
            },
        })
    }

    unsafe fn texture_format_capabilities(
        &self,
        format: wgt::TextureFormat,
    ) -> crate::TextureFormatCapability {
        crate::TextureFormatCapability::empty() //TODO
    }

    unsafe fn surface_capabilities(
        &self,
        surface: &super::Surface,
    ) -> Option<crate::SurfaceCapabilities> {
        if surface.presentable {
            Some(crate::SurfaceCapabilities {
                formats: vec![
                    wgt::TextureFormat::Rgba8UnormSrgb,
                    wgt::TextureFormat::Bgra8UnormSrgb,
                ],
                present_modes: vec![wgt::PresentMode::Fifo], //TODO
                composite_alpha_modes: vec![crate::CompositeAlphaMode::Opaque], //TODO
                swap_chain_sizes: 2..=2,
                current_extent: None,
                extents: wgt::Extent3d {
                    width: 4,
                    height: 4,
                    depth_or_array_layers: 1,
                }..=wgt::Extent3d {
                    width: 4096,
                    height: 4096,
                    depth_or_array_layers: 1,
                },
                usage: crate::TextureUse::COLOR_TARGET,
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::Adapter;

    #[test]
    fn test_version_parse() {
        let error = Err(crate::InstanceError);
        assert_eq!(Adapter::parse_version("1"), error);
        assert_eq!(Adapter::parse_version("1."), error);
        assert_eq!(Adapter::parse_version("1 h3l1o. W0rld"), error);
        assert_eq!(Adapter::parse_version("1. h3l1o. W0rld"), error);
        assert_eq!(Adapter::parse_version("1.2.3"), error);
        assert_eq!(Adapter::parse_version("OpenGL ES 3.1"), Ok((3, 1)));
        assert_eq!(
            Adapter::parse_version("OpenGL ES 2.0 Google Nexus"),
            Ok((2, 0))
        );
        assert_eq!(Adapter::parse_version("GLSL ES 1.1"), Ok((1, 1)));
        assert_eq!(Adapter::parse_version("OpenGL ES GLSL ES 3.20"), Ok((3, 2)));
        assert_eq!(
            // WebGL 2.0 should parse as OpenGL ES 3.0
            Adapter::parse_version("WebGL 2.0 (OpenGL ES 3.0 Chromium)"),
            Ok((3, 0))
        );
        assert_eq!(
            Adapter::parse_version("WebGL GLSL ES 3.00 (OpenGL ES GLSL ES 3.0 Chromium)"),
            Ok((3, 0))
        );
    }
}
