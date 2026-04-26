---
name: "dev-process"
description: "Guides PAC→HAL code generation workflow with temp directory validation. Invoke when starting new codegen tasks or modifying generators."
---

# Development Process

> **Strict priority order for code generation and modification.**

## Package Priority Order

1. **PAC package** - PRIMARY
   - Must be generated and built first
   - Has highest priority and can modify anything

2. **HAL package** - SECONDARY
   - Must be generated and built after PAC
   - Has lower priority and only adapts to PAC package

## PAC Package Development Workflow

When solving a new task that changes PAC:

1. Generate PAC package
2. Copy PAC to temporary directory `/temp`
3. Make changes to files in `/temp` and verify they build successfully
4. Determine the difference between initially generated files and files in `/temp`
5. Identify required changes to the PAC generator
6. Implement changes in the PAC generator
7. Verify that newly generated files build successfully

## HAL Package Development Workflow

After completing PAC development, proceed with HAL package using the same workflow:

1. Generate HAL package
2. Copy HAL to temporary directory `/temp`
3. Make changes to files in `/temp` and verify they build successfully
4. Determine the difference between initially generated files and files in `/temp`
5. Identify required changes to the HAL generator
6. Implement changes in the HAL generator
7. Verify that newly generated files build successfully

## Specification

1. PAC package MUST be generated and built before any HAL package work begins
2. PAC generator has priority and MAY modify any code
3. HAL generator MUST adapt to changes in PAC package
4. All changes MUST be validated by successful build before proceeding
5. Temporary directory `/temp` MAY be used for experimental modifications