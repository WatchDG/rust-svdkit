# Read-Only Directories

> Agents MUST treat the following directories as read-only:

- `generated` - auto-generated code
- `svds` - SVD files

All operations that would create, modify, or delete files within these directories are **prohibited**.

## Specification

1. Agents MUST NOT create new files in the read-only directories.
2. Agents MUST NOT modify existing files in the read-only directories.
3. Agents MUST NOT delete files from the read-only directories.
4. Agents MAY read files from the read-only directories for reference and analysis.
5. If an agent needs to regenerate content in the read-only directories, it MUST inform the user and request explicit permission before making any changes.
