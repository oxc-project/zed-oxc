<p align="center">
  <img alt="OXC Logo" src="https://cdn.jsdelivr.net/gh/oxc-project/oxc-assets/preview-universal.png" width="700">
</p>

# Oxc extension for Zed

This extension adds support for [Oxc](https://github.com/oxc-project/oxc) in [Zed](https://zed.dev/).

Languages currently supported:

- **JavaScript**
- **TypeScript**
- **JSX**
- **TSX**
- **Vue.js**
- **Astro**
- **Svelte**

## Installation

Requires Zed >= **v0.131.0**.

This extension is available in the extensions view inside the Zed editor. Open `zed: extensions` and search for _Oxc_.

## Configuration

To configure the oxc extension in the Zed editor, edit your settings.json file and add the following configuration:

```json
{
  "lsp": {
    "oxc": {
      "initialization_options": {
        "options": {
          "run": "onType",
          "configPath": null,
          "tsConfigPath": null,
          "unusedDisableDirectives": "allow",
          "typeAware": false,
          "flags": {}
        }
      }
    }
  }
}
```

Below are the available values and descriptions for each option:

| Option Key                | Value(s)                       | Default    | Description                                                                                                                                            |
| ------------------------- | ------------------------------ | ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `run`                     | `"onSave" \| "onType"`         | `"onType"` | Should the server lint the files when the user is typing or saving                                                                                     |
| `configPath`              | `<string>` \| `null`           | `null`     | Path to a oxlint configuration file, passing a string will disable nested configuration                                                                |
| `tsConfigPath`            | `<string>` \| `null`           | `null`     | Path to a TypeScript configuration file. If your `tsconfig.json` is not at the root, alias paths will not be resolve correctly for the `import` plugin |
| `unusedDisableDirectives` | `"allow" \| "warn"` \| "deny"` | `"allow"`  | Define how directive comments like `// oxlint-disable-line` should be reported, when no errors would have been reported on that line anyway            |
| `typeAware`               | `true` \| `false`              | `false`    | Enables type-aware linting                                                                                                                             |
| `flags`                   | `Map<string, string>`          | `<empty>`  | Special oxc language server flags, currently only one flag key is supported: `disable_nested_config`                                                   |

For more information, see <https://github.com/oxc-project/oxc/tree/main/crates/oxc_language_server>
