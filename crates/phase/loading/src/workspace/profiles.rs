use std::sync::Arc;

use ordermap::OrderMap;

use crate::{
    AndroidDrawableProfile, AndroidWebpProfile, CanBeExtendedBy, ComposeProfile, PdfProfile,
    PngProfile, Profile, Result, SvgProfile, WebpProfile,
    parser::{ProfileDto, ProfilesDto},
};

pub fn parse_profiles(
    ProfilesDto(profiles): ProfilesDto,
) -> Result<OrderMap<String, Arc<Profile>>> {
    let mut output = OrderMap::with_capacity(profiles.len());

    for (id, profile) in profiles {
        let profile = match profile {
            ProfileDto::Png(p) => Profile::Png(PngProfile::default().extend(&p)),
            ProfileDto::Svg(p) => Profile::Svg(SvgProfile::default().extend(&p)),
            ProfileDto::Pdf(p) => Profile::Pdf(PdfProfile::default().extend(&p)),
            ProfileDto::Webp(p) => Profile::Webp(WebpProfile::default().extend(&p)),
            ProfileDto::Compose(p) => Profile::Compose(ComposeProfile::default().extend(&p)),
            ProfileDto::AndroidWebp(p) => {
                Profile::AndroidWebp(AndroidWebpProfile::default().extend(&p))
            }
            ProfileDto::AndroidDrawable(p) => {
                Profile::AndroidDrawable(AndroidDrawableProfile::default().extend(&p))
            }
        };
        output.insert(id, Arc::new(profile));
    }

    Ok(output)
}
