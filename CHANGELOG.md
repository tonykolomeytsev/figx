# Unreleased

- The message that a node with a certain name was not found in Figma now contains a link to the resource declaration, leading to a file if you click on it in the IDE
- Figma file indexing now runs in parallel with resource import, streaming data as it becomes available. As soon as a Figma node ID is identified for a resource, it is immediately queued for download.
   
   **This greatly shortens the wait time for resource loading, especially in large Figma files.**

# 0.6.0

The `png`, `webp`, and `android-webp` profiles now default to downloading the original SVG asset from Figma and rendering the required resolution locally.

### Improvements with this approach:
* **`png` profile**: The asset no longer needs to be re-downloaded for different scale values. Since the source is in SVG format, it can be re-rendered at any resolution on demand.
* **`webp` profile**: Same benefits as PNG, plus local rendering enables better lossless compression. The resulting WEBP files are approximately 9% smaller compared to the previous method.
* **`android-webp` profile**: Inherits all improvements from PNG and WEBP. Importing a single android-webp asset is now 4–5× faster, with no quality loss and better compression.

> You can revert to the previous behavior by setting the `legacy_loader = true` property in the profile or any resource.

Special thanks to the developers of the brilliant [resvg](https://github.com/linebender/resvg) and [usvg](https://github.com/linebender/resvg/tree/main/crates/usvg) libraries for making this possible.