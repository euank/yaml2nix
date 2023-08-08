# yaml2nix

yaml2nix is a command line tool to convert yaml into a nix expression.

## Usage

Currently, `yaml2nix` takes a single argument, the yaml file to convert, and
outputs the nix expression on stdout.
If the input yaml file cannot be parsed, it will exit non-zero and print an
appropriate error message.

With flake enabled Nix, run:

```
nix run github:euank/yaml2nix $PATH_TO_YAML_FILE
```
