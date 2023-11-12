use crate::command::RustupCommand;
use crate::error::RustupInstallFailed;
use crate::reporter::event::SetupToolchain;
use crate::toolchain::ToolchainSpec;
use crate::{CargoMSRVError, Reporter, TResult};

pub trait DownloadToolchain {
    fn download(&self, toolchain: &ToolchainSpec) -> TResult<()>;
}

#[derive(Debug)]
pub struct ToolchainDownloader<'reporter, R: Reporter> {
    reporter: &'reporter R,
}

impl<'reporter, R: Reporter> ToolchainDownloader<'reporter, R> {
    pub fn new(reporter: &'reporter R) -> Self {
        Self { reporter }
    }
}

impl<'reporter, R: Reporter> DownloadToolchain for ToolchainDownloader<'reporter, R> {
    #[instrument(skip(self, toolchain))]
    fn download(&self, toolchain: &ToolchainSpec) -> TResult<()> {
        info!(toolchain = toolchain.spec(), "installing toolchain");

        self.reporter
            .run_scoped_event(SetupToolchain::new(toolchain.to_owned()), || {
                install_toolchain(toolchain).and_then(|_| add_target(toolchain))
            })
    }
}

fn install_toolchain(toolchain: &ToolchainSpec) -> TResult<()> {
    let rustup = RustupCommand::new()
        .with_stdout()
        .with_stderr()
        .with_args(["--profile", "minimal", &format!("{}", toolchain.version())])
        .install()?;

    let status = rustup.exit_status();

    if !status.success() {
        error!(
            toolchain = toolchain.spec(),
            stdout = rustup.stdout(),
            stderr = rustup.stderr(),
            "rustup failed to install toolchain"
        );

        return Err(CargoMSRVError::RustupInstallFailed(
            RustupInstallFailed::new(toolchain.spec(), rustup.stderr()),
        ));
    }

    Ok(())
}

fn add_target(toolchain: &ToolchainSpec) -> TResult<()> {
    let rustup = RustupCommand::new()
        .with_stdout()
        .with_stderr()
        .with_args([
            "add",
            "--toolchain",
            &format!("{}", toolchain.version()),
            toolchain.target(),
        ])
        .target()?;

    let status = rustup.exit_status();

    if !status.success() {
        error!(
            toolchain = toolchain.spec(),
            stdout = rustup.stdout(),
            stderr = rustup.stderr(),
            "rustup failed to add target to toolchain"
        );

        return Err(CargoMSRVError::RustupInstallFailed(
            RustupInstallFailed::new(toolchain.spec(), rustup.stderr()),
        ));
    }

    Ok(())
}
