use crate::parser::RemotesDtoContext;

use super::{ProfilesDto, RemotesDto};

#[cfg_attr(test, derive(PartialEq, Debug))]
pub(crate) struct WorkspaceDto {
    pub remotes: RemotesDto,
    pub profiles: ProfilesDto,
}

pub struct WorkspaceDtoContext {
    pub ignore_missing_access_token: bool,
}

impl From<WorkspaceDtoContext> for RemotesDtoContext {
    fn from(value: WorkspaceDtoContext) -> Self {
        Self {
            ignore_missing_access_token: value.ignore_missing_access_token,
        }
    }
}

mod de {
    use super::*;
    use crate::{ParseWithContext, parser::ProfilesDtoContext};
    use toml_span::{ErrorKind, de_helpers::TableHelper};

    impl<'de> ParseWithContext<'de> for WorkspaceDto {
        type Context = WorkspaceDtoContext;

        fn parse_with_ctx(
            value: &mut toml_span::Value<'de>,
            ctx: Self::Context,
        ) -> std::result::Result<Self, toml_span::DeserError> {
            // region: extract
            let mut th = TableHelper::new(value)?;
            let remotes = th.take("remotes");
            let profiles = th.take("profiles");
            th.finalize(None)?;
            // endregion: extract

            // region: validate
            let remotes = match remotes {
                Some((_, mut value)) => RemotesDto::parse_with_ctx(&mut value, ctx.into())?,
                None => {
                    return Err(toml_span::Error::from((
                        ErrorKind::Custom(
                            "at least one remote should be defined in workspace".into(),
                        ),
                        value.span,
                    ))
                    .into());
                }
            };
            let profiles = match profiles {
                Some((_, mut value)) => {
                    let ctx = ProfilesDtoContext {
                        declared_remote_ids: &remotes.0.iter().map(|(k, _)| k.clone()).collect(),
                    };
                    ProfilesDto::parse_with_ctx(&mut value, ctx)?
                }
                None => ProfilesDto::default(),
            };
            // endregion: validate

            Ok(Self { remotes, profiles })
        }
    }
}
