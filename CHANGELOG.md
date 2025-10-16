# Unreleased

# 0.8.4

- Added experimental syntax for `.figtree.toml`, allowing for the tagging of `container_node_ids` values for further use in the scan output.
- Each node in the scan output now has a "tag" value indicating its source container node.

# 0.8.3

- `figx` will automatically enable basic logging if it detects the environment variable `CI`.
- `figx` will automatically enable **debug** logging if it detects one of the following environment variable: `DEBUG`, `ACTIONS_RUNNER_DEBUG`, or `ACTIONS_STEP_DEBUG`.

# 0.8.2

- Improved logging for http requests in debug mode

# 0.8.1

- Add support for Linux with GLIBC 2.35 binary
- Add new profile `android-drawable` for Android Drawable XML vector icons

# 0.8.0

- Command `figx import` / `figx i` now imports only visible nodes with type `COMPONENT`. Before this change you could import any node type. This was done because in real projects, some components with illustrations can be used within others. This leads to confusion during import. Nodes with type `COMPONENT` have unique names, so now only those can be imported.
- Experimental command `figx scan` now outputs only metadata for visible nodes with type `COMPONENT` for the same reasons.

# 0.7.7

- Added experimental command `figx scan` which scans remote figma file content and outputs it to the file. The collected metadata can be used by external scripts for any purpose.

# 0.7.4

Access tokens can now be stored securely in the system keychain on local machines, eliminating the need to set them manually via environment variables — because nobody really likes dealing with environment variables.

To add an access token to the keychain, use the `figx auth` command. It will open a web interface with further instructions.

**This feature is currently supported on macOS and Windows only.**

Additionally, you can now explicitly configure the priority order for how `figx` searches for access tokens in the remote configuration. Example:

```toml
[remotes.icons]
...
access_token = [
   { env = "FIGMA_ACCESS_TOKEN" }, # Check environment variable first (e.g., for CI)
   { keychain = true }, # Then fallback to system keychain (for local use)
]
```

**ALSO:** Fixed a bug where a progress bar was incorrectly displayed in contexts where it shouldn’t appear.

# 0.7.3

- Minor improvements and optimizations for interactive output

# 0.7.2

- **Less verbose output**. By default, instead of a huge wall of logs, a cool and concise progress bar is now shown. To display logs or even more detailed logs, use the `-v`, `-vv`, and `-vvv` arguments.

# 0.7.1

- Show indexing progress during `fetch` and `import --refetch`

# 0.7.0

- Figma file indexing now runs in parallel with resource import, streaming data as it becomes available. As soon as a Figma node ID is identified for a resource, it is immediately queued for download.
   **This greatly shortens the wait time for resource loading, especially in large Figma files.**

- The `figx fetch` command no longer causes resource transformation. Only downloads from Figma.

- The message that a node with a certain name was not found in Figma now contains a link to the resource declaration, leading to a file if you click on it in the IDE

- **Experimental**: Metrics are now published in Prometheus format to the `.figx-out/metrics.prom` file, containing information about the import process.

# 0.6.0

The `png`, `webp`, and `android-webp` profiles now default to downloading the original SVG asset from Figma and rendering the required resolution locally.

### Improvements with this approach:
* **`png` profile**: The asset no longer needs to be re-downloaded for different scale values. Since the source is in SVG format, it can be re-rendered at any resolution on demand.
* **`webp` profile**: Same benefits as PNG, plus local rendering enables better lossless compression. The resulting WEBP files are approximately 9% smaller compared to the previous method.
* **`android-webp` profile**: Inherits all improvements from PNG and WEBP. Importing a single android-webp asset is now 4–5× faster, with no quality loss and better compression.

> You can revert to the previous behavior by setting the `legacy_loader = true` property in the profile or any resource.

Special thanks to the developers of the brilliant [resvg](https://github.com/linebender/resvg) and [usvg](https://github.com/linebender/resvg/tree/main/crates/usvg) libraries for making this possible.
