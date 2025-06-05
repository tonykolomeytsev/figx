// impl<'de> Deserialize<'de> for ResourcesDto {
//     fn deserialize(value: &mut toml_span::Value<'de>) -> Result<Self, toml_span::DeserError> {
//         let ctx = PROFILE_KIND_CONTEXT.get().unwrap();
//         let mut th = TableHelper::new(value)?;
//         let mut resources = OrderMap::with_capacity(1024);

//         for (key, value) in th.table.iter_mut() {
//             let profile_name = &key.name;
//             use super::ProfileKindMarker::*;
//             match ctx.get(profile_name.as_ref()) {
//                 Some(profile) => {
//                     let mut th = TableHelper::new(value)?;
//                     for (key, value) in th.table.iter_mut() {
//                         let res_name = &key.name;

//                         let res = if let Some(node_name) = value.as_str() {
//                             ResourceDto {
//                                 node_name: node_name.to_owned(),
//                                 profile_name: profile_name.to_string(),
//                                 override_profile: None,
//                             }
//                         } else {
//                             let mut th = TableHelper::new(value)?;
//                             let node_name = th.required("name")?;
//                             let override_profile = match profile {
//                                 Png => ProfileDto::Png(PngProfileDto::deserialize(value)?),
//                                 Svg => ProfileDto::Svg(SvgProfileDto::deserialize(value)?),
//                                 Pdf => ProfileDto::Pdf(PdfProfileDto::deserialize(value)?),
//                                 Webp => ProfileDto::Webp(WebpProfileDto::deserialize(value)?),
//                                 Compose => {
//                                     ProfileDto::Compose(ComposeProfileDto::deserialize(value)?)
//                                 }
//                                 AndroidWebp => ProfileDto::AndroidWebp(
//                                     AndroidWebpProfileDto::deserialize(value)?,
//                                 ),
//                             };

//                             validate_remote(&override_profile, remote_ids)?;
//                             ResourceDto {
//                                 node_name,
//                                 profile_name: profile_name.to_string(),
//                                 override_profile: Some(override_profile),
//                             }
//                         };

//                         let res_name =
//                             ResourceName::from_str(&res_name).map_err(|_| toml_span::Error {
//                                 kind: toml_span::ErrorKind::Custom("invalid resource name".into()),
//                                 span: key.span,
//                                 line_info: None,
//                             })?;
//                         resources.insert(res_name, res);
//                     }
//                 }
//                 _ => {
//                     return Err(toml_span::Error {
//                         kind: toml_span::ErrorKind::Custom("undeclared profile name".into()),
//                         span: value.span,
//                         line_info: None,
//                     }
//                     .into());
//                 }
//             }
//         }

//         Ok(Self(resources))
//     }
// }
