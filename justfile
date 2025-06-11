install:
  cargo build --release
  sudo install -Dm755 target/release/aletheia /usr/bin
  sudo install -Dm644 resources/linux/completions/aletheia.bash /usr/share/bash-completion/completions/aletheia
  sudo install -Dm644 resources/linux/completions/aletheia.fish /usr/share/fish/vendor_completions.d
  sudo install -Dm644 resources/linux/aletheia.desktop /usr/share/applications
  sudo install -Dm644 resources/linux/moe.spencer.Aletheia.metainfo.xml /usr/share/metainfo
