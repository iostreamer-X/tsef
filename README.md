# tsef

A CLI tool that filters TypeScript compiler output to show errors only from specified files and directories.



https://github.com/user-attachments/assets/075fad2b-8c14-4ae0-bfeb-fe993f4d3324




## Overview

`tsf` parses the output from `tsc` and filters errors based on file path patterns. This is particularly useful in large codebases where you want to focus on errors from specific modules or directories while ignoring errors from dependencies or unrelated code.

The tool supports both ANSI-colored output (`tsc --pretty`) and plain text output, automatically detecting the format and using appropriate parsing strategies.

## Installation

```bash
cargo install tsef
```

Build from source using Cargo:

```bash
cargo build --release
```

The binary will be available at `target/release/tsef`.

## Usage

### Basic Usage

Filter TypeScript errors to show only errors from specific files:

```bash
tsc --pretty *.ts | tsef -i src/components/Header.tsx -i src/utils/helpers.ts
```

### With Glob Patterns

Use glob patterns to filter by directory or file patterns:

```bash
tsc --pretty *.ts | tsef -i "src/features/**/*" -i "src/components/**/*"
```

### Show Complete Summary

Include the TypeScript compiler summary even when filtering:

```bash
tsc --pretty *.ts | tsef -i "src/**/*" --show-full
```

### Without Pretty Output

Works with plain `tsc` output as well:

```bash
tsc *.ts | tsef -i "src/**/*"
```

