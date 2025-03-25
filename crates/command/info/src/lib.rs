pub type Result<T> = ::std::result::Result<T, Error>;

pub enum Error {
    InitError(phase_loading::Error),
}

pub struct FeatureInfoOptions {
    pub entity: InfoEntity,
}

pub enum InfoEntity {
    Workspace,
    Package,
}

pub fn info(opts: FeatureInfoOptions) -> Result<()> {
    let ctx = phase_loading::load_invocation_context().map_err(Error::InitError)?;
    match opts.entity {
        InfoEntity::Workspace => println!("{}", ctx.workspace_dir.to_string_lossy()),
        InfoEntity::Package => match &ctx.current_package {
            Some(package) => println!("{package}"),
            None => eprintln!("Not in package!"),
        },
    }
    Ok(())
}
