use bevy::prelude::*;
use bevy_common_assets::toml::TomlAssetPlugin;
use serde::Deserialize;

use crate::blocks::render::RenderConfig;

#[derive(Clone, Copy, Debug, PartialEq, Asset, Resource, TypePath, Deserialize)]
#[serde(default)]
pub struct Config {
    pub block_size: f32,
    pub render: RenderConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            block_size: 64.,
            render: default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, States)]
pub enum ConfigLoadState {
    #[default]
    Loading,
    Ready,
}

pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ConfigLoadState>()
            .add_plugins(TomlAssetPlugin::<Config>::new(&["toml"]))
            .add_systems(Startup, load_config)
            .add_systems(Update, sync_config);
    }
}

#[derive(Resource)]
struct ConfigHandle {
    _handle: Handle<Config>,
}

fn load_config(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(ConfigHandle {
        _handle: asset_server.load("config.toml"),
    });
}

fn sync_config(
    mut message_reader: MessageReader<AssetEvent<Config>>,
    mut commands: Commands,
    configs: Res<Assets<Config>>,
) {
    for event in message_reader.read() {
        debug!("Got asset event: {event:?}");
        match event {
            AssetEvent::LoadedWithDependencies { id } | AssetEvent::Modified { id } => {
                let Some(config) = configs.get(*id) else {
                    warn!("Couldn't get config");
                    continue;
                };
                commands.insert_resource_if_neq(*config);
                commands.set_state_if_neq(ConfigLoadState::Ready);
            }
            _ => {}
        };
    }
}
