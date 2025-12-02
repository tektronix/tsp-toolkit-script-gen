#!/usr/bin/env just --justfile

set windows-shell := ['powershell']

vendor := if os() == 'windows' { "-pc" } else if os() == 'darwin' { "-apple" } else { "-unknown" }
build := if os() == 'windows' { "-msvc" } else if os() == 'darwin' { "" } else { "-gnu" }
vsc-os := if os() == 'windows' { "win32" } else { os() }
vsc-arch := if arch() == "x86_64" { "x64" } else { "arm64" } # we only support x86_64 or ARM-64

package-name := replace_regex(`npm pkg get name`, "\"", "")
orig-package-os := replace_regex(`npm pkg get os`, "\\{}", "")
orig-package-cpu := replace_regex(`npm pkg get cpu`, "\\{}", "")

native-triple := (arch() + vendor +"-" + os() + build)
native-vscode-platform := (vsc-os + "-" + vsc-arch)

default triple=native-triple:
    @echo "{{package-name}}"
    @just --list

pr triple=native-triple: init fmt lint (build triple) test (package triple)

init: init-script-gen-ui init-rust init-root

check-fmt: lint-script-gen-ui check-fmt-rust

fmt: fmt-script-gen-ui fmt-rust

lint: lint-script-gen-ui lint-rust

build triple=native-triple: build-script-gen-ui (build-rust triple)

build-release triple=native-triple:
    just build-release-script-gen-ui
    just build-release-rust {{triple}}

test: test-rust test-script-gen-ui

test-cov: test-cov-rust test-cov-script-gen-ui

sbom: sbom-script-gen-ui sbom-rust sbom-root

pre-package triple=native-triple target-dir="": pre-package-script-gen-ui (pre-package-rust triple)
    {{if target-dir != "" { "rm -r " + target-dir + "/bin" + "/*" } else { "" } }}
    {{if target-dir != "" { "cp -r bin/* '" + target-dir + "/bin'" } else { "" } }}

package vscode-platform=native-vscode-platform os=os() cpu=arch() triple=native-triple: && packaging-cleanup
    npm pkg set "name={{package-name}}-{{vscode-platform}}" --verbose
    npm pkg set "os[0]={{os}}" --verbose
    npm pkg set "cpu[0]={{cpu}}" --verbose
    cat package.json
    npm pack

packaging-cleanup:
    npm pkg set "name={{package-name}}" --verbose
    npm pkg {{if orig-package-os != "" { 'set "os[0]={{orig-package-os}}"' } else { 'delete "os"'} }} --verbose
    npm pkg {{if orig-package-cpu != "" { 'set "cpu[0]={{orig-package-cpu}}"' } else { 'delete "cpu"'} }} --verbose

################################################################################
# INIT #########################################################################
################################################################################

# Initialize the root npm project
[group("init")]
init-root:
    npm install --devDependencies

# Initialize the script-gen-ui project
[group("init")]
[group("script-gen-ui")]
[working-directory: 'script-gen-ui']
init-script-gen-ui:
    npm install --devDependencies

# Initialize all rust projects
[group("init")]
[group("rust")]
init-rust:
    cargo check

################################################################################
# CHECK-FMT ####################################################################
################################################################################

# Rust
[group("check-fmt")]
[group("rust")]
check-fmt-rust:
    cargo fmt --check

################################################################################
# FMT ##########################################################################
################################################################################

# script-gen-ui
[group("fmt")]
[group("script-gen-ui")]
[working-directory: 'script-gen-ui']
fmt-script-gen-ui: init-script-gen-ui
    npx eslint --fix --fix-type layout

# Rust
[group("fmt")]
[group("rust")]
fmt-rust:
    cargo fmt

################################################################################
# LINT #########################################################################
################################################################################
# script-gen-ui
[group("check-fmt")]
[group("lint")]
[working-directory: 'script-gen-ui']
lint-script-gen-ui: init-script-gen-ui
    npx eslint

# Rust
[group("lint")]
[group("rust")]
lint-rust: init-root
    cargo clippy
    cargo clippy --tests

################################################################################
# BUILD ########################################################################
################################################################################

# script-gen-ui
[group("build")]
[group("script-gen-ui")]
[working-directory: 'script-gen-ui']
build-script-gen-ui: init-script-gen-ui
    npx ng build

# Rust
[group("build")]
[group("rust")]
build-rust triple=native-triple:
    cargo build --target {{triple}}

################################################################################
# BUILD-RELEASE ################################################################
################################################################################

# script-gen-ui
[group("build-release")]
[group("script-gen-ui")]
[working-directory: 'script-gen-ui']
build-release-script-gen-ui: init-script-gen-ui
    npx ng build --configuration production

# Rust
[group("build-release")]
[group("rust")]
build-release-rust triple=native-triple:
    cargo build --release --target {{triple}}

# All
[group("build-release")]
[group("all")]
compile-toolkit:
    code . --"{{env("TOOLKIT_DIR", "\\")}}"
    code --profile-temp --extensionDevelopmentPath="{{env("TOOLKIT_DIR", "\\")}}"
build-release-dev triple=native-triple: 
    just build-release {{triple}}
    just pre-package {{triple}} "{{env("TARGET_DIR", "\\")}}"
    just compile-toolkit

################################################################################
# TEST #########################################################################
################################################################################

# script-gen-ui
[group("test")]
[group("script-gen-ui")]
[working-directory: 'script-gen-ui']
test-script-gen-ui: init-script-gen-ui
    #npx ng test # need chrome instance...

# Rust
[group("test")]
[group("rust")]
test-rust:
    -rm -r "{{env("TEST_DIR", "test-results")}}"
    -mkdir -p '{{env("TEST_DIR", "test-results")}}'
    cargo nextest r --all --all-targets
    @mv test-results/default/* "{{env("TEST_DIR", "test-results")}}"
    @rm -r test-results/default

################################################################################
# TEST-COV #####################################################################
################################################################################

# script-gen-ui
[group("test-cov")]
[group("script-gen-ui")]
[working-directory: 'script-gen-ui']
test-cov-script-gen-ui: init-script-gen-ui
    echo "TODO"
    #npx ng test --code-coverage #Requires chrome instance

# Rust
[group("test-cov")]
[group("rust")]
test-cov-rust $CARGO_TERM_VERBOSE="true":
    -rm -r "{{env("TEST_DIR", "test-results")}}"
    -mkdir -p '{{env("TEST_DIR", "test-results")}}'
    cargo llvm-cov nextest --cobertura --branch > "{{env("TEST_DIR", "test-results")}}/script-gen-rust.cobertura.xml"
    @mv test-results/default/* "{{env("TEST_DIR", "test-results")}}"
    @rm -r test-results/default

################################################################################
# SBOM #########################################################################
################################################################################

[group("sbom")]
[group("root")]
sbom-root: init-root
    npx @cyclonedx/cyclonedx-npm --output-format JSON --package-lock-only --output-reproducible --output-file root.npm.cdx.json

# script-gen-ui
[group("sbom")]
[group("script-gen-ui")]
[working-directory: 'script-gen-ui']
sbom-script-gen-ui: init-script-gen-ui
    npx @cyclonedx/cyclonedx-npm --output-format JSON --package-lock-only --output-reproducible --output-file script-gen-ui.npm.cdx.json

# Rust
[group("sbom")]
[group("rust")]
sbom-rust:
    cargo cyclonedx --format json --all --describe crate -vvv

################################################################################
# PACKAGE ######################################################################
################################################################################

prep-package:
    -rm -r bin
    -mkdir -p bin

# Build must be run first. Not a hard requirement here because we don't want to build
# again in CI
[group("package")]
[group("script-gen-ui")]
[working-directory: 'script-gen-ui']
pre-package-script-gen-ui: prep-package
    cp -r dist/script-gen-ui/* ../bin

# Build must be run first. Not a hard requirement here because we don't want to build
# again in CI
[group("package")]
[group("rust")]
pre-package-rust triple=native-triple:  prep-package
    cp target/{{triple}}/release/kic-* ./bin
    -rm bin/*.pdb
    -rm bin/*.d

