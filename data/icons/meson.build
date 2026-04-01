icon_sizes = ['256', '128', '64', '48']

foreach size : icon_sizes
  install_data(
    application_id + '-' + size + '.png',
    install_dir: get_option('datadir') / 'icons' / 'hicolor' / size + 'x' + size / 'apps',
    rename: application_id + '.png',
  )
endforeach
