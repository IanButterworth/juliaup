use crate::global_paths::GlobalPaths;
use crate::operations::{install_version, create_symlink};
use crate::config_file::{JuliaupConfigChannel, load_mut_config_db, save_config_db};
use crate::versions_file::load_versions_db;
use anyhow::{anyhow, bail, Context, Result};

pub fn run_command_add(channel: String, paths: &GlobalPaths) -> Result<()> {
    let version_db =
        load_versions_db().with_context(|| "`add` command failed to load versions db.")?;

    let required_version = &version_db
        .available_channels
        .get(&channel)
        .ok_or(anyhow!(
            "'{}' is not a valid Julia version or channel name.",
            &channel
        ))?
        .version;

    let mut config_file = load_mut_config_db(paths)
        .with_context(|| "`add` command failed to load configuration data.")?;

    if config_file.data.installed_channels.contains_key(&channel) {
        bail!("'{}' is already installed.", &channel);
    }
    
    install_version(&required_version, &mut config_file.data, &version_db, paths)?;

    config_file.data.installed_channels.insert(
        channel.clone(),
        JuliaupConfigChannel::SystemChannel {
            version: required_version.clone(),
        },
    );

    if config_file.data.default.is_none() {
        config_file.data.default = Some(channel.clone());
    }

    let create_symlinks = config_file.data.settings.create_channel_symlinks;

    save_config_db(&mut config_file)
        .with_context(|| format!("Failed to save configuration file from `add` command after '{}' was installed.", channel))?;

    #[cfg(not(target_os = "windows)"))]
    if create_symlinks {
        create_symlink(
            &JuliaupConfigChannel::SystemChannel {
                version: required_version.clone(),
            },
            &format!("julia-{}", channel),
            paths,
        )?;
    }

    Ok(())
}
