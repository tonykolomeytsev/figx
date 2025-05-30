# Remotes Reference

Remotes in figx function similarly to Git remotes - they define the source locations in Figma where your design assets are stored. A remote configuration specifies:
- Which Figma file to access
- Precisely where in that file to look for assets
- Authentication credentials

Key Concepts:
- **Multiple Remotes**: You can configure several remotes pointing to different Figma files or different sections within the same file
- **Default Remote**: One remote must be marked as default for fallback operations
- **Targeted Exporting**: By specifying node IDs, you optimize performance by only accessing relevant portions of the Figma file


## Configuration Syntax

```toml
[remotes.{remote_name}]
# Marks the default remote for fallback operations
default = true|false
# Unique Figma file identifier
file_key = "figma_file_identifier"
# Array of specific node IDs to target
container_node_ids = ["node_id_1", "node_id_2"]
# Figma API token (can use ENV vars)
# -- access_token = "your_figma_token"
# default is below:
access_token = { env = "FIGMA_PERSONAL_TOKEN" }
```
