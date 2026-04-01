magick = find_program('magick')
master_icon = files('com.davidedmiston.Larmindon.Source.png')
icon_sizes = ['256', '128', '64', '48']

foreach size : icon_sizes
  custom_target(
    'icon-' + size,
    input: master_icon,
    output: application_id + '-' + size + '.png',
    build_by_default: true,
    command: [
      magick, '@INPUT@', '-resize', size + 'x' + size, '@OUTPUT@',
    ],
  )
endforeach

meson.add_install_script(
  find_program(meson.project_source_root() / 'build-aux' / 'install-icons.sh'),
  meson.current_build_dir(),
  get_option('prefix') / get_option('datadir') / 'icons' / 'hicolor',
  application_id,
  icon_sizes,
)
