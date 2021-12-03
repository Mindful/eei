cargo build --manifest-path=src/predict/Cargo.toml
cmake -DCMAKE_BUILD_TYPE=release -DLIBEXECDIR=/usr/libexec . -DCMAKE_INSTALL_FULL_DATADIR=/usr/share/ -D RELEASE=0
make
sudo make install
