# ts-ef

A command-line tool that filters TypeScript compiler output to show errors only from specified files and directories.

![Demo](/demo.gif)

## Overview

`ts-ef` (TypeScript Error Filter) parses the output from `tsc` (TypeScript compiler) and filters errors based on file path patterns. This is particularly useful in large codebases where you want to focus on errors from specific modules or directories while ignoring errors from dependencies or unrelated code.

The tool supports both ANSI-colored output (`tsc --pretty`) and plain text output, automatically detecting the format and using appropriate parsing strategies.

## Installation

Build from source using Cargo:

```bash
cargo build --release
```

The binary will be available at `target/release/ts-ef`.

## Usage

### Basic Usage

Filter TypeScript errors to show only errors from specific files:

```bash
tsc --pretty *.ts | ts-ef -i src/components/Header.tsx -i src/utils/helpers.ts
```

### With Glob Patterns

Use glob patterns to filter by directory or file patterns:

```bash
tsc --pretty *.ts | ts-ef -i "src/features/**/*" -i "src/components/**/*"
```

### Show Complete Summary

Include the TypeScript compiler summary even when filtering:

```bash
tsc --pretty *.ts | ts-ef -i "src/**/*" --show-full
```

### Without Pretty Output

Works with plain `tsc` output as well:

```bash
tsc *.ts | ts-ef -i "src/**/*"
```

## Command Line Options

- `-i, --include <PATTERN>`: Glob pattern to match files for inclusion. Can be specified multiple times.
- `-s, --show-full`: Show the complete TypeScript compiler summary output when using `--pretty` with `tsc`.

## How It Works

The tool uses state machines to parse TypeScript compiler output:

### ANSI State Machine

When `tsc --pretty` is used, the output contains ANSI escape sequences for colors and formatting. The ANSI state machine:

1. Detects ANSI sequences that mark the beginning of error messages
2. Extracts file paths from the formatted output
3. Checks if the file path matches any of the include patterns
4. Manages state transitions to correctly handle multi-line error messages
5. Detects the end of error output to show summary information when requested

### Simple State Machine

For plain `tsc` output without ANSI formatting:

1. Extracts file paths from each line (expects format: `path(line,col): error message`)
2. Checks if the path matches include patterns
3. Outputs matching lines immediately

### Path Matching

The tool uses glob patterns for flexible file matching:

- `src/**/*` - matches all files under src directory
- `*.ts` - matches all TypeScript files in current directory
- `src/components/*.tsx` - matches all TSX files in src/components
- Multiple patterns can be specified with multiple `-i` flags

If no include patterns are provided, all errors are shown (no filtering).

## Exit Codes

- `0` - Success (no errors in filtered files or no input received)
- `1` - Failure (errors found in filtered files)

The exit code reflects whether errors were found in the files you care about, making it suitable for use in CI/CD pipelines.

