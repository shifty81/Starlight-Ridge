use anyhow::{anyhow, Context};
use glutin::config::{Config, ConfigTemplateBuilder, GlConfig};
use glutin_winit::DisplayBuilder;
use std::sync::Arc;
use winit::dpi::LogicalSize;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes};

#[derive(Debug, Clone)]
pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "Starlight Ridge".to_string(),
            width: 1280,
            height: 720,
        }
    }
}

pub struct WindowBootstrap {
    pub window: Arc<Window>,
    pub gl_config: Config,
}

impl WindowBootstrap {
    pub fn size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.window.inner_size()
    }
}

pub fn create_gl_window(
    event_loop: &ActiveEventLoop,
    config: &WindowConfig,
) -> anyhow::Result<WindowBootstrap> {
    let attrs = WindowAttributes::default()
        .with_title(config.title.clone())
        .with_inner_size(LogicalSize::new(config.width as f64, config.height as f64));

    let template = ConfigTemplateBuilder::new()
        .with_alpha_size(8)
        .with_depth_size(24)
        .with_stencil_size(8)
        .with_transparency(false);

    let display_builder = DisplayBuilder::new().with_window_attributes(Some(attrs));

    let (window, gl_config) = display_builder
        .build(event_loop, template, |configs| {
            configs
                .max_by_key(|cfg| cfg.num_samples())
                .expect("at least one OpenGL config should be available")
        })
        .map_err(|e| anyhow!("failed to create winit window + OpenGL display config: {e}"))?;

    let window = window.context("gl display builder did not return a window")?;
    let size = window.inner_size();

    log::info!(
        "window created: '{}' ({}x{}), samples={}, srgb_capable={:?}",
        config.title,
        size.width,
        size.height,
        gl_config.num_samples(),
        gl_config.srgb_capable()
    );

    Ok(WindowBootstrap {
        window: Arc::new(window),
        gl_config,
    })
}
