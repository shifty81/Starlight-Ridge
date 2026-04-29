#[derive(Debug, Default)]
pub struct AudioBootstrap {
    pub master_volume: f32,
}

impl AudioBootstrap {
    pub fn new() -> Self {
        let bootstrap = Self { master_volume: 1.0 };
        log::info!(
            "audio bootstrap ready at volume {}",
            bootstrap.master_volume
        );
        bootstrap
    }
}
