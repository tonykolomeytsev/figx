# Profiles Reference

This document describes all built-in import profiles available in figmagic. Profiles can be configured in the root `.figmagic.toml` file and extended to create custom import configurations.

## Profile Types
1. Image Profiles
    - **png**: Basic PNG asset import
    - **webp**: WebP format conversion
    - **android-webp**: Android-optimized WebP with density and theme support
2. Vector Profiles
    - **svg**: Raw SVG import
    - **compose**: Jetpack Compose ImageVector conversion
3. Document Profiles
    - **pdf**: Document export

## Configuration Structure
All profiles share common configuration properties:

```toml
[profiles.{profile_name}]
# Reference to remote configuration
remote = "default"
```

## Extending Profiles
Create custom profiles by inheriting from existing ones:

```toml
[profiles.custom-webp]
extends = "webp"  # Inherit all base webp settings
# Override specific properties
quality = 90
output_dir = "src/webp_assets"
```

## Best Practices
1. Naming Conventions:
    - Use lowercase with hyphens (profile-name)
    - Be descriptive (android-icon-webp)
2. Organization:
    - Group related profiles in sections
    - Use comments for documentation
3. Do not abuse inheritance!
