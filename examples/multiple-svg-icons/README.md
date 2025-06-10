# Minimal example with FigX

This example uses the default `svg` profile to import icons in SVG format.

The svg profile is configured in the `.figtree.toml` file, where it's specified that icons will be placed in the `./img` directory relative to their package.

The root of this project is also a FigX package because it contains a `.fig.toml` file that lists the icons.

One resource in `.fig.toml` overrides the scale value from the `svg` profile and sets it to `4.0`. As a result, one icon appears larger than the others after import.

### Example Commands

Import all icons:

```bash
figx import //...
```

Import a specific icon:

```bash
figx import :planet
```

List all **figx resources** in the project without importing them:

```bash
figx query //...
```

Explain the import flow of a specific icon:

```bash
figx explain :planet
```
Output:

```text
//:planet
├── 📤 Export SVG from remote @design/MhjeA23R15tAR3PO2JamCv
│      ┆ node: Environment / Planet
│      ┆ scale: 1
╰── 💾 Write to file
       ┆ output: planet.svg
```
