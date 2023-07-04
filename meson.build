project(
  'gtkwgpuarea',
  'rust',
  version: '0.1.0',
  meson_version: '>= 0.59',
  license: 'MIT',
)

version = meson.project_version()
base_id = 'com.github.flxzt.gtkwgpuarea'

prefix = get_option('prefix')
bindir = prefix / get_option('bindir')

dependency('glib-2.0', version: '>= 2.76')
dependency('gio-2.0', version: '>= 2.76')
dependency('gtk4', version: '>= 4.12')
cargo = find_program('cargo', required: true)

meson.add_dist_script(
  'build-aux/dist-vendor.sh',
  meson.project_build_root() / 'meson-dist' / meson.project_name() + '-' + version,
  meson.project_source_root()
)

cargo_options = [
  '--manifest-path', meson.project_source_root() / 'Cargo.toml',
  '--target-dir', meson.project_build_root() / 'src'
]
cargo_env = [ 'CARGO_HOME=' + meson.project_build_root() / 'cargo-home' ]
rust_target = 'debug'
message('Building in debug mode')

custom_target(
  'cargo-build',
  build_by_default: true,
  build_always_stale: true,
  output: meson.project_name(),
  console: true,
  install: true,
  install_dir: bindir,
  command: [
    'env',
    cargo_env,
    cargo, 'build',
    cargo_options,
    '&&',
    'cp', 'src' / rust_target / meson.project_name(), '@OUTPUT@',
  ]
)
