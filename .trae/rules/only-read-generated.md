# Only-Read Generated Directory

> Agents MUST treat the `generated` directory as read-only.

## Summary

The `generated` directory contains auto-generated code and should not be modified by agents.
All operations that would create, modify, or delete files within this directory are **prohibited**.

## Specification

1. Agents MUST NOT create new files in the `generated` directory.
2. Agents MUST NOT modify existing files in the `generated` directory.
3. Agents MUST NOT delete files from the `generated` directory.
4. Agents MAY read files from the `generated` directory for reference and analysis.
5. If an agent needs to regenerate content in the `generated` directory, it MUST inform the user and request explicit permission before making any changes.