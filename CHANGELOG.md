# Unreleased

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