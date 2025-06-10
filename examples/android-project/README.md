# Android project with FigX

This project demonstrates how to integrate FigX into an Android project.

In the root of the project, two custom import profiles are defined in the `.figtree.toml` file:
1. icons profile — Inherits all properties from the built-in compose profile, which is designed for importing icons from Figma into the ImageVector format for Jetpack Compose.
2. illustrations profile — Inherits all properties from the android-webp profile, which supports importing illustrations from Figma into all drawable-*dpi directories and compresses them into the WEBP format.

**Creating new profiles is optional.** You could configure and use the built-in profiles directly (`android-webp` + `compose`). However, defining custom profiles with domain-specific names makes the configuration more expressive and easier to understand. Using meaningful names like `icons` and `illustrations` is especially helpful for teams familiar with tools like [figma-export](https://github.com/RedMadRobot/figma-export).

Moreover, FigX is not limited to just these two profiles — you can define as many as you need to fit your project’s requirements and domain logic.

> **Note (by design):** Custom profiles can only extend built-in profiles. This is an intentional safeguard to prevent overly complex and unnecessary abstractions — a common issue known as "Java brain disease."

**More notes:**
- The `icons` profile loads resources from a Figma file that contains the icon library. See the remote named `icons`.
- The `illustrations` profile looks for resources in a completely different Figma file. See the remote named `illustrations`.

## Example Commands

Import all resources:

```bash
figx import //...
```

Import only WEBP drawable resources (from the `//app` package):

```bash
figx import //app:all
```
> Note: `//app:*` is also a valid argument, but it won’t work in **zsh** unless you escape the glob pattern with quotes `"//app:*"` or disable globbing using `setopt noglob`.

Import only Jetpack Compose icons (from the `//app/src/main/java/com/example/figxdemo/ui/icons` package):

```bash
figx import //.../ui/icons:all
```

List all **figx resources** in the project without importing them:

```bash
figx query //...
```

List all **figx packages** in the project:

```bash
figx query -o package //...
```

Explain the import flow for specific resources:

```bash
figx explain //app:all //.../ui/icons:Sun
```

Example output:

```text
//app:ill_travellers
├── Variant 'hdpi'
│   ├── 📤 Export PNG from remote @illustrations/rqeqh3mlmLVTDS0OEan67m
│   │      ┆ node: Image
│   │      ┆ scale: 1.5
│   ├── ✨ Transform PNG to WEBP
│   │      ┆ quality: 100
│   ╰── 💾 Write to file
│          ┆ output: drawable-hdpi/ill_travellers.webp
├── Variant 'xhdpi'
│   ├── 📤 Export PNG from remote @illustrations/rqeqh3mlmLVTDS0OEan67m
│   │      ┆ node: Image
│   │      ┆ scale: 2
│   ├── ✨ Transform PNG to WEBP
│   │      ┆ quality: 100
│   ╰── 💾 Write to file
│          ┆ output: drawable-xhdpi/ill_travellers.webp
├── Variant 'xxhdpi'
│   ├── 📤 Export PNG from remote @illustrations/rqeqh3mlmLVTDS0OEan67m
│   │      ┆ node: Image
│   │      ┆ scale: 3
│   ├── ✨ Transform PNG to WEBP
│   │      ┆ quality: 100
│   ╰── 💾 Write to file
│          ┆ output: drawable-xxhdpi/ill_travellers.webp
╰── Variant 'xxxhdpi'
    ├── 📤 Export PNG from remote @illustrations/rqeqh3mlmLVTDS0OEan67m
    │      ┆ node: Image
    │      ┆ scale: 4
    ├── ✨ Transform PNG to WEBP
    │      ┆ quality: 100
    ╰── 💾 Write to file
           ┆ output: drawable-xxxhdpi/ill_travellers.webp

//app/src/main/java/com/example/figxdemo/ui/icons:Sun
├── 📤 Export SVG from remote @icons/MhjeA23R15tAR3PO2JamCv
│      ┆ node: Environment / Sun
│      ┆ scale: 1
├── ✨ Transform SVG to Compose
│      ┆ package: com.example.figxdemo.ui.icons
╰── 💾 Write to file
       ┆ output: Sun.kt
```